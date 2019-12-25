// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::FloatType;
use crate::IntegerType;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ConstInteger {
    pub value: u64,
    pub integer_type: IntegerType,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConstFloat {
    Float16 { bits: u16 },
    Float32 { bits: u32 },
    Float64 { bits: u64 },
}

impl ConstFloat {
    pub fn get_type(self) -> FloatType {
        match self {
            ConstFloat::Float16 { .. } => FloatType::Float16,
            ConstFloat::Float32 { .. } => FloatType::Float32,
            ConstFloat::Float64 { .. } => FloatType::Float64,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ConstVector<'g> {
    element_type: Interned<'g, Type<'g>>,
    elements: Vec<Interned<'g, Const<'g>>>,
}

impl<'g> ConstVector<'g> {
    pub fn new(elements: Vec<Interned<'g, Const<'g>>>, global_state: &'g GlobalState<'g>) -> Self {
        let mut iter = elements.iter();
        let element_type = iter
            .next()
            .expect("vector must have non-zero size")
            .get()
            .get_type(global_state);
        for element in iter {
            assert_eq!(
                element.get().get_type(global_state),
                element_type,
                "vector must have consistent type"
            );
        }
        ConstVector {
            element_type,
            elements,
        }
    }
    pub fn element_type(&self) -> Interned<'g, Type<'g>> {
        self.element_type
    }
    pub fn elements(&self) -> &[Interned<'g, Const<'g>>] {
        &self.elements
    }
    pub fn get_type(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type> {
        global_state.intern(&Type::Vector {
            element: self.element_type,
            len: self.elements.len(),
        })
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Const<'g> {
    Integer(ConstInteger),
    Float(ConstFloat),
    Bool(bool),
    Vector(ConstVector<'g>),
    // FIXME: add Matrix
    Undef(Interned<'g, Type<'g>>),
}

impl<'g> Const<'g> {
    pub fn get_type(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type> {
        match *self {
            Const::Integer(ConstInteger { integer_type, .. }) => {
                global_state.intern(&Type::Integer { integer_type })
            }
            Const::Float(const_float) => global_state.intern(&Type::Float {
                float_type: const_float.get_type(),
            }),
            Const::Bool(_) => global_state.intern(&Type::Bool),
            Const::Vector(ref const_vector) => const_vector.get_type(global_state),
            Const::Undef(ref retval) => retval.clone(),
        }
    }
}
