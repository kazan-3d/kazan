// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextState;
use crate::text::Punctuation;
use crate::text::ToTextState;
use std::fmt;

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
        #[derive(Debug)]
        pub enum $instruction_data<$g> {
            $(
                $(#[doc = $instruction_doc])*
                $instruction(instructions::$instruction<$g>),
            )+
        }

        /// instruction kind
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        pub enum $instruction_kind {
            $(
                $(#[doc = $instruction_doc])+
                $instruction,
            )+
        }

        impl<$g> $instruction_data<$g> {
            pub fn kind(&self) -> $instruction_kind {
                match self {
                    $(
                        Self::$instruction(_) => $instruction_kind::$instruction,
                    )+
                }
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
                #[derive(Debug)]
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
                            let $instruction_extra_field = $instruction_extra_field_type::from_text(state)?;
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

            #[derive(Debug)]
            pub struct BranchTarget<'g> {
                pub value: Interned<'g, Const<'g>>,
                pub break_block: BreakBlock<'g>,
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

            impl<'g> ToText<'g> for BranchTarget<'g> {
                fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                    let Self {value,break_block} = self;
                    value.to_text(state)?;
                    write!(state, " -> ")?;
                    break_block.to_text(state)
                }
            }

            #[derive(Debug)]
            pub struct $branch_instruction<'g> {
                pub variable: ValueUse<'g>,
                pub targets: Vec<BranchTarget<'g>>,
            }

            impl<'g> CodeIO<'g> for $branch_instruction<'g> {
                fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
                    Inhabited(&[])
                }
                fn arguments(&self) -> &[ValueUse<'g>] {
                    std::slice::from_ref(&self.variable)
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

        #[break]
        /// break from a `Block`
        BreakBlock,

        #[branch]
        /// dynamically select a `Block` to break from
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

#[derive(Debug)]
pub struct Instruction<'g> {
    pub location: Option<Interned<'g, Location<'g>>>,
    pub data: InstructionData<'g>,
}

impl<'g> Instruction<'g> {
    pub fn new(
        location: Option<Interned<'g, Location<'g>>>,
        data: impl Into<InstructionData<'g>>,
    ) -> Self {
        Self {
            location: location,
            data: data.into(),
        }
    }
    pub fn with_location(
        location: Interned<'g, Location<'g>>,
        data: impl Into<InstructionData<'g>>,
    ) -> Self {
        Self {
            location: Some(location),
            data: data.into(),
        }
    }
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
