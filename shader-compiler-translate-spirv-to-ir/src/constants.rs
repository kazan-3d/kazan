// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    errors::TranslationResult,
    types::SPIRVType,
    values::{GenericSPIRVValue, SPIRVValue},
};
use shader_compiler_ir::{Const, GlobalState, Interned};
use spirv_parser::BuiltIn;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum SPIRVConstantValue<'g> {
    Simple(Interned<'g, Const<'g>>),
}

#[derive(Clone, Debug)]
pub(crate) struct SPIRVConstant<'g> {
    pub(crate) value: SPIRVConstantValue<'g>,
    pub(crate) spirv_type: SPIRVType<'g>,
    pub(crate) built_in: Option<BuiltIn>,
}

impl<'g> GenericSPIRVValue<'g> for SPIRVConstant<'g> {
    fn get_type(&self) -> SPIRVType<'g> {
        self.spirv_type.clone()
    }
    fn get_ir_value(
        &self,
        global_state: &'g GlobalState<'g>,
    ) -> TranslationResult<shader_compiler_ir::IdRef<'g, shader_compiler_ir::Value<'g>>> {
        match self.value {
            SPIRVConstantValue::Simple(v) => {
                Ok(shader_compiler_ir::Value::from_const(v, "", global_state))
            }
        }
    }
}

impl<'g> From<SPIRVConstant<'g>> for SPIRVValue<'g> {
    fn from(v: SPIRVConstant<'g>) -> Self {
        Self::Constant(v)
    }
}
