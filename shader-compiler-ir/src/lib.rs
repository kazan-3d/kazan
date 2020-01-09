// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![warn(missing_docs)]

//! Shader Compiler Intermediate Representation

#[macro_use]
pub mod text;
pub mod prelude;

mod block;
mod consts;
mod debug_info;
mod function;
mod global_state;
mod instructions_impl;
mod types;
mod values;

pub use crate::block::*;
pub use crate::consts::*;
pub use crate::debug_info::*;
pub use crate::function::*;
pub use crate::global_state::*;
pub use crate::instructions_impl::*;
pub use crate::types::*;
pub use crate::values::*;
pub use once_cell::unsync::OnceCell;

/// code structure input/output
pub trait CodeIO<'g> {
    /// the list of SSA value definitions that are the results of executing `self`, or `Uninhabited` if `self` doesn't return
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]>;
    /// the list of SSA values that are the arguments for `self`
    fn arguments(&self) -> &[ValueUse<'g>];
}
