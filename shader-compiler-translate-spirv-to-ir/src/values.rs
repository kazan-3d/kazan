// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    constants::SPIRVConstant,
    decorations::{
        GetDecorationAspect, MemoryObjectDeclaration, MemoryObjectDeclarationOrStructMember,
        SPIRVObject, VariableOrStructMember,
    },
    errors::TranslationResult,
    functions::SPIRVFunctionParameter,
    types::{PointerType, SPIRVType},
};
use alloc::rc::Rc;
use core::ops::Deref;
use once_cell::unsync::OnceCell;
use shader_compiler_ir::GlobalState;
use spirv_parser::{BuiltIn, StorageClass};

pub(crate) trait GenericSPIRVValue<'g>:
    Clone + Into<SPIRVValue<'g>> + GetDecorationAspect<SPIRVObject>
{
    fn get_type(&self) -> SPIRVType<'g>;
    fn get_ir_value(
        &self,
        global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<shader_compiler_ir::ValueUse<'g>>;
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
    pub(crate) storage_class: StorageClass,
    pub(crate) initializer: Option<spirv_parser::IdRef>,
    pub(crate) ir_value: OnceCell<shader_compiler_ir::ValueUse<'g>>,
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

impl<'g> GenericSPIRVValue<'g> for SPIRVVariable<'g> {
    fn get_type(&self) -> SPIRVType<'g> {
        self.result_type.clone().into()
    }
    fn get_ir_value(
        &self,
        _global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<shader_compiler_ir::ValueUse<'g>> {
        Ok(self
            .ir_value
            .get_or_try_init(|| -> TranslationResult<_> { todo!() })?
            .clone())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SimpleValue<'g> {
    pub(crate) ir_value: shader_compiler_ir::ValueUse<'g>,
    pub(crate) result_type: SPIRVType<'g>,
    pub(crate) object: SPIRVObject,
}

impl_decoration_aspect_members! {
    struct SimpleValue<'_> {
        object: SPIRVObject,
    }
}

impl<'g> GenericSPIRVValue<'g> for SimpleValue<'g> {
    fn get_type(&self) -> SPIRVType<'g> {
        self.result_type.clone()
    }
    fn get_ir_value(
        &self,
        _global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<shader_compiler_ir::ValueUse<'g>> {
        Ok(self.ir_value.clone())
    }
}

macro_rules! impl_spirv_value {
    (
        $vis:vis enum $name:ident<$g:lifetime> {
            $(
                $(#[doc = $enumerant_doc:expr])*
                $enumerant_name:ident($enumerant_type:ty),
            )+
        }
    ) => {
        #[derive(Clone, Debug)]
        $vis enum $name<$g> {
            $(
                $(#[doc = $enumerant_doc])*
                $enumerant_name($enumerant_type),
            )+
        }

        $(
            impl<$g> From<$enumerant_type> for $name<$g> {
                fn from(v: $enumerant_type) -> Self {
                    Self::$enumerant_name(v)
                }
            }
        )+

        impl<$g> GetDecorationAspect<SPIRVObject> for $name<$g> {
            fn get_decoration_aspect_impl(&self) -> &SPIRVObject {
                match self {
                    $(Self::$enumerant_name(v) => v.get_decoration_aspect_impl(),)+
                }
            }
        }

        impl<$g> GenericSPIRVValue<$g> for $name<$g> {
            fn get_type(&self) -> SPIRVType<$g> {
                match self {
                    $(Self::$enumerant_name(v) => v.get_type(),)+
                }
            }
            fn get_ir_value(
                &self,
                global_state: &$g GlobalState<$g>,
            ) -> TranslationResult<shader_compiler_ir::ValueUse<'g>> {
                match self {
                    $(Self::$enumerant_name(v) => v.get_ir_value(global_state),)+
                }
            }
        }
    };
}

impl_spirv_value! {
    pub(crate) enum SPIRVValue<'g> {
        Variable(SPIRVVariable<'g>),
        Constant(SPIRVConstant<'g>),
        FunctionParameter(SPIRVFunctionParameter<'g>),
        Simple(SimpleValue<'g>),
    }
}
