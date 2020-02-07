// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::cfg::CFGBlockId;
use crate::{
    cfg::TerminationInstruction,
    errors::TranslationResult,
    parse::{functions::TranslationStateParsingFunctionBody, ParseInstruction},
    structure_tree::{Child, Node, NodeKind, StructureTree},
};
use shader_compiler_ir::BlockRef;
use spirv_parser::{
    OpBranch, OpBranchConditional, OpKill, OpLabel, OpLoopMerge, OpPhi, OpReturn, OpReturnValue,
    OpSelectionMerge, OpSwitch32, OpSwitch64, OpUnreachable,
};

decl_translation_state! {
    pub(crate) struct TranslationStateTranslatingStructureTree<'f, 'g, 'i> {
        base: TranslationStateParsingFunctionBody<'f, 'g, 'i>,
    }
}

impl<'f, 'g, 'i> TranslationStateTranslatingStructureTree<'f, 'g, 'i> {
    fn translate_structure_tree_simple_basic_block(
        &mut self,
        block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        self.spirv_instructions_location = self.function.cfg[block_id].label_location();
        loop {
            let instruction = self
                .next_instruction()?
                .expect("missing termination instruction");
            instruction.parse_in_function_body_reachable(self, block_id)?;
            if TerminationInstruction::is_in_subset(instruction) {
                break;
            }
        }
        Ok(())
    }
    fn translate_structure_tree_if_node(&mut self, node: &Node) -> TranslationResult<()> {
        todo!()
    }
    fn translate_structure_tree_loop_node(&mut self, node: &Node) -> TranslationResult<()> {
        todo!()
    }
    fn translate_structure_tree_switch_node(&mut self, node: &Node) -> TranslationResult<()> {
        todo!()
    }
    fn translate_structure_tree_simple_node_body(&mut self, node: &Node) -> TranslationResult<()> {
        for child in node.children() {
            match child {
                Child::BasicBlock(block) => {
                    self.translate_structure_tree_simple_basic_block(*block)?
                }
                Child::Node(node) => match node.kind() {
                    NodeKind::Root
                    | NodeKind::Case
                    | NodeKind::Continue
                    | NodeKind::IfPart
                    | NodeKind::LoopBody => unreachable!(),
                    NodeKind::If => self.translate_structure_tree_if_node(node)?,
                    NodeKind::Loop => self.translate_structure_tree_loop_node(node)?,
                    NodeKind::Switch => self.translate_structure_tree_switch_node(node)?,
                },
            }
        }
        todo!()
    }
    fn translate_structure_tree_root(&mut self) -> TranslationResult<()> {
        self.translate_structure_tree_simple_node_body(self.function.cfg.structure_tree().root())
    }
}

impl<'f, 'g, 'i> TranslationStateParsingFunctionBody<'f, 'g, 'i> {
    pub(crate) fn translate_structure_tree(self) -> TranslationResult<Self> {
        let mut state = TranslationStateTranslatingStructureTree { base: self };
        state.translate_structure_tree_root()?;
        let TranslationStateTranslatingStructureTree { base } = state;
        Ok(base)
    }
}

macro_rules! unimplemented_control_flow_instruction {
    ($opname:ident) => {
        impl ParseInstruction for $opname {
            fn parse_in_function_body_prepass<'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBody<'_, '_, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                Ok(())
            }
            fn parse_in_function_body_reachable<'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateTranslatingStructureTree<'f, 'g, 'i>,
                _block_id: CFGBlockId,
            ) -> TranslationResult<()> {
                todo!(concat!(
                    "unimplemented control flow instruction: ",
                    stringify!($opname)
                ))
            }
        }
    };
}

unimplemented_control_flow_instruction!(OpBranch);
unimplemented_control_flow_instruction!(OpBranchConditional);
unimplemented_control_flow_instruction!(OpKill);
unimplemented_control_flow_instruction!(OpLabel);
unimplemented_control_flow_instruction!(OpLoopMerge);
unimplemented_control_flow_instruction!(OpPhi);
unimplemented_control_flow_instruction!(OpReturn);
unimplemented_control_flow_instruction!(OpReturnValue);
unimplemented_control_flow_instruction!(OpSelectionMerge);
unimplemented_control_flow_instruction!(OpSwitch32);
unimplemented_control_flow_instruction!(OpSwitch64);
unimplemented_control_flow_instruction!(OpUnreachable);
