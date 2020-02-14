// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    cfg::{CFGBlock, CFGBuilder, MergeInstruction, TerminationInstruction},
    decorations::DecorationClass,
    errors::{
        ConstAndPureAreNotAllowedTogether, DecorationNotAllowedOnInstruction,
        FunctionMustHaveABody, FunctionsFunctionTypeIsNotOpTypeFunction,
        FunctionsResultTypeMustMatchFunctionTypesReturnType,
        InlineAndDontInlineAreNotAllowedTogether, InstructionNotValidBeforeLabel,
        InvalidSPIRVInstructionInSection,
        MergeInstructionMustBeImmediatelyFollowedByTerminationInstruction,
        RelaxedPrecisionDecorationNotAllowed, SPIRVBlockMissingTerminationInstruction,
        SPIRVIdAlreadyDefined, SPIRVIdNotDefined, TooFewOpFunctionParameterInstructions,
        TooManyOpFunctionParameterInstructions, TranslationResult,
    },
    functions::{SPIRVFunction, SPIRVFunctionData},
    parse::{ParseInstruction, TranslationStateParsedTypesConstantsAndGlobals},
    types::GenericSPIRVType,
    SPIRVInstructionLocation, TranslatedSPIRVShader,
};
use alloc::vec::Vec;
use core::cell::RefCell;
use petgraph::visit::IntoNodeReferences;
use shader_compiler_ir::{
    Alignment, Block, DataPointerType, Function, FunctionHints, FunctionRef, FunctionSideEffects,
    Inhabited, InliningHint, InterfaceBlock, Module, StructSize, ValueDefinition,
};
use spirv_id_map::IdMap;
use spirv_parser::{
    FunctionControl, IdRef, IdResult, Instruction, OpFunction, OpFunctionEnd, OpFunctionParameter,
    OpLabel,
};

decl_translation_state! {
    pub(crate) struct TranslationStateParseFunctionsBase<'g, 'i> {
        base: TranslationStateParsedTypesConstantsAndGlobals<'g, 'i>,
        functions: IdMap<spirv_parser::IdRef, SPIRVFunction<'g, 'i>>,
        ir_functions: Vec<Function<'g>>,
    }
}

impl<'g, 'i> TranslationStateParseFunctionsBase<'g, 'i> {
    fn define_function(
        &mut self,
        id_result: IdResult,
        v: SPIRVFunction<'g, 'i>,
    ) -> TranslationResult<()> {
        if let spirv_id_map::Vacant(entry) = self.functions.entry(id_result.0)? {
            entry.insert(v);
            Ok(())
        } else {
            Err(SPIRVIdAlreadyDefined { id_result }.into())
        }
    }
    fn get_function(&mut self, function_id: IdRef) -> TranslationResult<&SPIRVFunction<'g, 'i>> {
        self.functions
            .get(function_id)?
            .ok_or_else(|| SPIRVIdNotDefined { id: function_id }.into())
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsingFunctionBody<'f, 'g, 'i> {
        base: TranslationStateParseFunctionsBase<'g, 'i>,
        function: &'f SPIRVFunction<'g, 'i>,
        local_variables: Vec<shader_compiler_ir::Variable<'g>>,
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsedFunctions<'g, 'i> {
        base: TranslationStateParseFunctionsBase<'g, 'i>,
    }
}

fn parse_cfg_block<'g, 'i>(
    state: &mut TranslationStateParseFunctionsBase<'g, 'i>,
    function_parameter_count: u32,
) -> TranslationResult<Option<CFGBlock<'g, 'i>>> {
    let (label_location, label_id) = loop {
        match state.next_instruction_and_location()? {
            Some((instruction @ Instruction::FunctionParameter(_), _)) => {
                return Err(TooManyOpFunctionParameterInstructions {
                    expected_count: function_parameter_count,
                    instruction: instruction.clone(),
                }
                .into());
            }
            Some((Instruction::Line(_), _)) | Some((Instruction::NoLine(_), _)) => {}
            Some((Instruction::FunctionEnd(_), _)) | None => {
                return Ok(None);
            }
            Some((&Instruction::Label(OpLabel { id_result }), location)) => {
                break (location, id_result.0)
            }
            Some((instruction, _)) => {
                return Err(InstructionNotValidBeforeLabel {
                    instruction: instruction.clone(),
                }
                .into());
            }
        }
    };
    let mut merge_location: Option<SPIRVInstructionLocation<'i>> = None;
    let termination_location = loop {
        match state.next_instruction_and_location()? {
            None | Some((Instruction::FunctionEnd(_), _)) | Some((Instruction::Label(_), _)) => {
                return Err(SPIRVBlockMissingTerminationInstruction { label_id }.into());
            }
            Some((instruction, location)) if TerminationInstruction::is_in_subset(instruction) => {
                break location;
            }
            Some((instruction, location)) if merge_location.is_some() => {
                return Err(
                    MergeInstructionMustBeImmediatelyFollowedByTerminationInstruction {
                        merge_instruction: merge_location
                            .expect("known to be some")
                            .get_instruction()
                            .expect("known to be non-empty")
                            .clone(),
                        instruction: instruction.clone(),
                    }
                    .into(),
                );
            }
            Some((instruction, location)) if MergeInstruction::is_in_subset(instruction) => {
                merge_location = Some(location);
            }
            Some(_) => {}
        }
    };
    Ok(Some(CFGBlock::new(
        label_location,
        merge_location,
        termination_location,
    )))
}

fn parse_function_structure<'g, 'i>(
    state: &mut TranslationStateParseFunctionsBase<'g, 'i>,
    function: &OpFunction,
) -> TranslationResult<SPIRVFunction<'g, 'i>> {
    let OpFunction {
        id_result_type,
        id_result,
        function_control:
            FunctionControl {
                inline: function_control_inline,
                dont_inline: function_control_dont_inline,
                pure_: function_control_pure,
                const_: function_control_const,
            },
        function_type,
    } = *function;
    let mut relaxed_precision = false;
    for decoration in state.take_decorations(id_result)? {
        match decoration {
            DecorationClass::Ignored(_) => {}
            DecorationClass::Invalid(_)
            | DecorationClass::MemoryObjectDeclaration(_)
            | DecorationClass::MemoryObjectDeclarationOrStructMember(_)
            | DecorationClass::Misc(_)
            | DecorationClass::Object(_)
            | DecorationClass::Struct(_)
            | DecorationClass::StructMember(_)
            | DecorationClass::Variable(_)
            | DecorationClass::VariableOrStructMember(_) => {
                return Err(DecorationNotAllowedOnInstruction {
                    decoration: decoration.into(),
                    instruction: function.clone().into(),
                }
                .into());
            }
            DecorationClass::RelaxedPrecision(_) => relaxed_precision = true,
        }
    }
    let function_type = state
        .get_type(function_type)?
        .function()
        .ok_or_else(|| FunctionsFunctionTypeIsNotOpTypeFunction {
            instruction: function.clone().into(),
        })?
        .clone();
    if state.get_type(id_result_type.0)? != &function_type.return_type {
        return Err(FunctionsResultTypeMustMatchFunctionTypesReturnType {
            instruction: function.clone().into(),
        }
        .into());
    }
    let mut return_type = function_type.return_type.clone();
    let mut parameter_types = function_type.parameter_types.clone();
    if relaxed_precision {
        return_type = return_type.get_relaxed_precision_type().ok_or_else(|| {
            RelaxedPrecisionDecorationNotAllowed {
                instruction: function.clone().into(),
            }
        })?;
    }

    let inlining_hint = match (function_control_inline, function_control_dont_inline) {
        (None, None) => InliningHint::None,
        (Some(_), None) => InliningHint::Inline,
        (None, Some(_)) => InliningHint::DontInline,
        (Some(_), Some(_)) => {
            return Err(InlineAndDontInlineAreNotAllowedTogether {
                instruction: function.clone().into(),
            }
            .into())
        }
    };
    let side_effects = match (function_control_pure, function_control_const) {
        (None, None) => FunctionSideEffects::Normal,
        (Some(_), None) => FunctionSideEffects::Pure,
        (None, Some(_)) => FunctionSideEffects::Const,
        (Some(_), Some(_)) => {
            return Err(ConstAndPureAreNotAllowedTogether {
                instruction: function.clone().into(),
            }
            .into())
        }
    };

    let hints = FunctionHints {
        inlining_hint,
        side_effects,
    };

    let function_debug_name = state.get_or_make_debug_name(id_result.0)?;

    let parameter_types_len = parameter_types.len();

    let argument_definitions = parameter_types
        .iter_mut()
        .enumerate()
        .map(|(parameter_index, parameter_type)| {
            if let Some((Instruction::FunctionParameter(instruction), _)) =
                state.next_instruction_and_location()?
            {
                let OpFunctionParameter {
                    id_result,
                    id_result_type,
                } = *instruction;
                let mut object_decorations = Vec::new();
                for decoration in state.take_decorations(id_result)? {
                    match decoration {
                        DecorationClass::Ignored(_) => {}
                        DecorationClass::Invalid(_)
                        | DecorationClass::MemoryObjectDeclaration(_)
                        | DecorationClass::MemoryObjectDeclarationOrStructMember(_)
                        | DecorationClass::Struct(_)
                        | DecorationClass::StructMember(_)
                        | DecorationClass::Variable(_)
                        | DecorationClass::VariableOrStructMember(_)
                        | DecorationClass::Misc(_) => {
                            return Err(DecorationNotAllowedOnInstruction {
                                decoration: decoration.into(),
                                instruction: instruction.clone().into(),
                            }
                            .into());
                        }
                        DecorationClass::RelaxedPrecision(_) => todo!(),
                        DecorationClass::Object(decoration) => object_decorations.push(decoration),
                    }
                }
                todo!()
            } else {
                Err(TooFewOpFunctionParameterInstructions {
                    expected_count: parameter_types_len as u32,
                    actual_count: parameter_index as u32,
                    instruction: function.clone().into(),
                }
                .into())
            }
        })
        .collect::<TranslationResult<Vec<ValueDefinition<'g>>>>()?;

    let entry_block = parse_cfg_block(state, parameter_types_len as u32)?.ok_or_else(|| {
        FunctionMustHaveABody {
            instruction: function.clone().into(),
        }
    })?;

    let body_debug_name = state.get_or_make_debug_name(entry_block.label_id())?;

    let mut cfg_builder = CFGBuilder::new(entry_block, &state.spirv_header)?;

    while let Some(block) = parse_cfg_block(state, parameter_types_len as u32)? {
        cfg_builder.insert(block)?;
    }

    let cfg = cfg_builder.into_cfg()?;

    writeln!(
        state.debug_output,
        "CFG (graphviz format):\n\n{}\n",
        cfg.dump_to_dot()
    )?;

    writeln!(
        state.debug_output,
        "structure tree:\n{:#?}",
        cfg.structure_tree()
    )?;

    let ir_return_type = return_type
        .get_ir_type(state.global_state)?
        .into_iter()
        .map(|ty| ValueDefinition::new(ty, "", state.global_state))
        .collect();

    let body = Block::new(
        body_debug_name,
        None,
        Inhabited(ir_return_type),
        state.global_state,
    );

    let ir_function = Function::new(
        function_debug_name,
        hints,
        argument_definitions,
        None,
        body,
        state.global_state,
    );

    let function = SPIRVFunction::new(SPIRVFunctionData {
        ir_value: ir_function.value(),
        cfg,
    });

    state.ir_functions.push(ir_function);

    state.define_function(id_result, function.clone())?;
    Ok(function)
}

impl<'g, 'i> TranslationStateParsedTypesConstantsAndGlobals<'g, 'i> {
    pub(crate) fn parse_functions_section(
        self,
    ) -> TranslationResult<TranslationStateParsedFunctions<'g, 'i>> {
        let mut base_state = TranslationStateParseFunctionsBase {
            functions: IdMap::new(self.spirv_header),
            base: self,
            ir_functions: Vec::new(),
        };
        writeln!(base_state.debug_output, "parsing functions section")?;
        let mut functions = Vec::new();
        while let Some(instruction) = base_state.next_instruction()? {
            if let Instruction::Function(function) = instruction {
                functions.push(parse_function_structure(&mut base_state, function)?);
            } else {
                return Err(InvalidSPIRVInstructionInSection {
                    instruction: instruction.clone(),
                    section_name: "functions",
                }
                .into());
            };
        }
        for function in &functions {
            let mut body_state = TranslationStateParsingFunctionBody {
                base: base_state,
                function,
                local_variables: Vec::new(),
            };
            writeln!(
                body_state.debug_output,
                "parsing function body (prepass): {:?}",
                function.ir_value.name
            )?;
            for (cfg_block_id, block) in function.cfg.node_references() {
                body_state.set_spirv_instructions_location(block.label_location());
                loop {
                    let instruction = body_state
                        .next_instruction()?
                        .expect("missing termination instruction");
                    instruction.parse_in_function_body_prepass(&mut body_state, cfg_block_id)?;
                    if TerminationInstruction::is_in_subset(instruction) {
                        break;
                    }
                }
            }
            let TranslationStateParsingFunctionBody {
                base,
                function: _function,
                local_variables,
            } = body_state.translate_structure_tree()?;
            function.ir_value.set_local_variables(local_variables);
            base_state = base;
        }
        Ok(TranslationStateParsedFunctions { base: base_state })
    }
}

impl<'g, 'i> TranslationStateParsedFunctions<'g, 'i> {
    pub(crate) fn translate(mut self) -> TranslationResult<TranslatedSPIRVShader<'g>> {
        let global_state = self.global_state;
        let target_properties = self.target_properties;
        let built_in_inputs_block = InterfaceBlock::new(
            ValueDefinition::new(DataPointerType, "built_in_inputs_block", global_state),
            StructSize::Fixed { size: 0 },
            Alignment::default(),
            vec![],
        );
        let user_inputs_block = InterfaceBlock::new(
            ValueDefinition::new(DataPointerType, "user_inputs_block", global_state),
            StructSize::Fixed { size: 0 },
            Alignment::default(),
            vec![],
        );
        let built_in_outputs_block = InterfaceBlock::new(
            ValueDefinition::new(DataPointerType, "built_in_outputs_block", global_state),
            StructSize::Fixed { size: 0 },
            Alignment::default(),
            vec![],
        );
        let user_outputs_block = InterfaceBlock::new(
            ValueDefinition::new(DataPointerType, "user_outputs_block", global_state),
            StructSize::Fixed { size: 0 },
            Alignment::default(),
            vec![],
        );
        let invocation_global_variables = vec![];
        let entry_point_id = self.entry_point_id;
        let entry_point = FunctionRef::new(self.get_function(entry_point_id)?.ir_value);
        let TranslationStateParsedFunctions {
            base:
                TranslationStateParseFunctionsBase {
                    base: state,
                    functions,
                    ir_functions,
                },
        } = self;
        let module = Module {
            target_properties,
            built_in_inputs_block,
            user_inputs_block,
            built_in_outputs_block,
            user_outputs_block,
            invocation_global_variables,
            functions: ir_functions,
            entry_point,
        };
        Ok(TranslatedSPIRVShader {
            global_state,
            module,
        })
    }
}

impl ParseInstruction for OpFunction {}
impl ParseInstruction for OpFunctionParameter {}
impl ParseInstruction for OpFunctionEnd {}
