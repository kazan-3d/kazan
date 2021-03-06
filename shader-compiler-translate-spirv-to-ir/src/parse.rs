// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

#[macro_use]
pub(crate) mod instruction_dispatch;

mod annotations;
mod capability;
mod composite_instructions;
mod constants;
mod conversion_instructions;
mod debug_locations;
mod debug_module_processed;
mod debug_names;
mod debug_strings_sources;
mod entry_point;
mod execution_mode;
mod ext_inst_import;
mod extension;
mod functions;
mod memory_instructions;
mod memory_model;
mod translate_structure_tree;
mod types;
mod unimplemented_instructions;
mod variables;

use crate::{
    cfg::CFGBlockId,
    errors::{InvalidSPIRVInstructionInSection, SPIRVIdAlreadyDefined, SPIRVIdNotDefined},
    parse::{
        annotations::TranslationStateParsedAnnotations,
        functions::{TranslationStateParsedFunctions, TranslationStateParsingFunctionBody},
        translate_structure_tree::TranslationStateParsingFunctionBodyBlock,
    },
    types::SPIRVType,
    values::SPIRVValue,
    SPIRVInstructionLocation, TranslationResult, TranslationStateBase,
};
use alloc::vec::Vec;
use shader_compiler_ir::{
    BuiltInInterfaceVariableAttributes, InterfaceBlock, InterfaceBlockMember, InterfaceVariable,
    UserInterfaceVariableAttributes,
};
use spirv_id_map::IdMap;
use spirv_parser::{IdRef, IdResult, Instruction};

#[derive(Default)]
pub(crate) struct ModuleState<'g> {
    pub(crate) built_in_inputs: Vec<InterfaceVariable<'g, BuiltInInterfaceVariableAttributes>>,
    pub(crate) built_in_outputs: Vec<InterfaceVariable<'g, BuiltInInterfaceVariableAttributes>>,
    pub(crate) user_inputs: Vec<InterfaceBlockMember<'g, UserInterfaceVariableAttributes>>,
    pub(crate) user_outputs: Vec<InterfaceBlockMember<'g, UserInterfaceVariableAttributes>>,
    pub(crate) invocation_global_variables: Vec<shader_compiler_ir::Variable<'g>>,
}

decl_translation_state! {
    pub(crate) struct TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i> {
        base: annotations::TranslationStateParsedAnnotations<'g, 'i>,
        types: IdMap<IdRef, SPIRVType<'g>>,
        values: IdMap<IdRef, SPIRVValue<'g>>,
        debug_locations: Vec<Option<shader_compiler_ir::Interned<'g, shader_compiler_ir::Location<'g>>>>,
        module_state: ModuleState<'g>,
    }
}

impl<'g, 'i> TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i> {
    fn define_value(
        &mut self,
        id_result: IdResult,
        v: impl Into<SPIRVValue<'g>>,
    ) -> TranslationResult<()> {
        if let spirv_id_map::Vacant(entry) = self.values.entry(id_result.0)? {
            entry.insert(v.into());
            Ok(())
        } else {
            Err(SPIRVIdAlreadyDefined { id_result }.into())
        }
    }
    pub(crate) fn get_value(&self, value_id: IdRef) -> TranslationResult<&SPIRVValue<'g>> {
        self.values
            .get(value_id)?
            .ok_or_else(|| SPIRVIdNotDefined { id: value_id }.into())
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

impl<'g, 'i> TranslationStateParsedAnnotations<'g, 'i> {
    pub(crate) fn parse_types_constants_globals_section(
        self,
    ) -> TranslationResult<TranslationStateParsedTypesConstantsAndGlobals<'g, 'i>> {
        let mut state = TranslationStateParsingTypesConstantsAndGlobals {
            base: TranslationStateParseBaseTypesConstantsAndGlobals {
                types: IdMap::new(&self.spirv_header),
                values: IdMap::new(&self.spirv_header),
                debug_locations: Vec::with_capacity(self.spirv_instructions.len()),
                base: self,
                module_state: ModuleState {
                    built_in_inputs: Vec::new(),
                    built_in_outputs: Vec::new(),
                    user_inputs: Vec::new(),
                    user_outputs: Vec::new(),
                    invocation_global_variables: Vec::new(),
                },
            },
        };
        writeln!(
            state.debug_output,
            "parsing types/constants/globals section"
        )?;
        while let Some((instruction, location)) = state.next_instruction_and_location()? {
            if let Instruction::Function(_) = instruction {
                state.set_spirv_instructions_location(location);
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
    fn parse_in_function_body_prepass<'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "function body (prepass)",
        }
        .into())
    }
    fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
        block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Err(InvalidSPIRVInstructionInSection {
            instruction: self.clone().into(),
            section_name: "function body (reachable)",
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
    fn parse_in_function_body_prepass<'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_function_body_prepass(state, block_id))
    }
    fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
        block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        instruction_dispatch!(self, v, v.parse_in_function_body_reachable(state, block_id))
    }
}

impl<'g, 'i> TranslationStateBase<'g, 'i> {
    fn next_instruction_and_location(
        &mut self,
    ) -> TranslationResult<Option<(&'i Instruction, SPIRVInstructionLocation<'i>)>> {
        self.spirv_instructions_current_location = self.spirv_instructions_next_location.clone();
        write!(
            self.debug_output,
            "{:?}",
            self.spirv_instructions_current_location
        )?;
        Ok(self
            .spirv_instructions_next_location
            .next()
            .map(|(instruction, location)| (&instruction.instruction, location)))
    }
    fn next_instruction(&mut self) -> TranslationResult<Option<&'i Instruction>> {
        Ok(self
            .next_instruction_and_location()?
            .map(|(instruction, _)| instruction))
    }
    pub(crate) fn parse(self) -> TranslationResult<TranslationStateParsedFunctions<'g, 'i>> {
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
            .parse_types_constants_globals_section()?
            .parse_functions_section()
    }
}
