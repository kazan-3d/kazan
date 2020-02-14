// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    cfg::CFG, decorations::SPIRVObject, errors::TranslationResult, types::SPIRVType,
    values::GenericSPIRVValue,
};
use alloc::rc::Rc;
use core::{cell::RefCell, ops::Deref};
use shader_compiler_ir::{Function, FunctionData, GlobalState};

#[derive(Clone, Debug)]
pub(crate) struct SPIRVFunctionParameter<'g> {
    pub(crate) ir_value: shader_compiler_ir::ValueUse<'g>,
    pub(crate) value_type: SPIRVType<'g>,
    pub(crate) parameter_index: usize,
    pub(crate) object: SPIRVObject,
}

impl_decoration_aspect_members! {
    struct SPIRVFunctionParameter<'_> {
        object: SPIRVObject,
    }
}

impl<'g> GenericSPIRVValue<'g> for SPIRVFunctionParameter<'g> {
    fn get_type(&self) -> SPIRVType<'g> {
        self.value_type.clone()
    }
    fn get_ir_value(
        &self,
        _global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<shader_compiler_ir::ValueUse<'g>> {
        Ok(self.ir_value.clone())
    }
}

pub(crate) struct SPIRVFunctionData<'g, 'i> {
    pub(crate) ir_value: shader_compiler_ir::IdRef<'g, FunctionData<'g>>,
    pub(crate) cfg: CFG<'g, 'i>,
}

#[derive(Clone)]
pub(crate) struct SPIRVFunction<'g, 'i>(Rc<SPIRVFunctionData<'g, 'i>>);

impl<'g, 'i> Deref for SPIRVFunction<'g, 'i> {
    type Target = SPIRVFunctionData<'g, 'i>;
    fn deref(&self) -> &SPIRVFunctionData<'g, 'i> {
        &self.0
    }
}

impl<'g, 'i> SPIRVFunction<'g, 'i> {
    pub(crate) fn new(v: SPIRVFunctionData<'g, 'i>) -> Self {
        Self(Rc::new(v))
    }
}
