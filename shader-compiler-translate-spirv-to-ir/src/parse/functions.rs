// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    decorations::{DecorationClass, DecorationClassMisc},
    errors::{
        ConstAndPureAreNotAllowedTogether, DecorationNotAllowedOnInstruction,
        FunctionMustHaveABody, FunctionsFunctionTypeIsNotOpTypeFunction,
        FunctionsResultTypeMustMatchFunctionTypesReturnType,
        InlineAndDontInlineAreNotAllowedTogether, InstructionNotValidBeforeLabel,
        InvalidSPIRVInstructionInSection, RelaxedPrecisionDecorationNotAllowed,
        SPIRVIdAlreadyDefined, TooFewOpFunctionParameterInstructions,
        TooManyOpFunctionParameterInstructions, TranslationResult,
    },
    parse::{ParseInstruction, TranslationStateParsedTypesConstantsAndGlobals},
    types::{FunctionType, FunctionTypeData, GenericSPIRVType},
    SPIRVInstructionsLocation, TranslatedSPIRVShader,
};
use alloc::{rc::Rc, vec::Vec};
use core::{cell::RefCell, ops::Deref};
use shader_compiler_ir::{
    Block, Function, FunctionHints, FunctionSideEffects, Inhabited, InliningHint, ValueDefinition,
};
use spirv_id_map::IdMap;
use spirv_parser::{
    FunctionControl, IdResult, Instruction, OpFunction, OpFunctionEnd, OpFunctionParameter, OpLabel,
};

pub(crate) struct SPIRVFunctionData<'g, 'i> {
    pub(crate) ir_function: RefCell<Option<shader_compiler_ir::Function<'g>>>,
    pub(crate) ir_value: shader_compiler_ir::IdRef<'g, shader_compiler_ir::FunctionData<'g>>,
    /// first instruction after last OpFunctionParameter
    pub(crate) body_start_location: SPIRVInstructionsLocation<'i>,
}

#[derive(Clone)]
pub(crate) struct SPIRVFunction<'g, 'i>(Rc<SPIRVFunctionData<'g, 'i>>);

impl<'g, 'i> Deref for SPIRVFunction<'g, 'i> {
    type Target = SPIRVFunctionData<'g, 'i>;
    fn deref(&self) -> &SPIRVFunctionData<'g, 'i> {
        &self.0
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParseFunctionsBase<'g, 'i> {
        base: TranslationStateParsedTypesConstantsAndGlobals<'g, 'i>,
        functions: IdMap<spirv_parser::IdRef, SPIRVFunction<'g, 'i>>,
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
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsingFunctionBody<'g, 'i> {
        base: TranslationStateParseFunctionsBase<'g, 'i>,
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsedFunctions<'g, 'i> {
        base: TranslationStateParseFunctionsBase<'g, 'i>,
    }
}

fn parse_function_header<'g, 'i>(
    state: TranslationStateParseFunctionsBase<'g, 'i>,
    function: &OpFunction,
) -> TranslationResult<TranslationStateParsingFunctionBody<'g, 'i>> {
    let mut state = TranslationStateParsingFunctionBody { base: state };
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
            | DecorationClass::Misc(DecorationClassMisc::ArrayStride(_))
            | DecorationClass::Misc(DecorationClassMisc::BuiltIn(_))
            | DecorationClass::Misc(DecorationClassMisc::FPRoundingMode(_))
            | DecorationClass::Misc(DecorationClassMisc::SpecId(_))
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
            DecorationClass::Misc(DecorationClassMisc::RelaxedPrecision(_)) => {
                relaxed_precision = true
            }
        }
    }
    let mut function_type = state
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
                state.get_instruction_and_location()?
            {
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

    let body_start_location = state.spirv_instructions_location.clone();

    let body_start_label_id = loop {
        match state
            .get_instruction_and_location()?
            .map(|(instruction, _)| instruction)
        {
            Some(instruction @ Instruction::FunctionParameter(_)) => {
                return Err(TooManyOpFunctionParameterInstructions {
                    expected_count: parameter_types_len as u32,
                    instruction: instruction.clone(),
                }
                .into());
            }
            Some(Instruction::Line(_)) | Some(Instruction::NoLine(_)) => {}
            Some(Instruction::FunctionEnd(_)) | None => {
                return Err(FunctionMustHaveABody {
                    instruction: function.clone().into(),
                }
                .into());
            }
            Some(Instruction::Label(OpLabel { id_result })) => break id_result,
            Some(instruction) => {
                return Err(InstructionNotValidBeforeLabel {
                    instruction: instruction.clone(),
                }
                .into());
            }
        }
    };

    state.spirv_instructions_location = body_start_location.clone();

    let body_debug_name = state.get_or_make_debug_name(body_start_label_id.0)?;

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
        body,
        state.global_state,
    );

    let function = SPIRVFunction(Rc::new(SPIRVFunctionData {
        ir_value: ir_function.value(),
        ir_function: RefCell::new(Some(ir_function)),
        body_start_location,
    }));

    state.define_function(id_result, function)?;
    Ok(state)
}

impl<'g, 'i> TranslationStateParsedTypesConstantsAndGlobals<'g, 'i> {
    pub(crate) fn parse_functions_section(
        self,
    ) -> TranslationResult<TranslationStateParsedFunctions<'g, 'i>> {
        let mut base_state = TranslationStateParseFunctionsBase {
            functions: IdMap::new(self.spirv_header),
            base: self,
        };
        writeln!(base_state.debug_output, "parsing functions section")?;
        while let Some((instruction, function_location)) =
            base_state.get_instruction_and_location()?
        {
            let mut body_state = if let Instruction::Function(function) = instruction {
                parse_function_header(base_state, function)?
            } else {
                return Err(InvalidSPIRVInstructionInSection {
                    instruction: instruction.clone(),
                    section_name: "functions",
                }
                .into());
            };
            todo!()
        }
        Ok(TranslationStateParsedFunctions { base: base_state })
    }
}

impl<'g, 'i> TranslationStateParsedFunctions<'g, 'i> {
    pub(crate) fn translate(self) -> TranslationResult<TranslatedSPIRVShader<'g>> {
        todo!()
    }
}

impl ParseInstruction for OpFunction {}
impl ParseInstruction for OpFunctionParameter {}
impl ParseInstruction for OpFunctionEnd {}
