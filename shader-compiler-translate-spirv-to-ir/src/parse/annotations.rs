// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    decorations::DecorationClass,
    errors::{
        MemberDecorationsAreOnlyAllowedOnStructTypes, SPIRVIdAlreadyDefined, SPIRVIdNotDefined,
    },
    parse::{debug_module_processed::TranslationStateParsedDebugModuleProcessed, ParseInstruction},
    TranslationResult,
};
use alloc::vec::Vec;
use hashbrown::HashMap;
use spirv_id_map::{Entry::Vacant, IdMap};
use spirv_parser::{
    Decoration, IdRef, IdResult, Instruction, OpDecorate, OpDecorateId, OpDecorateString,
    OpDecorationGroup, OpGroupDecorate, OpGroupMemberDecorate, OpMemberDecorate,
    OpMemberDecorateString,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct DecorationsAndMemberDecorations {
    pub(crate) decorations: Vec<DecorationClass>,
    pub(crate) member_decorations: HashMap<u32, Vec<DecorationClass>>,
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsedAnnotations<'g, 'i> {
        base: TranslationStateParsedDebugModuleProcessed<'g, 'i>,
        decorations: IdMap<IdRef, DecorationsAndMemberDecorations>,
    }
}

impl<'g, 'i> TranslationStateParsedAnnotations<'g, 'i> {
    /// only for Ids that aren't struct types
    pub(crate) fn take_decorations(
        &mut self,
        target: IdResult,
    ) -> TranslationResult<Vec<DecorationClass>> {
        let DecorationsAndMemberDecorations {
            decorations,
            member_decorations,
        } = self.decorations.remove(target.0)?.unwrap_or_default();
        for (_, member_decorations) in member_decorations {
            if !member_decorations.is_empty() {
                return Err(
                    MemberDecorationsAreOnlyAllowedOnStructTypes { target: target.0 }.into(),
                );
            }
        }
        Ok(decorations)
    }
    pub(crate) fn take_decorations_for_struct_type(
        &mut self,
        target: IdResult,
    ) -> TranslationResult<DecorationsAndMemberDecorations> {
        Ok(self.decorations.remove(target.0)?.unwrap_or_default())
    }
}

decl_translation_state! {
    struct TranslationStateParsingAnnotations<'g, 'i> {
        base: TranslationStateParsedAnnotations<'g, 'i>,
        decoration_groups: IdMap<IdRef, Vec<Decoration>>,
    }
}

impl<'g, 'i> TranslationStateParsingAnnotations<'g, 'i> {
    fn parse_decorate_instruction(
        &mut self,
        target: IdRef,
        decoration: &Decoration,
    ) -> TranslationResult<()> {
        self.decorations
            .entry(target)?
            .or_insert_default()
            .decorations
            .push(decoration.clone().into());
        Ok(())
    }
    fn parse_member_decorate_instruction(
        &mut self,
        structure_type: IdRef,
        member: u32,
        decoration: &Decoration,
    ) -> TranslationResult<()> {
        self.decorations
            .entry(structure_type)?
            .or_insert_default()
            .member_decorations
            .entry(member)
            .or_default()
            .push(decoration.clone().into());
        Ok(())
    }
    fn parse_decoration_group_instruction(
        &mut self,
        instruction: &'i OpDecorationGroup,
    ) -> TranslationResult<()> {
        let OpDecorationGroup { id_result } = *instruction;
        let decorations = self.take_decorations(id_result)?;
        if let Vacant(entry) = self.decoration_groups.entry(id_result.0)? {
            entry.insert(decorations.into_iter().map(Into::into).collect());
            Ok(())
        } else {
            Err(SPIRVIdAlreadyDefined { id_result }.into())
        }
    }
    fn parse_group_decorate_instruction(
        &mut self,
        instruction: &'i OpGroupDecorate,
    ) -> TranslationResult<()> {
        let OpGroupDecorate {
            decoration_group,
            ref targets,
        } = *instruction;
        let decorations = self
            .decoration_groups
            .get(decoration_group)?
            .ok_or(SPIRVIdNotDefined {
                id: decoration_group,
            })?
            .clone();
        for &target in targets {
            for decoration in &decorations {
                self.parse_decorate_instruction(target, decoration)?;
            }
        }
        Ok(())
    }
    fn parse_group_member_decorate_instruction(
        &mut self,
        instruction: &'i OpGroupMemberDecorate,
    ) -> TranslationResult<()> {
        let OpGroupMemberDecorate {
            decoration_group,
            ref targets,
        } = *instruction;
        let decorations = self
            .decoration_groups
            .get(decoration_group)?
            .ok_or(SPIRVIdNotDefined {
                id: decoration_group,
            })?
            .clone();
        for &(target, member) in targets {
            for decoration in &decorations {
                self.parse_member_decorate_instruction(target, member, decoration)?;
            }
        }
        Ok(())
    }
}

impl<'g, 'i> TranslationStateParsedDebugModuleProcessed<'g, 'i> {
    pub(crate) fn parse_annotations_section(
        self,
    ) -> TranslationResult<TranslationStateParsedAnnotations<'g, 'i>> {
        let mut state = TranslationStateParsingAnnotations {
            decoration_groups: IdMap::new(&self.spirv_header),
            base: TranslationStateParsedAnnotations {
                decorations: IdMap::new(&self.spirv_header),
                base: self,
            },
        };
        writeln!(state.debug_output, "parsing annotations section")?;
        while let Some((instruction, location)) = state.next_instruction_and_location()? {
            match *instruction {
                Instruction::Decorate(OpDecorate {
                    target,
                    ref decoration,
                })
                | Instruction::DecorateId(OpDecorateId {
                    target,
                    ref decoration,
                })
                | Instruction::DecorateString(OpDecorateString {
                    target,
                    ref decoration,
                }) => state.parse_decorate_instruction(target, decoration)?,
                Instruction::DecorationGroup(ref instruction) => {
                    state.parse_decoration_group_instruction(instruction)?
                }
                Instruction::GroupDecorate(ref instruction) => {
                    state.parse_group_decorate_instruction(instruction)?
                }
                Instruction::GroupMemberDecorate(ref instruction) => {
                    state.parse_group_member_decorate_instruction(instruction)?
                }
                Instruction::MemberDecorate(OpMemberDecorate {
                    structure_type,
                    member,
                    ref decoration,
                })
                | Instruction::MemberDecorateString(OpMemberDecorateString {
                    struct_type: structure_type,
                    member,
                    ref decoration,
                }) => {
                    state.parse_member_decorate_instruction(structure_type, member, decoration)?
                }
                _ => {
                    state.set_spirv_instructions_location(location);
                    break;
                }
            }
        }
        Ok(state.base)
    }
}

impl ParseInstruction for OpDecorate {}
impl ParseInstruction for OpDecorateId {}
impl ParseInstruction for OpDecorateString {}
impl ParseInstruction for OpDecorationGroup {}
impl ParseInstruction for OpGroupDecorate {}
impl ParseInstruction for OpGroupMemberDecorate {}
impl ParseInstruction for OpMemberDecorate {}
impl ParseInstruction for OpMemberDecorateString {}
