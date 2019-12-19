// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

//! debugger support

use crate::GlobalState;
use crate::InternedString;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

/// a debug location; you're probably looking for `Location` instead
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LocationData {
    /// the source file name
    pub file: InternedString,
    /// the line number
    pub line: u32,
    /// the column number
    pub column: u32,
}

impl LocationData {
    /// create an empty `LocationValue`
    pub const fn empty() -> Self {
        Self {
            file: InternedString::empty(),
            line: 0,
            column: 0,
        }
    }
    /// return `true` if `self` is empty
    pub fn is_empty(&self) -> bool {
        *self == Self::empty()
    }
}

/// a debug location
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Location(Option<Rc<LocationData>>);

impl Location {
    /// create an empty `Location`
    pub const fn empty() -> Self {
        Self(None)
    }
    /// return `true` if `self` is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }
    /// create a new `Location`
    pub fn new(value: &LocationData, global_state: &GlobalState) -> Self {
        if *value == LocationData::empty() {
            Self(None)
        } else {
            Self(Some(global_state.debug_location_interner.intern(value)))
        }
    }
}

impl Deref for Location {
    type Target = LocationData;
    fn deref(&self) -> &LocationData {
        const EMPTY: &LocationData = &LocationData::empty();
        self.0.as_deref().unwrap_or(EMPTY)
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        macro_rules! debug_fields {
            (
                $($field:ident,)+
            ) => {
                {
                    let LocationData {
                        $($field,)+
                    } = &**self;
                    f.debug_struct("Location")
                    $(.field(stringify!($field), $field))+
                    .finish()
                }
            };
        }
        debug_fields! {
            file,
            line,
            column,
        }
    }
}
