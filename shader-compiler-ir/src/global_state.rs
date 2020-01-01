// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextState;
use crate::text::ToTextState;
use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ptr::NonNull;
use typed_arena::Arena;

pub struct GlobalState<'g> {
    string_byte_arena: Arena<u8>,
    string_hashtable: RefCell<HashSet<&'g str>>,
    location_arena: Arena<Location<'g>>,
    location_hashtable: RefCell<HashSet<&'g Location<'g>>>,
    type_arena: Arena<Type<'g>>,
    type_hashtable: RefCell<HashSet<&'g Type<'g>>>,
    const_arena: Arena<Const<'g>>,
    const_hashtable: RefCell<HashSet<&'g Const<'g>>>,
    value_arena: Arena<Value<'g>>,
    block_arena: Arena<Block<'g>>,
    loop_arena: Arena<Loop<'g>>,
}

impl<'g> GlobalState<'g> {
    /// create a new `GlobalState`
    pub fn new() -> Self {
        Self {
            string_byte_arena: Arena::new(),
            string_hashtable: RefCell::new(HashSet::new()),
            location_arena: Arena::new(),
            location_hashtable: RefCell::new(HashSet::new()),
            type_arena: Arena::new(),
            type_hashtable: RefCell::new(HashSet::new()),
            const_arena: Arena::new(),
            const_hashtable: RefCell::new(HashSet::new()),
            value_arena: Arena::new(),
            block_arena: Arena::new(),
            loop_arena: Arena::new(),
        }
    }
}

impl<'g> Default for GlobalState<'g> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
pub struct Private {
    _private: (),
}

impl Private {
    const fn new() -> Self {
        Self { _private: () }
    }
}

pub trait Id<'g> {}

pub trait IdMethod<'g>: Id<'g> {
    fn id(&'g self) -> NonNull<Self> {
        self.into()
    }
}

impl<'g, T: Id<'g>> IdMethod<'g> for T {}

#[repr(transparent)]
pub struct IdRef<'g, T: Id<'g>>(&'g T);

impl<'g, T: fmt::Debug + Id<'g>> fmt::Debug for IdRef<'g, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Omitted;
        impl fmt::Debug for Omitted {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad("<omitted>")
            }
        }
        struct NumericId(u64);
        impl fmt::Debug for NumericId {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "#{}", self.0)
            }
        }
        #[derive(Default)]
        struct InProgressIds {
            map: HashMap<NonNull<u8>, u64>,
            next_numeric_id: u64,
        }
        thread_local! {
            static IN_PROGRESS_IDS: RefCell<InProgressIds> = RefCell::default();
        }
        struct RemoveOnDrop(NonNull<u8>);
        impl Drop for RemoveOnDrop {
            fn drop(&mut self) {
                let _ = IN_PROGRESS_IDS
                    .try_with(|in_progress_ids| in_progress_ids.borrow_mut().map.remove(&self.0));
            }
        }
        let id = self.id().cast();
        let (inserted, numeric_id) = IN_PROGRESS_IDS.with(|in_progress_ids| {
            let mut in_progress_ids = in_progress_ids.borrow_mut();
            let InProgressIds {
                map,
                next_numeric_id,
            } = &mut *in_progress_ids;
            if map.is_empty() {
                *next_numeric_id = 1;
            }
            match map.entry(id) {
                Entry::Vacant(entry) => {
                    let numeric_id = *next_numeric_id;
                    *next_numeric_id += 1;
                    entry.insert(numeric_id);
                    (true, numeric_id)
                }
                Entry::Occupied(entry) => (false, *entry.get()),
            }
        });
        let remove_on_drop = if inserted {
            Some(RemoveOnDrop(id))
        } else {
            None
        };
        let mut debug_helper = f.debug_tuple("IdRef");
        debug_helper.field(&NumericId(numeric_id));
        if inserted {
            debug_helper.field(self.get());
        } else {
            debug_helper.field(&Omitted);
        }
        let retval = debug_helper.finish();
        std::mem::drop(remove_on_drop);
        retval
    }
}

impl<'g, T: Id<'g>> IdRef<'g, T> {
    pub fn id(self) -> NonNull<T> {
        self.0.id()
    }
    pub fn get(self) -> &'g T {
        self.0
    }
}

impl<'g, T: Id<'g>> Deref for IdRef<'g, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<'g, T: Id<'g>> Eq for IdRef<'g, T> {}

impl<'g, T: Id<'g>> Copy for IdRef<'g, T> {}

impl<'g, T: Id<'g>> Clone for IdRef<'g, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'g, T: Id<'g>> PartialEq for IdRef<'g, T> {
    fn eq(&self, rhs: &IdRef<'g, T>) -> bool {
        self.id() == rhs.id()
    }
}

impl<'g, T: Id<'g>> Hash for IdRef<'g, T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.id().hash(h)
    }
}

/// allocate value from `GlobalState`
pub trait Allocate<'g, T: Id<'g>> {
    #[doc(hidden)]
    fn alloc_private(&'g self, _private: Private, value: T) -> &'g T;
    /// allocate value from `GlobalState`
    #[must_use]
    fn alloc(&'g self, value: T) -> IdRef<'g, T> {
        IdRef(self.alloc_private(Private::new(), value))
    }
}

#[repr(transparent)]
pub struct Interned<'g, T: ?Sized + Eq + Hash>(&'g T);

impl<'g, T: ?Sized + Eq + Hash + FromText<'g, Parsed = Self>> FromText<'g> for Interned<'g, T> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        T::from_text(state)
    }
}

impl<'g, T: ?Sized + Eq + Hash + ToText<'g>> ToText<'g> for Interned<'g, T> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        (**self).to_text(state)
    }
}

impl<T: ?Sized + Eq + Hash> Eq for Interned<'_, T> {}

impl<T: ?Sized + Eq + Hash> Copy for Interned<'_, T> {}

impl<T: ?Sized + Eq + Hash> Clone for Interned<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + Eq + Hash> PartialEq for Interned<'_, T> {
    fn eq(&self, rhs: &Self) -> bool {
        std::ptr::eq(self.0, rhs.0)
    }
}

impl<T: ?Sized + Eq + Hash> Hash for Interned<'_, T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        (self.0 as *const T).hash(h)
    }
}

impl<T: ?Sized + Eq + Hash> Deref for Interned<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T: ?Sized + Eq + Hash> AsRef<T> for Interned<'_, T> {
    fn as_ref(&self) -> &T {
        self.0
    }
}

impl<'g, T: ?Sized + Eq + Hash> Interned<'g, T> {
    pub fn get(self) -> &'g T {
        self.0
    }
}

impl<T: ?Sized + Eq + Hash + fmt::Debug> fmt::Debug for Interned<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ?Sized + Eq + Hash + fmt::Display> fmt::Display for Interned<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub trait Intern<'g, T: ?Sized + Eq + Hash> {
    #[doc(hidden)]
    fn intern_alloc(&'g self, _private: Private, value: &T) -> &'g T;
    #[doc(hidden)]
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g T>>;
    #[must_use]
    fn intern<'a>(&'g self, value: &'a T) -> Interned<'g, T> {
        let mut hashtable = self.get_hashtable(Private::new()).borrow_mut();
        if let Some(retval) = hashtable.get(value) {
            Interned(retval)
        } else {
            let retval = self.intern_alloc(Private::new(), value);
            let inserted = hashtable.insert(retval);
            assert!(inserted);
            Interned(retval)
        }
    }
}

impl<'g> Intern<'g, str> for GlobalState<'g> {
    fn intern_alloc(&'g self, _private: Private, value: &str) -> &'g str {
        self.string_byte_arena.alloc_str(value)
    }
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g str>> {
        &self.string_hashtable
    }
}

impl<'g> Allocate<'g, Value<'g>> for GlobalState<'g> {
    fn alloc_private(&'g self, _private: Private, value: Value<'g>) -> &'g Value<'g> {
        self.value_arena.alloc(value)
    }
}

impl<'g> Intern<'g, Const<'g>> for GlobalState<'g> {
    fn intern_alloc(&'g self, _private: Private, value: &Const<'g>) -> &'g Const<'g> {
        self.const_arena.alloc(value.clone())
    }
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g Const<'g>>> {
        &self.const_hashtable
    }
}

impl<'g> Intern<'g, Location<'g>> for GlobalState<'g> {
    fn intern_alloc(&'g self, _private: Private, value: &Location<'g>) -> &'g Location<'g> {
        self.location_arena.alloc(*value)
    }
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g Location<'g>>> {
        &self.location_hashtable
    }
}

impl<'g> Intern<'g, Type<'g>> for GlobalState<'g> {
    fn intern_alloc(&'g self, _private: Private, value: &Type<'g>) -> &'g Type<'g> {
        self.type_arena.alloc(value.clone())
    }
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g Type<'g>>> {
        &self.type_hashtable
    }
}

impl<'g> Allocate<'g, Loop<'g>> for GlobalState<'g> {
    fn alloc_private(&'g self, _private: Private, value: Loop<'g>) -> &'g Loop<'g> {
        self.loop_arena.alloc(value)
    }
}

impl<'g> Allocate<'g, Block<'g>> for GlobalState<'g> {
    fn alloc_private(&'g self, _private: Private, value: Block<'g>) -> &'g Block<'g> {
        self.block_arena.alloc(value)
    }
}
