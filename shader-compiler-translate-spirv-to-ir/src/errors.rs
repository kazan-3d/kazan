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
}

pub(crate) type TranslationResult<T> = Result<T, TranslationError>;
