// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;

/// a debug location
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Location<'g> {
    /// the source file name
    pub file: Interned<'g, str>,
    /// the line number
    pub line: u32,
    /// the column number
    pub column: u32,
}
