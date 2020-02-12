// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    cfg::{CFGBlockId, CFGEdgeId, MergeInstruction, TerminationInstruction, CFG},
    errors::{
        InvalidTerminationInstructionFollowingMergeInstruction, SwitchCaseBranchesToMultipleCases,
        SwitchCasesFormALoop, TranslationResult,
    },
};
use alloc::{
    rc::{Rc, Weak},
    vec::Vec,
};
use core::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    hash::{Hash, Hasher},
    iter, slice,
};
use hashbrown::{HashMap, HashSet};
use petgraph::visit::{EdgeRef, IntoEdges, IntoNeighbors};
use spirv_parser::{OpBranchConditional, OpLoopMerge, OpSelectionMerge, OpSwitch32, OpSwitch64};

#[derive(Clone, Debug)]
pub(crate) struct NodeControlProperties {
    exit_targets: HashMap<CFGBlockId, Vec<CFGEdgeId>>,
    return_or_kill_blocks: Vec<CFGBlockId>,
}

impl NodeControlProperties {
    pub(crate) fn exit_edges(&self) -> impl Iterator<Item = CFGEdgeId> + Clone + '_ {
        self.exit_targets
            .iter()
            .flat_map(|(_, edges)| edges)
            .copied()
    }
    pub(crate) fn exit_targets(&self) -> &HashMap<CFGBlockId, Vec<CFGEdgeId>> {
        &self.exit_targets
    }
    pub(crate) fn return_or_kill_blocks(&self) -> &[CFGBlockId] {
        &self.return_or_kill_blocks
    }
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn from_return_or_kill(block: CFGBlockId) -> Self {
        Self {
            exit_targets: HashMap::new(),
            return_or_kill_blocks: vec![block],
        }
    }
    pub(crate) fn from_exit_edge(exit_edge: CFGEdgeId, cfg: &CFG<'_, '_>) -> Self {
        Self::from_exit_edges(iter::once(exit_edge), cfg)
    }
    pub(crate) fn from_exit_edges<I: IntoIterator>(exit_edges: I, cfg: &CFG<'_, '_>) -> Self
    where
        I::Item: Borrow<CFGEdgeId>,
    {
        let mut exit_targets: HashMap<_, Vec<_>> = HashMap::new();
        for exit_edge in exit_edges {
            let exit_edge = *exit_edge.borrow();
            exit_targets
                .entry(cfg.edge_ref(exit_edge).target())
                .or_default()
                .push(exit_edge);
        }
        Self {
            exit_targets,
            return_or_kill_blocks: Vec::new(),
        }
    }
    fn append_exit_target_and_edges_vec<T: BorrowMut<Vec<CFGEdgeId>>>(
        &mut self,
        exit_target: CFGBlockId,
        mut exit_edges: T,
    ) {
        self.exit_targets
            .entry(exit_target)
            .or_default()
            .append(exit_edges.borrow_mut());
    }
    fn append_exit_target_and_edges(
        &mut self,
        exit_target: CFGBlockId,
        exit_edges: impl IntoIterator<Item = CFGEdgeId>,
    ) {
        self.exit_targets
            .entry(exit_target)
            .or_default()
            .extend(exit_edges);
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum NodeKind {
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
pub(crate) struct Node {
    kind: NodeKind,
    parent_node_and_index: RefCell<WeakNodeAndIndex>,
    children: Vec<Child>,
    nesting_depth: usize,
    first_basic_block: CFGBlockId,
    root: RefCell<Weak<Node>>,
    control_properties: NodeControlProperties,
}

impl Node {
    pub(crate) fn kind(&self) -> NodeKind {
        self.kind
    }
    pub(crate) fn parent_node_and_index(&self) -> Option<NodeAndIndex> {
        self.parent_node_and_index.borrow().upgrade()
    }
    pub(crate) fn children(&self) -> &Vec<Child> {
        &self.children
    }
    pub(crate) fn nesting_depth(&self) -> usize {
        self.nesting_depth
    }
    pub(crate) fn first_basic_block(&self) -> CFGBlockId {
        self.first_basic_block
    }
    pub(crate) fn root(&self) -> Rc<Node> {
        self.root.borrow().upgrade().expect("missing root node")
    }
    pub(crate) fn control_properties(&self) -> &NodeControlProperties {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct NodeAndIndex {
    pub(crate) node: Rc<Node>,
    pub(crate) index: usize,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct WeakNodeAndIndex {
    node: Weak<Node>,
    index: usize,
}

impl WeakNodeAndIndex {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn upgrade(&self) -> Option<NodeAndIndex> {
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
pub(crate) enum Child {
    Node(Rc<Node>),
    BasicBlock(CFGBlockId),
}

impl Child {
    pub(crate) fn node_kind(&self) -> Option<NodeKind> {
        match self {
            Child::Node(node) => Some(node.kind()),
            Child::BasicBlock(_) => None,
        }
    }
    pub(crate) fn node(&self) -> Option<&Rc<Node>> {
        match self {
            Child::Node(v) => Some(v),
            _ => None,
        }
    }
    pub(crate) fn basic_block(&self) -> Option<CFGBlockId> {
        match self {
            Child::BasicBlock(v) => Some(*v),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StructureTree(Rc<Node>);

impl StructureTree {
    pub(crate) fn root(&self) -> &Rc<Node> {
        &self.0
    }
    pub(crate) fn children_recursive(&self) -> ChildrenRecursiveIter {
        ChildrenRecursiveIter(vec![self.root().children().iter()])
    }
    pub(crate) fn basic_blocks_in_order(&self) -> BasicBlocksInOrderIter {
        BasicBlocksInOrderIter(self.children_recursive())
    }
    pub(crate) fn parse(cfg: &CFG<'_, '_>) -> TranslationResult<Self> {
        let mut parser = Parser { cfg };
        let ParseChildrenResult {
            children,
            control_properties,
        } = parser.parse_children(cfg.entry_block_id(), &[], &HashSet::new(), 1)?;
        assert!(control_properties.exit_targets().is_empty());
        let root = parser.new_node(NodeKind::Root, children, 0, control_properties);
        Ok(StructureTree(root))
    }
}

struct Parser<'a, 'g, 'i> {
    cfg: &'a CFG<'g, 'i>,
}

struct ParseChildResult {
    child: Child,
    control_properties: NodeControlProperties,
}

struct ParseChildrenResult {
    children: Vec<Child>,
    control_properties: NodeControlProperties,
}

impl Parser<'_, '_, '_> {
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
                    self.cfg[*basic_block].set_parent_structure_tree_node_and_index(NodeAndIndex {
                        node: retval.clone(),
                        index,
                    });
                }
            }
        }
        retval
    }
    fn parse_nonempty_children(
        &mut self,
        basic_block: CFGBlockId,
        exit_targets: &HashSet<CFGBlockId>,
        nesting_depth: usize,
    ) -> TranslationResult<ParseChildrenResult> {
        assert!(!exit_targets.contains(&basic_block));
        let mut target = basic_block;
        let mut children = Vec::new();
        let mut control_properties = NodeControlProperties::new();
        loop {
            let ParseChildResult {
                child,
                control_properties: mut child_control_properties,
            } = self.parse_child(target, exit_targets, nesting_depth)?;
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
        Ok(ParseChildrenResult {
            children,
            control_properties,
        })
    }
    fn parse_empty_children_with_sources(
        &mut self,
        basic_block: CFGBlockId,
        basic_block_sources: &[CFGBlockId],
    ) -> ParseChildrenResult {
        ParseChildrenResult {
            children: Vec::new(),
            control_properties: NodeControlProperties::from_exit_edges(
                basic_block_sources.iter().flat_map(|&source| {
                    self.cfg.edges(source).flat_map(|edge| {
                        if edge.target() == basic_block {
                            Some(edge.id())
                        } else {
                            None
                        }
                    })
                }),
                self.cfg,
            ),
        }
    }
    fn parse_children(
        &mut self,
        basic_block: CFGBlockId,
        basic_block_sources: &[CFGBlockId],
        exit_targets: &HashSet<CFGBlockId>,
        nesting_depth: usize,
    ) -> TranslationResult<ParseChildrenResult> {
        if exit_targets.contains(&basic_block) {
            Ok(self.parse_empty_children_with_sources(basic_block, basic_block_sources))
        } else {
            self.parse_nonempty_children(basic_block, exit_targets, nesting_depth)
        }
    }
    fn parse_simple_child(&mut self, basic_block: CFGBlockId) -> ParseChildResult {
        match self.cfg[basic_block].termination_instruction() {
            TerminationInstruction::Kill(_)
            | TerminationInstruction::Return(_)
            | TerminationInstruction::ReturnValue(_) => ParseChildResult {
                child: Child::BasicBlock(basic_block),
                control_properties: NodeControlProperties::from_return_or_kill(basic_block),
            },
            TerminationInstruction::Branch(_)
            | TerminationInstruction::BranchConditional(_)
            | TerminationInstruction::Switch32(_)
            | TerminationInstruction::Switch64(_)
            | TerminationInstruction::Unreachable(_) => ParseChildResult {
                child: Child::BasicBlock(basic_block),
                control_properties: NodeControlProperties::from_exit_edges(
                    self.cfg.edges(basic_block).map(|v| v.id()),
                    self.cfg,
                ),
            },
        }
    }
    fn parse_loop_child(
        &mut self,
        basic_block: CFGBlockId,
        exit_targets: &HashSet<CFGBlockId>,
        nesting_depth: usize,
        merge_target: CFGBlockId,
        continue_target: CFGBlockId,
    ) -> TranslationResult<ParseChildResult> {
        let mut exit_targets = exit_targets.clone();
        exit_targets.insert(merge_target);
        let mut body_exit_targets = exit_targets.clone();
        body_exit_targets.insert(continue_target);
        let mut loop_children = vec![Child::BasicBlock(basic_block)];
        let mut loop_control_properties = NodeControlProperties::new();
        for target in self.cfg.neighbors(basic_block) {
            let ParseChildrenResult {
                children,
                control_properties,
            } = self.parse_children(
                target,
                &[basic_block],
                &body_exit_targets,
                nesting_depth + 2,
            )?;
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
                )?;
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
        Ok(ParseChildResult {
            child: Child::Node(self.new_node(
                NodeKind::Loop,
                loop_children,
                nesting_depth,
                loop_control_properties.clone(),
            )),
            control_properties: loop_control_properties,
        })
    }
    #[allow(clippy::too_many_arguments)]
    fn parse_switch_child(
        &mut self,
        basic_block: CFGBlockId,
        exit_targets: &HashSet<CFGBlockId>,
        merge_target: CFGBlockId,
        default_target: CFGBlockId,
        case_targets: Vec<CFGBlockId>,
        nesting_depth: usize,
        switch_instruction: &TerminationInstruction,
    ) -> TranslationResult<ParseChildResult> {
        let mut exit_targets = exit_targets.clone();
        exit_targets.insert(merge_target);
        struct Case {
            node: Option<Rc<Node>>,
            case_target_and_edges: Option<(CFGBlockId, Vec<CFGEdgeId>)>,
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
            get_case_targets().filter(|&case_target| handled_cases_set.insert(case_target))
        {
            if exit_targets.contains(&case_target) {
                switch_control_properties.append_exit_target_and_edges(
                    case_target,
                    self.cfg.edges(basic_block).flat_map(|edge| {
                        if edge.target() == case_target {
                            Some(edge.id())
                        } else {
                            None
                        }
                    }),
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
                )?;
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
                    } else if case_target_and_edges.is_some() {
                        return Err(SwitchCaseBranchesToMultipleCases {
                            switch_instruction: switch_instruction.clone().into(),
                        }
                        .into());
                    } else {
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
            if cases[case_index].prev_case_index.is_some() {
                return Err(SwitchCasesFormALoop {
                    switch_instruction: switch_instruction.clone().into(),
                }
                .into());
            }
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
        Ok(ParseChildResult {
            child: Child::Node(self.new_node(
                NodeKind::Switch,
                switch_children,
                nesting_depth,
                switch_control_properties.clone(),
            )),
            control_properties: switch_control_properties,
        })
    }
    fn parse_conditional_child(
        &mut self,
        basic_block: CFGBlockId,
        exit_targets: &HashSet<CFGBlockId>,
        nesting_depth: usize,
        merge_target: CFGBlockId,
        true_target: CFGBlockId,
        false_target: CFGBlockId,
    ) -> TranslationResult<ParseChildResult> {
        let mut exit_targets = exit_targets.clone();
        exit_targets.insert(merge_target);
        let mut parse_if_part =
            |if_target: CFGBlockId| -> TranslationResult<(Option<Child>, NodeControlProperties)> {
                let ParseChildrenResult {
                    children,
                    control_properties,
                } = self.parse_children(
                    if_target,
                    &[basic_block],
                    &exit_targets,
                    nesting_depth + 2,
                )?;
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
                Ok((child, control_properties))
            };
        let (true_child, true_control_properties) = parse_if_part(true_target)?;
        let (false_child, false_control_properties) = parse_if_part(false_target)?;
        let children = iter::once(Child::BasicBlock(basic_block))
            .chain(true_child)
            .chain(false_child)
            .collect();
        let control_properties = true_control_properties.merge(false_control_properties);
        Ok(ParseChildResult {
            child: Child::Node(self.new_node(
                NodeKind::If,
                children,
                nesting_depth,
                control_properties.clone(),
            )),
            control_properties,
        })
    }
    fn parse_selection_child(
        &mut self,
        basic_block: CFGBlockId,
        exit_targets: &HashSet<CFGBlockId>,
        nesting_depth: usize,
        merge_target: CFGBlockId,
        selection_merge: &OpSelectionMerge,
    ) -> TranslationResult<ParseChildResult> {
        let termination_instruction = self.cfg[basic_block].termination_instruction();
        match *termination_instruction {
            TerminationInstruction::Switch32(OpSwitch32 {
                default: default_id,
                target: ref case_ids,
                ..
            }) => {
                let case_node_indexes = case_ids
                    .iter()
                    .map(|&(_, v)| self.cfg.label_id_to_block_id(v))
                    .collect();
                self.parse_switch_child(
                    basic_block,
                    exit_targets,
                    merge_target,
                    self.cfg.label_id_to_block_id(default_id),
                    case_node_indexes,
                    nesting_depth,
                    termination_instruction,
                )
            }
            TerminationInstruction::Switch64(OpSwitch64 {
                default: default_id,
                target: ref case_ids,
                ..
            }) => {
                let case_node_indexes = case_ids
                    .iter()
                    .map(|&(_, v)| self.cfg.label_id_to_block_id(v))
                    .collect();
                self.parse_switch_child(
                    basic_block,
                    exit_targets,
                    merge_target,
                    self.cfg.label_id_to_block_id(default_id),
                    case_node_indexes,
                    nesting_depth,
                    termination_instruction,
                )
            }
            TerminationInstruction::BranchConditional(OpBranchConditional {
                true_label: true_id,
                false_label: false_id,
                ..
            }) => self.parse_conditional_child(
                basic_block,
                exit_targets,
                nesting_depth,
                merge_target,
                self.cfg.label_id_to_block_id(true_id),
                self.cfg.label_id_to_block_id(false_id),
            ),
            TerminationInstruction::Branch(_)
            | TerminationInstruction::Kill(_)
            | TerminationInstruction::Return(_)
            | TerminationInstruction::ReturnValue(_)
            | TerminationInstruction::Unreachable(_) => {
                Err(InvalidTerminationInstructionFollowingMergeInstruction {
                    merge_instruction: selection_merge.clone().into(),
                    termination_instruction: termination_instruction.clone().into(),
                }
                .into())
            }
        }
    }
    fn parse_child(
        &mut self,
        basic_block: CFGBlockId,
        exit_targets: &HashSet<CFGBlockId>,
        nesting_depth: usize,
    ) -> TranslationResult<ParseChildResult> {
        match self.cfg[basic_block].merge_instruction() {
            None => Ok(self.parse_simple_child(basic_block)),
            Some(&MergeInstruction::LoopMerge(OpLoopMerge {
                merge_block: merge_id,
                continue_target: continue_id,
                ..
            })) => self.parse_loop_child(
                basic_block,
                exit_targets,
                nesting_depth,
                self.cfg.label_id_to_block_id(merge_id),
                self.cfg.label_id_to_block_id(continue_id),
            ),
            Some(MergeInstruction::SelectionMerge(selection_merge)) => self.parse_selection_child(
                basic_block,
                exit_targets,
                nesting_depth,
                self.cfg.label_id_to_block_id(selection_merge.merge_block),
                selection_merge,
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ChildrenRecursiveIter<'a>(Vec<slice::Iter<'a, Child>>);

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
pub(crate) struct BasicBlocksInOrderIter<'a>(ChildrenRecursiveIter<'a>);

impl Iterator for BasicBlocksInOrderIter<'_> {
    type Item = CFGBlockId;
    fn next(&mut self) -> Option<CFGBlockId> {
        while let Some(child) = self.0.next() {
            if let Child::BasicBlock(basic_block) = *child {
                return Some(basic_block);
            }
        }
        None
    }
}
