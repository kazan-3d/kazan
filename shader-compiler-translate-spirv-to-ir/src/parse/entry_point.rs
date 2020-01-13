// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::parse::ParseInstruction;
use crate::DuplicateSPIRVEntryPoint;
use crate::EntryPoint;
use crate::TranslationResult;
use crate::TranslationState;
use spirv_parser::Instruction;
use spirv_parser::OpEntryPoint;

impl<'g, 'i> TranslationState<'g, 'i> {
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
                entry_point_id: entry_point,
                interface_variables: interface.iter().copied().collect(),
            });
        }
        Ok(())
    }
    pub(crate) fn parse_entry_point_section(&mut self) -> TranslationResult<()> {
        writeln!(self.debug_output, "parsing OpEntryPoint section")?;
        while let Some((instruction, location)) = self.get_instruction_and_location()? {
            if let Instruction::EntryPoint(instruction) = instruction {
                self.parse_entry_point_instruction(instruction)?;
            } else {
                self.spirv_instructions_location = location;
                break;
            }
        }
        Ok(())
    }
}

impl ParseInstruction for OpEntryPoint {}
