// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    prelude::*,
    text::{
        FromText, FromTextError, FromTextState, Keyword, Punctuation, ToText, ToTextDisplay,
        ToTextState,
    },
    TargetProperties,
};
use alloc::vec::Vec;
use core::fmt;

/// a shader module
pub struct Module<'g> {
    /// the target properties
    pub target_properties: Interned<'g, TargetProperties>,
    /// the functions
    pub functions: Vec<Function<'g>>,
    /// the entry point
    pub entry_point: FunctionRef<'g>,
}

impl_display_as_to_text!(<'g> Module<'g>);

impl<'g> FromText<'g> for Module<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
        state.parse_keyword_token_or_error(Keyword::Module, "missing `module` keyword")?;
        state.parse_parenthesized(
            Punctuation::LCurlyBrace,
            "missing opening curly brace (`{`)",
            Punctuation::RCurlyBrace,
            "missing closing curly brace (`}`)",
            |state| {
                let target_properties = TargetProperties::from_text(state)?;
                let mut functions = Vec::new();
                while state.peek_token()?.kind.keyword() == Some(Keyword::Fn) {
                    functions.push(Function::from_text(state)?);
                }
                state.parse_keyword_token_or_error(
                    Keyword::EntryPoint,
                    "missing `entry_point` keyword",
                )?;
                state.parse_punct_token_or_error(
                    Punctuation::Colon,
                    "missing colon (`:`) between entry_point keyword and function name",
                )?;
                let entry_point = FunctionRef::from_text(state)?;
                state.parse_punct_token_or_error(
                    Punctuation::Semicolon,
                    "missing terminating semicolon (`;`) after entry_point declaration",
                )?;
                Ok(Module {
                    target_properties,
                    functions,
                    entry_point,
                })
            },
        )
    }
}

impl<'g> ToText<'g> for Module<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        writeln!(state, "module {{")?;
        state.indent(|state| {
            let Module {
                target_properties,
                functions,
                entry_point,
            } = self;
            target_properties.to_text(state)?;
            writeln!(state)?;
            for function in functions {
                function.to_text(state)?;
                writeln!(state)?;
            }
            write!(state, "entry_point: ")?;
            entry_point.to_text(state)?;
            writeln!(state, ";")
        })?;
        write!(state, "}}")
    }
    fn display(&self) -> ToTextDisplay<'g, '_, Self> {
        ToTextDisplay::new(self, false)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use alloc::string::ToString;

    #[test]
    fn test_from_to_text() {
        let global_state = GlobalState::new();
        let global_state = &global_state;

        let text = concat!(
            "module {\n",
            "    target_properties {\n",
            "        data_pointer_underlying_type: i64,\n",
            "        function_pointer_underlying_type: i64,\n",
            "    }\n",
            "    fn main[] -> [] {\n",
            "        hints {\n",
            "            inlining_hint: none,\n",
            "            side_effects: normal,\n",
            "        }\n",
            "        {\n",
            "        }\n",
            "        block1 {\n",
            "            break block1[];\n",
            "        }\n",
            "    }\n",
            "    entry_point: main;\n",
            "}"
        );
        let value = Module::parse("", text, global_state).unwrap();
        let text2 = value.to_string();
        assert_eq!(text, text2);
    }
}
