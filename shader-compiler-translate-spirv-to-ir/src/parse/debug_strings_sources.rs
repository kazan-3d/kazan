// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    errors::{SPIRVIdAlreadyDefined, SPIRVIdNotDefined},
    parse::{execution_mode::TranslationStateParsedExecutionModes, ParseInstruction},
    TranslationResult,
};
use shader_compiler_ir::{Internable, Interned};
use spirv_id_map::{IdMap, Vacant};
use spirv_parser::{IdRef, Instruction, OpSource, OpSourceContinued, OpSourceExtension, OpString};

decl_translation_state! {
    pub(crate) struct TranslationStateParsedDebugStringsSources<'g, 'i> {
        base: TranslationStateParsedExecutionModes<'g, 'i>,
        debug_strings: IdMap<IdRef, Interned<'g, str>>,
    }
}

impl<'g, 'i> TranslationStateParsedDebugStringsSources<'g, 'i> {
    fn parse_string_instruction(&mut self, instruction: &'i OpString) -> TranslationResult<()> {
        let OpString {
            id_result,
            ref string,
        } = *instruction;
        let string = string.intern(self.global_state);
        if let Vacant(entry) = self.debug_strings.entry(id_result.0)? {
            entry.insert(string);
            Ok(())
        } else {
            Err(SPIRVIdAlreadyDefined { id_result }.into())
        }
    }
    pub(crate) fn get_debug_string(&self, id: IdRef) -> TranslationResult<Interned<'g, str>> {
        self.debug_strings
            .get(id)?
            .copied()
            .ok_or_else(|| SPIRVIdNotDefined { id }.into())
    }
}

impl<'g, 'i> TranslationStateParsedExecutionModes<'g, 'i> {
    pub(crate) fn parse_debug_strings_sources_section(
        self,
    ) -> TranslationResult<TranslationStateParsedDebugStringsSources<'g, 'i>> {
        let mut state = TranslationStateParsedDebugStringsSources {
            debug_strings: IdMap::new(&self.spirv_header),
            base: self,
        };
        writeln!(state.debug_output, "parsing debug strings/sources section")?;
        while let Some((instruction, location)) = state.next_instruction_and_location()? {
            match instruction {
                Instruction::String(instruction) => state.parse_string_instruction(instruction)?,
                Instruction::Source(_)
                | Instruction::SourceExtension(_)
                | Instruction::SourceContinued(_) => {}
                _ => {
                    state.spirv_instructions_location = location;
                    break;
                }
            }
        }
        Ok(state)
    }
}

impl ParseInstruction for OpString {}
impl ParseInstruction for OpSource {}
impl ParseInstruction for OpSourceExtension {}
impl ParseInstruction for OpSourceContinued {}
