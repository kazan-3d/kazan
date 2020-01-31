// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    parse::ParseInstruction, SPIRVCapabilityNotSupported, TranslationResult, TranslationStateBase,
};
use hashbrown::HashSet;
use spirv_parser::{Capability, Instruction, OpCapability};

decl_translation_state! {
    pub(crate) struct TranslationStateParsedCapabilities<'g, 'i> {
        base: TranslationStateBase<'g, 'i>,
        enabled_capabilities: HashSet<Capability>,
    }
}

#[derive(Copy, Clone, Debug)]
struct CapabilityAndDependencies<'a> {
    capability: Capability,
    dependencies: &'a [Capability],
}

macro_rules! supported_capabilities_and_dependencies {
    ($($cap:ident => [$($dep:ident),*],)+) => {
        fn call_with_supported_capabilities_and_dependencies<R, T: FnOnce(&[CapabilityAndDependencies]) -> R>(f: T) -> R {
            f(&[$(
                CapabilityAndDependencies {
                    capability: Capability::$cap(Default::default()),
                    dependencies: &[$(
                        Capability::$dep(Default::default()),
                    )*],
                },
            )+])
        }
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

impl<'g, 'i> TranslationStateParsedCapabilities<'g, 'i> {
    fn parse_capability_instruction(
        &mut self,
        instruction: &'i OpCapability,
    ) -> TranslationResult<()> {
        let OpCapability { capability } = *instruction;
        call_with_supported_capabilities_and_dependencies(
            |supported_capabilities_and_dependencies| {
                for &CapabilityAndDependencies {
                    capability: supported_capability,
                    ref dependencies,
                } in supported_capabilities_and_dependencies
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
            },
        )
    }
}

impl<'g, 'i> TranslationStateBase<'g, 'i> {
    pub(crate) fn parse_capability_section(
        self,
    ) -> TranslationResult<TranslationStateParsedCapabilities<'g, 'i>> {
        let mut state = TranslationStateParsedCapabilities {
            base: self,
            enabled_capabilities: HashSet::new(),
        };
        writeln!(state.debug_output, "parsing OpCapability section")?;
        while let Some((instruction, location)) = state.get_instruction_and_location()? {
            if let Instruction::Capability(instruction) = instruction {
                state.parse_capability_instruction(instruction)?;
            } else {
                state.spirv_instructions_location = location;
                break;
            }
        }
        Ok(state)
    }
}

impl ParseInstruction for OpCapability {}
