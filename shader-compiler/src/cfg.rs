// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use spirv_parser::{IdRef, IdResult, Instruction};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter;
use std::mem;
use std::ops;
use std::rc::{Rc, Weak};

#[derive(Clone, Debug)]
pub(crate) struct BasicBlock {
    instructions: Vec<Instruction>,
    immediate_dominator: Option<IdRef>,
    predecessors: Vec<IdRef>,
    dominator_tree_children: Vec<IdRef>,
}

#[derive(Copy, Clone, Debug)]
enum BasicBlockSuccessorsState<'a> {
    None,
    ReturnOrKill,
    Single(IdRef),
    Two(IdRef, IdRef),
    Switch32WithoutDefault(&'a [(u32, IdRef)]),
    Switch32(IdRef, &'a [(u32, IdRef)]),
    Switch64WithoutDefault(&'a [(u64, IdRef)]),
    Switch64(IdRef, &'a [(u64, IdRef)]),
}

#[derive(Clone, Debug)]
pub(crate) struct BasicBlockSuccessors<'a>(BasicBlockSuccessorsState<'a>);

impl<'a> BasicBlockSuccessors<'a> {
    pub(crate) fn is_return_or_kill(&self) -> bool {
        match self.0 {
            BasicBlockSuccessorsState::ReturnOrKill => true,
            _ => false,
        }
    }
}

impl<'a> Iterator for BasicBlockSuccessors<'a> {
    type Item = IdRef;
    fn next(&mut self) -> Option<IdRef> {
        match self.0 {
            BasicBlockSuccessorsState::None => None,
            BasicBlockSuccessorsState::ReturnOrKill => None,
            BasicBlockSuccessorsState::Single(v) => {
                self.0 = BasicBlockSuccessorsState::None;
                Some(v)
            }
            BasicBlockSuccessorsState::Two(v1, v2) => {
                self.0 = BasicBlockSuccessorsState::Single(v2);
                Some(v1)
            }
            BasicBlockSuccessorsState::Switch32WithoutDefault(v) => {
                let (first, rest) = v.split_first()?;
                self.0 = BasicBlockSuccessorsState::Switch32WithoutDefault(rest);
                Some(first.1)
            }
            BasicBlockSuccessorsState::Switch32(v1, v2) => {
                self.0 = BasicBlockSuccessorsState::Switch32WithoutDefault(v2);
                Some(v1)
            }
            BasicBlockSuccessorsState::Switch64WithoutDefault(v) => {
                let (first, rest) = v.split_first()?;
                self.0 = BasicBlockSuccessorsState::Switch64WithoutDefault(rest);
                Some(first.1)
            }
            BasicBlockSuccessorsState::Switch64(v1, v2) => {
                self.0 = BasicBlockSuccessorsState::Switch64WithoutDefault(v2);
                Some(v1)
            }
        }
    }
}

fn get_terminating_instruction_targets(instruction: &Instruction) -> Option<BasicBlockSuccessors> {
    match *instruction {
        Instruction::Branch { target_label, .. } => Some(BasicBlockSuccessors(
            BasicBlockSuccessorsState::Single(target_label),
        )),
        Instruction::BranchConditional {
            true_label,
            false_label,
            ..
        } => Some(BasicBlockSuccessors(BasicBlockSuccessorsState::Two(
            true_label,
            false_label,
        ))),
        Instruction::Switch32 {
            default,
            ref target,
            ..
        } => Some(BasicBlockSuccessors(BasicBlockSuccessorsState::Switch32(
            default, target,
        ))),
        Instruction::Switch64 {
            default,
            ref target,
            ..
        } => Some(BasicBlockSuccessors(BasicBlockSuccessorsState::Switch64(
            default, target,
        ))),
        Instruction::Kill => Some(BasicBlockSuccessors(
            BasicBlockSuccessorsState::ReturnOrKill,
        )),
        Instruction::Return => Some(BasicBlockSuccessors(
            BasicBlockSuccessorsState::ReturnOrKill,
        )),
        Instruction::ReturnValue { .. } => Some(BasicBlockSuccessors(
            BasicBlockSuccessorsState::ReturnOrKill,
        )),
        Instruction::Unreachable => Some(BasicBlockSuccessors(BasicBlockSuccessorsState::None)),
        _ => None,
    }
}

impl BasicBlock {
    pub(crate) fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }
    pub(crate) fn terminating_instruction(&self) -> &Instruction {
        self.instructions.last().unwrap()
    }
    pub(crate) fn successors(&self) -> BasicBlockSuccessors {
        get_terminating_instruction_targets(self.terminating_instruction()).unwrap()
    }
    pub(crate) fn predecessors(&self) -> &[IdRef] {
        &self.predecessors
    }
    pub(crate) fn immediate_dominator(&self) -> Option<IdRef> {
        self.immediate_dominator
    }
    pub(crate) fn dominator_tree_children(&self) -> &[IdRef] {
        &self.dominator_tree_children
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CFG {
    basic_blocks: HashMap<IdRef, BasicBlock>,
    entry_block: IdRef,
}

impl CFG {
    pub(crate) fn new(function_instructions: &[Instruction]) -> CFG {
        // find blocks
        let mut current_label = None;
        let mut current_instructions = Vec::new();
        let mut basic_blocks = HashMap::new();
        let mut entry_block = None;
        let mut basic_block_labels = Vec::new();
        for instruction in function_instructions {
            if current_label.is_none() {
                match instruction {
                    Instruction::NoLine | Instruction::Line { .. } => {}
                    Instruction::Label { id_result } => {
                        current_label = Some(id_result.0);
                        entry_block = entry_block.or(current_label);
                    }
                    _ => unreachable!("invalid instruction before OpLabel"),
                }
                current_instructions.push(instruction.clone());
            } else {
                current_instructions.push(instruction.clone());
                if get_terminating_instruction_targets(instruction).is_some() {
                    let current_label = current_label.take().unwrap();
                    basic_block_labels.push(current_label);
                    basic_blocks.insert(
                        current_label,
                        BasicBlock {
                            instructions: current_instructions,
                            immediate_dominator: None,
                            predecessors: Vec::new(),
                            dominator_tree_children: Vec::new(),
                        },
                    );
                    current_instructions = Vec::new();
                }
            }
        }
        // compute predecessors
        assert!(current_instructions.is_empty());
        let entry_block = entry_block.expect("function has no basic blocks");
        let mut successors = Vec::new();
        for label in &basic_block_labels {
            successors.extend(basic_blocks[label].successors());
            for successor in &successors {
                basic_blocks
                    .get_mut(successor)
                    .expect("missing basic block")
                    .predecessors
                    .push(*label);
            }
            successors.clear();
        }
        // compute dominators
        let mut basic_blocks_strict_dominators: HashMap<_, _> = basic_block_labels
            .iter()
            .scan(
                basic_block_labels
                    .iter()
                    .map(|v| *v)
                    .collect::<HashSet<_>>(),
                |full_set, label| {
                    Some((
                        *label,
                        Some(if *label == entry_block {
                            HashSet::new()
                        } else {
                            let mut retval = full_set.clone();
                            retval.remove(label);
                            retval
                        }),
                    ))
                },
            )
            .collect();
        loop {
            let mut any_changes = false;
            for &label in &basic_block_labels {
                let mut new_set = basic_blocks_strict_dominators[&label].clone().unwrap();
                for &predecessor in basic_blocks[&label].predecessors() {
                    let predecessor_strict_dominators = basic_blocks_strict_dominators
                        [&predecessor]
                        .as_ref()
                        .unwrap();
                    new_set.retain(|&item| {
                        item == predecessor || predecessor_strict_dominators.contains(&item)
                    });
                }
                let mut target_set = basic_blocks_strict_dominators.get_mut(&label).unwrap();
                if !any_changes {
                    any_changes = *target_set.as_ref().unwrap() != new_set;
                }
                *target_set = Some(new_set);
            }
            if !any_changes {
                break;
            }
        }
        let mut basic_blocks_immediate_dominators = basic_blocks_strict_dominators;
        // compute immediate dominators
        let mut temp_vec = Vec::new();
        for &n in &basic_block_labels {
            let mut immediate_dominators = basic_blocks_immediate_dominators
                .get_mut(&n)
                .unwrap()
                .take()
                .unwrap();
            temp_vec.clear();
            temp_vec.extend(immediate_dominators.iter().map(|v| *v));
            for &s in &temp_vec {
                immediate_dominators.retain(|&t| {
                    if t == s {
                        return true;
                    }
                    !basic_blocks_immediate_dominators[&s]
                        .as_ref()
                        .unwrap()
                        .contains(&t)
                });
            }
            *basic_blocks_immediate_dominators.get_mut(&n).unwrap() = Some(immediate_dominators);
        }
        // fill in BasicBlock fields
        for &i in &basic_block_labels {
            let mut immediate_dominator_iter = basic_blocks_immediate_dominators[&i]
                .as_ref()
                .unwrap()
                .iter();
            let immediate_dominator = immediate_dominator_iter.next().map(|v| *v);
            assert!(immediate_dominator.is_none() || immediate_dominator_iter.next().is_none());
            basic_blocks.get_mut(&i).unwrap().immediate_dominator = immediate_dominator;
            if let Some(immediate_dominator) = immediate_dominator {
                basic_blocks
                    .get_mut(&immediate_dominator)
                    .unwrap()
                    .dominator_tree_children
                    .push(i);
            }
        }
        CFG {
            basic_blocks,
            entry_block,
        }
    }
    pub(crate) fn get_basic_block(&self, label_id: IdRef) -> Option<&BasicBlock> {
        self.basic_blocks.get(&label_id)
    }
}

impl ops::Index<IdRef> for CFG {
    type Output = BasicBlock;
    fn index(&self, label_id: IdRef) -> &BasicBlock {
        &self.basic_blocks[&label_id]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IdFactory(u32);

    impl IdFactory {
        fn new() -> IdFactory {
            IdFactory(1)
        }
        fn next(&mut self) -> IdRef {
            let retval = IdRef(self.0);
            self.0 += 1;
            retval
        }
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    enum SerializedCFGElement {
        Simple,
        Return,
        Discard,
        Switch,
        SwitchCase,
        SwitchDefaultCase,
        SwitchEnd,
        SwitchFallthrough,
        SwitchMerge,
        Condition,
        ConditionTrue,
        ConditionFalse,
        ConditionEnd,
        ConditionMerge,
    }

    fn test_cfg(instructions: &[Instruction], expected: &[SerializedCFGElement]) {
        println!("instructions:");
        for instruction in instructions {
            print!("{}", instruction);
        }
        println!();
        let cfg = CFG::new(&instructions);
        println!("{:#?}", cfg);
        unimplemented!();
    }

    #[test]
    fn test_cfg_return() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label1 = id_factory.next();
        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label1),
        });
        instructions.push(Instruction::Return);

        test_cfg(&instructions, &[SerializedCFGElement::Return]);
    }

    #[test]
    fn test_cfg_return_value() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label1 = id_factory.next();
        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label1),
        });
        instructions.push(Instruction::ReturnValue {
            value: id_factory.next(),
        });

        test_cfg(&instructions, &[SerializedCFGElement::Return]);
    }

    #[test]
    fn test_cfg_simple_discard() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label1 = id_factory.next();
        let label2 = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label1),
        });
        instructions.push(Instruction::Branch {
            target_label: label2,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label2),
        });
        instructions.push(Instruction::Kill);

        test_cfg(
            &instructions,
            &[SerializedCFGElement::Simple, SerializedCFGElement::Discard],
        );
    }

    #[test]
    fn test_cfg_conditional_none_none() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_endif = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_endif,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::BranchConditional {
            condition: id_factory.next(),
            true_label: label_endif,
            false_label: label_endif,
            branch_weights: vec![],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_endif),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Condition,
                SerializedCFGElement::ConditionEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_conditional_merge_none() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_then = id_factory.next();
        let label_endif = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_endif,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::BranchConditional {
            condition: id_factory.next(),
            true_label: label_then,
            false_label: label_endif,
            branch_weights: vec![],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_then),
        });
        instructions.push(Instruction::Branch {
            target_label: label_endif,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_endif),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Condition,
                SerializedCFGElement::ConditionTrue,
                SerializedCFGElement::ConditionMerge,
                SerializedCFGElement::ConditionEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_conditional_return_merge() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_then = id_factory.next();
        let label_else = id_factory.next();
        let label_endif = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_endif,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::BranchConditional {
            condition: id_factory.next(),
            true_label: label_then,
            false_label: label_else,
            branch_weights: vec![],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_then),
        });
        instructions.push(Instruction::Return);

        instructions.push(Instruction::Label {
            id_result: IdResult(label_else),
        });
        instructions.push(Instruction::Branch {
            target_label: label_endif,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_endif),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Condition,
                SerializedCFGElement::ConditionTrue,
                SerializedCFGElement::Return,
                SerializedCFGElement::ConditionFalse,
                SerializedCFGElement::ConditionMerge,
                SerializedCFGElement::ConditionEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_switch_default_break() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_default = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_merge,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::Switch64 {
            selector: id_factory.next(),
            default: label_default,
            target: vec![],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_default),
        });
        instructions.push(Instruction::Branch {
            target_label: label_merge,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Switch,
                SerializedCFGElement::SwitchDefaultCase,
                SerializedCFGElement::SwitchMerge,
                SerializedCFGElement::SwitchEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_switch_return_default_break() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_case1 = id_factory.next();
        let label_default = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_merge,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::Switch64 {
            selector: id_factory.next(),
            default: label_default,
            target: vec![(0, label_case1)],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_case1),
        });
        instructions.push(Instruction::Return);

        instructions.push(Instruction::Label {
            id_result: IdResult(label_default),
        });
        instructions.push(Instruction::Branch {
            target_label: label_merge,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Switch,
                SerializedCFGElement::SwitchCase,
                SerializedCFGElement::Return,
                SerializedCFGElement::SwitchDefaultCase,
                SerializedCFGElement::SwitchMerge,
                SerializedCFGElement::SwitchEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_switch_fallthrough_default_break() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_case1 = id_factory.next();
        let label_default = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_merge,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::Switch64 {
            selector: id_factory.next(),
            default: label_default,
            target: vec![(0, label_case1)],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_case1),
        });
        instructions.push(Instruction::Branch {
            target_label: label_default,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_default),
        });
        instructions.push(Instruction::Branch {
            target_label: label_merge,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Switch,
                SerializedCFGElement::SwitchCase,
                SerializedCFGElement::SwitchFallthrough,
                SerializedCFGElement::SwitchDefaultCase,
                SerializedCFGElement::SwitchMerge,
                SerializedCFGElement::SwitchEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_switch_fallthrough_default_fallthrough_break() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_case1 = id_factory.next();
        let label_default = id_factory.next();
        let label_case2 = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_merge,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::Switch64 {
            selector: id_factory.next(),
            default: label_default,
            target: vec![(0, label_case1), (1, label_case1), (2, label_case2)],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_case1),
        });
        instructions.push(Instruction::Branch {
            target_label: label_default,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_default),
        });
        instructions.push(Instruction::Branch {
            target_label: label_case2,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_case2),
        });
        instructions.push(Instruction::Branch {
            target_label: label_merge,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Switch,
                SerializedCFGElement::SwitchCase,
                SerializedCFGElement::SwitchFallthrough,
                SerializedCFGElement::SwitchDefaultCase,
                SerializedCFGElement::SwitchFallthrough,
                SerializedCFGElement::SwitchCase,
                SerializedCFGElement::SwitchMerge,
                SerializedCFGElement::SwitchEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_switch_break_default_fallthrough_break() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_case1 = id_factory.next();
        let label_default = id_factory.next();
        let label_case2 = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_merge,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::Switch32 {
            selector: id_factory.next(),
            default: label_default,
            target: vec![(0, label_case1), (1, label_case1), (2, label_case2)],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_case1),
        });
        instructions.push(Instruction::Branch {
            target_label: label_merge,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_default),
        });
        instructions.push(Instruction::Branch {
            target_label: label_case2,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_case2),
        });
        instructions.push(Instruction::Branch {
            target_label: label_merge,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Switch,
                SerializedCFGElement::SwitchCase,
                SerializedCFGElement::SwitchMerge,
                SerializedCFGElement::SwitchDefaultCase,
                SerializedCFGElement::SwitchFallthrough,
                SerializedCFGElement::SwitchCase,
                SerializedCFGElement::SwitchMerge,
                SerializedCFGElement::SwitchEnd,
                SerializedCFGElement::Return,
            ],
        );
    }
}
