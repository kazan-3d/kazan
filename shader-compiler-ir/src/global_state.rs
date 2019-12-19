// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::debug;
use crate::types::TypeValue;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::rc::Rc;

pub(crate) struct Interner<T: ?Sized>(RefCell<HashSet<Rc<T>>>);

impl<T: Eq + Hash + ?Sized> Interner<T> {
    fn intern_impl<'a, F: FnOnce(&'a T) -> Rc<T>>(&self, value: &'a T, to_rc: F) -> Rc<T> {
        let mut table = self.0.borrow_mut();
        if let Some(retval) = table.get(value) {
            retval.clone()
        } else {
            let retval = to_rc(value);
            table.insert(retval.clone());
            retval
        }
    }
}

impl<T: Eq + Hash + Clone> Interner<T> {
    pub(crate) fn intern(&self, value: &T) -> Rc<T> {
        self.intern_impl(value, |value| Rc::new(value.clone()))
    }
}

impl Interner<str> {
    pub(crate) fn intern(&self, value: &str) -> Rc<str> {
        self.intern_impl(value, Rc::from)
    }
}

/// global state
pub struct GlobalState {
    pub(crate) string_interner: Interner<str>,
    pub(crate) debug_location_interner: Interner<debug::LocationValue>,
    pub(crate) type_interner: Interner<TypeValue>,
}

impl fmt::Debug for GlobalState {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("GlobalState").finish()
    }
}
