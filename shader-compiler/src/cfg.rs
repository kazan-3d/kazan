// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use instruction_properties::{InstructionClass, InstructionProperties};
use petgraph::{algo::dominators, graph::IndexType, prelude::*};
use spirv_parser::{IdRef, Instruction};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops;

#[derive(Clone)]
struct Instructions(Vec<Instruction>);

impl ops::Deref for Instructions {
    type Target = [Instruction];
    fn deref(&self) -> &[Instruction] {
        &self.0
    }
}

impl fmt::Debug for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in &self.0 {
            write!(f, "{}", i)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct BasicBlock {
    label: IdRef,
    instructions: Instructions,
}

impl BasicBlock {
    pub fn label(&self) -> IdRef {
        self.label
    }
    pub fn instructions(&self) -> &[Instruction] {
        &*self.instructions
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(transparent)]
pub struct CFGIndexType(u32);

unsafe impl IndexType for CFGIndexType {
    fn new(v: usize) -> Self {
        CFGIndexType(v as _)
    }
    fn index(&self) -> usize {
        self.0 as _
    }
    fn max() -> Self {
        CFGIndexType(u32::max_value())
    }
}

pub type CFGNodeIndex = NodeIndex<CFGIndexType>;
pub type CFGEdgeIndex = EdgeIndex<CFGIndexType>;

pub type CFGGraph = DiGraph<BasicBlock, (), CFGIndexType>;

pub type CFGDominators = dominators::Dominators<CFGNodeIndex>;

#[derive(Clone, Debug)]
pub struct CFG(CFGGraph);

impl ops::Deref for CFG {
    type Target = CFGGraph;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CFG {
    pub fn entry_node_index(&self) -> CFGNodeIndex {
        self.0.node_indices().next().unwrap()
    }
    pub fn entry_block(&self) -> &BasicBlock {
        &self[self.entry_node_index()]
    }
    pub fn dominators(&self) -> CFGDominators {
        dominators::simple_fast(&**self, self.entry_node_index())
    }
    pub fn new(function_instructions: &[Instruction]) -> CFG {
        assert!(!function_instructions.is_empty());
        let mut retval = CFG(CFGGraph::default());
        let mut current_label = None;
        let mut current_instructions = Vec::new();
        let mut id_to_index_map = HashMap::new();
        for instruction in function_instructions {
            if current_label.is_none() {
                if let Instruction::Label { id_result } = instruction {
                    current_label = Some(id_result.0);
                } else {
                    assert_eq!(
                        InstructionProperties::new(instruction).class(),
                        InstructionClass::DebugLine,
                        "invalid instruction before OpLabel"
                    );
                }
                current_instructions.push(instruction.clone());
            } else {
                current_instructions.push(instruction.clone());
                if InstructionProperties::new(instruction).class()
                    == InstructionClass::BlockTerminator
                {
                    let label = current_label.take().unwrap();
                    id_to_index_map.insert(
                        label,
                        retval.0.add_node(BasicBlock {
                            label,
                            instructions: Instructions(current_instructions),
                        }),
                    );
                    current_instructions = Vec::new();
                }
            }
        }
        assert!(current_instructions.is_empty());
        for &node_index in id_to_index_map.values() {
            let mut successors: Vec<_> = InstructionProperties::new(
                retval
                    .0
                    .node_weight(node_index)
                    .unwrap()
                    .instructions()
                    .last()
                    .unwrap(),
            )
            .targets()
            .unwrap()
            .collect();
            successors.sort_unstable_by_key(|v| v.0);
            successors.dedup();
            for target in successors {
                retval.0.add_edge(node_index, id_to_index_map[&target], ());
            }
        }
        retval
    }
}

#[derive(Clone, Debug)]
pub struct Loop {
    header: CFGNodeIndex,
    backedge_source_nodes: Vec<CFGNodeIndex>,
    nodes: HashSet<CFGNodeIndex>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(transparent)]
pub struct LoopGraphIndexType(u32);

unsafe impl IndexType for LoopGraphIndexType {
    fn new(v: usize) -> Self {
        LoopGraphIndexType(v as _)
    }
    fn index(&self) -> usize {
        self.0 as _
    }
    fn max() -> Self {
        LoopGraphIndexType(u32::max_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::visit::IntoNodeReferences;
    use spirv_parser::{IdResult, LoopControl};

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

    #[derive(Debug)]
    struct CFGTemplateBlock<'a> {
        label: IdRef,
        successors: &'a [IdRef],
        immediate_dominator: Option<IdRef>,
    }

    #[derive(Debug)]
    struct CFGTemplate<'a> {
        blocks: &'a [CFGTemplateBlock<'a>],
        entry_block: IdRef,
    }

    fn test_cfg(instructions: &[Instruction], cfg_template: &CFGTemplate) {
        println!("instructions:");
        for instruction in instructions {
            print!("{}", instruction);
        }
        println!();
        let cfg = CFG::new(&instructions);
        println!("{:#?}", cfg);
        let dominators = cfg.dominators();
        println!("{:#?}", dominators);
        assert_eq!(cfg_template.entry_block, cfg.entry_block().label());
        assert_eq!(cfg_template.blocks.len(), cfg.node_count());
        let label_map: HashMap<_, _> = cfg
            .node_references()
            .map(|(node_index, basic_block)| (basic_block.label(), node_index))
            .collect();
        for &CFGTemplateBlock {
            label,
            successors: expected_successors,
            immediate_dominator,
        } in cfg_template.blocks
        {
            let expected_successors: HashSet<_> = expected_successors.iter().cloned().collect();
            let node_index = label_map[&label];
            let actual_successors: HashSet<_> = cfg
                .neighbors(node_index)
                .map(|node_index| cfg[node_index].label())
                .collect();
            assert_eq!(expected_successors, actual_successors);
            assert_eq!(
                dominators
                    .immediate_dominator(node_index)
                    .map(|v| cfg[v].label()),
                immediate_dominator
            );
        }
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

        test_cfg(
            &instructions,
            &CFGTemplate {
                blocks: &[CFGTemplateBlock {
                    label: label1,
                    successors: &[],
                    immediate_dominator: None,
                }],
                entry_block: label1,
            },
        );
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

        test_cfg(
            &instructions,
            &CFGTemplate {
                blocks: &[CFGTemplateBlock {
                    label: label1,
                    successors: &[],
                    immediate_dominator: None,
                }],
                entry_block: label1,
            },
        );
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
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label1,
                        successors: &[label2],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label2,
                        successors: &[],
                        immediate_dominator: Some(label1),
                    },
                ],
                entry_block: label1,
            },
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
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_endif],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_endif,
                        successors: &[],
                        immediate_dominator: Some(label_start),
                    },
                ],
                entry_block: label_start,
            },
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
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_then, label_endif],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_then,
                        successors: &[label_endif],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_endif,
                        successors: &[],
                        immediate_dominator: Some(label_start),
                    },
                ],
                entry_block: label_start,
            },
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
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_then, label_else],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_then,
                        successors: &[],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_else,
                        successors: &[label_endif],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_endif,
                        successors: &[],
                        immediate_dominator: Some(label_else),
                    },
                ],
                entry_block: label_start,
            },
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
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_default],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_default,
                        successors: &[label_merge],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_merge,
                        successors: &[],
                        immediate_dominator: Some(label_default),
                    },
                ],
                entry_block: label_start,
            },
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
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_case1, label_default],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_case1,
                        successors: &[],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_default,
                        successors: &[label_merge],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_merge,
                        successors: &[],
                        immediate_dominator: Some(label_default),
                    },
                ],
                entry_block: label_start,
            },
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
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_case1, label_default],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_case1,
                        successors: &[label_default],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_default,
                        successors: &[label_merge],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_merge,
                        successors: &[],
                        immediate_dominator: Some(label_default),
                    },
                ],
                entry_block: label_start,
            },
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
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_case1, label_case2, label_default],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_case1,
                        successors: &[label_default],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_default,
                        successors: &[label_case2],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_case2,
                        successors: &[label_merge],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_merge,
                        successors: &[],
                        immediate_dominator: Some(label_case2),
                    },
                ],
                entry_block: label_start,
            },
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
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_case1, label_case2, label_default],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_case1,
                        successors: &[label_merge],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_default,
                        successors: &[label_case2],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_case2,
                        successors: &[label_merge],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_merge,
                        successors: &[],
                        immediate_dominator: Some(label_start),
                    },
                ],
                entry_block: label_start,
            },
        );
    }

    #[test]
    fn test_cfg_while() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_header = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::Branch {
            target_label: label_header,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_header),
        });
        instructions.push(Instruction::LoopMerge {
            merge_block: label_merge,
            continue_target: label_header,
            loop_control: LoopControl {
                unroll: None,
                dont_unroll: None,
                dependency_infinite: None,
                dependency_length: None,
            },
        });
        instructions.push(Instruction::BranchConditional {
            condition: id_factory.next(),
            true_label: label_header,
            false_label: label_merge,
            branch_weights: Vec::new(),
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_header],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_header,
                        successors: &[label_merge, label_header],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_merge,
                        successors: &[],
                        immediate_dominator: Some(label_header),
                    },
                ],
                entry_block: label_start,
            },
        );
    }

    #[test]
    fn test_cfg_while_body() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_header = id_factory.next();
        let label_body = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::Branch {
            target_label: label_header,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_header),
        });
        instructions.push(Instruction::LoopMerge {
            merge_block: label_merge,
            continue_target: label_header,
            loop_control: LoopControl {
                unroll: None,
                dont_unroll: None,
                dependency_infinite: None,
                dependency_length: None,
            },
        });
        instructions.push(Instruction::BranchConditional {
            condition: id_factory.next(),
            true_label: label_body,
            false_label: label_merge,
            branch_weights: Vec::new(),
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_body),
        });
        instructions.push(Instruction::Branch {
            target_label: label_header,
        });
        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &CFGTemplate {
                blocks: &[
                    CFGTemplateBlock {
                        label: label_start,
                        successors: &[label_header],
                        immediate_dominator: None,
                    },
                    CFGTemplateBlock {
                        label: label_header,
                        successors: &[label_merge, label_body],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_body,
                        successors: &[label_header],
                        immediate_dominator: Some(label_header),
                    },
                    CFGTemplateBlock {
                        label: label_merge,
                        successors: &[],
                        immediate_dominator: Some(label_header),
                    },
                ],
                entry_block: label_start,
            },
        );
    }
}
