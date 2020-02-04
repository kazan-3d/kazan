// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    errors::{SPIRVIdAlreadyDefined, SPIRVIdNotDefined, TranslationResult},
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
use petgraph::{algo::dominators, dot::Dot, visit};
use spirv_id_map::{IdMap, ToIdBound};
use spirv_parser::{
    IdRef, IdResult, Instruction, OpBranch, OpBranchConditional, OpKill, OpLabel, OpLoopMerge,
    OpReturn, OpReturnValue, OpSelectionMerge, OpSwitch32, OpSwitch64, OpUnreachable,
};
use visit::{EdgeRef, IntoEdges, IntoNodeIdentifiers};

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
    Sequence32(Option<IdRef>, &'a [(u32, IdRef)]),
    Sequence64(Option<IdRef>, &'a [(u64, IdRef)]),
}

pub(crate) struct TerminationInstructionTargets<'a>(TerminationInstructionTargetsImpl<'a>);

impl<'a> Iterator for TerminationInstructionTargets<'a> {
    type Item = IdRef;
    fn next(&mut self) -> Option<IdRef> {
        use TerminationInstructionTargetsImpl::*;
        match self.0 {
            Pair(retval, rest) => {
                self.0 = Sequence32(Some(rest), &[]);
                Some(retval)
            }
            Sequence32(Some(retval), rest) => {
                self.0 = Sequence32(None, rest);
                Some(retval)
            }
            Sequence64(Some(retval), rest) => {
                self.0 = Sequence64(None, rest);
                Some(retval)
            }
            Sequence32(None, rest) => {
                let (&first, rest) = rest.split_first()?;
                self.0 = Sequence32(None, rest);
                Some(first.1)
            }
            Sequence64(None, rest) => {
                let (&first, rest) = rest.split_first()?;
                self.0 = Sequence64(None, rest);
                Some(first.1)
            }
        }
    }
}

impl TerminationInstruction {
    pub(crate) fn get_targets(&self) -> TerminationInstructionTargets {
        use TerminationInstructionTargetsImpl::*;
        TerminationInstructionTargets(match *self {
            TerminationInstruction::Branch(OpBranch { target_label }) => {
                Sequence32(Some(target_label), &[])
            }
            TerminationInstruction::BranchConditional(OpBranchConditional {
                true_label,
                false_label,
                ..
            }) => Pair(true_label, false_label),
            TerminationInstruction::Kill(_)
            | TerminationInstruction::Return(_)
            | TerminationInstruction::ReturnValue(_)
            | TerminationInstruction::Unreachable(_) => Sequence32(None, &[]),
            TerminationInstruction::Switch32(OpSwitch32 {
                default,
                ref target,
                ..
            }) => Sequence32(Some(default), target),
            TerminationInstruction::Switch64(OpSwitch64 {
                default,
                ref target,
                ..
            }) => Sequence64(Some(default), target),
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
    pub(crate) fn label_id_to_block_id(
        &self,
        label_id: IdRef,
    ) -> TranslationResult<Option<CFGBlockId>> {
        Ok(self.label_map.get(label_id)?.copied())
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
        for block in base.iter() {
            for target_label in block.termination_instruction().get_targets() {
                base.label_id_to_block_id(target_label)?
                    .ok_or_else(|| SPIRVIdNotDefined { id: target_label })?;
            }
        }
        let mut retval = CFG {
            base,
            dominators: None,
        };
        retval.dominators = Some(dominators::simple_fast(&retval, retval.entry_block_id()));
        Ok(retval)
    }
}

impl<'g, 'i> Index<CFGBlockId> for CFGBuilder<'g, 'i> {
    type Output = CFGBlock<'g, 'i>;
    fn index(&self, index: CFGBlockId) -> &Self::Output {
        &self.base[index]
    }
}

/// immutable CFG that is known to have valid labels
pub(crate) struct CFG<'g, 'i> {
    base: CFGBase<'g, 'i>,
    dominators: Option<dominators::Dominators<CFGBlockId>>,
}

impl<'g, 'i> CFG<'g, 'i> {
    pub(crate) fn dominators(&self) -> &dominators::Dominators<CFGBlockId> {
        self.dominators
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

impl<'g, 'i> Index<CFGBlockId> for CFG<'g, 'i> {
    type Output = CFGBlock<'g, 'i>;
    fn index(&self, index: CFGBlockId) -> &Self::Output {
        &self.base[index]
    }
}

impl<'g, 'i> Deref for CFG<'g, 'i> {
    type Target = CFGBase<'g, 'i>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl visit::GraphBase for CFG<'_, '_> {
    type NodeId = CFGBlockId;
    type EdgeId = CFGEdgeId;
}

impl visit::GraphProp for CFG<'_, '_> {
    type EdgeType = petgraph::Directed;
}

impl<'g, 'i> visit::Data for CFG<'g, 'i> {
    type NodeWeight = CFGBlock<'g, 'i>;
    type EdgeWeight = ();
}

impl visit::NodeIndexable for CFG<'_, '_> {
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

pub(crate) struct CFGVisitMap(FixedBitSet);

impl visit::VisitMap<CFGBlockId> for CFGVisitMap {
    fn visit(&mut self, id: CFGBlockId) -> bool {
        !self.0.put(id.0)
    }
    fn is_visited(&self, id: &CFGBlockId) -> bool {
        self.0.contains(id.0)
    }
}

impl<'g, 'i> visit::Visitable for CFG<'g, 'i> {
    type Map = CFGVisitMap;
    fn visit_map(&self) -> Self::Map {
        CFGVisitMap(FixedBitSet::with_capacity(self.blocks.len()))
    }
    fn reset_map(&self, map: &mut Self::Map) {
        map.0.clear()
    }
}

pub(crate) struct CFGBlockTargetEdges<'a, 'g, 'i> {
    source_block: CFGBlockId,
    targets: iter::Enumerate<TerminationInstructionTargets<'a>>,
    cfg: &'a CFG<'g, 'i>,
}

impl<'a, 'g, 'i> Iterator for CFGBlockTargetEdges<'a, 'g, 'i> {
    type Item = CFGEdgeRef<'a, 'g, 'i>;
    fn next(&mut self) -> Option<Self::Item> {
        let (target_index, label_id) = self.targets.next()?;
        let target = self
            .cfg
            .label_id_to_block_id(label_id)
            .expect("known to be valid")
            .expect("known to be valid");
        Some(CFGEdgeRef {
            id: CFGEdgeId {
                source_block: self.source_block,
                target_index,
            },
            target,
            cfg: self.cfg,
        })
    }
}

pub(crate) struct CFGBlockTargetBlockIds<'a, 'g, 'i>(CFGBlockTargetEdges<'a, 'g, 'i>);

impl<'a, 'g, 'i> Iterator for CFGBlockTargetBlockIds<'a, 'g, 'i> {
    type Item = CFGBlockId;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.next()?.target())
    }
}

impl<'a, 'g, 'i> visit::IntoNeighbors for &'a CFG<'g, 'i> {
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

#[derive(Copy, Clone)]
pub(crate) struct CFGEdgeRef<'a, 'g, 'i> {
    id: CFGEdgeId,
    target: CFGBlockId,
    cfg: &'a CFG<'g, 'i>,
}

impl<'a, 'g, 'i> EdgeRef for CFGEdgeRef<'a, 'g, 'i> {
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
        self.iter.next().map(CFGBlockId)
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
    type Item = CFGEdgeRef<'a, 'g, 'i>;
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

impl<'a, 'g, 'i> visit::IntoEdgeReferences for &'a CFG<'g, 'i> {
    type EdgeRef = CFGEdgeRef<'a, 'g, 'i>;
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
        let block_id = self.block_ids.next()?;
        Some((block_id, &self.cfg[block_id]))
    }
}

impl<'a, 'g, 'i> visit::IntoNodeReferences for &'a CFG<'g, 'i> {
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
                .ok()
                .expect("merge_location is not at a merge instruction");
            (location, merge_instruction)
        });
        let termination_instruction = termination_location
            .get_instruction()
            .expect("termination_location is not at a block termination instruction")
            .clone()
            .try_into()
            .ok()
            .expect("termination_location is not at a block termination instruction");
        Self {
            label_location,
            label_instruction,
            merge,
            termination_location,
            termination_instruction,
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
