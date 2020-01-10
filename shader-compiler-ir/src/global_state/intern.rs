// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

// use custom hashtable to work-around #[may_dangle] being unstable

use crate::global_state::Interned;
use ahash::ABuildHasher;
use alloc::boxed::Box;
use core::cell::Cell;
use core::cell::RefCell;
use core::hash::BuildHasher;
use core::hash::Hash;
use core::hash::Hasher;
use core::mem;
use typed_arena::Arena;

pub(super) trait InternStorage {
    type Arena: Default;
    fn alloc<'g>(arena: &'g Self::Arena, value: &Self) -> &'g Self;
}

impl InternStorage for str {
    type Arena = Arena<u8>;
    fn alloc<'g>(arena: &'g Self::Arena, value: &Self) -> &'g Self {
        arena.alloc_str(value)
    }
}

impl<T: Clone> InternStorage for T {
    type Arena = Arena<T>;
    fn alloc<'g>(arena: &'g Self::Arena, value: &Self) -> &'g Self {
        arena.alloc(value.clone())
    }
}

pub(super) struct Interner<'g, T: InternStorage + Hash + Eq + ?Sized> {
    value_arena: T::Arena,
    value_hashtable: RefCell<Box<[Option<&'g T>]>>,
    value_count: Cell<usize>,
    build_hasher: ABuildHasher,
}

struct HashChainIndexes {
    mask: usize,
    index: usize,
}

impl Iterator for HashChainIndexes {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let retval = self.index;
        self.index = self.index.wrapping_add(1) & self.mask;
        Some(retval)
    }
}

impl<'g, T: Hash + Eq + InternStorage + ?Sized> Interner<'g, T> {
    pub(super) fn new() -> Self {
        Self {
            value_arena: Default::default(),
            value_hashtable: Default::default(),
            value_count: Cell::new(0),
            build_hasher: ABuildHasher::new(),
        }
    }
    fn get_hashchain_indexes(&self, hashtable_len: usize, value: &T) -> HashChainIndexes {
        debug_assert!(hashtable_len.is_power_of_two());
        let mask = hashtable_len - 1;
        let mut hasher = self.build_hasher.build_hasher();
        value.hash(&mut hasher);
        HashChainIndexes {
            index: hasher.finish() as usize & mask,
            mask,
        }
    }
    fn expand_hashtable(&'g self, value_hashtable: &mut Box<[Option<&'g T>]>) {
        let new_size = value_hashtable
            .len()
            .checked_mul(2)
            .expect("hashtable too big")
            .max(1024);
        debug_assert!(new_size.is_power_of_two());
        debug_assert!(value_hashtable.len() < new_size);
        let old_hashtable = mem::replace(value_hashtable, (0..new_size).map(|_| None).collect());
        for value in old_hashtable.into_vec() {
            if let Some(value) = value {
                for index in self.get_hashchain_indexes(new_size, value) {
                    // loop is guaranteed to terminate since all indexes are
                    // visited and we have more indexes than values
                    match &mut value_hashtable[index] {
                        Some(_) => {}
                        entry @ None => {
                            *entry = Some(value);
                            break;
                        }
                    }
                }
            }
        }
    }
    fn needs_expand(&'g self, hashtable_len: usize) -> bool {
        // calculate hashtable_len * 7/8 without overflowing
        let limit = hashtable_len - hashtable_len / 8;

        // specifically include the case where hashtable_len == 0
        self.value_count.get() >= limit
    }
    pub(super) fn intern(&'g self, value: &T) -> Interned<'g, T> {
        let mut value_hashtable = self.value_hashtable.borrow_mut();
        if self.needs_expand(value_hashtable.len()) {
            self.expand_hashtable(&mut value_hashtable);
        }
        for index in self.get_hashchain_indexes(value_hashtable.len(), value) {
            // loop is guaranteed to terminate since all indexes are visited and
            // we have more indexes than values since we called expand_hashtable above
            match value_hashtable[index] {
                Some(entry) => {
                    if entry == value {
                        return Interned(entry);
                    }
                }
                ref mut entry @ None => {
                    let retval = InternStorage::alloc(&self.value_arena, value);
                    *entry = Some(retval);
                    self.value_count.set(self.value_count.get() + 1);
                    return Interned(retval);
                }
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::btree_map::{BTreeMap, Entry};
    use alloc::string::String;
    use alloc::string::ToString;

    #[test]
    fn test_interner() {
        let interner = Interner::<str>::new();
        let mut strings = BTreeMap::new();
        let mut check = |v: &str| {
            let interned = interner.intern(v);
            assert_eq!(*interned, *v);
            match strings.entry(String::from(v)) {
                Entry::Vacant(entry) => {
                    entry.insert(interned);
                }
                Entry::Occupied(entry) => {
                    assert_eq!(entry.get().get() as *const str, interned.get());
                }
            }
        };
        for _ in 0..3 {
            check("abc");
            check("");
            check("123");
            check("\0");
            check("\u{C4}"); // non-ascii
            for i in 0..50 {
                check(&i.to_string());
            }
        }
    }
}
