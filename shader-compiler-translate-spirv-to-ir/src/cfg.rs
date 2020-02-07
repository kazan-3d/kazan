// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    errors::{SPIRVIdAlreadyDefined, SPIRVIdNotDefined, TranslationResult},
    structure_tree::{NodeAndIndex, StructureTree},
    SPIRVInstructionLocation,
};
use alloc::vec::Vec;
use core::{
    convert::{TryFrom, TryInto},
    fmt, iter,
    marker::PhantomData,
    ops::{Deref, DerefMut, Index, Range},
    slice,
};
use fixedbitset::FixedBitSet;
use once_cell::unsync::OnceCell;
use petgraph::{
    algo::dominators,
    dot::Dot,
    visit::{
        Data, EdgeRef, GraphBase, GraphProp, IntoEdgeReferences, IntoEdges, IntoEdgesDirected,
        IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers, IntoNodeReferences,
        NodeCompactIndexable, NodeCount, NodeIndexable, VisitMap, Visitable,
    },
    Direction,
};
use spirv_id_map::{IdMap, ToIdBound};
use spirv_parser::{
    IdRef, IdResult, Instruction, OpBranch, OpBranchConditional, OpKill, OpLabel, OpLoopMerge,
    OpReturn, OpReturnValue, OpSelectionMerge, OpSwitch32, OpSwitch64, OpUnreachable,
};

macro_rules! impl_instruction_subset {
    (
        $vis:vis enum $enum_name:ident {
            $($name:ident($ty:ty),)+
        }
    ) => {
        #[derive(Clone, Debug)]
        $vis enum $enum_name {
            $($name($ty),)+
        }

        impl $enum_name {
            /// returns true if `instruction` is in the subset `Self`
            $vis fn is_in_subset(instruction: &Instruction) -> bool {
                match instruction {
                    $(Instruction::$name(_) => true,)+
                    _ => false,
                }
            }
        }

        impl TryFrom<Instruction> for $enum_name {
            type Error = Instruction;

            fn try_from(value: Instruction) -> Result<Self, Self::Error> {
                match value {
                    $(Instruction::$name(v) => Ok(Self::$name(v)),)+
                    _ => Err(value),
                }
            }
        }

        impl Into<Instruction> for $enum_name {
            fn into(self) -> Instruction {
                match self {
                    $(Self::$name(v) => Instruction::$name(v),)+
                }
            }
        }
    };
}

impl_instruction_subset! {
    pub(crate) enum TerminationInstruction {
        Branch(OpBranch),
        BranchConditional(OpBranchConditional),
        Switch32(OpSwitch32),
        Switch64(OpSwitch64),
        Kill(OpKill),
        Return(OpReturn),
        ReturnValue(OpReturnValue),
        Unreachable(OpUnreachable),
    }
}

enum TerminationInstructionTargetsImpl<'a> {
    Pair(IdRef, IdRef),
    Sequence32(Option<IdRef>, slice::Iter<'a, (u32, IdRef)>),
    Sequence64(Option<IdRef>, slice::Iter<'a, (u64, IdRef)>),
}

pub(crate) struct TerminationInstructionTargets<'a>(TerminationInstructionTargetsImpl<'a>);

impl<'a> Iterator for TerminationInstructionTargets<'a> {
    type Item = IdRef;
    fn next(&mut self) -> Option<IdRef> {
        self.nth(0)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        use TerminationInstructionTargetsImpl::*;
        match (&mut self.0, n) {
            (&mut Pair(retval, rest), 0) => {
                self.0 = Sequence32(Some(rest), [].iter());
                Some(retval)
            }
            (&mut Pair(_, retval), 1) => {
                self.0 = Sequence32(None, [].iter());
                Some(retval)
            }
            (Pair(_, _), _) => {
                self.0 = Sequence32(None, [].iter());
                None
            }
            (Sequence32(first @ Some(_), _), 0) => first.take(),
            (Sequence64(first @ Some(_), _), 0) => first.take(),
            (Sequence32(ref mut first @ Some(_), ref mut rest), _) => {
                *first = None;
                rest.nth(n - 1).map(|&(_, v)| v)
            }
            (Sequence64(ref mut first @ Some(_), ref mut rest), _) => {
                *first = None;
                rest.nth(n - 1).map(|&(_, v)| v)
            }
            (Sequence32(None, ref mut rest), _) => rest.nth(n).map(|&(_, v)| v),
            (Sequence64(None, ref mut rest), _) => rest.nth(n).map(|&(_, v)| v),
        }
    }
}

impl TerminationInstruction {
    pub(crate) fn get_targets(&self) -> TerminationInstructionTargets {
        use TerminationInstructionTargetsImpl::*;
        TerminationInstructionTargets(match *self {
            TerminationInstruction::Branch(OpBranch { target_label }) => {
                Sequence32(Some(target_label), [].iter())
            }
            TerminationInstruction::BranchConditional(OpBranchConditional {
                true_label,
                false_label,
                ..
            }) => Pair(true_label, false_label),
            TerminationInstruction::Kill(_)
            | TerminationInstruction::Return(_)
            | TerminationInstruction::ReturnValue(_)
            | TerminationInstruction::Unreachable(_) => Sequence32(None, [].iter()),
            TerminationInstruction::Switch32(OpSwitch32 {
                default,
                ref target,
                ..
            }) => Sequence32(Some(default), target.iter()),
            TerminationInstruction::Switch64(OpSwitch64 {
                default,
                ref target,
                ..
            }) => Sequence64(Some(default), target.iter()),
        })
    }
}

impl_instruction_subset! {
    pub(crate) enum MergeInstruction {
        LoopMerge(OpLoopMerge),
        SelectionMerge(OpSelectionMerge),
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct CFGBlockId(usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct CFGEdgeId {
    pub(crate) source_block: CFGBlockId,
    pub(crate) target_index: usize,
}

pub(crate) struct CFGBase<'g, 'i> {
    blocks: Vec<CFGBlock<'g, 'i>>,
    label_map: IdMap<IdRef, CFGBlockId>,
}

impl<'g, 'i> CFGBase<'g, 'i> {
    pub(crate) fn entry_block_id(&self) -> CFGBlockId {
        CFGBlockId(0)
    }
    pub(crate) fn iter(&self) -> slice::Iter<CFGBlock<'g, 'i>> {
        self.blocks.iter()
    }
    pub(crate) fn checked_label_id_to_block_id(
        &self,
        label_id: IdRef,
    ) -> TranslationResult<Option<CFGBlockId>> {
        Ok(self.label_map.get(label_id)?.copied())
    }
    /// convert a known-valid label id to the corresponding `CFGBlockId`
    ///
    /// # Panics
    ///
    /// panics when the label is invalid
    pub(crate) fn label_id_to_block_id(&self, label_id: IdRef) -> CFGBlockId {
        self.checked_label_id_to_block_id(label_id)
            .expect("invalid label_id")
            .expect("invalid label_id")
    }
    pub(crate) fn insert(&mut self, block: CFGBlock<'g, 'i>) -> TranslationResult<CFGBlockId> {
        let block_id = CFGBlockId(self.blocks.len());
        if let spirv_id_map::Vacant(entry) = self.label_map.entry(block.label_id())? {
            entry.insert(block_id);
            self.blocks.push(block);
            Ok(block_id)
        } else {
            Err(SPIRVIdAlreadyDefined {
                id_result: IdResult(block.label_id()),
            }
            .into())
        }
    }
}

impl<'g, 'i> Index<CFGBlockId> for CFGBase<'g, 'i> {
    type Output = CFGBlock<'g, 'i>;
    fn index(&self, index: CFGBlockId) -> &Self::Output {
        &self.blocks[index.0]
    }
}

pub(crate) struct CFGBuilder<'g, 'i> {
    base: CFGBase<'g, 'i>,
}

impl<'g, 'i> Deref for CFGBuilder<'g, 'i> {
    type Target = CFGBase<'g, 'i>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'g, 'i> DerefMut for CFGBuilder<'g, 'i> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<'g, 'i> CFGBuilder<'g, 'i> {
    pub(crate) fn new<T: ToIdBound>(
        entry_block: CFGBlock<'g, 'i>,
        id_bound: T,
    ) -> TranslationResult<Self> {
        let mut retval = Self {
            base: CFGBase {
                blocks: Vec::new(),
                label_map: IdMap::new(id_bound),
            },
        };
        retval.insert(entry_block)?;
        Ok(retval)
    }
    pub(crate) fn into_cfg(self) -> TranslationResult<CFG<'g, 'i>> {
        let CFGBuilder { base } = self;
        let mut block_source_edges: Vec<Vec<CFGEdgeId>> = base.iter().map(|_| Vec::new()).collect();
        for (source_block_id, source_block) in base.iter().enumerate() {
            for (target_index, target_label) in source_block
                .termination_instruction()
                .get_targets()
                .enumerate()
            {
                let target_block = base
                    .checked_label_id_to_block_id(target_label)?
                    .ok_or_else(|| SPIRVIdNotDefined { id: target_label })?;
                block_source_edges[target_block.0].push(CFGEdgeId {
                    source_block: CFGBlockId(source_block_id),
                    target_index,
                });
            }
        }
        let mut retval = CFG {
            base,
            block_source_edges,
            dominators: None,
            structure_tree: None,
        };
        retval.dominators = Some(dominators::simple_fast(&retval, retval.entry_block_id()));
        retval.structure_tree = Some(StructureTree::parse(&retval)?);
        Ok(retval)
    }
}

impl<'g, 'i, T> Index<T> for CFGBuilder<'g, 'i>
where
    CFGBase<'g, 'i>: Index<T>,
{
    type Output = <CFGBase<'g, 'i> as Index<T>>::Output;
    fn index(&self, index: T) -> &Self::Output {
        &self.base[index]
    }
}

/// immutable CFG that is known to have valid labels
pub(crate) struct CFG<'g, 'i> {
    base: CFGBase<'g, 'i>,
    block_source_edges: Vec<Vec<CFGEdgeId>>,
    dominators: Option<dominators::Dominators<CFGBlockId>>,
    structure_tree: Option<StructureTree>,
}

impl<'g, 'i> CFG<'g, 'i> {
    pub(crate) fn dominators(&self) -> &dominators::Dominators<CFGBlockId> {
        self.dominators
            .as_ref()
            .expect("filled by CFGBuilder::into_cfg, which is the only way to construct CFG")
    }
    pub(crate) fn structure_tree(&self) -> &StructureTree {
        self.structure_tree
            .as_ref()
            .expect("filled by CFGBuilder::into_cfg, which is the only way to construct CFG")
    }
    pub(crate) fn dump_to_dot<'a>(&'a self) -> impl fmt::Display + 'a {
        struct DisplayAsDebug<T: fmt::Debug>(T);
        impl<T: fmt::Debug> fmt::Display for DisplayAsDebug<T> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }
        DisplayAsDebug(Dot::new(self))
    }
}

impl<'g, 'i, T> Index<T> for CFG<'g, 'i>
where
    CFGBase<'g, 'i>: Index<T>,
{
    type Output = <CFGBase<'g, 'i> as Index<T>>::Output;
    fn index(&self, index: T) -> &Self::Output {
        &self.base[index]
    }
}

impl<'g, 'i> Deref for CFG<'g, 'i> {
    type Target = CFGBase<'g, 'i>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl GraphBase for CFG<'_, '_> {
    type NodeId = CFGBlockId;
    type EdgeId = CFGEdgeId;
}

impl GraphProp for CFG<'_, '_> {
    type EdgeType = petgraph::Directed;
}

impl<'g, 'i> Data for CFG<'g, 'i> {
    type NodeWeight = CFGBlock<'g, 'i>;
    type EdgeWeight = ();
}

impl NodeIndexable for CFG<'_, '_> {
    fn node_bound(&self) -> usize {
        self.blocks.len()
    }
    fn to_index(&self, v: CFGBlockId) -> usize {
        v.0
    }
    fn from_index(&self, v: usize) -> CFGBlockId {
        assert!(v < self.blocks.len());
        CFGBlockId(v)
    }
}

impl NodeCount for CFG<'_, '_> {
    fn node_count(&self) -> usize {
        self.blocks.len()
    }
}

impl NodeCompactIndexable for CFG<'_, '_> {}

pub(crate) struct CFGVisitMap(FixedBitSet);

impl VisitMap<CFGBlockId> for CFGVisitMap {
    fn visit(&mut self, id: CFGBlockId) -> bool {
        !self.0.put(id.0)
    }
    fn is_visited(&self, id: &CFGBlockId) -> bool {
        self.0.contains(id.0)
    }
}

impl<'g, 'i> Visitable for CFG<'g, 'i> {
    type Map = CFGVisitMap;
    fn visit_map(&self) -> Self::Map {
        CFGVisitMap(FixedBitSet::with_capacity(self.blocks.len()))
    }
    fn reset_map(&self, map: &mut Self::Map) {
        map.0.clear()
    }
}

pub(crate) struct CFGBlockSourceEdges<'a> {
    sources: slice::Iter<'a, CFGEdgeId>,
    target: CFGBlockId,
}

impl<'a> Iterator for CFGBlockSourceEdges<'a> {
    type Item = CFGEdgeRef;
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let id = *self.sources.nth(n)?;
        let target = self.target;
        Some(CFGEdgeRef { id, target })
    }
}

pub(crate) struct CFGBlockSourceBlockIds<'a>(CFGBlockSourceEdges<'a>);

impl<'a> Iterator for CFGBlockSourceBlockIds<'a> {
    type Item = CFGBlockId;
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.0.nth(n)?.source())
    }
}

pub(crate) struct CFGBlockTargetEdges<'a, 'g, 'i> {
    source_block: CFGBlockId,
    targets: iter::Enumerate<TerminationInstructionTargets<'a>>,
    cfg: &'a CFG<'g, 'i>,
}

impl<'a, 'g, 'i> Iterator for CFGBlockTargetEdges<'a, 'g, 'i> {
    type Item = CFGEdgeRef;
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let (target_index, label_id) = self.targets.nth(n)?;
        let target = self.cfg.label_id_to_block_id(label_id);
        Some(CFGEdgeRef {
            id: CFGEdgeId {
                source_block: self.source_block,
                target_index,
            },
            target,
        })
    }
}

pub(crate) struct CFGBlockTargetBlockIds<'a, 'g, 'i>(CFGBlockTargetEdges<'a, 'g, 'i>);

impl<'a, 'g, 'i> Iterator for CFGBlockTargetBlockIds<'a, 'g, 'i> {
    type Item = CFGBlockId;
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.0.nth(n)?.target())
    }
}

impl<'a, 'g, 'i> IntoNeighbors for &'a CFG<'g, 'i> {
    type Neighbors = CFGBlockTargetBlockIds<'a, 'g, 'i>;
    fn neighbors(self, source_block: Self::NodeId) -> Self::Neighbors {
        CFGBlockTargetBlockIds(self.edges(source_block))
    }
}

impl<'a, 'g, 'i> IntoEdges for &'a CFG<'g, 'i> {
    type Edges = CFGBlockTargetEdges<'a, 'g, 'i>;
    fn edges(self, source_block: Self::NodeId) -> Self::Edges {
        CFGBlockTargetEdges {
            source_block,
            targets: self[source_block]
                .termination_instruction()
                .get_targets()
                .enumerate(),
            cfg: self,
        }
    }
}

impl<'g, 'i> CFG<'g, 'i> {
    pub(crate) fn source_edges(&self, target: CFGBlockId) -> CFGBlockSourceEdges {
        CFGBlockSourceEdges {
            sources: self.block_source_edges[target.0].iter(),
            target,
        }
    }
    pub(crate) fn source_block_ids(&self, target: CFGBlockId) -> CFGBlockSourceBlockIds {
        CFGBlockSourceBlockIds(self.source_edges(target))
    }
    pub(crate) fn edge_ref(&self, id: CFGEdgeId) -> CFGEdgeRef {
        let target = self
            .neighbors(id.source_block)
            .nth(id.target_index)
            .expect("known to be valid target index");
        CFGEdgeRef { id, target }
    }
}

pub(crate) enum CFGBlockDirectedEdges<'a, 'g, 'i> {
    Source(CFGBlockSourceEdges<'a>),
    Target(CFGBlockTargetEdges<'a, 'g, 'i>),
}

impl<'a, 'g, 'i> Iterator for CFGBlockDirectedEdges<'a, 'g, 'i> {
    type Item = CFGEdgeRef;
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.nth(n),
            Self::Target(iter) => iter.nth(n),
        }
    }
}

impl<'a, 'g, 'i> IntoEdgesDirected for &'a CFG<'g, 'i> {
    type EdgesDirected = CFGBlockDirectedEdges<'a, 'g, 'i>;
    fn edges_directed(self, block_id: Self::NodeId, direction: Direction) -> Self::EdgesDirected {
        match direction {
            Direction::Incoming => CFGBlockDirectedEdges::Source(self.source_edges(block_id)),
            Direction::Outgoing => CFGBlockDirectedEdges::Target(self.edges(block_id)),
        }
    }
}

pub(crate) enum CFGBlockDirectedBlockIds<'a, 'g, 'i> {
    Source(CFGBlockSourceBlockIds<'a>),
    Target(CFGBlockTargetBlockIds<'a, 'g, 'i>),
}

impl<'a, 'g, 'i> Iterator for CFGBlockDirectedBlockIds<'a, 'g, 'i> {
    type Item = CFGBlockId;
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.nth(n),
            Self::Target(iter) => iter.nth(n),
        }
    }
}

impl<'a, 'g, 'i> IntoNeighborsDirected for &'a CFG<'g, 'i> {
    type NeighborsDirected = CFGBlockDirectedBlockIds<'a, 'g, 'i>;
    fn neighbors_directed(
        self,
        block_id: Self::NodeId,
        direction: Direction,
    ) -> Self::NeighborsDirected {
        match direction {
            Direction::Incoming => {
                CFGBlockDirectedBlockIds::Source(self.source_block_ids(block_id))
            }
            Direction::Outgoing => CFGBlockDirectedBlockIds::Target(self.neighbors(block_id)),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct CFGEdgeRef {
    id: CFGEdgeId,
    target: CFGBlockId,
}

impl EdgeRef for CFGEdgeRef {
    type NodeId = CFGBlockId;
    type EdgeId = CFGEdgeId;
    type Weight = ();
    fn source(&self) -> Self::NodeId {
        self.id.source_block
    }
    fn target(&self) -> Self::NodeId {
        self.target
    }
    fn weight(&self) -> &Self::Weight {
        &()
    }
    fn id(&self) -> Self::EdgeId {
        self.id
    }
}

pub(crate) struct CFGBlockIds {
    iter: Range<usize>,
}

impl Iterator for CFGBlockIds {
    type Item = CFGBlockId;
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.iter.nth(n).map(CFGBlockId)
    }
}

impl IntoNodeIdentifiers for &'_ CFG<'_, '_> {
    type NodeIdentifiers = CFGBlockIds;
    fn node_identifiers(self) -> Self::NodeIdentifiers {
        CFGBlockIds {
            iter: 0..self.blocks.len(),
        }
    }
}

pub(crate) struct CFGEdgeReferences<'a, 'g, 'i> {
    block_ids: CFGBlockIds,
    block_edges: Option<CFGBlockTargetEdges<'a, 'g, 'i>>,
    cfg: &'a CFG<'g, 'i>,
}

impl<'a, 'g, 'i> Iterator for CFGEdgeReferences<'a, 'g, 'i> {
    type Item = CFGEdgeRef;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(block_edges) = &mut self.block_edges {
                if let Some(retval) = block_edges.next() {
                    return Some(retval);
                } else {
                    self.block_edges = None;
                }
            }
            if let Some(block_id) = self.block_ids.next() {
                self.block_edges = Some(self.cfg.edges(block_id));
            } else {
                return None;
            }
        }
    }
}

impl<'a, 'g, 'i> IntoEdgeReferences for &'a CFG<'g, 'i> {
    type EdgeRef = CFGEdgeRef;
    type EdgeReferences = CFGEdgeReferences<'a, 'g, 'i>;
    fn edge_references(self) -> Self::EdgeReferences {
        CFGEdgeReferences {
            block_ids: self.node_identifiers(),
            block_edges: None,
            cfg: self,
        }
    }
}

pub(crate) struct CFGNodeReferences<'a, 'g, 'i> {
    block_ids: CFGBlockIds,
    cfg: &'a CFG<'g, 'i>,
}

impl<'a, 'g, 'i> Iterator for CFGNodeReferences<'a, 'g, 'i> {
    type Item = (CFGBlockId, &'a CFGBlock<'g, 'i>);
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let block_id = self.block_ids.nth(n)?;
        Some((block_id, &self.cfg[block_id]))
    }
}

impl<'a, 'g, 'i> IntoNodeReferences for &'a CFG<'g, 'i> {
    type NodeRef = (CFGBlockId, &'a CFGBlock<'g, 'i>);
    type NodeReferences = CFGNodeReferences<'a, 'g, 'i>;
    fn node_references(self) -> Self::NodeReferences {
        CFGNodeReferences {
            block_ids: self.node_identifiers(),
            cfg: self,
        }
    }
}

pub(crate) struct CFGBlock<'g, 'i> {
    label_location: SPIRVInstructionLocation<'i>,
    label_instruction: &'i OpLabel,
    merge: Option<(SPIRVInstructionLocation<'i>, MergeInstruction)>,
    termination_location: SPIRVInstructionLocation<'i>,
    termination_instruction: TerminationInstruction,
    parent_structure_tree_node_and_index: OnceCell<NodeAndIndex>,
    _phantom: PhantomData<&'g ()>,
}

impl<'g, 'i> CFGBlock<'g, 'i> {
    pub(crate) fn new(
        label_location: SPIRVInstructionLocation<'i>,
        merge_location: Option<SPIRVInstructionLocation<'i>>,
        termination_location: SPIRVInstructionLocation<'i>,
    ) -> Self {
        let label_instruction = match label_location.get_instruction() {
            Some(Instruction::Label(v)) => v,
            _ => unreachable!("label_location is not at a OpLabel"),
        };
        let merge = merge_location.map(|location| {
            let merge_instruction = location
                .get_instruction()
                .expect("merge_location is not at a merge instruction")
                .clone()
                .try_into()
                .expect("merge_location is not at a merge instruction");
            (location, merge_instruction)
        });
        let termination_instruction = termination_location
            .get_instruction()
            .expect("termination_location is not at a block termination instruction")
            .clone()
            .try_into()
            .expect("termination_location is not at a block termination instruction");
        Self {
            label_location,
            label_instruction,
            merge,
            termination_location,
            termination_instruction,
            parent_structure_tree_node_and_index: OnceCell::new(),
            _phantom: PhantomData,
        }
    }
    pub(crate) fn label_location(&self) -> SPIRVInstructionLocation<'i> {
        self.label_location.clone()
    }
    pub(crate) fn label_instruction(&self) -> &'i OpLabel {
        self.label_instruction
    }
    pub(crate) fn label_id(&self) -> IdRef {
        self.label_instruction().id_result.0
    }
    pub(crate) fn merge_location(&self) -> Option<SPIRVInstructionLocation<'i>> {
        Some(self.merge.as_ref()?.0.clone())
    }
    pub(crate) fn merge_instruction(&self) -> Option<&MergeInstruction> {
        Some(&self.merge.as_ref()?.1)
    }
    pub(crate) fn termination_instruction(&self) -> &TerminationInstruction {
        &self.termination_instruction
    }
    pub(crate) fn termination_location(&self) -> SPIRVInstructionLocation<'i> {
        self.termination_location.clone()
    }
    pub(crate) fn set_parent_structure_tree_node_and_index(&self, node_and_index: NodeAndIndex) {
        #[allow(clippy::ok_expect)]
        self.parent_structure_tree_node_and_index
            .set(node_and_index)
            .ok()
            .expect("parent_structure_tree_node_and_index already set");
    }
    pub(crate) fn parent_structure_tree_node_and_index(&self) -> &NodeAndIndex {
        self.parent_structure_tree_node_and_index
            .get()
            .expect("parent_structure_tree_node_and_index not set")
    }
}

impl<'g, 'i> fmt::Debug for CFGBlock<'g, 'i> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.label_location())?;
        writeln!(f, "...")?;
        if let Some(merge_location) = self.merge_location() {
            write!(f, "{:?}", merge_location)?;
        }
        write!(f, "{:?}", self.termination_location())
    }
}
