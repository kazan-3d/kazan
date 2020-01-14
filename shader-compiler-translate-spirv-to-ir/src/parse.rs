// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

macro_rules! decl_translation_state {
    (
        $vis:vis struct $state_name:ident<$g:lifetime, $i:lifetime> {
            base: $base_type:ty,
            $(
                $member_name:ident: $member_type:ty,
            )*
        }
    ) => {
        $vis struct $state_name<$g, $i> {
            $vis base: $base_type,
            $(
                $vis $member_name: $member_type,
            )*
        }

        impl<$g, $i> core::ops::Deref for $state_name<$g, $i> {
            type Target = $base_type;
            fn deref(&self) -> &Self::Target {
                &self.base
            }
        }

        impl<$g, $i> core::ops::DerefMut for $state_name<$g, $i> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.base
            }
        }
    };
}

#[macro_use]
pub(crate) mod instruction_dispatch;

mod capability;
mod entry_point;
mod execution_mode;
mod ext_inst_import;
mod extension;
mod memory_model;
mod unimplemented_instructions;

use crate::errors::InvalidSPIRVInstructionInSection;
use crate::SPIRVInstructionsLocation;
use crate::TranslationResult;
use crate::TranslationStateBase;
use spirv_parser::*;

pub(crate) trait ParseInstruction: Clone + Into<Instruction> {
    fn parse_in_debug_strings_sources_section<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "debug strings/sources",
        }
        .into())
    }
    fn parse_in_debug_names_section<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "debug names",
        }
        .into())
    }
    fn parse_in_module_processed_section<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "OpModuleProcessed",
        }
        .into())
    }
    fn parse_in_annotations_section<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "annotations",
        }
        .into())
    }
    fn parse_in_types_section<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "types",
        }
        .into())
    }
}

impl ParseInstruction for Instruction {
    fn parse_in_debug_strings_sources_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_debug_strings_sources_section(state))
    }
    fn parse_in_debug_names_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_debug_names_section(state))
    }
    fn parse_in_module_processed_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_module_processed_section(state))
    }
    fn parse_in_annotations_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_annotations_section(state))
    }
    fn parse_in_types_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_types_section(state))
    }
}

impl<'g, 'i> TranslationStateBase<'g, 'i> {
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
    pub(crate) fn parse(self) -> TranslationResult<()> {
        self.parse_capability_section()?
            .parse_extension_section()?
            .parse_ext_inst_import_section()?
            .parse_memory_model_section()?
            .parse_entry_point_section()?
            .parse_execution_mode_section()?;
        todo!()
        // TODO: debug strings/sources section
        // TODO: debug names section
        // TODO: module processed section
        // TODO: annotations section
        // TODO: types section
        // TODO: function declarations section
        // TODO: function definitions section
    }
}
