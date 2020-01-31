// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    decorations::{DecorationClass, DecorationClassMisc, DecorationClassObject},
    errors::{
        DecorationNotAllowedOnInstruction, FunctionsFunctionTypeIsNotOpTypeFunction,
        FunctionsResultTypeMustMatchFunctionTypesReturnType, InvalidSPIRVInstructionInSection,
        RelaxedPrecisionDecorationNotAllowed, TranslationResult,
    },
    parse::{ParseInstruction, TranslationStateParsedTypesConstantsAndGlobals},
    types::{FunctionType, FunctionTypeData, GenericSPIRVType},
    TranslatedSPIRVShader,
};
use alloc::rc::Rc;
use core::ops::Deref;
use once_cell::unsync::OnceCell;
use spirv_id_map::IdMap;
use spirv_parser::{FunctionControl, Instruction, OpFunction, OpFunctionEnd, OpFunctionParameter};

#[derive(Debug)]
pub(crate) struct SPIRVFunctionData {}

#[derive(Clone, Debug)]
pub(crate) struct SPIRVFunction(Rc<SPIRVFunctionData>);

impl Deref for SPIRVFunction {
    type Target = SPIRVFunctionData;
    fn deref(&self) -> &SPIRVFunctionData {
        &self.0
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParseFunctionsBase<'g, 'i> {
        base: TranslationStateParsedTypesConstantsAndGlobals<'g, 'i>,
        functions: IdMap<spirv_parser::IdRef, SPIRVFunction>,
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsingFunctionBodies<'g, 'i> {
        base: TranslationStateParseFunctionsBase<'g, 'i>,
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsedFunctions<'g, 'i> {
        base: TranslationStateParseFunctionsBase<'g, 'i>,
    }
}

fn parse_function_instruction<'g, 'i>(
    state: TranslationStateParseFunctionsBase<'g, 'i>,
    function: &OpFunction,
) -> TranslationResult<TranslationStateParsingFunctionBodies<'g, 'i>> {
    let mut state = TranslationStateParsingFunctionBodies { base: state };
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
    if relaxed_precision {
        let return_type = function_type
            .return_type
            .get_relaxed_precision_type()
            .ok_or_else(|| RelaxedPrecisionDecorationNotAllowed {
                instruction: function.clone().into(),
            })?;
        let parameter_types = function_type.parameter_types.clone();
        function_type = FunctionType::new(FunctionTypeData {
            parameter_types,
            return_type,
        });
    }

    todo!()
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
            let mut bodies_state = if let Instruction::Function(function) = instruction {
                parse_function_instruction(base_state, function)?
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
