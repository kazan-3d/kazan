// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use lattice::BoundedOrderedLattice;
use std::cmp;

/// a lattice for how much values vary between different shader invocations
/// Constant < UniformOverWorkgroup < Varying
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ValueUniformity {
    /// value is constant
    Constant = 0,
    /// value has same value for every invocation in a workgroup
    UniformOverWorkgroup = 1,
    /// value may be different in every invocation
    Varying = 2,
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
        ValueUniformity::Constant
    }
    fn max_value() -> Self {
        ValueUniformity::Varying
    }
}

impl ValueUniformity {}
