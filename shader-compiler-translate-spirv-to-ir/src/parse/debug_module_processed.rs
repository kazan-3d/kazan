// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    parse::{debug_names::TranslationStateParsedDebugNames, ParseInstruction},
    TranslationResult,
};
use spirv_parser::{Instruction, OpModuleProcessed};

decl_translation_state! {
    pub(crate) struct TranslationStateParsedDebugModuleProcessed<'g, 'i> {
        base: TranslationStateParsedDebugNames<'g, 'i>,
    }
}

impl<'g, 'i> TranslationStateParsedDebugNames<'g, 'i> {
    pub(crate) fn parse_debug_module_processed_section(
        self,
    ) -> TranslationResult<TranslationStateParsedDebugModuleProcessed<'g, 'i>> {
        let mut state = TranslationStateParsedDebugModuleProcessed { base: self };
        writeln!(
            state.debug_output,
            "parsing debug OpModuleProcessed section"
        )?;
        while let Some((instruction, location)) = state.next_instruction_and_location()? {
            match instruction {
                Instruction::ModuleProcessed(_) => {}
                _ => {
                    state.spirv_instructions_location = location;
                    break;
                }
            }
        }
        Ok(state)
    }
}

impl ParseInstruction for OpModuleProcessed {}
