// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    cfg::CFGBlockId,
    constants::{SPIRVConstant, SPIRVConstantValue},
    decorations::{DecorationClass, DecorationClassMisc},
    errors::{
        ConstantResultTypeMustBeBool, ConstantResultTypeMustBeIntOrFloat, ConstantValueTooBigSmall,
        DecorationNotAllowedOnInstruction, RelaxedPrecisionDecorationNotAllowed,
        SpecializationConstantMissingSpecId, TranslationResult, UnsupportedSPIRVType,
    },
    parse::{
        functions::TranslationStateParsingFunctionBody, ParseInstruction,
        TranslationStateParseBaseTypesConstantsAndGlobals,
        TranslationStateParsingTypesConstantsAndGlobals,
    },
    types::{GenericSPIRVType, IntegerType, ScalarType, Signedness},
    SpecializationResolutionFailed, SpecializationResolver, TranslationStateBase,
    UnresolvedSpecialization,
};
use alloc::vec::Vec;
use core::{
    convert::{TryFrom, TryInto},
    mem,
    num::TryFromIntError,
};
use shader_compiler_ir::{
    BoolType, Const, FloatType, Internable, Interned, RelaxedFloat32, RelaxedInt32,
};
use spirv_parser::{
    DecorationBuiltIn, DecorationSpecId, IdResult, IdResultType, Instruction, OpConstant32,
    OpConstant64, OpConstantComposite, OpConstantFalse, OpConstantNull, OpConstantPipeStorage,
    OpConstantSampler, OpConstantTrue, OpSpecConstant32, OpSpecConstant64, OpSpecConstantComposite,
    OpSpecConstantFalse, OpSpecConstantOp, OpSpecConstantTrue,
};

trait ParseConstantInstruction: ParseInstruction {
    fn parse_constant<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()>;
}

macro_rules! impl_parse_constant {
    ($ty:ty) => {
        impl ParseInstruction for $ty {
            fn parse_in_types_constants_globals_section<'g, 'i>(
                &'i self,
                state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
            ) -> TranslationResult<()> {
                self.parse_constant(state)
            }
            fn parse_in_function_body_generic<'f, 'g, 'i>(
                &'i self,
                state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                self.parse_constant(state)
            }
        }
    };
}

macro_rules! unimplemented_constant_instruction {
    ($opname:ident) => {
        impl ParseConstantInstruction for $opname {
            fn parse_constant<'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
            ) -> TranslationResult<()> {
                todo!(concat!(
                    "unimplemented constant instruction: ",
                    stringify!($opname)
                ))
            }
        }
    };
}

fn parse_constant_bool<'g, 'i, I: Fn() -> Instruction>(
    state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    id_result_type: IdResultType,
    id_result: IdResult,
    value: bool,
    decorations: Vec<DecorationClass>,
    instruction: I,
) -> TranslationResult<()> {
    let result_type = state.get_type(id_result_type.0)?.clone();
    if result_type.scalar() != Some(ScalarType::Bool(BoolType)) {
        return Err(ConstantResultTypeMustBeBool {
            instruction: instruction(),
        }
        .into());
    }
    let mut built_in = None;
    for decoration in decorations {
        match decoration {
            DecorationClass::Ignored(_) => {}
            DecorationClass::Invalid(_)
            | DecorationClass::MemoryObjectDeclaration(_)
            | DecorationClass::MemoryObjectDeclarationOrStructMember(_)
            | DecorationClass::Misc(DecorationClassMisc::ArrayStride(_))
            | DecorationClass::Misc(DecorationClassMisc::FPRoundingMode(_))
            | DecorationClass::Misc(DecorationClassMisc::SpecId(_))
            | DecorationClass::Object(_)
            | DecorationClass::Struct(_)
            | DecorationClass::StructMember(_)
            | DecorationClass::Variable(_)
            | DecorationClass::VariableOrStructMember(_) => {
                return Err(DecorationNotAllowedOnInstruction {
                    decoration: decoration.into(),
                    instruction: instruction(),
                }
                .into());
            }
            DecorationClass::RelaxedPrecision(_) => {
                return Err(RelaxedPrecisionDecorationNotAllowed {
                    instruction: instruction(),
                }
                .into());
            }
            DecorationClass::Misc(DecorationClassMisc::BuiltIn(DecorationBuiltIn {
                built_in: v,
            })) => built_in = Some(v),
        }
    }
    let value = SPIRVConstantValue::Simple(value.intern(state.global_state));
    state.define_value(
        id_result,
        SPIRVConstant {
            value,
            spirv_type: result_type,
            built_in,
        },
    )
}

impl_parse_constant!(OpConstantTrue);

impl ParseConstantInstruction for OpConstantTrue {
    fn parse_constant<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpConstantTrue {
            id_result_type,
            id_result,
        } = *self;
        let decorations = state.take_decorations(id_result)?;
        parse_constant_bool(state, id_result_type, id_result, true, decorations, || {
            self.clone().into()
        })
    }
}

impl_parse_constant!(OpConstantFalse);

impl ParseConstantInstruction for OpConstantFalse {
    fn parse_constant<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpConstantFalse {
            id_result_type,
            id_result,
        } = *self;
        let decorations = state.take_decorations(id_result)?;
        parse_constant_bool(state, id_result_type, id_result, false, decorations, || {
            self.clone().into()
        })
    }
}

fn resolve_relaxed_f32(
    specialization_resolver: &mut dyn SpecializationResolver,
    unresolved_specialization: UnresolvedSpecialization,
    default: RelaxedFloat32,
) -> Result<RelaxedFloat32, SpecializationResolutionFailed> {
    Ok(specialization_resolver
        .resolve_f32(unresolved_specialization, default.into())?
        .into())
}

fn resolve_relaxed_i32(
    specialization_resolver: &mut dyn SpecializationResolver,
    unresolved_specialization: UnresolvedSpecialization,
    default: RelaxedInt32,
) -> Result<RelaxedInt32, SpecializationResolutionFailed> {
    Ok(specialization_resolver
        .resolve_i32(unresolved_specialization, default.0 as i32)?
        .into())
}

fn resolve_relaxed_u32(
    specialization_resolver: &mut dyn SpecializationResolver,
    unresolved_specialization: UnresolvedSpecialization,
    default: RelaxedInt32,
) -> Result<RelaxedInt32, SpecializationResolutionFailed> {
    Ok(specialization_resolver
        .resolve_u32(unresolved_specialization, default.0)?
        .into())
}

fn parse_constant_scalar<'g, 'i, I: Fn() -> Instruction>(
    state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    id_result_type: IdResultType,
    id_result: IdResult,
    value_sign_extended: i64,
    value_zero_extended: u64,
    is_spec_constant: bool,
    instruction: I,
) -> TranslationResult<()> {
    fn get_constant_value<
        'g,
        'i,
        V: Internable<'g, Interned = Const<'g>> + 'static,
        I: Fn() -> Instruction,
        R: FnOnce(
            &mut dyn SpecializationResolver,
            UnresolvedSpecialization,
            V,
        ) -> Result<V, SpecializationResolutionFailed>,
    >(
        value: Result<V, TryFromIntError>,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
        specialization_constant_id: Option<u32>,
        instruction: I,
        resolve_specialization: R,
    ) -> TranslationResult<Interned<'g, Const<'g>>> {
        if let Ok(mut value) = value {
            if let Some(constant_id) = specialization_constant_id {
                // FIXME: rustc errors without following line, see rust issue #68590:
                let state: &mut TranslationStateBase = state;
                value = resolve_specialization(
                    state.specialization_resolver,
                    UnresolvedSpecialization { constant_id },
                    value,
                )?;
            }
            Ok(value.intern(state.global_state))
        } else {
            Err(ConstantValueTooBigSmall {
                instruction: instruction(),
            }
            .into())
        }
    }
    let mut decorations = state.take_decorations(id_result)?;
    let specialization_constant_id = if is_spec_constant {
        Some(get_spec_id(&mut decorations, &instruction)?.specialization_constant_id)
    } else {
        None
    };
    let mut built_in = None;
    let mut relaxed_precision = false;
    for decoration in decorations {
        match decoration {
            DecorationClass::Ignored(_) => {}
            DecorationClass::Invalid(_)
            | DecorationClass::MemoryObjectDeclaration(_)
            | DecorationClass::MemoryObjectDeclarationOrStructMember(_)
            | DecorationClass::Misc(DecorationClassMisc::ArrayStride(_))
            | DecorationClass::Misc(DecorationClassMisc::FPRoundingMode(_))
            | DecorationClass::Misc(DecorationClassMisc::SpecId(_))
            | DecorationClass::Object(_)
            | DecorationClass::Struct(_)
            | DecorationClass::StructMember(_)
            | DecorationClass::Variable(_)
            | DecorationClass::VariableOrStructMember(_) => {
                return Err(DecorationNotAllowedOnInstruction {
                    decoration: decoration.into(),
                    instruction: instruction(),
                }
                .into());
            }
            DecorationClass::RelaxedPrecision(_) => relaxed_precision = true,
            DecorationClass::Misc(DecorationClassMisc::BuiltIn(DecorationBuiltIn {
                built_in: v,
            })) => built_in = Some(v),
        }
    }
    let mut result_type = state.get_type(id_result_type.0)?.clone();
    if relaxed_precision {
        result_type = result_type.get_relaxed_precision_type().ok_or_else(|| {
            RelaxedPrecisionDecorationNotAllowed {
                instruction: instruction(),
            }
        })?;
    }
    macro_rules! match_value {
        (
            $result_type:ident,
            $instruction:ident,
            $state:ident,
            $value_zero_extended:ident,
            $specialization_constant_id:ident,
            {
                $(($float:ident, $resolve_float:path),)+
                @f64,
                $(($ir_type:ident, $signedness:ident, $resolve_int:path, $value_int:expr),)+
            }
        ) => {
            match $result_type.scalar() {
                $(
                    Some(ScalarType::Float(FloatType::$float)) => get_constant_value(
                        $value_zero_extended
                            .try_into()
                            .map(shader_compiler_ir::$float),
                        $state,
                        $specialization_constant_id,
                        $instruction,
                        |specialization_resolver,
                        unresolved_specialization,
                        default_value| $resolve_float(specialization_resolver, unresolved_specialization, default_value),
                    )?,
                )+
                Some(ScalarType::Float(FloatType::Float64)) => get_constant_value(
                    Ok(shader_compiler_ir::Float64(value_zero_extended)),
                    $state,
                    $specialization_constant_id,
                    $instruction,
                    |specialization_resolver,
                        unresolved_specialization,
                        default_value| SpecializationResolver::resolve_f64(specialization_resolver, unresolved_specialization, default_value)
                )?,
                $(
                    Some(ScalarType::Integer(IntegerType {
                        ir_type: shader_compiler_ir::IntegerType::$ir_type,
                        signedness: Signedness::$signedness,
                    })) => get_constant_value(
                        $value_int,
                        $state,
                        $specialization_constant_id,
                        $instruction,
                        |specialization_resolver,
                        unresolved_specialization,
                        default_value| $resolve_int(specialization_resolver, unresolved_specialization, default_value),
                    )?,
                )+
                None | Some(ScalarType::Bool(_)) => {
                    return Err(ConstantResultTypeMustBeIntOrFloat {
                        instruction: $instruction(),
                    }
                    .into())
                }
            }
        };
    }
    let value = SPIRVConstantValue::Simple(match_value! {
        result_type,
        instruction,
        state,
        value_zero_extended,
        specialization_constant_id,
        {
            (Float16, SpecializationResolver::resolve_f16),
            (Float32, SpecializationResolver::resolve_f32),
            (RelaxedFloat32, resolve_relaxed_f32),
            @f64,
            (Int8, UnsignedOrUnspecified, SpecializationResolver::resolve_u8, value_zero_extended.try_into()),
            (Int8, Signed, SpecializationResolver::resolve_i8, value_sign_extended.try_into()),
            (Int16, UnsignedOrUnspecified, SpecializationResolver::resolve_u16, value_zero_extended.try_into()),
            (Int16, Signed, SpecializationResolver::resolve_i16, value_sign_extended.try_into()),
            (Int32, UnsignedOrUnspecified, SpecializationResolver::resolve_u32, value_zero_extended.try_into()),
            (Int32, Signed, SpecializationResolver::resolve_i32, value_sign_extended.try_into()),
            (RelaxedInt32, UnsignedOrUnspecified, resolve_relaxed_u32, u32::try_from(value_zero_extended).map(Into::into)),
            (RelaxedInt32, Signed, resolve_relaxed_i32, i32::try_from(value_sign_extended).map(Into::into)),
            (Int64, UnsignedOrUnspecified, SpecializationResolver::resolve_u64, Ok(value_zero_extended)),
            (Int64, Signed, SpecializationResolver::resolve_i64, Ok(value_sign_extended)),
        }
    });
    state.define_value(
        id_result,
        SPIRVConstant {
            value,
            spirv_type: result_type,
            built_in,
        },
    )
}

impl_parse_constant!(OpConstant32);

impl ParseConstantInstruction for OpConstant32 {
    fn parse_constant<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpConstant32 {
            id_result_type,
            id_result,
            value,
        } = *self;
        parse_constant_scalar(
            state,
            id_result_type,
            id_result,
            value as i32 as i64,
            value as u64,
            false,
            || self.clone().into(),
        )
    }
}

impl_parse_constant!(OpConstant64);

impl ParseConstantInstruction for OpConstant64 {
    fn parse_constant<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpConstant64 {
            id_result_type,
            id_result,
            value,
        } = *self;
        parse_constant_scalar(
            state,
            id_result_type,
            id_result,
            value as i64,
            value,
            false,
            || self.clone().into(),
        )
    }
}

impl_parse_constant!(OpConstantComposite);
unimplemented_constant_instruction!(OpConstantComposite);

impl_parse_constant!(OpConstantNull);
unimplemented_constant_instruction!(OpConstantNull);

fn get_spec_id<I: Fn() -> Instruction>(
    decorations: &mut Vec<DecorationClass>,
    instruction: I,
) -> TranslationResult<DecorationSpecId> {
    let mut retval = None;
    for decoration in mem::replace(decorations, Vec::new()) {
        if let DecorationClass::Misc(DecorationClassMisc::SpecId(spec_id)) = decoration {
            retval = Some(spec_id);
        } else {
            decorations.push(decoration);
        }
    }
    retval.ok_or_else(|| {
        SpecializationConstantMissingSpecId {
            instruction: instruction(),
        }
        .into()
    })
}

fn parse_spec_constant_bool<'g, 'i, I: Fn() -> Instruction>(
    id_result_type: IdResultType,
    id_result: IdResult,
    state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    instruction: I,
    default_value: bool,
) -> TranslationResult<()> {
    let mut decorations = state.take_decorations(id_result)?;
    let DecorationSpecId {
        specialization_constant_id,
    } = get_spec_id(&mut decorations, &instruction)?;
    let value = state.specialization_resolver.resolve_bool(
        UnresolvedSpecialization {
            constant_id: specialization_constant_id,
        },
        default_value,
    )?;
    parse_constant_bool(
        state,
        id_result_type,
        id_result,
        value,
        decorations,
        instruction,
    )
}

impl_parse_constant!(OpSpecConstantTrue);

impl ParseConstantInstruction for OpSpecConstantTrue {
    fn parse_constant<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpSpecConstantTrue {
            id_result_type,
            id_result,
        } = *self;
        parse_spec_constant_bool(
            id_result_type,
            id_result,
            state,
            || self.clone().into(),
            true,
        )
    }
}

impl_parse_constant!(OpSpecConstantFalse);

impl ParseConstantInstruction for OpSpecConstantFalse {
    fn parse_constant<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpSpecConstantFalse {
            id_result_type,
            id_result,
        } = *self;
        parse_spec_constant_bool(
            id_result_type,
            id_result,
            state,
            || self.clone().into(),
            false,
        )
    }
}

impl_parse_constant!(OpSpecConstant32);

impl ParseConstantInstruction for OpSpecConstant32 {
    fn parse_constant<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpSpecConstant32 {
            id_result_type,
            id_result,
            value,
        } = *self;
        parse_constant_scalar(
            state,
            id_result_type,
            id_result,
            value as i32 as i64,
            value as u64,
            true,
            || self.clone().into(),
        )
    }
}

impl_parse_constant!(OpSpecConstant64);

impl ParseConstantInstruction for OpSpecConstant64 {
    fn parse_constant<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpSpecConstant64 {
            id_result_type,
            id_result,
            value,
        } = *self;
        parse_constant_scalar(
            state,
            id_result_type,
            id_result,
            value as i64,
            value,
            true,
            || self.clone().into(),
        )
    }
}

impl_parse_constant!(OpSpecConstantComposite);
unimplemented_constant_instruction!(OpSpecConstantComposite);

impl_parse_constant!(OpSpecConstantOp);
unimplemented_constant_instruction!(OpSpecConstantOp);

macro_rules! unsupported_constant_instruction {
    ($opname:ident) => {
        impl ParseConstantInstruction for $opname {
            fn parse_constant<'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
            ) -> TranslationResult<()> {
                Err(UnsupportedSPIRVType {
                    instruction: self.clone().into(),
                }
                .into())
            }
        }
    };
}

impl_parse_constant!(OpConstantSampler);
unsupported_constant_instruction!(OpConstantSampler);

impl_parse_constant!(OpConstantPipeStorage);
unsupported_constant_instruction!(OpConstantPipeStorage);
