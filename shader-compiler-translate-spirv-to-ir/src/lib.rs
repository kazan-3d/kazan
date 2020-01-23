// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![cfg_attr(not(test), no_std)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

macro_rules! impl_spirv_enum_partition {
    (
        $(#[doc = $class_enum_doc:expr])*
        $vis:vis enum $class_enum:ident($enum:ident) {
            $(
                $(#[doc = $class_enumerant_doc:expr])*
                $class_enumerant:ident($class_value:ident {
                    $(
                        $enumerant:ident($value:ident),
                    )+
                }),
            )+
        }
    ) => {
        $(
            $(#[doc = $class_enumerant_doc])*
            #[derive(Clone, Debug)]
            $vis enum $class_value {
                $(
                    $enumerant(spirv_parser::$value),
                )+
            }

            impl From<$class_value> for $class_enum {
                fn from(v: $class_value) -> Self {
                    $class_enum::$class_enumerant(v)
                }
            }

            impl Into<spirv_parser::$enum> for $class_value {
                fn into(self) -> spirv_parser::$enum {
                    match self {
                        $(
                            $class_value::$enumerant(v) => v.into(),
                        )+
                    }
                }
            }
        )+

        $(#[doc = $class_enum_doc])*
        #[derive(Clone, Debug)]
        $vis enum $class_enum {
            $(
                $(#[doc = $class_enumerant_doc])*
                $class_enumerant($class_value),
            )+
        }

        impl Into<spirv_parser::$enum> for $class_enum {
            fn into(self) -> spirv_parser::$enum {
                match self {
                    $(
                        $class_enum::$class_enumerant(v) => v.into(),
                    )+
                }
            }
        }

        impl From<spirv_parser::$enum> for $class_enum {
            fn from(v: spirv_parser::$enum) -> Self {
                match v {
                    $($(
                        spirv_parser::$enum::$enumerant(v) => $class_enum::$class_enumerant($class_value::$enumerant(v)),
                    )+)+
                }
            }
        }
    };
}

mod errors;
mod parse;
mod types;
mod values;

pub use crate::errors::*;

use alloc::vec::Vec;
use core::fmt;
use core::iter;
use core::slice;
use shader_compiler_ir::GlobalState;
use spirv_parser::ExecutionModel;

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

#[derive(Debug, Clone)]
struct SPIRVInstructionsLocation<'i>(iter::Enumerate<slice::Iter<'i, spirv_parser::Instruction>>);

struct TranslationStateBase<'g, 'i> {
    global_state: &'g GlobalState<'g>,
    specialization_resolver: &'i mut dyn SpecializationResolver,
    debug_output: &'i mut dyn fmt::Write,
    entry_point_name: &'i str,
    entry_point_execution_model: ExecutionModel,
    spirv_header: spirv_parser::Header,
    spirv_instructions: &'i [spirv_parser::Instruction],
    spirv_instructions_location: SPIRVInstructionsLocation<'i>,
}

impl<'g, 'i> TranslationStateBase<'g, 'i> {
    fn new(
        global_state: &'g GlobalState<'g>,
        specialization_resolver: &'i mut dyn SpecializationResolver,
        debug_output: &'i mut dyn fmt::Write,
        entry_point_name: &'i str,
        entry_point_execution_model: ExecutionModel,
        spirv_header: spirv_parser::Header,
        spirv_instructions: &'i [spirv_parser::Instruction],
    ) -> Self {
        Self {
            global_state,
            specialization_resolver,
            debug_output,
            entry_point_name,
            entry_point_execution_model,
            spirv_header,
            spirv_instructions,
            spirv_instructions_location: SPIRVInstructionsLocation(
                spirv_instructions.iter().enumerate(),
            ),
        }
    }
    fn translate(self) -> Result<TranslatedSPIRVShader<'g>, TranslationError> {
        self.parse()?;
        todo!()
    }
}

#[derive(Debug)]
pub struct TranslatedSPIRVShader<'g> {
    pub global_state: &'g GlobalState<'g>,
}

impl<'g> TranslatedSPIRVShader<'g> {
    pub fn new<'i>(
        global_state: &'g GlobalState<'g>,
        specialization_resolver: &'i mut dyn SpecializationResolver,
        debug_output: &'i mut dyn fmt::Write,
        entry_point_name: &'i str,
        entry_point_execution_model: ExecutionModel,
        spirv_code: &'i [u32],
    ) -> Result<Self, TranslationError> {
        let spirv_parser = spirv_parser::Parser::start(spirv_code)?;
        let spirv_header = *spirv_parser.header();
        let spirv_instructions = spirv_parser.collect::<Result<Vec<_>, spirv_parser::Error>>()?;
        TranslationStateBase::new(
            global_state,
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
        let _translated_shader = crate::TranslatedSPIRVShader::new(
            &global_state,
            &mut crate::DefaultSpecializationResolver,
            &mut PrintOutput,
            "main",
            spirv_parser::ExecutionModelVertex.into(),
            spirv_code,
        )
        .map_err(|e| e.to_string())
        .unwrap();
    }
}