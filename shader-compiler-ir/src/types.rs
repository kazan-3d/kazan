// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::from_text::FromTextError;
use crate::from_text::FromTextState;
use crate::from_text::IntegerToken;
use crate::from_text::Keyword;
use crate::from_text::Punctuation;
use crate::from_text::TokenKind;
use crate::prelude::*;
use std::convert::TryInto;
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
pub struct BoolType;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PointerType<'g> {
    pub pointee: Interned<'g, Type<'g>>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VectorType<'g> {
    pub len: usize,
    pub scalable: bool,
    pub element: Interned<'g, Type<'g>>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Type<'g> {
    Integer(IntegerType),
    Float(FloatType),
    Bool(BoolType),
    Pointer(PointerType<'g>),
    Vector(VectorType<'g>),
    Opaque(OpaqueType<'g>),
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

macro_rules! impl_from_text_for_keyword_type {
    ($type:ident {
        $($kw:ident => $value:expr,)+
        _ => $error_msg:expr,
    }) => {
        impl<'g> FromText<'g> for $type {
            fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
                let retval = match state.peek_token()?.kind {
                    $(TokenKind::Keyword(Keyword::$kw) => $value,)+
                    _ => state.error_at_peek_token($error_msg)?.into(),
                };
                state.parse_token()?;
                Ok(retval)
            }
        }
    };
}

impl_from_text_for_keyword_type! {
    IntegerType {
        I8 => IntegerType::Int8,
        I16 => IntegerType::Int16,
        I32 => IntegerType::Int32,
        I64 => IntegerType::Int64,
        _ => "invalid integer type",
    }
}

impl_from_text_for_keyword_type! {
    FloatType {
        F16 => FloatType::Float16,
        F32 => FloatType::Float32,
        F64 => FloatType::Float64,
        _ => "invalid float type",
    }
}

impl_from_text_for_keyword_type! {
    BoolType {
        Bool => BoolType,
        _ => "invalid bool type",
    }
}

impl<'g> FromText<'g> for VectorType<'g> {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_parenthesized(
            Punctuation::LessThan,
            "missing opening angle bracket: Punctuation::LessThan",
            Punctuation::GreaterThan,
            "missing closing angle bracket: '>'",
            |state| -> Result<VectorType<'g>, FromTextError> {
                let scalable = if let TokenKind::Keyword(Keyword::VScale) = state.peek_token()?.kind
                {
                    state.parse_token()?;
                    state.parse_keyword_token_or_error(Keyword::X, "missing x after vscale")?;
                    true
                } else {
                    false
                };
                let len = state.parse_token()?;
                let len: usize = match len.kind {
                    TokenKind::Integer(IntegerToken { value, suffix }) => {
                        if suffix.is_some() {
                            state.error_at(
                                len.span,
                                "vector length value must not have type suffix",
                            )?;
                        }
                        match value.try_into() {
                            Ok(len) => len,
                            Err(_) => state
                                .error_at(len.span, "vector length value too big")?
                                .into(),
                        }
                    }
                    _ => state
                        .error_at(len.span, "missing vector length value")?
                        .into(),
                };
                state.parse_keyword_token_or_error(Keyword::X, "missing x after vscale")?;
                Ok(VectorType {
                    len,
                    scalable,
                    element: FromText::from_text(state)?,
                })
            },
        )
    }
}

impl<'g> FromText<'g> for PointerType<'g> {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_punct_token_or_error(Punctuation::Asterisk, "expected pointer type")?;
        Ok(PointerType {
            pointee: FromText::from_text(state)?,
        })
    }
}

impl<'g> FromText<'g> for Type<'g> {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        match state.peek_token()?.kind {
            TokenKind::Keyword(Keyword::I8)
            | TokenKind::Keyword(Keyword::I16)
            | TokenKind::Keyword(Keyword::I32)
            | TokenKind::Keyword(Keyword::I64) => Ok(Type::Integer(FromText::from_text(state)?)),
            TokenKind::Keyword(Keyword::F16)
            | TokenKind::Keyword(Keyword::F32)
            | TokenKind::Keyword(Keyword::F64) => Ok(Type::Float(FromText::from_text(state)?)),
            TokenKind::Keyword(Keyword::Bool) => Ok(Type::Bool(FromText::from_text(state)?)),
            TokenKind::Punct(Punctuation::LParen) => state.parse_parenthesized(
                Punctuation::LParen,
                "",
                Punctuation::RParen,
                "missing closing parenthesis: ')'",
                FromText::from_text,
            ),
            TokenKind::Punct(Punctuation::LessThan) => {
                Ok(Type::Vector(FromText::from_text(state)?))
            }
            TokenKind::Punct(Punctuation::Asterisk) => {
                Ok(Type::Pointer(FromText::from_text(state)?))
            }
            _ => state.error_at_peek_token("expected type")?.into(),
        }
    }
}

impl<'g> FromText<'g> for Interned<'g, Type<'g>> {
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        Ok(state.global_state().intern(&Type::from_text(state)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_from_text() {
        let global_state = GlobalState::new();
        macro_rules! test_type {
            ($global_state:ident, $text:literal, $type:expr) => {
                let parsed_type: Interned<Type> =
                    FromText::parse("", $text, &$global_state).unwrap();
                let expected_type = $global_state.intern(&$type);
                assert_eq!(parsed_type, expected_type);
            };
        }
        test_type!(global_state, "i8", Type::Integer(IntegerType::Int8));
        test_type!(global_state, "i16", Type::Integer(IntegerType::Int16));
        test_type!(global_state, "i32", Type::Integer(IntegerType::Int32));
        test_type!(global_state, "i64", Type::Integer(IntegerType::Int64));
        test_type!(global_state, "f16", Type::Float(FloatType::Float16));
        test_type!(global_state, "f32", Type::Float(FloatType::Float32));
        test_type!(global_state, "f64", Type::Float(FloatType::Float64));
        test_type!(global_state, "bool", Type::Bool(BoolType));
        test_type!(
            global_state,
            "* i8",
            Type::Pointer(PointerType {
                pointee: global_state.intern(&Type::Integer(IntegerType::Int8))
            })
        );
        test_type!(
            global_state,
            "<4 x f16>",
            Type::Vector(VectorType {
                len: 4,
                scalable: false,
                element: global_state.intern(&Type::Float(FloatType::Float16))
            })
        );
        test_type!(
            global_state,
            "<vscale x 7 x f32>",
            Type::Vector(VectorType {
                len: 7,
                scalable: true,
                element: global_state.intern(&Type::Float(FloatType::Float32))
            })
        );
        test_type!(
            global_state,
            "(<vscale x 7 x ((* bool))>)",
            Type::Vector(VectorType {
                len: 7,
                scalable: true,
                element: global_state.intern(&Type::Pointer(PointerType {
                    pointee: global_state.intern(&Type::Bool(BoolType))
                }))
            })
        );
        // FIXME: add tests for opaque types
    }
}
