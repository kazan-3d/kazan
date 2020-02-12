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
    OpCompositeConstruct, OpCompositeExtract, OpCompositeInsert, OpCopyLogical, OpCopyObject,
    OpTranspose, OpVectorExtractDynamic, OpVectorInsertDynamic, OpVectorShuffle,
};

impl ParseInstruction for OpCompositeExtract {
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
        todo!("unimplemented composite instruction: OpCompositeExtract")
    }
}

impl ParseInstruction for OpCompositeConstruct {
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
        todo!("unimplemented composite instruction: OpCompositeConstruct")
    }
}

macro_rules! unimplemented_composite_instruction {
    ($opname:ident) => {
        impl ParseInstruction for $opname {
            fn parse_in_function_body_prepass<'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                todo!(concat!(
                    "unimplemented composite instruction: ",
                    stringify!($opname)
                ))
            }
            fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                todo!(concat!(
                    "unimplemented composite instruction: ",
                    stringify!($opname)
                ))
            }
        }
    };
}

unimplemented_composite_instruction!(OpCompositeInsert);
unimplemented_composite_instruction!(OpCopyLogical);
unimplemented_composite_instruction!(OpCopyObject);
unimplemented_composite_instruction!(OpTranspose);
unimplemented_composite_instruction!(OpVectorExtractDynamic);
unimplemented_composite_instruction!(OpVectorInsertDynamic);
unimplemented_composite_instruction!(OpVectorShuffle);
