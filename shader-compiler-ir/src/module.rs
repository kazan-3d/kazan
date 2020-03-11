// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    prelude::*,
    text::{
        FromText, FromTextError, FromTextState, FromToTextListForm, Keyword, ListForm, Punctuation,
        ToText, ToTextDisplay, ToTextState,
    },
    BuiltInInterfaceVariableAttributes, InterfaceBlock, InterfaceVariable, TargetProperties,
    UserInterfaceVariableAttributes, Variable,
};
use alloc::vec::Vec;
use core::fmt;

macro_rules! impl_module {
    (
        $(#[doc = $module_doc:expr])+
        $module_vis:vis struct Module<$g:lifetime> {
            $(
                $(#[doc = $doc:expr])+
                $(#[keyword = $keyword_text:literal])?
                $vis:vis $name:ident: $ty:ty,
            )+
            #[functions]
        }
    ) => {
        $(#[doc = $module_doc])+
        $module_vis struct Module<$g> {
            $(
                $(#[doc = $doc])+
                $vis $name: $ty,
            )+
            /// the functions
            $module_vis functions: Vec<Function<$g>>,
            /// the entry point
            $module_vis entry_point: FunctionRef<$g>,
        }

        impl_display_as_to_text!(<$g> Module<$g>);

        impl FromToTextListForm for Module<'_> {
            fn from_to_text_list_form() -> ListForm {
                ListForm::STATEMENTS
            }
        }

        impl<$g> FromText<$g> for Module<$g> {
            type Parsed = Self;
            fn from_text(state: &mut FromTextState<$g, '_>) -> Result<Self::Parsed, FromTextError> {
                state.parse_keyword_token_or_error(Keyword::Module, "missing `module` keyword")?;
                state.parse_parenthesized(
                    Punctuation::LCurlyBrace,
                    "missing opening curly brace (`{`)",
                    Punctuation::RCurlyBrace,
                    "missing closing curly brace (`}`)",
                    |state| {
                        $(
                            $(
                                if let Some($keyword_text) = state.peek_token()?.kind.raw_identifier() {
                                    state.parse_token()?;
                                } else {
                                    state.error_at_peek_token(concat!("missing `", $keyword_text, "` keyword"))?;
                                }
                            )?
                            let $name = <$ty>::from_text(state)?;
                        )+
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
                            $($name,)+
                            functions,
                            entry_point,
                        })
                    },
                )
            }
        }

        impl<$g> ToText<$g> for Module<$g> {
            fn to_text(&self, state: &mut ToTextState<$g, '_>) -> fmt::Result {
                writeln!(state, "module {{")?;
                state.indent(|state| {
                    let Module {
                        $($name,)+
                        functions,
                        entry_point,
                    } = self;
                    $(
                        $(
                            write!(state, "{} ", $keyword_text)?;
                        )?
                        $name.to_text(state)?;
                        writeln!(state)?;
                    )+
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
            fn display(&self) -> ToTextDisplay<$g, '_, Self> {
                ToTextDisplay::new(self, false)
            }
        }
    };
}

impl_module! {
    /// a shader module
    pub struct Module<'g> {
        /// the target properties
        pub target_properties: Interned<'g, TargetProperties>,
        /// the inputs interface variables for built-ins
        #[keyword = "built_in_inputs"]
        pub built_in_inputs: Vec<InterfaceVariable<'g, BuiltInInterfaceVariableAttributes>>,
        /// the inputs interface block for user variables
        #[keyword = "user_inputs_block"]
        pub user_inputs_block: InterfaceBlock<'g, UserInterfaceVariableAttributes>,
        /// the outputs interface variables for built-ins
        #[keyword = "built_in_outputs"]
        pub built_in_outputs: Vec<InterfaceVariable<'g, BuiltInInterfaceVariableAttributes>>,
        /// the outputs interface block for user variables
        #[keyword = "user_outputs_block"]
        pub user_outputs_block: InterfaceBlock<'g, UserInterfaceVariableAttributes>,
        /// the per-invocation global variables of this module
        #[keyword = "invocation_global_variables"]
        pub invocation_global_variables: Vec<Variable<'g>>,
        #[functions]
    }
}
