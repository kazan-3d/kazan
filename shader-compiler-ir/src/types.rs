// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

//! intermediate representation for value types

use crate::global_state::GlobalState;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone)]
pub struct Type(Rc<TypeValue>);

impl PartialEq for Type {
    fn eq(&self, rhs: &Self) -> bool {
        Rc::ptr_eq(&self.0, &rhs.0)
    }
}

impl Eq for Type {}

impl Hash for Type {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (&*self.0 as *const TypeValue).hash(hasher)
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl Deref for Type {
    type Target = TypeValue;
    fn deref(&self) -> &TypeValue {
        &*self.0
    }
}

impl Type {
    pub fn get(value: &TypeValue, global_state: &GlobalState) -> Type {
        Type(global_state.type_interner.intern(value))
    }
}

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

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum OpaqueType {}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TypeValue {
    Integer {
        integer_type: IntegerType,
    },
    Float {
        float_type: FloatType,
    },
    Bool,
    Pointer {
        pointee: Type,
    },
    Vector {
        size: usize,
        element: Type,
    },
    Matrix {
        columns: usize,
        rows: usize,
        element: Type,
    },
    VariableVector {
        element: Type,
    },
    Opaque {
        opaque_type: OpaqueType,
    },
}
