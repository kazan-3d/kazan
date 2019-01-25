// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

pub mod structure_tree;

use crate::instruction_properties::InstructionProperties;
use petgraph::{algo::dominators, graph::IndexType, prelude::*};
use spirv_parser::{IdRef, Instruction};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub struct Instructions(Vec<Instruction>);

impl ops::Deref for Instructions {
    type Target = [Instruction];
    fn deref(&self) -> &[Instruction] {
        &self.0
    }
}

impl fmt::Debug for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in &self.0 {
            write!(f, "{}", i)?;
        }
        Ok(())
    }
}

impl Instructions {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Instructions(instructions)
    }
    pub fn terminating_instruction(&self) -> Option<&Instruction> {
        self.last()
    }
    pub fn merge_instruction(&self) -> Option<&Instruction> {
        if self.len() < 2 {
            None
        } else {
            match &self[self.len() - 2] {
                instruction @ Instruction::LoopMerge { .. }
                | instruction @ Instruction::SelectionMerge { .. } => Some(instruction),
                _ => None,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BasicBlock {
    label: IdRef,
    instructions: Instructions,
    parent_structure_tree_node: RefCell<Weak<structure_tree::Node>>,
}

impl BasicBlock {
    pub fn label(&self) -> IdRef {
        self.label
    }
    pub fn instructions(&self) -> &Instructions {
        &self.instructions
    }
    pub fn parent_structure_tree_node(&self) -> Option<Rc<structure_tree::Node>> {
        self.parent_structure_tree_node.borrow().upgrade()
    }
    fn set_parent_structure_tree_node(&self, new_parent: &Rc<structure_tree::Node>) {
        let mut parent = self.parent_structure_tree_node.borrow_mut();
        assert!(parent.upgrade().is_none());
        *parent = Rc::downgrade(new_parent);
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
pub struct CFG {
    graph: CFGGraph,
    label_to_node_index_map: HashMap<IdRef, CFGNodeIndex>,
    dominators: CFGDominators,
    structure_tree: structure_tree::StructureTree,
}

impl ops::Deref for CFG {
    type Target = CFGGraph;
    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}

impl CFG {
    pub fn entry_node_index(&self) -> CFGNodeIndex {
        self.dominators().root()
    }
    pub fn entry_block(&self) -> &BasicBlock {
        &self[self.entry_node_index()]
    }
    pub fn dominators(&self) -> &CFGDominators {
        &self.dominators
    }
    pub fn structure_tree(&self) -> &structure_tree::StructureTree {
        &self.structure_tree
    }
    pub fn new(function_instructions: &[Instruction]) -> CFG {
        assert!(!function_instructions.is_empty());
        let mut graph = CFGGraph::default();
        let mut current_label = None;
        let mut current_instructions = Vec::new();
        let mut label_to_node_index_map = HashMap::new();
        for instruction in function_instructions {
            if current_label.is_none() {
                if let Instruction::Label { id_result } = instruction {
                    current_label = Some(id_result.0);
                } else {
                    assert!(
                        InstructionProperties::new(instruction).is_debug_line(),
                        "invalid instruction before OpLabel"
                    );
                }
                current_instructions.push(instruction.clone());
            } else {
                current_instructions.push(instruction.clone());
                if InstructionProperties::new(instruction).is_block_terminator() {
                    let label = current_label.take().unwrap();
                    label_to_node_index_map.insert(
                        label,
                        graph.add_node(BasicBlock {
                            label,
                            instructions: Instructions::new(current_instructions),
                            parent_structure_tree_node: RefCell::new(Weak::new()),
                        }),
                    );
                    current_instructions = Vec::new();
                }
            }
        }
        assert!(current_instructions.is_empty());
        let entry_node_index = graph.node_indices().next().unwrap();
        let mut successors_set = HashSet::new();
        for node_index in graph.node_indices() {
            successors_set.clear();
            let successors: Vec<_> = InstructionProperties::new(
                graph[node_index]
                    .instructions()
                    .terminating_instruction()
                    .unwrap(),
            )
            .targets()
            .unwrap()
            .filter(|&successor| successors_set.insert(successor)) // remove duplicates
            .collect(); // collect into Vec to retain order
            for target in successors {
                graph.add_edge(node_index, label_to_node_index_map[&target], ());
            }
        }
        let dominators = dominators::simple_fast(&graph, entry_node_index);
        let structure_tree = structure_tree::StructureTree::parse(
            &graph,
            &label_to_node_index_map,
            dominators.root(),
        );
        print!("{}", structure_tree.display(&graph));
        CFG {
            graph,
            label_to_node_index_map,
            dominators,
            structure_tree,
        }
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
    struct TemplateStructureTreeNode<'a> {
        kind: structure_tree::NodeKind,
        children: &'a [TemplateStructureTreeChild<'a>],
    }

    impl TemplateStructureTreeNode<'_> {
        fn is_equivalent(&self, rhs: &structure_tree::Node, graph: &CFGGraph) -> bool {
            if self.kind != rhs.kind() {
                return false;
            }
            if self.children.len() != rhs.children().len() {
                return false;
            }
            for (lhs_child, rhs_child) in self.children.iter().zip(rhs.children().iter()) {
                if !lhs_child.is_equivalent(rhs_child, graph) {
                    return false;
                }
            }
            true
        }
    }

    #[derive(Debug)]
    enum TemplateStructureTreeChild<'a> {
        Node(TemplateStructureTreeNode<'a>),
        BasicBlock(IdRef),
    }

    impl TemplateStructureTreeChild<'_> {
        fn is_equivalent(&self, rhs: &structure_tree::Child, graph: &CFGGraph) -> bool {
            match self {
                TemplateStructureTreeChild::Node(lhs_node) => {
                    if let structure_tree::Child::Node(rhs_node) = rhs {
                        lhs_node.is_equivalent(rhs_node, graph)
                    } else {
                        false
                    }
                }
                &TemplateStructureTreeChild::BasicBlock(lhs_block) => {
                    if let structure_tree::Child::BasicBlock(rhs_block) = *rhs {
                        lhs_block == graph[rhs_block].label()
                    } else {
                        false
                    }
                }
            }
        }
    }

    #[derive(Debug)]
    struct TemplateStructureTree<'a>(TemplateStructureTreeNode<'a>);

    impl TemplateStructureTree<'_> {
        fn is_equivalent(&self, rhs: &structure_tree::StructureTree, graph: &CFGGraph) -> bool {
            assert_eq!(self.0.kind, structure_tree::NodeKind::Root);
            self.0.is_equivalent(rhs.root(), graph)
        }
    }

    #[derive(Debug)]
    struct CFGTemplate<'a> {
        blocks: &'a [CFGTemplateBlock<'a>],
        entry_block: IdRef,
        structure_tree: TemplateStructureTree<'a>,
    }

    fn dump_template_structure_tree(
        structure_tree: &structure_tree::StructureTree,
        graph: &CFGGraph,
    ) {
        fn indent(indent_depth: usize) {
            for _ in 0..indent_depth {
                print!("    ");
            }
        }
        fn dump_node(node: &structure_tree::Node, graph: &CFGGraph, indent_depth: usize) {
            indent(indent_depth);
            println!("TemplateStructureTreeNode {{");
            indent(indent_depth + 1);
            println!("kind: structure_tree::NodeKind::{:?},", node.kind());
            indent(indent_depth + 1);
            println!("children: &[");
            for child in node.children().iter() {
                match child {
                    structure_tree::Child::Node(child_node) => {
                        indent(indent_depth + 2);
                        println!("TemplateStructureTreeChild::Node(");
                        dump_node(child_node, graph, indent_depth + 3);
                        indent(indent_depth + 2);
                        println!("),");
                    }
                    structure_tree::Child::BasicBlock(basic_block) => {
                        indent(indent_depth + 2);
                        println!(
                            "TemplateStructureTreeChild::BasicBlock({:?}),",
                            graph[*basic_block].label()
                        );
                    }
                }
            }
            indent(indent_depth + 1);
            println!("],");
            indent(indent_depth);
            println!("}}");
        }
        println!("TemplateStructureTree(");
        dump_node(structure_tree.root(), graph, 1);
        println!(")");
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
        let structure_tree = cfg.structure_tree();
        print!("{}", structure_tree.display(&*cfg));
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
                .neighbors_directed(node_index, Outgoing)
                .map(|node| cfg[node].label())
                .collect();
            assert_eq!(expected_successors, actual_successors, "label: {}", label);
            assert_eq!(
                dominators
                    .immediate_dominator(node_index)
                    .map(|v| cfg[v].label()),
                immediate_dominator,
                "label: {}",
                label
            );
        }
        if !cfg_template
            .structure_tree
            .is_equivalent(structure_tree, &cfg)
        {
            dump_template_structure_tree(structure_tree, &*cfg);
            panic!("Non-equivalent StructureTree");
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[TemplateStructureTreeChild::BasicBlock(label1)],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[TemplateStructureTreeChild::BasicBlock(label1)],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::BasicBlock(label1),
                        TemplateStructureTreeChild::BasicBlock(label2),
                    ],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[TemplateStructureTreeChild::BasicBlock(label_start)],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_endif),
                    ],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_start),
                                TemplateStructureTreeChild::BasicBlock(label_then),
                            ],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_endif),
                    ],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_start),
                                TemplateStructureTreeChild::BasicBlock(label_then),
                                TemplateStructureTreeChild::BasicBlock(label_else),
                            ],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_endif),
                    ],
                }),
            },
        );
    }

    #[test]
    fn test_cfg_conditional_return_return() {
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
        instructions.push(Instruction::Return);

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
                        successors: &[],
                        immediate_dominator: Some(label_start),
                    },
                    CFGTemplateBlock {
                        label: label_endif,
                        successors: &[],
                        immediate_dominator: None,
                    },
                ],
                entry_block: label_start,
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[TemplateStructureTreeChild::Node(
                        TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_start),
                                TemplateStructureTreeChild::BasicBlock(label_then),
                                TemplateStructureTreeChild::BasicBlock(label_else),
                            ],
                        },
                    )],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_start),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_default,
                                    )],
                                }),
                            ],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_merge),
                    ],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_start),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_case1,
                                    )],
                                }),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_default,
                                    )],
                                }),
                            ],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_merge),
                    ],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_start),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_case1,
                                    )],
                                }),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_default,
                                    )],
                                }),
                            ],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_merge),
                    ],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_start),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_case1,
                                    )],
                                }),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_default,
                                    )],
                                }),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_case2,
                                    )],
                                }),
                            ],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_merge),
                    ],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_start),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_case1,
                                    )],
                                }),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_default,
                                    )],
                                }),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_case2,
                                    )],
                                }),
                            ],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_merge),
                    ],
                }),
            },
        );
    }

    #[test]
    fn test_cfg_switch_break_default_fallthrough_break2() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_case1 = id_factory.next();
        let label_case2 = id_factory.next();
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
            id_result: IdResult(label_case2),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Selection,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_start),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_case1,
                                    )],
                                }),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_default,
                                    )],
                                }),
                                TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                                    kind: structure_tree::NodeKind::Case,
                                    children: &[TemplateStructureTreeChild::BasicBlock(
                                        label_case2,
                                    )],
                                }),
                            ],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_merge),
                    ],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::BasicBlock(label_start),
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Loop,
                            children: &[TemplateStructureTreeChild::BasicBlock(label_header)],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_merge),
                    ],
                }),
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
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::BasicBlock(label_start),
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Loop,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_header),
                                TemplateStructureTreeChild::BasicBlock(label_body),
                            ],
                        }),
                        TemplateStructureTreeChild::BasicBlock(label_merge),
                    ],
                }),
            },
        );
    }

    #[test]
    fn test_cfg_while_body_infinite() {
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
        instructions.push(Instruction::Branch {
            target_label: label_body,
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
                        successors: &[label_body],
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
                        immediate_dominator: None,
                    },
                ],
                entry_block: label_start,
                structure_tree: TemplateStructureTree(TemplateStructureTreeNode {
                    kind: structure_tree::NodeKind::Root,
                    children: &[
                        TemplateStructureTreeChild::BasicBlock(label_start),
                        TemplateStructureTreeChild::Node(TemplateStructureTreeNode {
                            kind: structure_tree::NodeKind::Loop,
                            children: &[
                                TemplateStructureTreeChild::BasicBlock(label_header),
                                TemplateStructureTreeChild::BasicBlock(label_body),
                            ],
                        }),
                    ],
                }),
            },
        );
    }
}
