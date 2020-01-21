// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

mod structs;

use crate::errors::DecorationNotAllowedOnInstruction;
use crate::errors::InvalidFloatTypeBitWidth;
use crate::errors::InvalidIntegerType;
use crate::errors::InvalidVectorComponentCount;
use crate::errors::InvalidVectorComponentType;
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
use crate::types::IntegerType;
use crate::types::PointerType;
use crate::types::PointerTypeData;
use crate::types::SPIRVType;
use crate::types::Signedness;
use crate::types::VectorType;
use crate::types::VoidType;
use shader_compiler_ir::BoolType;
use shader_compiler_ir::FloatType;
use spirv_id_map::Entry::Vacant;
use spirv_parser::Decoration;
use spirv_parser::IdRef;
use spirv_parser::IdResult;
use spirv_parser::{
    OpTypeArray, OpTypeBool, OpTypeFloat, OpTypeForwardPointer, OpTypeFunction, OpTypeImage,
    OpTypeInt, OpTypeMatrix, OpTypePointer, OpTypeRuntimeArray, OpTypeSampledImage, OpTypeSampler,
    OpTypeVector, OpTypeVoid,
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
        use shader_compiler_ir::IntegerType::*;
        let OpTypeInt {
            id_result,
            width,
            signedness,
        } = *self;
        state.error_if_any_decorations(id_result, || self.clone().into())?;
        let ir_type = match width {
            8 => Int8.into(),
            16 => Int16.into(),
            32 => Int32.into(),
            64 => Int64.into(),
            _ => return Err(InvalidIntegerType { width, signedness }.into()),
        };
        let signedness = match signedness {
            0 => Signedness::UnsignedOrUnspecified,
            1 => Signedness::Signed,
            _ => return Err(InvalidIntegerType { width, signedness }.into()),
        };
        state.define_type(
            id_result,
            IntegerType {
                ir_type,
                signedness,
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

impl ParseInstruction for OpTypeVector {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpTypeVector {
            id_result,
            component_type,
            component_count,
        } = *self;
        state.error_if_any_decorations(id_result, || self.clone().into())?;
        let component_type =
            state
                .get_type(component_type)?
                .scalar()
                .ok_or(InvalidVectorComponentType {
                    component_type_id: component_type,
                })?;
        let component_count = match component_count {
            2..=4 => component_count as usize,
            _ => return Err(InvalidVectorComponentCount { component_count }.into()),
        };
        state.define_type(
            id_result,
            VectorType {
                component_type,
                component_count: component_count as usize,
            },
        )
    }
}

impl ParseInstruction for OpTypeForwardPointer {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpTypeForwardPointer {
            pointer_type,
            storage_class: _storage_class,
        } = *self;
        state.define_type(
            IdResult(pointer_type),
            PointerType::new_forward_declaration(),
        )
    }
}

impl ParseInstruction for OpTypePointer {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpTypePointer {
            id_result,
            storage_class,
            type_: pointee_type,
        } = *self;
        let decorations = state.take_decorations(id_result)?;
        let mut array_stride = None;
        for decoration in decorations {
            match decoration {
                Decoration::ArrayStride { array_stride: v } => array_stride = Some(v),
                _ => {
                    return Err(DecorationNotAllowedOnInstruction {
                        decoration,
                        instruction: self.clone().into(),
                    }
                    .into());
                }
            }
        }
        let pointee_type = state.get_type(pointee_type)?.clone();
        let pointer_type = state
            .types
            .entry(id_result.0)?
            .or_insert_with(|| PointerType::new_forward_declaration().into());
        if let Some(pointer_type) = pointer_type.pointer() {
            pointer_type
                .resolve_forward_declaration(PointerTypeData {
                    pointee_type,
                    storage_class,
                    array_stride,
                })
                .map_err(|_| SPIRVIdAlreadyDefined { id_result })?;
            Ok(())
        } else {
            Err(SPIRVIdAlreadyDefined { id_result }.into())
        }
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

unimplemented_type_instruction!(OpTypeMatrix);
unimplemented_type_instruction!(OpTypeImage);
unimplemented_type_instruction!(OpTypeSampler);
unimplemented_type_instruction!(OpTypeSampledImage);
unimplemented_type_instruction!(OpTypeArray);
unimplemented_type_instruction!(OpTypeRuntimeArray);
