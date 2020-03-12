// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    decorations::{
        DecorationAspect, DecorationClass, DecorationClassMisc, DecorationClassStruct,
        MemoryObjectDeclarationOrStructMember, VariableOrStructMember,
    },
    errors::{
        BuiltInAndNonBuiltInNotAllowedInSameStruct, DecorationNotAllowedOnInstruction,
        MemberDecorationIndexOutOfBounds, MemberDecorationNotAllowed, TranslationResult,
    },
    parse::{
        annotations::DecorationsAndMemberDecorations, ParseInstruction,
        TranslationStateParsingTypesConstantsAndGlobals,
    },
    types::structs::{StructKind, StructMember, StructType, StructTypeData},
};
use alloc::vec::Vec;
use core::cell::RefCell;
use hashbrown::HashMap;
use spirv_parser::{DecorationBuiltIn, OpTypeStruct};

impl ParseInstruction for OpTypeStruct {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpTypeStruct {
            id_result,
            ref member_types,
        } = *self;
        let DecorationsAndMemberDecorations {
            decorations,
            member_decorations: member_decorations_in,
        } = state.take_decorations_for_struct_type(id_result)?;
        let mut member_decorations: Vec<Vec<DecorationClass>> =
            member_types.iter().map(|_| Vec::new()).collect();
        for (member_index, member_decorations_in) in member_decorations_in {
            if member_index >= member_types.len() as u32 {
                if let Some(decoration) = member_decorations_in.into_iter().next() {
                    return Err(MemberDecorationIndexOutOfBounds {
                        member_index,
                        decoration: decoration.into(),
                        instruction: self.clone().into(),
                    }
                    .into());
                }
            } else {
                member_decorations[member_index as usize] = member_decorations_in;
            }
        }
        let mut struct_kind = StructKind::Generic;
        for decoration in decorations {
            match decoration {
                DecorationClass::Ignored(_) => {}
                DecorationClass::Struct(DecorationClassStruct::Block(_)) => {
                    struct_kind = StructKind::Block {
                        is_buffer_block: false,
                    };
                }
                DecorationClass::Struct(DecorationClassStruct::BufferBlock(_)) => {
                    struct_kind = StructKind::Block {
                        is_buffer_block: true,
                    };
                }
                DecorationClass::Struct(DecorationClassStruct::GLSLShared(_))
                | DecorationClass::Struct(DecorationClassStruct::GLSLPacked(_)) => {}
                DecorationClass::Misc(_)
                | DecorationClass::MemoryObjectDeclaration(_)
                | DecorationClass::MemoryObjectDeclarationOrStructMember(_)
                | DecorationClass::Object(_)
                | DecorationClass::StructMember(_)
                | DecorationClass::Variable(_)
                | DecorationClass::VariableOrStructMember(_)
                | DecorationClass::Invalid(_)
                | DecorationClass::RelaxedPrecision(_) => {
                    return Err(DecorationNotAllowedOnInstruction {
                        decoration: decoration.into(),
                        instruction: self.clone().into(),
                    }
                    .into());
                }
            }
        }
        let mut is_built_ins = None;
        let mut members = Vec::with_capacity(member_types.len());
        for ((&member_type_id, member_decorations), member_index) in
            member_types.iter().zip(member_decorations).zip(0u32..)
        {
            let member_type = state.get_type(member_type_id)?.clone();
            let mut built_in = None;
            let mut memory_object_declaration_or_struct_member_decorations = Vec::new();
            let mut variable_or_struct_member_decorations = Vec::new();
            for member_decoration in member_decorations {
                match member_decoration {
                    DecorationClass::Ignored(_) => {}
                    DecorationClass::Misc(DecorationClassMisc::BuiltIn(DecorationBuiltIn {
                        built_in: v,
                    })) => built_in = Some(v),
                    DecorationClass::Misc(DecorationClassMisc::ArrayStride(_)) => todo!(),
                    DecorationClass::Misc(DecorationClassMisc::FPRoundingMode(_)) => todo!(),
                    DecorationClass::RelaxedPrecision(_) => todo!(),
                    DecorationClass::MemoryObjectDeclarationOrStructMember(member_decoration) => {
                        memory_object_declaration_or_struct_member_decorations
                            .push(member_decoration);
                    }
                    DecorationClass::Object(_) => todo!(),
                    DecorationClass::VariableOrStructMember(member_decoration) => {
                        variable_or_struct_member_decorations.push(member_decoration);
                    }
                    DecorationClass::StructMember(_) => todo!(),
                    DecorationClass::Misc(DecorationClassMisc::SpecId(_))
                    | DecorationClass::MemoryObjectDeclaration(_)
                    | DecorationClass::Variable(_)
                    | DecorationClass::Struct(_)
                    | DecorationClass::Invalid(_) => {
                        return Err(MemberDecorationNotAllowed {
                            member_index,
                            decoration: member_decoration.into(),
                            instruction: self.clone().into(),
                        }
                        .into());
                    }
                }
            }
            if let Some(is_built_ins) = is_built_ins {
                if is_built_ins != built_in.is_some() {
                    return Err(BuiltInAndNonBuiltInNotAllowedInSameStruct {
                        member_index,
                        instruction: self.clone().into(),
                    }
                    .into());
                }
            }
            is_built_ins = Some(built_in.is_some());
            if built_in.is_some() {
                struct_kind = StructKind::BuiltIns;
            }
            let memory_object_declaration_or_struct_member =
                MemoryObjectDeclarationOrStructMember::parse_decorations(
                    memory_object_declaration_or_struct_member_decorations,
                    Some(member_index),
                    || self.clone().into(),
                )?;
            let variable_or_struct_member = VariableOrStructMember::parse_decorations(
                variable_or_struct_member_decorations,
                Some(member_index),
                || self.clone().into(),
            )?;
            members.push(StructMember {
                built_in,
                member_type,
                member_type_id,
                memory_object_declaration_or_struct_member,
                variable_or_struct_member,
            });
        }
        state.define_type(
            id_result,
            StructType::new(StructTypeData {
                id: id_result.0,
                kind: struct_kind,
                members,
            }),
        )
    }
}
