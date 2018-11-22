// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use spirv_parser::{IdRef, IdResult, Instruction};
use std::collections::HashMap;
use std::fmt;
use std::rc::{Rc, Weak};

pub(crate) trait GenericNode: Clone + fmt::Debug {
    fn instructions(&self) -> &Vec<Instruction>;
    fn to_node(self) -> Node;
    fn label(&self) -> IdRef;
}

#[derive(Clone, Debug)]
pub(crate) struct SimpleNode {
    label: IdRef,
    instructions: Vec<Instruction>,
    next: Rc<Node>,
}

impl GenericNode for SimpleNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(self) -> Node {
        Node::Simple(self)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SwitchNode {
    label: IdRef,
    instructions: Vec<Instruction>,
    cases: Vec<Rc<Node>>,
    next: Option<Rc<Node>>,
}

impl GenericNode for SwitchNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(self) -> Node {
        Node::Switch(self)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SwitchFallthroughNode {
    label: IdRef,
    instructions: Vec<Instruction>,
    switch: Weak<SwitchNode>,
    next: Rc<Node>,
}

impl GenericNode for SwitchFallthroughNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(self) -> Node {
        Node::SwitchFallthrough(self)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SwitchBreakNode {
    label: IdRef,
    instructions: Vec<Instruction>,
    switch: Weak<SwitchNode>,
    next: Rc<Node>,
}

impl GenericNode for SwitchBreakNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(self) -> Node {
        Node::SwitchBreak(self)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ReturnNode {
    label: IdRef,
    instructions: Vec<Instruction>,
}

impl GenericNode for ReturnNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(self) -> Node {
        Node::Return(self)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DiscardNode {
    label: IdRef,
    instructions: Vec<Instruction>,
}

impl GenericNode for DiscardNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(self) -> Node {
        Node::Discard(self)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Node {
    Simple(SimpleNode),
    Return(ReturnNode),
    Discard(DiscardNode),
    Switch(SwitchNode),
    SwitchFallthrough(SwitchFallthroughNode),
    SwitchBreak(SwitchBreakNode),
}

impl<T: GenericNode> From<T> for Node {
    fn from(v: T) -> Node {
        v.to_node()
    }
}

struct BasicBlock<'a> {
    label_id: IdRef,
    label_line_instructions: &'a [Instruction],
    instructions: &'a [Instruction],
}

impl<'a> fmt::Debug for BasicBlock<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BasicBlock:\n")?;
        for instruction in self.label_line_instructions {
            write!(f, "{}", instruction)?;
        }
        write!(
            f,
            "{}",
            Instruction::Label {
                id_result: IdResult(self.label_id)
            }
        )?;
        for instruction in self.instructions {
            write!(f, "{}", instruction)?;
        }
        Ok(())
    }
}

struct ParseState<'a> {
    basic_blocks: HashMap<IdRef, BasicBlock<'a>>,
}

impl<'a> ParseState<'a> {
    fn parse(&mut self, label_id: IdRef) -> Rc<Node> {
        let basic_block = self
            .basic_blocks
            .get(&label_id)
            .unwrap_or_else(|| unreachable!("label not found: {}", label_id));
        let (terminating_instruction, instructions_without_terminator) = basic_block
            .instructions
            .split_last()
            .expect("missing terminating instruction");
        let control_header_instruction = instructions_without_terminator.last();
        match (terminating_instruction, control_header_instruction) {
            (
                Instruction::Branch { target_label },
                Some(Instruction::LoopMerge {
                    merge_block,
                    continue_target,
                    ..
                }),
            ) => unimplemented!(),
            (Instruction::Branch { target_label }, _) => unimplemented!(),
            (
                Instruction::BranchConditional {
                    true_label,
                    false_label,
                    ..
                },
                Some(Instruction::LoopMerge {
                    merge_block,
                    continue_target,
                    ..
                }),
            ) => unimplemented!(),
            (
                Instruction::BranchConditional {
                    true_label,
                    false_label,
                    ..
                },
                Some(Instruction::SelectionMerge { merge_block, .. }),
            ) => unimplemented!(),
            (Instruction::BranchConditional { .. }, _) => unreachable!("missing merge instruction"),
            (
                Instruction::Switch32 {
                    default,
                    target: targets,
                    ..
                },
                Some(Instruction::SelectionMerge { merge_block, .. }),
            ) => unimplemented!(),
            (
                Instruction::Switch64 {
                    default,
                    target: targets,
                    ..
                },
                Some(Instruction::SelectionMerge { merge_block, .. }),
            ) => unimplemented!(),
            (Instruction::Switch32 { .. }, _) => unreachable!("missing merge instruction"),
            (Instruction::Switch64 { .. }, _) => unreachable!("missing merge instruction"),
            (Instruction::Kill {}, _) => unimplemented!(),
            (Instruction::Return {}, _) => unimplemented!(),
            (Instruction::ReturnValue { .. }, _) => unimplemented!(),
            (Instruction::Unreachable {}, _) => unimplemented!(),
            _ => unreachable!(
                "invalid basic block terminating instruction:\n{}",
                terminating_instruction
            ),
        }
    }
}

pub(crate) fn create_cfg(mut input_instructions: &[Instruction]) -> Rc<Node> {
    let mut basic_blocks = HashMap::new();
    let mut first_block = None;
    'split_into_blocks: while !input_instructions.is_empty() {
        let (label_id, label_line_instructions) = 'find_label: loop {
            for (i, instruction) in input_instructions.iter().enumerate() {
                match instruction {
                    Instruction::Label { id_result } => {
                        break 'find_label (id_result.0, &input_instructions[..i]);
                    }
                    Instruction::NoLine {} | Instruction::Line { .. } => {}
                    _ => break,
                }
            }
            unreachable!("missing OpLabel")
        };
        if first_block.is_none() {
            first_block = Some(label_id);
        }
        for i in 0..input_instructions.len() {
            match &input_instructions[i] {
                Instruction::Branch { .. }
                | Instruction::BranchConditional { .. }
                | Instruction::Switch32 { .. }
                | Instruction::Switch64 { .. }
                | Instruction::Kill { .. }
                | Instruction::Return { .. }
                | Instruction::ReturnValue { .. }
                | Instruction::Unreachable { .. } => {
                    let (instructions, rest) = input_instructions.split_at(i + 1);
                    input_instructions = rest;
                    let previous = basic_blocks.insert(
                        label_id,
                        BasicBlock {
                            label_line_instructions,
                            label_id,
                            instructions,
                        },
                    );
                    assert!(previous.is_none(), "duplicate OpLabel: {}", label_id);
                    continue 'split_into_blocks;
                }
                _ => {}
            }
        }
        unreachable!("missing terminating instruction");
    }
    let first_block = first_block.expect("missing OpLabel");
    ParseState { basic_blocks }.parse(first_block)
}
