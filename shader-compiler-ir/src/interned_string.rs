// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::GlobalState;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

/// interned string
#[derive(Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
pub struct InternedString(Option<Rc<str>>);

impl fmt::Display for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl fmt::Debug for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl Deref for InternedString {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.as_deref().unwrap_or("")
    }
}

impl InternedString {
    /// get the empty string: `""`
    pub const fn empty() -> Self {
        Self(None)
    }
    /// create a new `InternedString`
    pub fn new(value: &str, global_state: &GlobalState) -> Self {
        if *value == *Self::empty() {
            Self::empty()
        } else {
            Self(Some(global_state.string_interner.intern(value)))
        }
    }
}
