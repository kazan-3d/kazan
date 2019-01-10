// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use spirv_parser::IdRef;
use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::iter;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign};
use std::slice;

#[derive(Clone, Debug)]
enum Impl {
    Empty,
    One(IdRef),
    Many(Vec<IdRef>),
}

#[derive(Clone)]
pub struct VariableSet(Impl);

impl VariableSet {
    pub fn new() -> Self {
        VariableSet(Impl::Empty)
    }
    pub fn as_slice(&self) -> &[IdRef] {
        match self.0 {
            Impl::Empty => &[],
            Impl::One(ref value) => slice::from_ref(value),
            Impl::Many(ref values) => &**values,
        }
    }
    pub fn iter(&self) -> Iter {
        Iter(self.as_slice())
    }
    pub fn len(&self) -> usize {
        match self.0 {
            Impl::Empty => 0,
            Impl::One(_) => 1,
            Impl::Many(ref values) => values.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn retain<F: FnMut(IdRef) -> bool>(&mut self, mut f: F) {
        match self.0 {
            Impl::Empty => {}
            Impl::One(value) => {
                if !f(value) {
                    self.0 = Impl::Empty;
                }
            }
            Impl::Many(ref mut values) => {
                values.retain(|&v| f(v));
                match **values {
                    [] => self.0 = Impl::Empty,
                    [value] => self.0 = Impl::One(value),
                    _ => {}
                }
            }
        }
    }
}

impl iter::FromIterator<IdRef> for VariableSet {
    fn from_iter<I: IntoIterator<Item = IdRef>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        if let Some(first) = iter.next() {
            if let Some(second) = iter.next() {
                let mut values: Vec<IdRef> = iter::once(first)
                    .chain(iter::once(second))
                    .chain(iter)
                    .collect();
                values.sort_unstable_by_key(|value| value.0);
                values.dedup();
                VariableSet(Impl::Many(values))
            } else {
                VariableSet(Impl::One(first))
            }
        } else {
            VariableSet(Impl::Empty)
        }
    }
}

impl From<IdRef> for VariableSet {
    fn from(value: IdRef) -> Self {
        VariableSet(Impl::One(value))
    }
}

impl Default for VariableSet {
    fn default() -> Self {
        VariableSet::new()
    }
}

impl fmt::Debug for VariableSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<'a> IntoIterator for &'a VariableSet {
    type Item = IdRef;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}

impl PartialEq for VariableSet {
    fn eq(&self, rhs: &Self) -> bool {
        self.as_slice() == rhs.as_slice()
    }
}

impl Eq for VariableSet {}

impl Hash for VariableSet {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.as_slice().hash(h)
    }
}

macro_rules! impl_ops {
    ($op_trait:ident, $op_assign_trait:ident, $op:ident, $op_assign:ident) => {
        impl $op_assign_trait for VariableSet {
            fn $op_assign(&mut self, rhs: Self) {
                self.$op_assign(&rhs)
            }
        }

        impl $op_trait for &'_ VariableSet {
            type Output = VariableSet;
            fn $op(self, rhs: Self) -> VariableSet {
                let mut retval = self.clone();
                retval.$op_assign(rhs);
                retval
            }
        }

        impl $op_trait for VariableSet {
            type Output = VariableSet;
            fn $op(mut self, rhs: Self) -> VariableSet {
                self.$op_assign(rhs);
                self
            }
        }

        impl $op_trait<&'_ VariableSet> for VariableSet {
            type Output = VariableSet;
            fn $op(mut self, rhs: &VariableSet) -> VariableSet {
                self.$op_assign(rhs);
                self
            }
        }

        impl $op_trait<VariableSet> for &'_ VariableSet {
            type Output = VariableSet;
            fn $op(self, mut rhs: VariableSet) -> VariableSet {
                rhs.$op_assign(self);
                rhs
            }
        }
    };
}

impl BitAndAssign<&'_ VariableSet> for VariableSet {
    fn bitand_assign(&mut self, rhs: &VariableSet) {
        let mut iter = rhs.iter().peekable();
        self.retain(|value| loop {
            match iter.peek().cloned() {
                None => break false,
                Some(rhs_value) => {
                    if value.0 < rhs_value.0 {
                        break false;
                    } else if value == rhs_value {
                        iter.next();
                        break true;
                    } else {
                        iter.next();
                    }
                }
            }
        });
    }
}

impl_ops!(BitAnd, BitAndAssign, bitand, bitand_assign);

impl BitOrAssign<&'_ VariableSet> for VariableSet {
    fn bitor_assign(&mut self, rhs: &VariableSet) {
        #[allow(clippy::suspicious_op_assign_impl)] // false positive
        match self.0 {
            Impl::Empty => *self = rhs.clone(),
            Impl::One(value) => match rhs.0 {
                Impl::Empty => {}
                Impl::One(rhs_value) => match value.0.cmp(&rhs_value.0) {
                    Ordering::Less => self.0 = Impl::Many(vec![value, rhs_value]),
                    Ordering::Equal => {}
                    Ordering::Greater => self.0 = Impl::Many(vec![rhs_value, value]),
                },
                Impl::Many(ref rhs_values) => {
                    let mut values = Vec::with_capacity(rhs_values.len() + 1);
                    values.push(value);
                    values.extend_from_slice(rhs_values);
                    values.sort_unstable_by_key(|v| v.0);
                    values.dedup_by_key(|v| v.0);
                    self.0 = Impl::Many(values);
                }
            },
            Impl::Many(ref mut values) => {
                if rhs.is_empty() {
                    return;
                }
                values.extend_from_slice(rhs.as_slice());
                values.sort_unstable_by_key(|v| v.0);
                values.dedup_by_key(|v| v.0);
            }
        }
    }
}

impl_ops!(BitOr, BitOrAssign, bitor, bitor_assign);

impl SubAssign<&'_ VariableSet> for VariableSet {
    fn sub_assign(&mut self, rhs: &VariableSet) {
        let mut iter = rhs.iter().peekable();
        self.retain(|value| loop {
            match iter.peek().cloned() {
                None => break true,
                Some(rhs_value) => {
                    if value.0 < rhs_value.0 {
                        break true;
                    } else if value == rhs_value {
                        iter.next();
                        break false;
                    } else {
                        iter.next();
                    }
                }
            }
        });
    }
}

impl_ops!(Sub, SubAssign, sub, sub_assign);

#[derive(Clone, Debug)]
pub struct IntoIter {
    set: VariableSet,
    index: usize,
}

impl Iterator for IntoIter {
    type Item = IdRef;
    fn next(&mut self) -> Option<IdRef> {
        if self.index >= self.set.len() {
            None
        } else {
            let retval = self.set.as_slice()[self.index];
            self.index += 1;
            Some(retval)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.set.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<IdRef> {
        match self.set.0 {
            Impl::Empty => None,
            Impl::One(value) => {
                if self.index == 0 {
                    self.set.0 = Impl::Empty;
                    Some(value)
                } else {
                    None
                }
            }
            Impl::Many(ref mut values) => {
                if self.index >= values.len() {
                    None
                } else {
                    Some(values.pop().unwrap())
                }
            }
        }
    }
}

impl ExactSizeIterator for IntoIter {}

impl iter::FusedIterator for IntoIter {}

#[derive(Clone, Debug)]
pub struct Iter<'a>(&'a [IdRef]);

impl Iterator for Iter<'_> {
    type Item = IdRef;
    fn next(&mut self) -> Option<IdRef> {
        let (&retval, rest) = self.0.split_first()?;
        self.0 = rest;
        Some(retval)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<IdRef> {
        let (&retval, rest) = self.0.split_last()?;
        self.0 = rest;
        Some(retval)
    }
}

impl ExactSizeIterator for Iter<'_> {}

impl iter::FusedIterator for Iter<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Borrow;

    fn to_bits<T: Borrow<VariableSet>>(set: T) -> u32 {
        let mut retval = 0;
        for v in set.borrow() {
            assert!(v.0 <= 32);
            let bit = 1 << (v.0 - 1);
            assert!(bit > retval); // values are in ascending order
            retval |= bit;
        }
        retval
    }

    fn from_bits(bits: u32) -> VariableSet {
        let mut retval = VariableSet::new();
        for i in 0..31 {
            if (bits & (1 << i)) != 0 {
                retval |= VariableSet::from(IdRef(i + 1));
            }
        }
        retval
    }

    fn make_set_vec() -> Vec<VariableSet> {
        (0..0x40).map(from_bits).collect()
    }

    #[test]
    fn test_construct() {
        for (bits, set) in make_set_vec().iter().enumerate() {
            assert_eq!(bits as u32, to_bits(set));
        }
    }

    #[test]
    fn test_ops() {
        let set_vec = make_set_vec();
        for (l, l_set) in set_vec.iter().enumerate() {
            for (r, r_set) in set_vec.iter().enumerate() {
                assert_eq!(l as u32 & r as u32, to_bits(l_set & r_set));
                assert_eq!(l as u32 | r as u32, to_bits(l_set | r_set));
                assert_eq!(l as u32 & !r as u32, to_bits(l_set - r_set));
            }
        }
    }
}
