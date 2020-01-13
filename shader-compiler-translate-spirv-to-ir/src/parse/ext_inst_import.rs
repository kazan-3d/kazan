// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::parse::ParseInstruction;
use crate::SPIRVExtensionInstructionSetNotSupported;
use crate::TranslationResult;
use crate::TranslationState;
use spirv_parser::ExtensionInstructionSet;
use spirv_parser::Instruction;
use spirv_parser::OpExtInstImport;

impl<'g, 'i> TranslationState<'g, 'i> {
    fn parse_ext_inst_import_instruction(
        &mut self,
        instruction: &'i OpExtInstImport,
    ) -> TranslationResult<()> {
        let OpExtInstImport { id_result, name } = instruction;
        match ExtensionInstructionSet::from(&**name) {
            ExtensionInstructionSet::GLSLStd450 | ExtensionInstructionSet::OpenCLStd => Ok(()),
            ExtensionInstructionSet::Other(name) => {
                Err(SPIRVExtensionInstructionSetNotSupported { name }.into())
            }
        }
    }
    pub(crate) fn parse_ext_inst_import_section(&mut self) -> TranslationResult<()> {
        writeln!(self.debug_output, "parsing OpExtInstImport section")?;
        while let Some((instruction, location)) = self.get_instruction_and_location()? {
            if let Instruction::ExtInstImport(instruction) = instruction {
                self.parse_ext_inst_import_instruction(instruction)?;
            } else {
                self.spirv_instructions_location = location;
                break;
            }
        }
        Ok(())
    }
}

impl ParseInstruction for OpExtInstImport {}
