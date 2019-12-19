// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![deny(missing_docs)]

//! Shader Compiler Intermediate Representation

pub mod code;
pub mod debug;
mod global_state;
mod interned_string;
pub mod types;
pub mod value;

pub use crate::global_state::GlobalState;
pub use crate::interned_string::InternedString;
