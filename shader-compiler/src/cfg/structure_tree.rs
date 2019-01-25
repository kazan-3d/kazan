// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use super::{CFGGraph, CFGNodeIndex};
use crate::instruction_properties::InstructionProperties;
use petgraph::prelude::*;
use spirv_parser::{IdRef, Instruction};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter;
use std::rc::{Rc, Weak};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum NodeKind {
    Selection,
    Continue,
    Loop,
    Case,
    Root,
}

#[derive(Clone, Debug)]
pub struct Node {
    kind: NodeKind,
    parent: RefCell<Weak<Node>>,
    children: Vec<Child>,
}

#[derive(Copy, Clone, Debug)]
struct Indent(usize);

impl Indent {
    fn make_more(self) -> Self {
        Indent(self.0 + 1)
    }
}

impl Default for Indent {
    fn default() -> Self {
        Indent(0)
    }
}

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..self.0 {
            write!(f, "    ")?;
        }
        Ok(())
    }
}

pub struct NodeDisplay<'a> {
    node: &'a Node,
    graph: &'a CFGGraph,
    indent: Indent,
}

impl fmt::Display for NodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}{:?}", self.indent, self.node.kind)?;
        for child in self.node.children.iter() {
            fmt::Display::fmt(
                &child.display_with_indent(self.graph, self.indent.make_more()),
                f,
            )?;
        }
        Ok(())
    }
}

impl fmt::Debug for NodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Node")
            .field("kind", &self.node.kind)
            .field("parent", &self.node.parent)
            .field(
                "children",
                &ChildrenDisplay {
                    children: &*self.node.children,
                    graph: self.graph,
                },
            )
            .finish()
    }
}

impl Node {
    pub fn kind(&self) -> NodeKind {
        self.kind
    }
    pub fn parent(&self) -> Option<Rc<Node>> {
        self.parent.borrow().upgrade()
    }
    pub fn children(&self) -> &Vec<Child> {
        &self.children
    }
    pub fn display<'a>(&'a self, graph: &'a CFGGraph) -> NodeDisplay<'a> {
        self.display_with_indent(graph, Indent::default())
    }
    fn display_with_indent<'a>(&'a self, graph: &'a CFGGraph, indent: Indent) -> NodeDisplay<'a> {
        NodeDisplay {
            node: self,
            graph,
            indent,
        }
    }
    fn set_parent(&self, new_parent: &Rc<Node>) {
        let mut parent = self.parent.borrow_mut();
        assert!(parent.upgrade().is_none());
        *parent = Rc::downgrade(new_parent);
    }
}

#[derive(Clone, Debug)]
pub enum Child {
    Node(Rc<Node>),
    BasicBlock(CFGNodeIndex),
}

struct BasicBlockDisplay<'a> {
    node_index: CFGNodeIndex,
    graph: &'a CFGGraph,
}

impl fmt::Display for BasicBlockDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            self.graph[self.node_index].label(),
            self.node_index.index(),
        )
    }
}

impl fmt::Debug for BasicBlockDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CFGNodeIndex({}) {}",
            self.node_index.index(),
            self.graph[self.node_index].label()
        )
    }
}

pub struct ChildDisplay<'a> {
    child: &'a Child,
    graph: &'a CFGGraph,
    indent: Indent,
}

impl fmt::Display for ChildDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.child {
            Child::Node(node) => {
                fmt::Display::fmt(&node.display_with_indent(self.graph, self.indent), f)
            }
            &Child::BasicBlock(node_index) => writeln!(
                f,
                "{}{}",
                self.indent,
                BasicBlockDisplay {
                    node_index,
                    graph: self.graph
                }
            ),
        }
    }
}

impl fmt::Debug for ChildDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.child {
            Child::Node(node) => f
                .debug_tuple("Node")
                .field(&node.display(self.graph))
                .finish(),
            &Child::BasicBlock(node_index) => f
                .debug_tuple("BasicBlock")
                .field(&BasicBlockDisplay {
                    node_index,
                    graph: self.graph,
                })
                .finish(),
        }
    }
}

impl Child {
    pub fn display<'a>(&'a self, graph: &'a CFGGraph) -> ChildDisplay<'a> {
        self.display_with_indent(graph, Indent::default())
    }
    fn display_with_indent<'a>(&'a self, graph: &'a CFGGraph, indent: Indent) -> ChildDisplay<'a> {
        ChildDisplay {
            child: self,
            graph,
            indent,
        }
    }
}

struct ChildrenDisplay<'a> {
    children: &'a [Child],
    graph: &'a CFGGraph,
}

impl fmt::Debug for ChildrenDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list()
            .entries(self.children.iter().map(|child| child.display(self.graph)))
            .finish()
    }
}

pub struct StructureTreeDisplay<'a> {
    structure_tree: &'a StructureTree,
    graph: &'a CFGGraph,
}

impl fmt::Display for StructureTreeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.structure_tree.root().display(self.graph), f)
    }
}

impl fmt::Debug for StructureTreeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("StructureTree")
            .field(&self.structure_tree.root().display(self.graph))
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct StructureTree(Rc<Node>);

impl StructureTree {
    pub fn root(&self) -> &Rc<Node> {
        &self.0
    }
    #[allow(dead_code)]
    pub fn display<'a>(&'a self, graph: &'a CFGGraph) -> StructureTreeDisplay<'a> {
        StructureTreeDisplay {
            structure_tree: self,
            graph,
        }
    }
    pub(super) fn parse(
        graph: &CFGGraph,
        label_to_node_index_map: &HashMap<IdRef, CFGNodeIndex>,
        root: CFGNodeIndex,
    ) -> Self {
        let parser = Parser {
            graph,
            label_to_node_index_map,
        };
        let ParseChildrenResult { children, targets } =
            parser.parse_children(root, Cow::Owned(HashSet::new()));
        assert!(targets.is_empty());
        StructureTree(parser.new_node(NodeKind::Root, children))
    }
}

#[derive(Debug)]
struct Parser<'a> {
    graph: &'a CFGGraph,
    label_to_node_index_map: &'a HashMap<IdRef, CFGNodeIndex>,
}

struct ParseChildResult {
    child: Child,
    targets: HashSet<CFGNodeIndex>,
}

struct ParseChildrenResult {
    children: Vec<Child>,
    targets: HashSet<CFGNodeIndex>,
}

struct ChildrenAccumulator {
    children: Vec<Child>,
    targets: HashSet<CFGNodeIndex>,
}

impl ChildrenAccumulator {
    fn new(children: Vec<Child>) -> Self {
        Self {
            children,
            targets: HashSet::new(),
        }
    }
    fn accumulate(&mut self, mut rhs: ParseChildrenResult) {
        self.children.append(&mut rhs.children);
        self.targets.extend(rhs.targets.into_iter());
    }
}

impl Parser<'_> {
    fn new_node(&self, kind: NodeKind, children: Vec<Child>) -> Rc<Node> {
        let retval = Rc::new(Node {
            kind,
            parent: RefCell::default(),
            children,
        });
        for child in &retval.children {
            match child {
                Child::Node(node) => node.set_parent(&retval),
                Child::BasicBlock(basic_block) => {
                    self.graph[*basic_block].set_parent_structure_tree_node(&retval);
                }
            }
        }
        retval
    }
    fn parse_children(
        &self,
        basic_block: CFGNodeIndex,
        exit_targets: Cow<HashSet<CFGNodeIndex>>,
    ) -> ParseChildrenResult {
        let exit_targets = &*exit_targets;
        if exit_targets.contains(&basic_block) {
            return ParseChildrenResult {
                children: Vec::new(),
                targets: iter::once(basic_block).collect(),
            };
        }
        let ParseChildResult { child, targets } =
            self.parse_child(basic_block, Cow::Borrowed(exit_targets));
        let mut children = vec![child];
        let mut final_targets = HashSet::new();
        let mut targets_iter = targets.into_iter();
        while let Some(target) = targets_iter.next() {
            if exit_targets.contains(&target) {
                final_targets.insert(target);
                continue;
            }
            assert_eq!(targets_iter.next(), None);
            let ParseChildResult { child, targets } =
                self.parse_child(target, Cow::Borrowed(exit_targets));
            children.push(child);
            targets_iter = targets.into_iter();
        }
        ParseChildrenResult {
            children,
            targets: final_targets,
        }
    }
    fn parse_switch(
        &self,
        basic_block: CFGNodeIndex,
        exit_targets: Cow<HashSet<CFGNodeIndex>>,
        merge_block: CFGNodeIndex,
        default: CFGNodeIndex,
        targets: Vec<CFGNodeIndex>,
    ) -> ParseChildResult {
        let mut exit_targets = exit_targets.into_owned();
        exit_targets.insert(merge_block);
        let mut default_exit_targets = exit_targets.clone();
        default_exit_targets.extend(targets.iter().cloned());
        let ParseChildrenResult {
            children: default_case_children,
            targets: default_case_targets,
        } = self.parse_children(default, Cow::Owned(default_exit_targets));
        let mut default_case_node = if default_case_children.is_empty() {
            None
        } else {
            Some(Child::Node(
                self.new_node(NodeKind::Case, default_case_children),
            ))
        };
        let mut switch_children = vec![Child::BasicBlock(basic_block)];
        let mut returned_targets: HashSet<_> = default_case_targets
            .intersection(&exit_targets)
            .cloned()
            .collect();
        for (target_index, &target) in targets.iter().enumerate() {
            let next_target = targets.get(target_index + 1).and_then(|&next_target| {
                if exit_targets.contains(&next_target) {
                    None
                } else {
                    Some(next_target)
                }
            });
            if exit_targets.contains(&target) {
                continue;
            }
            if next_target == Some(target) {
                // skip empty cases
                continue;
            }
            if default_case_targets.contains(&target) {
                // default falls through to this case
                if let Some(default_case_node) = default_case_node.take() {
                    switch_children.push(default_case_node);
                }
            }
            let mut case_exit_targets = exit_targets.clone();
            case_exit_targets.extend(next_target);
            if default_case_node.is_some() {
                case_exit_targets.insert(default);
            }
            let ParseChildrenResult { children, targets } =
                self.parse_children(target, Cow::Owned(case_exit_targets));
            if !children.is_empty() {
                switch_children.push(Child::Node(self.new_node(NodeKind::Case, children)));
            }
            if targets.contains(&default) && !exit_targets.contains(&default) {
                // this case falls through to default
                if let Some(default_case_node) = default_case_node.take() {
                    switch_children.push(default_case_node);
                }
            }
            returned_targets.extend(targets.intersection(&exit_targets).cloned());
        }
        if let Some(default_case_node) = default_case_node {
            switch_children.push(default_case_node);
        }
        ParseChildResult {
            child: Child::Node(self.new_node(NodeKind::Selection, switch_children)),
            targets: returned_targets,
        }
    }
    fn parse_child(
        &self,
        basic_block: CFGNodeIndex,
        exit_targets: Cow<HashSet<CFGNodeIndex>>,
    ) -> ParseChildResult {
        match self.graph[basic_block].instructions().merge_instruction() {
            None => {
                let targets: HashSet<_> = InstructionProperties::new(
                    self.graph[basic_block]
                        .instructions()
                        .terminating_instruction()
                        .unwrap(),
                )
                .targets()
                .unwrap()
                .map(|label| self.label_to_node_index_map[&label])
                .collect();
                ParseChildResult {
                    child: Child::BasicBlock(basic_block),
                    targets,
                }
            }
            Some(&Instruction::LoopMerge {
                merge_block,
                continue_target,
                ..
            }) => {
                let mut exit_targets = exit_targets.into_owned();
                exit_targets.insert(self.label_to_node_index_map[&merge_block]);
                let mut body_exit_targets = exit_targets.clone();
                let continue_target = self.label_to_node_index_map[&continue_target];
                body_exit_targets.insert(continue_target);
                let mut children = ChildrenAccumulator::new(vec![Child::BasicBlock(basic_block)]);
                for successor in self.graph.neighbors_directed(basic_block, Outgoing) {
                    children.accumulate(
                        self.parse_children(successor, Cow::Borrowed(&body_exit_targets)),
                    );
                }
                let ChildrenAccumulator {
                    mut children,
                    mut targets,
                } = children;
                if targets.remove(&continue_target) {
                    let mut continue_exit_targets = exit_targets;
                    continue_exit_targets.insert(basic_block);
                    let ParseChildrenResult {
                        children: continue_children,
                        targets: mut continue_targets,
                    } = self.parse_children(continue_target, Cow::Owned(continue_exit_targets));
                    if !continue_children.is_empty() {
                        children.push(Child::Node(
                            self.new_node(NodeKind::Continue, continue_children),
                        ));
                    }
                    continue_targets.remove(&basic_block);
                    targets.extend(continue_targets);
                }
                ParseChildResult {
                    child: Child::Node(self.new_node(NodeKind::Loop, children)),
                    targets,
                }
            }
            Some(&Instruction::SelectionMerge { merge_block, .. }) => {
                let terminating_instruction = self.graph[basic_block]
                    .instructions()
                    .terminating_instruction()
                    .expect("missing terminating instruction");
                match *terminating_instruction {
                    Instruction::Switch32 {
                        default,
                        ref target,
                        ..
                    } => self.parse_switch(
                        basic_block,
                        exit_targets,
                        self.label_to_node_index_map[&merge_block],
                        self.label_to_node_index_map[&default],
                        target
                            .iter()
                            .map(|&(_, v)| self.label_to_node_index_map[&v])
                            .collect(),
                    ),
                    Instruction::Switch64 {
                        default,
                        ref target,
                        ..
                    } => self.parse_switch(
                        basic_block,
                        exit_targets,
                        self.label_to_node_index_map[&merge_block],
                        self.label_to_node_index_map[&default],
                        target
                            .iter()
                            .map(|&(_, v)| self.label_to_node_index_map[&v])
                            .collect(),
                    ),
                    Instruction::BranchConditional {
                        true_label,
                        false_label,
                        ..
                    } => {
                        let mut children =
                            ChildrenAccumulator::new(vec![Child::BasicBlock(basic_block)]);
                        let mut children_exit_targets = exit_targets.into_owned();
                        children_exit_targets.insert(self.label_to_node_index_map[&merge_block]);
                        children.accumulate(self.parse_children(
                            self.label_to_node_index_map[&true_label],
                            Cow::Borrowed(&children_exit_targets),
                        ));
                        children.accumulate(self.parse_children(
                            self.label_to_node_index_map[&false_label],
                            Cow::Owned(children_exit_targets),
                        ));
                        let ChildrenAccumulator { children, targets } = children;
                        ParseChildResult {
                            child: Child::Node(self.new_node(NodeKind::Selection, children)),
                            targets,
                        }
                    }
                    Instruction::Branch { .. }
                    | Instruction::Kill
                    | Instruction::Return
                    | Instruction::ReturnValue { .. }
                    | Instruction::Unreachable => unreachable!(
                        "unexpected terminating instruction after OpSelectionMerge:\n{}",
                        terminating_instruction
                    ),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
