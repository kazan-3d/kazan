// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
use alloc::string::String;
use core::fmt;

#[derive(Debug)]
pub struct InvalidSPIRVInstructionInSection {
    pub instruction: spirv_parser::Instruction,
    pub section_name: &'static str,
}

impl fmt::Display for InvalidSPIRVInstructionInSection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "invalid SPIR-V instruction in \'{}\' section:\n{}",
            self.section_name, self.instruction
        )
    }
}

#[derive(Debug)]
pub struct SpecializationResolutionFailed;

impl fmt::Display for SpecializationResolutionFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("shader specialization failed")
    }
}

#[derive(Debug)]
pub struct SPIRVExtensionNotSupported {
    pub name: String,
}

impl fmt::Display for SPIRVExtensionNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SPIR-V extension \"{:?}\" not supported", self.name)
    }
}

#[derive(Debug)]
pub struct SPIRVExtensionInstructionSetNotSupported {
    pub name: String,
}

impl fmt::Display for SPIRVExtensionInstructionSetNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SPIR-V extension instruction set \"{:?}\" not supported",
            self.name
        )
    }
}

#[derive(Debug)]
pub struct SPIRVCapabilityNotSupported {
    pub capability: spirv_parser::Capability,
}

impl fmt::Display for SPIRVCapabilityNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SPIR-V capability \"{:?}\" not supported",
            self.capability
        )
    }
}

#[derive(Debug)]
pub struct SPIRVMemoryModelNotSupported {
    pub memory_model: spirv_parser::MemoryModel,
}

impl fmt::Display for SPIRVMemoryModelNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SPIR-V memory model \"{:?}\" not supported",
            self.memory_model
        )
    }
}

#[derive(Debug)]
pub struct MissingSPIRVOpMemoryModel;

impl fmt::Display for MissingSPIRVOpMemoryModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "missing SPIR-V OpMemoryModel instruction")
    }
}

#[derive(Debug)]
pub struct SPIRVAddressingModelNotSupported {
    pub addressing_model: spirv_parser::AddressingModel,
}

impl fmt::Display for SPIRVAddressingModelNotSupported {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "SPIR-V addressing model \"{:?}\" not supported",
            self.addressing_model
        )
    }
}

#[derive(Debug)]
pub struct DuplicateSPIRVEntryPoint {
    pub name: String,
    pub execution_model: spirv_parser::ExecutionModel,
}

impl fmt::Display for DuplicateSPIRVEntryPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "duplicate SPIR-V entry point with name \"{:?}\" and execution model {:?}",
            self.name, self.execution_model
        )
    }
}

#[derive(Debug)]
pub struct MatchingSPIRVEntryPointNotFound {
    pub name: String,
    pub execution_model: spirv_parser::ExecutionModel,
}

impl fmt::Display for MatchingSPIRVEntryPointNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "matching SPIR-V entry point with name \"{:?}\" and execution model {:?} not found",
            self.name, self.execution_model
        )
    }
}

#[derive(Debug)]
pub struct UnsupportedSPIRVExecutionMode {
    pub execution_mode: spirv_parser::ExecutionMode,
}

impl fmt::Display for UnsupportedSPIRVExecutionMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "unsupported SPIR-V execution mode: {:?}",
            self.execution_mode
        )
    }
}

#[derive(Debug)]
pub struct DuplicateSPIRVLocalSize;

impl fmt::Display for DuplicateSPIRVLocalSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "duplicate SPIR-V LocalSize annotation for entry point")
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
}

pub(crate) type TranslationResult<T> = Result<T, TranslationError>;
