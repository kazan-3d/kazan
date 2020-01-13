// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::parse::ParseInstruction;
use crate::SPIRVCapabilityNotSupported;
use crate::TranslationResult;
use crate::TranslationState;
use spirv_parser::Capability;
use spirv_parser::Instruction;
use spirv_parser::OpCapability;

#[derive(Copy, Clone, Debug)]
struct CapabilityAndDependencies<'a> {
    capability: Capability,
    dependencies: &'a [Capability],
}

macro_rules! supported_capabilities_and_dependencies {
    ($($cap:ident => [$($dep:ident),*],)+) => {
        const SUPPORTED_CAPABILITIES_AND_DEPENDENCIES: &[CapabilityAndDependencies] =
            &[$(
                CapabilityAndDependencies {
                    capability: Capability::$cap,
                    dependencies: &[$(
                        Capability::$dep,
                    )*],
                },
            )+];
    };
}

supported_capabilities_and_dependencies! {
    Matrix => [],
    Shader => [Matrix],
    Float16 => [],
    Float64 => [],
    Int64 => [],
    Int64Atomics => [Int64],
    Int16 => [],
    Int8 => [],
}

impl<'g, 'i> TranslationState<'g, 'i> {
    fn parse_capability_instruction(
        &mut self,
        instruction: &'i OpCapability,
    ) -> TranslationResult<()> {
        let OpCapability { capability } = *instruction;
        for &CapabilityAndDependencies {
            capability: supported_capability,
            dependencies,
        } in SUPPORTED_CAPABILITIES_AND_DEPENDENCIES
        {
            if capability == supported_capability {
                self.enabled_capabilities.insert(capability);
                writeln!(self.debug_output, "added capability: {:?}", capability)?;
                self.enabled_capabilities
                    .extend(dependencies.iter().copied());
                writeln!(
                    self.debug_output,
                    "added dependency capabilities: {:?}",
                    dependencies
                )?;
                return Ok(());
            }
        }
        Err(SPIRVCapabilityNotSupported { capability }.into())
    }
    pub(crate) fn parse_capability_section(&mut self) -> TranslationResult<()> {
        writeln!(self.debug_output, "parsing OpCapability section")?;
        while let Some((instruction, location)) = self.get_instruction_and_location()? {
            if let Instruction::Capability(instruction) = instruction {
                self.parse_capability_instruction(instruction)?;
            } else {
                self.spirv_instructions_location = location;
                break;
            }
        }
        Ok(())
    }
}

impl ParseInstruction for OpCapability {}
