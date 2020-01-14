// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::parse::capability::TranslationStateParsedCapabilities;
use crate::parse::ParseInstruction;
use crate::SPIRVExtensionNotSupported;
use crate::TranslationResult;
use spirv_parser::Instruction;
use spirv_parser::OpExtension;

decl_translation_state! {
    pub(crate) struct TranslationStateParsedExtensions<'g, 'i> {
        base: TranslationStateParsedCapabilities<'g, 'i>,
    }
}

impl<'g, 'i> TranslationStateParsedExtensions<'g, 'i> {
    fn parse_extension_instruction(
        &mut self,
        instruction: &'i OpExtension,
    ) -> TranslationResult<()> {
        let OpExtension { name } = instruction;
        match &**name {
            _ => Err(SPIRVExtensionNotSupported { name: name.clone() }.into()),
        }
    }
}

impl<'g, 'i> TranslationStateParsedCapabilities<'g, 'i> {
    pub(crate) fn parse_extension_section(
        self,
    ) -> TranslationResult<TranslationStateParsedExtensions<'g, 'i>> {
        let mut state = TranslationStateParsedExtensions { base: self };
        writeln!(state.debug_output, "parsing OpExtension section")?;
        while let Some((instruction, location)) = state.get_instruction_and_location()? {
            if let Instruction::Extension(instruction) = instruction {
                state.parse_extension_instruction(instruction)?;
            } else {
                state.spirv_instructions_location = location;
                break;
            }
        }
        Ok(state)
    }
}

impl ParseInstruction for OpExtension {}
