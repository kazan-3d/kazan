// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    parse::{memory_model::TranslationStateParsedMemoryModel, ParseInstruction},
    DuplicateSPIRVEntryPoint, MatchingSPIRVEntryPointNotFound, TranslationResult,
};
use alloc::string::ToString;
use hashbrown::HashSet;
use spirv_parser::{IdRef, Instruction, OpEntryPoint};

struct EntryPoint {
    id: IdRef,
    interface_variables: HashSet<IdRef>,
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsedEntryPoints<'g, 'i> {
        base: TranslationStateParsedMemoryModel<'g, 'i>,
        entry_point_id: IdRef,
        entry_point_interface_variables: HashSet<IdRef>,
    }
}

decl_translation_state! {
    struct TranslationStateParsingEntryPoints<'g, 'i> {
        base: TranslationStateParsedMemoryModel<'g, 'i>,
        entry_point: Option<EntryPoint>,
    }
}

impl<'g, 'i> TranslationStateParsingEntryPoints<'g, 'i> {
    fn parse_entry_point_instruction(
        &mut self,
        instruction: &'i OpEntryPoint,
    ) -> TranslationResult<()> {
        let OpEntryPoint {
            execution_model,
            entry_point,
            ref name,
            ref interface,
        } = *instruction;
        if name == self.entry_point_name && execution_model == self.entry_point_execution_model {
            if self.entry_point.is_some() {
                return Err(DuplicateSPIRVEntryPoint {
                    name: name.clone(),
                    execution_model,
                }
                .into());
            }
            self.entry_point = Some(EntryPoint {
                id: entry_point,
                interface_variables: interface.iter().copied().collect(),
            });
        }
        Ok(())
    }
}

impl<'g, 'i> TranslationStateParsedMemoryModel<'g, 'i> {
    pub(crate) fn parse_entry_point_section(
        self,
    ) -> TranslationResult<TranslationStateParsedEntryPoints<'g, 'i>> {
        let mut state = TranslationStateParsingEntryPoints {
            base: self,
            entry_point: None,
        };
        writeln!(state.debug_output, "parsing OpEntryPoint section")?;
        while let Some((instruction, location)) = state.next_instruction_and_location()? {
            if let Instruction::EntryPoint(instruction) = instruction {
                state.parse_entry_point_instruction(instruction)?;
            } else {
                state.set_spirv_instructions_location(location);
                break;
            }
        }
        match state {
            TranslationStateParsingEntryPoints {
                base,
                entry_point:
                    Some(EntryPoint {
                        id: entry_point_id,
                        interface_variables: entry_point_interface_variables,
                    }),
            } => Ok(TranslationStateParsedEntryPoints {
                base,
                entry_point_id,
                entry_point_interface_variables,
            }),
            _ => Err(MatchingSPIRVEntryPointNotFound {
                name: state.entry_point_name.to_string(),
                execution_model: state.entry_point_execution_model,
            }
            .into()),
        }
    }
}

impl ParseInstruction for OpEntryPoint {}
