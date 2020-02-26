// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    prelude::*,
    text::{
        FromTextError, FromTextState, FromToTextListForm, IntegerToken, Keyword, ListForm,
        NewOrOld, Punctuation, ToTextState, TokenKind,
    },
    TargetProperties,
};
use alloc::vec::Vec;
use core::{
    convert::TryInto,
    fmt,
    num::NonZeroU32,
    ops::{Deref, DerefMut},
};

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
    /// get the required alignment for `self`
    fn alignment(&self, target_properties: &TargetProperties) -> Alignment;
    /// get the required size for `self`
    fn size(&self, target_properties: &TargetProperties) -> StructSize;
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
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for IntegerType {
    fn alignment(&self, _target_properties: &TargetProperties) -> Alignment {
        match self {
            IntegerType::Int8 => Alignment::new(1).unwrap(),
            IntegerType::Int16 => Alignment::new(2).unwrap(),
            IntegerType::Int32 | IntegerType::RelaxedInt32 => Alignment::new(4).unwrap(),
            IntegerType::Int64 => Alignment::new(8).unwrap(),
        }
    }
    fn size(&self, _target_properties: &TargetProperties) -> StructSize {
        StructSize::Fixed {
            size: match self {
                IntegerType::Int8 => 1,
                IntegerType::Int16 => 2,
                IntegerType::Int32 | IntegerType::RelaxedInt32 => 4,
                IntegerType::Int64 => 8,
            },
        }
    }
}

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

impl<'g> GenericType<'g> for FloatType {
    fn alignment(&self, _target_properties: &TargetProperties) -> Alignment {
        match self {
            FloatType::Float16 => Alignment::new(2).unwrap(),
            FloatType::Float32 | FloatType::RelaxedFloat32 => Alignment::new(4).unwrap(),
            FloatType::Float64 => Alignment::new(8).unwrap(),
        }
    }
    fn size(&self, _target_properties: &TargetProperties) -> StructSize {
        StructSize::Fixed {
            size: match self {
                FloatType::Float16 => 2,
                FloatType::Float32 | FloatType::RelaxedFloat32 => 4,
                FloatType::Float64 => 8,
            },
        }
    }
}

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

impl<'g> GenericType<'g> for OpaqueType<'g> {
    fn alignment(&self, _target_properties: &TargetProperties) -> Alignment {
        match self {
            OpaqueType::_Unimplemented(_, v) => match *v {},
        }
    }
    fn size(&self, _target_properties: &TargetProperties) -> StructSize {
        match self {
            OpaqueType::_Unimplemented(_, v) => match *v {},
        }
    }
}

/// the `bool` type
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct BoolType;

impl<'g> Internable<'g> for BoolType {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for BoolType {
    fn alignment(&self, _target_properties: &TargetProperties) -> Alignment {
        Alignment::new(1).unwrap()
    }
    fn size(&self, _target_properties: &TargetProperties) -> StructSize {
        StructSize::Fixed { size: 1 }
    }
}

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
        Type::from(*self).intern(global_state)
    }
}

impl<'g> GenericType<'g> for DataPointerType {
    fn alignment(&self, target_properties: &TargetProperties) -> Alignment {
        target_properties
            .data_pointer_underlying_type
            .alignment(target_properties)
    }
    fn size(&self, target_properties: &TargetProperties) -> StructSize {
        target_properties
            .data_pointer_underlying_type
            .size(target_properties)
    }
}

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

impl<'g> GenericType<'g> for PointerType<'g> {
    fn alignment(&self, target_properties: &TargetProperties) -> Alignment {
        match self {
            PointerType::Data(v) => v.alignment(target_properties),
            PointerType::Function(v) => v.alignment(target_properties),
        }
    }
    fn size(&self, target_properties: &TargetProperties) -> StructSize {
        match self {
            PointerType::Data(v) => v.size(target_properties),
            PointerType::Function(v) => v.size(target_properties),
        }
    }
}

/// a vector type.
///
/// There are two variants of a vector:
/// * a non-scalable vector where the number of elements is just `self.len`
/// * a scalable vector where the number of elements is a constant multiple (called
///   `vscale`) of `self.len`, where `vscale` may not be known till runtime.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct VectorType<'g> {
    /// the number of elements for non-scalable vectors and the multiplier of the number of elements for scalable vectors
    pub len: u32,
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

impl<'g> GenericType<'g> for VectorType<'g> {
    fn alignment(&self, target_properties: &TargetProperties) -> Alignment {
        self.element.alignment(target_properties)
    }
    fn size(&self, target_properties: &TargetProperties) -> StructSize {
        if self.scalable {
            StructSize::Variable { fixed_part_size: 0 }
        } else {
            match self.element.size(target_properties) {
                StructSize::Variable { .. } => {
                    unreachable!("vector element size must not be variable-sized")
                }
                StructSize::Fixed { size } => StructSize::Fixed {
                    size: size
                        .checked_mul(self.len)
                        .expect("overflow calculating VectorType size"),
                },
            }
        }
    }
}

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

impl<'g> GenericType<'g> for FunctionPointerType<'g> {
    fn alignment(&self, target_properties: &TargetProperties) -> Alignment {
        target_properties
            .function_pointer_underlying_type
            .alignment(target_properties)
    }
    fn size(&self, target_properties: &TargetProperties) -> StructSize {
        target_properties
            .function_pointer_underlying_type
            .size(target_properties)
    }
}

impl<'g> FunctionPointerType<'g> {
    /// create a null pointer constant
    pub fn null(self) -> Const<'g> {
        Const::Null(self.into())
    }
}

/// a struct/union member
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct StructMember<'g> {
    /// the number of bytes from the start of the struct that this member starts at
    pub offset: u32,
    /// the type of this member
    pub member_type: Interned<'g, Type<'g>>,
}

impl_display_as_to_text!(<'g> StructMember<'g>);

/// a struct/union's size in bytes
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum StructSize {
    /// a variable-sized struct/union
    Variable {
        /// the size in bytes of the fixed-sized part of the struct/union
        fixed_part_size: u32,
    },
    /// a fixed-sized struct/union
    Fixed {
        /// the size in bytes of the struct/union
        size: u32,
    },
}

impl_display_as_to_text!(StructSize);

/// a type's alignment in bytes -- must be a power of 2
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Alignment(NonZeroU32);

impl Default for Alignment {
    fn default() -> Alignment {
        Alignment(NonZeroU32::new(1).expect("known to be non-zero"))
    }
}

impl Alignment {
    /// create a new `Alignment` if the passed-in integer is a power-of-two, otherwise return `None`.
    pub fn new(alignment: u32) -> Option<Self> {
        if alignment.is_power_of_two() {
            // safety: alignment was just checked to be a power-of-two
            unsafe { Some(Self::new_unchecked(alignment)) }
        } else {
            None
        }
    }
    /// create a new `Alignment` without checking that it is a power-of-two.
    ///
    /// # Safety
    ///
    /// `alignment` must be a power-of-two.
    pub const unsafe fn new_unchecked(alignment: u32) -> Self {
        // safety: all powers of two are non-zero
        Alignment(NonZeroU32::new_unchecked(alignment))
    }
    /// gets the alignment in bytes.
    pub const fn get(self) -> u32 {
        self.0.get()
    }
    /// Calculates the first byte position/count greater than or equal to `byte_count` that is properly aligned to alignment `self`. Wraps on overflow.
    ///
    /// Example:
    /// ```
    /// # use shader_compiler_ir::Alignment;
    /// let align = Alignment::new(4).unwrap();
    /// assert_eq!(align.align_up_wrapping(22), 24);
    /// assert_eq!(align.align_up_wrapping(20), 20);
    /// assert_eq!(align.align_up_wrapping(0xFFFF_FFFF), 0);
    /// ```
    pub const fn align_up_wrapping(self, byte_count: u32) -> u32 {
        byte_count.wrapping_add(self.get()).wrapping_sub(1) & !self.get().wrapping_sub(1)
    }
    /// Calculates the first byte position/count greater than or equal to `byte_count` that is properly aligned to alignment `self`. Returns `None` on overflow.
    ///
    /// Example:
    /// ```
    /// # use shader_compiler_ir::Alignment;
    /// let align = Alignment::new(4).unwrap();
    /// assert_eq!(align.align_up_checked(22), Some(24));
    /// assert_eq!(align.align_up_checked(20), Some(20));
    /// assert_eq!(align.align_up_checked(0xFFFF_FFFF), None);
    /// ```
    pub fn align_up_checked(self, byte_count: u32) -> Option<u32> {
        let retval = self.align_up_wrapping(byte_count);
        if retval >= byte_count {
            Some(retval)
        } else {
            None
        }
    }
}

impl_display_as_to_text!(Alignment);

/// a struct/union
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct StructType<'g> {
    /// the size of this struct/union
    pub size: StructSize,
    /// the alignment of this struct/union
    pub alignment: Alignment,
    /// the members of this struct, not necessarily sorted by offset
    pub members: Vec<StructMember<'g>>,
}

impl<'g> From<StructType<'g>> for Type<'g> {
    fn from(v: StructType<'g>) -> Self {
        Type::Struct(v)
    }
}

impl<'g> Internable<'g> for StructType<'g> {
    type Interned = Type<'g>;
    fn intern(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type<'g>> {
        Type::from(self.clone()).intern(global_state)
    }
}

impl<'g> GenericType<'g> for StructType<'g> {
    fn alignment(&self, _target_properties: &TargetProperties) -> Alignment {
        self.alignment
    }
    fn size(&self, _target_properties: &TargetProperties) -> StructSize {
        self.size
    }
}

impl_display_as_to_text!(<'g> StructType<'g>);

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
    /// a struct/union type
    Struct(StructType<'g>),
}

impl<'g> GenericType<'g> for Type<'g> {
    fn alignment(&self, target_properties: &TargetProperties) -> Alignment {
        match self {
            Type::Integer(v) => v.alignment(target_properties),
            Type::Float(v) => v.alignment(target_properties),
            Type::Bool(v) => v.alignment(target_properties),
            Type::Pointer(v) => v.alignment(target_properties),
            Type::Vector(v) => v.alignment(target_properties),
            Type::Opaque(v) => v.alignment(target_properties),
            Type::Struct(v) => v.alignment(target_properties),
        }
    }
    fn size(&self, target_properties: &TargetProperties) -> StructSize {
        match self {
            Type::Integer(v) => v.size(target_properties),
            Type::Float(v) => v.size(target_properties),
            Type::Bool(v) => v.size(target_properties),
            Type::Pointer(v) => v.size(target_properties),
            Type::Vector(v) => v.size(target_properties),
            Type::Opaque(v) => v.size(target_properties),
            Type::Struct(v) => v.size(target_properties),
        }
    }
}

impl<'g> GenericType<'g> for Interned<'g, Type<'g>> {
    fn alignment(&self, target_properties: &TargetProperties) -> Alignment {
        self.get().alignment(target_properties)
    }
    fn size(&self, target_properties: &TargetProperties) -> StructSize {
        self.get().size(target_properties)
    }
}

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

impl<T: FromToTextListForm> FromToTextListForm for Inhabitable<T> {
    fn from_to_text_list_form() -> ListForm {
        T::from_to_text_list_form()
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
        impl FromToTextListForm for $type {}

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

impl FromToTextListForm for VectorType<'_> {}

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
                let len: u32 = match len.kind {
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

impl FromToTextListForm for DataPointerType {}

impl<'g> FromText<'g> for DataPointerType {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_keyword_token_or_error(Keyword::DataPtr, "expected data_ptr type")?;
        Ok(DataPointerType)
    }
}

impl FromToTextListForm for PointerType<'_> {}

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

impl FromToTextListForm for OpaqueType<'_> {}

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

impl FromToTextListForm for FunctionPointerType<'_> {}

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

impl FromToTextListForm for StructSize {}

impl<'g> FromText<'g> for StructSize {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        fn parse_value<F: FnOnce(u32) -> StructSize>(
            state: &mut FromTextState<'_, '_>,
            f: F,
        ) -> Result<StructSize, FromTextError> {
            state.parse_token()?;
            match state.peek_token()?.kind.integer() {
                Some(IntegerToken {
                    value: _value,
                    suffix: Some(_),
                }) => state
                    .error_at_peek_token("suffix not allowed on struct size")?
                    .into(),
                Some(IntegerToken {
                    value,
                    suffix: None,
                }) => {
                    if let Ok(value) = value.try_into() {
                        state.parse_token()?;
                        Ok(f(value))
                    } else {
                        state.error_at_peek_token("struct size overflow")?.into()
                    }
                }
                None => state
                    .error_at_peek_token("missing struct size: must be an integer")?
                    .into(),
            }
        }
        match state.peek_token()?.kind.keyword() {
            Some(Keyword::Fixed) => parse_value(state, |v| StructSize::Fixed { size: v }),
            Some(Keyword::Variable) => {
                parse_value(state, |v| StructSize::Variable { fixed_part_size: v })
            }
            _ => state
                .error_at_peek_token("missing struct size kind: must be `fixed` or `variable`")?
                .into(),
        }
    }
}

impl<'g> ToText<'g> for StructSize {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match *self {
            StructSize::Fixed { size } => write!(state, "fixed {:#X}", size),
            StructSize::Variable { fixed_part_size } => {
                write!(state, "variable {:#X}", fixed_part_size)
            }
        }
    }
}

impl FromToTextListForm for Alignment {}

impl<'g> FromText<'g> for Alignment {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_keyword_token_or_error(
            Keyword::Align,
            "missing alignment: must be of the form `align: 16`",
        )?;
        state.parse_punct_token_or_error(
            Punctuation::Colon,
            "missing colon after `align` keyword: must be of the form `align: 16`",
        )?;
        match state.peek_token()?.kind.integer() {
            Some(IntegerToken {
                value: _value,
                suffix: Some(_),
            }) => state
                .error_at_peek_token("suffix not allowed on alignment value")?
                .into(),
            Some(IntegerToken {
                value,
                suffix: None,
            }) => {
                if let Ok(value) = value.try_into() {
                    if let Some(retval) = Alignment::new(value) {
                        state.parse_token()?;
                        Ok(retval)
                    } else {
                        state
                            .error_at_peek_token("alignment must be an integer power-of-two")?
                            .into()
                    }
                } else {
                    state
                        .error_at_peek_token("alignment value overflow")?
                        .into()
                }
            }
            None => state
                .error_at_peek_token("missing alignment value: must be an integer power-of-two")?
                .into(),
        }
    }
}

impl<'g> ToText<'g> for Alignment {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "align: {:#X}", self.get())
    }
}

impl<'g> FromText<'g> for StructMember<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        match state.peek_token()?.kind.integer() {
            Some(IntegerToken {
                value: _value,
                suffix: Some(_),
            }) => state
                .error_at_peek_token("suffix not allowed on struct member offset")?
                .into(),
            Some(IntegerToken {
                value,
                suffix: None,
            }) => {
                if let Ok(offset) = value.try_into() {
                    state.parse_token()?;
                    state.parse_punct_token_or_error(
                        Punctuation::Colon,
                        "missing colon between struct member's offset and type",
                    )?;
                    let member_type = Type::from_text(state)?;
                    Ok(StructMember {
                        offset,
                        member_type,
                    })
                } else {
                    state
                        .error_at_peek_token("struct member offset overflow")?
                        .into()
                }
            }
            None => state
                .error_at_peek_token("missing struct member offset: must be an integer")?
                .into(),
        }
    }
}

impl FromToTextListForm for StructMember<'_> {
    fn from_to_text_list_form() -> ListForm {
        ListForm::STATEMENTS
    }
}

impl<'g> ToText<'g> for StructMember<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let StructMember {
            offset,
            member_type,
        } = *self;
        write!(state, "{:#X}: ", offset)?;
        member_type.to_text(state)
    }
}

impl FromToTextListForm for StructType<'_> {}

impl<'g> FromText<'g> for StructType<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        use hashbrown::hash_map::Entry::*;
        state.parse_keyword_token_or_error(Keyword::Struct, "missing struct keyword")?;
        let id = match state.peek_token()?.kind.integer() {
            Some(IntegerToken {
                value: _value,
                suffix: Some(_),
            }) => state
                .error_at_peek_token("suffix not allowed on struct id")?
                .into(),
            Some(IntegerToken {
                value,
                suffix: None,
            }) => value,
            None => state
                .error_at_peek_token("missing alignment value: must be an integer power-of-two")?
                .into(),
        };
        let id_location = state.parse_token()?.span;
        state.parse_parenthesized(
            Punctuation::LCurlyBrace,
            "struct body is missing opening curly brace ('{')",
            Punctuation::RCurlyBrace,
            "struct body is missing closing curly brace ('}')",
            |state| {
                if state.peek_token()?.kind.punct() == Some(Punctuation::RCurlyBrace) {
                    if let Some(retval) = state.structs.get(&id) {
                        Ok(retval.clone())
                    } else {
                        state.error_at(id_location, "struct id not defined")?.into()
                    }
                } else {
                    state.parse_keyword_token_or_error(
                        Keyword::Size,
                        "missing struct size keyword (`size`)",
                    )?;
                    state.parse_punct_token_or_error(
                        Punctuation::Colon,
                        "missing colon between struct size keyword and struct size",
                    )?;
                    let size = StructSize::from_text(state)?;
                    state.parse_punct_token_or_error(
                        Punctuation::Comma,
                        "missing comma after struct size",
                    )?;
                    let alignment = Alignment::from_text(state)?;
                    state.parse_punct_token_or_error(
                        Punctuation::Comma,
                        "missing comma after struct alignment",
                    )?;
                    let mut members = Vec::new();
                    while Some(Punctuation::RCurlyBrace) != state.peek_token()?.kind.punct() {
                        members.push(StructMember::from_text(state)?);
                        state.parse_punct_token_or_error(
                            Punctuation::Comma,
                            "missing comma after struct member",
                        )?;
                    }
                    let retval = StructType {
                        size,
                        alignment,
                        members,
                    };
                    match state.structs.entry(id) {
                        Occupied(_) => state
                            .error_at(id_location, "struct id already defined")?
                            .into(),
                        Vacant(entry) => {
                            entry.insert(retval.clone());
                            Ok(retval)
                        }
                    }
                }
            },
        )
    }
}

impl<'g> ToText<'g> for StructType<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match state.get_struct_type_id(self) {
            NewOrOld::Old(id) => write!(state, "struct {} {{}}", id),
            NewOrOld::New(id) => {
                writeln!(state, "struct {} {{", id)?;
                state.indent(|state| -> fmt::Result {
                    let StructType {
                        size,
                        alignment,
                        ref members,
                    } = *self;
                    write!(state, "size: ")?;
                    size.to_text(state)?;
                    writeln!(state, ",")?;
                    alignment.to_text(state)?;
                    writeln!(state, ",")?;
                    for member in members {
                        member.to_text(state)?;
                        writeln!(state, ",")?;
                    }
                    Ok(())
                })?;
                write!(state, "}}")
            }
        }
    }
}

impl FromToTextListForm for Type<'_> {}

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
            TokenKind::Keyword(Keyword::Struct) => {
                StructType::from_text(state)?.intern(state.global_state())
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
            Type::Struct(v) => v.to_text(state),
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

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn test_type_from_to_text() {
        let global_state = GlobalState::new();
        macro_rules! test_type {
            ($global_state:ident, $text:expr, $type:expr, $formatted_text:expr) => {
                let parsed_type = Type::parse("", $text, &$global_state).unwrap();
                let expected_type = $type.intern(&$global_state);
                assert_eq!(parsed_type, expected_type);
                let text = expected_type.display().to_string();
                assert_eq!($formatted_text, text);
            };
            ($global_state:ident, $text:expr, $type:expr) => {
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
        test_type!(
            global_state,
            concat!(
                "struct 0 {\n",
                "    size: fixed 0x10,\n",
                "    align: 0x4,\n",
                "    0x0: i32,\n",
                "    0x4: i32,\n",
                "    0x8: i32,\n",
                "    0xC: i32,\n",
                "}"
            ),
            StructType {
                size: StructSize::Fixed { size: 0x10 },
                alignment: Alignment::new(0x4).unwrap(),
                members: vec![
                    StructMember {
                        offset: 0x0,
                        member_type: IntegerType::Int32.intern(&global_state)
                    },
                    StructMember {
                        offset: 0x4,
                        member_type: IntegerType::Int32.intern(&global_state)
                    },
                    StructMember {
                        offset: 0x8,
                        member_type: IntegerType::Int32.intern(&global_state)
                    },
                    StructMember {
                        offset: 0xC,
                        member_type: IntegerType::Int32.intern(&global_state)
                    }
                ]
            }
        );
        test_type!(
            global_state,
            concat!(
                "struct 0 {\n",
                "    size: fixed 0x1,\n",
                "    align: 0x1,\n",
                "    0x0: struct 1 {\n",
                "        size: fixed 0x0,\n",
                "        align: 0x1,\n",
                "    },\n",
                "    0x1: struct 1 {},\n",
                "    0x0: i8,\n",
                "}"
            ),
            StructType {
                size: StructSize::Fixed { size: 0x1 },
                alignment: Alignment::new(0x1).unwrap(),
                members: vec![
                    StructMember {
                        offset: 0x0,
                        member_type: StructType {
                            size: StructSize::Fixed { size: 0x0 },
                            alignment: Alignment::new(0x1).unwrap(),
                            members: vec![]
                        }
                        .intern(&global_state)
                    },
                    StructMember {
                        offset: 0x1,
                        member_type: StructType {
                            size: StructSize::Fixed { size: 0x0 },
                            alignment: Alignment::new(0x1).unwrap(),
                            members: vec![]
                        }
                        .intern(&global_state)
                    },
                    StructMember {
                        offset: 0x0,
                        member_type: IntegerType::Int8.intern(&global_state)
                    }
                ]
            }
        );
        test_type!(
            global_state,
            concat!(
                "struct 0 {\n",
                "    size: variable 0x0,\n",
                "    align: 0x4,\n",
                "}"
            ),
            StructType {
                size: StructSize::Variable {
                    fixed_part_size: 0x0
                },
                alignment: Alignment::new(0x4).unwrap(),
                members: vec![]
            }
        );
        // FIXME: add tests for opaque types
    }
}
