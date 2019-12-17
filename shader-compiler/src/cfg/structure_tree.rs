// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use super::{CFGEdgeIndex, CFGGraph, CFGNodeIndex};
use crate::debug_display::{DisplaySetWithCFG, DisplayWithCFG, HandleIsDebugWrapper, Indent};
use crate::instruction_properties::InstructionProperties;
use petgraph::prelude::*;
use spirv_parser::{IdRef, Instruction};
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter;
use std::rc::{Rc, Weak};

#[derive(Clone, Debug)]
pub struct NodeControlProperties {
    exit_targets: HashMap<CFGNodeIndex, Vec<CFGEdgeIndex>>,
    return_or_kill_blocks: Vec<CFGNodeIndex>,
}

impl NodeControlProperties {
    pub fn exit_edges(
        &self,
    ) -> impl Iterator<Item = CFGEdgeIndex> + iter::FusedIterator + fmt::Debug + Clone + '_ {
        self.exit_targets
            .iter()
            .flat_map(|(_, edges)| edges)
            .cloned()
    }
    pub fn exit_targets(&self) -> &HashMap<CFGNodeIndex, Vec<CFGEdgeIndex>> {
        &self.exit_targets
    }
    pub fn return_or_kill_blocks(&self) -> &Vec<CFGNodeIndex> {
        &self.return_or_kill_blocks
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_return_or_kill(block: CFGNodeIndex) -> Self {
        Self {
            exit_targets: HashMap::new(),
            return_or_kill_blocks: vec![block],
        }
    }
    pub fn from_exit_edge(exit_edge: CFGEdgeIndex, cfg: &CFGGraph) -> Self {
        Self::from_exit_edges(iter::once(exit_edge), cfg)
    }
    pub fn from_exit_edges<I: IntoIterator>(exit_edges: I, cfg: &CFGGraph) -> Self
    where
        I::Item: Borrow<CFGEdgeIndex>,
    {
        let mut exit_targets: HashMap<_, Vec<_>> = HashMap::new();
        for exit_edge in exit_edges {
            let exit_edge = *exit_edge.borrow();
            exit_targets
                .entry(cfg.edge_endpoints(exit_edge).unwrap().1)
                .or_default()
                .push(exit_edge);
        }
        Self {
            exit_targets,
            return_or_kill_blocks: Vec::new(),
        }
    }
    fn append_exit_target_and_edges_vec<T: BorrowMut<Vec<CFGEdgeIndex>>>(
        &mut self,
        exit_target: CFGNodeIndex,
        mut exit_edges: T,
    ) {
        self.exit_targets
            .entry(exit_target)
            .or_default()
            .append(exit_edges.borrow_mut());
    }
    fn append_exit_target_and_edge(&mut self, exit_target: CFGNodeIndex, exit_edge: CFGEdgeIndex) {
        self.exit_targets
            .entry(exit_target)
            .or_default()
            .push(exit_edge);
    }
    fn merge_assign(&mut self, rhs: Self) {
        let Self {
            exit_targets,
            mut return_or_kill_blocks,
        } = rhs;
        for (exit_target, exit_edges) in exit_targets.into_iter() {
            self.append_exit_target_and_edges_vec(exit_target, exit_edges);
        }
        self.return_or_kill_blocks
            .append(&mut return_or_kill_blocks);
    }
    fn merge(mut self, rhs: Self) -> Self {
        self.merge_assign(rhs);
        self
    }
}

impl Default for NodeControlProperties {
    fn default() -> Self {
        Self {
            exit_targets: HashMap::new(),
            return_or_kill_blocks: Vec::new(),
        }
    }
}

impl<'a> DisplayWithCFG<'a> for NodeControlProperties {
    type DisplayType = HandleIsDebugWrapper<NodeControlPropertiesDisplay<'a>>;
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        HandleIsDebugWrapper {
            value: NodeControlPropertiesDisplay {
                node_control_properties: self,
                cfg,
                indent,
            },
            is_debug,
        }
    }
}

pub struct NodeControlPropertiesDisplay<'a> {
    node_control_properties: &'a NodeControlProperties,
    cfg: &'a CFGGraph,
    indent: Indent,
}

impl fmt::Display for NodeControlPropertiesDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}NodeControlProperties {{", self.indent)?;
        let body_indent = self.indent.make_more();
        writeln!(
            f,
            "{}exit_edges: {}",
            body_indent,
            DisplaySetWithCFG::new(self.node_control_properties.exit_edges(), self.cfg, None),
        )?;
        writeln!(
            f,
            "{}return_or_kill_blocks: {},",
            body_indent,
            DisplaySetWithCFG::new(
                self.node_control_properties.return_or_kill_blocks(),
                self.cfg,
                None
            ),
        )?;
        writeln!(f, "{}}}", self.indent)
    }
}

impl fmt::Debug for NodeControlPropertiesDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NodeControlProperties")
            .field(
                "exit_edges",
                &DisplaySetWithCFG::new(self.node_control_properties.exit_edges(), self.cfg, None),
            )
            .field(
                "return_or_kill_blocks",
                &DisplaySetWithCFG::new(
                    self.node_control_properties.return_or_kill_blocks(),
                    self.cfg,
                    None,
                ),
            )
            .finish()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum NodeKind {
    If,
    IfPart,
    Continue,
    Loop,
    LoopBody,
    Switch,
    Case,
    Root,
}

#[derive(Clone, Debug)]
pub struct Node {
    kind: NodeKind,
    parent_node_and_index: RefCell<WeakNodeAndIndex>,
    children: Vec<Child>,
    nesting_depth: usize,
    first_basic_block: CFGNodeIndex,
    root: RefCell<Weak<Node>>,
    control_properties: NodeControlProperties,
}

impl<'a> DisplayWithCFG<'a> for Node {
    type DisplayType = HandleIsDebugWrapper<NodeDisplay<'a>>;
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        HandleIsDebugWrapper {
            value: NodeDisplay {
                node: self,
                cfg,
                indent,
            },
            is_debug,
        }
    }
}

impl Node {
    pub fn kind(&self) -> NodeKind {
        self.kind
    }
    pub fn parent_node_and_index(&self) -> Option<NodeAndIndex> {
        self.parent_node_and_index.borrow().upgrade()
    }
    pub fn children(&self) -> &Vec<Child> {
        &self.children
    }
    pub fn nesting_depth(&self) -> usize {
        self.nesting_depth
    }
    pub fn first_basic_block(&self) -> CFGNodeIndex {
        self.first_basic_block
    }
    pub fn root(&self) -> Rc<Node> {
        self.root.borrow().upgrade().expect("missing root node")
    }
    pub fn control_properties(&self) -> &NodeControlProperties {
        &self.control_properties
    }
    fn set_parent_node_and_index(&self, new_parent_node_and_index: &NodeAndIndex) {
        let mut parent_node_and_index = self.parent_node_and_index.borrow_mut();
        assert!(parent_node_and_index.upgrade().is_none());
        *parent_node_and_index = new_parent_node_and_index.into();
    }
}

impl PartialEq for Node {
    fn eq(&self, rhs: &Self) -> bool {
        assert!(
            Rc::ptr_eq(&self.root(), &rhs.root()),
            "Nodes from different StructureTrees are not comparable"
        );
        self.first_basic_block == rhs.first_basic_block && self.nesting_depth == rhs.nesting_depth
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.first_basic_block.hash(state);
        self.nesting_depth.hash(state);
    }
}

pub struct NodeDisplay<'a> {
    node: &'a Node,
    cfg: &'a CFGGraph,
    indent: Indent,
}

impl fmt::Display for NodeDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{}{:?} depth={}",
            self.indent, self.node.kind, self.node.nesting_depth
        )?;
        fmt::Display::fmt(
            &self
                .node
                .control_properties
                .display_with_cfg_and_indent(self.cfg, self.indent.make_more()),
            f,
        )?;
        for child in self.node.children.iter() {
            fmt::Display::fmt(
                &child.display_with_cfg_and_indent(self.cfg, self.indent.make_more()),
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
            .field("parent_node_and_index", &self.node.parent_node_and_index)
            .field("nesting_depth", &self.node.nesting_depth)
            .field(
                "control_properties",
                &self.node.control_properties.display_with_cfg(self.cfg),
            )
            .field("children", &self.node.children.display_with_cfg(self.cfg))
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeAndIndex {
    pub node: Rc<Node>,
    pub index: usize,
}

#[derive(Clone, Debug, Default)]
pub struct WeakNodeAndIndex {
    node: Weak<Node>,
    index: usize,
}

impl WeakNodeAndIndex {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn upgrade(&self) -> Option<NodeAndIndex> {
        Some(NodeAndIndex {
            node: self.node.upgrade()?,
            index: self.index,
        })
    }
}

impl From<&'_ NodeAndIndex> for WeakNodeAndIndex {
    fn from(node_and_index: &NodeAndIndex) -> Self {
        Self {
            node: Rc::downgrade(&node_and_index.node),
            index: node_and_index.index,
        }
    }
}

impl From<&'_ mut NodeAndIndex> for WeakNodeAndIndex {
    fn from(node_and_index: &mut NodeAndIndex) -> Self {
        Self::from(&*node_and_index)
    }
}

impl From<NodeAndIndex> for WeakNodeAndIndex {
    fn from(node_and_index: NodeAndIndex) -> Self {
        Self::from(&node_and_index)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Child {
    Node(Rc<Node>),
    BasicBlock(CFGNodeIndex),
}

impl Child {
    pub fn node_kind(&self) -> Option<NodeKind> {
        match self {
            Child::Node(node) => Some(node.kind()),
            Child::BasicBlock(_) => None,
        }
    }
}

pub struct ChildDisplay<'a> {
    child: &'a Child,
    cfg: &'a CFGGraph,
    indent: Indent,
}

impl fmt::Display for ChildDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.child {
            Child::Node(node) => {
                fmt::Display::fmt(&node.display_with_cfg_and_indent(self.cfg, self.indent), f)
            }
            &Child::BasicBlock(node_index) => writeln!(
                f,
                "{}{}",
                self.indent,
                node_index.display_with_cfg(self.cfg),
            ),
        }
    }
}

impl fmt::Debug for ChildDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.child {
            Child::Node(node) => f
                .debug_tuple("Node")
                .field(&node.display_with_cfg(self.cfg))
                .finish(),
            &Child::BasicBlock(node_index) => f
                .debug_tuple("BasicBlock")
                .field(&node_index.display_with_cfg(self.cfg))
                .finish(),
        }
    }
}

impl<'a> DisplayWithCFG<'a> for Child {
    type DisplayType = HandleIsDebugWrapper<ChildDisplay<'a>>;
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        HandleIsDebugWrapper {
            value: ChildDisplay {
                child: self,
                cfg,
                indent,
            },
            is_debug,
        }
    }
}

#[derive(Clone, Debug)]
pub struct StructureTree(Rc<Node>);

impl<'a> DisplayWithCFG<'a> for StructureTree {
    type DisplayType = <Node as DisplayWithCFG<'a>>::DisplayType;
    fn display_with_cfg_and_indent_and_is_debug(
        &'a self,
        cfg: &'a CFGGraph,
        indent: Indent,
        is_debug: Option<bool>,
    ) -> Self::DisplayType {
        self.root()
            .display_with_cfg_and_indent_and_is_debug(cfg, indent, is_debug)
    }
}

impl StructureTree {
    pub fn root(&self) -> &Rc<Node> {
        &self.0
    }
    pub fn children_recursive(&self) -> ChildrenRecursiveIter {
        ChildrenRecursiveIter(vec![self.root().children().iter()])
    }
    pub fn basic_blocks_in_order(&self) -> BasicBlocksInOrderIter {
        BasicBlocksInOrderIter(self.children_recursive())
    }
    pub(super) fn parse(
        graph: &mut CFGGraph,
        label_to_node_index_map: &HashMap<IdRef, CFGNodeIndex>,
        root: CFGNodeIndex,
    ) -> Self {
        let mut parser = Parser {
            graph,
            label_to_node_index_map,
        };
        let ParseChildrenResult {
            children,
            control_properties,
        } = parser.parse_children(root, &[], &HashSet::new(), 1);
        assert!(control_properties.exit_targets().is_empty());
        let root = parser.new_node(NodeKind::Root, children, 0, control_properties);
        StructureTree(root)
    }
}

#[derive(Debug)]
struct Parser<'a> {
    graph: &'a mut CFGGraph,
    label_to_node_index_map: &'a HashMap<IdRef, CFGNodeIndex>,
}

struct ParseChildResult {
    child: Child,
    control_properties: NodeControlProperties,
}

struct ParseChildrenResult {
    children: Vec<Child>,
    control_properties: NodeControlProperties,
}

impl Parser<'_> {
    fn new_node(
        &mut self,
        kind: NodeKind,
        children: Vec<Child>,
        nesting_depth: usize,
        control_properties: NodeControlProperties,
    ) -> Rc<Node> {
        let first_basic_block = match *children.first().expect("Node has no children") {
            Child::Node(ref node) => node.first_basic_block,
            Child::BasicBlock(basic_block) => basic_block,
        };
        let retval = Rc::new(Node {
            kind,
            parent_node_and_index: RefCell::default(),
            children,
            nesting_depth,
            first_basic_block,
            root: RefCell::default(),
            control_properties,
        });
        match kind {
            NodeKind::Root => assert_eq!(nesting_depth, 0),
            NodeKind::If => {
                assert!(retval.children.len() <= 3);
                assert_eq!(retval.children[0].node_kind(), None);
                assert!(retval
                    .children
                    .iter()
                    .skip(1)
                    .all(|child| child.node_kind() == Some(NodeKind::IfPart)));
            }
            NodeKind::Loop => {
                let mut children = retval.children.iter().peekable();
                assert_eq!(
                    children.next().expect("Node has no children").node_kind(),
                    None
                );
                match children.peek() {
                    Some(Child::Node(node)) if node.kind() == NodeKind::LoopBody => {
                        let _ = children.next();
                    }
                    _ => {}
                }
                match children.peek() {
                    Some(Child::Node(node)) if node.kind() == NodeKind::Continue => {
                        let _ = children.next();
                    }
                    _ => {}
                }
                assert!(children.peek().is_none());
            }
            NodeKind::Switch => {
                assert_eq!(retval.children[0].node_kind(), None);
                assert!(retval
                    .children
                    .iter()
                    .skip(1)
                    .all(|child| child.node_kind() == Some(NodeKind::Case)));
            }
            _ => {}
        }
        for (index, child) in retval.children.iter().enumerate() {
            match child {
                Child::Node(node) => {
                    assert_eq!(node.nesting_depth(), nesting_depth + 1);
                    node.set_parent_node_and_index(&NodeAndIndex {
                        node: retval.clone(),
                        index,
                    });
                }
                Child::BasicBlock(basic_block) => {
                    self.graph[*basic_block].set_parent_structure_tree_node_and_index(
                        &NodeAndIndex {
                            node: retval.clone(),
                            index,
                        },
                    );
                }
            }
        }
        retval
    }
    fn parse_nonempty_children(
        &mut self,
        basic_block: CFGNodeIndex,
        exit_targets: &HashSet<CFGNodeIndex>,
        nesting_depth: usize,
    ) -> ParseChildrenResult {
        assert!(!exit_targets.contains(&basic_block));
        let mut target = basic_block;
        let mut children = Vec::new();
        let mut control_properties = NodeControlProperties::new();
        loop {
            let ParseChildResult {
                child,
                control_properties: mut child_control_properties,
            } = self.parse_child(target, exit_targets, nesting_depth);
            children.push(child);
            let mut next_target = None;
            child_control_properties.exit_targets.retain(|&target, _| {
                if exit_targets.contains(&target) {
                    true
                } else {
                    assert_eq!(next_target, None, "too many non-exit targets");
                    next_target = Some(target);
                    false
                }
            });
            control_properties.merge_assign(child_control_properties);
            if let Some(next_target) = next_target {
                target = next_target;
            } else {
                break;
            }
        }
        ParseChildrenResult {
            children,
            control_properties,
        }
    }
    fn parse_empty_children_with_sources(
        &mut self,
        basic_block: CFGNodeIndex,
        basic_block_sources: &[CFGNodeIndex],
    ) -> ParseChildrenResult {
        ParseChildrenResult {
            children: Vec::new(),
            control_properties: NodeControlProperties::from_exit_edges(
                basic_block_sources.iter().map(|&source| {
                    self.graph
                        .find_edge(source, basic_block)
                        .expect("graph edge not found")
                }),
                self.graph,
            ),
        }
    }
    fn parse_children(
        &mut self,
        basic_block: CFGNodeIndex,
        basic_block_sources: &[CFGNodeIndex],
        exit_targets: &HashSet<CFGNodeIndex>,
        nesting_depth: usize,
    ) -> ParseChildrenResult {
        if exit_targets.contains(&basic_block) {
            self.parse_empty_children_with_sources(basic_block, basic_block_sources)
        } else {
            self.parse_nonempty_children(basic_block, exit_targets, nesting_depth)
        }
    }
    fn parse_simple_child(&mut self, basic_block: CFGNodeIndex) -> ParseChildResult {
        let terminating_instruction_properties = InstructionProperties::new(
            self.graph[basic_block]
                .instructions()
                .terminating_instruction()
                .unwrap(),
        );
        if terminating_instruction_properties.is_return_or_kill() {
            ParseChildResult {
                child: Child::BasicBlock(basic_block),
                control_properties: NodeControlProperties::from_return_or_kill(basic_block),
            }
        } else {
            ParseChildResult {
                child: Child::BasicBlock(basic_block),
                control_properties: NodeControlProperties::from_exit_edges(
                    self.graph
                        .edges_directed(basic_block, Outgoing)
                        .map(|v| v.id()),
                    self.graph,
                ),
            }
        }
    }
    fn parse_loop_child(
        &mut self,
        basic_block: CFGNodeIndex,
        exit_targets: &HashSet<CFGNodeIndex>,
        nesting_depth: usize,
        merge_target: CFGNodeIndex,
        continue_target: CFGNodeIndex,
    ) -> ParseChildResult {
        let mut exit_targets = exit_targets.clone();
        exit_targets.insert(merge_target);
        let mut body_exit_targets = exit_targets.clone();
        body_exit_targets.insert(continue_target);
        let mut loop_children = vec![Child::BasicBlock(basic_block)];
        let mut loop_control_properties = NodeControlProperties::new();
        for target in self
            .graph
            .neighbors_directed(basic_block, Outgoing)
            .collect::<Vec<_>>()
        {
            let ParseChildrenResult {
                children,
                control_properties,
            } = self.parse_children(
                target,
                &[basic_block],
                &body_exit_targets,
                nesting_depth + 2,
            );
            if !children.is_empty() {
                loop_children.push(Child::Node(self.new_node(
                    NodeKind::LoopBody,
                    children,
                    nesting_depth + 1,
                    control_properties.clone(),
                )));
            }
            loop_control_properties.merge_assign(control_properties);
        }
        if let Some(continue_edges) = loop_control_properties
            .exit_targets
            .remove(&continue_target)
        {
            let mut continue_exit_targets = exit_targets;
            continue_exit_targets.insert(basic_block);
            if continue_exit_targets.contains(&continue_target) {
                loop_control_properties
                    .append_exit_target_and_edges_vec(continue_target, continue_edges);
            } else {
                let ParseChildrenResult {
                    children,
                    control_properties,
                } = self.parse_nonempty_children(
                    continue_target,
                    &continue_exit_targets,
                    nesting_depth + 2,
                );
                loop_children.push(Child::Node(self.new_node(
                    NodeKind::Continue,
                    children,
                    nesting_depth + 1,
                    control_properties.clone(),
                )));
                loop_control_properties.merge_assign(control_properties);
            }
            loop_control_properties.exit_targets.remove(&basic_block);
        }
        ParseChildResult {
            child: Child::Node(self.new_node(
                NodeKind::Loop,
                loop_children,
                nesting_depth,
                loop_control_properties.clone(),
            )),
            control_properties: loop_control_properties,
        }
    }
    fn parse_switch_child(
        &mut self,
        basic_block: CFGNodeIndex,
        exit_targets: &HashSet<CFGNodeIndex>,
        merge_target: CFGNodeIndex,
        default_target: CFGNodeIndex,
        case_targets: Vec<CFGNodeIndex>,
        nesting_depth: usize,
    ) -> ParseChildResult {
        let mut exit_targets = exit_targets.clone();
        exit_targets.insert(merge_target);
        struct Case {
            node: Option<Rc<Node>>,
            case_target_and_edges: Option<(CFGNodeIndex, Vec<CFGEdgeIndex>)>,
            next_case_index: Option<usize>,
            prev_case_index: Option<usize>,
        }
        let get_case_targets = || {
            case_targets
                .iter()
                .cloned()
                .chain(iter::once(default_target))
        };
        let mut switch_control_properties = NodeControlProperties::new();
        let mut cases = Vec::new();
        let mut case_map = HashMap::new();
        let mut handled_cases_set = HashSet::new();
        for case_target in
            get_case_targets().filter(move |&case_target| handled_cases_set.insert(case_target))
        {
            if exit_targets.contains(&case_target) {
                switch_control_properties.append_exit_target_and_edge(
                    case_target,
                    self.graph
                        .find_edge(basic_block, case_target)
                        .expect("graph edge not found"),
                );
            } else {
                let mut case_exit_targets = exit_targets.clone();
                case_exit_targets.extend(get_case_targets().filter(|&v| v != case_target));
                let ParseChildrenResult {
                    children,
                    mut control_properties,
                } = self.parse_nonempty_children(
                    case_target,
                    &case_exit_targets,
                    nesting_depth + 2,
                );
                let node = Some(self.new_node(
                    NodeKind::Case,
                    children,
                    nesting_depth + 1,
                    control_properties.clone(),
                ));
                let mut case_target_and_edges = None;
                for (exit_target, exit_edges) in control_properties.exit_targets.drain() {
                    if exit_targets.contains(&exit_target) {
                        switch_control_properties
                            .append_exit_target_and_edges_vec(exit_target, exit_edges);
                    } else {
                        assert!(
                            case_target_and_edges.is_none(),
                            "case branches to more than one other case"
                        );
                        case_target_and_edges = Some((exit_target, exit_edges));
                    }
                }
                switch_control_properties.merge_assign(control_properties);
                case_map.insert(case_target, cases.len());
                cases.push(Case {
                    node,
                    case_target_and_edges,
                    prev_case_index: None,
                    next_case_index: None,
                });
            }
        }
        for case_index in 0..cases.len() {
            if let Some((case_target, _)) = cases[case_index].case_target_and_edges {
                let target_case_index = case_map[&case_target];
                assert!(cases[target_case_index].prev_case_index.is_none());
                cases[target_case_index].prev_case_index = Some(case_index);
                cases[case_index].next_case_index = Some(target_case_index);
            }
        }
        let mut switch_children = Vec::with_capacity(1 + cases.len());
        switch_children.push(Child::BasicBlock(basic_block));
        for case_index in 0..cases.len() {
            if cases[case_index].node.is_none() {
                continue;
            }
            let mut case_index = case_index;
            for _ in 0..cases.len() {
                if let Some(prev_case_index) = cases[case_index].prev_case_index {
                    case_index = prev_case_index;
                } else {
                    break;
                }
            }
            assert!(
                cases[case_index].prev_case_index.is_none(),
                "switch children form a loop"
            );
            loop {
                switch_children.push(Child::Node(
                    cases[case_index].node.take().expect("missing switch child"),
                ));
                if let Some(next_case_index) = cases[case_index].next_case_index {
                    case_index = next_case_index;
                } else {
                    break;
                }
            }
        }
        ParseChildResult {
            child: Child::Node(self.new_node(
                NodeKind::Switch,
                switch_children,
                nesting_depth,
                switch_control_properties.clone(),
            )),
            control_properties: switch_control_properties,
        }
    }
    fn parse_conditional_child(
        &mut self,
        basic_block: CFGNodeIndex,
        exit_targets: &HashSet<CFGNodeIndex>,
        nesting_depth: usize,
        merge_target: CFGNodeIndex,
        true_target: CFGNodeIndex,
        false_target: CFGNodeIndex,
    ) -> ParseChildResult {
        let mut exit_targets = exit_targets.clone();
        exit_targets.insert(merge_target);
        let mut parse_if_part =
            |if_target: CFGNodeIndex| -> (Option<Child>, NodeControlProperties) {
                let ParseChildrenResult {
                    children,
                    control_properties,
                } = self.parse_children(
                    if_target,
                    &[basic_block],
                    &exit_targets,
                    nesting_depth + 2,
                );
                let child = if children.is_empty() {
                    None
                } else {
                    Some(Child::Node(self.new_node(
                        NodeKind::IfPart,
                        children,
                        nesting_depth + 1,
                        control_properties.clone(),
                    )))
                };
                (child, control_properties)
            };
        let (true_child, true_control_properties) = parse_if_part(true_target);
        let (false_child, false_control_properties) = parse_if_part(false_target);
        let children = iter::once(Child::BasicBlock(basic_block))
            .chain(true_child)
            .chain(false_child)
            .collect();
        let control_properties = true_control_properties.merge(false_control_properties);
        ParseChildResult {
            child: Child::Node(self.new_node(
                NodeKind::If,
                children,
                nesting_depth,
                control_properties.clone(),
            )),
            control_properties,
        }
    }
    fn parse_selection_child(
        &mut self,
        basic_block: CFGNodeIndex,
        exit_targets: &HashSet<CFGNodeIndex>,
        nesting_depth: usize,
        merge_target: CFGNodeIndex,
    ) -> ParseChildResult {
        match *self.graph[basic_block]
            .instructions()
            .terminating_instruction()
            .expect("missing terminating instruction")
        {
            Instruction::Switch32 {
                default: default_id,
                target: ref case_ids,
                ..
            } => {
                let case_node_indexes = case_ids
                    .iter()
                    .map(|(_, v)| self.label_to_node_index_map[v])
                    .collect();
                self.parse_switch_child(
                    basic_block,
                    exit_targets,
                    merge_target,
                    self.label_to_node_index_map[&default_id],
                    case_node_indexes,
                    nesting_depth,
                )
            }
            Instruction::Switch64 {
                default: default_id,
                target: ref case_ids,
                ..
            } => {
                let case_node_indexes = case_ids
                    .iter()
                    .map(|(_, v)| self.label_to_node_index_map[v])
                    .collect();
                self.parse_switch_child(
                    basic_block,
                    exit_targets,
                    merge_target,
                    self.label_to_node_index_map[&default_id],
                    case_node_indexes,
                    nesting_depth,
                )
            }
            Instruction::BranchConditional {
                true_label: true_id,
                false_label: false_id,
                ..
            } => self.parse_conditional_child(
                basic_block,
                exit_targets,
                nesting_depth,
                merge_target,
                self.label_to_node_index_map[&true_id],
                self.label_to_node_index_map[&false_id],
            ),
            Instruction::Branch { .. }
            | Instruction::Kill
            | Instruction::Return
            | Instruction::ReturnValue { .. }
            | Instruction::Unreachable => unreachable!(
                "unexpected terminating instruction after OpSelectionMerge:\n{}",
                self.graph[basic_block]
                    .instructions()
                    .terminating_instruction()
                    .unwrap()
            ),
            _ => unreachable!(),
        }
    }
    fn parse_child(
        &mut self,
        basic_block: CFGNodeIndex,
        exit_targets: &HashSet<CFGNodeIndex>,
        nesting_depth: usize,
    ) -> ParseChildResult {
        match self.graph[basic_block].instructions().merge_instruction() {
            None => self.parse_simple_child(basic_block),
            Some(&Instruction::LoopMerge {
                merge_block: merge_id,
                continue_target: continue_id,
                ..
            }) => self.parse_loop_child(
                basic_block,
                exit_targets,
                nesting_depth,
                self.label_to_node_index_map[&merge_id],
                self.label_to_node_index_map[&continue_id],
            ),
            Some(&Instruction::SelectionMerge {
                merge_block: merge_id,
                ..
            }) => self.parse_selection_child(
                basic_block,
                exit_targets,
                nesting_depth,
                self.label_to_node_index_map[&merge_id],
            ),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChildrenRecursiveIter<'a>(Vec<std::slice::Iter<'a, Child>>);

impl<'a> Iterator for ChildrenRecursiveIter<'a> {
    type Item = &'a Child;
    fn next(&mut self) -> Option<&'a Child> {
        while let Some(mut iter) = self.0.pop() {
            if let Some(child) = iter.next() {
                self.0.push(iter);
                if let Child::Node(node) = child {
                    self.0.push(node.children().iter());
                }
                return Some(child);
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct BasicBlocksInOrderIter<'a>(ChildrenRecursiveIter<'a>);

impl Iterator for BasicBlocksInOrderIter<'_> {
    type Item = CFGNodeIndex;
    fn next(&mut self) -> Option<CFGNodeIndex> {
        while let Some(child) = self.0.next() {
            if let Child::BasicBlock(basic_block) = *child {
                return Some(basic_block);
            }
        }
        None
    }
}
