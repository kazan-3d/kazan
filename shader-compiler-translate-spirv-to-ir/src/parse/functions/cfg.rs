// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::SPIRVInstructionLocation;
use core::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
    ops::Index,
};
use petgraph::{algo::dominators, visit};
use spirv_id_map::IdMap;
use spirv_parser::{
    IdRef, Instruction, OpBranch, OpBranchConditional, OpKill, OpLabel, OpLoopMerge, OpReturn,
    OpReturnValue, OpSelectionMerge, OpSwitch32, OpSwitch64, OpUnreachable,
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
pub(crate) struct CFGBlockId(IdRef);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct CFGEdgeId {
    pub(crate) source_block: CFGBlockId,
    pub(crate) target_index: usize,
}

pub(crate) struct CFG<'g, 'i> {
    blocks: IdMap<IdRef, CFGBlock<'g, 'i>>,
    entry_block_id: CFGBlockId,
    dominators: Option<dominators::Dominators<CFGBlockId>>,
}

impl<'g, 'i> CFG<'g, 'i> {
    pub(crate) fn new(blocks: IdMap<IdRef, CFGBlock<'g, 'i>>, entry_block_id: CFGBlockId) -> Self {
        let mut retval = Self {
            blocks,
            entry_block_id,
            dominators: None,
        };
        retval.dominators = Some(dominators::simple_fast(&retval, entry_block_id));
        retval
    }
    pub(crate) fn dominators(&self) -> &dominators::Dominators<CFGBlockId> {
        self.dominators
            .as_ref()
            .expect("dominators was filled by CFG::new")
    }
}

impl<'g, 'i> Index<CFGBlockId> for CFG<'g, 'i> {
    type Output = CFGBlock<'g, 'i>;
    fn index(&self, index: CFGBlockId) -> &Self::Output {
        self.blocks
            .get(index.0)
            .expect("invalid index")
            .expect("invalid index")
    }
}

impl visit::GraphBase for CFG<'_, '_> {
    type NodeId = CFGBlockId;
    type EdgeId = CFGEdgeId;
}

pub(crate) struct CFGVisitMap(IdMap<IdRef, ()>);

impl visit::VisitMap<CFGBlockId> for CFGVisitMap {
    fn visit(&mut self, id: CFGBlockId) -> bool {
        if let spirv_id_map::Vacant(entry) = self.0.entry(id.0).expect("invalid index") {
            entry.insert(());
            true
        } else {
            false
        }
    }
    fn is_visited(&self, id: &CFGBlockId) -> bool {
        self.0.get(id.0).expect("invalid index").is_some()
    }
}

impl<'g, 'i> visit::Visitable for CFG<'g, 'i> {
    type Map = CFGVisitMap;
    fn visit_map(&self) -> Self::Map {
        CFGVisitMap(IdMap::with_same_bound(&self.blocks))
    }
    fn reset_map(&self, map: &mut Self::Map) {
        map.0.clear()
    }
}

pub(crate) struct CFGBlockTargetBlockIds<'a>(TerminationInstructionTargets<'a>);

impl<'a> Iterator for CFGBlockTargetBlockIds<'a> {
    type Item = CFGBlockId;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(CFGBlockId)
    }
}

impl<'a, 'g, 'i> visit::IntoNeighbors for &'a CFG<'g, 'i> {
    type Neighbors = CFGBlockTargetBlockIds<'a>;
    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        self[a].target_block_ids()
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
    pub(crate) fn target_block_ids(&self) -> CFGBlockTargetBlockIds {
        CFGBlockTargetBlockIds(self.termination_instruction().get_targets())
    }
}
