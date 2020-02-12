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
    OpBitcast, OpConvertFToS, OpConvertFToU, OpConvertPtrToU, OpConvertSToF, OpConvertUToF,
    OpConvertUToPtr, OpFConvert, OpGenericCastToPtr, OpGenericCastToPtrExplicit,
    OpPtrCastToGeneric, OpQuantizeToF16, OpSConvert, OpSatConvertSToU, OpSatConvertUToS,
    OpUConvert,
};

impl ParseInstruction for OpConvertFToU {
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
        todo!("unimplemented conversion instruction: OpConvertFToU")
    }
}

impl ParseInstruction for OpUConvert {
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
        todo!("unimplemented conversion instruction: OpUConvert")
    }
}

impl ParseInstruction for OpBitcast {
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
        todo!("unimplemented conversion instruction: OpBitcast")
    }
}

macro_rules! unsupported_conversion_instruction {
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

unsupported_conversion_instruction!(OpGenericCastToPtr);
unsupported_conversion_instruction!(OpGenericCastToPtrExplicit);
unsupported_conversion_instruction!(OpPtrCastToGeneric);
unsupported_conversion_instruction!(OpSatConvertSToU);
unsupported_conversion_instruction!(OpSatConvertUToS);

macro_rules! unimplemented_conversion_instruction {
    ($opname:ident) => {
        impl ParseInstruction for $opname {
            fn parse_in_function_body_prepass<'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                todo!(concat!(
                    "unimplemented conversion instruction: ",
                    stringify!($opname)
                ))
            }
            fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                todo!(concat!(
                    "unimplemented conversion instruction: ",
                    stringify!($opname)
                ))
            }
        }
    };
}

unimplemented_conversion_instruction!(OpConvertFToS);
unimplemented_conversion_instruction!(OpConvertPtrToU);
unimplemented_conversion_instruction!(OpConvertSToF);
unimplemented_conversion_instruction!(OpConvertUToF);
unimplemented_conversion_instruction!(OpConvertUToPtr);
unimplemented_conversion_instruction!(OpFConvert);
unimplemented_conversion_instruction!(OpQuantizeToF16);
unimplemented_conversion_instruction!(OpSConvert);
