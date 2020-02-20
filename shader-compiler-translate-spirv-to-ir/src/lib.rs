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

    struct PrintOutput;

    impl fmt::Write for PrintOutput {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            print!("{}", s);
            Ok(())
        }
    }

    #[test]
    fn trivial_test() {
        let spirv_code = &[
            0x0723_0203, // ..#.
            0x0001_0000, // ....
            0x0008_0007, // ....
            0x0000_0008, // ....
            0x0000_0000, // ....
            0x0002_0011, // ....
            0x0000_0001, // ....
            0x0006_000B, // ....
            0x0000_0002, // ....
            0x4C53_4C47, // GLSL
            0x6474_732E, // .std
            0x3035_342E, // .450
            0x0000_0000, // ....
            0x0003_000E, // ....
            0x0000_0000, // ....
            0x0000_0001, // ....
            0x0005_000F, // ....
            0x0000_0005, // ....
            0x0000_0005, // ....
            0x6E69_616D, // main
            0x0000_0000, // ....
            0x0006_0010, // ....
            0x0000_0005, // ....
            0x0000_0011, // ....
            0x0000_0001, // ....
            0x0000_0001, // ....
            0x0000_0001, // ....
            0x0004_0007, // ....
            0x0000_0001, // ....
            0x6964_7473, // stdi
            0x0000_006E, // n...
            0x002E_0003, // ....
            0x0000_0002, // ....
            0x0000_01C2, // ....
            0x0000_0001, // ....
            0x4F20_2F2F, // // O
            0x646F_4D70, // pMod
            0x5065_6C75, // uleP
            0x6563_6F72, // roce
            0x6465_7373, // ssed
            0x696C_6320, //  cli
            0x2074_6E65, // ent
            0x6B6C_7576, // vulk
            0x3031_6E61, // an10
            0x2F2F_0A30, // 0.//
            0x4D70_4F20, //  OpM
            0x6C75_646F, // odul
            0x6F72_5065, // ePro
            0x7373_6563, // cess
            0x7420_6465, // ed t
            0x6567_7261, // arge
            0x6E65_2D74, // t-en
            0x7576_2076, // v vu
            0x6E61_6B6C, // lkan
            0x0A30_2E31, // 1.0.
            0x4F20_2F2F, // // O
            0x646F_4D70, // pMod
            0x5065_6C75, // uleP
            0x6563_6F72, // roce
            0x6465_7373, // ssed
            0x746E_6520, //  ent
            0x702D_7972, // ry-p
            0x746E_696F, // oint
            0x6961_6D20, //  mai
            0x6C23_0A6E, // n.#l
            0x2065_6E69, // ine
            0x7623_0A31, // 1.#v
            0x6973_7265, // ersi
            0x3420_6E6F, // on 4
            0x0A0A_3035, // 50..
            0x6469_6F76, // void
            0x6961_6D20, //  mai
            0x2029_286E, // n()
            0x2020_0A7B, // {.
            0x6572_2020, //   re
            0x6E72_7574, // turn
            0x007D_0A3B, // ;.}.
            0x0004_0005, // ....
            0x0000_0005, // ....
            0x6E69_616D, // main
            0x0000_0000, // ....
            0x0002_0013, // ....
            0x0000_0003, // ....
            0x0003_0021, // !...
            0x0000_0004, // ....
            0x0000_0003, // ....
            0x0005_0036, // 6...
            0x0000_0003, // ....
            0x0000_0005, // ....
            0x0000_0000, // ....
            0x0000_0004, // ....
            0x0002_00F8, // ....
            0x0000_0006, // ....
            0x0004_0008, // ....
            0x0000_0001, // ....
            0x0000_0004, // ....
            0x0000_0000, // ....
            0x0001_00FD, // ....
            0x0001_0038, // 8...
        ];
        let global_state = shader_compiler_ir::GlobalState::new();
        let translated_shader = crate::TranslatedSPIRVShader::new(
            &global_state,
            shader_compiler_ir::TargetProperties::default(),
            &mut crate::DefaultSpecializationResolver,
            &mut PrintOutput,
            "main",
            spirv_parser::ExecutionModelGLCompute.into(),
            spirv_code,
        )
        .map_err(|e| e.to_string())
        .unwrap();
        let expected_text = concat!(
            "module {\n",
            "    target_properties {\n",
            "        data_pointer_underlying_type: i64,\n",
            "        function_pointer_underlying_type: i64,\n",
            "    }\n",
            "    built_in_inputs_block {\n",
            "        -> built_in_inputs_block : data_ptr;\n",
            "        size: fixed 0x0;\n",
            "        align: 0x1;\n",
            "    }\n",
            "    user_inputs_block {\n",
            "        -> user_inputs_block : data_ptr;\n",
            "        size: fixed 0x0;\n",
            "        align: 0x1;\n",
            "    }\n",
            "    built_in_outputs_block {\n",
            "        -> built_in_outputs_block : data_ptr;\n",
            "        size: fixed 0x0;\n",
            "        align: 0x1;\n",
            "    }\n",
            "    user_outputs_block {\n",
            "        -> user_outputs_block : data_ptr;\n",
            "        size: fixed 0x0;\n",
            "        align: 0x1;\n",
            "    }\n",
            "    invocation_global_variables {\n",
            "    }\n",
            "    fn main[] -> [] {\n",
            "        hints {\n",
            "            inlining_hint: none,\n",
            "            side_effects: normal,\n",
            "        }\n",
            "        {\n",
            "        }\n",
            "        id_6 {\n",
            "            break id_6[] @ \"stdin\":4:0;\n",
            "        }\n",
            "    }\n",
            "    entry_point: main;\n",
            "}",
        );
        let text = translated_shader.module.to_string();
        println!("translated module:\n{}", text);
        assert_eq!(expected_text, text);
    }

    #[test]
    #[ignore] // FIXME: parsing not completely implemented yet; remove #[ignore] once implemented
    fn simple_test() {
        let spirv_code = &[
            0x0723_0203,
            0x0001_0000,
            0x0008_0001,
            0x0000_002C,
            0x0000_0000,
            0x0002_0011,
            0x0000_0001,
            0x0002_0011,
            0x0000_000B,
            0x0006_000B,
            0x0000_0001,
            0x4C53_4C47,
            0x6474_732E,
            0x3035_342E,
            0x0000_0000,
            0x0003_000E,
            0x0000_0000,
            0x0000_0001,
            0x0007_000F,
            0x0000_0000,
            0x0000_0004,
            0x6E69_616D,
            0x0000_0000,
            0x0000_000A,
            0x0000_000F,
            0x0005_0048,
            0x0000_0008,
            0x0000_0000,
            0x0000_000B,
            0x0000_0000,
            0x0003_0047,
            0x0000_0008,
            0x0000_0002,
            0x0004_0047,
            0x0000_000F,
            0x0000_001E,
            0x0000_0000,
            0x0002_0013,
            0x0000_0002,
            0x0003_0021,
            0x0000_0003,
            0x0000_0002,
            0x0003_0016,
            0x0000_0006,
            0x0000_0020,
            0x0004_0017,
            0x0000_0007,
            0x0000_0006,
            0x0000_0004,
            0x0003_001E,
            0x0000_0008,
            0x0000_0007,
            0x0004_0020,
            0x0000_0009,
            0x0000_0003,
            0x0000_0008,
            0x0004_003B,
            0x0000_0009,
            0x0000_000A,
            0x0000_0003,
            0x0004_0015,
            0x0000_000B,
            0x0000_0020,
            0x0000_0001,
            0x0004_002B,
            0x0000_000B,
            0x0000_000C,
            0x0000_0000,
            0x0004_0017,
            0x0000_000D,
            0x0000_0006,
            0x0000_0003,
            0x0004_0020,
            0x0000_000E,
            0x0000_0001,
            0x0000_000D,
            0x0004_003B,
            0x0000_000E,
            0x0000_000F,
            0x0000_0001,
            0x0004_002B,
            0x0000_0006,
            0x0000_0011,
            0x3F80_0000,
            0x0004_0020,
            0x0000_0016,
            0x0000_0003,
            0x0000_0007,
            0x0004_0020,
            0x0000_0018,
            0x0000_0007,
            0x0000_000B,
            0x0004_0015,
            0x0000_001A,
            0x0000_0020,
            0x0000_0000,
            0x0004_002B,
            0x0000_001A,
            0x0000_001B,
            0x0000_0002,
            0x0004_0020,
            0x0000_001C,
            0x0000_0001,
            0x0000_0006,
            0x0004_0015,
            0x0000_001F,
            0x0000_0040,
            0x0000_0000,
            0x0004_002B,
            0x0000_0006,
            0x0000_0028,
            0x0000_0000,
            0x0004_002B,
            0x0000_001A,
            0x0000_0029,
            0x0000_0000,
            0x0004_0020,
            0x0000_002A,
            0x0000_0003,
            0x0000_0006,
            0x0005_0036,
            0x0000_0002,
            0x0000_0004,
            0x0000_0000,
            0x0000_0003,
            0x0002_00F8,
            0x0000_0005,
            0x0004_003B,
            0x0000_0018,
            0x0000_0019,
            0x0000_0007,
            0x0004_003D,
            0x0000_000D,
            0x0000_0010,
            0x0000_000F,
            0x0005_0051,
            0x0000_0006,
            0x0000_0012,
            0x0000_0010,
            0x0000_0000,
            0x0005_0051,
            0x0000_0006,
            0x0000_0013,
            0x0000_0010,
            0x0000_0001,
            0x0005_0051,
            0x0000_0006,
            0x0000_0014,
            0x0000_0010,
            0x0000_0002,
            0x0007_0050,
            0x0000_0007,
            0x0000_0015,
            0x0000_0012,
            0x0000_0013,
            0x0000_0014,
            0x0000_0011,
            0x0005_0041,
            0x0000_0016,
            0x0000_0017,
            0x0000_000A,
            0x0000_000C,
            0x0003_003E,
            0x0000_0017,
            0x0000_0015,
            0x0005_0041,
            0x0000_001C,
            0x0000_001D,
            0x0000_000F,
            0x0000_001B,
            0x0004_003D,
            0x0000_0006,
            0x0000_001E,
            0x0000_001D,
            0x0004_006D,
            0x0000_001F,
            0x0000_0020,
            0x0000_001E,
            0x0004_0071,
            0x0000_001A,
            0x0000_0021,
            0x0000_0020,
            0x0004_007C,
            0x0000_000B,
            0x0000_0022,
            0x0000_0021,
            0x0003_003E,
            0x0000_0019,
            0x0000_0022,
            0x0004_003D,
            0x0000_000B,
            0x0000_0023,
            0x0000_0019,
            0x0003_00F7,
            0x0000_0026,
            0x0000_0000,
            0x000C_00FB,
            0x0000_0020,
            0x0000_0025,
            0x0000_0001,
            0x0000_0000,
            0x0000_0024,
            0x0000_0002,
            0x0000_0000,
            0x0000_0024,
            0x0000_0008,
            0x0000_0000,
            0x0000_0024,
            0x0002_00F8,
            0x0000_0025,
            0x0006_0041,
            0x0000_002A,
            0x0000_002B,
            0x0000_000A,
            0x0000_000C,
            0x0000_0029,
            0x0003_003E,
            0x0000_002B,
            0x0000_0028,
            0x0002_00F9,
            0x0000_0026,
            0x0002_00F8,
            0x0000_0024,
            0x0002_00F9,
            0x0000_0026,
            0x0002_00F8,
            0x0000_0026,
            0x0001_00FD,
            0x0001_0038,
        ];
        let global_state = shader_compiler_ir::GlobalState::new();
        let translated_shader = crate::TranslatedSPIRVShader::new(
            &global_state,
            shader_compiler_ir::TargetProperties::default(),
            &mut crate::DefaultSpecializationResolver,
            &mut PrintOutput,
            "main",
            spirv_parser::ExecutionModelVertex.into(),
            spirv_code,
        )
        .map_err(|e| e.to_string())
        .unwrap();
        let expected_text = concat!("");
        let text = translated_shader.module.to_string();
        println!("translated module:\n{}", text);
        assert_eq!(expected_text, text);
    }
}
