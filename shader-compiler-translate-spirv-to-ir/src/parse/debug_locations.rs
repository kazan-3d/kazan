// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    cfg::{CFGBlockId, TerminationInstruction},
    errors::TranslationResult,
    parse::{
        functions::TranslationStateParsingFunctionBody,
        translate_structure_tree::TranslationStateParsingFunctionBodyBlock, ParseInstruction,
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
        location: SPIRVInstructionLocation<'i>,
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
                Instruction::NoLine(_) => current_debug_location = None,
                ref instruction if TerminationInstruction::is_in_subset(instruction) => {
                    current_debug_location = None;
                }
                _ => {}
            }
            self.debug_locations.push(current_debug_location);
        }
        Ok(self.debug_locations.get(location.index).copied().flatten())
    }
    pub(crate) fn get_current_debug_location(
        &mut self,
    ) -> TranslationResult<Option<Interned<'g, Location<'g>>>> {
        self.get_debug_location(self.spirv_instructions_current_location.clone())
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
    fn parse_in_function_body_prepass<'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        state.get_debug_string(self.file)?;
        Ok(())
    }
    fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
        block_id: CFGBlockId,
    ) -> TranslationResult<()> {
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
    fn parse_in_function_body_prepass<'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Ok(())
    }
    fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Ok(())
    }
}
