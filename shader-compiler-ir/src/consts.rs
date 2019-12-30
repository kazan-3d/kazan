// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::from_text::FromTextError;
use crate::from_text::FromTextState;
use crate::from_text::IntegerSuffix;
use crate::from_text::IntegerToken;
use crate::from_text::Keyword;
use crate::from_text::TokenKind;
use crate::prelude::*;
use crate::BoolType;
use crate::FloatType;
use crate::IntegerType;
use crate::VectorType;
use std::convert::TryInto;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConstInteger {
    Int8(u8),
    Int16(u16),
    Int32(u32),
    Int64(u64),
}

pub struct InvalidFloatSize;

impl ConstInteger {
    pub fn bitcast_to_float(self) -> Result<ConstFloat, InvalidFloatSize> {
        match self {
            ConstInteger::Int8(_) => Err(InvalidFloatSize),
            ConstInteger::Int16(v) => Ok(ConstFloat::Float16(v)),
            ConstInteger::Int32(v) => Ok(ConstFloat::Float32(v)),
            ConstInteger::Int64(v) => Ok(ConstFloat::Float64(v)),
        }
    }
    pub fn get_type(self) -> IntegerType {
        match self {
            ConstInteger::Int8(_) => IntegerType::Int8,
            ConstInteger::Int16(_) => IntegerType::Int16,
            ConstInteger::Int32(_) => IntegerType::Int32,
            ConstInteger::Int64(_) => IntegerType::Int64,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConstFloat {
    Float16(u16),
    Float32(u32),
    Float64(u64),
}

impl ConstFloat {
    pub fn get_type(self) -> FloatType {
        match self {
            ConstFloat::Float16(_) => FloatType::Float16,
            ConstFloat::Float32(_) => FloatType::Float32,
            ConstFloat::Float64(_) => FloatType::Float64,
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
        global_state.intern(&Type::Vector(VectorType {
            element: self.element_type,
            scalable: false,
            len: self.elements.len(),
        }))
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Const<'g> {
    Integer(ConstInteger),
    Float(ConstFloat),
    Bool(bool),
    Vector(ConstVector<'g>),
    // FIXME: add scalable vectors
    Undef(Interned<'g, Type<'g>>),
}

impl<'g> Const<'g> {
    pub fn get_type(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type> {
        match *self {
            Const::Integer(const_int) => global_state.intern(&Type::Integer(const_int.get_type())),
            Const::Float(const_float) => global_state.intern(&Type::Float(const_float.get_type())),
            Const::Bool(_) => global_state.intern(&Type::Bool(BoolType)),
            Const::Vector(ref const_vector) => const_vector.get_type(global_state),
            Const::Undef(ref retval) => retval.clone(),
        }
    }
}

impl<'g> FromText<'g> for ConstInteger {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let IntegerToken { value, suffix } = match state.peek_token()?.kind.integer() {
            Some(v) => v,
            _ => state
                .error_at_peek_token("expected integer literal")?
                .into(),
        };
        let retval = match suffix {
            Some(IntegerSuffix::I8) => value.try_into().map(ConstInteger::Int8),
            Some(IntegerSuffix::I16) => value.try_into().map(ConstInteger::Int16),
            Some(IntegerSuffix::I32) => value.try_into().map(ConstInteger::Int32),
            Some(IntegerSuffix::I64) => Ok(value.into()).map(ConstInteger::Int64),
            None => state
                .error_at_peek_token(
                    "integer literal must have type suffix (for example, use `23i32` rather than `23`)",
                )?
                .into(),
        };
        let retval = match retval {
            Ok(retval) => retval,
            Err(_) => state
                .error_at_peek_token("integer literal too big for type")?
                .into(),
        };
        state.parse_token()?;
        Ok(retval)
    }
}

impl<'g> FromText<'g> for ConstFloat {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let float_type = FloatType::from_text(state)?;
        let IntegerToken { value, suffix } = match state.peek_token()?.kind.integer() {
            Some(v) => v,
            _ => state
                .error_at_peek_token("expected integer literal")?
                .into(),
        };
        if suffix != None {
            state.error_at_peek_token("integer literal must not have suffix")?;
        }
        let retval = match float_type {
            FloatType::Float16 => value.try_into().map(ConstFloat::Float16),
            FloatType::Float32 => value.try_into().map(ConstFloat::Float32),
            FloatType::Float64 => Ok(value.into()).map(ConstFloat::Float64),
        };
        let retval = match retval {
            Ok(retval) => retval,
            Err(_) => state
                .error_at_peek_token("integer literal too big for type")?
                .into(),
        };
        state.parse_token()?;
        Ok(retval)
    }
}

impl<'g> FromText<'g> for bool {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let retval = match state.peek_token()?.kind.keyword() {
            Some(Keyword::False) => false,
            Some(Keyword::True) => true,
            _ => state.error_at_peek_token("expected bool literal")?.into(),
        };
        state.parse_token()?;
        Ok(retval)
    }
}

impl<'g> FromText<'g> for ConstVector<'g> {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_parenthesized(
            '<',
            "missing vector constant",
            '>',
            "missing closing angle bracket ('>')",
            |state| -> Result<Self, FromTextError> {
                let element = Interned::<Const>::from_text(state)?;
                let element_type = element.get().get_type(state.global_state());
                let mut elements = vec![element];
                while state.peek_token()?.kind.punct() == Some(',') {
                    state.parse_token()?;
                    let element_location = state.peek_token()?.span;
                    let element = Interned::<Const>::from_text(state)?;
                    if element.get().get_type(state.global_state()) != element_type {
                        state.error_at(element_location, "vector must have consistent type")?;
                    }
                    elements.push(element);
                }
                Ok(ConstVector::new(elements, state.global_state()))
            },
        )
    }
}

impl<'g> FromText<'g> for Const<'g> {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        match state.peek_token()?.kind {
            TokenKind::Integer(_) => Ok(Const::Integer(FromText::from_text(state)?)),
            TokenKind::Keyword(Keyword::F16)
            | TokenKind::Keyword(Keyword::F32)
            | TokenKind::Keyword(Keyword::F64) => Ok(Const::Float(FromText::from_text(state)?)),
            TokenKind::Keyword(Keyword::False) | TokenKind::Keyword(Keyword::True) => {
                Ok(Const::Bool(FromText::from_text(state)?))
            }
            TokenKind::Punct('<') => Ok(Const::Vector(FromText::from_text(state)?)),
            TokenKind::Keyword(Keyword::Undef) => {
                state.parse_token()?;
                Ok(Const::Undef(FromText::from_text(state)?))
            }
            // FIXME: add scalable vectors
            _ => state.error_at_peek_token("missing constant")?.into(),
        }
    }
}

impl<'g> FromText<'g> for Interned<'g, Const<'g>> {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        Ok(state.global_state().intern(&Const::from_text(state)?))
    }
}
