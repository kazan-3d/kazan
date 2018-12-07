// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use cfg::{CFGNodeIndex, CFG};
use instruction_properties::InstructionProperties;
use lattice::BoundedOrderedLattice;
use petgraph::visit::IntoNodeReferences;
use spirv_parser::IdRef;
use std::cmp;
use std::collections::HashMap;
use Ids;

/// a lattice for how little values vary
/// Varying < UniformOverWorkgroup < Constant
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValueUniformity {
    /// value may be different in every invocation
    Varying = 0,
    /// value has same value for every invocation in a workgroup
    UniformOverWorkgroup = 1,
    /// value is constant
    Constant = 2,
}

impl Default for ValueUniformity {
    fn default() -> Self {
        ValueUniformity::Varying
    }
}

impl Ord for ValueUniformity {
    fn cmp(&self, rhs: &Self) -> cmp::Ordering {
        (*self as u32).cmp(&(*rhs as u32))
    }
}

impl PartialOrd for ValueUniformity {
    fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(&rhs))
    }
}

impl BoundedOrderedLattice for ValueUniformity {
    fn min_value() -> Self {
        ValueUniformity::Varying
    }
    fn max_value() -> Self {
        ValueUniformity::Constant
    }
}

#[derive(Copy, Clone, Debug)]
struct ValueUniformityEntry {
    value_uniformity: ValueUniformity,
    defining_location: Option<(CFGNodeIndex, usize)>,
}

impl Default for ValueUniformityEntry {
    fn default() -> Self {
        Self {
            value_uniformity: ValueUniformity::Constant,
            defining_location: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ValueUniformities {
    entries: HashMap<IdRef, ValueUniformityEntry>,
}

impl ValueUniformities {
    pub(crate) fn new<'ctx, C: shader_compiler_backend::Context<'ctx>>(
        cfg: &CFG,
        ids: &Ids<'ctx, C>,
    ) -> Self {
        let mut entries: HashMap<IdRef, ValueUniformityEntry> = HashMap::new();
        for (node_index, basic_block) in cfg.node_references() {
            for (index, instruction) in basic_block.instructions().iter().enumerate() {
                if let Some(result) = InstructionProperties::new(instruction).result() {
                    let mut entry = entries.entry(result.0).or_default();
                    let defining_location = (node_index, index);
                    if let Some(old_location) = entry.defining_location {
                        assert_eq!(old_location, defining_location, "duplicate id definition");
                    } else {
                        entry.defining_location = Some(defining_location);
                    }
                }
            }
        }
        unimplemented!();
        ValueUniformities { entries }
    }
}
