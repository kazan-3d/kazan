// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
use alloc::string::String;
use core::fmt;
use spirv_parser::Decoration;
use spirv_parser::IdRef;
use spirv_parser::IdResult;

macro_rules! impl_error {
    (
        $(#[doc = $doc:expr])*
        #[display = $display:literal]
        pub struct $name:ident {
            $(
                $(#[doc = $member_doc:expr])*
                pub $member_name:ident: $member_ty:ty,
            )*
        }
    ) => {
        $(#[doc = $doc])*
        #[derive(Debug)]
        pub struct $name {
            $(
                $(#[doc = $member_doc])*
                pub $member_name: $member_ty,
            )*
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    f,
                    $display,
                    $($member_name = self.$member_name,)*
                )
            }
        }
    };
    (
        $(#[doc = $doc:expr])*
        #[display = $display:literal]
        pub struct $name:ident;
    ) => {
        $(#[doc = $doc])*
        #[derive(Debug)]
        pub struct $name;

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, $display)
            }
        }
    };
}

pub(crate) fn decoration_not_allowed(
    member_index: Option<u32>,
    decoration: Decoration,
    instruction: spirv_parser::Instruction,
) -> TranslationError {
    if let Some(member_index) = member_index {
        MemberDecorationNotAllowed {
            member_index,
            decoration,
            instruction,
        }
        .into()
    } else {
        DecorationNotAllowedOnInstruction {
            decoration,
            instruction,
        }
        .into()
    }
}

macro_rules! impl_translation_error {
    (
        $(
            $(#[doc = $doc:expr])*
            $error:ident($wrapped_error:ty),
        )+
    ) => {
        $(
            impl From<$wrapped_error> for TranslationError {
                fn from(v: $wrapped_error) -> Self {
                    TranslationError::$error(v)
                }
            }
        )+

        #[derive(Debug)]
        pub enum TranslationError {
            $(
                $(#[doc = $doc])*
                $error($wrapped_error),
            )+
        }

        impl fmt::Display for TranslationError {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(
                        Self::$error(v) => fmt::Display::fmt(v, f),
                    )+
                }
            }
        }
    };
}

macro_rules! impl_errors {
    (
        {
            $(
                $(#[doc = $wrapped_error_doc:expr])*
                $wrapped_error_name:ident($wrapped_error:ty),
            )+
        }
        $(
            $(#[doc = $doc:expr])*
            #[display = $display:literal]
            pub struct $name:ident $body:tt
        )+
    ) => {
        impl_translation_error! {
            $(
                $(#[doc = $wrapped_error_doc])*
                $wrapped_error_name($wrapped_error),
            )+
            $(
                $(#[doc = $doc])*
                $name($name),
            )+
        }

        $(
            impl_error! {
                $(#[doc = $doc])*
                #[display = $display]
                pub struct $name $body
            }
        )+
    };
}

impl_errors! {
    {
        SPIRVParserError(spirv_parser::Error),
        FormattingFailed(fmt::Error),
        SPIRVIdOutOfBounds(spirv_id_map::IdOutOfBounds),
    }

    #[display = "invalid SPIR-V instruction in \'{section_name}\' section:\n{instruction}"]
    pub struct InvalidSPIRVInstructionInSection {
        pub instruction: spirv_parser::Instruction,
        pub section_name: &'static str,
    }

    #[display = "shader specialization failed"]
    pub struct SpecializationResolutionFailed;

    #[display = "SPIR-V extension \"{name:?}\" not supported"]
    pub struct SPIRVExtensionNotSupported {
        pub name: String,
    }

    #[display = "SPIR-V extension instruction set \"{name:?}\" not supported"]
    pub struct SPIRVExtensionInstructionSetNotSupported {
        pub name: String,
    }

    #[display = "SPIR-V capability \"{capability:?}\" not supported"]
    pub struct SPIRVCapabilityNotSupported {
        pub capability: spirv_parser::Capability,
    }

    #[display = "SPIR-V memory model \"{memory_model:?}\" not supported"]
    pub struct SPIRVMemoryModelNotSupported {
        pub memory_model: spirv_parser::MemoryModel,
    }

    #[display = "missing SPIR-V OpMemoryModel instruction"]
    pub struct MissingSPIRVOpMemoryModel;

    #[display = "SPIR-V addressing model \"{addressing_model:?}\" not supported"]
    pub struct SPIRVAddressingModelNotSupported {
        pub addressing_model: spirv_parser::AddressingModel,
    }

    #[display = "duplicate SPIR-V entry point with name \"{name:?}\" and execution model {execution_model:?}"]
    pub struct DuplicateSPIRVEntryPoint {
        pub name: String,
        pub execution_model: spirv_parser::ExecutionModel,
    }

    #[display = "matching SPIR-V entry point with name \"{name:?}\" and execution model {execution_model:?} not found"]
    pub struct MatchingSPIRVEntryPointNotFound {
        pub name: String,
        pub execution_model: spirv_parser::ExecutionModel,
    }

    #[display = "unsupported SPIR-V execution mode: {execution_mode:?}"]
    pub struct UnsupportedSPIRVExecutionMode {
        pub execution_mode: spirv_parser::ExecutionMode,
    }

    #[display = "duplicate SPIR-V LocalSize annotation for entry point"]
    pub struct DuplicateSPIRVLocalSize;

    #[display = "SPIR-V Result <id> ({id_result}) already defined"]
    pub struct SPIRVIdAlreadyDefined {
        pub id_result: IdResult,
    }

    #[display = "SPIR-V <id> ({id}) not defined"]
    pub struct SPIRVIdNotDefined {
        pub id: IdRef,
    }

    #[display = "SPIR-V member decorations are only allowed on struct types: \
                 target <id> ({target}) is not a struct type"]
    pub struct MemberDecorationsAreOnlyAllowedOnStructTypes {
        pub target: IdRef,
    }

    #[display = "unsupported SPIR-V type:\n{instruction}"]
    pub struct UnsupportedSPIRVType {
        pub instruction: spirv_parser::Instruction,
    }

    #[display = "SPIR-V void type ({type_id}) not allowed here:\n{instruction}"]
    pub struct VoidNotAllowedHere {
        pub type_id: IdRef,
        pub instruction: spirv_parser::Instruction,
    }

    #[display = "SPIR-V decoration is not allowed on instruction: {decoration:?}\n{instruction}"]
    pub struct DecorationNotAllowedOnInstruction {
        pub decoration: Decoration,
        pub instruction: spirv_parser::Instruction,
    }

    #[display = "invalid floating-point type bit-width: {width}"]
    pub struct InvalidFloatTypeBitWidth {
        pub width: u32,
    }

    #[display = "invalid integer type with bit-width {width} and signedness {signedness}"]
    pub struct InvalidIntegerType {
        pub width: u32,
        pub signedness: u32,
    }

    #[display = "invalid vector component type ({component_type_id}): \
            must be a SPIR-V scalar type (a floating-point type, an integer type, or a boolean type)"]
    pub struct InvalidVectorComponentType {
        pub component_type_id: IdRef,
    }

    #[display = "invalid vector component count ({component_count}): Vulkan requires vectors to have 2 through 4 components"]
    pub struct InvalidVectorComponentCount {
        pub component_count: u32,
    }

    #[display = "member decoration's member index ({member_index}) out of bounds: Decoration: {decoration:?}\n{instruction}"]
    pub struct MemberDecorationIndexOutOfBounds {
        pub member_index: u32,
        pub decoration: Decoration,
        pub instruction: spirv_parser::Instruction,
    }

    #[display = "SPIR-V member decoration is not allowed: member index {member_index}: {decoration:?}\n{instruction}"]
    pub struct MemberDecorationNotAllowed {
        pub member_index: u32,
        pub decoration: Decoration,
        pub instruction: spirv_parser::Instruction,
    }

    #[display = "BuiltIn and non-BuiltIn struct members are not allowed in the same struct: member index {member_index}:\n{instruction}"]
    pub struct BuiltInAndNonBuiltInNotAllowedInSameStruct {
        pub member_index: u32,
        pub instruction: spirv_parser::Instruction,
    }

    #[display = "OpVariable's result type must be OpTypePointer:\n{instruction}"]
    pub struct VariableResultTypeMustBePointer {
        pub instruction: spirv_parser::Instruction,
    }

    #[display = "RelaxedPrecision decoration not allowed:\n{instruction}"]
    pub struct RelaxedPrecisionDecorationNotAllowed {
        pub instruction: spirv_parser::Instruction,
    }
}

pub(crate) type TranslationResult<T> = Result<T, TranslationError>;
