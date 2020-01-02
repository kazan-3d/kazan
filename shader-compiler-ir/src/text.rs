// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

//! conversion from/to text

use crate::prelude::*;
use crate::OnceCell;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::Range;
use std::str::FromStr;
use unicode_width::UnicodeWidthChar;

#[derive(Debug)]
pub struct FromTextSourceCode<'a> {
    pub file_name: &'a str,
    pub text: &'a str,
    line_start_byte_indexes: OnceCell<Vec<usize>>,
}

impl<'a> FromTextSourceCode<'a> {
    pub fn new(file_name: &'a str, text: &'a str) -> Self {
        Self {
            file_name,
            text,
            line_start_byte_indexes: OnceCell::new(),
        }
    }
    /// byte indexes of line starts
    /// always starts with 0
    pub fn line_start_byte_indexes(&self) -> &[usize] {
        self.line_start_byte_indexes.get_or_init(|| {
            let mut line_start_byte_indexes = vec![0];
            for (index, byte) in self.text.bytes().enumerate() {
                if byte == b'\n' {
                    // don't need to specifically check for "\r\n" since
                    // line start still is right after '\n'
                    line_start_byte_indexes.push(index + 1);
                }
            }
            line_start_byte_indexes
        })
    }
    /// 0-based line number of the line containing byte_index
    pub fn line_index_of_containing_line(&self, byte_index: usize) -> usize {
        let line_start_byte_indexes = self.line_start_byte_indexes();
        match line_start_byte_indexes.binary_search(&byte_index) {
            Ok(index) => index,
            Err(index) => index - 1,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct FromTextErrorLocation {
    pub file_name: String,
    pub byte_index: usize,
    pub line_number: usize,
    pub column_number: usize,
}

impl fmt::Display for FromTextErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.file_name, self.line_number, self.column_number
        )
    }
}

#[derive(Clone, Debug)]
pub struct FromTextError {
    pub location: FromTextErrorLocation,
    pub message: String,
}

impl fmt::Display for FromTextError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: error: {}", self.location, self.message)
    }
}

impl Error for FromTextError {}

#[derive(Copy, Clone, Debug)]
pub struct TextLocation<'a> {
    byte_index: usize,
    source_code: &'a FromTextSourceCode<'a>,
}

impl PartialEq for TextLocation<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        assert_eq!(
            self.source_code as *const _, rhs.source_code,
            "can only compare TextLocation values within the same source"
        );
        self.byte_index == rhs.byte_index
    }
}

impl Eq for TextLocation<'_> {}

impl Iterator for TextLocation<'_> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        let mut chars = self.source_code.text[self.byte_index..].chars();
        let retval = chars.next()?;
        self.byte_index = self.source_code.text.len() - chars.as_str().len();
        Some(retval)
    }
}

impl<'a> TextLocation<'a> {
    pub fn new(byte_index: usize, source_code: &'a FromTextSourceCode<'a>) -> Self {
        assert!(source_code.text.is_char_boundary(byte_index));
        Self {
            byte_index,
            source_code,
        }
    }
    pub fn peek(&self) -> Option<char> {
        let mut copy = *self;
        copy.next()
    }
    pub fn source_code(&self) -> &'a FromTextSourceCode<'a> {
        self.source_code
    }
    pub fn byte_index(&self) -> usize {
        self.byte_index
    }
    pub fn to_error_location(&self) -> FromTextErrorLocation {
        let file_name = self.source_code.file_name.into();
        let byte_index = self.byte_index();
        let text = self.source_code.text;
        let line_index = self
            .source_code
            .line_index_of_containing_line(self.byte_index);
        let line_start_index = self.source_code.line_start_byte_indexes()[line_index];
        let line_number = line_index + 1;
        const TAB_WIDTH: usize = 4;
        let column_number = 1 + text[line_start_index..byte_index]
            .chars()
            .fold(0, |col, ch| {
                // col is zero-based
                if ch == '\t' {
                    (col + TAB_WIDTH) / TAB_WIDTH * TAB_WIDTH
                } else {
                    col + ch.width().unwrap_or(0)
                }
            });
        FromTextErrorLocation {
            file_name,
            byte_index,
            line_number,
            column_number,
        }
    }
}

impl From<TextLocation<'_>> for FromTextErrorLocation {
    fn from(v: TextLocation) -> FromTextErrorLocation {
        v.to_error_location()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TextSpan<'a> {
    start_byte_index: usize,
    end_byte_index: usize,
    source_code: &'a FromTextSourceCode<'a>,
}

impl Eq for TextSpan<'_> {}

impl PartialEq for TextSpan<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        assert_eq!(
            self.source_code as *const _, rhs.source_code,
            "can only compare TextSpan values within the same source"
        );
        self.byte_indexes() == rhs.byte_indexes()
    }
}

impl<'a> TextSpan<'a> {
    pub fn new(start: TextLocation<'a>, end: TextLocation<'a>) -> Self {
        assert_eq!(
            start.source_code as *const _, end.source_code,
            "TextSpan start and end must be within the same source"
        );
        assert!(
            start.byte_index <= end.byte_index,
            "TextSpan start must not come after end"
        );
        Self {
            start_byte_index: start.byte_index,
            end_byte_index: end.byte_index,
            source_code: start.source_code,
        }
    }
    pub fn byte_indexes(self) -> Range<usize> {
        self.start_byte_index..self.end_byte_index
    }
    pub fn source_code(self) -> &'a FromTextSourceCode<'a> {
        self.source_code
    }
    pub fn text(self) -> &'a str {
        &self.source_code().text[self.byte_indexes()]
    }
    pub fn len(self) -> usize {
        self.end_byte_index - self.start_byte_index
    }
    pub fn start(self) -> TextLocation<'a> {
        TextLocation::new(self.start_byte_index, self.source_code)
    }
    pub fn end(self) -> TextLocation<'a> {
        TextLocation::new(self.end_byte_index, self.source_code)
    }
}

impl From<TextSpan<'_>> for FromTextErrorLocation {
    fn from(span: TextSpan) -> FromTextErrorLocation {
        span.start().into()
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for char {}
}

pub trait FromTextCharExt: Copy + private::Sealed {
    fn is_identifier_start(self) -> bool;
    fn is_identifier_continue(self) -> bool;
}

impl FromTextCharExt for char {
    fn is_identifier_start(self) -> bool {
        self == '_' || self.is_ascii_alphabetic()
    }
    fn is_identifier_continue(self) -> bool {
        self.is_identifier_start() || self.is_ascii_digit()
    }
}

#[derive(Debug)]
pub struct ParseKeywordError;

macro_rules! keywords {
    (
        $(#[doc = $keyword_enum_doc:literal])*
        $keyword_enum:ident,
        $(#[doc = $doc1:expr] $name1:ident = $text1:literal,)*
        $name2:ident = $text2:literal,
        $($(#[doc = $doc3:expr])* $name3:ident = $text3:literal,)*
    ) => {
        keywords! {
            $(#[doc = $keyword_enum_doc])*
            $keyword_enum,
            $(#[doc = $doc1] $name1 = $text1,)*
            #[doc = concat!("The keyword \"", $text2, "\"")]
            $name2 = $text2,
            $($(#[doc = $doc3])* $name3 = $text3,)*
        }
    };
    (
        $(#[doc = $keyword_enum_doc:literal])*
        $keyword_enum:ident,
        $(#[doc = $doc:expr] $name:ident = $text:literal,)+
    ) => {
        $(#[doc = $keyword_enum_doc])*
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        pub enum $keyword_enum {
            $(
                #[doc = $doc]
                $name,
            )+
        }

        impl $keyword_enum {
            /// Get the textual form of `self`
            pub fn text(self) -> &'static str {
                match self {
                    $(
                        $keyword_enum::$name => $text,
                    )+
                }
            }
            pub const VALUES: &'static [$keyword_enum] = &[
                $(
                    $keyword_enum::$name,
                )+
            ];
        }

        impl fmt::Display for $keyword_enum {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad(self.text())
            }
        }

        impl FromStr for $keyword_enum {
            type Err = ParseKeywordError;
            fn from_str(text: &str) -> Result<Self, ParseKeywordError> {
                match text {
                    $(
                        $text => Ok($keyword_enum::$name),
                    )+
                    _ => Err(ParseKeywordError),
                }
            }
        }
    };
}

keywords! {
    /// a keyword
    Keyword,
    I8 = "i8",
    I16 = "i16",
    I32 = "i32",
    I64 = "i64",
    F16 = "f16",
    F32 = "f32",
    F64 = "f64",
    Bool = "bool",
    X = "x",
    VScale = "vscale",
    Undef = "undef",
    True = "true",
    False = "false",
    Const = "const",
    Null = "null",
}

keywords! {
    /// an integer suffix
    IntegerSuffix,
    I8 = "i8",
    I16 = "i16",
    I32 = "i32",
    I64 = "i64",
}

macro_rules! punctuation {
    (
        $(#[doc = $enum_doc:literal])*
        $enum:ident,
        $(#[doc = $doc1:expr] $name1:ident = $text1:literal,)*
        $name2:ident = $text2:literal,
        $($(#[doc = $doc3:expr])* $name3:ident = $text3:literal,)*
    ) => {
        punctuation! {
            $(#[doc = $enum_doc])*
            $enum,
            $(#[doc = $doc1] $name1 = $text1,)*
            #[doc = concat!("The punctuation \"", $text2, "\"")]
            $name2 = $text2,
            $($(#[doc = $doc3])* $name3 = $text3,)*
        }
    };
    (
        $(#[doc = $enum_doc:literal])*
        $enum:ident,
        $(#[doc = $doc:expr] $name:ident = $text:literal,)+
    ) => {
        $(#[doc = $enum_doc])*
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        pub enum $enum {
            $(
                #[doc = $doc]
                $name,
            )+
        }

        impl $enum {
            /// Get the textual form of `self`
            pub fn text(self) -> &'static str {
                match self {
                    $(
                        $enum::$name => $text,
                    )+
                }
            }
            pub const VALUES: &'static [$enum] = &[
                $(
                    $enum::$name,
                )+
            ];
        }

        impl fmt::Display for $enum {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad(self.text())
            }
        }

        impl FromStr for $enum {
            type Err = ParseKeywordError;
            fn from_str(text: &str) -> Result<Self, ParseKeywordError> {
                match text {
                    $(
                        $text => Ok($enum::$name),
                    )+
                    _ => Err(ParseKeywordError),
                }
            }
        }
    };
}

punctuation! {
    /// punctuation
    Punctuation,
    ExMark = "!",
    Dollar = "$",
    Percent = "%",
    Ampersand = "&",
    LParen = "(",
    RParen = ")",
    Asterisk = "*",
    Plus = "+",
    Comma = ",",
    Minus = "-",
    Period = ".",
    Slash = "/",
    Colon = ":",
    Semicolon = ";",
    LessThan = "<",
    Equal = "=",
    GreaterThan = ">",
    QMark = "?",
    At = "@",
    LSquareBracket = "[",
    RSquareBracket = "]",
    Caret = "^",
    Underscore = "_",
    LCurlyBrace = "{",
    VBar = "|",
    RCurlyBrace = "}",
    Tilde = "~",
    Arrow = "->",
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum IdentifierOrKeyword<'t> {
    Identifier(&'t str),
    Keyword(Keyword),
}

impl<'t> From<&'t str> for IdentifierOrKeyword<'t> {
    fn from(text: &'t str) -> IdentifierOrKeyword<'t> {
        match text.parse() {
            Ok(keyword) => IdentifierOrKeyword::Keyword(keyword),
            Err(_) => IdentifierOrKeyword::Identifier(text),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct StringToken<'t> {
    pub source_text: &'t str,
}

#[derive(Copy, Clone, Debug)]
struct ShortEscapeSequence {
    value: char,
}

macro_rules! short_escape_sequences {
    ($($source_char:literal => $value:literal,)+) => {
        impl ShortEscapeSequence {
            fn from_source(source_char: char) -> Option<ShortEscapeSequence> {
                match source_char {
                    $(
                        $source_char => Some(ShortEscapeSequence { value: $value }),
                    )+
                    _ => None,
                }
            }
        }
    };
}

short_escape_sequences! {
    '0' => '\0',
    'n' => '\n',
    'r' => '\r',
    't' => '\t',
    '\'' => '\'',
    '\"' => '\"',
    '\\' => '\\',
}

impl StringToken<'_> {
    pub const QUOTE: char = '\"';
    pub fn parse_escape_sequence(location: &mut TextLocation) -> Result<char, &'static str> {
        if let Some(ShortEscapeSequence { value, .. }) =
            location.peek().and_then(ShortEscapeSequence::from_source)
        {
            location.next();
            return Ok(value);
        }
        match location.next() {
            None => return Err("truncated escape sequence"),
            Some('u') => {}
            _ => return Err("invalid escape sequence"),
        }
        match location.next() {
            None => return Err("truncated escape sequence"),
            Some('{') => {}
            _ => {
                return Err(
                    "invalid escape sequence; unicode escapes must be of the form \\u{1234}",
                )
            }
        }
        let digits_start_location = *location;
        while location.peek().map(|ch| ch.is_ascii_hexdigit()) == Some(true) {
            location.next();
        }
        let digits = TextSpan::new(digits_start_location, *location).text();
        if digits.is_empty() {
            return Err("invalid unicode escape sequence -- no hexadecimal digits");
        }
        match location.next() {
            None => Err("truncated escape sequence"),
            Some('}') => {
                let value = u32::from_str_radix(digits, 0x10)
                    .map_err(|_| "unicode escape value too big")?;
                if value > std::char::MAX as u32 {
                    return Err("unicode escape value too big");
                }
                std::char::from_u32(value).ok_or("invalid unicode escape value")
            }
            _ => Err("invalid escape sequence; unicode escapes must be of the form \\u{1234}"),
        }
    }
    pub fn parse_char(location: &mut TextLocation) -> Result<char, &'static str> {
        match location.next().ok_or("missing character")? {
            '\\' => Self::parse_escape_sequence(location),
            '\n' | '\r' => {
                Err(r#"line-ending not allowed in string, use "\n" and/or "\r" instead"#)
            }
            '\0' => Err(r#"null byte not allowed in string, use "\0" instead"#),
            ch => Ok(ch),
        }
    }
    pub fn value(self) -> String {
        let mut value = String::with_capacity(self.source_text.len());
        let source_code = FromTextSourceCode::new("", self.source_text);
        let mut location = TextLocation::new(0, &source_code);
        while location.peek().is_some() {
            value.push(
                Self::parse_char(&mut location).expect("StringToken should have valid source_text"),
            );
        }
        value
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IntegerToken {
    pub value: u64,
    pub suffix: Option<IntegerSuffix>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum TokenKind<'t> {
    Keyword(Keyword),
    Identifier(&'t str),
    EndOfFile,
    Integer(IntegerToken),
    String(StringToken<'t>),
    Punct(Punctuation),
}

impl<'t> TokenKind<'t> {
    pub fn keyword(self) -> Option<Keyword> {
        if let TokenKind::Keyword(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
    pub fn identifier(self) -> Option<&'t str> {
        if let TokenKind::Identifier(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
    pub fn identifier_or_keyword(self) -> Option<IdentifierOrKeyword<'t>> {
        match self {
            Self::Identifier(v) => Some(IdentifierOrKeyword::Identifier(v)),
            Self::Keyword(v) => Some(IdentifierOrKeyword::Keyword(v)),
            _ => None,
        }
    }
    pub fn raw_identifier(self) -> Option<&'t str> {
        match self {
            Self::Identifier(v) => Some(v),
            Self::Keyword(v) => Some(v.text()),
            _ => None,
        }
    }
    pub fn is_end_of_file(self) -> bool {
        if let TokenKind::EndOfFile = self {
            true
        } else {
            false
        }
    }
    pub fn integer(self) -> Option<IntegerToken> {
        if let TokenKind::Integer(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
    pub fn string(self) -> Option<StringToken<'t>> {
        if let TokenKind::String(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
    pub fn punct(self) -> Option<Punctuation> {
        if let TokenKind::Punct(retval) = self {
            Some(retval)
        } else {
            None
        }
    }
}

impl<'t> From<IdentifierOrKeyword<'t>> for TokenKind<'t> {
    fn from(value: IdentifierOrKeyword<'t>) -> TokenKind<'t> {
        match value {
            IdentifierOrKeyword::Identifier(identifier) => TokenKind::Identifier(identifier),
            IdentifierOrKeyword::Keyword(keyword) => TokenKind::Keyword(keyword),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Token<'t> {
    pub span: TextSpan<'t>,
    pub kind: TokenKind<'t>,
}

pub const COMMENT_START_CHAR: char = '#';

#[derive(Copy, Clone)]
pub enum Void {}

impl Void {
    pub fn into(self) -> ! {
        match self {}
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct FromTextScopeId {
    index: usize,
}

impl FromTextScopeId {
    pub const ROOT: Self = Self { index: 0 };
}

#[derive(Debug)]
pub struct FromTextSymbol<'g, T: Id<'g>> {
    pub value: IdRef<'g, T>,
    pub scope: FromTextScopeId,
}

impl<'g, T: Id<'g>> Clone for FromTextSymbol<'g, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'g, T: Id<'g>> Copy for FromTextSymbol<'g, T> {}

pub trait FromTextSymbolsStateBase<'g, 't>: BorrowMut<FromTextState<'g, 't>> {
    fn get_parent_scope(&self, scope: FromTextScopeId) -> Option<FromTextScopeId> {
        if scope == FromTextScopeId::ROOT {
            None
        } else {
            Some(self.borrow().parent_scopes[scope.index])
        }
    }
    fn allocate_scope(&mut self, parent_scope: FromTextScopeId) -> FromTextScopeId {
        let parent_scopes = &mut self.borrow_mut().parent_scopes;
        let index = parent_scopes.len();
        debug_assert_ne!(index, FromTextScopeId::ROOT.index, "invalid state");
        parent_scopes.push(parent_scope);
        FromTextScopeId { index }
    }
    fn is_scope_visible(&self, search_for_scope: FromTextScopeId) -> bool {
        let mut scope = self.borrow().scope_stack_top;
        loop {
            if scope == search_for_scope {
                break true;
            }
            if let Some(parent_scope) = self.get_parent_scope(scope) {
                scope = parent_scope;
            } else {
                break false;
            }
        }
    }
    fn push_new_nested_scope(&mut self) -> FromTextScopeId {
        let this = self.borrow_mut();
        let scope = this.allocate_scope(this.scope_stack_top);
        this.scope_stack_top = scope;
        scope
    }
}

#[doc(hidden)]
pub struct Private {
    _private: (),
}

impl Private {
    const fn new() -> Self {
        Self { _private: () }
    }
}

impl<'g, 't> FromTextSymbolsStateBase<'g, 't> for FromTextState<'g, 't> {}

pub trait FromTextSymbolsState<'g, 't, T: Id<'g>>: FromTextSymbolsStateBase<'g, 't> {
    #[doc(hidden)]
    fn get_symbol_table(&self, _: Private) -> &HashMap<NamedId<'g>, FromTextSymbol<'g, T>>;
    #[doc(hidden)]
    fn get_symbol_table_mut(
        &mut self,
        _: Private,
    ) -> &mut HashMap<NamedId<'g>, FromTextSymbol<'g, T>>;
    fn get_symbol(&self, name: NamedId<'g>) -> Option<FromTextSymbol<'g, T>> {
        self.get_symbol_table(Private::new()).get(&name).copied()
    }
    fn insert_symbol(
        &mut self,
        name: NamedId<'g>,
        symbol: FromTextSymbol<'g, T>,
    ) -> Result<(), ()> {
        if let Entry::Vacant(entry) = self.get_symbol_table_mut(Private::new()).entry(name) {
            entry.insert(symbol);
            Ok(())
        } else {
            Err(())
        }
    }
}

pub struct FromTextState<'g, 't> {
    global_state: &'g GlobalState<'g>,
    pub location: TextLocation<'t>,
    cached_token: Option<Token<'t>>,
    values: HashMap<NamedId<'g>, FromTextSymbol<'g, Value<'g>>>,
    blocks: HashMap<NamedId<'g>, FromTextSymbol<'g, BlockData<'g>>>,
    loops: HashMap<NamedId<'g>, FromTextSymbol<'g, LoopData<'g>>>,
    parent_scopes: Vec<FromTextScopeId>,
    pub scope_stack_top: FromTextScopeId,
}

impl<'g, 't> FromTextSymbolsState<'g, 't, Value<'g>> for FromTextState<'g, 't> {
    fn get_symbol_table(&self, _: Private) -> &HashMap<NamedId<'g>, FromTextSymbol<'g, Value<'g>>> {
        &self.values
    }
    fn get_symbol_table_mut(
        &mut self,
        _: Private,
    ) -> &mut HashMap<NamedId<'g>, FromTextSymbol<'g, Value<'g>>> {
        &mut self.values
    }
}

impl<'g, 't> FromTextSymbolsState<'g, 't, BlockData<'g>> for FromTextState<'g, 't> {
    fn get_symbol_table(
        &self,
        _: Private,
    ) -> &HashMap<NamedId<'g>, FromTextSymbol<'g, BlockData<'g>>> {
        &self.blocks
    }
    fn get_symbol_table_mut(
        &mut self,
        _: Private,
    ) -> &mut HashMap<NamedId<'g>, FromTextSymbol<'g, BlockData<'g>>> {
        &mut self.blocks
    }
}

impl<'g, 't> FromTextSymbolsState<'g, 't, LoopData<'g>> for FromTextState<'g, 't> {
    fn get_symbol_table(
        &self,
        _: Private,
    ) -> &HashMap<NamedId<'g>, FromTextSymbol<'g, LoopData<'g>>> {
        &self.loops
    }
    fn get_symbol_table_mut(
        &mut self,
        _: Private,
    ) -> &mut HashMap<NamedId<'g>, FromTextSymbol<'g, LoopData<'g>>> {
        &mut self.loops
    }
}

impl<'g, 't> FromTextState<'g, 't> {
    fn new(source_code: &'t FromTextSourceCode<'t>, global_state: &'g GlobalState<'g>) -> Self {
        Self {
            global_state,
            location: TextLocation::new(0, source_code),
            cached_token: None,
            values: HashMap::new(),
            blocks: HashMap::new(),
            loops: HashMap::new(),
            parent_scopes: vec![FromTextScopeId::ROOT],
            scope_stack_top: FromTextScopeId::ROOT,
        }
    }
    pub fn global_state(&self) -> &'g GlobalState<'g> {
        self.global_state
    }
    pub fn error_at<L: Into<FromTextErrorLocation>>(
        &mut self,
        location: L,
        message: impl ToString,
    ) -> Result<Void, FromTextError> {
        Err(FromTextError {
            location: location.into(),
            message: message.to_string(),
        })
    }
    fn peek_char(&self) -> Option<char> {
        self.location.peek()
    }
    fn next_char(&mut self) -> Option<char> {
        self.location.next()
    }
    pub fn error_at_peek_token(&mut self, message: impl ToString) -> Result<Void, FromTextError> {
        let span = self.peek_token()?.span;
        self.error_at(span, message.to_string())
    }
    fn error_at_peek_char(&mut self, message: impl ToString) -> Result<Void, FromTextError> {
        self.error_at(self.location, message.to_string())
    }
    fn parse_comment(&mut self) -> Result<(), FromTextError> {
        if self.peek_char() != Some(COMMENT_START_CHAR) {
            self.error_at_peek_char("missing comment")?;
        }
        loop {
            match self.next_char() {
                None | Some('\n') => break,
                _ => {}
            }
        }
        Ok(())
    }
    fn skip_whitespace(&mut self) -> Result<(), FromTextError> {
        loop {
            match self.peek_char() {
                Some(COMMENT_START_CHAR) => self.parse_comment()?,
                Some(ch) => {
                    if !ch.is_ascii_whitespace() {
                        break;
                    }
                }
                None => break,
            }
            self.next_char();
        }
        Ok(())
    }
    fn parse_raw_identifier(&mut self) -> Result<&'t str, FromTextError> {
        let start_location = self.location;
        if self.peek_char().map(char::is_identifier_start) != Some(true) {
            self.error_at_peek_char("missing identifier")?;
        }
        while self.peek_char().map(char::is_identifier_continue) == Some(true) {
            self.next_char();
        }
        Ok(TextSpan::new(start_location, self.location).text())
    }
    fn parse_identifier_or_keyword(&mut self) -> Result<IdentifierOrKeyword<'t>, FromTextError> {
        self.parse_raw_identifier().map(Into::into)
    }
    fn parse_optional_integer_suffix(&mut self) -> Result<Option<IntegerSuffix>, FromTextError> {
        let start_location = self.location;
        if self.peek_char().map(char::is_identifier_start) != Some(true) {
            return Ok(None);
        }
        while self.peek_char().map(char::is_identifier_continue) == Some(true) {
            self.next_char();
        }
        let span = TextSpan::new(start_location, self.location);
        match span.text().parse::<IntegerSuffix>() {
            Ok(retval) => Ok(Some(retval)),
            Err(_) => self.error_at(span, "invalid integer suffix")?.into(),
        }
    }
    fn parse_integer(&mut self) -> Result<IntegerToken, FromTextError> {
        if self.peek_char().map(|ch| ch.is_ascii_digit()) != Some(true) {
            self.error_at_peek_char("expected number")?;
        }
        let mut digits_start_location = self.location;
        let radix;
        if self.peek_char() == Some('0') {
            self.next_char();
            match self.peek_char() {
                Some('x') | Some('X') => {
                    self.next_char();
                    digits_start_location = self.location;
                    radix = 16;
                }
                Some('o') | Some('O') => {
                    self.next_char();
                    digits_start_location = self.location;
                    radix = 8;
                }
                Some('b') | Some('B') => {
                    self.next_char();
                    digits_start_location = self.location;
                    radix = 2;
                }
                Some(ch) if ch.is_ascii_digit() => self
                    .error_at_peek_char("octal numbers must start with 0o or 0O")?
                    .into(),
                _ => {
                    return Ok(IntegerToken {
                        value: 0,
                        suffix: self.parse_optional_integer_suffix()?,
                    })
                }
            }
        } else {
            radix = 10;
        }
        while self
            .location
            .peek()
            .and_then(|ch| ch.to_digit(radix))
            .is_some()
        {
            self.next_char();
        }
        let digits = TextSpan::new(digits_start_location, self.location).text();
        let suffix = self.parse_optional_integer_suffix()?;
        match u64::from_str_radix(digits, radix) {
            Ok(value) => Ok(IntegerToken { value, suffix }),
            _ => self
                .error_at(digits_start_location, "number too big")?
                .into(),
        }
    }
    fn parse_string(&mut self) -> Result<StringToken<'t>, FromTextError> {
        if self.peek_char() != Some(StringToken::QUOTE) {
            self.error_at_peek_char("missing string")?;
        }
        let quote_location = self.location;
        self.next_char();
        let string_body_start_location = self.location;
        loop {
            match self.peek_char() {
                None => self.error_at(quote_location, "unterminated string")?.into(),
                Some(StringToken::QUOTE) => {
                    let string_body_end_location = self.location;
                    self.next_char();
                    return Ok(StringToken {
                        source_text: TextSpan::new(
                            string_body_start_location,
                            string_body_end_location,
                        )
                        .text(),
                    });
                }
                _ => match StringToken::parse_char(&mut self.location) {
                    Ok(_) => {}
                    Err(message) => self.error_at_peek_char(message)?.into(),
                },
            }
        }
    }
    fn parse_punct(&mut self) -> Result<Punctuation, FromTextError> {
        if self.peek_char().is_none() {
            self.error_at_peek_char("missing punctuation")?;
        }
        let start_location = self.location;
        let mut matched = None;
        while self.next_char().is_some() {
            let peek_text = TextSpan::new(start_location, self.location).text();
            let mut is_prefix = false;
            for &punct in Punctuation::VALUES {
                let punct_text = punct.text();
                if peek_text == punct_text {
                    matched = Some((punct, self.location));
                } else if punct_text.starts_with(peek_text) {
                    is_prefix = true;
                }
            }
            if !is_prefix {
                break;
            }
        }
        if let Some((retval, end_location)) = matched {
            self.location = end_location;
            Ok(retval)
        } else {
            self.location = start_location;
            self.error_at_peek_char("invalid punctuation")?.into()
        }
    }
    fn parse_token_impl(&mut self) -> Result<Token<'t>, FromTextError> {
        self.skip_whitespace()?;
        let start_location = self.location;
        match self.peek_char() {
            None => Ok(Token {
                kind: TokenKind::EndOfFile,
                span: TextSpan::new(start_location, self.location),
            }),
            Some(StringToken::QUOTE) => Ok(Token {
                kind: TokenKind::String(self.parse_string()?),
                span: TextSpan::new(start_location, self.location),
            }),
            Some(ch) if ch.is_identifier_start() => Ok(Token {
                kind: self.parse_identifier_or_keyword()?.into(),
                span: TextSpan::new(start_location, self.location),
            }),
            Some(ch) if ch.is_ascii_digit() => Ok(Token {
                kind: TokenKind::Integer(self.parse_integer()?),
                span: TextSpan::new(start_location, self.location),
            }),
            _ => Ok(Token {
                kind: TokenKind::Punct(self.parse_punct()?),
                span: TextSpan::new(start_location, self.location),
            }),
        }
    }
    pub fn peek_token(&mut self) -> Result<Token<'t>, FromTextError> {
        if let Some(cached_token) = self.cached_token {
            if cached_token.span.start() == self.location {
                return Ok(cached_token);
            }
        }
        let token = self.parse_token_impl()?;
        self.location = token.span.start();
        self.cached_token = Some(token);
        Ok(token)
    }
    pub fn parse_token(&mut self) -> Result<Token<'t>, FromTextError> {
        if let Some(cached_token) = self.cached_token.take() {
            if cached_token.span.start() == self.location {
                self.location = cached_token.span.end();
                return Ok(cached_token);
            }
        }
        self.parse_token_impl()
    }
    pub fn parse_punct_token_or_error(
        &mut self,
        punct: Punctuation,
        error_msg: impl ToString,
    ) -> Result<Token<'t>, FromTextError> {
        let token = self.parse_token()?;
        if token.kind.punct() != Some(punct) {
            self.error_at(token.span, error_msg)?;
        }
        Ok(token)
    }
    pub fn parse_keyword_token_or_error(
        &mut self,
        keyword: Keyword,
        error_msg: impl ToString,
    ) -> Result<Token<'t>, FromTextError> {
        let token = self.parse_token()?;
        if token.kind.keyword() != Some(keyword) {
            self.error_at(token.span, error_msg)?;
        }
        Ok(token)
    }
    pub fn parse_parenthesized<T, F: FnOnce(&mut Self) -> Result<T, FromTextError>>(
        &mut self,
        open_paren: Punctuation,
        missing_open_paren_error_msg: impl ToString,
        close_paren: Punctuation,
        missing_close_paren_error_msg: impl ToString,
        body: F,
    ) -> Result<T, FromTextError> {
        self.parse_punct_token_or_error(open_paren, missing_open_paren_error_msg)?;
        let retval = body(self)?;
        self.parse_punct_token_or_error(close_paren, missing_close_paren_error_msg)?;
        Ok(retval)
    }
}

/// parse text
pub trait FromText<'g> {
    /// the type produced by parsing text successfully
    type Parsed: Sized;
    /// top-level parse function -- should not be called from `from_text` implementations
    fn parse(
        file_name: impl Borrow<str>,
        text: impl Borrow<str>,
        global_state: &'g GlobalState<'g>,
    ) -> Result<Self::Parsed, FromTextError> {
        let file_name = file_name.borrow();
        let text = text.borrow();
        let source_code = FromTextSourceCode::new(file_name, text);
        let mut state = FromTextState::new(&source_code, global_state);
        let retval = Self::from_text(&mut state)?;
        if !state.peek_token()?.kind.is_end_of_file() {
            state.error_at_peek_token("extra tokens at end")?;
        }
        Ok(retval)
    }
    /// do the actual parsing work
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError>;
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct NamedId<'g> {
    pub name: Interned<'g, str>,
    pub name_suffix: u64,
}

impl<'g> NamedId<'g> {
    pub fn needs_quoted_form(self) -> bool {
        let NamedId { name, name_suffix } = self;
        if name_suffix != 0 {
            true
        } else {
            let mut chars = name.chars();
            if let Some(first) = chars.next() {
                if !first.is_identifier_start() {
                    true
                } else {
                    !chars.all(|ch| ch.is_identifier_continue())
                }
            } else {
                true
            }
        }
    }
}

impl<'g> FromText<'g> for NamedId<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        match state.peek_token()?.kind {
            TokenKind::Identifier(name) => {
                state.parse_token()?;
                Ok(Self {
                    name: state.global_state().intern(name),
                    name_suffix: 0,
                })
            }
            TokenKind::Keyword(name) => {
                state.parse_token()?;
                Ok(Self {
                    name: state.global_state().intern(name.text()),
                    name_suffix: 0,
                })
            }
            TokenKind::String(name) => {
                state.parse_token()?;
                let name = state.global_state().intern(&*name.value());
                if let Some(IntegerToken { value, suffix }) = state.peek_token()?.kind.integer() {
                    if suffix.is_some() {
                        state.error_at_peek_token(r#"name suffix must be unsuffixed integer ("my_name"123 and not "my_name"123i8)"#)?;
                    }
                    state.parse_token()?;
                    Ok(Self {
                        name: state.global_state().intern(&name),
                        name_suffix: value,
                    })
                } else {
                    state.error_at_peek_token("missing name suffix")?.into()
                }
            }
            _ => state
                .error_at_peek_token("missing name -- must be identifier or string")?
                .into(),
        }
    }
}

impl<'g> ToText<'g> for NamedId<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        if self.needs_quoted_form() {
            self.name.to_text(state)?;
            write!(state, "{}", self.name_suffix)
        } else {
            write!(state, "{}", self.name)
        }
    }
}

impl<'g> ToText<'g> for str {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "\"{}\"", self.escape_default())
    }
}

trait NameMapGetName<'g>: Id<'g> {
    fn name(&self) -> Interned<'g, str>;
}

impl<'g> NameMapGetName<'g> for Value<'g> {
    fn name(&self) -> Interned<'g, str> {
        self.name
    }
}

impl<'g> NameMapGetName<'g> for BlockData<'g> {
    fn name(&self) -> Interned<'g, str> {
        self.name
    }
}

impl<'g> NameMapGetName<'g> for LoopData<'g> {
    fn name(&self) -> Interned<'g, str> {
        self.name
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum NewOrOld<T> {
    New(T),
    Old(T),
}

struct NameMap<'g, T: NameMapGetName<'g>> {
    named_ids: HashMap<IdRef<'g, T>, NamedId<'g>>,
    name_suffixes: HashMap<Interned<'g, str>, u64>,
}

impl<'g, T: NameMapGetName<'g>> NameMap<'g, T> {
    fn new() -> Self {
        Self {
            named_ids: HashMap::new(),
            name_suffixes: HashMap::new(),
        }
    }
    fn get(&mut self, value: IdRef<'g, T>) -> NewOrOld<NamedId<'g>> {
        match self.named_ids.entry(value) {
            Entry::Occupied(entry) => NewOrOld::Old(*entry.get()),
            Entry::Vacant(entry) => {
                let name = value.name();
                let next_name_suffix = self.name_suffixes.entry(name).or_insert(0);
                let name_suffix = *next_name_suffix;
                *next_name_suffix += 1;
                NewOrOld::New(*entry.insert(NamedId { name, name_suffix }))
            }
        }
    }
}

pub struct ToTextState<'g, 'w> {
    indent: usize,
    at_start_of_line: bool,
    base_writer: &'w mut dyn FnMut(&str) -> fmt::Result,
    values: NameMap<'g, Value<'g>>,
    blocks: NameMap<'g, BlockData<'g>>,
    loops: NameMap<'g, LoopData<'g>>,
}

impl<'g> ToTextState<'g, '_> {
    pub(crate) fn get_value_named_id(
        &mut self,
        value: IdRef<'g, Value<'g>>,
    ) -> NewOrOld<NamedId<'g>> {
        self.values.get(value)
    }
    pub(crate) fn get_block_named_id(
        &mut self,
        value: IdRef<'g, BlockData<'g>>,
    ) -> NewOrOld<NamedId<'g>> {
        self.blocks.get(value)
    }
    pub(crate) fn get_loop_named_id(
        &mut self,
        value: IdRef<'g, LoopData<'g>>,
    ) -> NewOrOld<NamedId<'g>> {
        self.loops.get(value)
    }
    pub fn indent<R, E, F: FnOnce(&mut Self) -> Result<R, E>>(&mut self, f: F) -> Result<R, E> {
        assert!(
            self.at_start_of_line,
            "can't call indent() in the middle of a text line"
        );
        self.indent += 1;
        let retval = f(self)?;
        assert!(
            self.at_start_of_line,
            "can't return Ok to indent() in the middle of a text line"
        );
        self.indent -= 1;
        Ok(retval)
    }
    /// rebind `std::fmt::Write::write_fmt` to make it easily visible for use with the `write!` macro
    #[inline]
    pub fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        fmt::Write::write_fmt(self, args)
    }
}

impl fmt::Write for ToTextState<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let mut first = true;
        for text in text.split('\n') {
            if !mem::replace(&mut first, false) {
                (self.base_writer)("\n")?;
                self.at_start_of_line = true;
            }
            if text.is_empty() {
                continue;
            }
            let do_indent = mem::replace(&mut self.at_start_of_line, false);
            if do_indent && self.indent != 0 {
                // 256 spaces arranged in a 16x16 grid
                const SPACES: &str = concat!(
                    "                ",
                    "                ",
                    "                ",
                    "                ",
                    //
                    "                ",
                    "                ",
                    "                ",
                    "                ",
                    //
                    "                ",
                    "                ",
                    "                ",
                    "                ",
                    //
                    "                ",
                    "                ",
                    "                ",
                    "                ",
                );
                const INDENT_MULTIPLE: usize = 4;

                // write in larger chunks to speed-up output

                let mut indent = self.indent * INDENT_MULTIPLE;
                while indent >= SPACES.len() {
                    (self.base_writer)(SPACES)?;
                    indent -= SPACES.len();
                }
                (self.base_writer)(&SPACES[..indent])?;
            }
            (self.base_writer)(text)?;
        }
        Ok(())
    }
}

pub trait ToText<'g> {
    fn display(&self) -> ToTextDisplay<'g, '_, Self> {
        ToTextDisplay(self, PhantomData)
    }
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result;
}

pub struct ToTextDisplay<'g, 'a, T: ToText<'g> + ?Sized>(&'a T, PhantomData<&'g ()>);

impl<'g, T: ToText<'g> + ?Sized> fmt::Display for ToTextDisplay<'g, '_, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.to_text(&mut ToTextState {
            indent: 0,
            at_start_of_line: true,
            base_writer: &mut |text: &str| formatter.write_str(text),
            values: NameMap::new(),
            blocks: NameMap::new(),
            loops: NameMap::new(),
        })
    }
}

impl<'g, T: ToText<'g> + ?Sized> fmt::Debug for ToTextDisplay<'g, '_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<'g, T: ToText<'g> + ?Sized> ToText<'g> for &'_ T {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        (**self).to_text(state)
    }
}

impl<'g, T: ToText<'g>> ToText<'g> for [T] {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let mut iter = self.iter();
        write!(state, "[")?;
        if let Some(first) = iter.next() {
            first.to_text(state)?;
            for element in iter {
                write!(state, ", ")?;
                element.to_text(state)?;
            }
        }
        write!(state, "]")
    }
}

impl<'g, T: ToText<'g>> ToText<'g> for Vec<T> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        (**self).to_text(state)
    }
}

fn list_from_text_helper<'g, 't, T: FromText<'g>>(
    state: &mut FromTextState<'g, 't>,
    output: Option<&mut [Option<T::Parsed>]>,
) -> Result<Vec<T::Parsed>, FromTextError> {
    let expected_len = output.as_ref().map(|v| v.len());
    fn too_short_msg(expected_len: usize, actual_len: usize) -> String {
        format!(
            "list is too short, expected {} items, got {}",
            expected_len, actual_len
        )
    }
    let missing_close = "list missing closing square bracket: ']'";
    state.parse_parenthesized(
        Punctuation::LSquareBracket,
        "missing list: must start with '['",
        Punctuation::RSquareBracket,
        missing_close,
        |state| {
            let mut retval = Vec::new();
            let mut output = output.map(IntoIterator::into_iter);
            let mut actual_len = 0;
            let mut write_output = |v: T::Parsed| {
                if let Some(output_element) = output.as_mut().and_then(Iterator::next) {
                    *output_element = Some(v);
                } else {
                    retval.push(v);
                }
            };
            match state.peek_token()?.kind {
                TokenKind::EndOfFile => state.error_at_peek_token(missing_close)?.into(),
                TokenKind::Punct(Punctuation::RSquareBracket) => {
                    match expected_len {
                        Some(0) | None => {}
                        Some(expected_len) => state
                            .error_at_peek_token(too_short_msg(expected_len, 0))?
                            .into(),
                    }
                    return Ok(retval);
                }
                _ => {}
            }
            let mut too_long_location = None;
            let mut check_len = |state: &mut FromTextState<'g, 't>,
                                 actual_len: usize|
             -> Result<(), FromTextError> {
                if expected_len == Some(actual_len) {
                    too_long_location = Some(state.peek_token()?.span);
                }
                Ok(())
            };
            check_len(state, actual_len)?;
            write_output(T::from_text(state)?);
            actual_len += 1;
            while state.peek_token()?.kind.punct() == Some(Punctuation::Comma) {
                state.parse_token()?;
                if state.peek_token()?.kind.punct() == Some(Punctuation::RSquareBracket) {
                    break;
                }
                check_len(state, actual_len)?;
                write_output(T::from_text(state)?);
                actual_len += 1;
            }
            if let Some(too_long_location) = too_long_location {
                state
                    .error_at(
                        too_long_location,
                        format!(
                            "list too long, expected {} items, got {}",
                            expected_len.unwrap_or_default(),
                            actual_len
                        ),
                    )?
                    .into()
            } else {
                match expected_len {
                    Some(expected_len) if expected_len != actual_len => state
                        .error_at_peek_token(too_short_msg(expected_len, actual_len))?
                        .into(),
                    _ => Ok(retval),
                }
            }
        },
    )
}

impl<'g, T: FromText<'g>> FromText<'g> for [T] {
    type Parsed = Vec<T::Parsed>;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Vec<T::Parsed>, FromTextError> {
        list_from_text_helper::<T>(state, None)
    }
}

impl<'g, T: FromText<'g>> FromText<'g> for Vec<T> {
    type Parsed = Vec<T::Parsed>;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Vec<T::Parsed>, FromTextError> {
        list_from_text_helper::<T>(state, None)
    }
}

impl<'g, T: ToText<'g>> ToText<'g> for [T; 0] {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        (self as &[T]).to_text(state)
    }
}

impl<'g, T: FromText<'g>> FromText<'g> for [T; 0] {
    type Parsed = [T::Parsed; 0];
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<[T::Parsed; 0], FromTextError> {
        list_from_text_helper::<T>(state, Some(&mut []))?;
        Ok([])
    }
}

macro_rules! impl_from_to_text_for_arrays {
    ($n:literal, [$($element:ident,)*]) => {
        impl<'g, T: ToText<'g>> ToText<'g> for [T; $n] {
            fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                (self as &[T]).to_text(state)
            }
        }

        impl<'g, T: FromText<'g>> FromText<'g> for [T; $n] {
            type Parsed = [T::Parsed; $n];
            fn from_text(
                state: &mut FromTextState<'g, '_>,
            ) -> Result<[T::Parsed; $n], FromTextError> {
                let mut elements: [Option<T::Parsed>; $n] = Default::default();
                list_from_text_helper::<T>(state, Some(&mut elements))?;
                match elements {
                    [$(Some($element)),*] => Ok([$($element),*]),
                    _ => unreachable!(),
                }
            }
        }
    };
}

impl_from_to_text_for_arrays!(1, [e1,]);
impl_from_to_text_for_arrays!(2, [e1, e2,]);
impl_from_to_text_for_arrays!(3, [e1, e2, e3,]);
impl_from_to_text_for_arrays!(4, [e1, e2, e3, e4,]);
impl_from_to_text_for_arrays!(5, [e1, e2, e3, e4, e5,]);
impl_from_to_text_for_arrays!(6, [e1, e2, e3, e4, e5, e6,]);
impl_from_to_text_for_arrays!(7, [e1, e2, e3, e4, e5, e6, e7,]);
impl_from_to_text_for_arrays!(8, [e1, e2, e3, e4, e5, e6, e7, e8,]);
impl_from_to_text_for_arrays!(9, [e1, e2, e3, e4, e5, e6, e7, e8, e9,]);
impl_from_to_text_for_arrays!(10, [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10,]);
impl_from_to_text_for_arrays!(11, [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11,]);
impl_from_to_text_for_arrays!(12, [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12,]);
impl_from_to_text_for_arrays!(
    13,
    [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13,]
);
impl_from_to_text_for_arrays!(
    14,
    [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14,]
);
impl_from_to_text_for_arrays!(
    15,
    [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15,]
);
impl_from_to_text_for_arrays!(
    16,
    [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16,]
);
impl_from_to_text_for_arrays!(
    17,
    [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17,]
);
impl_from_to_text_for_arrays!(
    18,
    [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18,]
);
impl_from_to_text_for_arrays!(
    19,
    [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19,]
);
impl_from_to_text_for_arrays!(
    20,
    [e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,]
);
impl_from_to_text_for_arrays!(
    21,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21,
    ]
);
impl_from_to_text_for_arrays!(
    22,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22,
    ]
);
impl_from_to_text_for_arrays!(
    23,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23,
    ]
);
impl_from_to_text_for_arrays!(
    24,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23, e24,
    ]
);
impl_from_to_text_for_arrays!(
    25,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23, e24, e25,
    ]
);
impl_from_to_text_for_arrays!(
    26,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23, e24, e25, e26,
    ]
);
impl_from_to_text_for_arrays!(
    27,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23, e24, e25, e26, e27,
    ]
);
impl_from_to_text_for_arrays!(
    28,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23, e24, e25, e26, e27, e28,
    ]
);
impl_from_to_text_for_arrays!(
    29,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23, e24, e25, e26, e27, e28, e29,
    ]
);
impl_from_to_text_for_arrays!(
    30,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23, e24, e25, e26, e27, e28, e29, e30,
    ]
);
impl_from_to_text_for_arrays!(
    31,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23, e24, e25, e26, e27, e28, e29, e30, e31,
    ]
);
impl_from_to_text_for_arrays!(
    32,
    [
        e1, e2, e3, e4, e5, e6, e7, e8, e9, e10, e11, e12, e13, e14, e15, e16, e17, e18, e19, e20,
        e21, e22, e23, e24, e25, e26, e27, e28, e29, e30, e31, e32,
    ]
);
