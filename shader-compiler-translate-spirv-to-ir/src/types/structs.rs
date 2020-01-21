// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::errors::TranslationResult;
use crate::types::GenericSPIRVType;
use crate::types::GetIrTypeState;
use crate::types::SPIRVType;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::ops::Deref;
use shader_compiler_ir::Interned;
use spirv_parser::BuiltIn;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum StructKind {
    Generic,
    Block { is_buffer_block: bool },
    BuiltIns,
}

#[derive(Debug)]
pub(crate) struct StructMember<'g> {
    pub(crate) built_in: Option<BuiltIn>,
    pub(crate) member_type: SPIRVType<'g>,
}

#[derive(Debug)]
pub(crate) struct StructTypeData<'g> {
    pub(crate) kind: StructKind,
    pub(crate) members: Vec<StructMember<'g>>,
}

#[derive(Clone, Debug)]
pub(crate) struct StructType<'g>(Rc<StructTypeData<'g>>);

impl<'g> StructType<'g> {
    pub(crate) fn new(v: StructTypeData<'g>) -> Self {
        // TODO: assert validity
        Self(Rc::new(v))
    }
}

impl<'g> Deref for StructType<'g> {
    type Target = StructTypeData<'g>;
    fn deref(&self) -> &StructTypeData<'g> {
        &self.0
    }
}

impl<'g> From<StructType<'g>> for SPIRVType<'g> {
    fn from(v: StructType<'g>) -> Self {
        Self::Struct(v)
    }
}

impl<'g> GenericSPIRVType<'g> for StructType<'g> {
    fn get_ir_type_with_state(
        &self,
        state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        todo!()
    }
}
