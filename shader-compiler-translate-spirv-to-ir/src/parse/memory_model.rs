// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::parse::ParseInstruction;
use crate::MissingSPIRVOpMemoryModel;
use crate::SPIRVAddressingModelNotSupported;
use crate::SPIRVMemoryModelNotSupported;
use crate::TranslationResult;
use crate::TranslationState;
use spirv_parser::AddressingModel;
use spirv_parser::Instruction;
use spirv_parser::MemoryModel;
use spirv_parser::OpMemoryModel;

impl<'g, 'i> TranslationState<'g, 'i> {
    fn parse_memory_model_instruction(
        &mut self,
        instruction: &'i OpMemoryModel,
    ) -> TranslationResult<()> {
        let OpMemoryModel {
            addressing_model,
            memory_model,
        } = *instruction;
        match memory_model {
            MemoryModel::Simple | MemoryModel::GLSL450 | MemoryModel::Vulkan => {}
            _ => return Err(SPIRVMemoryModelNotSupported { memory_model }.into()),
        }
        match addressing_model {
            AddressingModel::Logical => Ok(()),
            _ => Err(SPIRVAddressingModelNotSupported { addressing_model }.into()),
        }
    }
    pub(crate) fn parse_memory_model_section(&mut self) -> TranslationResult<()> {
        writeln!(self.debug_output, "parsing OpMemoryModel section")?;
        if let Some((Instruction::MemoryModel(instruction), _)) =
            self.get_instruction_and_location()?
        {
            self.parse_memory_model_instruction(instruction)
        } else {
            Err(MissingSPIRVOpMemoryModel.into())
        }
    }
}

impl ParseInstruction for OpMemoryModel {}
