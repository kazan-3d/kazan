// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
#![deny(missing_docs)]

//! Shader Compiler for Kazan

/// Shader Compiler Backend traits
pub mod backend {
    /// equivalent to LLVM's 'Module'
    pub trait Module<'a> {
        /// set's the source file name for this module
        fn set_source_file_name(&mut self, source_file_name: &str);
    }

    /// instance of a compiler backend; equivalent to LLVM's `LLVMContext`
    pub trait Context<'a> {
        /// the `Module` type
        type Module: Module<'a>;
        /// create a new `Module`
        fn create_module(&self, name: &str) -> Self::Module;
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
        /// get shader compiler's name
        fn name() -> &'static str;
        /// run a passed-in function with a new compiler context.
        /// this round-about method is used because generic associated types are not in stable Rust yet
        fn run_with_user<SCU: ShaderCompilerUser>(shader_compiler_user: SCU) -> SCU::ReturnType;
    }
}
