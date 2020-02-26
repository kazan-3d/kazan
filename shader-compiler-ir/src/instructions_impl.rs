// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    prelude::*,
    text::{FromTextError, FromTextState, FromToTextListForm, ListForm, Punctuation, ToTextState},
    Alignment,
};
use alloc::vec::Vec;
use core::fmt;

macro_rules! impl_instructions {
    (
        #[kind = $instruction_kind:ident]
        pub enum $instruction_data:ident<$g:lifetime> {
            $(
                $(#[doc = $instruction_doc:expr])+
                #[text = $instruction_text:literal]
                $instruction:ident,
            )+
        }
    ) => {
        $(
            impl<$g> instructions::$instruction<$g> {
                /// instruction kind
                pub const KIND: $instruction_kind = $instruction_kind::$instruction;
            }

            impl<$g> From<instructions::$instruction<$g>> for $instruction_data<$g> {
                fn from(v: instructions::$instruction<$g>) -> Self {
                    Self::$instruction(v)
                }
            }
        )+

        /// instruction data
        pub enum $instruction_data<$g> {
            $(
                $(#[doc = $instruction_doc])*
                $instruction(instructions::$instruction<$g>),
            )+
        }

        /// instruction kind
        #[derive(Copy, Clone, Eq, PartialEq, Hash)]
        pub enum $instruction_kind {
            $(
                $(#[doc = $instruction_doc])+
                $instruction,
            )+
        }

        impl<$g> $instruction_data<$g> {
            /// get the kind of instruction
            pub fn kind(&self) -> $instruction_kind {
                match self {
                    $(
                        Self::$instruction(_) => $instruction_kind::$instruction,
                    )+
                }
            }
        }

        impl FromToTextListForm for $instruction_kind {
            fn from_to_text_list_form() -> ListForm {
                ListForm::STATEMENTS
            }
        }

        impl<$g> FromText<$g> for $instruction_kind {
            type Parsed = Self;
            fn from_text(state: &mut FromTextState<$g, '_>) -> Result<Self, FromTextError> {
                let retval = match state.peek_token()?.kind.raw_identifier() {
                    $(
                        Some($instruction_text) => $instruction_kind::$instruction,
                    )+
                    Some(_) => state.error_at_peek_token("invalid instruction kind")?.into(),
                    _ => state.error_at_peek_token("missing instruction kind")?.into(),
                };
                state.parse_token()?;
                Ok(retval)
            }
        }

        impl $instruction_kind {
            /// get textual form of `self`
            pub fn text(self) -> &'static str {
                match self {
                    $(
                        $instruction_kind::$instruction => $instruction_text,
                    )+
                }
            }
        }

        impl_display_as_to_text!(<$g> $instruction_kind);

        impl<$g> ToText<$g> for $instruction_kind {
            fn to_text(&self, state: &mut ToTextState<$g, '_>) -> fmt::Result {
                write!(state, "{}", self.text())
            }
        }

        impl<$g> CodeIO<$g> for $instruction_data<$g> {
            fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
                match self {
                    $(
                        $instruction_data::$instruction(v) => v.results(),
                    )+
                }
            }
            fn arguments(&self) -> &[ValueUse<'g>] {
                match self {
                    $(
                        $instruction_data::$instruction(v) => v.arguments(),
                    )+
                }
            }
        }

        impl<$g> FromToTextListForm for $instruction_data<$g> {
            fn from_to_text_list_form() -> ListForm {
                ListForm::STATEMENTS
            }
        }

        impl<$g> FromText<$g> for $instruction_data<$g> {
            type Parsed = Self;
            fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
                let start_location = state.location;
                let kind = $instruction_kind::from_text(state)?;
                state.location = start_location;
                match kind {
                    $(
                        $instruction_kind::$instruction => Ok(instructions::$instruction::from_text(state)?.into()),
                    )+
                }
            }
        }

        impl_display_as_to_text!(<$g> $instruction_data<$g>);

        impl<$g> ToText<$g> for $instruction_data<$g> {
            fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                match self {
                    $(
                        $instruction_data::$instruction(v) => v.to_text(state),
                    )+
                }
            }
        }
    };
}

macro_rules! instructions {
    (
        #[kind = $instruction_kind:ident]
        pub enum $instruction_data:ident<$g:lifetime> {
            $(
                $(#[doc = $instruction_doc:expr])+
                #[text = $instruction_text:literal]
                #[argument_count = $argument_count:literal]
                #[result_count = $result_count:literal]
                $instruction:ident {
                    $(
                        $(#[doc = $instruction_extra_field_doc:expr])+
                        pub $instruction_extra_field:ident: $instruction_extra_field_type:ty,
                    )*
                },
            )+
            #[break]
            $(#[doc = $break_instruction_doc:expr])+
            $break_instruction:ident,
            #[branch]
            $(#[doc = $branch_instruction_doc:expr])+
            $branch_instruction:ident,
            #[continue]
            $(#[doc = $continue_instruction_doc:expr])+
            $continue_instruction:ident,
            #[block]
            $(#[doc = $block_instruction_doc:expr])+
            $block_instruction:ident,
            #[loop]
            $(#[doc = $loop_instruction_doc:expr])+
            $loop_instruction:ident,
        }
    ) => {
        impl_instructions! {
            #[kind = $instruction_kind]
            pub enum $instruction_data<$g> {
                $(
                    $(#[doc = $instruction_doc])+
                    #[text = $instruction_text]
                    $instruction,
                )+
                $(#[doc = $break_instruction_doc])+
                #[text = "break"]
                $break_instruction,
                $(#[doc = $branch_instruction_doc])+
                #[text = "branch"]
                $branch_instruction,
                $(#[doc = $continue_instruction_doc])+
                #[text = "continue"]
                $continue_instruction,
                $(#[doc = $block_instruction_doc])+
                #[text = "block"]
                $block_instruction,
                $(#[doc = $loop_instruction_doc])+
                #[text = "loop"]
                $loop_instruction,
            }
        }

        /// instruction types
        pub mod instructions {
            use super::*;

            $(
                $(#[doc = $instruction_doc])+
                pub struct $instruction<$g> {
                    /// arguments
                    pub arguments: [ValueUse<$g>; $argument_count],
                    /// results
                    pub results: [ValueDefinition<$g>; $result_count],
                    $(
                        $(#[doc = $instruction_extra_field_doc])+
                        pub $instruction_extra_field: $instruction_extra_field_type,
                    )*
                }

                impl<$g> CodeIO<$g> for $instruction<$g> {
                    fn results(&self) -> Inhabitable<&[ValueDefinition<$g>]> {
                        Inhabited(&self.results)
                    }
                    fn arguments(&self) -> &[ValueUse<'g>] {
                        &self.arguments
                    }
                }

                impl<$g> FromToTextListForm for $instruction<$g> {
                    fn from_to_text_list_form() -> ListForm {
                        ListForm::STATEMENTS
                    }
                }

                impl<$g> FromText<$g> for $instruction<$g> {
                    type Parsed = Self;
                    fn from_text(state: &mut FromTextState<$g, '_>) -> Result<Self, FromTextError> {
                        let kind_location = state.peek_token()?.span;
                        if $instruction::KIND != $instruction_kind::from_text(state)? {
                            state.error_at(kind_location, format!("expected {} instruction", $instruction::KIND.text()))?;
                        }
                        let arguments = <[ValueUse; $argument_count]>::from_text(state)?;
                        $(
                            state.parse_punct_token_or_error(Punctuation::Comma, "missing comma: ','")?;
                            let $instruction_extra_field = <$instruction_extra_field_type>::from_text(state)?;
                        )*
                        state.parse_punct_token_or_error(Punctuation::Arrow, "missing arrow: '->'")?;
                        let results = <[ValueDefinition; $result_count]>::from_text(state)?;
                        Ok(Self {
                            arguments,
                            results,
                            $($instruction_extra_field,)*
                        })
                    }
                }

                impl_display_as_to_text!(<$g> $instruction<$g>);

                impl<$g> ToText<$g> for $instruction<$g> {
                    fn to_text(&self, state: &mut ToTextState<$g, '_>) -> fmt::Result {
                        write!(state, "{} ", $instruction::KIND.text())?;
                        let Self {
                            arguments,
                            results,
                            $($instruction_extra_field,)*
                        } = self;
                        arguments.to_text(state)?;
                        $(
                            write!(state, ", ")?;
                            $instruction_extra_field.to_text(state)?;
                        )*
                        write!(state, " -> ")?;
                        results.to_text(state)
                    }
                }
            )+

            pub use crate::$break_instruction;

            /// the target of a `Branch` instruction
            pub struct BranchTarget<'g> {
                /// the value the `Branch` instruction must match for this target to be executed
                pub value: Interned<'g, Const<'g>>,
                /// the break instruction that is executed when this target is executed
                pub break_block: BreakBlock<'g>,
            }

            impl<'g> FromToTextListForm for BranchTarget<'g> {
                fn from_to_text_list_form() -> ListForm {
                    ListForm::STATEMENTS
                }
            }

            impl<'g> FromText<'g> for BranchTarget<'g> {
                type Parsed = Self;
                fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
                    let value = Const::from_text(state)?;
                    state.parse_punct_token_or_error(Punctuation::Arrow, "missing arrow: '->'")?;
                    let break_block = BreakBlock::from_text(state)?;
                    Ok(Self {value,break_block})
                }
            }

            impl_display_as_to_text!(<'g> BranchTarget<'g>);

            impl<'g> ToText<'g> for BranchTarget<'g> {
                fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                    let Self {value,break_block} = self;
                    value.to_text(state)?;
                    write!(state, " -> ")?;
                    break_block.to_text(state)
                }
            }

            $(#[doc = $branch_instruction_doc])+
            pub struct $branch_instruction<'g> {
                /// the value to be matched against the list of `BranchTarget`s.
                pub variable: ValueUse<'g>,
                /// the list of branch targets. When this branch instruction is
                /// executed, the first matching target is executed (which
                /// executes the contained break instruction). If no targets
                /// match, execution proceeds to the following instruction.
                pub targets: Vec<BranchTarget<'g>>,
            }

            impl<'g> CodeIO<'g> for $branch_instruction<'g> {
                fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
                    Inhabited(&[])
                }
                fn arguments(&self) -> &[ValueUse<'g>] {
                    core::slice::from_ref(&self.variable)
                }
            }

            impl_display_as_to_text!(<'g> $branch_instruction<'g>);

            impl<'g> FromToTextListForm for $branch_instruction<'g> {
                fn from_to_text_list_form() -> ListForm {
                    ListForm::STATEMENTS
                }
            }

            impl<'g> ToText<'g> for $branch_instruction<'g> {
                fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                    write!(state, "{} ", $branch_instruction::KIND.text())?;
                    let Self {
                        variable,
                        targets,
                    } = self;
                    [variable].to_text(state)?;
                    write!(state, ", ")?;
                    targets.to_text(state)?;
                    write!(state, " -> ")?;
                    self.results().to_text(state)
                }
            }

            impl<'g> FromText<'g> for $branch_instruction<'g> {
                type Parsed = Self;
                fn from_text(state: &mut FromTextState<$g, '_>) -> Result<Self, FromTextError> {
                    let kind_location = state.peek_token()?.span;
                    if Self::KIND != $instruction_kind::from_text(state)? {
                        state.error_at(kind_location, format!("expected {} instruction", Self::KIND.text()))?;
                    }
                    let [variable] = <[ValueUse<'g>; 1]>::from_text(state)?;
                    state.parse_punct_token_or_error(Punctuation::Comma, "missing comma: ','")?;
                    let targets = Vec::<BranchTarget<'g>>::from_text(state)?;
                    state.parse_punct_token_or_error(Punctuation::Arrow, "missing arrow: '->'")?;
                    <[ValueDefinition<'g>; 0]>::from_text(state)?;
                    Ok(Self {
                        variable,
                        targets,
                    })
                }
            }

            pub use crate::$continue_instruction;
            pub use crate::$block_instruction;
            pub use crate::$loop_instruction;
        }
    };
}

instructions! {
    #[kind = InstructionKind]
    pub enum InstructionData<'g> {
        /// add
        #[text = "add"]
        #[argument_count = 2]
        #[result_count = 1]
        Add {},

        /// load
        #[text = "load"]
        #[argument_count = 1]
        #[result_count = 1]
        Load {
            /// pointer alignment
            pub alignment: Alignment,
        },

        #[break]
        /// break from a `Block`
        BreakBlock,

        #[branch]
        /// dynamically select a `Block` to break from, equivalent of Rust's `match` and C, SPIR-V, and LLVM's `switch`.
        Branch,

        #[continue]
        /// continue a loop
        ContinueLoop,

        #[block]
        /// block of code
        Block,

        #[loop]
        /// a loop
        Loop,
    }
}

/// a instruction
pub struct Instruction<'g> {
    /// the debug location of this instruction
    pub location: Option<Interned<'g, Location<'g>>>,
    /// the `InstructionData` for this instruction
    pub data: InstructionData<'g>,
}

impl<'g> Instruction<'g> {
    /// create a new `Instruction`, optionally with a debug location
    pub fn new(
        location: Option<Interned<'g, Location<'g>>>,
        data: impl Into<InstructionData<'g>>,
    ) -> Self {
        Self {
            location,
            data: data.into(),
        }
    }
    /// create a new `Instruction`, with a debug location
    pub fn with_location(
        location: Interned<'g, Location<'g>>,
        data: impl Into<InstructionData<'g>>,
    ) -> Self {
        Self {
            location: Some(location),
            data: data.into(),
        }
    }
    /// create a new `Instruction`, with a debug location
    pub fn with_internable_location(
        location: impl Internable<'g, Interned = Location<'g>>,
        data: impl Into<InstructionData<'g>>,
        global_state: &'g GlobalState<'g>,
    ) -> Self {
        Self {
            location: Some(location.intern(global_state)),
            data: data.into(),
        }
    }
    /// create a new `Instruction`, without a debug location.
    /// Having a debug location should be preferred when that information is available.
    pub fn without_location(data: impl Into<InstructionData<'g>>) -> Self {
        Self {
            location: None,
            data: data.into(),
        }
    }
}

impl<'g> CodeIO<'g> for Instruction<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        self.data.results()
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        self.data.arguments()
    }
}

impl_display_as_to_text!(<'g> Instruction<'g>);

impl FromToTextListForm for Instruction<'_> {
    fn from_to_text_list_form() -> ListForm {
        ListForm::STATEMENTS
    }
}

impl<'g> ToText<'g> for Instruction<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let Self { location, data } = self;
        data.to_text(state)?;
        if let Some(location) = location {
            write!(state, " @ ")?;
            location.to_text(state)?;
        }
        Ok(())
    }
}

impl<'g> FromText<'g> for Instruction<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let data = InstructionData::from_text(state)?;
        let location = if Some(Punctuation::At) == state.peek_token()?.kind.punct() {
            state.parse_token()?;
            Some(Location::from_text(state)?)
        } else {
            None
        };
        Ok(Self { location, data })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        instructions,
        text::{FromText, FromTextError, FromTextState, FromToTextListForm, ToText, ToTextState},
        Alignment, DataPointerType, GlobalState, Instruction, IntegerType, ValueDefinition,
        ValueUse,
    };
    use alloc::{string::ToString, vec::Vec};
    use core::fmt;

    struct ValueDefinitionsThenInstruction<'g> {
        value_definitions: Vec<ValueDefinition<'g>>,
        instruction: Instruction<'g>,
    }

    impl FromToTextListForm for ValueDefinitionsThenInstruction<'_> {}

    impl<'g> FromText<'g> for ValueDefinitionsThenInstruction<'g> {
        type Parsed = Self;
        fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
            let value_definitions = Vec::<ValueDefinition>::from_text(state)?;
            let instruction = Instruction::from_text(state)?;
            Ok(Self {
                value_definitions,
                instruction,
            })
        }
    }

    impl<'g> ToText<'g> for ValueDefinitionsThenInstruction<'g> {
        fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
            let Self {
                value_definitions,
                instruction,
            } = self;
            value_definitions.to_text(state)?;
            writeln!(state)?;
            instruction.to_text(state)
        }
    }

    fn test_instruction_from_to_text<'g>(
        global_state: &'g GlobalState<'g>,
        value: ValueDefinitionsThenInstruction<'g>,
        text: &str,
        assert_eq: fn(&str, &str),
    ) {
        assert_eq(&value.display().to_string(), text);
        let value =
            ValueDefinitionsThenInstruction::parse("test_input", text, global_state).unwrap();
        assert_eq(&value.display().to_string(), text);
    }

    macro_rules! test_instruction_from_to_text {
        (
            $test_name:ident, $global_state:ident;
            $instr:ident [$($arg:ident: $arg_type:expr),*] $(, $field:ident: $field_value:expr)* => [$($result:ident: $result_type:expr),*];
            $($text:literal),+
        ) => {
            #[test]
            fn $test_name() {
                let $global_state = GlobalState::new();
                let $global_state = &$global_state;
                $(let $arg = ValueDefinition::new($arg_type, stringify!($arg), $global_state);)*
                $(let $result = ValueDefinition::new($result_type, stringify!($result), $global_state);)*
                test_instruction_from_to_text(
                    $global_state,
                    ValueDefinitionsThenInstruction {
                        instruction: Instruction::without_location(instructions::$instr {
                            $($field: $field_value,)*
                            arguments: [$(ValueUse::new($arg.value()),)*],
                            results: [$($result,)*],
                        }),
                        value_definitions: vec![$($arg,)*],
                    },
                    concat!($($text,)+),
                    |a, b| assert_eq!(a, b)
                );
            }
        };
    }

    test_instruction_from_to_text! {
        test_add_instruction_from_to_text, global_state;
        Add [arg0: IntegerType::Int32, arg1: IntegerType::Int32] => [result: IntegerType::Int32];
        "[arg0: i32, arg1: i32]\n",
        "add [arg0, arg1] -> [result: i32]"
    }

    test_instruction_from_to_text! {
        test_load_instruction_from_to_text, global_state;
        Load [arg0: DataPointerType], alignment: Alignment::new(4).unwrap() => [result: IntegerType::Int32];
        "[arg0: data_ptr]\n",
        "load [arg0], align: 0x4 -> [result: i32]"
    }
}
