// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum IntegerType {
    Int8,
    Int16,
    Int32,
    Int64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FloatType {
    Float16,
    Float32,
    Float64,
}

#[doc(hidden)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Void {}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum OpaqueType<'g> {
    // TODO: implement
    #[doc(hidden)]
    _Unimplemented(&'g (), Void),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Type<'g> {
    Integer {
        integer_type: IntegerType,
    },
    Float {
        float_type: FloatType,
    },
    Bool,
    Pointer {
        pointee: Interned<'g, Type<'g>>,
    },
    Vector {
        len: usize,
        element: Interned<'g, Type<'g>>,
    },
    Matrix {
        columns: usize,
        rows: usize,
        element: Interned<'g, Type<'g>>,
    },
    VariableVector {
        element: Interned<'g, Type<'g>>,
    },
    Opaque {
        opaque_type: OpaqueType<'g>,
    },
}

/// if a type or value `T` is inhabited (is reachable)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Inhabitable<T> {
    /// type or value `T` is inhabited (is reachable)
    Inhabited(T),
    /// uninhabited (unreachable)
    Uninhabited,
}

pub use Inhabitable::*;

impl<T> Inhabitable<T> {
    /// like `Option::as_ref`
    pub fn as_ref(&self) -> Inhabitable<&T> {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    /// like `Option::as_mut`
    pub fn as_mut(&mut self) -> Inhabitable<&mut T> {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    /// like `Option::map`
    pub fn map<F: FnOnce(T) -> R, R>(self, f: F) -> Inhabitable<R> {
        match self {
            Inhabited(v) => Inhabited(f(v)),
            Uninhabited => Uninhabited,
        }
    }
    /// like `Option::as_deref`
    pub fn as_deref(&self) -> Inhabitable<&T::Target>
    where
        T: Deref,
    {
        self.as_ref().map(|v| &**v)
    }
    /// like `Option::as_deref_mut`
    pub fn as_deref_mut(&mut self) -> Inhabitable<&mut T::Target>
    where
        T: DerefMut,
    {
        self.as_mut().map(|v| &mut **v)
    }
    /// return `Some` if `self` is `Inhabited`
    pub fn inhabited(self) -> Option<T> {
        match self {
            Inhabited(v) => Some(v),
            Uninhabited => None,
        }
    }
}
