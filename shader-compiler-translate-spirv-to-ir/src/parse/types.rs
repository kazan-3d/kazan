// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::errors::InvalidFloatTypeBitWidth;
use crate::errors::InvalidIntegerType;
use crate::errors::SPIRVIdAlreadyDefined;
use crate::errors::SPIRVIdNotDefined;
use crate::errors::TranslationResult;
use crate::errors::UnsupportedSPIRVType;
use crate::errors::VoidNotAllowedHere;
use crate::parse::ParseInstruction;
use crate::parse::TranslationStateParseBaseTypesConstantsAndGlobals;
use crate::parse::TranslationStateParsingTypesConstantsAndGlobals;
use crate::types::FunctionType;
use crate::types::FunctionTypeData;
use crate::types::SPIRVType;
use crate::types::VoidType;
use shader_compiler_ir::BoolType;
use shader_compiler_ir::FloatType;
use shader_compiler_ir::IntegerType;
use spirv_id_map::Entry::Vacant;
use spirv_parser::IdRef;
use spirv_parser::IdResult;
use spirv_parser::{
    OpTypeArray, OpTypeBool, OpTypeFloat, OpTypeForwardPointer, OpTypeFunction, OpTypeImage,
    OpTypeInt, OpTypeMatrix, OpTypeOpaque, OpTypePointer, OpTypeRuntimeArray, OpTypeSampledImage,
    OpTypeSampler, OpTypeStruct, OpTypeVector, OpTypeVoid,
};

impl<'g, 'i> TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i> {
    fn define_type(
        &mut self,
        id_result: IdResult,
        ty: impl Into<SPIRVType<'g>>,
    ) -> TranslationResult<()> {
        if let Vacant(entry) = self.types.entry(id_result.0)? {
            entry.insert(ty.into());
            Ok(())
        } else {
            Err(SPIRVIdAlreadyDefined { id_result }.into())
        }
    }
    pub(crate) fn get_type(&self, type_id: IdRef) -> TranslationResult<&SPIRVType<'g>> {
        self.types
            .get(type_id)?
            .ok_or_else(|| SPIRVIdNotDefined { id: type_id }.into())
    }
    pub(crate) fn get_nonvoid_type<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        type_id: IdRef,
        instruction: I,
    ) -> TranslationResult<&SPIRVType<'g>> {
        let retval = self.get_type(type_id)?;
        if retval.is_void() {
            Err(VoidNotAllowedHere {
                type_id,
                instruction: instruction(),
            }
            .into())
        } else {
            Ok(retval)
        }
    }
}

impl ParseInstruction for OpTypeVoid {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpTypeVoid { id_result } = *self;
        state.error_if_any_decorations(id_result, || self.clone().into())?;
        state.define_type(id_result, VoidType)
    }
}

impl ParseInstruction for OpTypeBool {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpTypeBool { id_result } = *self;
        state.error_if_any_decorations(id_result, || self.clone().into())?;
        state.define_type(id_result, BoolType)
    }
}

impl ParseInstruction for OpTypeFloat {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpTypeFloat { id_result, width } = *self;
        state.error_if_any_decorations(id_result, || self.clone().into())?;
        state.define_type(
            id_result,
            match width {
                16 => FloatType::Float16,
                32 => FloatType::Float32,
                64 => FloatType::Float64,
                _ => return Err(InvalidFloatTypeBitWidth { width }.into()),
            },
        )
    }
}

impl ParseInstruction for OpTypeInt {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpTypeInt {
            id_result,
            width,
            signedness,
        } = *self;
        state.error_if_any_decorations(id_result, || self.clone().into())?;
        match signedness {
            0 | 1 => {}
            _ => return Err(InvalidIntegerType { width, signedness }.into()),
        }
        state.define_type(
            id_result,
            match width {
                8 => IntegerType::Int8,
                16 => IntegerType::Int16,
                32 => IntegerType::Int32,
                64 => IntegerType::Int64,
                _ => return Err(InvalidIntegerType { width, signedness }.into()),
            },
        )
    }
}

impl ParseInstruction for OpTypeFunction {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpTypeFunction {
            id_result,
            return_type,
            ref parameter_types,
        } = *self;
        state.error_if_any_decorations(id_result, || self.clone().into())?;
        let return_type = state.get_type(return_type)?.clone();
        let parameter_types = parameter_types
            .iter()
            .map(|&parameter_type| {
                Ok(state
                    .get_nonvoid_type(parameter_type, || self.clone().into())?
                    .clone())
            })
            .collect::<TranslationResult<_>>()?;
        state.define_type(
            id_result,
            FunctionType::new(FunctionTypeData {
                parameter_types,
                return_type,
            }),
        )
    }
}

macro_rules! unsupported_type_instruction {
    ($opname:ident) => {
        impl ParseInstruction for spirv_parser::$opname {
            fn parse_in_types_constants_globals_section<'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
            ) -> TranslationResult<()> {
                Err(UnsupportedSPIRVType {
                    instruction: self.clone().into(),
                }
                .into())
            }
        }
    };
}

unsupported_type_instruction!(OpTypeOpaque);
unsupported_type_instruction!(OpTypeEvent);
unsupported_type_instruction!(OpTypeDeviceEvent);
unsupported_type_instruction!(OpTypeReserveId);
unsupported_type_instruction!(OpTypeQueue);
unsupported_type_instruction!(OpTypePipe);
unsupported_type_instruction!(OpTypePipeStorage);
unsupported_type_instruction!(OpTypeNamedBarrier);

macro_rules! unimplemented_type_instruction {
    ($opname:ident) => {
        impl ParseInstruction for $opname {
            fn parse_in_types_constants_globals_section<'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
            ) -> TranslationResult<()> {
                todo!(concat!(
                    "unimplemented type instruction: ",
                    stringify!($opname)
                ))
            }
        }
    };
}

unimplemented_type_instruction!(OpTypeVector);
unimplemented_type_instruction!(OpTypeMatrix);
unimplemented_type_instruction!(OpTypeImage);
unimplemented_type_instruction!(OpTypeSampler);
unimplemented_type_instruction!(OpTypeSampledImage);
unimplemented_type_instruction!(OpTypeArray);
unimplemented_type_instruction!(OpTypeRuntimeArray);
unimplemented_type_instruction!(OpTypeStruct);
unimplemented_type_instruction!(OpTypePointer);
unimplemented_type_instruction!(OpTypeForwardPointer);
