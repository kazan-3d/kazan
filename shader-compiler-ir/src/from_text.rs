// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use once_cell::unsync::OnceCell;
use std::borrow::Borrow;
use std::error::Error;
use std::fmt;
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
}

keywords! {
    /// an integer suffix
    IntegerSuffix,
    I8 = "i8",
    I16 = "i16",
    I32 = "i32",
    I64 = "i64",
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
    source_char: char,
    value: char,
}

macro_rules! short_escape_sequences {
    ($($source_char:literal => $value:literal,)+) => {
        impl ShortEscapeSequence {
            fn from_value(value: char) -> Option<ShortEscapeSequence> {
                match value {
                    $(
                        $value => Some(ShortEscapeSequence { source_char: $source_char, value }),
                    )+
                    _ => None
                }
            }
            fn from_source(source_char: char) -> Option<ShortEscapeSequence> {
                match source_char {
                    $(
                        $source_char => Some(ShortEscapeSequence { source_char, value: $value }),
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
    Punct(char),
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
    pub fn punct(self) -> Option<char> {
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

pub struct FromTextState<'g, 't> {
    global_state: &'g GlobalState<'g>,
    pub location: TextLocation<'t>,
    cached_token: Option<Token<'t>>,
}

pub const COMMENT_START_CHAR: char = '#';

#[derive(Copy, Clone)]
pub enum Void {}

impl Void {
    pub fn into(self) -> ! {
        match self {}
    }
}

impl<'g, 't> FromTextState<'g, 't> {
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
    fn parse_punct(&mut self) -> Result<char, FromTextError> {
        let ch = match self.peek_char() {
            Some(ch) => ch,
            None => self.error_at_peek_char("missing punctuation")?.into(),
        };
        match ch {
            '(' | ')' | '{' | '}' | ',' | '=' | '[' | ']' | '<' | '>' | ';' | '*' | '+' | '-' => {
                self.next_char();
                Ok(ch)
            }
            _ => self.error_at_peek_char("invalid token")?.into(),
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
        punct: char,
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
        open_paren: char,
        missing_open_paren_error_msg: impl ToString,
        close_paren: char,
        missing_close_paren_error_msg: impl ToString,
        body: F,
    ) -> Result<T, FromTextError> {
        self.parse_punct_token_or_error(open_paren, missing_open_paren_error_msg)?;
        let retval = body(self)?;
        self.parse_punct_token_or_error(close_paren, missing_close_paren_error_msg)?;
        Ok(retval)
    }
}

pub trait FromText<'g>: Sized {
    fn parse(
        file_name: impl Borrow<str>,
        text: impl Borrow<str>,
        global_state: &'g GlobalState<'g>,
    ) -> Result<Self, FromTextError> {
        let file_name = file_name.borrow();
        let text = text.borrow();
        let source_code = FromTextSourceCode::new(file_name, text);
        let mut state = FromTextState {
            global_state,
            location: TextLocation::new(0, &source_code),
            cached_token: None,
        };
        let retval = Self::from_text(&mut state)?;
        if !state.peek_token()?.kind.is_end_of_file() {
            state.error_at_peek_token("extra tokens at end")?;
        }
        Ok(retval)
    }
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError>;
}
