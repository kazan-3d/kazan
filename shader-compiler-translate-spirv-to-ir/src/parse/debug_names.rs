// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    parse::{debug_strings_sources::TranslationStateParsedDebugStringsSources, ParseInstruction},
    TranslationResult,
};
use hashbrown::HashMap;
use shader_compiler_ir::{Internable, Interned};
use spirv_id_map::IdMap;
use spirv_parser::{IdRef, Instruction, OpMemberName, OpName};

decl_translation_state! {
    pub(crate) struct TranslationStateParsedDebugNames<'g, 'i> {
        base: TranslationStateParsedDebugStringsSources<'g, 'i>,
        debug_names: IdMap<IdRef, Interned<'g, str>>,
        debug_member_names: IdMap<IdRef, HashMap<u32, Interned<'g, str>>>,
    }
}

impl<'g, 'i> TranslationStateParsedDebugNames<'g, 'i> {
    pub(crate) fn get_debug_name(&self, id: IdRef) -> TranslationResult<Option<Interned<'g, str>>> {
        Ok(self.debug_names.get(id)?.copied())
    }
    pub(crate) fn get_or_make_debug_name(&self, id: IdRef) -> TranslationResult<Interned<'g, str>> {
        if let Some(retval) = self.get_debug_name(id)? {
            Ok(retval)
        } else {
            Ok(format!("id_{}", id.0).intern(self.global_state))
        }
    }
    fn parse_name_instruction(&mut self, instruction: &'i OpName) -> TranslationResult<()> {
        let OpName { target, ref name } = *instruction;
        let name = name.intern(self.global_state);
        self.debug_names.insert(target, name)?;
        Ok(())
    }
    fn parse_member_name_instruction(
        &mut self,
        instruction: &'i OpMemberName,
    ) -> TranslationResult<()> {
        let OpMemberName {
            type_,
            member,
            ref name,
        } = *instruction;
        let name = name.intern(self.global_state);
        self.debug_member_names
            .entry(type_)?
            .or_insert_default()
            .insert(member, name);
        Ok(())
    }
}

impl<'g, 'i> TranslationStateParsedDebugStringsSources<'g, 'i> {
    pub(crate) fn parse_debug_names_section(
        self,
    ) -> TranslationResult<TranslationStateParsedDebugNames<'g, 'i>> {
        let mut state = TranslationStateParsedDebugNames {
            debug_names: IdMap::new(&self.spirv_header),
            debug_member_names: IdMap::new(&self.spirv_header),
            base: self,
        };
        writeln!(state.debug_output, "parsing debug names section")?;
        while let Some((instruction, location)) = state.get_instruction_and_location()? {
            match instruction {
                Instruction::Name(instruction) => state.parse_name_instruction(instruction)?,
                Instruction::MemberName(instruction) => {
                    state.parse_member_name_instruction(instruction)?
                }
                _ => {
                    state.spirv_instructions_location = location;
                    break;
                }
            }
        }
        Ok(state)
    }
}

impl ParseInstruction for OpName {}
impl ParseInstruction for OpMemberName {}
