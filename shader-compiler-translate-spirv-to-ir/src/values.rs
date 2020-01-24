// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::decorations::MemoryObjectDeclaration;
use crate::decorations::MemoryObjectDeclarationOrStructMember;
use crate::decorations::SPIRVObject;
use crate::decorations::VariableOrStructMember;
use crate::errors::TranslationResult;
use crate::types::PointerType;
use crate::types::SPIRVType;
use alloc::rc::Rc;
use core::ops::Deref;
use shader_compiler_ir::GlobalState;
use spirv_parser::BuiltIn;

pub(crate) trait GenericSPIRVValue<'g>: Clone + Into<SPIRVValue<'g>> {
    fn get_type(&self) -> SPIRVType<'g>;
    fn get_ir_value(
        &self,
        global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<shader_compiler_ir::IdRef<'g, shader_compiler_ir::Value<'g>>>;
}

#[derive(Debug)]
pub(crate) struct SPIRVVariableData<'g> {
    pub(crate) blend_equation_input_index: Option<u32>,
    pub(crate) result_type: PointerType<'g>,
    pub(crate) binding_point: Option<u32>,
    pub(crate) descriptor_set: Option<u32>,
    pub(crate) input_attachment_index: Option<u32>,
    pub(crate) built_in: Option<BuiltIn>,
    pub(crate) memory_object_declaration_or_struct_member: MemoryObjectDeclarationOrStructMember,
    pub(crate) memory_object_declaration: MemoryObjectDeclaration,
    pub(crate) variable_or_struct_member: VariableOrStructMember,
    pub(crate) object: SPIRVObject,
}

impl_decoration_aspect_members! {
    struct SPIRVVariableData<'_> {
        memory_object_declaration_or_struct_member: MemoryObjectDeclarationOrStructMember,
        memory_object_declaration: MemoryObjectDeclaration,
        variable_or_struct_member: VariableOrStructMember,
        object: SPIRVObject,
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SPIRVVariable<'g>(Rc<SPIRVVariableData<'g>>);

impl<'g> SPIRVVariable<'g> {
    pub(crate) fn new(v: SPIRVVariableData<'g>) -> Self {
        assert!(v.result_type.get().is_some());
        Self(Rc::new(v))
    }
}

impl<'g> Deref for SPIRVVariable<'g> {
    type Target = SPIRVVariableData<'g>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'g> From<SPIRVVariable<'g>> for SPIRVValue<'g> {
    fn from(v: SPIRVVariable<'g>) -> Self {
        Self::Variable(v)
    }
}

impl<'g> GenericSPIRVValue<'g> for SPIRVVariable<'g> {
    fn get_type(&self) -> SPIRVType<'g> {
        self.result_type.clone().into()
    }
    fn get_ir_value(
        &self,
        global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<shader_compiler_ir::IdRef<'g, shader_compiler_ir::Value<'g>>> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum SPIRVValue<'g> {
    Variable(SPIRVVariable<'g>),
}

impl<'g> GenericSPIRVValue<'g> for SPIRVValue<'g> {
    fn get_type(&self) -> SPIRVType<'g> {
        match self {
            Self::Variable(v) => v.get_type(),
        }
    }
    fn get_ir_value(
        &self,
        global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<shader_compiler_ir::IdRef<'g, shader_compiler_ir::Value<'g>>> {
        match self {
            Self::Variable(v) => v.get_ir_value(global_state),
        }
    }
}
