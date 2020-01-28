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

mod annotations;
mod capability;
mod constants;
mod debug_module_processed;
mod debug_names;
mod debug_strings_sources;
mod entry_point;
mod execution_mode;
mod ext_inst_import;
mod extension;
mod memory_model;
mod types;
mod unimplemented_instructions;
mod variables;

use crate::errors::InvalidSPIRVInstructionInSection;
use crate::errors::SPIRVIdAlreadyDefined;
use crate::errors::SPIRVIdNotDefined;
use crate::parse::annotations::TranslationStateParsedAnnotations;
use crate::types::SPIRVType;
use crate::values::SPIRVValue;
use crate::SPIRVInstructionsLocation;
use crate::TranslationResult;
use crate::TranslationStateBase;
use spirv_id_map::IdMap;
use spirv_parser::IdRef;
use spirv_parser::IdResult;
use spirv_parser::Instruction;

decl_translation_state! {
    pub(crate) struct TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i> {
        base: annotations::TranslationStateParsedAnnotations<'g, 'i>,
        types: IdMap<IdRef, SPIRVType<'g>>,
        values: IdMap<IdRef, SPIRVValue<'g>>,
    }
}

impl<'g, 'i> TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i> {
    fn define_value(
        &mut self,
        id_result: IdResult,
        ty: impl Into<SPIRVValue<'g>>,
    ) -> TranslationResult<()> {
        if let spirv_id_map::Vacant(entry) = self.values.entry(id_result.0)? {
            entry.insert(ty.into());
            Ok(())
        } else {
            Err(SPIRVIdAlreadyDefined { id_result }.into())
        }
    }
    pub(crate) fn get_value(&self, type_id: IdRef) -> TranslationResult<&SPIRVValue<'g>> {
        self.values
            .get(type_id)?
            .ok_or_else(|| SPIRVIdNotDefined { id: type_id }.into())
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsingTypesConstantsAndGlobals<'g, 'i> {
        base: TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsedTypesConstantsAndGlobals<'g, 'i> {
        base: TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsingFunctionBodies<'g, 'i> {
        base: TranslationStateParsedTypesConstantsAndGlobals<'g, 'i>,
    }
}

impl<'g, 'i> TranslationStateParsedAnnotations<'g, 'i> {
    pub(crate) fn parse_types_constants_globals_section(
        self,
    ) -> TranslationResult<TranslationStateParsedTypesConstantsAndGlobals<'g, 'i>> {
        let mut state = TranslationStateParsingTypesConstantsAndGlobals {
            base: TranslationStateParseBaseTypesConstantsAndGlobals {
                types: IdMap::new(&self.spirv_header),
                values: IdMap::new(&self.spirv_header),
                base: self,
            },
        };
        writeln!(
            state.debug_output,
            "parsing types/constants/globals section"
        )?;
        while let Some((instruction, location)) = state.get_instruction_and_location()? {
            if let Instruction::Function(_) = instruction {
                state.spirv_instructions_location = location;
                break;
            }
            instruction.parse_in_types_constants_globals_section(&mut state)?;
        }
        let TranslationStateParsingTypesConstantsAndGlobals { base } = state;
        Ok(TranslationStateParsedTypesConstantsAndGlobals { base })
    }
}

pub(crate) trait ParseInstruction: Clone + Into<Instruction> {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "types/constants/globals",
        }
        .into())
    }
    fn parse_in_function_body<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBodies<'g, 'i>,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "function body",
        }
        .into())
    }
}

impl ParseInstruction for Instruction {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_types_constants_globals_section(state))
    }
    fn parse_in_function_body<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBodies<'g, 'i>,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_function_body(state))
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
            .parse_execution_mode_section()?
            .parse_debug_strings_sources_section()?
            .parse_debug_names_section()?
            .parse_debug_module_processed_section()?
            .parse_annotations_section()?
            .parse_types_constants_globals_section()?;
        todo!()
        // TODO: function declarations section
        // TODO: function definitions section
    }
}
