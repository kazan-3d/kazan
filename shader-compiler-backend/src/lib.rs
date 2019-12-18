// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![deny(missing_docs)]

//! Shader Compiler Backend Traits for Kazan

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hash;
use std::io;
use std::marker::PhantomData;

#[macro_use]
pub mod types;

/// equivalent to LLVM's 'IRBuilder'
pub trait AttachedBuilder<'a>: Sized {
    /// the `Context` type
    type Context: Context<'a>;
    /// get the current `BasicBlock`
    fn current_basic_block(&self) -> <Self::Context as Context<'a>>::BasicBlock;
    /// build an alloca instruction
    fn build_alloca(
        &mut self,
        variable_type: <Self::Context as Context<'a>>::Type,
    ) -> <Self::Context as Context<'a>>::Value;
    /// build a return instruction
    fn build_return(
        self,
        value: Option<<Self::Context as Context<'a>>::Value>,
    ) -> <Self::Context as Context<'a>>::DetachedBuilder;
}

/// equivalent to LLVM's 'IRBuilder'
pub trait DetachedBuilder<'a>: Sized {
    /// the `Context` type
    type Context: Context<'a>;
    /// attach `basic_block` to `Self`, converting into an `AttachedBuilder`
    fn attach(
        self,
        basic_block: <Self::Context as Context<'a>>::BuildableBasicBlock,
    ) -> <Self::Context as Context<'a>>::AttachedBuilder;
}

/// equivalent to LLVM's 'Value'
pub trait Value<'a>: Clone + Debug {
    /// the `Context` type
    type Context: Context<'a>;
}

/// equivalent to LLVM's 'BasicBlock'
pub trait BasicBlock<'a>: Clone + Debug {
    /// the `Context` type
    type Context: Context<'a>;
    /// get the `Value` corresponding to `Self`
    fn as_value(&self) -> <Self::Context as Context<'a>>::Value;
}

/// equivalent to LLVM's 'BasicBlock'
pub trait BuildableBasicBlock<'a>: Debug + Sized {
    /// the `Context` type
    type Context: Context<'a>;
    /// get the `BasicBlock` corresponding to `Self`
    fn as_basic_block(&self) -> <Self::Context as Context<'a>>::BasicBlock;
    /// get the `Value` corresponding to `Self`
    fn as_value(&self) -> <Self::Context as Context<'a>>::Value {
        self.as_basic_block().as_value()
    }
}

/// equivalent to LLVM's 'Function'
pub trait Function<'a>: Debug + Sized {
    /// the `Context` type
    type Context: Context<'a>;
    /// get the `Value` corresponding to `Self`
    fn as_value(&self) -> <Self::Context as Context<'a>>::Value;
    /// append a new `BasicBlock` to `Self`
    fn append_new_basic_block(
        &mut self,
        name: Option<&str>,
    ) -> <Self::Context as Context<'a>>::BuildableBasicBlock;
    /// get this function's parameters
    fn parameters(&self) -> &[<Self::Context as Context<'a>>::Value];
}

/// module verification failure; returned from `Module::verify`
pub struct VerificationFailure<'a, M: Module<'a>> {
    module: M,
    message: String,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a, M: Module<'a>> VerificationFailure<'a, M> {
    /// create a new `VerificationFailure`
    pub fn new<T: ToString + ?Sized>(module: M, message: &T) -> Self {
        VerificationFailure {
            module,
            message: message.to_string(),
            _phantom_data: PhantomData,
        }
    }
    /// get the `Module` that failed verification
    pub fn into_module(self) -> M {
        self.module
    }
}

impl<'a, M: Module<'a>> fmt::Display for VerificationFailure<'a, M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "module verification failure: {}", self.message,)
    }
}

impl<'a, M: Module<'a>> Debug for VerificationFailure<'a, M> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VerificationFailure")
            .field("message", &self.message)
            .field("module", &self.module)
            .finish()
    }
}

impl<'a, M: Module<'a>> Error for VerificationFailure<'a, M> {}

impl<'a, M: Module<'a>> From<VerificationFailure<'a, M>> for io::Error {
    fn from(v: VerificationFailure<'a, M>) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("{}", v))
    }
}

/// equivalent to LLVM's 'Module'
pub trait Module<'a>: Debug + Sized {
    /// the `Context` type
    type Context: Context<'a>;
    /// set's the source file name for this module
    fn set_source_file_name(&mut self, source_file_name: &str);
    /// add a new empty function to `Self`
    fn add_function(
        &mut self,
        name: &str,
        ty: <Self::Context as Context<'a>>::Type,
    ) -> <Self::Context as Context<'a>>::Function;
    /// verify `Self`, converting into a `VerifiedModule`
    fn verify(
        self,
    ) -> Result<<Self::Context as Context<'a>>::VerifiedModule, VerificationFailure<'a, Self>>;
    /// convert into a `VerifiedModule` without verifying
    ///
    /// # Safety
    ///
    /// Must pass `verify`
    unsafe fn to_verified_module_unchecked(self) -> <Self::Context as Context<'a>>::VerifiedModule;
}

/// equivalent to LLVM's 'Module'; create using `Module::verify` or `Module::to_verified_module_unchecked`
pub trait VerifiedModule<'a>: Debug + Sized {
    /// the `Context` type
    type Context: Context<'a>;
    /// convert back to an unverified module
    fn into_module(self) -> <Self::Context as Context<'a>>::Module;
}

/// instance of a compiler backend; equivalent to LLVM's `LLVMContext`
pub trait Context<'a>: Sized + fmt::Debug {
    /// the `Value` type
    type Value: Value<'a, Context = Self>;
    /// the `BasicBlock` type
    type BasicBlock: BasicBlock<'a, Context = Self>;
    /// the `BuildableBasicBlock` type
    type BuildableBasicBlock: BuildableBasicBlock<'a, Context = Self>;
    /// the `Function` type
    type Function: Function<'a, Context = Self>;
    /// the `Module` type
    type Module: Module<'a, Context = Self>;
    /// the `VerifiedModule` type
    type VerifiedModule: VerifiedModule<'a, Context = Self>;
    /// the `AttachedBuilder` type
    type AttachedBuilder: AttachedBuilder<'a, Context = Self>;
    /// the `DetachedBuilder` type
    type DetachedBuilder: DetachedBuilder<'a, Context = Self>;
    /// the `Type` type
    type Type: types::Type<'a, Context = Self>;
    /// the `TypeBuilder` type
    type TypeBuilder: types::TypeBuilder<'a, Self::Type>;
    /// create a new `Module`
    fn create_module(&self, name: &str) -> Self::Module;
    /// create a new `DetachedBuilder`
    fn create_builder(&self) -> Self::DetachedBuilder;
    /// create a new `TypeBuilder`
    fn create_type_builder(&self) -> Self::TypeBuilder;
}

/// inputs to the final compilation
pub struct CompileInputs<'a, C: Context<'a>, K: Hash + Eq + Send + Sync + 'static> {
    /// the input module
    pub module: C::VerifiedModule,
    /// the list of functions that can be called from the final `CompiledCode`
    pub callable_functions: HashMap<K, C::Function>,
}

/// the final compiled code
pub trait CompiledCode<K: Hash + Eq + Send + Sync + 'static>: Send + Sync {
    /// get a function in the final compiled code.
    /// the returned function needs to be cast to the correct type and
    /// `Self` needs to still exist while the returned function exists
    fn get(&self, which: &K) -> Option<unsafe extern "C" fn()>;
}

/// trait that the user of `Compiler` implements
pub trait CompilerUser {
    /// the type used as a key for visible functions
    type FunctionKey: Hash + Eq + Send + Sync + 'static;
    /// the user's error type
    type Error;
    /// create an instance of `Error`
    fn create_error(message: String) -> Self::Error;
    /// the function that the user of `Compiler` implements
    fn run<'a, C: Context<'a>>(
        self,
        context: &'a C,
    ) -> Result<CompileInputs<'a, C, Self::FunctionKey>, Self::Error>;
}

/// optimization mode
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum OptimizationMode {
    /// no optimizations are enabled
    NoOptimizations,
    /// default optimizations are enabled
    Normal,
}

impl Default for OptimizationMode {
    fn default() -> Self {
        OptimizationMode::Normal
    }
}

/// compiler independent config options
#[derive(Clone, Debug, Default)]
pub struct CompilerIndependentConfig {
    /// optimization mode
    pub optimization_mode: OptimizationMode,
}

/// main compiler backend trait
pub trait Compiler: Copy + Send + Sync + 'static {
    /// the compiler's configuration
    type Config: Default + Clone + From<CompilerIndependentConfig> + Send + Sync;
    /// get shader compiler's name
    fn name(self) -> &'static str;
    /// run a passed-in function with a new compiler context.
    /// this round-about method is used because generic associated types are not in stable Rust yet
    fn run<U: CompilerUser>(
        self,
        user: U,
        config: Self::Config,
    ) -> Result<Box<dyn CompiledCode<U::FunctionKey>>, U::Error>;
}

#[cfg(test)]
mod test {
    #![allow(dead_code)]

    buildable_struct! {
        struct S1 {
        }
    }

    buildable_struct! {
        pub struct S2 {
            v: u32,
        }
    }

    buildable_struct! {
        struct S3 {
            p: *mut S2,
            v: crate::types::VecNx4<f32>,
        }
    }
}
