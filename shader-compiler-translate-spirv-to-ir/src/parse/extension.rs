// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::parse::ParseInstruction;
use crate::SPIRVExtensionNotSupported;
use crate::TranslationResult;
use crate::TranslationState;
use spirv_parser::Instruction;
use spirv_parser::OpExtension;

impl<'g, 'i> TranslationState<'g, 'i> {
    fn parse_extension_instruction(
        &mut self,
        instruction: &'i OpExtension,
    ) -> TranslationResult<()> {
        let OpExtension { name } = instruction;
        match &**name {
            _ => Err(SPIRVExtensionNotSupported { name: name.clone() }.into()),
        }
    }
    pub(crate) fn parse_extension_section(&mut self) -> TranslationResult<()> {
        writeln!(self.debug_output, "parsing OpExtension section")?;
        while let Some((instruction, location)) = self.get_instruction_and_location()? {
            if let Instruction::Extension(instruction) = instruction {
                self.parse_extension_instruction(instruction)?;
            } else {
                self.spirv_instructions_location = location;
                break;
            }
        }
        Ok(())
    }
}

impl ParseInstruction for OpExtension {}
