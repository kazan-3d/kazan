// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use core::borrow::Borrow;
use core::convert::TryFrom;
use core::fmt;
use core::iter;
use core::marker::PhantomData;
use core::mem;
use core::slice;
use spirv_parser::Header;
use spirv_parser::IdRef;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct IdOutOfBounds;

impl fmt::Display for IdOutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("SPIR-V Id is out of bounds")
    }
}

pub trait Id: Copy + Eq + 'static {
    const ZERO: Self;
    fn map_index(self) -> Result<usize, IdOutOfBounds>;
    fn from_map_index_checked(index: usize) -> Option<Self>;
    fn from_map_index(index: usize) -> Self {
        if let Some(retval) = Self::from_map_index_checked(index) {
            retval
        } else {
            panic!("index out of range: {}", index);
        }
    }
}

impl Id for IdRef {
    const ZERO: Self = IdRef(0);
    fn map_index(self) -> Result<usize, IdOutOfBounds> {
        usize::try_from(self.0)
            .map_err(|_| IdOutOfBounds)?
            .checked_sub(1)
            .ok_or(IdOutOfBounds)
    }
    fn from_map_index_checked(index: usize) -> Option<Self> {
        u32::try_from(index.checked_add(1)?).map(Self).ok()
    }
}

macro_rules! impl_id {
    ($t:ty) => {
        impl Id for $t {
            const ZERO: Self = Self(IdRef(0));
            fn map_index(self) -> Result<usize, IdOutOfBounds> {
                self.0.map_index()
            }
            fn from_map_index_checked(index: usize) -> Option<Self> {
                IdRef::from_map_index_checked(index).map(Self)
            }
        }
    };
}

impl_id!(spirv_parser::IdResult);
impl_id!(spirv_parser::IdResultType);
impl_id!(spirv_parser::IdMemorySemantics);
impl_id!(spirv_parser::IdScope);

type KeyPhantomData<K> = PhantomData<fn(K) -> K>;

#[derive(Clone)]
pub struct IdMap<K: Id, V> {
    values: Box<[Option<V>]>,
    len: usize,
    _phantom: KeyPhantomData<K>,
}

impl<K: Id, V> IdMap<K, V> {
    pub fn with_bound(id_bound: u32) -> Self {
        let values = (1..id_bound).map(|_| None).collect();
        Self {
            values,
            len: 0,
            _phantom: PhantomData,
        }
    }
    pub fn new<T: Borrow<Header>>(header: T) -> Self {
        Self::with_bound(header.borrow().bound)
    }
    pub fn capacity(&self) -> usize {
        self.values.len()
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            base: self.values.iter().enumerate(),
            _phantom: PhantomData,
        }
    }
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            base: self.values.iter_mut().enumerate(),
            _phantom: PhantomData,
        }
    }
    pub fn drain(&mut self) -> Drain<K, V> {
        Drain {
            base: self.values.iter_mut().enumerate(),
            len: &mut self.len,
            _phantom: PhantomData,
        }
    }
    pub fn drain_filter<F: FnMut(K, &mut V) -> bool>(&mut self, filter: F) -> DrainFilter<K, V, F> {
        DrainFilter {
            base: self.values.iter_mut().enumerate(),
            len: &mut self.len,
            filter,
            _phantom: PhantomData,
        }
    }
    pub fn retain<F: FnMut(K, &mut V) -> bool>(&mut self, mut f: F) {
        self.drain_filter(|k, v| !f(k, v));
    }
    pub fn clear(&mut self) {
        self.drain();
    }
    pub fn get(&self, key: K) -> Result<Option<&V>, IdOutOfBounds> {
        Ok(self
            .values
            .get(key.map_index()?)
            .ok_or(IdOutOfBounds)?
            .as_ref())
    }
    pub fn entry(&mut self, key: K) -> Result<Entry<K, V>, IdOutOfBounds> {
        let entry = self.values.get_mut(key.map_index()?).ok_or(IdOutOfBounds)?;
        match entry {
            None => Ok(Vacant(VacantEntry {
                entry,
                key,
                len: &mut self.len,
            })),
            Some(_) => Ok(Occupied(OccupiedEntry {
                entry,
                key,
                len: &mut self.len,
            })),
        }
    }
    pub fn contains_key(&self, key: K) -> Result<bool, IdOutOfBounds> {
        Ok(self.get(key)?.is_some())
    }
    pub fn get_mut(&mut self, key: K) -> Result<Option<&mut V>, IdOutOfBounds> {
        Ok(self
            .values
            .get_mut(key.map_index()?)
            .ok_or(IdOutOfBounds)?
            .as_mut())
    }
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, IdOutOfBounds> {
        Ok(match self.entry(key)? {
            Vacant(entry) => {
                entry.insert(value);
                None
            }
            Occupied(mut entry) => Some(entry.insert(value)),
        })
    }
    pub fn remove(&mut self, key: K) -> Result<Option<V>, IdOutOfBounds> {
        Ok(match self.entry(key)? {
            Vacant(_) => None,
            Occupied(entry) => Some(entry.remove()),
        })
    }
}

impl<K: Id + fmt::Debug, V: fmt::Debug> fmt::Debug for IdMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

pub struct Iter<'a, K: Id, V> {
    base: iter::Enumerate<slice::Iter<'a, Option<V>>>,
    _phantom: KeyPhantomData<K>,
}

impl<K: Id, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, K: Id, V> Iterator for Iter<'a, K, V> {
    type Item = (K, &'a V);
    fn next(&mut self) -> Option<(K, &'a V)> {
        Some(loop {
            if let (index, Some(v)) = self.base.next()? {
                break (K::from_map_index(index), v);
            }
        })
    }
}

pub struct IterMut<'a, K: Id, V> {
    base: iter::Enumerate<slice::IterMut<'a, Option<V>>>,
    _phantom: KeyPhantomData<K>,
}

impl<'a, K: Id, V> Iterator for IterMut<'a, K, V> {
    type Item = (K, &'a mut V);
    fn next(&mut self) -> Option<(K, &'a mut V)> {
        Some(loop {
            if let (index, Some(v)) = self.base.next()? {
                break (K::from_map_index(index), v);
            }
        })
    }
}

pub struct IntoIter<K: Id, V> {
    base: iter::Enumerate<vec::IntoIter<Option<V>>>,
    _phantom: KeyPhantomData<K>,
}

impl<K: Id, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> {
        Some(loop {
            if let (index, Some(v)) = self.base.next()? {
                break (K::from_map_index(index), v);
            }
        })
    }
}

impl<K: Id, V> IntoIterator for IdMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> IntoIter<K, V> {
        IntoIter {
            base: self.values.into_vec().into_iter().enumerate(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, K: Id, V> IntoIterator for &'a IdMap<K, V> {
    type Item = (K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<'a, K: Id, V> IntoIterator for &'a mut IdMap<K, V> {
    type Item = (K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> IterMut<'a, K, V> {
        self.iter_mut()
    }
}

pub struct DrainFilter<'a, K: Id, V, F: FnMut(K, &mut V) -> bool> {
    base: iter::Enumerate<slice::IterMut<'a, Option<V>>>,
    len: &'a mut usize,
    filter: F,
    _phantom: KeyPhantomData<K>,
}

impl<'a, K: Id, V, F: FnMut(K, &mut V) -> bool> Iterator for DrainFilter<'a, K, V, F> {
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> {
        loop {
            let (index, entry) = self.base.next()?;
            if let Some(value) = entry {
                let key = K::from_map_index(index);
                if (self.filter)(key, value) {
                    let value = entry.take().expect("entry known to be occupied");
                    *self.len -= 1;
                    return Some((key, value));
                }
            }
        }
    }
}

impl<'a, K: Id, V, F: FnMut(K, &mut V) -> bool> Drop for DrainFilter<'a, K, V, F> {
    fn drop(&mut self) {
        self.for_each(mem::drop);
    }
}

pub struct Drain<'a, K: Id, V> {
    base: iter::Enumerate<slice::IterMut<'a, Option<V>>>,
    len: &'a mut usize,
    _phantom: KeyPhantomData<K>,
}

impl<'a, K: Id, V> Iterator for Drain<'a, K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<(K, V)> {
        loop {
            let (index, entry) = self.base.next()?;
            if let Some(value) = entry.take() {
                let key = K::from_map_index(index);
                *self.len -= 1;
                return Some((key, value));
            }
        }
    }
}

impl<'a, K: Id, V> Drop for Drain<'a, K, V> {
    fn drop(&mut self) {
        self.for_each(mem::drop);
    }
}

pub struct Keys<'a, K: Id, V> {
    base: Iter<'a, K, V>,
}

impl<K: Id, V> Iterator for Keys<'_, K, V> {
    type Item = K;
    fn next(&mut self) -> Option<K> {
        self.base.next().map(|v| v.0)
    }
}

pub struct Values<'a, K: Id, V> {
    base: Iter<'a, K, V>,
}

impl<'a, K: Id, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<&'a V> {
        self.base.next().map(|v| v.1)
    }
}

pub struct ValuesMut<'a, K: Id, V> {
    base: IterMut<'a, K, V>,
}

impl<'a, K: Id, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<&'a mut V> {
        self.base.next().map(|v| v.1)
    }
}

pub struct VacantEntry<'a, K: Id, V> {
    key: K,
    entry: &'a mut Option<V>,
    len: &'a mut usize,
}

impl<'a, K: Id, V> VacantEntry<'a, K, V> {
    pub fn key(&self) -> K {
        self.key
    }
    pub fn insert(self, value: V) -> &'a mut V {
        debug_assert!(self.entry.is_none());
        *self.entry = Some(value);
        *self.len += 1;
        self.entry.as_mut().expect("just set to Some")
    }
}

impl<K: Id + fmt::Debug, V> fmt::Debug for VacantEntry<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("VacantEntry").field(&self.key()).finish()
    }
}

pub struct OccupiedEntry<'a, K: Id, V> {
    key: K,
    entry: &'a mut Option<V>,
    len: &'a mut usize,
}

impl<'a, K: Id, V> OccupiedEntry<'a, K, V> {
    pub fn key(&self) -> K {
        self.key
    }
    pub fn get(&self) -> &V {
        self.entry.as_ref().expect("entry known to be occupied")
    }
    pub fn get_mut(&mut self) -> &mut V {
        self.entry.as_mut().expect("entry known to be occupied")
    }
    pub fn insert(&mut self, value: V) -> V {
        mem::replace(self.get_mut(), value)
    }
    pub fn into_mut(self) -> &'a mut V {
        self.entry.as_mut().expect("entry known to be occupied")
    }
    pub fn remove(self) -> V {
        self.remove_entry().1
    }
    pub fn remove_entry(self) -> (K, V) {
        let value = self.entry.take().expect("entry known to be occupied");
        *self.len -= 1;
        (self.key, value)
    }
}

impl<K: Id + fmt::Debug, V: fmt::Debug> fmt::Debug for OccupiedEntry<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("key", &self.key())
            .field("value", self.get())
            .finish()
    }
}

pub enum Entry<'a, K: Id, V> {
    Vacant(VacantEntry<'a, K, V>),
    Occupied(OccupiedEntry<'a, K, V>),
}

pub use Entry::Occupied;
pub use Entry::Vacant;

impl<'a, K: Id, V> Entry<'a, K, V> {
    pub fn key(&self) -> K {
        match self {
            Vacant(v) => v.key(),
            Occupied(v) => v.key(),
        }
    }
    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Occupied(ref mut entry) = self {
            f(entry.get_mut());
        }
        self
    }
    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            Vacant(entry) => entry.insert(f()),
            Occupied(entry) => entry.into_mut(),
        }
    }
    pub fn or_insert(self, value: V) -> &'a mut V {
        self.or_insert_with(|| value)
    }
    pub fn or_insert_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert_with(Default::default)
    }
}

impl<K: Id + fmt::Debug, V: fmt::Debug> fmt::Debug for Entry<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Vacant(v) => f.debug_tuple("Entry").field(v).finish(),
            Occupied(v) => f.debug_tuple("Entry").field(v).finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_map<K: Id + fmt::Debug, V: Eq + Copy + fmt::Debug>(
        map: &mut IdMap<K, V>,
        expected_map: &[(K, V)],
    ) {
        let map_entries: Vec<_> = map.iter().map(|(k, &v)| (k, v)).collect();
        let map_entries2: Vec<_> = map.iter_mut().map(|(k, v)| (k, *v)).collect();
        assert_eq!(map_entries.len(), map.len());
        assert_eq!(map_entries, expected_map);
        assert_eq!(map_entries2, expected_map);
    }

    #[test]
    fn test_map() {
        const BOUND: u32 = 10;
        let mut map = IdMap::<IdRef, i32>::with_bound(BOUND);
        let map = &mut map;
        check_map(map, &[]);
        assert_eq!(map.insert(IdRef(0), 0), Err(IdOutOfBounds));
        check_map(map, &[]);
        assert_eq!(map.insert(IdRef(1), 0), Ok(None));
        check_map(map, &[(IdRef(1), 0)]);
        assert_eq!(map.insert(IdRef(1), 2), Ok(Some(0)));
        check_map(map, &[(IdRef(1), 2)]);
        assert_eq!(map.insert(IdRef(BOUND), 4), Err(IdOutOfBounds));
        check_map(map, &[(IdRef(1), 2)]);
        assert_eq!(map.insert(IdRef(BOUND - 1), 4), Ok(None));
        check_map(map, &[(IdRef(1), 2), (IdRef(BOUND - 1), 4)]);
        assert_eq!(map.remove(IdRef(BOUND - 1)), Ok(Some(4)));
        check_map(map, &[(IdRef(1), 2)]);
        assert_eq!(map.remove(IdRef(BOUND - 1)), Ok(None));
        check_map(map, &[(IdRef(1), 2)]);
        assert_eq!(map.remove(IdRef(0)), Err(IdOutOfBounds));
        check_map(map, &[(IdRef(1), 2)]);
        assert_eq!(map.remove(IdRef(!0)), Err(IdOutOfBounds));
        check_map(map, &[(IdRef(1), 2)]);
        // TODO: add more tests
    }
}
