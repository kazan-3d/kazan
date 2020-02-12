// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    cfg::CFGBlockId,
    decorations::{DecorationAspect, DecorationClass, SPIRVObject},
    errors::{DecorationNotAllowedOnInstruction, TranslationResult, UnsupportedSPIRVInstruction},
    parse::{
        functions::TranslationStateParsingFunctionBody,
        translate_structure_tree::TranslationStateParsingFunctionBodyBlock, ParseInstruction,
    },
    types::GenericSPIRVType,
    values::{GenericSPIRVValue, SimpleValue},
};
use alloc::vec::Vec;
use shader_compiler_ir::{ValueDefinition, ValueUse};
use spirv_parser::{
    MemoryAccess, OpAccessChain, OpArrayLength, OpCopyMemory, OpCopyMemorySized,
    OpGenericPtrMemSemantics, OpInBoundsAccessChain, OpInBoundsPtrAccessChain, OpLoad,
    OpPtrAccessChain, OpPtrDiff, OpPtrEqual, OpPtrNotEqual, OpStore,
};

impl ParseInstruction for OpLoad {
    fn parse_in_function_body_prepass<'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Ok(())
    }
    fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        let Self {
            id_result_type,
            id_result,
            pointer,
            ref memory_access,
        } = *self;
        let mut object_decorations = Vec::new();
        for decoration in state.take_decorations(id_result)? {
            match decoration {
                DecorationClass::Ignored(_) => {}
                DecorationClass::Invalid(_)
                | DecorationClass::MemoryObjectDeclaration(_)
                | DecorationClass::MemoryObjectDeclarationOrStructMember(_)
                | DecorationClass::Misc(_)
                | DecorationClass::Struct(_)
                | DecorationClass::StructMember(_)
                | DecorationClass::Variable(_)
                | DecorationClass::VariableOrStructMember(_) => {
                    return Err(DecorationNotAllowedOnInstruction {
                        decoration: decoration.into(),
                        instruction: self.clone().into(),
                    }
                    .into());
                }
                DecorationClass::Object(decoration) => object_decorations.push(decoration),
                DecorationClass::RelaxedPrecision(_) => todo!(),
            }
        }
        let object =
            SPIRVObject::parse_decorations(object_decorations, None, || self.clone().into())?;
        let result_type = state.get_type(id_result_type.0)?.clone();
        let alignment = result_type.get_alignment(
            state.target_properties,
            state.global_state,
            id_result_type.0,
            || self.clone().into(),
        )?;
        let pointer_value = state.get_value(pointer)?.clone();
        if let Some(MemoryAccess {
            volatile,
            aligned,
            nontemporal,
            make_pointer_available,
            make_pointer_visible,
            non_private_pointer,
        }) = memory_access
        {
            todo!()
        }
        let result_name = state.get_or_make_debug_name(id_result.0)?;
        let result_value = ValueDefinition::new(
            result_type.get_nonvoid_ir_type(state.global_state, id_result_type.0, || {
                self.clone().into()
            })?,
            result_name,
            state.global_state,
        );
        let ir_value = ValueUse::new(result_value.value());
        state.push_instruction_with_current_location(shader_compiler_ir::instructions::Load {
            arguments: [pointer_value.get_ir_value(state.global_state)?],
            results: [result_value],
            alignment,
        })?;
        todo!("unimplemented memory instruction: OpLoad");
        state.define_value(
            id_result,
            SimpleValue {
                ir_value,
                result_type,
                object,
            },
        )
    }
}

impl ParseInstruction for OpStore {
    fn parse_in_function_body_prepass<'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Ok(())
    }
    fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        todo!("unimplemented memory instruction: OpStore")
    }
}

impl ParseInstruction for OpAccessChain {
    fn parse_in_function_body_prepass<'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Ok(())
    }
    fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        todo!("unimplemented memory instruction: OpAccessChain")
    }
}

macro_rules! unsupported_memory_instruction {
    ($opname:ident) => {
        impl ParseInstruction for $opname {
            fn parse_in_function_body_prepass<'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                Err(UnsupportedSPIRVInstruction {
                    instruction: self.clone().into(),
                }
                .into())
            }
            fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                Err(UnsupportedSPIRVInstruction {
                    instruction: self.clone().into(),
                }
                .into())
            }
        }
    };
}

unsupported_memory_instruction!(OpCopyMemorySized);
unsupported_memory_instruction!(OpGenericPtrMemSemantics);

macro_rules! unimplemented_memory_instruction {
    ($opname:ident) => {
        impl ParseInstruction for $opname {
            fn parse_in_function_body_prepass<'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                todo!(concat!(
                    "unimplemented memory instruction: ",
                    stringify!($opname)
                ))
            }
            fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                todo!(concat!(
                    "unimplemented memory instruction: ",
                    stringify!($opname)
                ))
            }
        }
    };
}

unimplemented_memory_instruction!(OpArrayLength);
unimplemented_memory_instruction!(OpCopyMemory);
unimplemented_memory_instruction!(OpInBoundsAccessChain);
unimplemented_memory_instruction!(OpInBoundsPtrAccessChain);
unimplemented_memory_instruction!(OpPtrAccessChain);
unimplemented_memory_instruction!(OpPtrDiff);
unimplemented_memory_instruction!(OpPtrEqual);
unimplemented_memory_instruction!(OpPtrNotEqual);
