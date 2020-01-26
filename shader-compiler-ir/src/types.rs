// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextState;
use crate::text::IntegerToken;
use crate::text::Keyword;
use crate::text::Punctuation;
use crate::text::ToTextState;
use crate::text::TokenKind;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::fmt;
use core::ops::Deref;
use core::ops::DerefMut;

/// extension trait for types
pub trait GenericType<'g>: Internable<'g, Interned = Type<'g>> {
    /// create an `undef` constant
    fn undef(&self, global_state: &'g GlobalState<'g>) -> Const<'g> {
        self.intern(global_state).undef()
    }
    /// create a new `ValueDefinition` using `self` as the new value's type
    fn new_value_definition<Name: Internable<'g, Interned = str>>(
        &self,
        name: Name,
        global_state: &'g GlobalState<'g>,
    ) -> ValueDefinition<'g> {
        ValueDefinition::new(self, name, global_state)
    }
}

/// an integer type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum IntegerType {
    /// 8-bit signed or unsigned integer type
    Int8,
    /// 16-bit signed or unsigned integer type
    Int16,
    /// 32-bit signed or unsigned integer type
    Int32,
    /// 32-bit signed or unsigned integer type with reduced range, the range is at least as big as `Int16`
    RelaxedInt32,
    /// 64-bit signed or unsigned integer type
    Int64,
}

impl<'g> Internable<'g> for IntegerType {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(self.clone()).intern(global_state)
    }
}

impl<'g> GenericType<'g> for IntegerType {}

impl From<IntegerType> for Type<'_> {
    fn from(v: IntegerType) -> Self {
        Type::Integer(v)
    }
}

/// a float type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum FloatType {
    /// 16-bit float type
    Float16,
    /// 32-bit float type
    Float32,
    /// 32-bit float type with reduced range and precision, the range and precision is at least as big as `Float16`
    RelaxedFloat32,
    /// 64-bit float type
    Float64,
}

impl<'g> Internable<'g> for FloatType {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for FloatType {}

impl From<FloatType> for Type<'_> {
    fn from(v: FloatType) -> Self {
        Type::Float(v)
    }
}

mod private {
    #[doc(hidden)]
    #[derive(Clone, Eq, PartialEq, Hash, Debug)]
    pub enum Void {}
}

/// an opaque type.
///
/// currently there aren't any defined opaque types.
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum OpaqueType<'g> {
    // TODO: implement
    #[doc(hidden)]
    _Unimplemented(&'g (), private::Void),
}

impl<'g> Internable<'g> for OpaqueType<'g> {
    type Interned = Type<'g>;
    fn intern(&self, _global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        match self {
            OpaqueType::_Unimplemented(_, v) => match *v {},
        }
    }
}

impl<'g> GenericType<'g> for OpaqueType<'g> {}

/// the `bool` type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct BoolType;

impl<'g> Internable<'g> for BoolType {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for BoolType {}

impl From<BoolType> for Type<'_> {
    fn from(v: BoolType) -> Self {
        Type::Bool(v)
    }
}

/// a pointer type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct DataPointerType;

impl<'g> Internable<'g> for DataPointerType {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(self.clone()).intern(global_state)
    }
}

impl<'g> GenericType<'g> for DataPointerType {}

impl DataPointerType {
    /// create a null pointer constant
    pub fn null<'g>(self) -> Const<'g> {
        Const::Null(self.into())
    }
}

impl<'g> From<DataPointerType> for PointerType<'g> {
    fn from(v: DataPointerType) -> Self {
        PointerType::Data(v)
    }
}

impl<'g> From<DataPointerType> for Type<'g> {
    fn from(v: DataPointerType) -> Self {
        Type::Pointer(v.into())
    }
}

/// either a function pointer or a data pointer type
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum PointerType<'g> {
    /// a data pointer type
    Data(DataPointerType),
    /// a function pointer type
    Function(FunctionPointerType<'g>),
}

impl<'g> PointerType<'g> {
    /// create a null pointer constant
    pub fn null(self) -> Const<'g> {
        Const::Null(self)
    }
}

impl<'g> From<PointerType<'g>> for Type<'g> {
    fn from(v: PointerType<'g>) -> Self {
        Type::Pointer(v)
    }
}

impl<'g> Internable<'g> for PointerType<'g> {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(self.clone()).intern(global_state)
    }
}

impl<'g> GenericType<'g> for PointerType<'g> {}

/// a vector type.
///
/// There are two variants of a vector:
/// * a non-scalable vector where the number of elements is just `self.len`
/// * a scalable vector where the number of elements is a constant multiple (called
///   `vscale`) of `self.len`, where `vscale` may not be known till runtime.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct VectorType<'g> {
    /// the number of elements for non-scalable vectors and the multiplier of the number of elements for scalable vectors
    pub len: usize,
    /// determines if the vector type is scalable or not.
    pub scalable: bool,
    /// the type of an element
    pub element: Interned<'g, Type<'g>>,
}

impl<'g> Internable<'g> for VectorType<'g> {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(self.clone()).intern(global_state)
    }
}

impl<'g> GenericType<'g> for VectorType<'g> {}

impl<'g> From<VectorType<'g>> for Type<'g> {
    fn from(v: VectorType<'g>) -> Self {
        Type::Vector(v)
    }
}

impl<'g> From<OpaqueType<'g>> for Type<'g> {
    fn from(v: OpaqueType<'g>) -> Self {
        Type::Opaque(v)
    }
}

/// a function pointer type
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct FunctionPointerType<'g> {
    /// the function argument types
    pub arguments: Vec<Interned<'g, Type<'g>>>,
    /// the function return types
    pub returns: Inhabitable<Vec<Interned<'g, Type<'g>>>>,
}

impl<'g> From<FunctionPointerType<'g>> for PointerType<'g> {
    fn from(v: FunctionPointerType<'g>) -> Self {
        PointerType::Function(v)
    }
}

impl<'g> From<FunctionPointerType<'g>> for Type<'g> {
    fn from(v: FunctionPointerType<'g>) -> Self {
        Type::Pointer(v.into())
    }
}

impl<'g> Internable<'g> for FunctionPointerType<'g> {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(self.clone()).intern(global_state)
    }
}

impl<'g> GenericType<'g> for FunctionPointerType<'g> {}

impl<'g> FunctionPointerType<'g> {
    /// create a null pointer constant
    pub fn null(self) -> Const<'g> {
        Const::Null(self.into())
    }
}

/// an IR type
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Type<'g> {
    /// an integer type
    Integer(IntegerType),
    /// a float type
    Float(FloatType),
    /// the `bool` type
    Bool(BoolType),
    /// a pointer type
    Pointer(PointerType<'g>),
    /// a vector type
    Vector(VectorType<'g>),
    /// an opaque type
    Opaque(OpaqueType<'g>),
}

impl<'g> GenericType<'g> for Type<'g> {}

impl<'g> GenericType<'g> for Interned<'g, Type<'g>> {}

/// if a type or value `T` is inhabited (is reachable)
#[derive(Clone, Eq, PartialEq, Hash)]
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

impl<'g, T: FromText<'g>> FromText<'g> for Inhabitable<T> {
    type Parsed = Inhabitable<T::Parsed>;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self::Parsed, FromTextError> {
        match state.peek_token()?.kind {
            TokenKind::Punct(Punctuation::ExMark) => {
                state.parse_token()?;
                Ok(Uninhabited)
            }
            _ => Ok(Inhabited(T::from_text(state)?)),
        }
    }
}

impl<'g, T: ToText<'g>> ToText<'g> for Inhabitable<T> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            Uninhabited => write!(state, "!"),
            Inhabited(v) => v.to_text(state),
        }
    }
}

macro_rules! impl_from_to_text_for_keyword_type {
    ($type:ident {
        $($kw:ident => $value:path,)+
        _ => $error_msg:expr,
    }) => {
        impl<'g> FromText<'g> for $type {
            type Parsed = Self;
            fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
                let retval = match state.peek_token()?.kind {
                    $(TokenKind::Keyword(Keyword::$kw) => $value,)+
                    _ => state.error_at_peek_token($error_msg)?.into(),
                };
                state.parse_token()?;
                Ok(retval)
            }
        }

        impl_display_as_to_text!(<'g> $type);

        impl<'g> ToText<'g> for $type {
            fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                match self {
                    $(
                        $value => write!(state, "{}", Keyword::$kw),
                    )+
                }
            }
        }
    };
}

impl_from_to_text_for_keyword_type! {
    IntegerType {
        I8 => IntegerType::Int8,
        I16 => IntegerType::Int16,
        I32 => IntegerType::Int32,
        RI32 => IntegerType::RelaxedInt32,
        I64 => IntegerType::Int64,
        _ => "invalid integer type",
    }
}

impl_from_to_text_for_keyword_type! {
    FloatType {
        F16 => FloatType::Float16,
        F32 => FloatType::Float32,
        RF32 => FloatType::RelaxedFloat32,
        F64 => FloatType::Float64,
        _ => "invalid float type",
    }
}

impl_from_to_text_for_keyword_type! {
    BoolType {
        Bool => BoolType,
        _ => "invalid bool type",
    }
}

impl<'g> FromText<'g> for VectorType<'g> {
    type Parsed = Self;
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
                    element: Type::from_text(state)?,
                })
            },
        )
    }
}

impl_display_as_to_text!(<'g> VectorType<'g>);

impl<'g> ToText<'g> for VectorType<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let VectorType {
            len,
            scalable,
            element,
        } = *self;
        write!(state, "<")?;
        if scalable {
            write!(state, "vscale x ")?;
        }
        write!(state, "{} x ", len)?;
        element.to_text(state)?;
        write!(state, ">")
    }
}

impl<'g> FromText<'g> for DataPointerType {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_keyword_token_or_error(Keyword::DataPtr, "expected data_ptr type")?;
        Ok(DataPointerType)
    }
}

impl<'g> FromText<'g> for PointerType<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let start_location = state.location;
        let keyword = state.peek_token()?.kind.keyword();
        state.location = start_location;
        if keyword == Some(Keyword::Fn) {
            Ok(FunctionPointerType::from_text(state)?.into())
        } else {
            Ok(DataPointerType::from_text(state)?.into())
        }
    }
}

impl_display_as_to_text!(DataPointerType);

impl<'g> ToText<'g> for DataPointerType {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "data_ptr")
    }
}

impl_display_as_to_text!(<'g> PointerType<'g>);

impl<'g> ToText<'g> for PointerType<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            PointerType::Data(v) => v.to_text(state),
            PointerType::Function(v) => v.to_text(state),
        }
    }
}

impl<'g> FromText<'g> for OpaqueType<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        // TODO: implement
        state
            .error_at_peek_token("OpaqueType can't be parsed")?
            .into()
    }
}

impl_display_as_to_text!(<'g> OpaqueType<'g>);

impl<'g> ToText<'g> for OpaqueType<'g> {
    fn to_text(&self, _state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            OpaqueType::_Unimplemented(_, v) => match *v {},
        }
    }
}

impl<'g> FromText<'g> for FunctionPointerType<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_keyword_token_or_error(Keyword::Fn, "expected function pointer type")?;
        let arguments = Vec::<Type>::from_text(state)?;
        state.parse_punct_token_or_error(
            Punctuation::Arrow,
            "function pointer type is missing arrow before return types -- expected `->`",
        )?;
        let returns = Inhabitable::<Vec<Type>>::from_text(state)?;
        Ok(FunctionPointerType { arguments, returns })
    }
}

impl_display_as_to_text!(<'g> FunctionPointerType<'g>);

impl<'g> ToText<'g> for FunctionPointerType<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let Self { arguments, returns } = self;
        write!(state, "fn")?;
        arguments.to_text(state)?;
        write!(state, " -> ")?;
        returns.to_text(state)
    }
}

impl<'g> FromText<'g> for Type<'g> {
    type Parsed = Interned<'g, Type<'g>>;
    fn from_text(
        state: &mut FromTextState<'g, '_>,
    ) -> Result<Interned<'g, Type<'g>>, FromTextError> {
        let retval = match state.peek_token()?.kind {
            TokenKind::Keyword(Keyword::I8)
            | TokenKind::Keyword(Keyword::I16)
            | TokenKind::Keyword(Keyword::I32)
            | TokenKind::Keyword(Keyword::RI32)
            | TokenKind::Keyword(Keyword::I64) => {
                IntegerType::from_text(state)?.intern(state.global_state())
            }
            TokenKind::Keyword(Keyword::F16)
            | TokenKind::Keyword(Keyword::F32)
            | TokenKind::Keyword(Keyword::RF32)
            | TokenKind::Keyword(Keyword::F64) => {
                FloatType::from_text(state)?.intern(state.global_state())
            }
            TokenKind::Keyword(Keyword::Bool) => {
                BoolType::from_text(state)?.intern(state.global_state())
            }
            TokenKind::Punct(Punctuation::LParen) => {
                return state.parse_parenthesized(
                    Punctuation::LParen,
                    "",
                    Punctuation::RParen,
                    "missing closing parenthesis: ')'",
                    Type::from_text,
                );
            }
            TokenKind::Punct(Punctuation::LessThan) => {
                VectorType::from_text(state)?.intern(state.global_state())
            }
            TokenKind::Keyword(Keyword::Fn) | TokenKind::Keyword(Keyword::DataPtr) => {
                PointerType::from_text(state)?.intern(state.global_state())
            }
            _ => state.error_at_peek_token("expected type")?.into(),
        };
        Ok(retval)
    }
}

impl_display_as_to_text!(<'g> Type<'g>);

impl<'g> ToText<'g> for Type<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            Type::Integer(v) => v.to_text(state),
            Type::Float(v) => v.to_text(state),
            Type::Bool(v) => v.to_text(state),
            Type::Pointer(v) => v.to_text(state),
            Type::Vector(v) => v.to_text(state),
            Type::Opaque(v) => v.to_text(state),
        }
    }
}

impl<'g> Interned<'g, Type<'g>> {
    /// create an `undef` constant
    pub fn undef(self) -> Const<'g> {
        Const::Undef(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_type_from_to_text() {
        let global_state = GlobalState::new();
        macro_rules! test_type {
            ($global_state:ident, $text:literal, $type:expr, $formatted_text:literal) => {
                let parsed_type = Type::parse("", $text, &$global_state).unwrap();
                let expected_type = $type.intern(&$global_state);
                assert_eq!(parsed_type, expected_type);
                let text = expected_type.display().to_string();
                assert_eq!($formatted_text, text);
            };
            ($global_state:ident, $text:literal, $type:expr) => {
                test_type!($global_state, $text, $type, $text);
            };
        }
        test_type!(global_state, "i8", IntegerType::Int8);
        test_type!(global_state, "i16", IntegerType::Int16);
        test_type!(global_state, "i32", IntegerType::Int32);
        test_type!(global_state, "ri32", IntegerType::RelaxedInt32);
        test_type!(global_state, "i64", IntegerType::Int64);
        test_type!(global_state, "f16", FloatType::Float16);
        test_type!(global_state, "f32", FloatType::Float32);
        test_type!(global_state, "rf32", FloatType::RelaxedFloat32);
        test_type!(global_state, "f64", FloatType::Float64);
        test_type!(global_state, "bool", BoolType);
        test_type!(global_state, "data_ptr", DataPointerType);
        test_type!(
            global_state,
            "<4 x f16>",
            VectorType {
                len: 4,
                scalable: false,
                element: FloatType::Float16.intern(&global_state)
            }
        );
        test_type!(
            global_state,
            "<vscale x 7 x f32>",
            VectorType {
                len: 7,
                scalable: true,
                element: FloatType::Float32.intern(&global_state)
            }
        );
        test_type!(
            global_state,
            "(<vscale x 7 x ((data_ptr))>)",
            VectorType {
                len: 7,
                scalable: true,
                element: DataPointerType.intern(&global_state)
            },
            "<vscale x 7 x data_ptr>"
        );
        test_type!(
            global_state,
            "fn[] -> []",
            FunctionPointerType {
                arguments: vec![],
                returns: Inhabited(vec![]),
            }
        );
        test_type!(
            global_state,
            "fn[] -> !",
            FunctionPointerType {
                arguments: vec![],
                returns: Uninhabited,
            }
        );
        test_type!(
            global_state,
            "fn[i8] -> !",
            FunctionPointerType {
                arguments: vec![IntegerType::Int8.intern(&global_state)],
                returns: Uninhabited,
            }
        );
        test_type!(
            global_state,
            "fn[i8, i16] -> !",
            FunctionPointerType {
                arguments: vec![
                    IntegerType::Int8.intern(&global_state),
                    IntegerType::Int16.intern(&global_state),
                ],
                returns: Uninhabited,
            }
        );
        test_type!(
            global_state,
            "fn[i8, i16] -> [data_ptr]",
            FunctionPointerType {
                arguments: vec![
                    IntegerType::Int8.intern(&global_state),
                    IntegerType::Int16.intern(&global_state),
                ],
                returns: Inhabited(vec![DataPointerType.intern(&global_state)]),
            }
        );
        // FIXME: add tests for opaque types
    }
}
