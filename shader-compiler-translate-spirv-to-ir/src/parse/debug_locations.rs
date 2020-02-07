// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::cfg::CFGBlockId;
use crate::{
    errors::TranslationResult,
    parse::{
        functions::TranslationStateParsingFunctionBody, ParseInstruction,
        TranslationStateParseBaseTypesConstantsAndGlobals,
        TranslationStateParsingTypesConstantsAndGlobals,
    },
    SPIRVInstructionLocation,
};
use shader_compiler_ir::{Internable, Interned, Location};
use spirv_parser::{Instruction, OpLine, OpNoLine};

impl<'g, 'i> TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i> {
    pub(crate) fn get_debug_location(
        &mut self,
        location: SPIRVInstructionLocation,
    ) -> TranslationResult<Option<Interned<'g, Location<'g>>>> {
        if location.index >= self.spirv_instructions.len() {
            return Ok(None);
        }
        let mut current_debug_location = self.debug_locations.last().copied().flatten();
        for index in self.debug_locations.len()..=location.index {
            match self.spirv_instructions[index] {
                Instruction::Line(OpLine { file, line, column }) => {
                    let file = self.get_debug_string(file)?;
                    current_debug_location =
                        Some(Location { file, line, column }.intern(self.global_state));
                }
                Instruction::NoLine(_)
                | Instruction::Branch(_)
                | Instruction::BranchConditional(_)
                | Instruction::Switch32(_)
                | Instruction::Switch64(_)
                | Instruction::Kill(_)
                | Instruction::Return(_)
                | Instruction::ReturnValue(_)
                | Instruction::Unreachable(_) => current_debug_location = None,
                _ => {}
            }
            self.debug_locations.push(current_debug_location);
        }
        Ok(self.debug_locations.get(location.index).copied().flatten())
    }
}

impl ParseInstruction for OpLine {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        state.get_debug_string(self.file)?;
        Ok(())
    }
    fn parse_in_function_body_generic<'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        state.get_debug_string(self.file)?;
        Ok(())
    }
}

impl ParseInstruction for OpNoLine {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        Ok(())
    }
    fn parse_in_function_body_generic<'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Ok(())
    }
}
