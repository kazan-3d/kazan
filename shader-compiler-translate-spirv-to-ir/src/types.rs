// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

pub(crate) mod structs;

use crate::{
    errors::{
        InvalidComponentDecorationOnVariableOrStructMember,
        MissingLocationDecorationOnVariableOrStructMember, TranslationResult,
        TypeNotAllowedInUserDefinedVariableInterface, VoidNotAllowedHere,
    },
    io_layout::{COMPONENT_SIZE_IN_BYTES, LOCATION_SIZE_IN_BYTES},
    TranslationError,
};
use alloc::{rc::Rc, vec::Vec};
use core::{marker::PhantomData, ops::Deref};
use once_cell::unsync::OnceCell;
use shader_compiler_ir::{
    prelude::*, Alignment, BoolType, BuiltInInterfaceVariableAttributes, FloatType,
    InterfaceBlockMember, InterfaceVariable, TargetProperties, UserInterfaceVariableAttributes,
};
use spirv_parser::{IdRef, StorageClass};
use structs::StructType;

pub(crate) struct GetIrTypeState<'g> {
    global_state: &'g GlobalState<'g>,
}

pub(crate) enum IOInterfaceIR<'g> {
    IRType(Interned<'g, shader_compiler_ir::Type<'g>>),
    UserInterfaceBlockMembers(Vec<InterfaceBlockMember<'g, UserInterfaceVariableAttributes>>),
    BuiltInInterfaceVariables(Vec<InterfaceVariable<'g, BuiltInInterfaceVariableAttributes>>),
}

impl<'g> From<Interned<'g, shader_compiler_ir::Type<'g>>> for IOInterfaceIR<'g> {
    fn from(v: Interned<'g, shader_compiler_ir::Type<'g>>) -> Self {
        Self::IRType(v)
    }
}

impl<'g> IOInterfaceIR<'g> {
    pub(crate) fn into_user_interface_block_members<E: Into<TranslationError>>(
        self,
        byte_offset: u32,
        error_on_built_in_interface_variables: impl FnOnce() -> E,
    ) -> TranslationResult<Vec<InterfaceBlockMember<'g, UserInterfaceVariableAttributes>>> {
        match self {
            IOInterfaceIR::IRType(member_type) => Ok(vec![InterfaceBlockMember {
                struct_member: shader_compiler_ir::StructMember {
                    member_type,
                    offset: byte_offset,
                },
                attributes: UserInterfaceVariableAttributes {},
            }]),
            IOInterfaceIR::UserInterfaceBlockMembers(members) => Ok(members),
            IOInterfaceIR::BuiltInInterfaceVariables(_) => {
                Err(error_on_built_in_interface_variables().into())
            }
        }
    }
    pub(crate) fn into_built_in_interface_variables<E: Into<TranslationError>>(
        self,
        error_on_user_interface_block_members: impl FnOnce() -> E,
    ) -> TranslationResult<Vec<InterfaceVariable<'g, BuiltInInterfaceVariableAttributes>>> {
        match self {
            IOInterfaceIR::IRType(member_type) => todo!(),
            IOInterfaceIR::BuiltInInterfaceVariables(variables) => Ok(variables),
            IOInterfaceIR::UserInterfaceBlockMembers(_) => {
                Err(error_on_user_interface_block_members().into())
            }
        }
    }
}

pub(crate) struct IOInterfaceIRResult<'g> {
    pub(crate) byte_offset: u32,
    pub(crate) size_in_bytes: u32,
    pub(crate) first_location_after: Option<u32>,
    pub(crate) ir: IOInterfaceIR<'g>,
}

pub(crate) trait GenericSPIRVType<'g>: Eq + Clone + Into<SPIRVType<'g>> {
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
    fn get_nonvoid_ir_type<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        instruction: I,
    ) -> TranslationResult<Interned<'g, shader_compiler_ir::Type<'g>>> {
        self.get_ir_type(global_state)?.ok_or_else(|| {
            VoidNotAllowedHere {
                type_id,
                instruction: instruction(),
            }
            .into()
        })
    }
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>>;
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        target_properties: Interned<'g, TargetProperties>,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        instruction: I,
    ) -> TranslationResult<Alignment>;
    fn translate_io_interface_to_ir(
        &self,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        start_location: Option<u32>,
        start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>>;
}

pub(crate) trait GenericIOScalarType {
    fn io_component_count(&self) -> Option<u32>;
    fn io_size_in_bytes(&self) -> Option<u32>;
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

fn is_start_io_component_valid(
    vector_component_type: &impl GenericIOScalarType,
    vector_component_count: u32,
    start_io_component: u32,
) -> bool {
    match (
        vector_component_type
            .io_component_count()
            .expect("invalid component_type"),
        vector_component_count,
    ) {
        (1, 1) => start_io_component < 4,
        (1, 2) => start_io_component < 3,
        (1, 3) => start_io_component < 2,
        (1, 4) => start_io_component == 0,
        (2, 1) => start_io_component == 0 || start_io_component == 2,
        (2, 2) | (2, 3) | (2, 4) => start_io_component == 0,
        _ => unreachable!(),
    }
}

impl GenericIOScalarType for IntegerType {
    fn io_component_count(&self) -> Option<u32> {
        Some(match self.ir_type {
            shader_compiler_ir::IntegerType::Int8
            | shader_compiler_ir::IntegerType::Int16
            | shader_compiler_ir::IntegerType::Int32
            | shader_compiler_ir::IntegerType::RelaxedInt32 => 1,
            shader_compiler_ir::IntegerType::Int64 => 2,
        })
    }
    fn io_size_in_bytes(&self) -> Option<u32> {
        Some(match self.ir_type {
            shader_compiler_ir::IntegerType::Int8 => 1,
            shader_compiler_ir::IntegerType::Int16 => 2,
            shader_compiler_ir::IntegerType::Int32
            | shader_compiler_ir::IntegerType::RelaxedInt32 => 4,
            shader_compiler_ir::IntegerType::Int64 => 8,
        })
    }
}

impl<'g> GenericSPIRVType<'g> for IntegerType {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        Ok(Some(self.ir_type.intern(state.global_state)))
    }
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        let IntegerType {
            ir_type,
            signedness,
        } = *self;
        match ir_type {
            shader_compiler_ir::IntegerType::Int32
            | shader_compiler_ir::IntegerType::RelaxedInt32 => Some(
                IntegerType {
                    ir_type: shader_compiler_ir::IntegerType::RelaxedInt32,
                    signedness,
                }
                .into(),
            ),
            shader_compiler_ir::IntegerType::Int8
            | shader_compiler_ir::IntegerType::Int16
            | shader_compiler_ir::IntegerType::Int64 => None,
        }
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        target_properties: Interned<'g, TargetProperties>,
        _global_state: &'g GlobalState<'g>,
        _type_id: IdRef,
        _instruction: I,
    ) -> TranslationResult<Alignment> {
        Ok(self.ir_type.alignment(&target_properties))
    }
    fn translate_io_interface_to_ir(
        &self,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        start_location: Option<u32>,
        start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>> {
        let start_location = start_location
            .ok_or_else(|| MissingLocationDecorationOnVariableOrStructMember { type_id })?;
        let start_io_component = start_io_component.unwrap_or(0);
        if !is_start_io_component_valid(self, 1, start_io_component) {
            return Err(InvalidComponentDecorationOnVariableOrStructMember {
                type_id,
                component: start_io_component,
            }
            .into());
        }
        let byte_offset =
            LOCATION_SIZE_IN_BYTES * start_location + COMPONENT_SIZE_IN_BYTES * start_io_component;
        Ok(IOInterfaceIRResult {
            byte_offset,
            size_in_bytes: self.io_size_in_bytes().expect("known to be valid"),
            first_location_after: Some(start_location + 1),
            ir: self.ir_type.intern(global_state).into(),
        })
    }
}

impl GenericIOScalarType for FloatType {
    fn io_component_count(&self) -> Option<u32> {
        Some(match self {
            FloatType::Float16 | FloatType::Float32 | FloatType::RelaxedFloat32 => 1,
            FloatType::Float64 => 2,
        })
    }
    fn io_size_in_bytes(&self) -> Option<u32> {
        Some(match self {
            FloatType::Float16 => 2,
            FloatType::Float32 | FloatType::RelaxedFloat32 => 4,
            FloatType::Float64 => 8,
        })
    }
}

impl<'g> GenericSPIRVType<'g> for FloatType {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        Ok(Some(self.intern(state.global_state)))
    }
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        match self {
            FloatType::Float32 | FloatType::RelaxedFloat32 => {
                Some(FloatType::RelaxedFloat32.into())
            }
            FloatType::Float16 | FloatType::Float64 => None,
        }
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        target_properties: Interned<'g, TargetProperties>,
        _global_state: &'g GlobalState<'g>,
        _type_id: IdRef,
        _instruction: I,
    ) -> TranslationResult<Alignment> {
        Ok(self.alignment(&target_properties))
    }
    fn translate_io_interface_to_ir(
        &self,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        start_location: Option<u32>,
        start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>> {
        let start_location = start_location
            .ok_or_else(|| MissingLocationDecorationOnVariableOrStructMember { type_id })?;
        let start_io_component = start_io_component.unwrap_or(0);
        if !is_start_io_component_valid(self, 1, start_io_component) {
            return Err(InvalidComponentDecorationOnVariableOrStructMember {
                type_id,
                component: start_io_component,
            }
            .into());
        }
        let byte_offset =
            LOCATION_SIZE_IN_BYTES * start_location + COMPONENT_SIZE_IN_BYTES * start_io_component;
        Ok(IOInterfaceIRResult {
            byte_offset,
            size_in_bytes: self.io_size_in_bytes().expect("known to be valid"),
            first_location_after: Some(start_location + 1),
            ir: self.intern(global_state).into(),
        })
    }
}

impl<'g> GenericSPIRVType<'g> for BoolType {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        Ok(Some(self.intern(state.global_state)))
    }
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        None
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        target_properties: Interned<'g, TargetProperties>,
        _global_state: &'g GlobalState<'g>,
        _type_id: IdRef,
        _instruction: I,
    ) -> TranslationResult<Alignment> {
        Ok(self.alignment(&target_properties))
    }
    fn translate_io_interface_to_ir(
        &self,
        _global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        _start_location: Option<u32>,
        _start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>> {
        Err(TypeNotAllowedInUserDefinedVariableInterface { type_id }.into())
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
            fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
                match self {
                    $(
                        Self::$member_name(ty) => ty.get_relaxed_precision_type(),
                    )+
                }
            }
            fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
                &self,
                target_properties: Interned<'g, TargetProperties>,
                global_state: &'g GlobalState<'g>,
                type_id: IdRef,
                instruction: I,
            ) -> TranslationResult<Alignment> {
                match self {
                    $(
                        Self::$member_name(ty) => ty.get_alignment(target_properties, global_state, type_id, instruction),
                    )+
                }
            }
            fn translate_io_interface_to_ir(
                &self,
                global_state: &'g GlobalState<'g>,
                type_id: IdRef,
                start_location: Option<u32>,
                start_io_component: Option<u32>,
            ) -> TranslationResult<IOInterfaceIRResult<'g>> {
                match self {
                    $(
                        Self::$member_name(ty) => ty.translate_io_interface_to_ir(
                            global_state,
                            type_id,
                            start_location,
                            start_io_component,
                        ),
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

impl GenericIOScalarType for ScalarType {
    fn io_component_count(&self) -> Option<u32> {
        match self {
            ScalarType::Integer(v) => v.io_component_count(),
            ScalarType::Float(v) => v.io_component_count(),
            ScalarType::Bool(_) => None,
        }
    }
    fn io_size_in_bytes(&self) -> Option<u32> {
        match self {
            ScalarType::Integer(v) => v.io_size_in_bytes(),
            ScalarType::Float(v) => v.io_size_in_bytes(),
            ScalarType::Bool(_) => None,
        }
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
    pub(crate) fn as_never(&self) -> ! {
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
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        None
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        _target_properties: Interned<'g, TargetProperties>,
        _global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        instruction: I,
    ) -> TranslationResult<Alignment> {
        Err(VoidNotAllowedHere {
            type_id,
            instruction: instruction(),
        }
        .into())
    }
    fn translate_io_interface_to_ir(
        &self,
        _global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        _start_location: Option<u32>,
        _start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>> {
        Err(TypeNotAllowedInUserDefinedVariableInterface { type_id }.into())
    }
}

#[derive(Eq, PartialEq, Debug)]
pub(crate) struct FunctionTypeData<'g> {
    pub(crate) parameter_types: Vec<SPIRVType<'g>>,
    pub(crate) return_type: SPIRVType<'g>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
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
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        None
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        _target_properties: Interned<'g, TargetProperties>,
        _global_state: &'g GlobalState<'g>,
        _type_id: IdRef,
        _instruction: I,
    ) -> TranslationResult<Alignment> {
        todo!()
    }
    fn translate_io_interface_to_ir(
        &self,
        _global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        _start_location: Option<u32>,
        _start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>> {
        Err(TypeNotAllowedInUserDefinedVariableInterface { type_id }.into())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct VectorType {
    pub(crate) component_type: ScalarType,
    pub(crate) component_count: u32,
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
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        let VectorType {
            component_type,
            component_count,
        } = *self;
        let component_type = component_type
            .get_relaxed_precision_type()?
            .scalar()
            .expect("known to be a scalar type");
        Some(
            VectorType {
                component_type,
                component_count,
            }
            .into(),
        )
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        target_properties: Interned<'g, TargetProperties>,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        instruction: I,
    ) -> TranslationResult<Alignment> {
        Ok(self
            .get_ir_type(global_state)?
            .expect("known to be non-void")
            .alignment(&target_properties))
    }
    fn translate_io_interface_to_ir(
        &self,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        start_location: Option<u32>,
        start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>> {
        let start_location = start_location
            .ok_or_else(|| MissingLocationDecorationOnVariableOrStructMember { type_id })?;
        let start_io_component = start_io_component.unwrap_or(0);
        let io_components_per_component = self
            .component_type
            .io_component_count()
            .ok_or_else(|| TypeNotAllowedInUserDefinedVariableInterface { type_id })?;
        let size_in_locations = match (self.component_count, io_components_per_component) {
            (2, 1) | (3, 1) | (4, 1) | (2, 2) => 1,
            (3, 2) | (4, 2) => 2,
            _ => unreachable!(),
        };
        if !is_start_io_component_valid(
            &self.component_type,
            self.component_count,
            start_io_component,
        ) {
            return Err(InvalidComponentDecorationOnVariableOrStructMember {
                type_id,
                component: start_io_component,
            }
            .into());
        }
        let byte_offset =
            LOCATION_SIZE_IN_BYTES * start_location + COMPONENT_SIZE_IN_BYTES * start_io_component;
        Ok(IOInterfaceIRResult {
            byte_offset,
            size_in_bytes: self
                .component_type
                .io_size_in_bytes()
                .expect("known to be valid")
                * self.component_count,
            first_location_after: Some(start_location + size_in_locations),
            ir: self
                .get_ir_type(global_state)?
                .expect("known to be non-void")
                .into(),
        })
    }
}

#[derive(Debug)]
pub(crate) struct PointerTypeData<'g> {
    pub(crate) pointee_type: SPIRVType<'g>,
    pub(crate) pointee_type_id: spirv_parser::IdRef,
    pub(crate) storage_class: StorageClass,
    pub(crate) array_stride: Option<u32>,
}

#[derive(Clone, Debug)]
pub(crate) struct PointerType<'g> {
    id: spirv_parser::IdRef,
    data: Rc<OnceCell<PointerTypeData<'g>>>,
}

impl<'g> PointerType<'g> {
    pub(crate) fn new(id: spirv_parser::IdRef, v: PointerTypeData<'g>) -> Self {
        Self {
            id,
            data: Rc::new(OnceCell::from(v)),
        }
    }
    pub(crate) fn new_forward_declaration(id: spirv_parser::IdRef) -> Self {
        Self {
            id,
            data: Rc::new(OnceCell::new()),
        }
    }
    pub(crate) fn id(&self) -> spirv_parser::IdRef {
        self.id
    }
    pub(crate) fn get(&self) -> Option<&PointerTypeData<'g>> {
        self.data.get()
    }
    pub(crate) fn resolve_forward_declaration(
        &self,
        v: PointerTypeData<'g>,
    ) -> Result<(), PointerTypeData<'g>> {
        self.data.set(v)
    }
}

impl<'g> From<PointerType<'g>> for SPIRVType<'g> {
    fn from(v: PointerType<'g>) -> Self {
        Self::Pointer(v)
    }
}

impl<'g> GenericSPIRVType<'g> for PointerType<'g> {
    fn get_ir_type_with_state(
        &self,
        _state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        todo!()
    }
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        None
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        _target_properties: Interned<'g, TargetProperties>,
        _global_state: &'g GlobalState<'g>,
        _type_id: IdRef,
        _instruction: I,
    ) -> TranslationResult<Alignment> {
        todo!()
    }
    fn translate_io_interface_to_ir(
        &self,
        _global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        _start_location: Option<u32>,
        _start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>> {
        Err(TypeNotAllowedInUserDefinedVariableInterface { type_id }.into())
    }
}

impl PartialEq<PointerType<'_>> for PointerType<'_> {
    fn eq(&self, rhs: &PointerType<'_>) -> bool {
        self.id == rhs.id
    }
}

impl Eq for PointerType<'_> {}

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) enum SPIRVType<'g> {
    Scalar(ScalarType),
    Void(VoidType),
    Function(FunctionType<'g>),
    Vector(VectorType),
    Struct(StructType<'g>),
    Pointer(PointerType<'g>),
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
    pub(crate) fn function(&self) -> Option<&FunctionType<'g>> {
        match self {
            Self::Function(retval) => Some(retval),
            _ => None,
        }
    }
    pub(crate) fn vector(&self) -> Option<VectorType> {
        match *self {
            Self::Vector(retval) => Some(retval),
            _ => None,
        }
    }
    pub(crate) fn struct_type(&self) -> Option<&StructType<'g>> {
        match self {
            Self::Struct(retval) => Some(retval),
            _ => None,
        }
    }
    pub(crate) fn pointer(&self) -> Option<&PointerType<'g>> {
        match self {
            Self::Pointer(retval) => Some(retval),
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
            SPIRVType::Struct(ty) => ty.get_ir_type_with_state(state),
            SPIRVType::Pointer(ty) => ty.get_ir_type_with_state(state),
            SPIRVType::_Uninhabited(v) => v.as_never(),
        }
    }
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        match self {
            SPIRVType::Scalar(ty) => ty.get_relaxed_precision_type(),
            SPIRVType::Void(ty) => ty.get_relaxed_precision_type(),
            SPIRVType::Function(ty) => ty.get_relaxed_precision_type(),
            SPIRVType::Vector(ty) => ty.get_relaxed_precision_type(),
            SPIRVType::Struct(ty) => ty.get_relaxed_precision_type(),
            SPIRVType::Pointer(ty) => ty.get_relaxed_precision_type(),
            SPIRVType::_Uninhabited(v) => v.as_never(),
        }
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        target_properties: Interned<'g, TargetProperties>,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        instruction: I,
    ) -> TranslationResult<Alignment> {
        match self {
            SPIRVType::Scalar(ty) => {
                ty.get_alignment(target_properties, global_state, type_id, instruction)
            }
            SPIRVType::Void(ty) => {
                ty.get_alignment(target_properties, global_state, type_id, instruction)
            }
            SPIRVType::Function(ty) => {
                ty.get_alignment(target_properties, global_state, type_id, instruction)
            }
            SPIRVType::Vector(ty) => {
                ty.get_alignment(target_properties, global_state, type_id, instruction)
            }
            SPIRVType::Struct(ty) => {
                ty.get_alignment(target_properties, global_state, type_id, instruction)
            }
            SPIRVType::Pointer(ty) => {
                ty.get_alignment(target_properties, global_state, type_id, instruction)
            }
            SPIRVType::_Uninhabited(v) => v.as_never(),
        }
    }
    fn translate_io_interface_to_ir(
        &self,
        global_state: &'g GlobalState<'g>,
        type_id: IdRef,
        start_location: Option<u32>,
        start_io_component: Option<u32>,
    ) -> TranslationResult<IOInterfaceIRResult<'g>> {
        match self {
            SPIRVType::Scalar(ty) => ty.translate_io_interface_to_ir(
                global_state,
                type_id,
                start_location,
                start_io_component,
            ),
            SPIRVType::Void(ty) => ty.translate_io_interface_to_ir(
                global_state,
                type_id,
                start_location,
                start_io_component,
            ),
            SPIRVType::Function(ty) => ty.translate_io_interface_to_ir(
                global_state,
                type_id,
                start_location,
                start_io_component,
            ),
            SPIRVType::Vector(ty) => ty.translate_io_interface_to_ir(
                global_state,
                type_id,
                start_location,
                start_io_component,
            ),
            SPIRVType::Struct(ty) => ty.translate_io_interface_to_ir(
                global_state,
                type_id,
                start_location,
                start_io_component,
            ),
            SPIRVType::Pointer(ty) => ty.translate_io_interface_to_ir(
                global_state,
                type_id,
                start_location,
                start_io_component,
            ),
            SPIRVType::_Uninhabited(v) => v.as_never(),
        }
    }
}
