// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
extern crate shader_compiler_backend;

#[cfg_attr(feature = "cargo-clippy", allow(clippy::const_static_lifetime))]
#[allow(dead_code)]
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
mod llvm {
    include!(concat!(env!("OUT_DIR"), "/llvm_c.rs"));
}

mod backend;
mod tests;

pub use backend::LLVM7CompilerConfig;

pub const LLVM_7_SHADER_COMPILER: backend::LLVM7Compiler = backend::LLVM7Compiler;
