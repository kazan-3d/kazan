// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    errors::TranslationResult,
    types::{GenericSPIRVType, GetIrTypeState, SPIRVType},
};
use alloc::{rc::Rc, vec::Vec};
use core::ops::Deref;
use shader_compiler_ir::{Alignment, GlobalState, Interned, TargetProperties};
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
    pub(crate) id: spirv_parser::IdRef,
    pub(crate) kind: StructKind,
    pub(crate) members: Vec<StructMember<'g>>,
}

impl PartialEq<StructTypeData<'_>> for StructTypeData<'_> {
    fn eq(&self, rhs: &StructTypeData<'_>) -> bool {
        self.id == rhs.id
    }
}

impl Eq for StructTypeData<'_> {}

#[derive(Clone, Eq, PartialEq, Debug)]
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
        _state: &mut GetIrTypeState<'g>,
    ) -> TranslationResult<Option<Interned<'g, shader_compiler_ir::Type<'g>>>> {
        todo!()
    }
    fn get_relaxed_precision_type(&self) -> Option<SPIRVType<'g>> {
        None
    }
    fn get_alignment<I: FnOnce() -> spirv_parser::Instruction>(
        &self,
        target_properties: Interned<'g, TargetProperties>,
        global_state: &'g GlobalState<'g>,
        type_id: spirv_parser::IdRef,
        instruction: I,
    ) -> TranslationResult<Alignment> {
        todo!()
    }
}
