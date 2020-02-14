// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![deny(missing_docs)]
#![no_std]

//! Shader Compiler Intermediate Representation

#[cfg(test)]
#[macro_use]
extern crate std;
#[macro_use]
extern crate alloc;

#[macro_use]
pub mod text;
pub mod prelude;

mod block;
mod consts;
mod debug_info;
mod function;
mod global_state;
mod instructions_impl;
mod interface;
mod module;
mod target_properties;
mod types;
mod values;

pub use crate::{
    block::*, consts::*, debug_info::*, function::*, global_state::*, instructions_impl::*,
    interface::*, module::*, target_properties::*, types::*, values::*,
};
pub use once_cell::unsync::OnceCell;

/// code structure input/output
pub trait CodeIO<'g> {
    /// the list of SSA value definitions that are the results of executing `self`, or `Uninhabited` if `self` doesn't return
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]>;
    /// the list of SSA values that are the arguments for `self`
    fn arguments(&self) -> &[ValueUse<'g>];
}
