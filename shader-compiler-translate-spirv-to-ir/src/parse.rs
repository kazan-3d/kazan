// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

#[macro_use]
pub(crate) mod instruction_dispatch;

mod capability;
mod ext_inst_import;
mod extension;
mod unimplemented_instructions;

use crate::errors::InvalidSPIRVInstructionInSection;
use crate::SPIRVInstructionsLocation;
use crate::TranslationResult;
use crate::TranslationState;
use spirv_parser::*;

pub(crate) trait ParseInstruction: Clone + Into<Instruction> {
    fn parse_in_entry_point_section<'g, 'i>(
        &'i self,
        _state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "OpEntryPoint",
        }
        .into())
    }
    fn parse_in_execution_mode_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "OpExecutionMode",
        }
        .into())
    }
    fn parse_in_debug_strings_sources_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "debug strings/sources",
        }
        .into())
    }
    fn parse_in_debug_names_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "debug names",
        }
        .into())
    }
    fn parse_in_module_processed_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "OpModuleProcessed",
        }
        .into())
    }
    fn parse_in_annotations_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "annotations",
        }
        .into())
    }
    fn parse_in_types_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "types",
        }
        .into())
    }
}

impl ParseInstruction for Instruction {
    fn parse_in_entry_point_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_entry_point_section(state))
    }
    fn parse_in_execution_mode_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_execution_mode_section(state))
    }
    fn parse_in_debug_strings_sources_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_debug_strings_sources_section(state))
    }
    fn parse_in_debug_names_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_debug_names_section(state))
    }
    fn parse_in_module_processed_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_module_processed_section(state))
    }
    fn parse_in_annotations_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_annotations_section(state))
    }
    fn parse_in_types_section<'g, 'i>(
        &'i self,
        state: &mut TranslationState<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_types_section(state))
    }
}

impl<'g, 'i> TranslationState<'g, 'i> {
    fn get_instruction_and_location(
        &mut self,
    ) -> TranslationResult<Option<(&'i Instruction, SPIRVInstructionsLocation<'i>)>> {
        let location = self.spirv_instructions_location.clone();
        if let Some((index, instruction)) = self.spirv_instructions_location.0.next() {
            write!(self.debug_output, "{:05}: {}", index, instruction)?;
            Ok(Some((instruction, location)))
        } else {
            Ok(None)
        }
    }
    pub(crate) fn parse(&mut self) -> TranslationResult<()> {
        self.parse_capability_section()?;
        self.parse_extension_section()?;
        self.parse_ext_inst_import_section()?;
        todo!()
        // TODO: memory model section
        // TODO: entry point section
        // TODO: execution mode section
        // TODO: debug strings/sources section
        // TODO: debug names section
        // TODO: module processed section
        // TODO: annotations section
        // TODO: types section
        // TODO: function declarations section
        // TODO: function definitions section
    }
}
