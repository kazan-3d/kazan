// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use std::error::Error;
use std::fmt;
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

pub struct FromTextState<'g, 't> {
    pub global_state: &'g GlobalState<'g>,
    pub iter: TextIterator<'t>,
    _private: (),
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

#[derive(Clone, Debug)]
pub struct TextIterator<'a> {
    iter: std::str::Chars<'a>,
    source_code: &'a FromTextSourceCode<'a>,
}

impl Iterator for TextIterator<'_> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        self.iter.next()
    }
}

impl<'a> TextIterator<'a> {
    pub fn new(source_code: &'a FromTextSourceCode<'a>, byte_index: usize) -> Self {
        Self {
            iter: source_code.text[byte_index..].chars(),
            source_code,
        }
    }
    pub fn peek(&self) -> Option<char> {
        self.clone().next()
    }
    pub fn source_code(&self) -> &'a FromTextSourceCode<'a> {
        self.source_code
    }
    pub fn byte_index(&self) -> usize {
        self.source_code.text.len() - self.iter.as_str().len()
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

impl From<TextIterator<'_>> for FromTextErrorLocation {
    fn from(v: TextIterator) -> FromTextErrorLocation {
        v.to_location()
    }
}

impl<'g, 't> FromTextState<'g, 't> {
    pub fn error<L: Into<FromTextErrorLocation>>(
        &mut self,
        message: String,
    ) -> Result<std::convert::Infallible, FromTextError> {
        self.error_at(self.iter.clone(), message)
    }
    pub fn error_at<L: Into<FromTextErrorLocation>>(
        &mut self,
        location: L,
        message: String,
    ) -> Result<std::convert::Infallible, FromTextError> {
        Err(FromTextError {
            location: location.into(),
            message,
        })
    }
}

pub trait FromText<'g>: Sized {
    fn parse<'t>(
        file_name: &'t str,
        text: &'t str,
        global_state: &'g GlobalState<'g>,
    ) -> Result<Self, FromTextError> {
        let source_code = FromTextSourceCode { file_name, text };
        let mut state = FromTextState {
            global_state,
            iter: TextIterator::new(&source_code, 0),
            _private: (),
        };
        Self::from_text(&mut state)
    }
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError>;
}
