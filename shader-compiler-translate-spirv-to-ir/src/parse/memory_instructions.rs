// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    cfg::CFGBlockId,
    errors::{TranslationResult, UnsupportedSPIRVInstruction},
    parse::{
        functions::TranslationStateParsingFunctionBody,
        translate_structure_tree::TranslationStateParsingFunctionBodyBlock, ParseInstruction,
    },
};
use spirv_parser::{
    OpAccessChain, OpArrayLength, OpCopyMemory, OpCopyMemorySized, OpGenericPtrMemSemantics,
    OpInBoundsAccessChain, OpInBoundsPtrAccessChain, OpLoad, OpPtrAccessChain, OpPtrDiff,
    OpPtrEqual, OpPtrNotEqual, OpStore,
};

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

unimplemented_memory_instruction!(OpAccessChain);
unimplemented_memory_instruction!(OpArrayLength);
unimplemented_memory_instruction!(OpCopyMemory);
unimplemented_memory_instruction!(OpInBoundsAccessChain);
unimplemented_memory_instruction!(OpInBoundsPtrAccessChain);
unimplemented_memory_instruction!(OpLoad);
unimplemented_memory_instruction!(OpPtrAccessChain);
unimplemented_memory_instruction!(OpPtrDiff);
unimplemented_memory_instruction!(OpPtrEqual);
unimplemented_memory_instruction!(OpPtrNotEqual);
unimplemented_memory_instruction!(OpStore);
