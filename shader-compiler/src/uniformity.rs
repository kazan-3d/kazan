// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

mod anf;
mod variable_set;

use self::variable_set::VariableSet;
use crate::cfg::{CFGEdgeIndex, CFGNodeIndex, CFG};
use crate::lattice::{BoundedOrderedLattice, MeetSemilattice};
use crate::BuiltInVariable;
use crate::IdKind;
use crate::Ids;
use petgraph::prelude::*;
use petgraph::visit::NodeCompactIndexable;
use spirv_parser::IdResult;
use spirv_parser::{BuiltIn, IdRef, Instruction, StorageClass};
use std::borrow::Borrow;
use std::cmp;
use std::collections::{HashMap, HashSet};
use std::iter;
use std::ops::Deref;

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

#[derive(Clone, Debug, Eq, PartialEq)]
struct PointeeUniformity {
    value_uniformity: ValueUniformity,
    variables: VariableSet,
}

impl Default for PointeeUniformity {
    fn default() -> Self {
        PointeeUniformity {
            value_uniformity: ValueUniformity::Constant,
            variables: VariableSet::new(),
        }
    }
}

impl PointeeUniformity {
    fn check_update_with(&self, new_value: &Self) {
        let PointeeUniformity {
            value_uniformity: old_value_uniformity,
            variables: ref old_variables,
        } = *self;
        let PointeeUniformity {
            value_uniformity: new_value_uniformity,
            variables: ref new_variables,
        } = *new_value;
        assert_eq!(
            new_value_uniformity,
            old_value_uniformity.meet(new_value_uniformity),
            "invalid PointeeUniformity::value_uniformity update"
        );
        // faster check first
        if old_variables != new_variables {
            assert!(
                (old_variables - new_variables).is_empty(),
                "invalid PointeeUniformity::variables update"
            );
        }
    }
}

trait UpdatableEntryType: Clone + Default + Eq {
    fn check_update_with(&self, new_value: &Self);
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BasicBlockUniformityEntry {
    value_uniformity: ValueUniformity,
}

impl UpdatableEntryType for BasicBlockUniformityEntry {
    fn check_update_with(&self, new_value: &Self) {
        let BasicBlockUniformityEntry {
            value_uniformity: old_value_uniformity,
        } = *self;
        let BasicBlockUniformityEntry {
            value_uniformity: new_value_uniformity,
        } = *new_value;
        assert_eq!(
            new_value_uniformity,
            old_value_uniformity.meet(new_value_uniformity),
            "invalid BasicBlockUniformityEntry::value_uniformity update"
        );
    }
}

impl Default for BasicBlockUniformityEntry {
    fn default() -> Self {
        Self {
            value_uniformity: ValueUniformity::Constant,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CFGEdgeUniformityEntry {
    value_uniformity: ValueUniformity,
}

impl UpdatableEntryType for CFGEdgeUniformityEntry {
    fn check_update_with(&self, new_value: &Self) {
        let CFGEdgeUniformityEntry {
            value_uniformity: old_value_uniformity,
        } = *self;
        let CFGEdgeUniformityEntry {
            value_uniformity: new_value_uniformity,
        } = *new_value;
        assert_eq!(
            new_value_uniformity,
            old_value_uniformity.meet(new_value_uniformity),
            "invalid CFGEdgeUniformityEntry::value_uniformity update"
        );
    }
}

impl Default for CFGEdgeUniformityEntry {
    fn default() -> Self {
        Self {
            value_uniformity: ValueUniformity::Constant,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ValueUniformityEntry {
    value_uniformity: ValueUniformity,
    pointee_uniformity: Option<PointeeUniformity>,
}

impl UpdatableEntryType for ValueUniformityEntry {
    fn check_update_with(&self, new_value: &Self) {
        let ValueUniformityEntry {
            value_uniformity: old_value_uniformity,
            pointee_uniformity: ref old_pointee_uniformity,
        } = *self;
        let ValueUniformityEntry {
            value_uniformity: new_value_uniformity,
            pointee_uniformity: ref new_pointee_uniformity,
        } = *new_value;
        assert_eq!(
            new_value_uniformity,
            old_value_uniformity.meet(new_value_uniformity),
            "invalid ValueUniformityEntry::value_uniformity update"
        );
        match (old_pointee_uniformity, new_pointee_uniformity) {
            (Some(_), None) => {
                unreachable!("invalid ValueUniformityEntry::pointee_uniformity update");
            }
            (Some(old_pointee_uniformity), Some(new_pointee_uniformity)) => {
                old_pointee_uniformity.check_update_with(new_pointee_uniformity);
            }
            _ => {}
        }
    }
}

impl Default for ValueUniformityEntry {
    fn default() -> Self {
        Self {
            value_uniformity: ValueUniformity::Constant,
            pointee_uniformity: None,
        }
    }
}

fn get_built_in_initial_value_uniformity_entry(
    built_in_variable: &BuiltInVariable,
    id: IdRef,
) -> ValueUniformityEntry {
    let value_uniformity = match built_in_variable.built_in {
        BuiltIn::Position => unimplemented!(),
        BuiltIn::PointSize => unimplemented!(),
        BuiltIn::ClipDistance => unimplemented!(),
        BuiltIn::CullDistance => unimplemented!(),
        BuiltIn::VertexId => unimplemented!(),
        BuiltIn::InstanceId => unimplemented!(),
        BuiltIn::PrimitiveId => unimplemented!(),
        BuiltIn::InvocationId => unimplemented!(),
        BuiltIn::Layer => unimplemented!(),
        BuiltIn::ViewportIndex => unimplemented!(),
        BuiltIn::TessLevelOuter => unimplemented!(),
        BuiltIn::TessLevelInner => unimplemented!(),
        BuiltIn::TessCoord => unimplemented!(),
        BuiltIn::PatchVertices => unimplemented!(),
        BuiltIn::FragCoord => unimplemented!(),
        BuiltIn::PointCoord => unimplemented!(),
        BuiltIn::FrontFacing => unimplemented!(),
        BuiltIn::SampleId => unimplemented!(),
        BuiltIn::SamplePosition => unimplemented!(),
        BuiltIn::SampleMask => unimplemented!(),
        BuiltIn::FragDepth => unimplemented!(),
        BuiltIn::HelperInvocation => unimplemented!(),
        BuiltIn::NumWorkgroups => unimplemented!(),
        BuiltIn::WorkgroupSize => unimplemented!(),
        BuiltIn::WorkgroupId => unimplemented!(),
        BuiltIn::LocalInvocationId => unimplemented!(),
        BuiltIn::GlobalInvocationId => ValueUniformity::Varying,
        BuiltIn::LocalInvocationIndex => unimplemented!(),
        BuiltIn::WorkDim => unimplemented!(),
        BuiltIn::GlobalSize => unimplemented!(),
        BuiltIn::EnqueuedWorkgroupSize => unimplemented!(),
        BuiltIn::GlobalOffset => unimplemented!(),
        BuiltIn::GlobalLinearId => unimplemented!(),
        BuiltIn::SubgroupSize => unimplemented!(),
        BuiltIn::SubgroupMaxSize => unimplemented!(),
        BuiltIn::NumSubgroups => unimplemented!(),
        BuiltIn::NumEnqueuedSubgroups => unimplemented!(),
        BuiltIn::SubgroupId => unimplemented!(),
        BuiltIn::SubgroupLocalInvocationId => unimplemented!(),
        BuiltIn::VertexIndex => unimplemented!(),
        BuiltIn::InstanceIndex => unimplemented!(),
        BuiltIn::SubgroupEqMask => unimplemented!(),
        BuiltIn::SubgroupGeMask => unimplemented!(),
        BuiltIn::SubgroupGtMask => unimplemented!(),
        BuiltIn::SubgroupLeMask => unimplemented!(),
        BuiltIn::SubgroupLtMask => unimplemented!(),
        BuiltIn::BaseVertex => unimplemented!(),
        BuiltIn::BaseInstance => unimplemented!(),
        BuiltIn::DrawIndex => unimplemented!(),
        BuiltIn::DeviceIndex => unimplemented!(),
        BuiltIn::ViewIndex => unimplemented!(),
        BuiltIn::LaunchIdNV => unimplemented!(),
        BuiltIn::LaunchSizeNV => unimplemented!(),
        BuiltIn::WorldRayOriginNV => unimplemented!(),
        BuiltIn::WorldRayDirectionNV => unimplemented!(),
        BuiltIn::ObjectRayOriginNV => unimplemented!(),
        BuiltIn::ObjectRayDirectionNV => unimplemented!(),
        BuiltIn::RayTminNV => unimplemented!(),
        BuiltIn::RayTmaxNV => unimplemented!(),
        BuiltIn::InstanceCustomIndexNV => unimplemented!(),
        BuiltIn::ObjectToWorldNV => unimplemented!(),
        BuiltIn::WorldToObjectNV => unimplemented!(),
        BuiltIn::HitTNV => unimplemented!(),
        BuiltIn::HitKindNV => unimplemented!(),
        BuiltIn::IncomingRayFlagsNV => unimplemented!(),
    };
    ValueUniformityEntry {
        value_uniformity: ValueUniformity::UniformOverWorkgroup,
        pointee_uniformity: Some(PointeeUniformity {
            value_uniformity,
            variables: VariableSet::from(id),
        }),
    }
}

trait AccessValueUniformityEntry<Key: Copy + std::hash::Hash + Eq> {
    type EntryType: UpdatableEntryType;
    fn get_entry_table(&self) -> &HashMap<Key, Self::EntryType>;
    fn get_entry_table_mut(&mut self) -> &mut HashMap<Key, Self::EntryType>;
    fn report_changes(&mut self);
    fn set_entry(&mut self, key: Key, v: Self::EntryType) {
        self.with_entry(key, |value| *value = v);
    }
    fn get_entry(&mut self, key: Key) -> Self::EntryType {
        if let Some(v) = self.get_entry_table().get(&key) {
            v.clone()
        } else {
            Default::default()
        }
    }
    fn with_entry<F: FnOnce(&mut Self::EntryType)>(&mut self, key: Key, f: F) {
        use std::collections::hash_map::Entry;
        match self.get_entry_table_mut().entry(key) {
            Entry::Vacant(entry) => {
                let mut value = Self::EntryType::default();
                f(&mut value);
                if value != Default::default() {
                    entry.insert(value);
                    self.report_changes();
                }
            }
            Entry::Occupied(entry) => {
                let entry = entry.into_mut();
                let mut value = entry.clone();
                f(&mut value);
                if value != *entry {
                    entry.check_update_with(&value);
                    *entry = value;
                    self.report_changes();
                }
            }
        }
    }
}

struct ValueUniformityCalculator<'a, 'ctx, C: shader_compiler_backend::Context<'ctx>> {
    entries: HashMap<IdRef, ValueUniformityEntry>,
    basic_blocks: HashMap<CFGNodeIndex, BasicBlockUniformityEntry>,
    edges: HashMap<CFGEdgeIndex, CFGEdgeUniformityEntry>,
    cfg: &'a CFG,
    ids: &'a Ids<'ctx, C>,
    any_changes: bool,
}

impl<'a, 'ctx, C: shader_compiler_backend::Context<'ctx>> AccessValueUniformityEntry<IdRef>
    for ValueUniformityCalculator<'a, 'ctx, C>
{
    type EntryType = ValueUniformityEntry;
    fn get_entry_table(&self) -> &HashMap<IdRef, Self::EntryType> {
        &self.entries
    }
    fn get_entry_table_mut(&mut self) -> &mut HashMap<IdRef, Self::EntryType> {
        &mut self.entries
    }
    fn report_changes(&mut self) {
        self.any_changes = true;
    }
}

impl<'a, 'ctx, C: shader_compiler_backend::Context<'ctx>> AccessValueUniformityEntry<CFGNodeIndex>
    for ValueUniformityCalculator<'a, 'ctx, C>
{
    type EntryType = BasicBlockUniformityEntry;
    fn get_entry_table(&self) -> &HashMap<CFGNodeIndex, Self::EntryType> {
        &self.basic_blocks
    }
    fn get_entry_table_mut(&mut self) -> &mut HashMap<CFGNodeIndex, Self::EntryType> {
        &mut self.basic_blocks
    }
    fn report_changes(&mut self) {
        self.any_changes = true;
    }
}

impl<'a, 'ctx, C: shader_compiler_backend::Context<'ctx>> AccessValueUniformityEntry<CFGEdgeIndex>
    for ValueUniformityCalculator<'a, 'ctx, C>
{
    type EntryType = CFGEdgeUniformityEntry;
    fn get_entry_table(&self) -> &HashMap<CFGEdgeIndex, Self::EntryType> {
        &self.edges
    }
    fn get_entry_table_mut(&mut self) -> &mut HashMap<CFGEdgeIndex, Self::EntryType> {
        &mut self.edges
    }
    fn report_changes(&mut self) {
        self.any_changes = true;
    }
}

impl<'a, 'ctx, C: shader_compiler_backend::Context<'ctx>> ValueUniformityCalculator<'a, 'ctx, C> {
    fn new(cfg: &'a CFG, ids: &'a Ids<'ctx, C>) -> Self
    where
        <CFG as Deref>::Target: NodeCompactIndexable,
    {
        ValueUniformityCalculator {
            entries: HashMap::new(),
            basic_blocks: HashMap::new(),
            edges: HashMap::new(),
            cfg,
            ids,
            any_changes: false,
        }
    }
    fn visit_simple_instruction<I: IntoIterator>(&mut self, id_result: IdResult, arguments: I)
    where
        I::Item: Borrow<IdRef>,
    {
        let mut value_uniformity = ValueUniformity::Constant;
        for argument in arguments {
            let argument = *argument.borrow();
            let ValueUniformityEntry {
                value_uniformity: argument_value_uniformity,
                pointee_uniformity,
            } = self.get_entry(argument);
            assert!(
                pointee_uniformity.is_none(),
                "pointer is invalid argument to simple arithmatic/logic instruction"
            );
            value_uniformity.meet_assign(argument_value_uniformity);
        }
        self.set_entry(
            id_result.0,
            ValueUniformityEntry {
                value_uniformity,
                pointee_uniformity: None,
            },
        );
    }
    fn calculate_block_value_uniformity(&mut self, node_index: CFGNodeIndex) -> ValueUniformity {
        let mut value_uniformity = ValueUniformity::Constant;
        unimplemented!();
        //for
        value_uniformity
    }
    fn visit_branch_instruction(
        &mut self,
        node_index: CFGNodeIndex,
        condition_uniformity: ValueUniformity,
    ) {
        let BasicBlockUniformityEntry {
            value_uniformity: basic_block_uniformity,
        } = self.get_entry(node_index);
        for edge in self.cfg.edges_directed(node_index, Outgoing) {
            self.set_entry(
                edge.id(),
                CFGEdgeUniformityEntry {
                    value_uniformity: basic_block_uniformity.meet(condition_uniformity),
                },
            );
        }
        unimplemented!();
    }
    fn visit_instruction(&mut self, node_index: CFGNodeIndex, instruction: &Instruction) {
        match *instruction {
            Instruction::Nop {} => {}
            Instruction::Undef {
                id_result_type: _id_result_type,
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::SourceContinued { .. } => {}
            Instruction::Source { .. } => {}
            Instruction::SourceExtension { .. } => {}
            Instruction::Name { .. } => {}
            Instruction::MemberName { .. } => {}
            Instruction::String { .. } => {}
            Instruction::Line { .. } => {}
            Instruction::Extension { .. } => {}
            Instruction::ExtInstImport { .. } => {}
            Instruction::ExtInst { .. } => {
                unreachable!("unimplemented OpExtInst:\n{}", instruction);
            }
            Instruction::MemoryModel { .. } => {}
            Instruction::EntryPoint { .. } => {}
            Instruction::ExecutionMode {
                entry_point: _entry_point,
                mode: ref _mode,
            } => unimplemented!(),
            Instruction::Capability {
                capability: _capability,
            } => unimplemented!(),
            Instruction::TypeVoid {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::TypeBool {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::TypeInt {
                id_result: _id_result,
                width: _width,
                signedness: _signedness,
            } => unimplemented!(),
            Instruction::TypeFloat {
                id_result: _id_result,
                width: _width,
            } => unimplemented!(),
            Instruction::TypeVector {
                id_result: _id_result,
                component_type: _component_type,
                component_count: _component_count,
            } => unimplemented!(),
            Instruction::TypeMatrix {
                id_result: _id_result,
                column_type: _column_type,
                column_count: _column_count,
            } => unimplemented!(),
            Instruction::TypeImage {
                id_result: _id_result,
                sampled_type: _sampled_type,
                dim: _dim,
                depth: _depth,
                arrayed: _arrayed,
                ms: _ms,
                sampled: _sampled,
                image_format: _image_format,
                access_qualifier: _access_qualifier,
            } => unimplemented!(),
            Instruction::TypeSampler {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::TypeSampledImage {
                id_result: _id_result,
                image_type: _image_type,
            } => unimplemented!(),
            Instruction::TypeArray {
                id_result: _id_result,
                element_type: _element_type,
                length: _length,
            } => unimplemented!(),
            Instruction::TypeRuntimeArray {
                id_result: _id_result,
                element_type: _element_type,
            } => unimplemented!(),
            Instruction::TypeStruct {
                id_result: _id_result,
                member_types: ref _member_types,
            } => unimplemented!(),
            Instruction::TypeOpaque {
                id_result: _id_result,
                the_name_of_the_opaque_type: ref _the_name_of_the_opaque_type,
            } => unimplemented!(),
            Instruction::TypePointer {
                id_result: _id_result,
                storage_class: _storage_class,
                type_: _type_,
            } => unimplemented!(),
            Instruction::TypeFunction {
                id_result: _id_result,
                return_type: _return_type,
                parameter_types: ref _parameter_types,
            } => unimplemented!(),
            Instruction::TypeEvent {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::TypeDeviceEvent {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::TypeReserveId {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::TypeQueue {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::TypePipe {
                id_result: _id_result,
                qualifier: _qualifier,
            } => unimplemented!(),
            Instruction::TypeForwardPointer {
                pointer_type: _pointer_type,
                storage_class: _storage_class,
            } => unimplemented!(),
            Instruction::ConstantTrue {
                id_result_type: _id_result_type,
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::ConstantFalse {
                id_result_type: _id_result_type,
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::Constant32 {
                id_result_type: _id_result_type,
                id_result: _id_result,
                value: _value,
            } => unimplemented!(),
            Instruction::Constant64 {
                id_result_type: _id_result_type,
                id_result: _id_result,
                value: _value,
            } => unimplemented!(),
            Instruction::ConstantComposite {
                id_result_type: _id_result_type,
                id_result: _id_result,
                constituents: ref _constituents,
            } => unimplemented!(),
            Instruction::ConstantSampler {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampler_addressing_mode: _sampler_addressing_mode,
                param: _param,
                sampler_filter_mode: _sampler_filter_mode,
            } => unimplemented!(),
            Instruction::ConstantNull {
                id_result_type: _id_result_type,
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::SpecConstantTrue {
                id_result_type: _id_result_type,
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::SpecConstantFalse {
                id_result_type: _id_result_type,
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::SpecConstant32 {
                id_result_type: _id_result_type,
                id_result: _id_result,
                value: _value,
            } => unimplemented!(),
            Instruction::SpecConstant64 {
                id_result_type: _id_result_type,
                id_result: _id_result,
                value: _value,
            } => unimplemented!(),
            Instruction::SpecConstantComposite {
                id_result_type: _id_result_type,
                id_result: _id_result,
                constituents: ref _constituents,
            } => unimplemented!(),
            Instruction::SpecConstantOp {
                operation: ref _operation,
            } => unimplemented!(),
            Instruction::Function {
                id_result_type: _id_result_type,
                id_result: _id_result,
                function_control: ref _function_control,
                function_type: _function_type,
            } => unimplemented!(),
            Instruction::FunctionParameter {
                id_result_type: _id_result_type,
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::FunctionEnd {} => unimplemented!(),
            Instruction::FunctionCall {
                id_result_type: _id_result_type,
                id_result: _id_result,
                function: _function,
                arguments: ref _arguments,
            } => unimplemented!(),
            Instruction::Variable {
                id_result_type,
                id_result,
                storage_class,
                initializer,
            } => {
                if initializer.is_some() {
                    unimplemented!("variable initializers aren't implemented:\n{}", instruction);
                }
                assert_eq!(storage_class, StorageClass::Function);
                assert!(
                    !self.ids[id_result_type.0]
                        .get_nonvoid_type()
                        .get_nonvoid_pointee()
                        .is_pointer(),
                    "pointers to pointers are not implemented"
                );
                self.with_entry(
                    id_result.0,
                    |ValueUniformityEntry {
                         value_uniformity,
                         pointee_uniformity,
                     }| {
                        let pointee_uniformity =
                            pointee_uniformity.get_or_insert_with(PointeeUniformity::default);
                        *value_uniformity = ValueUniformity::UniformOverWorkgroup;
                        pointee_uniformity
                            .value_uniformity
                            .meet_assign(ValueUniformity::Constant);
                        pointee_uniformity.variables |= VariableSet::from(id_result.0);
                    },
                );
            }
            Instruction::ImageTexelPointer {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
                coordinate: _coordinate,
                sample: _sample,
            } => unimplemented!(),
            Instruction::Load {
                id_result_type: _,
                id_result,
                pointer,
                ref memory_access,
            } => {
                let ValueUniformityEntry {
                    value_uniformity,
                    pointee_uniformity,
                } = self.get_entry(pointer);
                let pointee_uniformity = pointee_uniformity.expect("pointer");
                let value_uniformity =
                    if memory_access.clone().unwrap_or_default().volatile.is_some() {
                        ValueUniformity::Varying
                    } else {
                        value_uniformity.meet(pointee_uniformity.value_uniformity)
                    };
                self.set_entry(
                    id_result.0,
                    ValueUniformityEntry {
                        value_uniformity: value_uniformity
                            .meet(pointee_uniformity.value_uniformity),
                        pointee_uniformity: None,
                    },
                );
            }
            Instruction::Store {
                pointer: _pointer,
                object: _object,
                memory_access: ref _memory_access,
            } => unimplemented!(),
            Instruction::CopyMemory {
                target: _target,
                source: _source,
                memory_access: ref _memory_access,
            } => unimplemented!(),
            Instruction::CopyMemorySized {
                target: _target,
                source: _source,
                size: _size,
                memory_access: ref _memory_access,
            } => unimplemented!(),
            Instruction::AccessChain {
                id_result_type: _,
                id_result,
                base,
                ref indexes,
            } => {
                let ValueUniformityEntry {
                    mut value_uniformity,
                    pointee_uniformity,
                } = self.get_entry(base);
                let pointee_uniformity = pointee_uniformity.expect("pointer");
                for &index in indexes.iter() {
                    value_uniformity.meet_assign(self.get_entry(index).value_uniformity);
                }
                self.set_entry(
                    id_result.0,
                    ValueUniformityEntry {
                        value_uniformity,
                        pointee_uniformity: Some(pointee_uniformity),
                    },
                );
            }
            Instruction::InBoundsAccessChain {
                id_result_type: _id_result_type,
                id_result: _id_result,
                base: _base,
                indexes: ref _indexes,
            } => unimplemented!(),
            Instruction::PtrAccessChain {
                id_result_type: _id_result_type,
                id_result: _id_result,
                base: _base,
                element: _element,
                indexes: ref _indexes,
            } => unimplemented!(),
            Instruction::ArrayLength {
                id_result_type: _id_result_type,
                id_result: _id_result,
                structure: _structure,
                array_member: _array_member,
            } => unimplemented!(),
            Instruction::GenericPtrMemSemantics {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
            } => unimplemented!(),
            Instruction::InBoundsPtrAccessChain {
                id_result_type: _id_result_type,
                id_result: _id_result,
                base: _base,
                element: _element,
                indexes: ref _indexes,
            } => unimplemented!(),
            Instruction::Decorate {
                target: _target,
                decoration: ref _decoration,
            } => unimplemented!(),
            Instruction::MemberDecorate {
                structure_type: _structure_type,
                member: _member,
                decoration: ref _decoration,
            } => unimplemented!(),
            Instruction::DecorationGroup {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::GroupDecorate {
                decoration_group: _decoration_group,
                targets: ref _targets,
            } => unimplemented!(),
            Instruction::GroupMemberDecorate {
                decoration_group: _decoration_group,
                targets: ref _targets,
            } => unimplemented!(),
            Instruction::VectorExtractDynamic {
                id_result_type: _,
                id_result,
                vector,
                index,
            } => self.visit_simple_instruction(id_result, &[vector, index]),
            Instruction::VectorInsertDynamic {
                id_result_type: _,
                id_result,
                vector,
                component,
                index,
            } => self.visit_simple_instruction(id_result, &[vector, component, index]),
            Instruction::VectorShuffle {
                id_result_type: _,
                id_result,
                vector_1,
                vector_2,
                components: _,
            } => self.visit_simple_instruction(id_result, &[vector_1, vector_2]),
            Instruction::CompositeConstruct {
                id_result_type: _,
                id_result,
                ref constituents,
            } => self.visit_simple_instruction(id_result, constituents),
            Instruction::CompositeExtract {
                id_result_type: _,
                id_result,
                composite,
                indexes: _,
            } => self.visit_simple_instruction(id_result, iter::once(composite)),
            Instruction::CompositeInsert {
                id_result_type: _,
                id_result,
                object,
                composite,
                indexes: _,
            } => self.visit_simple_instruction(id_result, &[object, composite]),
            Instruction::CopyObject {
                id_result_type: _,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::Transpose {
                id_result_type: _,
                id_result,
                matrix,
            } => self.visit_simple_instruction(id_result, iter::once(matrix)),
            Instruction::SampledImage {
                id_result_type: _,
                id_result,
                image,
                sampler,
            } => self.visit_simple_instruction(id_result, &[image, sampler]),
            Instruction::ImageSampleImplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleExplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleDrefImplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                d_ref: _d_ref,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleDrefExplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                d_ref: _d_ref,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleProjImplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleProjExplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleProjDrefImplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                d_ref: _d_ref,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleProjDrefExplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                d_ref: _d_ref,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageFetch {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageGather {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                component: _component,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageDrefGather {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                d_ref: _d_ref,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageRead {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageWrite {
                image: _image,
                coordinate: _coordinate,
                texel: _texel,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::Image {
                id_result_type: _,
                id_result,
                sampled_image,
            } => self.visit_simple_instruction(id_result, iter::once(sampled_image)),
            Instruction::ImageQueryFormat {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
            } => unimplemented!(),
            Instruction::ImageQueryOrder {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
            } => unimplemented!(),
            Instruction::ImageQuerySizeLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
                level_of_detail: _level_of_detail,
            } => unimplemented!(),
            Instruction::ImageQuerySize {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
            } => unimplemented!(),
            Instruction::ImageQueryLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
            } => unimplemented!(),
            Instruction::ImageQueryLevels {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
            } => unimplemented!(),
            Instruction::ImageQuerySamples {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
            } => unimplemented!(),
            Instruction::ConvertFToU {
                id_result_type: _,
                id_result,
                float_value,
            } => self.visit_simple_instruction(id_result, iter::once(float_value)),
            Instruction::ConvertFToS {
                id_result_type: _,
                id_result,
                float_value,
            } => self.visit_simple_instruction(id_result, iter::once(float_value)),
            Instruction::ConvertSToF {
                id_result_type: _,
                id_result,
                signed_value,
            } => self.visit_simple_instruction(id_result, iter::once(signed_value)),
            Instruction::ConvertUToF {
                id_result_type: _,
                id_result,
                unsigned_value,
            } => self.visit_simple_instruction(id_result, iter::once(unsigned_value)),
            Instruction::UConvert {
                id_result_type: _,
                id_result,
                unsigned_value,
            } => self.visit_simple_instruction(id_result, iter::once(unsigned_value)),
            Instruction::SConvert {
                id_result_type: _,
                id_result,
                signed_value,
            } => self.visit_simple_instruction(id_result, iter::once(signed_value)),
            Instruction::FConvert {
                id_result_type: _,
                id_result,
                float_value,
            } => self.visit_simple_instruction(id_result, iter::once(float_value)),
            Instruction::QuantizeToF16 {
                id_result_type: _,
                id_result,
                value,
            } => self.visit_simple_instruction(id_result, iter::once(value)),
            Instruction::ConvertPtrToU {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
            } => unimplemented!(),
            Instruction::SatConvertSToU {
                id_result_type: _id_result_type,
                id_result: _id_result,
                signed_value: _signed_value,
            } => unimplemented!(),
            Instruction::SatConvertUToS {
                id_result_type: _id_result_type,
                id_result: _id_result,
                unsigned_value: _unsigned_value,
            } => unimplemented!(),
            Instruction::ConvertUToPtr {
                id_result_type: _id_result_type,
                id_result: _id_result,
                integer_value: _integer_value,
            } => unimplemented!(),
            Instruction::PtrCastToGeneric {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
            } => unimplemented!(),
            Instruction::GenericCastToPtr {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
            } => unimplemented!(),
            Instruction::GenericCastToPtrExplicit {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                storage: _storage,
            } => unimplemented!(),
            Instruction::Bitcast {
                id_result_type: _id_result_type,
                id_result: _id_result,
                operand: _operand,
            } => unimplemented!(),
            Instruction::SNegate {
                id_result_type: _,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::FNegate {
                id_result_type: _,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::IAdd {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FAdd {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ISub {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FSub {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::IMul {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FMul {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UDiv {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SDiv {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FDiv {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UMod {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SRem {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SMod {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FRem {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FMod {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::VectorTimesScalar {
                id_result_type: _,
                id_result,
                vector,
                scalar,
            } => self.visit_simple_instruction(id_result, &[vector, scalar]),
            Instruction::MatrixTimesScalar {
                id_result_type: _,
                id_result,
                matrix,
                scalar,
            } => self.visit_simple_instruction(id_result, &[matrix, scalar]),
            Instruction::VectorTimesMatrix {
                id_result_type: _,
                id_result,
                vector,
                matrix,
            } => self.visit_simple_instruction(id_result, &[vector, matrix]),
            Instruction::MatrixTimesVector {
                id_result_type: _,
                id_result,
                matrix,
                vector,
            } => self.visit_simple_instruction(id_result, &[matrix, vector]),
            Instruction::MatrixTimesMatrix {
                id_result_type: _,
                id_result,
                left_matrix,
                right_matrix,
            } => self.visit_simple_instruction(id_result, &[left_matrix, right_matrix]),
            Instruction::OuterProduct {
                id_result_type: _,
                id_result,
                vector_1,
                vector_2,
            } => self.visit_simple_instruction(id_result, &[vector_1, vector_2]),
            Instruction::Dot {
                id_result_type: _,
                id_result,
                vector_1,
                vector_2,
            } => self.visit_simple_instruction(id_result, &[vector_1, vector_2]),
            Instruction::IAddCarry {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ISubBorrow {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UMulExtended {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SMulExtended {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::Any {
                id_result_type: _,
                id_result,
                vector,
            } => self.visit_simple_instruction(id_result, iter::once(vector)),
            Instruction::All {
                id_result_type: _,
                id_result,
                vector,
            } => self.visit_simple_instruction(id_result, iter::once(vector)),
            Instruction::IsNan {
                id_result_type: _,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::IsInf {
                id_result_type: _,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::IsFinite {
                id_result_type: _,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::IsNormal {
                id_result_type: _,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::SignBitSet {
                id_result_type: _,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::LessOrGreater {
                id_result_type: _,
                id_result,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::Ordered {
                id_result_type: _,
                id_result,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::Unordered {
                id_result_type: _,
                id_result,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::LogicalEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::LogicalNotEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::LogicalOr {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::LogicalAnd {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::LogicalNot {
                id_result_type: _,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::Select {
                id_result_type: _id_result_type,
                id_result: _id_result,
                condition: _condition,
                object_1: _object_1,
                object_2: _object_2,
            } => unimplemented!(),
            Instruction::IEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::INotEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UGreaterThan {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SGreaterThan {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UGreaterThanEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SGreaterThanEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ULessThan {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SLessThan {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ULessThanEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SLessThanEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdNotEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordNotEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdLessThan {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordLessThan {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdGreaterThan {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordGreaterThan {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdLessThanEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordLessThanEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdGreaterThanEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordGreaterThanEqual {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ShiftRightLogical {
                id_result_type: _,
                id_result,
                base,
                shift,
            } => self.visit_simple_instruction(id_result, &[base, shift]),
            Instruction::ShiftRightArithmetic {
                id_result_type: _,
                id_result,
                base,
                shift,
            } => self.visit_simple_instruction(id_result, &[base, shift]),
            Instruction::ShiftLeftLogical {
                id_result_type: _,
                id_result,
                base,
                shift,
            } => self.visit_simple_instruction(id_result, &[base, shift]),
            Instruction::BitwiseOr {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::BitwiseXor {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::BitwiseAnd {
                id_result_type: _,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::Not {
                id_result_type: _,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::BitFieldInsert {
                id_result_type: _,
                id_result,
                base,
                insert,
                offset,
                count,
            } => self.visit_simple_instruction(id_result, &[base, insert, offset, count]),
            Instruction::BitFieldSExtract {
                id_result_type: _,
                id_result,
                base,
                offset,
                count,
            } => self.visit_simple_instruction(id_result, &[base, offset, count]),
            Instruction::BitFieldUExtract {
                id_result_type: _,
                id_result,
                base,
                offset,
                count,
            } => self.visit_simple_instruction(id_result, &[base, offset, count]),
            Instruction::BitReverse {
                id_result_type: _,
                id_result,
                base,
            } => self.visit_simple_instruction(id_result, iter::once(base)),
            Instruction::BitCount {
                id_result_type: _,
                id_result,
                base,
            } => self.visit_simple_instruction(id_result, iter::once(base)),
            Instruction::DPdx {
                id_result_type: _id_result_type,
                id_result: _id_result,
                p: _p,
            } => unimplemented!(),
            Instruction::DPdy {
                id_result_type: _id_result_type,
                id_result: _id_result,
                p: _p,
            } => unimplemented!(),
            Instruction::Fwidth {
                id_result_type: _id_result_type,
                id_result: _id_result,
                p: _p,
            } => unimplemented!(),
            Instruction::DPdxFine {
                id_result_type: _id_result_type,
                id_result: _id_result,
                p: _p,
            } => unimplemented!(),
            Instruction::DPdyFine {
                id_result_type: _id_result_type,
                id_result: _id_result,
                p: _p,
            } => unimplemented!(),
            Instruction::FwidthFine {
                id_result_type: _id_result_type,
                id_result: _id_result,
                p: _p,
            } => unimplemented!(),
            Instruction::DPdxCoarse {
                id_result_type: _id_result_type,
                id_result: _id_result,
                p: _p,
            } => unimplemented!(),
            Instruction::DPdyCoarse {
                id_result_type: _id_result_type,
                id_result: _id_result,
                p: _p,
            } => unimplemented!(),
            Instruction::FwidthCoarse {
                id_result_type: _id_result_type,
                id_result: _id_result,
                p: _p,
            } => unimplemented!(),
            Instruction::EmitVertex {} => unimplemented!(),
            Instruction::EndPrimitive {} => unimplemented!(),
            Instruction::EmitStreamVertex { stream: _stream } => unimplemented!(),
            Instruction::EndStreamPrimitive { stream: _stream } => unimplemented!(),
            Instruction::ControlBarrier {
                execution: _execution,
                memory: _memory,
                semantics: _semantics,
            } => unimplemented!(),
            Instruction::MemoryBarrier {
                memory: _memory,
                semantics: _semantics,
            } => unimplemented!(),
            Instruction::AtomicLoad {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
            } => unimplemented!(),
            Instruction::AtomicStore {
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicExchange {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicCompareExchange {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                equal: _equal,
                unequal: _unequal,
                value: _value,
                comparator: _comparator,
            } => unimplemented!(),
            Instruction::AtomicCompareExchangeWeak {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                equal: _equal,
                unequal: _unequal,
                value: _value,
                comparator: _comparator,
            } => unimplemented!(),
            Instruction::AtomicIIncrement {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
            } => unimplemented!(),
            Instruction::AtomicIDecrement {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
            } => unimplemented!(),
            Instruction::AtomicIAdd {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicISub {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicSMin {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicUMin {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicSMax {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicUMax {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicAnd {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicOr {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::AtomicXor {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
                value: _value,
            } => unimplemented!(),
            Instruction::Phi {
                id_result_type: _id_result_type,
                id_result: _id_result,
                variable_parent: ref _variable_parent,
            } => unimplemented!(),
            Instruction::LoopMerge { .. } | Instruction::SelectionMerge { .. } => {}
            Instruction::Label { .. } => {}
            Instruction::Branch { .. } => {
                self.visit_branch_instruction(node_index, ValueUniformity::Constant);
            }
            Instruction::BranchConditional { condition, .. } => {
                let condition_uniformity = self.get_entry(condition).value_uniformity;
                self.visit_branch_instruction(node_index, condition_uniformity);
            }
            Instruction::Switch32 { selector, .. } | Instruction::Switch64 { selector, .. } => {
                let condition_uniformity = self.get_entry(selector).value_uniformity;
                self.visit_branch_instruction(node_index, condition_uniformity);
            }
            Instruction::Kill {}
            | Instruction::Return {}
            | Instruction::ReturnValue { .. }
            | Instruction::Unreachable {} => {}
            Instruction::LifetimeStart {
                pointer: _pointer,
                size: _size,
            } => unimplemented!(),
            Instruction::LifetimeStop {
                pointer: _pointer,
                size: _size,
            } => unimplemented!(),
            Instruction::GroupAsyncCopy {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                destination: _destination,
                source: _source,
                num_elements: _num_elements,
                stride: _stride,
                event: _event,
            } => unimplemented!(),
            Instruction::GroupWaitEvents {
                execution: _execution,
                num_events: _num_events,
                events_list: _events_list,
            } => unimplemented!(),
            Instruction::GroupAll {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                predicate: _predicate,
            } => unimplemented!(),
            Instruction::GroupAny {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                predicate: _predicate,
            } => unimplemented!(),
            Instruction::GroupBroadcast {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
                local_id: _local_id,
            } => unimplemented!(),
            Instruction::GroupIAdd {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                x: _x,
            } => unimplemented!(),
            Instruction::GroupFAdd {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                x: _x,
            } => unimplemented!(),
            Instruction::GroupFMin {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                x: _x,
            } => unimplemented!(),
            Instruction::GroupUMin {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                x: _x,
            } => unimplemented!(),
            Instruction::GroupSMin {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                x: _x,
            } => unimplemented!(),
            Instruction::GroupFMax {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                x: _x,
            } => unimplemented!(),
            Instruction::GroupUMax {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                x: _x,
            } => unimplemented!(),
            Instruction::GroupSMax {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                x: _x,
            } => unimplemented!(),
            Instruction::ReadPipe {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pipe: _pipe,
                pointer: _pointer,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::WritePipe {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pipe: _pipe,
                pointer: _pointer,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::ReservedReadPipe {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pipe: _pipe,
                reserve_id: _reserve_id,
                index: _index,
                pointer: _pointer,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::ReservedWritePipe {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pipe: _pipe,
                reserve_id: _reserve_id,
                index: _index,
                pointer: _pointer,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::ReserveReadPipePackets {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pipe: _pipe,
                num_packets: _num_packets,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::ReserveWritePipePackets {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pipe: _pipe,
                num_packets: _num_packets,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::CommitReadPipe {
                pipe: _pipe,
                reserve_id: _reserve_id,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::CommitWritePipe {
                pipe: _pipe,
                reserve_id: _reserve_id,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::IsValidReserveId {
                id_result_type: _id_result_type,
                id_result: _id_result,
                reserve_id: _reserve_id,
            } => unimplemented!(),
            Instruction::GetNumPipePackets {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pipe: _pipe,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::GetMaxPipePackets {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pipe: _pipe,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::GroupReserveReadPipePackets {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                pipe: _pipe,
                num_packets: _num_packets,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::GroupReserveWritePipePackets {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                pipe: _pipe,
                num_packets: _num_packets,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::GroupCommitReadPipe {
                execution: _execution,
                pipe: _pipe,
                reserve_id: _reserve_id,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::GroupCommitWritePipe {
                execution: _execution,
                pipe: _pipe,
                reserve_id: _reserve_id,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
            } => unimplemented!(),
            Instruction::EnqueueMarker {
                id_result_type: _id_result_type,
                id_result: _id_result,
                queue: _queue,
                num_events: _num_events,
                wait_events: _wait_events,
                ret_event: _ret_event,
            } => unimplemented!(),
            Instruction::EnqueueKernel {
                id_result_type: _id_result_type,
                id_result: _id_result,
                queue: _queue,
                flags: _flags,
                nd_range: _nd_range,
                num_events: _num_events,
                wait_events: _wait_events,
                ret_event: _ret_event,
                invoke: _invoke,
                param: _param,
                param_size: _param_size,
                param_align: _param_align,
                local_size: ref _local_size,
            } => unimplemented!(),
            Instruction::GetKernelNDrangeSubGroupCount {
                id_result_type: _id_result_type,
                id_result: _id_result,
                nd_range: _nd_range,
                invoke: _invoke,
                param: _param,
                param_size: _param_size,
                param_align: _param_align,
            } => unimplemented!(),
            Instruction::GetKernelNDrangeMaxSubGroupSize {
                id_result_type: _id_result_type,
                id_result: _id_result,
                nd_range: _nd_range,
                invoke: _invoke,
                param: _param,
                param_size: _param_size,
                param_align: _param_align,
            } => unimplemented!(),
            Instruction::GetKernelWorkGroupSize {
                id_result_type: _id_result_type,
                id_result: _id_result,
                invoke: _invoke,
                param: _param,
                param_size: _param_size,
                param_align: _param_align,
            } => unimplemented!(),
            Instruction::GetKernelPreferredWorkGroupSizeMultiple {
                id_result_type: _id_result_type,
                id_result: _id_result,
                invoke: _invoke,
                param: _param,
                param_size: _param_size,
                param_align: _param_align,
            } => unimplemented!(),
            Instruction::RetainEvent { event: _event } => unimplemented!(),
            Instruction::ReleaseEvent { event: _event } => unimplemented!(),
            Instruction::CreateUserEvent {
                id_result_type: _id_result_type,
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::IsValidEvent {
                id_result_type: _id_result_type,
                id_result: _id_result,
                event: _event,
            } => unimplemented!(),
            Instruction::SetUserEventStatus {
                event: _event,
                status: _status,
            } => unimplemented!(),
            Instruction::CaptureEventProfilingInfo {
                event: _event,
                profiling_info: _profiling_info,
                value: _value,
            } => unimplemented!(),
            Instruction::GetDefaultQueue {
                id_result_type: _id_result_type,
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::BuildNDRange {
                id_result_type: _id_result_type,
                id_result: _id_result,
                global_work_size: _global_work_size,
                local_work_size: _local_work_size,
                global_work_offset: _global_work_offset,
            } => unimplemented!(),
            Instruction::ImageSparseSampleImplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseSampleExplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseSampleDrefImplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                d_ref: _d_ref,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseSampleDrefExplicitLod {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                d_ref: _d_ref,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseFetch {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseGather {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                component: _component,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseDrefGather {
                id_result_type: _id_result_type,
                id_result: _id_result,
                sampled_image: _sampled_image,
                coordinate: _coordinate,
                d_ref: _d_ref,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseTexelsResident {
                id_result_type: _id_result_type,
                id_result: _id_result,
                resident_code: _resident_code,
            } => unimplemented!(),
            Instruction::NoLine {} => {}
            Instruction::AtomicFlagTestAndSet {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
            } => unimplemented!(),
            Instruction::AtomicFlagClear {
                pointer: _pointer,
                scope: _scope,
                semantics: _semantics,
            } => unimplemented!(),
            Instruction::ImageSparseRead {
                id_result_type: _id_result_type,
                id_result: _id_result,
                image: _image,
                coordinate: _coordinate,
                image_operands: ref _image_operands,
            } => unimplemented!(),
            Instruction::SizeOf {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pointer: _pointer,
            } => unimplemented!(),
            Instruction::TypePipeStorage {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::ConstantPipeStorage {
                id_result_type: _id_result_type,
                id_result: _id_result,
                packet_size: _packet_size,
                packet_alignment: _packet_alignment,
                capacity: _capacity,
            } => unimplemented!(),
            Instruction::CreatePipeFromPipeStorage {
                id_result_type: _id_result_type,
                id_result: _id_result,
                pipe_storage: _pipe_storage,
            } => unimplemented!(),
            Instruction::GetKernelLocalSizeForSubgroupCount {
                id_result_type: _id_result_type,
                id_result: _id_result,
                subgroup_count: _subgroup_count,
                invoke: _invoke,
                param: _param,
                param_size: _param_size,
                param_align: _param_align,
            } => unimplemented!(),
            Instruction::GetKernelMaxNumSubgroups {
                id_result_type: _id_result_type,
                id_result: _id_result,
                invoke: _invoke,
                param: _param,
                param_size: _param_size,
                param_align: _param_align,
            } => unimplemented!(),
            Instruction::TypeNamedBarrier {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::NamedBarrierInitialize {
                id_result_type: _id_result_type,
                id_result: _id_result,
                subgroup_count: _subgroup_count,
            } => unimplemented!(),
            Instruction::MemoryNamedBarrier {
                named_barrier: _named_barrier,
                memory: _memory,
                semantics: _semantics,
            } => unimplemented!(),
            Instruction::ModuleProcessed {
                process: ref _process,
            } => unimplemented!(),
            Instruction::ExecutionModeId { .. } => {}
            Instruction::DecorateId { .. } => {}
            Instruction::GroupNonUniformElect {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
            } => unimplemented!(),
            Instruction::GroupNonUniformAll {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                predicate: _predicate,
            } => unimplemented!(),
            Instruction::GroupNonUniformAny {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                predicate: _predicate,
            } => unimplemented!(),
            Instruction::GroupNonUniformAllEqual {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBroadcast {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
                id: _id,
            } => unimplemented!(),
            Instruction::GroupNonUniformBroadcastFirst {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallot {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                predicate: _predicate,
            } => unimplemented!(),
            Instruction::GroupNonUniformInverseBallot {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallotBitExtract {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
                index: _index,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallotBitCount {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallotFindLSB {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallotFindMSB {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
            } => unimplemented!(),
            Instruction::GroupNonUniformShuffle {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
                id: _id,
            } => unimplemented!(),
            Instruction::GroupNonUniformShuffleXor {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
                mask: _mask,
            } => unimplemented!(),
            Instruction::GroupNonUniformShuffleUp {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
                delta: _delta,
            } => unimplemented!(),
            Instruction::GroupNonUniformShuffleDown {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
                delta: _delta,
            } => unimplemented!(),
            Instruction::GroupNonUniformIAdd {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformFAdd {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformIMul {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformFMul {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformSMin {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformUMin {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformFMin {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformSMax {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformUMax {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformFMax {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformBitwiseAnd {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformBitwiseOr {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformBitwiseXor {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformLogicalAnd {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformLogicalOr {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformLogicalXor {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                operation: _operation,
                value: _value,
                cluster_size: _cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformQuadBroadcast {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
                index: _index,
            } => unimplemented!(),
            Instruction::GroupNonUniformQuadSwap {
                id_result_type: _id_result_type,
                id_result: _id_result,
                execution: _execution,
                value: _value,
                direction: _direction,
            } => unimplemented!(),
            Instruction::ReportIntersectionNV {
                id_result_type: _id_result_type,
                id_result: _id_result,
                hit: _hit,
                hit_kind: _hit_kind,
            } => unimplemented!(),
            Instruction::IgnoreIntersectionNV {} => unimplemented!(),
            Instruction::TerminateRayNV {} => unimplemented!(),
            Instruction::TraceNV {
                accel: _accel,
                ray_flags: _ray_flags,
                cull_mask: _cull_mask,
                sbt_offset: _sbt_offset,
                sbt_stride: _sbt_stride,
                miss_index: _miss_index,
                ray_origin: _ray_origin,
                ray_tmin: _ray_tmin,
                ray_direction: _ray_direction,
                ray_tmax: _ray_tmax,
                payload_id: _payload_id,
            } => unimplemented!(),
            Instruction::TypeAccelerationStructureNV {
                id_result: _id_result,
            } => unimplemented!(),
            Instruction::ExecuteCallableNV {
                sbt_index: _sbt_index,
                callable_data_id: _callable_data_id,
            } => unimplemented!(),
            Instruction::OpenCLStdAcos {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAcosh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAcospi {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAsin {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAsinh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAsinpi {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAtan {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAtan2 {
                id_result_type: _,
                id_result,
                set: _,
                y,
                x,
            } => self.visit_simple_instruction(id_result, &[y, x]),
            Instruction::OpenCLStdAtanh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAtanpi {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAtan2pi {
                id_result_type: _,
                id_result,
                set: _,
                y,
                x,
            } => self.visit_simple_instruction(id_result, &[y, x]),
            Instruction::OpenCLStdCbrt {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCeil {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCopysign {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdCos {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCosh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCospi {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdErfc {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdErf {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdExp {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdExp2 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdExp10 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdExpm1 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdFabs {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdFdim {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFloor {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdFma {
                id_result_type: _,
                id_result,
                set: _,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdFmax {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFmin {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFmod {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFract {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                x: _x,
                ptr: _ptr,
            } => unimplemented!(),
            Instruction::OpenCLStdFrexp {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                x: _x,
                exp: _exp,
            } => unimplemented!(),
            Instruction::OpenCLStdHypot {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdIlogb {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLdexp {
                id_result_type: _,
                id_result,
                set: _,
                x,
                k,
            } => self.visit_simple_instruction(id_result, &[x, k]),
            Instruction::OpenCLStdLgamma {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLgammaR {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                x: _x,
                signp: _signp,
            } => unimplemented!(),
            Instruction::OpenCLStdLog {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLog2 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLog10 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLog1p {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLogb {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdMad {
                id_result_type: _,
                id_result,
                set: _,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdMaxmag {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdMinmag {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdModf {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                x: _x,
                iptr: _iptr,
            } => unimplemented!(),
            Instruction::OpenCLStdNan {
                id_result_type: _,
                id_result,
                set: _,
                nancode,
            } => self.visit_simple_instruction(id_result, iter::once(nancode)),
            Instruction::OpenCLStdNextafter {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdPow {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdPown {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdPowr {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdRemainder {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdRemquo {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                x: _x,
                y: _y,
                quo: _quo,
            } => unimplemented!(),
            Instruction::OpenCLStdRint {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdRootn {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdRound {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdRsqrt {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSin {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSincos {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                x: _x,
                cosval: _cosval,
            } => unimplemented!(),
            Instruction::OpenCLStdSinh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSinpi {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSqrt {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTan {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTanh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTanpi {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTgamma {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTrunc {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfCos {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfDivide {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdHalfExp {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfExp2 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfExp10 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfLog {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfLog2 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfLog10 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfPowr {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdHalfRecip {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfRsqrt {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfSin {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfSqrt {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfTan {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeCos {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeDivide {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdNativeExp {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeExp2 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeExp10 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeLog {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeLog2 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeLog10 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativePowr {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdNativeRecip {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeRsqrt {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeSin {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeSqrt {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeTan {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSAbs {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSAbsDiff {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSAddSat {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUAddSat {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSHadd {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUHadd {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSRhadd {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdURhadd {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSClamp {
                id_result_type: _,
                id_result,
                set: _,
                x,
                minval,
                maxval,
            } => self.visit_simple_instruction(id_result, &[x, minval, maxval]),
            Instruction::OpenCLStdUClamp {
                id_result_type: _,
                id_result,
                set: _,
                x,
                minval,
                maxval,
            } => self.visit_simple_instruction(id_result, &[x, minval, maxval]),
            Instruction::OpenCLStdClz {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCtz {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSMadHi {
                id_result_type: _,
                id_result,
                set: _,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdUMadSat {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
                z,
            } => self.visit_simple_instruction(id_result, &[x, y, z]),
            Instruction::OpenCLStdSMadSat {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
                z,
            } => self.visit_simple_instruction(id_result, &[x, y, z]),
            Instruction::OpenCLStdSMax {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMax {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSMin {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMin {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSMulHi {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdRotate {
                id_result_type: _,
                id_result,
                set: _,
                v,
                i,
            } => self.visit_simple_instruction(id_result, &[v, i]),
            Instruction::OpenCLStdSSubSat {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUSubSat {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUUpsample {
                id_result_type: _,
                id_result,
                set: _,
                hi,
                lo,
            } => self.visit_simple_instruction(id_result, &[hi, lo]),
            Instruction::OpenCLStdSUpsample {
                id_result_type: _,
                id_result,
                set: _,
                hi,
                lo,
            } => self.visit_simple_instruction(id_result, &[hi, lo]),
            Instruction::OpenCLStdPopcount {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSMad24 {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
                z,
            } => self.visit_simple_instruction(id_result, &[x, y, z]),
            Instruction::OpenCLStdUMad24 {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
                z,
            } => self.visit_simple_instruction(id_result, &[x, y, z]),
            Instruction::OpenCLStdSMul24 {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMul24 {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUAbs {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdUAbsDiff {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMulHi {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMadHi {
                id_result_type: _,
                id_result,
                set: _,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdFclamp {
                id_result_type: _,
                id_result,
                set: _,
                x,
                minval,
                maxval,
            } => self.visit_simple_instruction(id_result, &[x, minval, maxval]),
            Instruction::OpenCLStdDegrees {
                id_result_type: _,
                id_result,
                set: _,
                radians,
            } => self.visit_simple_instruction(id_result, iter::once(radians)),
            Instruction::OpenCLStdFmaxCommon {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFminCommon {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdMix {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
                a,
            } => self.visit_simple_instruction(id_result, &[x, y, a]),
            Instruction::OpenCLStdRadians {
                id_result_type: _,
                id_result,
                set: _,
                degrees,
            } => self.visit_simple_instruction(id_result, iter::once(degrees)),
            Instruction::OpenCLStdStep {
                id_result_type: _,
                id_result,
                set: _,
                edge,
                x,
            } => self.visit_simple_instruction(id_result, &[edge, x]),
            Instruction::OpenCLStdSmoothstep {
                id_result_type: _,
                id_result,
                set: _,
                edge0,
                edge1,
                x,
            } => self.visit_simple_instruction(id_result, &[edge0, edge1, x]),
            Instruction::OpenCLStdSign {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCross {
                id_result_type: _,
                id_result,
                set: _,
                p0,
                p1,
            } => self.visit_simple_instruction(id_result, &[p0, p1]),
            Instruction::OpenCLStdDistance {
                id_result_type: _,
                id_result,
                set: _,
                p0,
                p1,
            } => self.visit_simple_instruction(id_result, &[p0, p1]),
            Instruction::OpenCLStdLength {
                id_result_type: _,
                id_result,
                set: _,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::OpenCLStdNormalize {
                id_result_type: _,
                id_result,
                set: _,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::OpenCLStdFastDistance {
                id_result_type: _,
                id_result,
                set: _,
                p0,
                p1,
            } => self.visit_simple_instruction(id_result, &[p0, p1]),
            Instruction::OpenCLStdFastLength {
                id_result_type: _,
                id_result,
                set: _,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::OpenCLStdFastNormalize {
                id_result_type: _,
                id_result,
                set: _,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::OpenCLStdBitselect {
                id_result_type: _,
                id_result,
                set: _,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdSelect {
                id_result_type: _,
                id_result,
                set: _,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdVloadn {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                offset: _offset,
                p: _p,
                n: _n,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoren {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                data: _data,
                offset: _offset,
                p: _p,
            } => unimplemented!(),
            Instruction::OpenCLStdVloadHalf {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                offset: _offset,
                p: _p,
            } => unimplemented!(),
            Instruction::OpenCLStdVloadHalfn {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                offset: _offset,
                p: _p,
                n: _n,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalf {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                data: _data,
                offset: _offset,
                p: _p,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalfR {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                data: _data,
                offset: _offset,
                p: _p,
                mode: _mode,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalfn {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                data: _data,
                offset: _offset,
                p: _p,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalfnR {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                data: _data,
                offset: _offset,
                p: _p,
                mode: _mode,
            } => unimplemented!(),
            Instruction::OpenCLStdVloadaHalfn {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                offset: _offset,
                p: _p,
                n: _n,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreaHalfn {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                data: _data,
                offset: _offset,
                p: _p,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreaHalfnR {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                data: _data,
                offset: _offset,
                p: _p,
                mode: _mode,
            } => unimplemented!(),
            Instruction::OpenCLStdShuffle {
                id_result_type: _,
                id_result,
                set: _,
                x,
                shuffle_mask,
            } => self.visit_simple_instruction(id_result, &[x, shuffle_mask]),
            Instruction::OpenCLStdShuffle2 {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
                shuffle_mask,
            } => self.visit_simple_instruction(id_result, &[x, y, shuffle_mask]),
            Instruction::OpenCLStdPrintf {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                format: _format,
                additional_arguments: ref _additional_arguments,
            } => unimplemented!(),
            Instruction::OpenCLStdPrefetch {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                ptr: _ptr,
                num_elements: _num_elements,
            } => unimplemented!(),
            Instruction::GLSLStd450Round {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450RoundEven {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Trunc {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450FAbs {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450SAbs {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450FSign {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450SSign {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Floor {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Ceil {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Fract {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Radians {
                id_result_type: _,
                id_result,
                set: _,
                degrees,
            } => self.visit_simple_instruction(id_result, iter::once(degrees)),
            Instruction::GLSLStd450Degrees {
                id_result_type: _,
                id_result,
                set: _,
                radians,
            } => self.visit_simple_instruction(id_result, iter::once(radians)),
            Instruction::GLSLStd450Sin {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Cos {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Tan {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Asin {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Acos {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Atan {
                id_result_type: _,
                id_result,
                set: _,
                y_over_x,
            } => self.visit_simple_instruction(id_result, iter::once(y_over_x)),
            Instruction::GLSLStd450Sinh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Cosh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Tanh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Asinh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Acosh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Atanh {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Atan2 {
                id_result_type: _,
                id_result,
                set: _,
                y,
                x,
            } => self.visit_simple_instruction(id_result, &[y, x]),
            Instruction::GLSLStd450Pow {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450Exp {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Log {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Exp2 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Log2 {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Sqrt {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450InverseSqrt {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Determinant {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450MatrixInverse {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Modf {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                x: _x,
                i: _i,
            } => unimplemented!(),
            Instruction::GLSLStd450ModfStruct {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450FMin {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450UMin {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450SMin {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450FMax {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450UMax {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450SMax {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450FClamp {
                id_result_type: _,
                id_result,
                set: _,
                x,
                min_val,
                max_val,
            } => self.visit_simple_instruction(id_result, &[x, min_val, max_val]),
            Instruction::GLSLStd450UClamp {
                id_result_type: _,
                id_result,
                set: _,
                x,
                min_val,
                max_val,
            } => self.visit_simple_instruction(id_result, &[x, min_val, max_val]),
            Instruction::GLSLStd450SClamp {
                id_result_type: _,
                id_result,
                set: _,
                x,
                min_val,
                max_val,
            } => self.visit_simple_instruction(id_result, &[x, min_val, max_val]),
            Instruction::GLSLStd450FMix {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
                a,
            } => self.visit_simple_instruction(id_result, &[x, y, a]),
            Instruction::GLSLStd450IMix { .. } => {
                unreachable!("imix was removed from spec before release");
            }
            Instruction::GLSLStd450Step {
                id_result_type: _,
                id_result,
                set: _,
                edge,
                x,
            } => self.visit_simple_instruction(id_result, &[edge, x]),
            Instruction::GLSLStd450SmoothStep {
                id_result_type: _,
                id_result,
                set: _,
                edge0,
                edge1,
                x,
            } => self.visit_simple_instruction(id_result, &[edge0, edge1, x]),
            Instruction::GLSLStd450Fma {
                id_result_type: _,
                id_result,
                set: _,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::GLSLStd450Frexp {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                x: _x,
                exp: _exp,
            } => unimplemented!(),
            Instruction::GLSLStd450FrexpStruct {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Ldexp {
                id_result_type: _,
                id_result,
                set: _,
                x,
                exp,
            } => self.visit_simple_instruction(id_result, &[x, exp]),
            Instruction::GLSLStd450PackSnorm4x8 {
                id_result_type: _,
                id_result,
                set: _,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackUnorm4x8 {
                id_result_type: _,
                id_result,
                set: _,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackSnorm2x16 {
                id_result_type: _,
                id_result,
                set: _,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackUnorm2x16 {
                id_result_type: _,
                id_result,
                set: _,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackHalf2x16 {
                id_result_type: _,
                id_result,
                set: _,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackDouble2x32 {
                id_result_type: _,
                id_result,
                set: _,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450UnpackSnorm2x16 {
                id_result_type: _,
                id_result,
                set: _,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::GLSLStd450UnpackUnorm2x16 {
                id_result_type: _,
                id_result,
                set: _,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::GLSLStd450UnpackHalf2x16 {
                id_result_type: _,
                id_result,
                set: _,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450UnpackSnorm4x8 {
                id_result_type: _,
                id_result,
                set: _,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::GLSLStd450UnpackUnorm4x8 {
                id_result_type: _,
                id_result,
                set: _,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::GLSLStd450UnpackDouble2x32 {
                id_result_type: _,
                id_result,
                set: _,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450Length {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Distance {
                id_result_type: _,
                id_result,
                set: _,
                p0,
                p1,
            } => self.visit_simple_instruction(id_result, &[p0, p1]),
            Instruction::GLSLStd450Cross {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450Normalize {
                id_result_type: _,
                id_result,
                set: _,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450FaceForward {
                id_result_type: _,
                id_result,
                set: _,
                n,
                i,
                nref,
            } => self.visit_simple_instruction(id_result, &[n, i, nref]),
            Instruction::GLSLStd450Reflect {
                id_result_type: _,
                id_result,
                set: _,
                i,
                n,
            } => self.visit_simple_instruction(id_result, &[i, n]),
            Instruction::GLSLStd450Refract {
                id_result_type: _,
                id_result,
                set: _,
                i,
                n,
                eta,
            } => self.visit_simple_instruction(id_result, &[i, n, eta]),
            Instruction::GLSLStd450FindILsb {
                id_result_type: _,
                id_result,
                set: _,
                value,
            } => self.visit_simple_instruction(id_result, iter::once(value)),
            Instruction::GLSLStd450FindSMsb {
                id_result_type: _,
                id_result,
                set: _,
                value,
            } => self.visit_simple_instruction(id_result, iter::once(value)),
            Instruction::GLSLStd450FindUMsb {
                id_result_type: _,
                id_result,
                set: _,
                value,
            } => self.visit_simple_instruction(id_result, iter::once(value)),
            Instruction::GLSLStd450InterpolateAtCentroid {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                interpolant: _interpolant,
            } => unimplemented!(),
            Instruction::GLSLStd450InterpolateAtSample {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                interpolant: _interpolant,
                sample: _sample,
            } => unimplemented!(),
            Instruction::GLSLStd450InterpolateAtOffset {
                id_result_type: _id_result_type,
                id_result: _id_result,
                set: _set,
                interpolant: _interpolant,
                offset: _offset,
            } => unimplemented!(),
            Instruction::GLSLStd450NMin {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450NMax {
                id_result_type: _,
                id_result,
                set: _,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450NClamp {
                id_result_type: _,
                id_result,
                set: _,
                x,
                min_val,
                max_val,
            } => self.visit_simple_instruction(id_result, &[x, min_val, max_val]),
        }
    }
    fn run(mut self) -> ValueUniformities {
        for (id, id_properties) in self.ids.iter() {
            match &id_properties.kind {
                IdKind::Undefined => {}
                IdKind::DecorationGroup => {}
                IdKind::Type(..) => {}
                IdKind::VoidType => {}
                IdKind::FunctionType { .. } => {}
                IdKind::ForwardPointer(..) => {}
                IdKind::BuiltInVariable(built_in_variable) => self.set_entry(
                    id,
                    get_built_in_initial_value_uniformity_entry(built_in_variable, id),
                ),
                IdKind::Constant(..) => self.set_entry(
                    id,
                    ValueUniformityEntry {
                        value_uniformity: ValueUniformity::Constant,
                        pointee_uniformity: None,
                    },
                ),
                IdKind::UniformVariable(..) => self.set_entry(
                    id,
                    ValueUniformityEntry {
                        value_uniformity: ValueUniformity::UniformOverWorkgroup,
                        pointee_uniformity: Some(PointeeUniformity {
                            value_uniformity: ValueUniformity::UniformOverWorkgroup,
                            variables: VariableSet::from(id),
                        }),
                    },
                ),
                IdKind::Function(..) => {}
                IdKind::BasicBlock { .. } => {}
                IdKind::Value(..) => {}
            }
        }
        let basic_blocks: Vec<CFGNodeIndex> =
            self.cfg.structure_tree().basic_blocks_in_order().collect();
        loop {
            self.any_changes = false;
            for &basic_block in basic_blocks.iter() {
                let value_uniformity = self.calculate_block_value_uniformity(basic_block);
                self.set_entry(basic_block, BasicBlockUniformityEntry { value_uniformity });
                for instruction in self.cfg[basic_block].instructions().iter() {
                    self.visit_instruction(basic_block, instruction);
                }
            }
            if !self.any_changes {
                break;
            }
        }
        ValueUniformities {
            entries: self.entries,
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
        ValueUniformityCalculator::new(cfg, ids).run()
    }
}
