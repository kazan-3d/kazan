// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::ConstInteger;
use crate::IntegerType;
use std::borrow::Borrow;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use unicode_width::UnicodeWidthChar;

#[derive(Copy, Clone, Debug)]
pub struct FromTextSourceCode<'a> {
    pub file_name: &'a str,
    pub text: &'a str,
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
    pub fn get_text_to(&self, end: TextLocation<'a>) -> &'a str {
        assert_eq!(self.source_code() as *const _, end.source_code());
        &self.source_code().text[self.byte_index()..end.byte_index()]
    }
    pub fn to_location(&self) -> FromTextErrorLocation {
        let file_name = self.source_code.file_name.into();
        let byte_index = self.byte_index();
        let text = self.source_code.text;
        let mut line_number = 1;
        let mut line_start_index = 0;
        const NEWLINE: char = '\n';
        while let Some(next_newline) = text[line_start_index..].find(NEWLINE) {
            let next_line_start_index = line_start_index + next_newline + NEWLINE.len_utf8();
            if byte_index < next_line_start_index {
                break;
            }
            line_number += 1;
            line_start_index = next_line_start_index;
        }
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
        v.to_location()
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

macro_rules! keywords {
    (
        $(#[doc = $doc1:expr] $name1:ident = $text1:literal,)*
        $name2:ident = $text2:literal,
        $($(#[doc = $doc3:expr])* $name3:ident = $text3:literal,)*
    ) => {
        keywords! {
            $(#[doc = $doc1] $name1 = $text1,)*
            #[doc = concat!("The keyword \"", $text2, "\"")]
            $name2 = $text2,
            $($(#[doc = $doc3])* $name3 = $text3,)*
        }
    };
    (
        $(#[doc = $doc:expr] $name:ident = $text:literal,)+
    ) => {
        /// a keyword
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        pub enum Keyword {
            $(
                #[doc = $doc]
                $name,
            )+
        }

        impl Keyword {
            /// Get the textual form of `self`
            pub fn text(self) -> &'static str {
                match self {
                    $(
                        Keyword::$name => $text,
                    )+
                }
            }
        }

        impl fmt::Display for Keyword {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad(self.text())
            }
        }

        #[derive(Debug)]
        pub struct ParseKeywordError;

        impl FromStr for Keyword {
            type Err = ParseKeywordError;
            fn from_str(text: &str) -> Result<Self, ParseKeywordError> {
                match text {
                    $(
                        $text => Ok(Keyword::$name),
                    )+
                    _ => Err(ParseKeywordError),
                }
            }
        }
    };
}

keywords! {
    U8 = "u8",
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
        let digits = digits_start_location.get_text_to(*location);
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
        let source_code = FromTextSourceCode {
            file_name: "",
            text: self.source_text,
        };
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
pub enum TokenKind<'t> {
    Keyword(Keyword),
    Identifier(&'t str),
    EndOfFile,
    Integer(ConstInteger),
    String(StringToken<'t>),
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
    pub location: TextLocation<'t>,
    pub kind: TokenKind<'t>,
}

#[derive(Copy, Clone, Debug)]
struct CachedToken<'t> {
    start_location: TextLocation<'t>,
    token: Token<'t>,
    next_location: TextLocation<'t>,
}

pub struct FromTextState<'g, 't> {
    global_state: &'g GlobalState<'g>,
    pub location: TextLocation<'t>,
    cached_token: Option<CachedToken<'t>>,
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
    pub fn error(&mut self, message: impl ToString) -> Result<Void, FromTextError> {
        self.error_at(self.location.clone(), message.to_string())
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
    pub fn parse_comment(&mut self) -> Result<(), FromTextError> {
        if self.location.peek() != Some(COMMENT_START_CHAR) {
            self.error("missing comment")?;
        }
        loop {
            match self.location.next() {
                None | Some('\n') => break,
                _ => {}
            }
        }
        Ok(())
    }
    pub fn skip_whitespace(&mut self) -> Result<(), FromTextError> {
        loop {
            match self.location.peek() {
                Some(COMMENT_START_CHAR) => self.parse_comment()?,
                Some(ch) => {
                    if !ch.is_ascii_whitespace() {
                        break;
                    }
                }
                None => break,
            }
            self.location.next();
        }
        Ok(())
    }
    pub fn parse_raw_identifier(&mut self) -> Result<&'t str, FromTextError> {
        let start_location = self.location;
        if self.location.peek().map(char::is_identifier_start) == Some(true) {
            self.error("missing identifier")?;
        }
        while self.location.peek().map(char::is_identifier_continue) == Some(true) {
            self.location.next();
        }
        Ok(start_location.get_text_to(self.location))
    }
    pub fn parse_identifier_or_keyword(
        &mut self,
    ) -> Result<IdentifierOrKeyword<'t>, FromTextError> {
        self.parse_raw_identifier().map(Into::into)
    }
    pub fn parse_integer_suffix(&mut self) -> Result<IntegerType, FromTextError> {
        let suffix_location = self.location;
        let error = |this: &mut Self| {
            this.error_at(
                suffix_location,
                "invalid integer suffix: must be one of i8, i16, i32, or i64",
            )
        };
        if self.location.peek() != Some('i') {
            error(self)?;
        }
        self.location.next();
        let mut value = 0;
        let mut any_digits = false;
        if self.location.peek() == Some('0') {
            error(self)?;
        }
        while let Some(digit_value) = self.location.peek().and_then(|ch| ch.to_digit(10)) {
            any_digits = true;
            if value >= 1000 {
                error(self)?;
            }
            self.location.next();
            value = value * 10 + digit_value;
        }
        if !any_digits {
            error(self)?;
        }
        Ok(match value {
            8 => IntegerType::Int8,
            16 => IntegerType::Int16,
            32 => IntegerType::Int32,
            64 => IntegerType::Int64,
            _ => error(self)?.into(),
        })
    }
    pub fn parse_integer(&mut self) -> Result<ConstInteger, FromTextError> {
        let start_location = self.location;
        #[derive(Copy, Clone)]
        enum Sign {
            Unsigned,
            Negative,
            Positive,
        }
        let sign = match self.location.peek() {
            Some('-') => {
                self.location.next();
                Sign::Negative
            }
            Some('+') => {
                self.location.next();
                Sign::Positive
            }
            _ => Sign::Unsigned,
        };
        if self.location.peek().map(|ch| ch.is_ascii_digit()) != Some(true) {
            self.error("expected number")?;
        }
        let mut digits_start_location = self.location;
        let radix;
        if self.location.peek() == Some('0') {
            self.location.next();
            match self.location.peek() {
                Some('x') | Some('X') => {
                    self.location.next();
                    digits_start_location = self.location;
                    radix = 16;
                }
                Some('o') | Some('O') => {
                    self.location.next();
                    digits_start_location = self.location;
                    radix = 8;
                }
                Some('b') | Some('B') => {
                    self.location.next();
                    digits_start_location = self.location;
                    radix = 2;
                }
                Some(ch) if ch.is_ascii_digit() => {
                    self.error("octal numbers must start with 0o or 0O")?.into()
                }
                _ => {
                    return Ok(ConstInteger {
                        value: 0,
                        integer_type: self.parse_integer_suffix()?,
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
            self.location.next();
        }
        let digits = digits_start_location.get_text_to(self.location);
        let integer_type = self.parse_integer_suffix()?;
        macro_rules! get_limit {
            ($integer_type:expr, $sign:expr, $(($enumerant:ident, $unsigned_type:ident, $signed_type:ident),)+) => {
                match ($integer_type, $sign) {
                    $(
                        (IntegerType::$enumerant, Sign::Negative) => -($signed_type::min_value() as i128),
                        (IntegerType::$enumerant, Sign::Positive) => $signed_type::max_value() as i128,
                        (IntegerType::$enumerant, Sign::Unsigned) => $unsigned_type::max_value() as i128,
                    )+
                }
            };
        }
        let limit = get_limit!(
            integer_type,
            sign,
            (Int8, u8, i8),
            (Int16, u16, i16),
            (Int32, u32, i32),
            (Int64, u64, i64),
        );
        let value = match i128::from_str_radix(digits, radix) {
            Ok(value) if value <= limit => value,
            _ => self
                .error_at(digits_start_location, "number doesn't fit in type")?
                .into(),
        };
        match sign {
            Sign::Negative => Ok(ConstInteger {
                integer_type,
                value: -value as u64,
            }),
            Sign::Positive | Sign::Unsigned => Ok(ConstInteger {
                integer_type,
                value: value as u64,
            }),
        }
    }
    pub fn parse_string(&mut self) -> Result<StringToken<'t>, FromTextError> {
        if self.location.peek() != Some(StringToken::QUOTE) {
            self.error("missing string")?;
        }
        self.location.next();
        let string_body_start_location = self.location;
        loop {
            match self.location.peek() {
                None => self.error("unterminated string")?.into(),
                Some(StringToken::QUOTE) => {
                    let string_body_end_location = self.location;
                    self.location.next();
                    return Ok(StringToken {
                        source_text: string_body_start_location
                            .get_text_to(string_body_end_location),
                    });
                }
                _ => match StringToken::parse_char(&mut self.location) {
                    Ok(_) => {}
                    Err(message) => self.error(message)?.into(),
                },
            }
        }
    }
    fn parse_token_impl(&mut self) -> Result<Token<'t>, FromTextError> {
        self.skip_whitespace()?;
        let location = self.location;
        match self.location.peek() {
            None => Ok(Token {
                kind: TokenKind::EndOfFile,
                location,
            }),
            Some(StringToken::QUOTE) => Ok(Token {
                kind: TokenKind::String(self.parse_string()?),
                location,
            }),
            Some(ch) if ch.is_identifier_start() => Ok(Token {
                kind: self.parse_identifier_or_keyword()?.into(),
                location,
            }),
            Some(ch) if ch.is_ascii_digit() || ch == '+' || ch == '-' => Ok(Token {
                kind: TokenKind::String(self.parse_string()?),
                location,
            }),
            _ => self.error("invalid token")?.into(),
        }
    }
    pub fn peek_token(&mut self) -> Result<Token<'t>, FromTextError> {
        if let Some(cached_token) = self.cached_token {
            if cached_token.start_location == self.location {
                return Ok(cached_token.token);
            }
        }
        let start_location = self.location;
        let token = self.parse_token_impl()?;
        let next_location = self.location;
        self.location = start_location;
        self.cached_token = Some(CachedToken {
            start_location,
            token,
            next_location,
        });
        Ok(token)
    }
    pub fn parse_token(&mut self) -> Result<Token<'t>, FromTextError> {
        if let Some(cached_token) = self.cached_token.take() {
            if cached_token.start_location == self.location {
                self.location = cached_token.next_location;
                return Ok(cached_token.token);
            }
        }
        self.parse_token_impl()
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
        let source_code = FromTextSourceCode { file_name, text };
        let mut state = FromTextState {
            global_state,
            location: TextLocation::new(0, &source_code),
            cached_token: None,
        };
        Self::from_text(&mut state)
    }
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError>;
}
