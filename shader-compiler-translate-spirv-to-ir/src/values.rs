// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::errors::TranslationResult;
use crate::types::SPIRVType;
use alloc::rc::Rc;
use core::ops::Deref;
use shader_compiler_ir::GlobalState;

pub(crate) trait GenericSPIRVValue<'g>: Clone + Into<SPIRVValue<'g>> {
    fn get_type(&self) -> SPIRVType<'g>;
    fn get_ir_value(
        &self,
        global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<shader_compiler_ir::IdRef<'g, shader_compiler_ir::Value<'g>>>;
}

#[derive(Debug)]
pub(crate) struct SPIRVVariableData<'g> {
    pub(crate) ty: SPIRVType<'g>,
}

#[derive(Clone, Debug)]
pub(crate) struct SPIRVVariable<'g>(Rc<SPIRVVariableData<'g>>);

impl<'g> SPIRVVariable<'g> {
    pub(crate) fn new(v: SPIRVVariableData<'g>) -> Self {
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
        self.ty.clone()
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
