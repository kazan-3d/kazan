// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    cfg::{CFGBlockId, TerminationInstruction},
    decorations::DecorationClass,
    errors::{DecorationNotAllowedOnInstruction, TranslationResult},
    parse::{functions::TranslationStateParsingFunctionBody, ParseInstruction},
    structure_tree::{Child, Node, NodeKind, StructureTree},
    SPIRVInstructionLocation,
};
use alloc::{collections::VecDeque, vec::Vec};
use shader_compiler_ir::{Block, BlockRef, Inhabitable, ValueDefinition};
use spirv_parser::{
    OpBranch, OpBranchConditional, OpKill, OpLabel, OpLoopMerge, OpPhi, OpReturn, OpReturnValue,
    OpSelectionMerge, OpSwitch32, OpSwitch64, OpUnreachable,
};

decl_translation_state! {
    pub(crate) struct TranslationStateTranslatingStructureTree<'f, 'g, 'i> {
        base: TranslationStateParsingFunctionBody<'f, 'g, 'i>,
    }
}

decl_translation_state! {
    pub(crate) struct TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i> {
        base: &'b mut TranslationStateTranslatingStructureTree<'f, 'g, 'i>,
        block_instructions: &'b mut Vec<shader_compiler_ir::Instruction<'g>>,
    }
}

impl<'b, 'f, 'g, 'i> TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i> {
    pub(crate) fn push_instruction(
        &mut self,
        location: SPIRVInstructionLocation<'i>,
        instruction: impl Into<shader_compiler_ir::InstructionData<'g>>,
    ) -> TranslationResult<()> {
        let location = self.get_debug_location(location)?;
        self.block_instructions
            .push(shader_compiler_ir::Instruction::new(
                location,
                instruction.into(),
            ));
        Ok(())
    }
    pub(crate) fn push_instruction_with_current_location(
        &mut self,
        instruction: impl Into<shader_compiler_ir::InstructionData<'g>>,
    ) -> TranslationResult<()> {
        self.push_instruction(
            self.spirv_instructions_current_location.clone(),
            instruction,
        )
    }
    fn create_block_without_body(&mut self, block_id: CFGBlockId) -> TranslationResult<Block<'g>> {
        let label_id = self.function.cfg[block_id].label_id();
        let name = self.get_or_make_debug_name(label_id)?;
        // FIXME: calculate actual result definitions
        let result_definitions = Inhabitable::Uninhabited;
        Ok(Block::without_body(
            name,
            result_definitions,
            self.global_state,
        ))
    }
    fn translate_structure_tree_basic_block(
        &mut self,
        block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        let label_location = self.function.cfg[block_id].label_location();
        self.set_spirv_instructions_location(label_location);
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
    fn translate_structure_tree_case_node(
        &mut self,
        switch_block: CFGBlockId,
        cases: &[Child],
        mut case_blocks: VecDeque<(CFGBlockId, BlockRef<'g>)>,
    ) -> TranslationResult<()> {
        match cases.split_last() {
            Some((Child::Node(case), cases)) => {
                let block = self.create_block_without_body(case.first_basic_block())?;
                let block_ref = BlockRef::new(block.value());
                case_blocks.push_front((case.first_basic_block(), block_ref));
                self.push_instruction(
                    self.function.cfg[switch_block].termination_location(),
                    block,
                )?;
                let mut block_instructions = Vec::new();
                TranslationStateParsingFunctionBodyBlock {
                    base: &mut *self.base,
                    block_instructions: &mut block_instructions,
                }
                .translate_structure_tree_case_node(
                    switch_block,
                    cases,
                    case_blocks,
                )?;
                todo!();
                block_ref.set_body(block_instructions);
                Ok(())
            }
            None => {
                let variable = todo!();
                let targets: Vec<shader_compiler_ir::instructions::BranchTarget> = todo!();
                let default_target: shader_compiler_ir::instructions::BreakBlock = todo!();
                if !targets.is_empty() {
                    self.push_instruction(
                        self.function.cfg[switch_block].termination_location(),
                        shader_compiler_ir::instructions::Branch { variable, targets },
                    )?;
                }
                self.push_instruction(
                    self.function.cfg[switch_block].termination_location(),
                    default_target,
                )
            }
            Some((Child::BasicBlock(_), _)) => unreachable!(),
        }
    }
    fn translate_structure_tree_switch_node(&mut self, node: &Node) -> TranslationResult<()> {
        let switch_block = node.children()[0]
            .basic_block()
            .expect("known to be a BasicBlock");
        self.translate_structure_tree_basic_block(switch_block)?;
        self.translate_structure_tree_case_node(
            switch_block,
            &node.children()[1..],
            VecDeque::with_capacity(node.children().len() - 1),
        )?;
        todo!()
    }
    fn translate_structure_tree_simple_node_body(&mut self, node: &Node) -> TranslationResult<()> {
        for child in node.children() {
            match child {
                Child::BasicBlock(block) => self.translate_structure_tree_basic_block(*block)?,
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
}

impl<'f, 'g, 'i> TranslationStateTranslatingStructureTree<'f, 'g, 'i> {
    fn translate_structure_tree_root(&mut self) -> TranslationResult<()> {
        let function = self.function;
        let mut block_instructions = Vec::new();
        TranslationStateParsingFunctionBodyBlock {
            base: self,
            block_instructions: &mut block_instructions,
        }
        .translate_structure_tree_simple_node_body(function.cfg.structure_tree().root())?;
        function.ir_value.body.set_body(block_instructions);
        Ok(())
    }
}

impl<'f, 'g, 'i> TranslationStateParsingFunctionBody<'f, 'g, 'i> {
    pub(crate) fn translate_structure_tree(self) -> TranslationResult<Self> {
        let mut state = TranslationStateTranslatingStructureTree { base: self };
        let ir_value = state.function.ir_value;
        writeln!(
            state.debug_output,
            "function body: translate structure tree: {:?}",
            ir_value.name
        )?;
        state.translate_structure_tree_root()?;
        let TranslationStateTranslatingStructureTree { base } = state;
        Ok(base)
    }
}

impl ParseInstruction for OpLabel {
    fn parse_in_function_body_prepass<'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBody<'_, '_, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Ok(())
    }
    fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        let Self { id_result } = *self;
        for decoration in state.take_decorations(id_result)? {
            match decoration {
                DecorationClass::Ignored(_) => {}
                DecorationClass::Invalid(_)
                | DecorationClass::MemoryObjectDeclaration(_)
                | DecorationClass::MemoryObjectDeclarationOrStructMember(_)
                | DecorationClass::Misc(_)
                | DecorationClass::Object(_)
                | DecorationClass::RelaxedPrecision(_)
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
            }
        }
        Ok(())
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
            fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
                &'i self,
                _state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
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
unimplemented_control_flow_instruction!(OpLoopMerge);
unimplemented_control_flow_instruction!(OpPhi);
unimplemented_control_flow_instruction!(OpReturn);
unimplemented_control_flow_instruction!(OpReturnValue);
unimplemented_control_flow_instruction!(OpSelectionMerge);
unimplemented_control_flow_instruction!(OpSwitch32);
unimplemented_control_flow_instruction!(OpSwitch64);
unimplemented_control_flow_instruction!(OpUnreachable);
