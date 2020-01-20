// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::errors::BuiltInAndNonBuiltInNotAllowedInSameStruct;
use crate::errors::DecorationNotAllowedOnInstruction;
use crate::errors::MemberDecorationIndexOutOfBounds;
use crate::errors::MemberDecorationNotAllowed;
use crate::errors::TranslationResult;
use crate::parse::annotations::DecorationsAndMemberDecorations;
use crate::parse::ParseInstruction;
use crate::parse::TranslationStateParsingTypesConstantsAndGlobals;
use crate::types::structs::StructKind;
use crate::types::structs::StructMember;
use crate::types::structs::StructType;
use crate::types::structs::StructTypeData;
use alloc::vec::Vec;
use spirv_parser::Decoration;
use spirv_parser::OpTypeStruct;

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
        let mut member_decorations: Vec<Vec<Decoration>> =
            member_types.iter().map(|_| Vec::new()).collect();
        for (member_index, member_decorations_in) in member_decorations_in {
            if member_index >= member_types.len() as u32 {
                for decoration in member_decorations_in {
                    return Err(MemberDecorationIndexOutOfBounds {
                        member_index,
                        decoration,
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
                Decoration::Block => {
                    struct_kind = StructKind::Block {
                        is_buffer_block: false,
                    };
                }
                Decoration::BufferBlock => {
                    struct_kind = StructKind::Block {
                        is_buffer_block: true,
                    };
                }
                // TODO: finish
                _ => {
                    return Err(DecorationNotAllowedOnInstruction {
                        decoration,
                        instruction: self.clone().into(),
                    }
                    .into());
                }
            }
        }
        let mut is_built_ins = None;
        let mut members = Vec::with_capacity(member_types.len());
        for (member_index, (&member_type, member_decorations)) in
            member_types.iter().zip(member_decorations).enumerate()
        {
            let member_type = state.get_type(member_type)?.clone();
            let mut built_in = None;
            for member_decoration in member_decorations {
                match member_decoration {
                    Decoration::BuiltIn { built_in: v } => built_in = Some(v),
                    // TODO: finish
                    _ => {
                        return Err(MemberDecorationNotAllowed {
                            member_index: member_index as u32,
                            decoration: member_decoration,
                            instruction: self.clone().into(),
                        }
                        .into());
                    }
                }
            }
            if let Some(is_built_ins) = is_built_ins {
                if is_built_ins != built_in.is_some() {
                    return Err(BuiltInAndNonBuiltInNotAllowedInSameStruct {
                        member_index: member_index as u32,
                        instruction: self.clone().into(),
                    }
                    .into());
                }
            }
            is_built_ins = Some(built_in.is_some());
            if built_in.is_some() {
                struct_kind = StructKind::BuiltIns;
            }
            members.push(StructMember {
                built_in,
                member_type,
            });
        }
        state.define_type(
            id_result,
            StructType::new(StructTypeData {
                kind: struct_kind,
                members,
            }),
        )
    }
}
