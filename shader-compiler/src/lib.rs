// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
#![deny(missing_docs)]

//! Shader Compiler for Kazan

/// Shader Compiler Backend traits
pub mod backend {
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::marker::PhantomData;

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
    pub trait Type<'a>: Clone + Eq + Hash + Debug {}

    /// trait for building types
    pub trait TypeBuilder<'a>: Sized {
        /// the `Type` type
        type Type: Type<'a>;
        /// build a `bool` type
        fn build_bool(&self) -> Self::Type;
        /// build an 8-bit sign-agnostic integer type
        fn build_i8(&self) -> Self::Type;
        /// build an 16-bit sign-agnostic integer type
        fn build_i16(&self) -> Self::Type;
        /// build an 32-bit sign-agnostic integer type
        fn build_i32(&self) -> Self::Type;
        /// build an 64-bit sign-agnostic integer type
        fn build_i64(&self) -> Self::Type;
        /// build an 32-bit IEEE 754 floating-point type
        fn build_f32(&self) -> Self::Type;
        /// build an 64-bit IEEE 754 floating-point type
        fn build_f64(&self) -> Self::Type;
        /// build a pointer
        fn build_pointer(&self, target: Self::Type) -> Self::Type;
        /// build an array
        fn build_array(&self, element: Self::Type, count: usize) -> Self::Type;
        /// build a vector
        fn build_vector(&self, element: Self::Type, length: VectorLength) -> Self::Type;
        /// build a type
        fn build<T: BuildableType>(&self) -> Self::Type {
            T::build(self)
        }
    }

    /// trait for rust types that can be built using `TypeBuilder`
    pub trait BuildableType {
        /// build the type represented by `Self`
        fn build<'a, TB: TypeBuilder<'a>>(type_builder: &TB) -> TB::Type;
    }

    /// trait for rust types that can be an element of a vector and be built using `TypeBuilder`
    pub trait ScalarBuildableType: BuildableType {}

    macro_rules! build_basic_scalar {
        ($type:ty, $build_fn:ident) => {
            impl BuildableType for $type {
                fn build<'a, TB: TypeBuilder<'a>>(type_builder: &TB) -> TB::Type {
                    type_builder.$build_fn()
                }
            }

            impl ScalarBuildableType for $type {}
        };
    }

    build_basic_scalar!(bool, build_bool);
    build_basic_scalar!(u8, build_i8);
    build_basic_scalar!(i8, build_i8);
    build_basic_scalar!(u16, build_i16);
    build_basic_scalar!(i16, build_i16);
    build_basic_scalar!(u32, build_i32);
    build_basic_scalar!(i32, build_i32);
    build_basic_scalar!(u64, build_i64);
    build_basic_scalar!(i64, build_i64);
    build_basic_scalar!(f32, build_f32);
    build_basic_scalar!(f64, build_f64);

    impl<'b, T: BuildableType> BuildableType for &'b T {
        fn build<'a, TB: TypeBuilder<'a>>(type_builder: &TB) -> TB::Type {
            type_builder.build_pointer(T::build(type_builder))
        }
    }

    impl<'b, T: BuildableType> ScalarBuildableType for &'b T {}

    impl<'b, T: BuildableType> BuildableType for &'b mut T {
        fn build<'a, TB: TypeBuilder<'a>>(type_builder: &TB) -> TB::Type {
            type_builder.build_pointer(T::build(type_builder))
        }
    }

    impl<'b, T: BuildableType> ScalarBuildableType for &'b mut T {}

    impl<T: BuildableType> BuildableType for *mut T {
        fn build<'a, TB: TypeBuilder<'a>>(type_builder: &TB) -> TB::Type {
            type_builder.build_pointer(T::build(type_builder))
        }
    }

    impl<'b, T: BuildableType> ScalarBuildableType for *mut T {}

    impl<T: BuildableType> BuildableType for *const T {
        fn build<'a, TB: TypeBuilder<'a>>(type_builder: &TB) -> TB::Type {
            type_builder.build_pointer(T::build(type_builder))
        }
    }

    impl<'b, T: BuildableType> ScalarBuildableType for *const T {}

    macro_rules! build_array0 {
        ($length:expr) => {
            impl<T: BuildableType> BuildableType for [T; $length + 1] {
                fn build<'a, TB: TypeBuilder<'a>>(type_builder: &TB) -> TB::Type {
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

    macro_rules! build_vector {
        ($name:ident, $length:expr) => {
            /// Vector of elements `Element`
            pub struct $name<Element: ScalarBuildableType>(PhantomData<*const Element>);

            impl<Element: ScalarBuildableType> BuildableType for $name<Element> {
                fn build<'a, TB: TypeBuilder<'a>>(type_builder: &TB) -> TB::Type {
                    type_builder.build_vector(Element::build(type_builder), Self::LENGTH)
                }
            }

            impl<Element: ScalarBuildableType> Vector for $name<Element> {
                type Element = Element;
                const LENGTH: VectorLength = {
                    use self::VectorLength::*;
                    $length
                };
            }
        };
    }

    build_vector!(Vec1, Fixed { length: 1 });
    build_vector!(VecN, Variable { base_length: 1 });
    /// alternate name for VecN
    pub type VecNx1<Element> = VecN<Element>;
    build_vector!(Vec2, Fixed { length: 2 });
    build_vector!(VecNx2, Variable { base_length: 2 });
    build_vector!(Vec3, Fixed { length: 3 });
    build_vector!(VecNx3, Variable { base_length: 3 });
    build_vector!(Vec4, Fixed { length: 4 });
    build_vector!(VecNx4, Variable { base_length: 4 });
    build_vector!(Vec5, Fixed { length: 5 });
    build_vector!(VecNx5, Variable { base_length: 5 });
    build_vector!(Vec6, Fixed { length: 6 });
    build_vector!(VecNx6, Variable { base_length: 6 });
    build_vector!(Vec7, Fixed { length: 7 });
    build_vector!(VecNx7, Variable { base_length: 7 });
    build_vector!(Vec8, Fixed { length: 8 });
    build_vector!(VecNx8, Variable { base_length: 8 });
    build_vector!(Vec9, Fixed { length: 9 });
    build_vector!(VecNx9, Variable { base_length: 9 });
    build_vector!(Vec10, Fixed { length: 10 });
    build_vector!(VecNx10, Variable { base_length: 10 });
    build_vector!(Vec11, Fixed { length: 11 });
    build_vector!(VecNx11, Variable { base_length: 11 });
    build_vector!(Vec12, Fixed { length: 12 });
    build_vector!(VecNx12, Variable { base_length: 12 });
    build_vector!(Vec13, Fixed { length: 13 });
    build_vector!(VecNx13, Variable { base_length: 13 });
    build_vector!(Vec14, Fixed { length: 14 });
    build_vector!(VecNx14, Variable { base_length: 14 });
    build_vector!(Vec15, Fixed { length: 15 });
    build_vector!(VecNx15, Variable { base_length: 15 });
    build_vector!(Vec16, Fixed { length: 16 });
    build_vector!(VecNx16, Variable { base_length: 16 });

    /// equivalent to LLVM's 'IRBuilder'
    pub trait Builder<'a> {}

    /// equivalent to LLVM's 'Module'
    pub trait Module<'a> {
        /// set's the source file name for this module
        fn set_source_file_name(&mut self, source_file_name: &str);
    }

    /// instance of a compiler backend; equivalent to LLVM's `LLVMContext`
    pub trait Context<'a> {
        /// the `Module` type
        type Module: Module<'a>;
        /// the `Builder` type
        type Builder: Builder<'a>;
        /// the `Type` type
        type Type: Type<'a>;
        /// the `TypeBuilder` type
        type TypeBuilder: TypeBuilder<'a, Type = Self::Type>;
        /// create a new `Module`
        fn create_module(&self, name: &str) -> Self::Module;
        /// create a new `Builder`
        fn create_builder(&self) -> Self::Builder;
        /// create a new `TypeBuilder`
        fn create_type_builder(&self) -> Self::TypeBuilder;
    }

    /// trait that the user of `ShaderCompiler` implements
    pub trait ShaderCompilerUser {
        /// the return type of `run_with_context`
        type ReturnType;
        /// the function that the user of `ShaderCompiler` implements
        fn run_with_context<'a, C: Context<'a>>(self, context: &'a C) -> Self::ReturnType;
    }

    /// main shader compiler backend trait
    pub trait ShaderCompiler: Send + Sync + 'static {
        /// the shader compiler's configuration
        type Config: Default + Clone;
        /// get shader compiler's name
        fn name() -> &'static str;
        /// run a passed-in function with a new compiler context.
        /// this round-about method is used because generic associated types are not in stable Rust yet
        fn run_with_user<SCU: ShaderCompilerUser>(
            shader_compiler_user: SCU,
            config: Self::Config,
        ) -> SCU::ReturnType;
    }
}
