// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
extern crate llvm_sys;
extern crate shader_compiler;

mod backend;
mod tests;

pub use backend::LLVM7CompilerConfig;

pub const LLVM_7_SHADER_COMPILER: backend::LLVM7Compiler = backend::LLVM7Compiler;
