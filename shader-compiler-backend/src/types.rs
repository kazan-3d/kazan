// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

//! types in backend IR

use crate::Context;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

#[doc(hidden)]
#[macro_export]
macro_rules! buildable_struct_helper {
    {
        struct $name:ident {
            $($member_name:ident: $member_type:ty,)*
        }
    } => {
        impl $crate::types::BuildableType for $name {
            fn build<'a, Ty: $crate::types::Type<'a>, TB: $crate::types::TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
                type_builder.build_struct(&[$(<$member_type as $crate::types::BuildableType>::build(type_builder),)*])
            }
        }

        impl $crate::types::BuildableStruct for $name {
            fn get_members(
            ) -> &'static [$crate::types::BuildableStructMemberDescriptor] {
                #[allow(dead_code, non_camel_case_types)]
                #[repr(usize)]
                enum MemberIndices {
                    $($member_name,)*
                    __Last,
                }
                const MEMBERS: &'static [$crate::types::BuildableStructMemberDescriptor] = &[
                    $($crate::types::BuildableStructMemberDescriptor {
                        name: stringify!($member_name),
                        index: MemberIndices::$member_name as usize,
                    },)*
                ];
                MEMBERS
            }
        }
    }
}

/// create a struct that implements `BuildableType`
#[macro_export]
macro_rules! buildable_struct {
    {
        $(#[derive($derives:ident)])*
        pub struct $name:ident {
            $($member_name:ident: $member_type:ty,)*
        }
    } => {
        $(#[derive($derives)])*
        #[repr(C)]
        pub struct $name {
            $($member_name: $member_type,)*
        }

        buildable_struct_helper!{
            struct $name {
                $($member_name: $member_type,)*
            }
        }
    };
    {
        $(#[derive($derives:ident)])*
        struct $name:ident {
            $($member_name:ident: $member_type:ty,)*
        }
    } => {
        $(#[derive($derives)])*
        #[repr(C)]
        struct $name {
            $($member_name: $member_type,)*
        }

        buildable_struct_helper!{
            struct $name {
                $($member_name: $member_type,)*
            }
        }
    };
}

/// length of a vector
pub enum VectorLength {
    /// fixed length vector
    Fixed {
        /// length in elements
        length: u32,
    },
    /// variable length vector
    Variable {
        /// base length in elements which the runtime vector length is a multiple of
        base_length: u32,
    },
}

/// equivalent to LLVM's 'Type'
pub trait Type<'a>: Clone + Eq + Hash + Debug {
    /// the `Context` type
    type Context: Context<'a>;
}

/// trait for building types
pub trait TypeBuilder<'a, Ty: Type<'a>> {
    /// build a `bool` type
    fn build_bool(&self) -> Ty;
    /// build an 8-bit 2's complement integer type
    fn build_i8(&self) -> Ty;
    /// build an 16-bit 2's complement integer type
    fn build_i16(&self) -> Ty;
    /// build an 32-bit 2's complement integer type
    fn build_i32(&self) -> Ty;
    /// build an 64-bit 2's complement integer type
    fn build_i64(&self) -> Ty;
    /// build an 8-bit unsigned integer type
    fn build_u8(&self) -> Ty;
    /// build an 16-bit unsigned integer type
    fn build_u16(&self) -> Ty;
    /// build an 32-bit unsigned integer type
    fn build_u32(&self) -> Ty;
    /// build an 64-bit unsigned integer type
    fn build_u64(&self) -> Ty;
    /// build an 32-bit IEEE 754 floating-point type
    fn build_f32(&self) -> Ty;
    /// build an 64-bit IEEE 754 floating-point type
    fn build_f64(&self) -> Ty;
    /// build a pointer
    fn build_pointer(&self, target: Ty) -> Ty;
    /// build an array
    fn build_array(&self, element: Ty, count: usize) -> Ty;
    /// build a vector
    fn build_vector(&self, element: Ty, length: VectorLength) -> Ty;
    /// build a struct
    fn build_struct(&self, members: &[Ty]) -> Ty;
    /// build a function type
    fn build_function(&self, arguments: &[Ty], return_type: Option<Ty>) -> Ty;
    /// build a type
    fn build<T: BuildableType>(&self) -> Ty
    where
        Self: Sized,
    {
        T::build(self)
    }
}

/// trait for rust types that can be built using `TypeBuilder`
pub trait BuildableType {
    /// build the type represented by `Self`
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty;
}

impl<T: BuildableType> BuildableType for UnsafeCell<T> {
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
        T::build(type_builder)
    }
}

mod hidden {
    pub trait ScalarBuildableTypeBase {}
}

impl<T: hidden::ScalarBuildableTypeBase> hidden::ScalarBuildableTypeBase for UnsafeCell<T> {}

/// trait for rust types that can be an element of a vector and be built using `TypeBuilder`
pub trait ScalarBuildableType: BuildableType + hidden::ScalarBuildableTypeBase {}

impl<T: ScalarBuildableType> ScalarBuildableType for UnsafeCell<T> {}

/// descriptor for members of types implementing `BuildableStruct`
pub struct BuildableStructMemberDescriptor {
    /// name of member
    pub name: &'static str,
    /// index of member
    pub index: usize,
}

/// trait for structs that can be built using TypeBuilder
/// implementing types are usually created using `buildable_struct!`
pub trait BuildableStruct: BuildableType {
    /// get the list of members for `Self`
    fn get_members() -> &'static [BuildableStructMemberDescriptor];
    /// get the member for `Self` that is named `name`
    fn get_member_by_name(name: &str) -> &'static BuildableStructMemberDescriptor {
        for member in Self::get_members() {
            if name == member.name {
                return member;
            }
        }
        unreachable!("{} is not a member", name);
    }
}

macro_rules! build_basic_scalar {
    ($type:ty, $build_fn:ident) => {
        impl BuildableType for $type {
            fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
                type_builder.$build_fn()
            }
        }

        impl hidden::ScalarBuildableTypeBase for $type {}

        impl ScalarBuildableType for $type {}
    };
}

build_basic_scalar!(bool, build_bool);
build_basic_scalar!(u8, build_u8);
build_basic_scalar!(i8, build_i8);
build_basic_scalar!(u16, build_u16);
build_basic_scalar!(i16, build_i16);
build_basic_scalar!(u32, build_u32);
build_basic_scalar!(i32, build_i32);
build_basic_scalar!(u64, build_u64);
build_basic_scalar!(i64, build_i64);
build_basic_scalar!(f32, build_f32);
build_basic_scalar!(f64, build_f64);

impl<'b, T: BuildableType> BuildableType for Option<&'b T> {
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
        type_builder.build_pointer(T::build(type_builder))
    }
}

impl<'b, T: BuildableType> hidden::ScalarBuildableTypeBase for Option<&'b T> {}

impl<'b, T: BuildableType> ScalarBuildableType for Option<&'b T> {}

impl<'b, T: BuildableType> BuildableType for Option<&'b mut T> {
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
        type_builder.build_pointer(T::build(type_builder))
    }
}

impl<'b, T: BuildableType> hidden::ScalarBuildableTypeBase for Option<&'b mut T> {}

impl<'b, T: BuildableType> ScalarBuildableType for Option<&'b mut T> {}

impl<'b, T: BuildableType> BuildableType for &'b T {
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
        type_builder.build_pointer(T::build(type_builder))
    }
}

impl<'b, T: BuildableType> hidden::ScalarBuildableTypeBase for &'b T {}

impl<'b, T: BuildableType> ScalarBuildableType for &'b T {}

impl<'b, T: BuildableType> BuildableType for &'b mut T {
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
        type_builder.build_pointer(T::build(type_builder))
    }
}

impl<'b, T: BuildableType> hidden::ScalarBuildableTypeBase for &'b mut T {}

impl<'b, T: BuildableType> ScalarBuildableType for &'b mut T {}

impl<T: BuildableType> BuildableType for *mut T {
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
        type_builder.build_pointer(T::build(type_builder))
    }
}

impl<'b, T: BuildableType> hidden::ScalarBuildableTypeBase for *mut T {}

impl<'b, T: BuildableType> ScalarBuildableType for *mut T {}

impl<T: BuildableType> BuildableType for *const T {
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
        type_builder.build_pointer(T::build(type_builder))
    }
}

impl<'b, T: BuildableType> hidden::ScalarBuildableTypeBase for *const T {}

impl<'b, T: BuildableType> ScalarBuildableType for *const T {}

impl<T: BuildableType> BuildableType for NonNull<T> {
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
        type_builder.build_pointer(T::build(type_builder))
    }
}

impl<'b, T: BuildableType> hidden::ScalarBuildableTypeBase for NonNull<T> {}

impl<'b, T: BuildableType> ScalarBuildableType for NonNull<T> {}

impl<T: BuildableType> BuildableType for Option<NonNull<T>> {
    fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
        type_builder.build_pointer(T::build(type_builder))
    }
}

impl<'b, T: BuildableType> hidden::ScalarBuildableTypeBase for Option<NonNull<T>> {}

impl<'b, T: BuildableType> ScalarBuildableType for Option<NonNull<T>> {}

macro_rules! build_unit_function_type {
        ($($arguments:ident,)*) => {
            impl<$($arguments: BuildableType),*> BuildableType for Option<unsafe extern "C" fn($($arguments,)*)> {
                fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
                    type_builder.build_function(&[$($arguments::build(type_builder),)*], None)
                }
            }

            impl<$($arguments: BuildableType),*> hidden::ScalarBuildableTypeBase for Option<unsafe extern "C" fn($($arguments,)*)> {}

            impl<$($arguments: BuildableType),*> ScalarBuildableType for Option<unsafe extern "C" fn($($arguments,)*)> {}

            impl<$($arguments: BuildableType),*> BuildableType for unsafe extern "C" fn($($arguments,)*) {
                fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
                    type_builder.build_function(&[$($arguments::build(type_builder),)*], None)
                }
            }

            impl<$($arguments: BuildableType),*> hidden::ScalarBuildableTypeBase for unsafe extern "C" fn($($arguments,)*) {}

            impl<$($arguments: BuildableType),*> ScalarBuildableType for unsafe extern "C" fn($($arguments,)*) {}
        };
    }

macro_rules! build_function_type {
        ($($arguments:ident,)*) => {
            impl<R: BuildableType, $($arguments: BuildableType),*> BuildableType for Option<unsafe extern "C" fn($($arguments,)*) -> R> {
                fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
                    type_builder.build_function(&[$($arguments::build(type_builder),)*], Some(R::build(type_builder)))
                }
            }

            impl<R: BuildableType, $($arguments: BuildableType),*> hidden::ScalarBuildableTypeBase for Option<unsafe extern "C" fn($($arguments,)*) -> R> {}

            impl<R: BuildableType, $($arguments: BuildableType),*> ScalarBuildableType for Option<unsafe extern "C" fn($($arguments,)*) -> R> {}

            impl<R: BuildableType, $($arguments: BuildableType),*> BuildableType for unsafe extern "C" fn($($arguments,)*) -> R {
                fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
                    type_builder.build_function(&[$($arguments::build(type_builder),)*], Some(R::build(type_builder)))
                }
            }

            impl<R: BuildableType, $($arguments: BuildableType),*> hidden::ScalarBuildableTypeBase for unsafe extern "C" fn($($arguments,)*) -> R {}

            impl<R: BuildableType, $($arguments: BuildableType),*> ScalarBuildableType for unsafe extern "C" fn($($arguments,)*) -> R {}

        };
    }

macro_rules! build_function_types {
        () => {
            build_unit_function_type!();
            build_function_type!();
        };
        ($first_argument:ident, $($arguments:ident,)*) => {
            build_unit_function_type!($first_argument, $($arguments,)*);
            build_function_type!($first_argument, $($arguments,)*);
            build_function_types!($($arguments,)*);
        }
    }

build_function_types!(
    T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
);

macro_rules! build_array0 {
    ($length:expr) => {
        impl<T: BuildableType> BuildableType for [T; $length + 1] {
            fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
                type_builder.build_array(T::build(type_builder), $length + 1)
            }
        }
    };
}

macro_rules! build_array1 {
    ($length:expr) => {
        build_array0!($length * 2);
        build_array0!($length * 2 + 1);
    };
}

macro_rules! build_array2 {
    ($length:expr) => {
        build_array1!($length * 2);
        build_array1!($length * 2 + 1);
    };
}

macro_rules! build_array3 {
    ($length:expr) => {
        build_array2!($length * 2);
        build_array2!($length * 2 + 1);
    };
}

macro_rules! build_array4 {
    ($length:expr) => {
        build_array3!($length * 2);
        build_array3!($length * 2 + 1);
    };
}

macro_rules! build_array5 {
    ($length:expr) => {
        build_array4!($length * 2);
        build_array4!($length * 2 + 1);
    };
}

build_array5!(0);
build_array5!(1);

/// buildable vector types
pub trait Vector: BuildableType {
    /// element type
    type Element: ScalarBuildableType;
    /// vector length
    const LENGTH: VectorLength;
}

#[doc(hidden)]
pub enum __VectorNeverType {}

macro_rules! build_fixed_vector {
    ($name:ident, $length:expr) => {
        /// Vector of elements `Element`
        #[derive(Copy, Clone)]
        pub struct $name<Element: ScalarBuildableType> {
            /// elements of the vector `Self`
            pub elements: [Element; $length],
        }

        impl<Element: ScalarBuildableType> Deref for $name<Element> {
            type Target = [Element; $length];
            fn deref(&self) -> &Self::Target {
                &self.elements
            }
        }

        impl<Element: ScalarBuildableType> DerefMut for $name<Element> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.elements
            }
        }

        impl<Element: ScalarBuildableType> BuildableType for $name<Element> {
            fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
                type_builder.build_vector(Element::build(type_builder), Self::LENGTH)
            }
        }

        impl<Element: ScalarBuildableType> Vector for $name<Element> {
            type Element = Element;
            const LENGTH: VectorLength = { VectorLength::Fixed { length: $length } };
        }
    };
}

macro_rules! build_variable_vector {
    ($name:ident, $base_length:expr) => {
        /// Vector of elements `Element`
        pub enum $name<Element: ScalarBuildableType> {
            #[doc(hidden)]
            __Dummy(__VectorNeverType, PhantomData<Element>),
        }

        impl<Element: ScalarBuildableType> BuildableType for $name<Element> {
            fn build<'a, Ty: Type<'a>, TB: TypeBuilder<'a, Ty>>(type_builder: &TB) -> Ty {
                type_builder.build_vector(Element::build(type_builder), Self::LENGTH)
            }
        }

        impl<Element: ScalarBuildableType> Vector for $name<Element> {
            type Element = Element;
            const LENGTH: VectorLength = {
                VectorLength::Variable {
                    base_length: $base_length,
                }
            };
        }
    };
}

/// alternate name for `VecNx1`
pub type VecN<Element> = VecNx1<Element>;

build_fixed_vector!(Vec1, 1);
build_fixed_vector!(Vec2, 2);
build_fixed_vector!(Vec3, 3);
build_fixed_vector!(Vec4, 4);
build_fixed_vector!(Vec5, 5);
build_fixed_vector!(Vec6, 6);
build_fixed_vector!(Vec7, 7);
build_fixed_vector!(Vec8, 8);
build_fixed_vector!(Vec9, 9);
build_fixed_vector!(Vec10, 10);
build_fixed_vector!(Vec11, 11);
build_fixed_vector!(Vec12, 12);
build_fixed_vector!(Vec13, 13);
build_fixed_vector!(Vec14, 14);
build_fixed_vector!(Vec15, 15);
build_fixed_vector!(Vec16, 16);
build_variable_vector!(VecNx1, 1);
build_variable_vector!(VecNx2, 2);
build_variable_vector!(VecNx3, 3);
build_variable_vector!(VecNx4, 4);
build_variable_vector!(VecNx5, 5);
build_variable_vector!(VecNx6, 6);
build_variable_vector!(VecNx7, 7);
build_variable_vector!(VecNx8, 8);
build_variable_vector!(VecNx9, 9);
build_variable_vector!(VecNx10, 10);
build_variable_vector!(VecNx11, 11);
build_variable_vector!(VecNx12, 12);
build_variable_vector!(VecNx13, 13);
build_variable_vector!(VecNx14, 14);
build_variable_vector!(VecNx15, 15);
build_variable_vector!(VecNx16, 16);
