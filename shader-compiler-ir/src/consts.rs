// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextState;
use crate::text::IntegerSuffix;
use crate::text::IntegerToken;
use crate::text::Keyword;
use crate::text::Punctuation;
use crate::text::ToTextState;
use crate::text::TokenKind;
use crate::types::PointerType;
use crate::BoolType;
use crate::FloatType;
use crate::IntegerType;
use crate::VectorType;
use std::convert::TryInto;
use std::fmt;

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
    Null(PointerType<'g>),
}

impl<'g> Const<'g> {
    pub fn get_type(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type> {
        match *self {
            Const::Integer(const_int) => global_state.intern(&Type::Integer(const_int.get_type())),
            Const::Float(const_float) => global_state.intern(&Type::Float(const_float.get_type())),
            Const::Bool(_) => global_state.intern(&Type::Bool(BoolType)),
            Const::Vector(ref const_vector) => const_vector.get_type(global_state),
            Const::Undef(ref retval) => retval.clone(),
            Const::Null(ref pointer_type) => {
                global_state.intern(&Type::Pointer(pointer_type.clone()))
            }
        }
    }
}

impl<'g> FromText<'g> for ConstInteger {
    type Parsed = Self;
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

impl<'g> ToText<'g> for ConstInteger {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match *self {
            ConstInteger::Int8(v) => write!(state, "{:#X}i8", v),
            ConstInteger::Int16(v) => write!(state, "{:#X}i16", v),
            ConstInteger::Int32(v) => write!(state, "{:#X}i32", v),
            ConstInteger::Int64(v) => write!(state, "{:#X}i64", v),
        }
    }
}

impl<'g> FromText<'g> for ConstFloat {
    type Parsed = Self;
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

impl<'g> ToText<'g> for ConstFloat {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match *self {
            ConstFloat::Float16(v) => write!(state, "f16 {:#X}", v),
            ConstFloat::Float32(v) => write!(state, "f32 {:#X}", v),
            ConstFloat::Float64(v) => write!(state, "f64 {:#X}", v),
        }
    }
}

impl<'g> FromText<'g> for bool {
    type Parsed = Self;
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

impl<'g> ToText<'g> for bool {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        if *self {
            write!(state, "true")
        } else {
            write!(state, "false")
        }
    }
}

impl<'g> FromText<'g> for ConstVector<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_parenthesized(
            Punctuation::LessThan,
            "missing vector constant",
            Punctuation::GreaterThan,
            "missing closing angle bracket ('>')",
            |state| -> Result<Self, FromTextError> {
                let element = Interned::<Const>::from_text(state)?;
                let element_type = element.get().get_type(state.global_state());
                let mut elements = vec![element];
                while state.peek_token()?.kind.punct() == Some(Punctuation::Comma) {
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

impl<'g> ToText<'g> for ConstVector<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let mut iter = self.elements.iter().copied();
        write!(state, "<")?;
        let first = iter.next().expect("vector must have non-zero size");
        first.to_text(state)?;
        for element in iter {
            write!(state, ", ")?;
            element.to_text(state)?;
        }
        write!(state, ">")
    }
}

impl<'g> FromText<'g> for Const<'g> {
    type Parsed = Interned<'g, Const<'g>>;
    fn from_text(
        state: &mut FromTextState<'g, '_>,
    ) -> Result<Interned<'g, Const<'g>>, FromTextError> {
        let retval = match state.peek_token()?.kind {
            TokenKind::Integer(_) => Const::Integer(ConstInteger::from_text(state)?),
            TokenKind::Keyword(Keyword::F16)
            | TokenKind::Keyword(Keyword::F32)
            | TokenKind::Keyword(Keyword::F64) => Const::Float(ConstFloat::from_text(state)?),
            TokenKind::Keyword(Keyword::False) | TokenKind::Keyword(Keyword::True) => {
                Const::Bool(bool::from_text(state)?)
            }
            TokenKind::Punct(Punctuation::LessThan) => {
                Const::Vector(ConstVector::from_text(state)?)
            }
            TokenKind::Keyword(Keyword::Undef) => {
                state.parse_token()?;
                Const::Undef(Type::from_text(state)?)
            }
            TokenKind::Keyword(Keyword::Null) => {
                state.parse_token()?;
                Const::Null(PointerType::from_text(state)?)
            }
            // FIXME: add scalable vectors
            _ => state.error_at_peek_token("missing constant")?.into(),
        };
        Ok(state.global_state().intern(&retval))
    }
}

impl<'g> ToText<'g> for Const<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            Const::Integer(v) => v.to_text(state),
            Const::Float(v) => v.to_text(state),
            Const::Bool(v) => v.to_text(state),
            Const::Vector(v) => v.to_text(state),
            Const::Undef(ty) => {
                write!(state, "undef ")?;
                ty.to_text(state)
            }
            Const::Null(pointer_type) => {
                write!(state, "null ")?;
                pointer_type.to_text(state)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PointerType;

    #[test]
    fn test_const_from_to_text() {
        let global_state = GlobalState::new();
        macro_rules! test_const {
            ($global_state:ident, $text:literal, $const:expr, $formatted_text:literal) => {
                let parsed_const = Const::parse("", $text, &$global_state).unwrap();
                let expected_const = $global_state.intern(&$const);
                assert_eq!(parsed_const, expected_const);
                let text = expected_const.display().to_string();
                assert_eq!($formatted_text, text);
            };
            ($global_state:ident, $text:literal, $const:expr) => {
                test_const!($global_state, $text, $const, $text);
            };
        }
        test_const!(
            global_state,
            "0i8",
            Const::Integer(ConstInteger::Int8(0)),
            "0x0i8"
        );
        test_const!(
            global_state,
            "10i8",
            Const::Integer(ConstInteger::Int8(10)),
            "0xAi8"
        );
        test_const!(
            global_state,
            "0b10i8",
            Const::Integer(ConstInteger::Int8(2)),
            "0x2i8"
        );
        test_const!(
            global_state,
            "0B10i8",
            Const::Integer(ConstInteger::Int8(2)),
            "0x2i8"
        );
        test_const!(
            global_state,
            "0o10i8",
            Const::Integer(ConstInteger::Int8(8)),
            "0x8i8"
        );
        test_const!(
            global_state,
            "0O10i8",
            Const::Integer(ConstInteger::Int8(8)),
            "0x8i8"
        );
        test_const!(
            global_state,
            "0X10i8",
            Const::Integer(ConstInteger::Int8(0x10)),
            "0x10i8"
        );
        test_const!(
            global_state,
            "0xFFi8",
            Const::Integer(ConstInteger::Int8(0xFF))
        );
        test_const!(global_state, "0x0i8", Const::Integer(ConstInteger::Int8(0)));
        test_const!(
            global_state,
            "0xFFFFi16",
            Const::Integer(ConstInteger::Int16(0xFFFF))
        );
        test_const!(
            global_state,
            "0xFFFFFFFFi32",
            Const::Integer(ConstInteger::Int32(0xFFFF_FFFF))
        );
        test_const!(
            global_state,
            "0xFFFFFFFFFFFFFFFFi64",
            Const::Integer(ConstInteger::Int64(0xFFFF_FFFF_FFFF_FFFF))
        );
        test_const!(
            global_state,
            "f16 0xF000",
            Const::Float(ConstFloat::Float16(0xF000))
        );
        test_const!(
            global_state,
            "f32 0xFF000000",
            Const::Float(ConstFloat::Float32(0xFF00_0000))
        );
        test_const!(
            global_state,
            "f64 0xFF00000000000000",
            Const::Float(ConstFloat::Float64(0xFF00_0000_0000_0000))
        );
        test_const!(
            global_state,
            "<0x1i8>",
            Const::Vector(ConstVector::new(
                vec![global_state.intern(&Const::Integer(ConstInteger::Int8(1)))],
                &global_state
            ))
        );
        test_const!(
            global_state,
            "<0x1i8, 0x2i8>",
            Const::Vector(ConstVector::new(
                vec![
                    global_state.intern(&Const::Integer(ConstInteger::Int8(1))),
                    global_state.intern(&Const::Integer(ConstInteger::Int8(2))),
                ],
                &global_state
            ))
        );
        test_const!(
            global_state,
            "<0x1i8, 0x2i8, 0x3i8, 0x4i8>",
            Const::Vector(ConstVector::new(
                vec![
                    global_state.intern(&Const::Integer(ConstInteger::Int8(1))),
                    global_state.intern(&Const::Integer(ConstInteger::Int8(2))),
                    global_state.intern(&Const::Integer(ConstInteger::Int8(3))),
                    global_state.intern(&Const::Integer(ConstInteger::Int8(4))),
                ],
                &global_state
            ))
        );
        test_const!(
            global_state,
            "undef i8",
            Const::Undef(global_state.intern(&Type::Integer(IntegerType::Int8)))
        );
        test_const!(
            global_state,
            "undef *i8",
            Const::Undef(global_state.intern(&Type::Pointer(PointerType {
                pointee: global_state.intern(&Type::Integer(IntegerType::Int8))
            })))
        );
        test_const!(
            global_state,
            "null *i8",
            Const::Null(PointerType {
                pointee: global_state.intern(&Type::Integer(IntegerType::Int8))
            })
        );
    }
}
