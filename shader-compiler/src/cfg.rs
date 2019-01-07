// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018,2019 Jacob Lifshay

use crate::instruction_properties::InstructionProperties;
use petgraph::{
    algo::dominators,
    graph::IndexType,
    prelude::*,
    visit::{VisitMap, Visitable},
};
use spirv_parser::{IdRef, Instruction};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Write};
use std::ops;

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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum EdgeKind {
    Unreachable,
    Normal,
    LoopBackEdge,
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
    pub fn instructions(&self) -> &Instructions {
        &self.instructions
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

pub type CFGGraph = DiGraph<BasicBlock, EdgeKind, CFGIndexType>;

pub type CFGDominators = dominators::Dominators<CFGNodeIndex>;

#[derive(Clone, Debug)]
pub struct CFG {
    graph: CFGGraph,
    label_to_node_index_map: HashMap<IdRef, CFGNodeIndex>,
    dominators: CFGDominators,
    structure_tree: CFGStructureTree,
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
    pub fn structure_tree(&self) -> &CFGStructureTree {
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
                        }),
                    );
                    current_instructions = Vec::new();
                }
            }
        }
        assert!(current_instructions.is_empty());
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
                graph.add_edge(
                    node_index,
                    label_to_node_index_map[&target],
                    EdgeKind::Normal,
                );
            }
        }
        let dominators = dominators::simple_fast(&graph, graph.node_indices().next().unwrap());
        for edge_index in graph.edge_indices() {
            let (source, target) = graph.edge_endpoints(edge_index).unwrap();
            if let Some(mut source_dominators) = dominators.dominators(source) {
                if source_dominators.any(|dominator| dominator == target) {
                    graph[edge_index] = EdgeKind::LoopBackEdge;
                    let target_block_instructions = graph[target].instructions();
                    match target_block_instructions[target_block_instructions.len() - 2] {
                        Instruction::LoopMerge { .. } => {}
                        _ => unreachable!("back edge must go to loop header block"),
                    }
                }
            } else {
                graph[edge_index] = EdgeKind::Unreachable;
            }
        }
        let structure_tree =
            CFGStructureTree::parse(&graph, &label_to_node_index_map, dominators.root());
        CFG {
            graph,
            label_to_node_index_map,
            dominators,
            structure_tree,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CFGStructureTreeNode {
    Loop { children: CFGStructureTree },
    Node { node_index: CFGNodeIndex },
}

/// nodes are in a topological order
#[derive(Clone, Debug)]
pub struct CFGStructureTree(Vec<CFGStructureTreeNode>);

impl CFGStructureTree {
    pub fn dump(&self, graph: &CFGGraph) -> String {
        let mut stack = vec![self.iter()];
        let mut retval = String::new();
        while let Some(mut iter) = stack.pop() {
            if let Some(node) = iter.next() {
                for _ in 0..stack.len() {
                    write!(&mut retval, "    ").unwrap();
                }
                stack.push(iter);
                match *node {
                    CFGStructureTreeNode::Loop { ref children } => {
                        writeln!(&mut retval, "Loop:").unwrap();
                        stack.push(children.iter());
                    }
                    CFGStructureTreeNode::Node { node_index } => {
                        writeln!(
                            &mut retval,
                            "{}: (index {})",
                            graph[node_index].label,
                            node_index.index()
                        )
                        .unwrap();
                    }
                }
            }
        }
        retval
    }
}

impl ops::Deref for CFGStructureTree {
    type Target = Vec<CFGStructureTreeNode>;
    fn deref(&self) -> &Vec<CFGStructureTreeNode> {
        &self.0
    }
}

impl CFGStructureTree {
    fn parse(
        graph: &CFGGraph,
        label_to_node_index_map: &HashMap<IdRef, CFGNodeIndex>,
        root: CFGNodeIndex,
    ) -> Self {
        CFGStructureTreeParser {
            graph,
            label_to_node_index_map,
        }
        .parse_tree(root, None, IgnoreInitialLoop::NotIgnored)
        .structure_tree
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum MergeReached {
    Reached,
    NotReached,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum IgnoreInitialLoop {
    Ignored,
    NotIgnored,
}

struct CFGStructureTreeNodeParseResults {
    successors: Vec<CFGNodeIndex>,
    structure_tree_node: CFGStructureTreeNode,
    merge_reached: MergeReached,
}

struct CFGStructureTreeParseResults {
    structure_tree: CFGStructureTree,
    merge_reached: MergeReached,
}

struct CFGStructureTreeParser<'a> {
    graph: &'a CFGGraph,
    label_to_node_index_map: &'a HashMap<IdRef, CFGNodeIndex>,
}

impl CFGStructureTreeParser<'_> {
    fn parse_node(
        &self,
        start: CFGNodeIndex,
        merge_target: Option<CFGNodeIndex>,
        ignore_initial_loop: IgnoreInitialLoop,
    ) -> CFGStructureTreeNodeParseResults {
        match self.graph[start].instructions().merge_instruction() {
            Some(Instruction::LoopMerge { merge_block, .. })
                if ignore_initial_loop != IgnoreInitialLoop::Ignored =>
            {
                let merge_block = self.label_to_node_index_map[merge_block];
                let CFGStructureTreeParseResults {
                    structure_tree,
                    merge_reached,
                } = self.parse_tree(start, Some(merge_block), IgnoreInitialLoop::Ignored);
                let successors = if merge_reached == MergeReached::Reached {
                    vec![merge_block]
                } else {
                    Vec::new()
                };
                CFGStructureTreeNodeParseResults {
                    successors,
                    structure_tree_node: CFGStructureTreeNode::Loop {
                        children: structure_tree,
                    },
                    merge_reached: MergeReached::NotReached,
                }
            }
            _ => {
                let mut merge_reached = MergeReached::NotReached;
                let mut successors = Vec::new();
                for edge in self.graph.edges_directed(start, Outgoing) {
                    // exclude the merge target and loop back edge to
                    // only visit nodes in the same loop nesting level
                    if Some(edge.target()) == merge_target {
                        merge_reached = MergeReached::Reached;
                    } else if *edge.weight() != EdgeKind::LoopBackEdge {
                        successors.push(edge.target());
                    }
                }
                CFGStructureTreeNodeParseResults {
                    successors,
                    structure_tree_node: CFGStructureTreeNode::Node { node_index: start },
                    merge_reached,
                }
            }
        }
    }
    fn parse_tree(
        &self,
        start: CFGNodeIndex,
        merge_target: Option<CFGNodeIndex>,
        ignore_initial_loop: IgnoreInitialLoop,
    ) -> CFGStructureTreeParseResults {
        let mut structures = Vec::new();
        let mut merge_reached = MergeReached::NotReached;
        // visit nodes using a depth-first search recording them to `structures` in post-order
        let mut reachable = self.graph.visit_map();
        enum DFSNodeState {
            Pre(CFGNodeIndex),
            Post(CFGStructureTreeNode),
        }
        impl fmt::Debug for DFSNodeState {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    DFSNodeState::Pre(node) => write!(f, "Pre(#{})", node.index()),
                    DFSNodeState::Post(node) => write!(f, "Post({:?})", node),
                }
            }
        }
        let mut stack = vec![DFSNodeState::Pre(start)];
        loop {
            match stack.pop() {
                Some(DFSNodeState::Pre(node)) => {
                    if reachable.visit(node) {
                        let CFGStructureTreeNodeParseResults {
                            successors,
                            structure_tree_node,
                            merge_reached: current_merge_reached,
                        } = self.parse_node(
                            node,
                            merge_target,
                            if node == start {
                                ignore_initial_loop
                            } else {
                                IgnoreInitialLoop::NotIgnored
                            },
                        );
                        if current_merge_reached == MergeReached::Reached {
                            merge_reached = MergeReached::Reached;
                        }
                        stack.push(DFSNodeState::Post(structure_tree_node));
                        for &successor in successors.iter().rev() {
                            stack.push(DFSNodeState::Pre(successor));
                        }
                    }
                }
                Some(DFSNodeState::Post(structure)) => {
                    structures.push(structure); // build structures in post-order
                }
                None => break,
            }
        }
        structures.reverse(); // change to reverse post-order since we need them in a topological order
        CFGStructureTreeParseResults {
            merge_reached,
            structure_tree: CFGStructureTree(structures),
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
    enum CFGTemplateStructureTreeNode<'a> {
        Loop {
            children: CFGTemplateStructureTree<'a>,
        },
        Node {
            label: IdRef,
        },
    }

    impl CFGTemplateStructureTreeNode<'_> {
        fn is_equivalent(&self, rhs: &CFGStructureTreeNode, graph: &CFGGraph) -> bool {
            match self {
                CFGTemplateStructureTreeNode::Loop { children } => {
                    if let CFGStructureTreeNode::Loop {
                        children: rhs_children,
                    } = rhs
                    {
                        children.is_equivalent(rhs_children, graph)
                    } else {
                        false
                    }
                }
                CFGTemplateStructureTreeNode::Node { label } => {
                    if let CFGStructureTreeNode::Node { node_index } = *rhs {
                        graph[node_index].label == *label
                    } else {
                        false
                    }
                }
            }
        }
    }

    #[derive(Debug)]
    struct CFGTemplateStructureTree<'a>(&'a [CFGTemplateStructureTreeNode<'a>]);

    impl CFGTemplateStructureTree<'_> {
        fn is_equivalent(&self, rhs: &CFGStructureTree, graph: &CFGGraph) -> bool {
            if self.0.len() != rhs.len() {
                return false;
            }
            for (a, b) in self.0.iter().zip(rhs.iter()) {
                if !a.is_equivalent(b, graph) {
                    return false;
                }
            }
            true
        }
    }

    #[derive(Debug)]
    struct CFGTemplate<'a> {
        blocks: &'a [CFGTemplateBlock<'a>],
        entry_block: IdRef,
        structure_tree: CFGTemplateStructureTree<'a>,
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
        println!("{}", structure_tree.dump(&cfg));
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
        assert!(cfg_template
            .structure_tree
            .is_equivalent(structure_tree, &cfg));
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
                structure_tree: CFGTemplateStructureTree(&[CFGTemplateStructureTreeNode::Node {
                    label: label1,
                }]),
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
                structure_tree: CFGTemplateStructureTree(&[CFGTemplateStructureTreeNode::Node {
                    label: label1,
                }]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label1 },
                    CFGTemplateStructureTreeNode::Node { label: label2 },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Node { label: label_endif },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Node { label: label_then },
                    CFGTemplateStructureTreeNode::Node { label: label_endif },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Node { label: label_then },
                    CFGTemplateStructureTreeNode::Node { label: label_else },
                    CFGTemplateStructureTreeNode::Node { label: label_endif },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Node {
                        label: label_default,
                    },
                    CFGTemplateStructureTreeNode::Node { label: label_merge },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Node {
                        label: label_default,
                    },
                    CFGTemplateStructureTreeNode::Node { label: label_merge },
                    CFGTemplateStructureTreeNode::Node { label: label_case1 },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Node { label: label_case1 },
                    CFGTemplateStructureTreeNode::Node {
                        label: label_default,
                    },
                    CFGTemplateStructureTreeNode::Node { label: label_merge },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Node { label: label_case1 },
                    CFGTemplateStructureTreeNode::Node {
                        label: label_default,
                    },
                    CFGTemplateStructureTreeNode::Node { label: label_case2 },
                    CFGTemplateStructureTreeNode::Node { label: label_merge },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Node {
                        label: label_default,
                    },
                    CFGTemplateStructureTreeNode::Node { label: label_case1 },
                    CFGTemplateStructureTreeNode::Node { label: label_case2 },
                    CFGTemplateStructureTreeNode::Node { label: label_merge },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Node {
                        label: label_default,
                    },
                    CFGTemplateStructureTreeNode::Node { label: label_case1 },
                    CFGTemplateStructureTreeNode::Node { label: label_case2 },
                    CFGTemplateStructureTreeNode::Node { label: label_merge },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Loop {
                        children: CFGTemplateStructureTree(&[CFGTemplateStructureTreeNode::Node {
                            label: label_header,
                        }]),
                    },
                    CFGTemplateStructureTreeNode::Node { label: label_merge },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Loop {
                        children: CFGTemplateStructureTree(&[
                            CFGTemplateStructureTreeNode::Node {
                                label: label_header,
                            },
                            CFGTemplateStructureTreeNode::Node { label: label_body },
                        ]),
                    },
                    CFGTemplateStructureTreeNode::Node { label: label_merge },
                ]),
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
                structure_tree: CFGTemplateStructureTree(&[
                    CFGTemplateStructureTreeNode::Node { label: label_start },
                    CFGTemplateStructureTreeNode::Loop {
                        children: CFGTemplateStructureTree(&[
                            CFGTemplateStructureTreeNode::Node {
                                label: label_header,
                            },
                            CFGTemplateStructureTreeNode::Node { label: label_body },
                        ]),
                    },
                ]),
            },
        );
    }
}
