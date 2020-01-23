// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::errors::TranslationResult;
use crate::parse::annotations::DecorationClass;
use crate::parse::ParseInstruction;
use crate::parse::TranslationStateParsingTypesConstantsAndGlobals;
use crate::TranslationStateBase;
use spirv_parser::OpVariable;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum InterpolationKind {
    Flat,
    NoPerspective,
    Perspective,
}

#[derive(Debug)]
pub(crate) struct MemoryObjectDeclaration {
    pub(crate) interpolation_kind: InterpolationKind,
    // FIXME: finish
}

#[derive(Debug)]
pub(crate) struct SPIRVVariable {}

impl ParseInstruction for OpVariable {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpVariable {
            id_result_type,
            id_result,
            storage_class,
            initializer,
        } = *self;
        let decorations = state.take_decorations(id_result)?;
        let result_type = state.get_type(id_result_type.0)?.clone();
        for decoration in decorations {
            match decoration {
                DecorationClass::Ignored(_) => {}
                DecorationClass::Invalid(_) => todo!(),
                DecorationClass::MemoryObjectDeclaration(_) => todo!(),
                DecorationClass::MemoryObjectDeclarationOrStructMember(_) => todo!(),
                DecorationClass::Misc(_) => todo!(),
                DecorationClass::Object(_) => todo!(),
                DecorationClass::Struct(_) => todo!(),
                DecorationClass::StructMember(_) => todo!(),
                DecorationClass::Variable(_) => todo!(),
                DecorationClass::VariableOrStructMember(_) => todo!(),
            }
        }
        todo!()
    }
    fn parse_in_function_body<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        todo!()
    }
}
