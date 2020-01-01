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

impl<'g> Location<'g> {
    pub fn new_interned(
        file: impl Internable<'g, Interned = str>,
        line: u32,
        column: u32,
        global_state: &'g GlobalState<'g>,
    ) -> Interned<'g, Location<'g>> {
        let file = file.intern(global_state);
        Location { file, line, column }.intern(global_state)
    }
}

impl<'g> Internable<'g> for Location<'g> {
    type Interned = Location<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Location<'g>> {
        global_state.intern(self)
    }
}
