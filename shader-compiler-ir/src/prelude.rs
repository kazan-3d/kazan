// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

//! IR prelude

pub use crate::{
    text::{FromText, FromTextCharExt, ToText},
    Block, BlockData, BlockRef, CodeIO, Const, Function, FunctionData, FunctionRef, GenericType,
    GlobalState, Id, IdMethod, Inhabitable, Inhabited, Instruction, Internable, Interned, Location,
    Loop, LoopData, LoopRef, Module, Type, Uninhabited, Value, ValueDefinition, ValueUse,
};
