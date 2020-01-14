// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::parse::extension::TranslationStateParsedExtensions;
use crate::parse::ParseInstruction;
use crate::SPIRVExtensionInstructionSetNotSupported;
use crate::TranslationResult;
use spirv_parser::ExtensionInstructionSet;
use spirv_parser::Instruction;
use spirv_parser::OpExtInstImport;

decl_translation_state! {
    pub(crate) struct TranslationStateParsedExtInstImports<'g, 'i> {
        base: TranslationStateParsedExtensions<'g, 'i>,
    }
}

impl<'g, 'i> TranslationStateParsedExtInstImports<'g, 'i> {
    fn parse_ext_inst_import_instruction(
        &mut self,
        instruction: &'i OpExtInstImport,
    ) -> TranslationResult<()> {
        let OpExtInstImport {
            id_result: _id_result,
            name,
        } = instruction;
        match ExtensionInstructionSet::from(&**name) {
            ExtensionInstructionSet::GLSLStd450 | ExtensionInstructionSet::OpenCLStd => Ok(()),
            ExtensionInstructionSet::Other(name) => {
                Err(SPIRVExtensionInstructionSetNotSupported { name }.into())
            }
        }
    }
}

impl<'g, 'i> TranslationStateParsedExtensions<'g, 'i> {
    pub(crate) fn parse_ext_inst_import_section(
        self,
    ) -> TranslationResult<TranslationStateParsedExtInstImports<'g, 'i>> {
        let mut state = TranslationStateParsedExtInstImports { base: self };
        writeln!(state.debug_output, "parsing OpExtInstImport section")?;
        while let Some((instruction, location)) = state.get_instruction_and_location()? {
            if let Instruction::ExtInstImport(instruction) = instruction {
                state.parse_ext_inst_import_instruction(instruction)?;
            } else {
                state.spirv_instructions_location = location;
                break;
            }
        }
        Ok(state)
    }
}

impl ParseInstruction for OpExtInstImport {}
