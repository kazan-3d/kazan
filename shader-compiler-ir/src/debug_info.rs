// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    prelude::*,
    text::{FromTextError, FromTextState, IntegerToken, Punctuation, ToTextState},
};
use core::{convert::TryInto, fmt};

/// a debug location
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Location<'g> {
    /// the source file name
    pub file: Interned<'g, str>,
    /// the line number
    pub line: u32,
    /// the column number
    pub column: u32,
}

impl<'g> Location<'g> {
    /// create a new `Location`
    pub fn new_interned(
        file: impl Internable<'g, Interned = str>,
        line: u32,
        column: u32,
        global_state: &'g GlobalState<'g>,
    ) -> Interned<'g, Location<'g>> {
        let file = file.intern(global_state);
        Location { file, line, column }.intern(global_state)
    }
}

impl_display_as_to_text!(<'g> Location<'g>);

impl<'g> ToText<'g> for Location<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let Self { file, line, column } = self;
        file.to_text(state)?;
        write!(state, ":{}:{}", line, column)
    }
}

impl<'g> FromText<'g> for Location<'g> {
    type Parsed = Interned<'g, Location<'g>>;
    fn from_text(
        state: &mut FromTextState<'g, '_>,
    ) -> Result<Interned<'g, Location<'g>>, FromTextError> {
        let file = match state.peek_token()?.kind.string() {
            Some(file) => {
                state.parse_token()?;
                file.value().intern(state.global_state())
            }
            None => state
                .error_at_peek_token(
                    "expected file name as string: (example: \"shaders/my-shader.vertex\")",
                )?
                .into(),
        };
        let mut parse_colon_then_int = |name| -> Result<u32, FromTextError> {
            state.parse_punct_token_or_error(
                Punctuation::Colon,
                format_args!("missing colon before {}: ':'", name),
            )?;
            match state.peek_token()?.kind.integer() {
                Some(IntegerToken { value, suffix }) => {
                    if suffix.is_some() {
                        state.error_at_peek_token(format_args!(
                            "{} must be unsuffixed integer (123 and not 123i8)",
                            name
                        ))?;
                    }
                    if let Ok(value) = value.try_into() {
                        state.parse_token()?;
                        Ok(value)
                    } else {
                        state
                            .error_at_peek_token(format_args!("{} too big", name))?
                            .into()
                    }
                }
                None => state
                    .error_at_peek_token(format_args!("missing {}", name))?
                    .into(),
            }
        };
        let line = parse_colon_then_int("line number")?;
        let column = parse_colon_then_int("column number")?;
        Ok(Self { file, line, column }.intern(state.global_state()))
    }
}
