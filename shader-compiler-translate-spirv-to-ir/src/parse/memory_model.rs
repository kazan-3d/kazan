// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::parse::ext_inst_import::TranslationStateParsedExtInstImports;
use crate::parse::ParseInstruction;
use crate::MissingSPIRVOpMemoryModel;
use crate::SPIRVAddressingModelNotSupported;
use crate::SPIRVMemoryModelNotSupported;
use crate::TranslationResult;
use spirv_parser::AddressingModel;
use spirv_parser::Instruction;
use spirv_parser::MemoryModel;
use spirv_parser::OpMemoryModel;

decl_translation_state! {
    pub(crate) struct TranslationStateParsedMemoryModel<'g, 'i> {
        base: TranslationStateParsedExtInstImports<'g, 'i>,
    }
}

impl<'g, 'i> TranslationStateParsedMemoryModel<'g, 'i> {
    fn parse_memory_model_instruction(
        &mut self,
        instruction: &'i OpMemoryModel,
    ) -> TranslationResult<()> {
        let OpMemoryModel {
            addressing_model,
            memory_model,
        } = *instruction;
        match memory_model {
            MemoryModel::Simple(_) | MemoryModel::GLSL450(_) | MemoryModel::Vulkan(_) => {}
            _ => return Err(SPIRVMemoryModelNotSupported { memory_model }.into()),
        }
        match addressing_model {
            AddressingModel::Logical(_) => Ok(()),
            _ => Err(SPIRVAddressingModelNotSupported { addressing_model }.into()),
        }
    }
}

impl<'g, 'i> TranslationStateParsedExtInstImports<'g, 'i> {
    pub(crate) fn parse_memory_model_section(
        self,
    ) -> TranslationResult<TranslationStateParsedMemoryModel<'g, 'i>> {
        let mut state = TranslationStateParsedMemoryModel { base: self };
        writeln!(state.debug_output, "parsing OpMemoryModel section")?;
        if let Some((Instruction::MemoryModel(instruction), _)) =
            state.get_instruction_and_location()?
        {
            state.parse_memory_model_instruction(instruction)?;
            Ok(state)
        } else {
            Err(MissingSPIRVOpMemoryModel.into())
        }
    }
}

impl ParseInstruction for OpMemoryModel {}
