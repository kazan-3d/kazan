// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![cfg_attr(not(test), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

#[macro_use]
mod macros;

mod cfg;
mod constants;
mod decorations;
mod errors;
mod functions;
mod io_layout;
mod parse;
mod structure_tree;
mod types;
mod values;

pub use crate::errors::*;

use alloc::vec::Vec;
use core::{fmt, slice};
use shader_compiler_ir::{GlobalState, Internable, Interned, Module, TargetProperties};
use spirv_parser::{ExecutionModel, InstructionAndLocation};

pub struct UnresolvedSpecialization {
    pub constant_id: u32,
}

macro_rules! decl_specialization_resolver {
    (
        $(
            $(#[doc = $doc:expr])+
            $fn_name:ident -> $ty:ty;
        )+
    ) => {
        pub trait SpecializationResolver {
            $(
                $(#[doc = $doc])+
                fn $fn_name(
                    &mut self,
                    unresolved_specialization: UnresolvedSpecialization,
                    default: $ty,
                ) -> Result<$ty, SpecializationResolutionFailed>;
            )+
        }

        impl SpecializationResolver for DefaultSpecializationResolver {
            $(
                fn $fn_name(
                    &mut self,
                    _unresolved_specialization: UnresolvedSpecialization,
                    default: $ty,
                ) -> Result<$ty, SpecializationResolutionFailed> {
                    Ok(default)
                }
            )+
        }
    };
}

decl_specialization_resolver! {
    /// resolve a boolean specialization constant
    resolve_bool -> bool;
    /// resolve an unsigned 8-bit integer specialization constant
    resolve_u8 -> u8;
    /// resolve a signed 8-bit integer specialization constant
    resolve_i8 -> i8;
    /// resolve an unsigned 16-bit integer specialization constant
    resolve_u16 -> u16;
    /// resolve a signed 16-bit integer specialization constant
    resolve_i16 -> i16;
    /// resolve an unsigned 32-bit integer specialization constant
    resolve_u32 -> u32;
    /// resolve a signed 32-bit integer specialization constant
    resolve_i32 -> i32;
    /// resolve an unsigned 64-bit integer specialization constant
    resolve_u64 -> u64;
    /// resolve a signed 64-bit integer specialization constant
    resolve_i64 -> i64;
    /// resolve a 16-bit float specialization constant
    resolve_f16 -> shader_compiler_ir::Float16;
    /// resolve a 32-bit float specialization constant
    resolve_f32 -> shader_compiler_ir::Float32;
    /// resolve a 64-bit float specialization constant
    resolve_f64 -> shader_compiler_ir::Float64;
}

#[derive(Default)]
pub struct DefaultSpecializationResolver;

#[derive(Clone)]
struct SPIRVInstructionLocation<'i> {
    instruction_index: usize,
    iter: slice::Iter<'i, InstructionAndLocation>,
}

impl<'i> fmt::Debug for SPIRVInstructionLocation<'i> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((instruction, _)) = self.clone().next() {
            writeln!(f, "{}", instruction)
        } else {
            writeln!(f, "<EOF>")
        }
    }
}

impl<'i> SPIRVInstructionLocation<'i> {
    fn get_instruction(&self) -> Option<&'i spirv_parser::Instruction> {
        self.clone()
            .next()
            .map(|(instruction, _)| &instruction.instruction)
    }
}

impl<'i> Iterator for SPIRVInstructionLocation<'i> {
    type Item = (&'i InstructionAndLocation, Self);
    fn next(&mut self) -> Option<Self::Item> {
        let location = self.clone();
        let instruction = self.iter.next()?;
        self.instruction_index += 1;
        Some((instruction, location))
    }
}

struct TranslationStateBase<'g, 'i> {
    global_state: &'g GlobalState<'g>,
    target_properties: Interned<'g, TargetProperties>,
    specialization_resolver: &'i mut dyn SpecializationResolver,
    debug_output: &'i mut dyn fmt::Write,
    entry_point_name: &'i str,
    entry_point_execution_model: ExecutionModel,
    spirv_header: spirv_parser::Header,
    spirv_instructions: &'i [InstructionAndLocation],
    spirv_instructions_current_location: SPIRVInstructionLocation<'i>,
    spirv_instructions_next_location: SPIRVInstructionLocation<'i>,
}

impl<'g, 'i> TranslationStateBase<'g, 'i> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        global_state: &'g GlobalState<'g>,
        target_properties: Interned<'g, TargetProperties>,
        specialization_resolver: &'i mut dyn SpecializationResolver,
        debug_output: &'i mut dyn fmt::Write,
        entry_point_name: &'i str,
        entry_point_execution_model: ExecutionModel,
        spirv_header: spirv_parser::Header,
        spirv_instructions: &'i [InstructionAndLocation],
    ) -> Self {
        let spirv_instructions_location = SPIRVInstructionLocation {
            instruction_index: 0,
            iter: spirv_instructions.iter(),
        };
        Self {
            global_state,
            target_properties,
            specialization_resolver,
            debug_output,
            entry_point_name,
            entry_point_execution_model,
            spirv_header,
            spirv_instructions,
            spirv_instructions_current_location: spirv_instructions_location.clone(),
            spirv_instructions_next_location: spirv_instructions_location,
        }
    }
    fn translate(self) -> Result<TranslatedSPIRVShader<'g>, TranslationError> {
        self.parse()?.translate()
    }
    fn set_spirv_instructions_location(
        &mut self,
        spirv_instructions_location: SPIRVInstructionLocation<'i>,
    ) {
        self.spirv_instructions_current_location = spirv_instructions_location.clone();
        self.spirv_instructions_next_location = spirv_instructions_location;
    }
}

#[derive(Debug)]
pub struct TranslatedSPIRVShader<'g> {
    pub global_state: &'g GlobalState<'g>,
    pub module: Module<'g>,
}

impl<'g> TranslatedSPIRVShader<'g> {
    pub fn new<'i>(
        global_state: &'g GlobalState<'g>,
        target_properties: impl Internable<'g, Interned = TargetProperties>,
        specialization_resolver: &'i mut dyn SpecializationResolver,
        debug_output: &'i mut dyn fmt::Write,
        entry_point_name: &'i str,
        entry_point_execution_model: ExecutionModel,
        spirv_code: &'i [u32],
    ) -> Result<Self, TranslationError> {
        let spirv_parser = spirv_parser::Parser::start(spirv_code)?;
        let spirv_header = *spirv_parser.header();
        let spirv_instructions = spirv_parser.collect::<Result<Vec<_>, _>>()?;
        TranslationStateBase::new(
            global_state,
            target_properties.intern(global_state),
            specialization_resolver,
            debug_output,
            entry_point_name,
            entry_point_execution_model,
            spirv_header,
            &spirv_instructions,
        )
        .translate()
    }
}

#[cfg(test)]
mod tests {
    use core::fmt;
    use shader_compiler_ir::assert_ir_matches_file;
    use spirv_parser::convert_bytes_to_words;

    struct PrintOutput;

    impl fmt::Write for PrintOutput {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            print!("{}", s);
            Ok(())
        }
    }

    #[test]
    fn trivial_test() {
        let spirv_code = convert_bytes_to_words(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/trivial_test.spv"
        )))
        .unwrap();
        let global_state = shader_compiler_ir::GlobalState::new();
        let translated_shader = crate::TranslatedSPIRVShader::new(
            &global_state,
            shader_compiler_ir::TargetProperties::default(),
            &mut crate::DefaultSpecializationResolver,
            &mut PrintOutput,
            "main",
            spirv_parser::ExecutionModelGLCompute.into(),
            &spirv_code,
        )
        .map_err(|e| e.to_string())
        .unwrap();
        assert_ir_matches_file!(
            translated_shader.module,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_data/trivial_test.kazan-ir"
            )
        );
    }

    #[test]
    #[ignore] // FIXME: parsing not completely implemented yet; remove #[ignore] once implemented
    fn simple_test() {
        let spirv_code = convert_bytes_to_words(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/test_data/simple_test.spv"
        )))
        .unwrap();
        let global_state = shader_compiler_ir::GlobalState::new();
        let translated_shader = crate::TranslatedSPIRVShader::new(
            &global_state,
            shader_compiler_ir::TargetProperties::default(),
            &mut crate::DefaultSpecializationResolver,
            &mut PrintOutput,
            "main",
            spirv_parser::ExecutionModelVertex.into(),
            &spirv_code,
        )
        .map_err(|e| e.to_string())
        .unwrap();
        assert_ir_matches_file!(
            translated_shader.module,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_data/simple_test.kazan-ir"
            )
        );
    }
}
