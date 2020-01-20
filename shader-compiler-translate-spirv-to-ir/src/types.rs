// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::errors::TranslationResult;
use crate::errors::VoidNotAllowedHere;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::marker::PhantomData;
use core::ops::Deref;
use shader_compiler_ir::prelude::*;
use shader_compiler_ir::BoolType;
use shader_compiler_ir::FloatType;
use spirv_parser::IdRef;

pub(crate) struct GetIrTypeState<'g> {
    global_state: &'g GlobalState<'g>,
}

pub(crate) trait GenericSPIRVType<'g>: Clone + Into<SPIRVType<'g>> {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>>;
    fn get_ir_type(
        &self,
        global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        self.get_ir_type_with_state(&mut GetIrTypeState { global_state })
    }
    fn get_nonvoid_ir_type<B: Borrow<I>, I: Clone + Into<spirv_parser::Instruction>>(
        &self,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        instruction: B,
    ) -> TranslationResult<Interned<'g, shader_compiler_ir::Type<'g>>> {
        self.get_ir_type(global_state)?.ok_or_else(|| {
            VoidNotAllowedHere {
                type_id,
                instruction: instruction.borrow().clone().into(),
            }
            .into()
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum Signedness {
    Signed,
    UnsignedOrUnspecified,
}

impl Signedness {
    pub(crate) fn is_signed(self) -> bool {
        match self {
            Signedness::Signed => true,
            Signedness::UnsignedOrUnspecified => false,
        }
    }
    pub(crate) fn is_unsigned_or_unspecified(self) -> bool {
        match self {
            Signedness::UnsignedOrUnspecified => true,
            Signedness::Signed => false,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct IntegerType {
    pub(crate) ir_type: shader_compiler_ir::IntegerType,
    pub(crate) signedness: Signedness,
}

impl<'g> GenericSPIRVType<'g> for IntegerType {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        Ok(Some(self.ir_type.intern(state.global_state)))
    }
}

impl<'g> GenericSPIRVType<'g> for FloatType {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        Ok(Some(self.intern(state.global_state)))
    }
}

impl<'g> GenericSPIRVType<'g> for BoolType {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        Ok(Some(self.intern(state.global_state)))
    }
}

macro_rules! impl_scalar_type {
    (
        $vis:vis enum $name:ident {
            $($member_name:ident($member_type:ty),)+
        }
    ) => {
        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
        $vis enum $name {
            $(
                $member_name($member_type),
            )+
        }

        $(
            impl From<$member_type> for $name {
                fn from(v: $member_type) -> Self {
                    Self::$member_name(v)
                }
            }

            impl From<$member_type> for SPIRVType<'_> {
                fn from(v: $member_type) -> Self {
                    $name::from(v).into()
                }
            }
        )+

        impl<'g> GenericSPIRVType<'g> for $name {
            fn get_ir_type_with_state(
                &self,
                state: &mut GetIrTypeState<'g>,
            ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
                match self {
                    $(
                        Self::$member_name(ty) => ty.get_ir_type_with_state(state),
                    )+
                }
            }
        }
    };
}

impl_scalar_type! {
    pub(crate) enum ScalarType {
        Integer(IntegerType),
        Float(FloatType),
        Bool(BoolType),
    }
}

impl From<ScalarType> for SPIRVType<'_> {
    fn from(v: ScalarType) -> Self {
        Self::Scalar(v)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum UninhabitedHelper {}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum Uninhabited<'g> {
    _Uninhabited(PhantomData<&'g ()>, UninhabitedHelper),
}

impl Uninhabited<'_> {
    pub(crate) fn into(&self) -> ! {
        match *self {
            Uninhabited::_Uninhabited(_, v) => match v {},
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct VoidType;

impl From<VoidType> for SPIRVType<'_> {
    fn from(v: VoidType) -> Self {
        Self::Void(v)
    }
}

impl<'g> GenericSPIRVType<'g> for VoidType {
    fn get_ir_type_with_state(
        &self,
        _state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        Ok(None)
    }
}

#[derive(Debug)]
pub(crate) struct FunctionTypeData<'g> {
    pub(crate) parameter_types: Vec<SPIRVType<'g>>,
    pub(crate) return_type: SPIRVType<'g>,
}

#[derive(Clone, Debug)]
pub(crate) struct FunctionType<'g>(Rc<FunctionTypeData<'g>>);

impl<'g> FunctionType<'g> {
    pub(crate) fn new(v: FunctionTypeData<'g>) -> Self {
        assert!(!v.parameter_types.iter().any(SPIRVType::is_void));
        Self(Rc::new(v))
    }
}

impl<'g> Deref for FunctionType<'g> {
    type Target = FunctionTypeData<'g>;
    fn deref(&self) -> &FunctionTypeData<'g> {
        &self.0
    }
}

impl<'g> From<FunctionType<'g>> for SPIRVType<'g> {
    fn from(v: FunctionType<'g>) -> Self {
        Self::Function(v)
    }
}

impl<'g> GenericSPIRVType<'g> for FunctionType<'g> {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        let FunctionTypeData {
            parameter_types,
            return_type,
        } = &**self;
        let arguments = parameter_types
            .iter()
            .map(|parameter_type| {
                Ok(parameter_type
                    .get_ir_type_with_state(state)?
                    .expect("function parameters are known to be non-void"))
            })
            .collect::<TranslationResult<_>>()?;
        let returns = if let Some(return_type) = return_type.get_ir_type_with_state(state)? {
            Inhabited(vec![return_type])
        } else {
            Inhabited(vec![])
        };
        Ok(Some(
            shader_compiler_ir::FunctionPointerType { arguments, returns }
                .intern(state.global_state),
        ))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VectorType {
    pub(crate) component_type: ScalarType,
    pub(crate) component_count: usize,
}

impl From<VectorType> for SPIRVType<'_> {
    fn from(v: VectorType) -> Self {
        Self::Vector(v)
    }
}

impl<'g> GenericSPIRVType<'g> for VectorType {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        let element = self
            .component_type
            .get_ir_type_with_state(state)?
            .expect("known to be non-void");
        Ok(Some(
            shader_compiler_ir::VectorType {
                element,
                len: self.component_count,
                scalable: false,
            }
            .intern(state.global_state),
        ))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum SPIRVType<'g> {
    Scalar(ScalarType),
    Void(VoidType),
    Function(FunctionType<'g>),
    Vector(VectorType),
    _Uninhabited(Uninhabited<'g>),
}

impl<'g> SPIRVType<'g> {
    pub(crate) fn is_void(&self) -> bool {
        match self {
            Self::Void(_) => true,
            _ => false,
        }
    }
    pub(crate) fn scalar(&self) -> Option<ScalarType> {
        match *self {
            Self::Scalar(retval) => Some(retval),
            _ => None,
        }
    }
}

impl<'g> GenericSPIRVType<'g> for SPIRVType<'g> {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        match self {
            SPIRVType::Scalar(ty) => ty.get_ir_type_with_state(state),
            SPIRVType::Void(ty) => ty.get_ir_type_with_state(state),
            SPIRVType::Function(ty) => ty.get_ir_type_with_state(state),
            SPIRVType::Vector(ty) => ty.get_ir_type_with_state(state),
            SPIRVType::_Uninhabited(v) => v.into(),
        }
    }
}
