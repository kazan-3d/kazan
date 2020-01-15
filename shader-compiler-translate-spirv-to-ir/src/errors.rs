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

impl_error! {
    #[display = "invalid SPIR-V instruction in \'{section_name}\' section:\n{instruction}"]
    pub struct InvalidSPIRVInstructionInSection {
        pub instruction: spirv_parser::Instruction,
        pub section_name: &'static str,
    }
}

impl_error! {
    #[display = "shader specialization failed"]
    pub struct SpecializationResolutionFailed;
}

impl_error! {
    #[display = "SPIR-V extension \"{name:?}\" not supported"]
    pub struct SPIRVExtensionNotSupported {
        pub name: String,
    }
}

impl_error! {
    #[display = "SPIR-V extension instruction set \"{name:?}\" not supported"]
    pub struct SPIRVExtensionInstructionSetNotSupported {
        pub name: String,
    }
}

impl_error! {
    #[display = "SPIR-V capability \"{capability:?}\" not supported"]
    pub struct SPIRVCapabilityNotSupported {
        pub capability: spirv_parser::Capability,
    }
}

impl_error! {
    #[display = "SPIR-V memory model \"{memory_model:?}\" not supported"]
    pub struct SPIRVMemoryModelNotSupported {
        pub memory_model: spirv_parser::MemoryModel,
    }
}

impl_error! {
    #[display = "missing SPIR-V OpMemoryModel instruction"]
    pub struct MissingSPIRVOpMemoryModel;
}

impl_error! {
    #[display = "SPIR-V addressing model \"{addressing_model:?}\" not supported"]
    pub struct SPIRVAddressingModelNotSupported {
        pub addressing_model: spirv_parser::AddressingModel,
    }
}

impl_error! {
    #[display = "duplicate SPIR-V entry point with name \"{name:?}\" and execution model {execution_model:?}"]
    pub struct DuplicateSPIRVEntryPoint {
        pub name: String,
        pub execution_model: spirv_parser::ExecutionModel,
    }
}

impl_error! {
    #[display = "matching SPIR-V entry point with name \"{name:?}\" and execution model {execution_model:?} not found"]
    pub struct MatchingSPIRVEntryPointNotFound {
        pub name: String,
        pub execution_model: spirv_parser::ExecutionModel,
    }
}

impl_error! {
    #[display = "unsupported SPIR-V execution mode: {execution_mode:?}"]
    pub struct UnsupportedSPIRVExecutionMode {
        pub execution_mode: spirv_parser::ExecutionMode,
    }
}

impl_error! {
    #[display = "duplicate SPIR-V LocalSize annotation for entry point"]
    pub struct DuplicateSPIRVLocalSize;
}

impl_error! {
    #[display = "SPIR-V Result <id> ({id_result}) already defined"]
    pub struct SPIRVIdAlreadyDefined {
        pub id_result: IdResult,
    }
}

impl_error! {
    #[display = "SPIR-V <id> ({id}) not defined"]
    pub struct SPIRVIdNotDefined {
        pub id: IdRef,
    }
}

impl_error! {
    #[display = "SPIR-V member decorations are only allowed on struct types: \
        target <id> ({target}) is not a struct type"]
    pub struct MemberDecorationsAreOnlyAllowedOnStructTypes {
        pub target: IdRef,
    }
}

impl_error! {
    #[display = "unsupported SPIR-V type:\n{instruction}"]
    pub struct UnsupportedSPIRVType {
        pub instruction: spirv_parser::Instruction,
    }
}

impl_error! {
    #[display = "SPIR-V void type ({type_id}) not allowed here:\n{instruction}"]
    pub struct VoidNotAllowedHere {
        pub type_id: IdRef,
        pub instruction: spirv_parser::Instruction,
    }
}

impl_error! {
    #[display = "SPIR-V decoration is not allowed on instruction: {decoration:?}\n{instruction}"]
    pub struct DecorationNotAllowedOnInstruction {
        pub decoration: Decoration,
        pub instruction: spirv_parser::Instruction,
    }
}

impl_error! {
    #[display = "invalid floating-point type bit-width: {width}"]
    pub struct InvalidFloatTypeBitWidth {
        pub width: u32,
    }
}

impl_error! {
    #[display = "invalid integer type with bit-width {width} and signedness {signedness}"]
    pub struct InvalidIntegerType {
        pub width: u32,
        pub signedness: u32,
    }
}

macro_rules! impl_translation_error {
    ($($error:ident($wrapped_error:ty),)+) => {
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

impl_translation_error! {
    SpecializationResolutionFailed(SpecializationResolutionFailed),
    SPIRVParserError(spirv_parser::Error),
    InvalidSPIRVInstructionInSection(InvalidSPIRVInstructionInSection),
    SPIRVCapabilityNotSupported(SPIRVCapabilityNotSupported),
    FormattingFailed(fmt::Error),
    SPIRVExtensionNotSupported(SPIRVExtensionNotSupported),
    SPIRVExtensionInstructionSetNotSupported(SPIRVExtensionInstructionSetNotSupported),
    SPIRVMemoryModelNotSupported(SPIRVMemoryModelNotSupported),
    MissingSPIRVOpMemoryModel(MissingSPIRVOpMemoryModel),
    SPIRVAddressingModelNotSupported(SPIRVAddressingModelNotSupported),
    DuplicateSPIRVEntryPoint(DuplicateSPIRVEntryPoint),
    MatchingSPIRVEntryPointNotFound(MatchingSPIRVEntryPointNotFound),
    UnsupportedSPIRVExecutionMode(UnsupportedSPIRVExecutionMode),
    DuplicateSPIRVLocalSize(DuplicateSPIRVLocalSize),
    SPIRVIdAlreadyDefined(SPIRVIdAlreadyDefined),
    SPIRVIdOutOfBounds(spirv_id_map::IdOutOfBounds),
    SPIRVIdNotDefined(SPIRVIdNotDefined),
    MemberDecorationsAreOnlyAllowedOnStructTypes(MemberDecorationsAreOnlyAllowedOnStructTypes),
    UnsupportedSPIRVType(UnsupportedSPIRVType),
    VoidNotAllowedHere(VoidNotAllowedHere),
    DecorationNotAllowedOnInstruction(DecorationNotAllowedOnInstruction),
    InvalidFloatTypeBitWidth(InvalidFloatTypeBitWidth),
    InvalidIntegerType(InvalidIntegerType),
}

pub(crate) type TranslationResult<T> = Result<T, TranslationError>;
