// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
use shader_compiler_ir::prelude::*;
use std::fmt;

#[derive(Debug)]
pub enum TranslationError {}

impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl std::error::Error for TranslationError {}

#[derive(Debug)]
pub struct TranslatedSPIRVShader<'g> {
    pub global_state: &'g GlobalState<'g>,
}

impl<'g> TranslatedSPIRVShader<'g> {
    pub fn new(global_state: &'g GlobalState<'g>) -> Result<Self, TranslationError> {
        todo!()
    }
}
