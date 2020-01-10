// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![cfg_attr(not(test), no_std)]

#[macro_use]
extern crate alloc;

use core::fmt;
use shader_compiler_ir::prelude::*;

#[derive(Debug)]
pub struct SpecializationResolutionFailed;

impl fmt::Display for SpecializationResolutionFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad("shader specialization failed")
    }
}

impl From<SpecializationResolutionFailed> for TranslationError {
    fn from(v: SpecializationResolutionFailed) -> TranslationError {
        TranslationError::SpecializationResolutionFailed(v)
    }
}

#[derive(Debug)]
pub enum TranslationError {
    SpecializationResolutionFailed(SpecializationResolutionFailed),
    SPIRVParserError(spirv_parser::Error),
}

impl From<spirv_parser::Error> for TranslationError {
    fn from(v: spirv_parser::Error) -> TranslationError {
        TranslationError::SPIRVParserError(v)
    }
}

impl fmt::Display for TranslationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SpecializationResolutionFailed(v) => v.fmt(f),
            Self::SPIRVParserError(v) => v.fmt(f),
        }
    }
}

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

struct TranslationState<'g, 'i> {
    global_state: &'g GlobalState<'g>,
    specialization_resolver: &'i mut (dyn SpecializationResolver + 'i),
    spirv_code: &'i [u32],
    entry_point_name: &'i str,
    spirv_parser: spirv_parser::Parser<'i>,
}

impl<'g, 'i> TranslationState<'g, 'i> {
    fn new(
        global_state: &'g GlobalState<'g>,
        specialization_resolver: &'i mut (dyn SpecializationResolver + 'i),
        entry_point_name: &'i str,
        spirv_code: &'i [u32],
    ) -> Result<Self, TranslationError> {
        Ok(Self {
            global_state,
            specialization_resolver,
            spirv_code,
            entry_point_name,
            spirv_parser: spirv_parser::Parser::start(spirv_code)?,
        })
    }
    fn translate(&mut self) -> Result<TranslatedSPIRVShader<'g>, TranslationError> {
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
        specialization_resolver: &'i mut (dyn SpecializationResolver + 'i),
        entry_point_name: &'i str,
        spirv_code: &'i [u32],
    ) -> Result<Self, TranslationError> {
        TranslationState::new(
            global_state,
            specialization_resolver,
            entry_point_name,
            spirv_code,
        )?
        .translate()
    }
}
