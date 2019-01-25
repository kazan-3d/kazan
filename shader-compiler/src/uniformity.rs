// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

mod anf;
mod variable_set;

use self::variable_set::VariableSet;
use crate::cfg::{CFGNodeIndex, CFG};
use crate::lattice::{BoundedOrderedLattice, MeetSemilattice};
use crate::BuiltInVariable;
use crate::IdKind;
use crate::Ids;
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct BasicBlockUniformityEntry {
    value_uniformity: ValueUniformity,
}

impl BasicBlockUniformityEntry {
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
struct ValueUniformityEntry {
    value_uniformity: ValueUniformity,
    pointee_uniformity: Option<PointeeUniformity>,
}

impl ValueUniformityEntry {
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

struct ValueUniformityCalculator<'a, 'ctx, C: shader_compiler_backend::Context<'ctx>> {
    entries: HashMap<IdRef, ValueUniformityEntry>,
    basic_blocks: HashMap<CFGNodeIndex, BasicBlockUniformityEntry>,
    cfg: &'a CFG,
    ids: &'a Ids<'ctx, C>,
    any_changes: bool,
}

impl<'a, 'ctx, C: shader_compiler_backend::Context<'ctx>> ValueUniformityCalculator<'a, 'ctx, C> {
    fn new(cfg: &'a CFG, ids: &'a Ids<'ctx, C>) -> Self
    where
        <CFG as Deref>::Target: NodeCompactIndexable,
    {
        let mut basic_blocks = HashMap::new();
        ValueUniformityCalculator {
            entries: HashMap::new(),
            basic_blocks,
            cfg,
            ids,
            any_changes: false,
        }
    }
    fn get_basic_block(&mut self, node_index: CFGNodeIndex) -> BasicBlockUniformityEntry {
        if let Some(v) = self.basic_blocks.get(&node_index) {
            v.clone()
        } else {
            Default::default()
        }
    }
    fn with_basic_block<F: FnOnce(&mut BasicBlockUniformityEntry)>(
        &mut self,
        node_index: CFGNodeIndex,
        f: F,
    ) {
        use std::collections::hash_map::Entry;
        match self.basic_blocks.entry(node_index) {
            Entry::Vacant(entry) => {
                let mut value = BasicBlockUniformityEntry::default();
                f(&mut value);
                if value != Default::default() {
                    entry.insert(value);
                    self.any_changes = true;
                }
            }
            Entry::Occupied(entry) => {
                let entry = entry.into_mut();
                let mut value = entry.clone();
                f(&mut value);
                if value != *entry {
                    entry.check_update_with(&value);
                    *entry = value;
                    self.any_changes = true;
                }
            }
        }
    }
    fn set_basic_block(&mut self, node_index: CFGNodeIndex, v: BasicBlockUniformityEntry) {
        self.with_basic_block(node_index, |value| *value = v);
    }
    fn set_entry(&mut self, id: IdRef, v: ValueUniformityEntry) {
        self.with_entry(id, |value| *value = v);
    }
    fn get_entry(&mut self, id: IdRef) -> ValueUniformityEntry {
        if let Some(v) = self.entries.get(&id) {
            v.clone()
        } else {
            Default::default()
        }
    }
    fn with_entry<F: FnOnce(&mut ValueUniformityEntry)>(&mut self, id: IdRef, f: F) {
        use std::collections::hash_map::Entry;
        match self.entries.entry(id) {
            Entry::Vacant(entry) => {
                let mut value = ValueUniformityEntry::default();
                f(&mut value);
                if value != Default::default() {
                    entry.insert(value);
                    self.any_changes = true;
                }
            }
            Entry::Occupied(entry) => {
                let entry = entry.into_mut();
                let mut value = entry.clone();
                f(&mut value);
                if value != *entry {
                    entry.check_update_with(&value);
                    *entry = value;
                    self.any_changes = true;
                }
            }
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
    fn visit_instruction(&mut self, node_index: CFGNodeIndex, instruction: &Instruction) {
        #[allow(unused_variables)]
        match *instruction {
            Instruction::Nop {} => {}
            Instruction::Undef {
                id_result_type,
                id_result,
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
            Instruction::ExtInst {
                id_result_type,
                id_result,
                set,
                instruction,
                ref operands,
            } => unreachable!("unimplemented OpExtInst:\n{}", instruction),
            Instruction::MemoryModel { .. } => {}
            Instruction::EntryPoint { .. } => {}
            Instruction::ExecutionMode {
                entry_point,
                ref mode,
            } => unimplemented!(),
            Instruction::Capability { capability } => unimplemented!(),
            Instruction::TypeVoid { id_result } => unimplemented!(),
            Instruction::TypeBool { id_result } => unimplemented!(),
            Instruction::TypeInt {
                id_result,
                width,
                signedness,
            } => unimplemented!(),
            Instruction::TypeFloat { id_result, width } => unimplemented!(),
            Instruction::TypeVector {
                id_result,
                component_type,
                component_count,
            } => unimplemented!(),
            Instruction::TypeMatrix {
                id_result,
                column_type,
                column_count,
            } => unimplemented!(),
            Instruction::TypeImage {
                id_result,
                sampled_type,
                dim,
                depth,
                arrayed,
                ms,
                sampled,
                image_format,
                access_qualifier,
            } => unimplemented!(),
            Instruction::TypeSampler { id_result } => unimplemented!(),
            Instruction::TypeSampledImage {
                id_result,
                image_type,
            } => unimplemented!(),
            Instruction::TypeArray {
                id_result,
                element_type,
                length,
            } => unimplemented!(),
            Instruction::TypeRuntimeArray {
                id_result,
                element_type,
            } => unimplemented!(),
            Instruction::TypeStruct {
                id_result,
                ref member_types,
            } => unimplemented!(),
            Instruction::TypeOpaque {
                id_result,
                ref the_name_of_the_opaque_type,
            } => unimplemented!(),
            Instruction::TypePointer {
                id_result,
                storage_class,
                type_,
            } => unimplemented!(),
            Instruction::TypeFunction {
                id_result,
                return_type,
                ref parameter_types,
            } => unimplemented!(),
            Instruction::TypeEvent { id_result } => unimplemented!(),
            Instruction::TypeDeviceEvent { id_result } => unimplemented!(),
            Instruction::TypeReserveId { id_result } => unimplemented!(),
            Instruction::TypeQueue { id_result } => unimplemented!(),
            Instruction::TypePipe {
                id_result,
                qualifier,
            } => unimplemented!(),
            Instruction::TypeForwardPointer {
                pointer_type,
                storage_class,
            } => unimplemented!(),
            Instruction::ConstantTrue {
                id_result_type,
                id_result,
            } => unimplemented!(),
            Instruction::ConstantFalse {
                id_result_type,
                id_result,
            } => unimplemented!(),
            Instruction::Constant32 {
                id_result_type,
                id_result,
                value,
            } => unimplemented!(),
            Instruction::Constant64 {
                id_result_type,
                id_result,
                value,
            } => unimplemented!(),
            Instruction::ConstantComposite {
                id_result_type,
                id_result,
                ref constituents,
            } => unimplemented!(),
            Instruction::ConstantSampler {
                id_result_type,
                id_result,
                sampler_addressing_mode,
                param,
                sampler_filter_mode,
            } => unimplemented!(),
            Instruction::ConstantNull {
                id_result_type,
                id_result,
            } => unimplemented!(),
            Instruction::SpecConstantTrue {
                id_result_type,
                id_result,
            } => unimplemented!(),
            Instruction::SpecConstantFalse {
                id_result_type,
                id_result,
            } => unimplemented!(),
            Instruction::SpecConstant32 {
                id_result_type,
                id_result,
                value,
            } => unimplemented!(),
            Instruction::SpecConstant64 {
                id_result_type,
                id_result,
                value,
            } => unimplemented!(),
            Instruction::SpecConstantComposite {
                id_result_type,
                id_result,
                ref constituents,
            } => unimplemented!(),
            Instruction::SpecConstantOp { ref operation } => unimplemented!(),
            Instruction::Function {
                id_result_type,
                id_result,
                ref function_control,
                function_type,
            } => unimplemented!(),
            Instruction::FunctionParameter {
                id_result_type,
                id_result,
            } => unimplemented!(),
            Instruction::FunctionEnd {} => unimplemented!(),
            Instruction::FunctionCall {
                id_result_type,
                id_result,
                function,
                ref arguments,
            } => unimplemented!(),
            Instruction::Variable {
                id_result_type,
                id_result,
                storage_class,
                initializer,
            } => {
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
                id_result_type,
                id_result,
                image,
                coordinate,
                sample,
            } => unimplemented!(),
            Instruction::Load {
                id_result_type,
                id_result,
                pointer,
                ref memory_access,
            } => {
                let ValueUniformityEntry {
                    value_uniformity,
                    pointee_uniformity,
                } = self.get_entry(pointer);
                let pointee_uniformity = pointee_uniformity.expect("pointer");
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
                pointer,
                object,
                ref memory_access,
            } => unimplemented!(),
            Instruction::CopyMemory {
                target,
                source,
                ref memory_access,
            } => unimplemented!(),
            Instruction::CopyMemorySized {
                target,
                source,
                size,
                ref memory_access,
            } => unimplemented!(),
            Instruction::AccessChain {
                id_result_type,
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
                id_result_type,
                id_result,
                base,
                ref indexes,
            } => unimplemented!(),
            Instruction::PtrAccessChain {
                id_result_type,
                id_result,
                base,
                element,
                ref indexes,
            } => unimplemented!(),
            Instruction::ArrayLength {
                id_result_type,
                id_result,
                structure,
                array_member,
            } => unimplemented!(),
            Instruction::GenericPtrMemSemantics {
                id_result_type,
                id_result,
                pointer,
            } => unimplemented!(),
            Instruction::InBoundsPtrAccessChain {
                id_result_type,
                id_result,
                base,
                element,
                ref indexes,
            } => unimplemented!(),
            Instruction::Decorate {
                target,
                ref decoration,
            } => unimplemented!(),
            Instruction::MemberDecorate {
                structure_type,
                member,
                ref decoration,
            } => unimplemented!(),
            Instruction::DecorationGroup { id_result } => unimplemented!(),
            Instruction::GroupDecorate {
                decoration_group,
                ref targets,
            } => unimplemented!(),
            Instruction::GroupMemberDecorate {
                decoration_group,
                ref targets,
            } => unimplemented!(),
            Instruction::VectorExtractDynamic {
                id_result_type,
                id_result,
                vector,
                index,
            } => self.visit_simple_instruction(id_result, &[vector, index]),
            Instruction::VectorInsertDynamic {
                id_result_type,
                id_result,
                vector,
                component,
                index,
            } => self.visit_simple_instruction(id_result, &[vector, component, index]),
            Instruction::VectorShuffle {
                id_result_type,
                id_result,
                vector_1,
                vector_2,
                ref components,
            } => self.visit_simple_instruction(id_result, &[vector_1, vector_2]),
            Instruction::CompositeConstruct {
                id_result_type,
                id_result,
                ref constituents,
            } => self.visit_simple_instruction(id_result, constituents),
            Instruction::CompositeExtract {
                id_result_type,
                id_result,
                composite,
                ref indexes,
            } => self.visit_simple_instruction(id_result, iter::once(composite)),
            Instruction::CompositeInsert {
                id_result_type,
                id_result,
                object,
                composite,
                ref indexes,
            } => self.visit_simple_instruction(id_result, &[object, composite]),
            Instruction::CopyObject {
                id_result_type,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::Transpose {
                id_result_type,
                id_result,
                matrix,
            } => self.visit_simple_instruction(id_result, iter::once(matrix)),
            Instruction::SampledImage {
                id_result_type,
                id_result,
                image,
                sampler,
            } => self.visit_simple_instruction(id_result, &[image, sampler]),
            Instruction::ImageSampleImplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleExplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleDrefImplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                d_ref,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleDrefExplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                d_ref,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleProjImplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleProjExplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleProjDrefImplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                d_ref,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSampleProjDrefExplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                d_ref,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageFetch {
                id_result_type,
                id_result,
                image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageGather {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                component,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageDrefGather {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                d_ref,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageRead {
                id_result_type,
                id_result,
                image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageWrite {
                image,
                coordinate,
                texel,
                ref image_operands,
            } => unimplemented!(),
            Instruction::Image {
                id_result_type,
                id_result,
                sampled_image,
            } => self.visit_simple_instruction(id_result, iter::once(sampled_image)),
            Instruction::ImageQueryFormat {
                id_result_type,
                id_result,
                image,
            } => unimplemented!(),
            Instruction::ImageQueryOrder {
                id_result_type,
                id_result,
                image,
            } => unimplemented!(),
            Instruction::ImageQuerySizeLod {
                id_result_type,
                id_result,
                image,
                level_of_detail,
            } => unimplemented!(),
            Instruction::ImageQuerySize {
                id_result_type,
                id_result,
                image,
            } => unimplemented!(),
            Instruction::ImageQueryLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
            } => unimplemented!(),
            Instruction::ImageQueryLevels {
                id_result_type,
                id_result,
                image,
            } => unimplemented!(),
            Instruction::ImageQuerySamples {
                id_result_type,
                id_result,
                image,
            } => unimplemented!(),
            Instruction::ConvertFToU {
                id_result_type,
                id_result,
                float_value,
            } => self.visit_simple_instruction(id_result, iter::once(float_value)),
            Instruction::ConvertFToS {
                id_result_type,
                id_result,
                float_value,
            } => self.visit_simple_instruction(id_result, iter::once(float_value)),
            Instruction::ConvertSToF {
                id_result_type,
                id_result,
                signed_value,
            } => self.visit_simple_instruction(id_result, iter::once(signed_value)),
            Instruction::ConvertUToF {
                id_result_type,
                id_result,
                unsigned_value,
            } => self.visit_simple_instruction(id_result, iter::once(unsigned_value)),
            Instruction::UConvert {
                id_result_type,
                id_result,
                unsigned_value,
            } => self.visit_simple_instruction(id_result, iter::once(unsigned_value)),
            Instruction::SConvert {
                id_result_type,
                id_result,
                signed_value,
            } => self.visit_simple_instruction(id_result, iter::once(signed_value)),
            Instruction::FConvert {
                id_result_type,
                id_result,
                float_value,
            } => self.visit_simple_instruction(id_result, iter::once(float_value)),
            Instruction::QuantizeToF16 {
                id_result_type,
                id_result,
                value,
            } => self.visit_simple_instruction(id_result, iter::once(value)),
            Instruction::ConvertPtrToU {
                id_result_type,
                id_result,
                pointer,
            } => unimplemented!(),
            Instruction::SatConvertSToU {
                id_result_type,
                id_result,
                signed_value,
            } => unimplemented!(),
            Instruction::SatConvertUToS {
                id_result_type,
                id_result,
                unsigned_value,
            } => unimplemented!(),
            Instruction::ConvertUToPtr {
                id_result_type,
                id_result,
                integer_value,
            } => unimplemented!(),
            Instruction::PtrCastToGeneric {
                id_result_type,
                id_result,
                pointer,
            } => unimplemented!(),
            Instruction::GenericCastToPtr {
                id_result_type,
                id_result,
                pointer,
            } => unimplemented!(),
            Instruction::GenericCastToPtrExplicit {
                id_result_type,
                id_result,
                pointer,
                storage,
            } => unimplemented!(),
            Instruction::Bitcast {
                id_result_type,
                id_result,
                operand,
            } => unimplemented!(),
            Instruction::SNegate {
                id_result_type,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::FNegate {
                id_result_type,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::IAdd {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FAdd {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ISub {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FSub {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::IMul {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FMul {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UDiv {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SDiv {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FDiv {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UMod {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SRem {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SMod {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FRem {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FMod {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::VectorTimesScalar {
                id_result_type,
                id_result,
                vector,
                scalar,
            } => self.visit_simple_instruction(id_result, &[vector, scalar]),
            Instruction::MatrixTimesScalar {
                id_result_type,
                id_result,
                matrix,
                scalar,
            } => self.visit_simple_instruction(id_result, &[matrix, scalar]),
            Instruction::VectorTimesMatrix {
                id_result_type,
                id_result,
                vector,
                matrix,
            } => self.visit_simple_instruction(id_result, &[vector, matrix]),
            Instruction::MatrixTimesVector {
                id_result_type,
                id_result,
                matrix,
                vector,
            } => self.visit_simple_instruction(id_result, &[matrix, vector]),
            Instruction::MatrixTimesMatrix {
                id_result_type,
                id_result,
                left_matrix,
                right_matrix,
            } => self.visit_simple_instruction(id_result, &[left_matrix, right_matrix]),
            Instruction::OuterProduct {
                id_result_type,
                id_result,
                vector_1,
                vector_2,
            } => self.visit_simple_instruction(id_result, &[vector_1, vector_2]),
            Instruction::Dot {
                id_result_type,
                id_result,
                vector_1,
                vector_2,
            } => self.visit_simple_instruction(id_result, &[vector_1, vector_2]),
            Instruction::IAddCarry {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ISubBorrow {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UMulExtended {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SMulExtended {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::Any {
                id_result_type,
                id_result,
                vector,
            } => self.visit_simple_instruction(id_result, iter::once(vector)),
            Instruction::All {
                id_result_type,
                id_result,
                vector,
            } => self.visit_simple_instruction(id_result, iter::once(vector)),
            Instruction::IsNan {
                id_result_type,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::IsInf {
                id_result_type,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::IsFinite {
                id_result_type,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::IsNormal {
                id_result_type,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::SignBitSet {
                id_result_type,
                id_result,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::LessOrGreater {
                id_result_type,
                id_result,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::Ordered {
                id_result_type,
                id_result,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::Unordered {
                id_result_type,
                id_result,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::LogicalEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::LogicalNotEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::LogicalOr {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::LogicalAnd {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::LogicalNot {
                id_result_type,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::Select {
                id_result_type,
                id_result,
                condition,
                object_1,
                object_2,
            } => unimplemented!(),
            Instruction::IEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::INotEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UGreaterThan {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SGreaterThan {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::UGreaterThanEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SGreaterThanEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ULessThan {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SLessThan {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ULessThanEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::SLessThanEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdNotEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordNotEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdLessThan {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordLessThan {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdGreaterThan {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordGreaterThan {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdLessThanEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordLessThanEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FOrdGreaterThanEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::FUnordGreaterThanEqual {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::ShiftRightLogical {
                id_result_type,
                id_result,
                base,
                shift,
            } => self.visit_simple_instruction(id_result, &[base, shift]),
            Instruction::ShiftRightArithmetic {
                id_result_type,
                id_result,
                base,
                shift,
            } => self.visit_simple_instruction(id_result, &[base, shift]),
            Instruction::ShiftLeftLogical {
                id_result_type,
                id_result,
                base,
                shift,
            } => self.visit_simple_instruction(id_result, &[base, shift]),
            Instruction::BitwiseOr {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::BitwiseXor {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::BitwiseAnd {
                id_result_type,
                id_result,
                operand_1,
                operand_2,
            } => self.visit_simple_instruction(id_result, &[operand_1, operand_2]),
            Instruction::Not {
                id_result_type,
                id_result,
                operand,
            } => self.visit_simple_instruction(id_result, iter::once(operand)),
            Instruction::BitFieldInsert {
                id_result_type,
                id_result,
                base,
                insert,
                offset,
                count,
            } => self.visit_simple_instruction(id_result, &[base, insert, offset, count]),
            Instruction::BitFieldSExtract {
                id_result_type,
                id_result,
                base,
                offset,
                count,
            } => self.visit_simple_instruction(id_result, &[base, offset, count]),
            Instruction::BitFieldUExtract {
                id_result_type,
                id_result,
                base,
                offset,
                count,
            } => self.visit_simple_instruction(id_result, &[base, offset, count]),
            Instruction::BitReverse {
                id_result_type,
                id_result,
                base,
            } => self.visit_simple_instruction(id_result, iter::once(base)),
            Instruction::BitCount {
                id_result_type,
                id_result,
                base,
            } => self.visit_simple_instruction(id_result, iter::once(base)),
            Instruction::DPdx {
                id_result_type,
                id_result,
                p,
            } => unimplemented!(),
            Instruction::DPdy {
                id_result_type,
                id_result,
                p,
            } => unimplemented!(),
            Instruction::Fwidth {
                id_result_type,
                id_result,
                p,
            } => unimplemented!(),
            Instruction::DPdxFine {
                id_result_type,
                id_result,
                p,
            } => unimplemented!(),
            Instruction::DPdyFine {
                id_result_type,
                id_result,
                p,
            } => unimplemented!(),
            Instruction::FwidthFine {
                id_result_type,
                id_result,
                p,
            } => unimplemented!(),
            Instruction::DPdxCoarse {
                id_result_type,
                id_result,
                p,
            } => unimplemented!(),
            Instruction::DPdyCoarse {
                id_result_type,
                id_result,
                p,
            } => unimplemented!(),
            Instruction::FwidthCoarse {
                id_result_type,
                id_result,
                p,
            } => unimplemented!(),
            Instruction::EmitVertex {} => unimplemented!(),
            Instruction::EndPrimitive {} => unimplemented!(),
            Instruction::EmitStreamVertex { stream } => unimplemented!(),
            Instruction::EndStreamPrimitive { stream } => unimplemented!(),
            Instruction::ControlBarrier {
                execution,
                memory,
                semantics,
            } => unimplemented!(),
            Instruction::MemoryBarrier { memory, semantics } => unimplemented!(),
            Instruction::AtomicLoad {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
            } => unimplemented!(),
            Instruction::AtomicStore {
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicExchange {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicCompareExchange {
                id_result_type,
                id_result,
                pointer,
                scope,
                equal,
                unequal,
                value,
                comparator,
            } => unimplemented!(),
            Instruction::AtomicCompareExchangeWeak {
                id_result_type,
                id_result,
                pointer,
                scope,
                equal,
                unequal,
                value,
                comparator,
            } => unimplemented!(),
            Instruction::AtomicIIncrement {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
            } => unimplemented!(),
            Instruction::AtomicIDecrement {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
            } => unimplemented!(),
            Instruction::AtomicIAdd {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicISub {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicSMin {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicUMin {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicSMax {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicUMax {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicAnd {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicOr {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::AtomicXor {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
                value,
            } => unimplemented!(),
            Instruction::Phi {
                id_result_type,
                id_result,
                ref variable_parent,
            } => unimplemented!(),
            Instruction::LoopMerge {
                merge_block,
                continue_target,
                ref loop_control,
            } => {}
            Instruction::SelectionMerge {
                merge_block,
                ref selection_control,
            } => {}
            Instruction::Label { .. } => {}
            Instruction::Branch { target_label } => unimplemented!(),
            Instruction::BranchConditional {
                condition,
                true_label,
                false_label,
                ref branch_weights,
            } => {
                unimplemented!();
            }
            Instruction::Switch32 {
                selector,
                default,
                ref target,
            } => unimplemented!(),
            Instruction::Switch64 {
                selector,
                default,
                ref target,
            } => unimplemented!(),
            Instruction::Kill {} => unimplemented!(),
            Instruction::Return {} => unimplemented!(),
            Instruction::ReturnValue { value } => unimplemented!(),
            Instruction::Unreachable {} => unimplemented!(),
            Instruction::LifetimeStart { pointer, size } => unimplemented!(),
            Instruction::LifetimeStop { pointer, size } => unimplemented!(),
            Instruction::GroupAsyncCopy {
                id_result_type,
                id_result,
                execution,
                destination,
                source,
                num_elements,
                stride,
                event,
            } => unimplemented!(),
            Instruction::GroupWaitEvents {
                execution,
                num_events,
                events_list,
            } => unimplemented!(),
            Instruction::GroupAll {
                id_result_type,
                id_result,
                execution,
                predicate,
            } => unimplemented!(),
            Instruction::GroupAny {
                id_result_type,
                id_result,
                execution,
                predicate,
            } => unimplemented!(),
            Instruction::GroupBroadcast {
                id_result_type,
                id_result,
                execution,
                value,
                local_id,
            } => unimplemented!(),
            Instruction::GroupIAdd {
                id_result_type,
                id_result,
                execution,
                operation,
                x,
            } => unimplemented!(),
            Instruction::GroupFAdd {
                id_result_type,
                id_result,
                execution,
                operation,
                x,
            } => unimplemented!(),
            Instruction::GroupFMin {
                id_result_type,
                id_result,
                execution,
                operation,
                x,
            } => unimplemented!(),
            Instruction::GroupUMin {
                id_result_type,
                id_result,
                execution,
                operation,
                x,
            } => unimplemented!(),
            Instruction::GroupSMin {
                id_result_type,
                id_result,
                execution,
                operation,
                x,
            } => unimplemented!(),
            Instruction::GroupFMax {
                id_result_type,
                id_result,
                execution,
                operation,
                x,
            } => unimplemented!(),
            Instruction::GroupUMax {
                id_result_type,
                id_result,
                execution,
                operation,
                x,
            } => unimplemented!(),
            Instruction::GroupSMax {
                id_result_type,
                id_result,
                execution,
                operation,
                x,
            } => unimplemented!(),
            Instruction::ReadPipe {
                id_result_type,
                id_result,
                pipe,
                pointer,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::WritePipe {
                id_result_type,
                id_result,
                pipe,
                pointer,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::ReservedReadPipe {
                id_result_type,
                id_result,
                pipe,
                reserve_id,
                index,
                pointer,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::ReservedWritePipe {
                id_result_type,
                id_result,
                pipe,
                reserve_id,
                index,
                pointer,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::ReserveReadPipePackets {
                id_result_type,
                id_result,
                pipe,
                num_packets,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::ReserveWritePipePackets {
                id_result_type,
                id_result,
                pipe,
                num_packets,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::CommitReadPipe {
                pipe,
                reserve_id,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::CommitWritePipe {
                pipe,
                reserve_id,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::IsValidReserveId {
                id_result_type,
                id_result,
                reserve_id,
            } => unimplemented!(),
            Instruction::GetNumPipePackets {
                id_result_type,
                id_result,
                pipe,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::GetMaxPipePackets {
                id_result_type,
                id_result,
                pipe,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::GroupReserveReadPipePackets {
                id_result_type,
                id_result,
                execution,
                pipe,
                num_packets,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::GroupReserveWritePipePackets {
                id_result_type,
                id_result,
                execution,
                pipe,
                num_packets,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::GroupCommitReadPipe {
                execution,
                pipe,
                reserve_id,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::GroupCommitWritePipe {
                execution,
                pipe,
                reserve_id,
                packet_size,
                packet_alignment,
            } => unimplemented!(),
            Instruction::EnqueueMarker {
                id_result_type,
                id_result,
                queue,
                num_events,
                wait_events,
                ret_event,
            } => unimplemented!(),
            Instruction::EnqueueKernel {
                id_result_type,
                id_result,
                queue,
                flags,
                nd_range,
                num_events,
                wait_events,
                ret_event,
                invoke,
                param,
                param_size,
                param_align,
                ref local_size,
            } => unimplemented!(),
            Instruction::GetKernelNDrangeSubGroupCount {
                id_result_type,
                id_result,
                nd_range,
                invoke,
                param,
                param_size,
                param_align,
            } => unimplemented!(),
            Instruction::GetKernelNDrangeMaxSubGroupSize {
                id_result_type,
                id_result,
                nd_range,
                invoke,
                param,
                param_size,
                param_align,
            } => unimplemented!(),
            Instruction::GetKernelWorkGroupSize {
                id_result_type,
                id_result,
                invoke,
                param,
                param_size,
                param_align,
            } => unimplemented!(),
            Instruction::GetKernelPreferredWorkGroupSizeMultiple {
                id_result_type,
                id_result,
                invoke,
                param,
                param_size,
                param_align,
            } => unimplemented!(),
            Instruction::RetainEvent { event } => unimplemented!(),
            Instruction::ReleaseEvent { event } => unimplemented!(),
            Instruction::CreateUserEvent {
                id_result_type,
                id_result,
            } => unimplemented!(),
            Instruction::IsValidEvent {
                id_result_type,
                id_result,
                event,
            } => unimplemented!(),
            Instruction::SetUserEventStatus { event, status } => unimplemented!(),
            Instruction::CaptureEventProfilingInfo {
                event,
                profiling_info,
                value,
            } => unimplemented!(),
            Instruction::GetDefaultQueue {
                id_result_type,
                id_result,
            } => unimplemented!(),
            Instruction::BuildNDRange {
                id_result_type,
                id_result,
                global_work_size,
                local_work_size,
                global_work_offset,
            } => unimplemented!(),
            Instruction::ImageSparseSampleImplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseSampleExplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseSampleDrefImplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                d_ref,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseSampleDrefExplicitLod {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                d_ref,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseFetch {
                id_result_type,
                id_result,
                image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseGather {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                component,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseDrefGather {
                id_result_type,
                id_result,
                sampled_image,
                coordinate,
                d_ref,
                ref image_operands,
            } => unimplemented!(),
            Instruction::ImageSparseTexelsResident {
                id_result_type,
                id_result,
                resident_code,
            } => unimplemented!(),
            Instruction::NoLine {} => {}
            Instruction::AtomicFlagTestAndSet {
                id_result_type,
                id_result,
                pointer,
                scope,
                semantics,
            } => unimplemented!(),
            Instruction::AtomicFlagClear {
                pointer,
                scope,
                semantics,
            } => unimplemented!(),
            Instruction::ImageSparseRead {
                id_result_type,
                id_result,
                image,
                coordinate,
                ref image_operands,
            } => unimplemented!(),
            Instruction::SizeOf {
                id_result_type,
                id_result,
                pointer,
            } => unimplemented!(),
            Instruction::TypePipeStorage { id_result } => unimplemented!(),
            Instruction::ConstantPipeStorage {
                id_result_type,
                id_result,
                packet_size,
                packet_alignment,
                capacity,
            } => unimplemented!(),
            Instruction::CreatePipeFromPipeStorage {
                id_result_type,
                id_result,
                pipe_storage,
            } => unimplemented!(),
            Instruction::GetKernelLocalSizeForSubgroupCount {
                id_result_type,
                id_result,
                subgroup_count,
                invoke,
                param,
                param_size,
                param_align,
            } => unimplemented!(),
            Instruction::GetKernelMaxNumSubgroups {
                id_result_type,
                id_result,
                invoke,
                param,
                param_size,
                param_align,
            } => unimplemented!(),
            Instruction::TypeNamedBarrier { id_result } => unimplemented!(),
            Instruction::NamedBarrierInitialize {
                id_result_type,
                id_result,
                subgroup_count,
            } => unimplemented!(),
            Instruction::MemoryNamedBarrier {
                named_barrier,
                memory,
                semantics,
            } => unimplemented!(),
            Instruction::ModuleProcessed { ref process } => unimplemented!(),
            Instruction::ExecutionModeId {
                entry_point,
                ref mode,
            } => {}
            Instruction::DecorateId {
                target,
                ref decoration,
            } => {}
            Instruction::GroupNonUniformElect {
                id_result_type,
                id_result,
                execution,
            } => unimplemented!(),
            Instruction::GroupNonUniformAll {
                id_result_type,
                id_result,
                execution,
                predicate,
            } => unimplemented!(),
            Instruction::GroupNonUniformAny {
                id_result_type,
                id_result,
                execution,
                predicate,
            } => unimplemented!(),
            Instruction::GroupNonUniformAllEqual {
                id_result_type,
                id_result,
                execution,
                value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBroadcast {
                id_result_type,
                id_result,
                execution,
                value,
                id,
            } => unimplemented!(),
            Instruction::GroupNonUniformBroadcastFirst {
                id_result_type,
                id_result,
                execution,
                value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallot {
                id_result_type,
                id_result,
                execution,
                predicate,
            } => unimplemented!(),
            Instruction::GroupNonUniformInverseBallot {
                id_result_type,
                id_result,
                execution,
                value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallotBitExtract {
                id_result_type,
                id_result,
                execution,
                value,
                index,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallotBitCount {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallotFindLSB {
                id_result_type,
                id_result,
                execution,
                value,
            } => unimplemented!(),
            Instruction::GroupNonUniformBallotFindMSB {
                id_result_type,
                id_result,
                execution,
                value,
            } => unimplemented!(),
            Instruction::GroupNonUniformShuffle {
                id_result_type,
                id_result,
                execution,
                value,
                id,
            } => unimplemented!(),
            Instruction::GroupNonUniformShuffleXor {
                id_result_type,
                id_result,
                execution,
                value,
                mask,
            } => unimplemented!(),
            Instruction::GroupNonUniformShuffleUp {
                id_result_type,
                id_result,
                execution,
                value,
                delta,
            } => unimplemented!(),
            Instruction::GroupNonUniformShuffleDown {
                id_result_type,
                id_result,
                execution,
                value,
                delta,
            } => unimplemented!(),
            Instruction::GroupNonUniformIAdd {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformFAdd {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformIMul {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformFMul {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformSMin {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformUMin {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformFMin {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformSMax {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformUMax {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformFMax {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformBitwiseAnd {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformBitwiseOr {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformBitwiseXor {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformLogicalAnd {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformLogicalOr {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformLogicalXor {
                id_result_type,
                id_result,
                execution,
                operation,
                value,
                cluster_size,
            } => unimplemented!(),
            Instruction::GroupNonUniformQuadBroadcast {
                id_result_type,
                id_result,
                execution,
                value,
                index,
            } => unimplemented!(),
            Instruction::GroupNonUniformQuadSwap {
                id_result_type,
                id_result,
                execution,
                value,
                direction,
            } => unimplemented!(),
            Instruction::ReportIntersectionNV {
                id_result_type,
                id_result,
                hit,
                hit_kind,
            } => unimplemented!(),
            Instruction::IgnoreIntersectionNV {} => unimplemented!(),
            Instruction::TerminateRayNV {} => unimplemented!(),
            Instruction::TraceNV {
                accel,
                ray_flags,
                cull_mask,
                sbt_offset,
                sbt_stride,
                miss_index,
                ray_origin,
                ray_tmin,
                ray_direction,
                ray_tmax,
                payload_id,
            } => unimplemented!(),
            Instruction::TypeAccelerationStructureNV { id_result } => unimplemented!(),
            Instruction::ExecuteCallableNV {
                sbt_index,
                callable_data_id,
            } => unimplemented!(),
            Instruction::OpenCLStdAcos {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAcosh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAcospi {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAsin {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAsinh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAsinpi {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAtan {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAtan2 {
                id_result_type,
                id_result,
                set,
                y,
                x,
            } => self.visit_simple_instruction(id_result, &[y, x]),
            Instruction::OpenCLStdAtanh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAtanpi {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdAtan2pi {
                id_result_type,
                id_result,
                set,
                y,
                x,
            } => self.visit_simple_instruction(id_result, &[y, x]),
            Instruction::OpenCLStdCbrt {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCeil {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCopysign {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdCos {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCosh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCospi {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdErfc {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdErf {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdExp {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdExp2 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdExp10 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdExpm1 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdFabs {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdFdim {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFloor {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdFma {
                id_result_type,
                id_result,
                set,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdFmax {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFmin {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFmod {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFract {
                id_result_type,
                id_result,
                set,
                x,
                ptr,
            } => unimplemented!(),
            Instruction::OpenCLStdFrexp {
                id_result_type,
                id_result,
                set,
                x,
                exp,
            } => unimplemented!(),
            Instruction::OpenCLStdHypot {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdIlogb {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLdexp {
                id_result_type,
                id_result,
                set,
                x,
                k,
            } => self.visit_simple_instruction(id_result, &[x, k]),
            Instruction::OpenCLStdLgamma {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLgammaR {
                id_result_type,
                id_result,
                set,
                x,
                signp,
            } => unimplemented!(),
            Instruction::OpenCLStdLog {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLog2 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLog10 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLog1p {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdLogb {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdMad {
                id_result_type,
                id_result,
                set,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdMaxmag {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdMinmag {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdModf {
                id_result_type,
                id_result,
                set,
                x,
                iptr,
            } => unimplemented!(),
            Instruction::OpenCLStdNan {
                id_result_type,
                id_result,
                set,
                nancode,
            } => self.visit_simple_instruction(id_result, iter::once(nancode)),
            Instruction::OpenCLStdNextafter {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdPow {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdPown {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdPowr {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdRemainder {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdRemquo {
                id_result_type,
                id_result,
                set,
                x,
                y,
                quo,
            } => unimplemented!(),
            Instruction::OpenCLStdRint {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdRootn {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdRound {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdRsqrt {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSin {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSincos {
                id_result_type,
                id_result,
                set,
                x,
                cosval,
            } => unimplemented!(),
            Instruction::OpenCLStdSinh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSinpi {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSqrt {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTan {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTanh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTanpi {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTgamma {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdTrunc {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfCos {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfDivide {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdHalfExp {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfExp2 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfExp10 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfLog {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfLog2 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfLog10 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfPowr {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdHalfRecip {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfRsqrt {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfSin {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfSqrt {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdHalfTan {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeCos {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeDivide {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdNativeExp {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeExp2 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeExp10 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeLog {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeLog2 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeLog10 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativePowr {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeRecip {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeRsqrt {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeSin {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeSqrt {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdNativeTan {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSAbs {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSAbsDiff {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSAddSat {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUAddSat {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSHadd {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUHadd {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSRhadd {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdURhadd {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSClamp {
                id_result_type,
                id_result,
                set,
                x,
                minval,
                maxval,
            } => self.visit_simple_instruction(id_result, &[x, minval, maxval]),
            Instruction::OpenCLStdUClamp {
                id_result_type,
                id_result,
                set,
                x,
                minval,
                maxval,
            } => self.visit_simple_instruction(id_result, &[x, minval, maxval]),
            Instruction::OpenCLStdClz {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCtz {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSMadHi {
                id_result_type,
                id_result,
                set,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdUMadSat {
                id_result_type,
                id_result,
                set,
                x,
                y,
                z,
            } => self.visit_simple_instruction(id_result, &[x, y, z]),
            Instruction::OpenCLStdSMadSat {
                id_result_type,
                id_result,
                set,
                x,
                y,
                z,
            } => self.visit_simple_instruction(id_result, &[x, y, z]),
            Instruction::OpenCLStdSMax {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMax {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSMin {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMin {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdSMulHi {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdRotate {
                id_result_type,
                id_result,
                set,
                v,
                i,
            } => self.visit_simple_instruction(id_result, &[v, i]),
            Instruction::OpenCLStdSSubSat {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUSubSat {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUUpsample {
                id_result_type,
                id_result,
                set,
                hi,
                lo,
            } => self.visit_simple_instruction(id_result, &[hi, lo]),
            Instruction::OpenCLStdSUpsample {
                id_result_type,
                id_result,
                set,
                hi,
                lo,
            } => self.visit_simple_instruction(id_result, &[hi, lo]),
            Instruction::OpenCLStdPopcount {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdSMad24 {
                id_result_type,
                id_result,
                set,
                x,
                y,
                z,
            } => self.visit_simple_instruction(id_result, &[x, y, z]),
            Instruction::OpenCLStdUMad24 {
                id_result_type,
                id_result,
                set,
                x,
                y,
                z,
            } => self.visit_simple_instruction(id_result, &[x, y, z]),
            Instruction::OpenCLStdSMul24 {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMul24 {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUAbs {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdUAbsDiff {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMulHi {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdUMadHi {
                id_result_type,
                id_result,
                set,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdFclamp {
                id_result_type,
                id_result,
                set,
                x,
                minval,
                maxval,
            } => self.visit_simple_instruction(id_result, &[x, minval, maxval]),
            Instruction::OpenCLStdDegrees {
                id_result_type,
                id_result,
                set,
                radians,
            } => self.visit_simple_instruction(id_result, iter::once(radians)),
            Instruction::OpenCLStdFmaxCommon {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdFminCommon {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::OpenCLStdMix {
                id_result_type,
                id_result,
                set,
                x,
                y,
                a,
            } => self.visit_simple_instruction(id_result, &[x, y, a]),
            Instruction::OpenCLStdRadians {
                id_result_type,
                id_result,
                set,
                degrees,
            } => self.visit_simple_instruction(id_result, iter::once(degrees)),
            Instruction::OpenCLStdStep {
                id_result_type,
                id_result,
                set,
                edge,
                x,
            } => self.visit_simple_instruction(id_result, &[edge, x]),
            Instruction::OpenCLStdSmoothstep {
                id_result_type,
                id_result,
                set,
                edge0,
                edge1,
                x,
            } => self.visit_simple_instruction(id_result, &[edge0, edge1, x]),
            Instruction::OpenCLStdSign {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::OpenCLStdCross {
                id_result_type,
                id_result,
                set,
                p0,
                p1,
            } => self.visit_simple_instruction(id_result, &[p0, p1]),
            Instruction::OpenCLStdDistance {
                id_result_type,
                id_result,
                set,
                p0,
                p1,
            } => self.visit_simple_instruction(id_result, &[p0, p1]),
            Instruction::OpenCLStdLength {
                id_result_type,
                id_result,
                set,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::OpenCLStdNormalize {
                id_result_type,
                id_result,
                set,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::OpenCLStdFastDistance {
                id_result_type,
                id_result,
                set,
                p0,
                p1,
            } => self.visit_simple_instruction(id_result, &[p0, p1]),
            Instruction::OpenCLStdFastLength {
                id_result_type,
                id_result,
                set,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::OpenCLStdFastNormalize {
                id_result_type,
                id_result,
                set,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::OpenCLStdBitselect {
                id_result_type,
                id_result,
                set,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdSelect {
                id_result_type,
                id_result,
                set,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::OpenCLStdVloadn {
                id_result_type,
                id_result,
                set,
                offset,
                p,
                n,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoren {
                id_result_type,
                id_result,
                set,
                data,
                offset,
                p,
            } => unimplemented!(),
            Instruction::OpenCLStdVloadHalf {
                id_result_type,
                id_result,
                set,
                offset,
                p,
            } => unimplemented!(),
            Instruction::OpenCLStdVloadHalfn {
                id_result_type,
                id_result,
                set,
                offset,
                p,
                n,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalf {
                id_result_type,
                id_result,
                set,
                data,
                offset,
                p,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalfR {
                id_result_type,
                id_result,
                set,
                data,
                offset,
                p,
                mode,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalfn {
                id_result_type,
                id_result,
                set,
                data,
                offset,
                p,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalfnR {
                id_result_type,
                id_result,
                set,
                data,
                offset,
                p,
                mode,
            } => unimplemented!(),
            Instruction::OpenCLStdVloadaHalfn {
                id_result_type,
                id_result,
                set,
                offset,
                p,
                n,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreaHalfn {
                id_result_type,
                id_result,
                set,
                data,
                offset,
                p,
            } => unimplemented!(),
            Instruction::OpenCLStdVstoreaHalfnR {
                id_result_type,
                id_result,
                set,
                data,
                offset,
                p,
                mode,
            } => unimplemented!(),
            Instruction::OpenCLStdShuffle {
                id_result_type,
                id_result,
                set,
                x,
                shuffle_mask,
            } => self.visit_simple_instruction(id_result, &[x, shuffle_mask]),
            Instruction::OpenCLStdShuffle2 {
                id_result_type,
                id_result,
                set,
                x,
                y,
                shuffle_mask,
            } => self.visit_simple_instruction(id_result, &[x, y, shuffle_mask]),
            Instruction::OpenCLStdPrintf {
                id_result_type,
                id_result,
                set,
                format,
                ref additional_arguments,
            } => unimplemented!(),
            Instruction::OpenCLStdPrefetch {
                id_result_type,
                id_result,
                set,
                ptr,
                num_elements,
            } => unimplemented!(),
            Instruction::GLSLStd450Round {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450RoundEven {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Trunc {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450FAbs {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450SAbs {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450FSign {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450SSign {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Floor {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Ceil {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Fract {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Radians {
                id_result_type,
                id_result,
                set,
                degrees,
            } => self.visit_simple_instruction(id_result, iter::once(degrees)),
            Instruction::GLSLStd450Degrees {
                id_result_type,
                id_result,
                set,
                radians,
            } => self.visit_simple_instruction(id_result, iter::once(radians)),
            Instruction::GLSLStd450Sin {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Cos {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Tan {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Asin {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Acos {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Atan {
                id_result_type,
                id_result,
                set,
                y_over_x,
            } => self.visit_simple_instruction(id_result, iter::once(y_over_x)),
            Instruction::GLSLStd450Sinh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Cosh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Tanh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Asinh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Acosh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Atanh {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Atan2 {
                id_result_type,
                id_result,
                set,
                y,
                x,
            } => self.visit_simple_instruction(id_result, &[y, x]),
            Instruction::GLSLStd450Pow {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450Exp {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Log {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Exp2 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Log2 {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Sqrt {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450InverseSqrt {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Determinant {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450MatrixInverse {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Modf {
                id_result_type,
                id_result,
                set,
                x,
                i,
            } => unimplemented!(),
            Instruction::GLSLStd450ModfStruct {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450FMin {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450UMin {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450SMin {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450FMax {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450UMax {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450SMax {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450FClamp {
                id_result_type,
                id_result,
                set,
                x,
                min_val,
                max_val,
            } => self.visit_simple_instruction(id_result, &[x, min_val, max_val]),
            Instruction::GLSLStd450UClamp {
                id_result_type,
                id_result,
                set,
                x,
                min_val,
                max_val,
            } => self.visit_simple_instruction(id_result, &[x, min_val, max_val]),
            Instruction::GLSLStd450SClamp {
                id_result_type,
                id_result,
                set,
                x,
                min_val,
                max_val,
            } => self.visit_simple_instruction(id_result, &[x, min_val, max_val]),
            Instruction::GLSLStd450FMix {
                id_result_type,
                id_result,
                set,
                x,
                y,
                a,
            } => self.visit_simple_instruction(id_result, &[x, y, a]),
            Instruction::GLSLStd450IMix { .. } => {
                unreachable!("imix was removed from spec before release");
            }
            Instruction::GLSLStd450Step {
                id_result_type,
                id_result,
                set,
                edge,
                x,
            } => self.visit_simple_instruction(id_result, &[edge, x]),
            Instruction::GLSLStd450SmoothStep {
                id_result_type,
                id_result,
                set,
                edge0,
                edge1,
                x,
            } => self.visit_simple_instruction(id_result, &[edge0, edge1, x]),
            Instruction::GLSLStd450Fma {
                id_result_type,
                id_result,
                set,
                a,
                b,
                c,
            } => self.visit_simple_instruction(id_result, &[a, b, c]),
            Instruction::GLSLStd450Frexp {
                id_result_type,
                id_result,
                set,
                x,
                exp,
            } => unimplemented!(),
            Instruction::GLSLStd450FrexpStruct {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Ldexp {
                id_result_type,
                id_result,
                set,
                x,
                exp,
            } => self.visit_simple_instruction(id_result, &[x, exp]),
            Instruction::GLSLStd450PackSnorm4x8 {
                id_result_type,
                id_result,
                set,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackUnorm4x8 {
                id_result_type,
                id_result,
                set,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackSnorm2x16 {
                id_result_type,
                id_result,
                set,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackUnorm2x16 {
                id_result_type,
                id_result,
                set,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackHalf2x16 {
                id_result_type,
                id_result,
                set,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450PackDouble2x32 {
                id_result_type,
                id_result,
                set,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450UnpackSnorm2x16 {
                id_result_type,
                id_result,
                set,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::GLSLStd450UnpackUnorm2x16 {
                id_result_type,
                id_result,
                set,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::GLSLStd450UnpackHalf2x16 {
                id_result_type,
                id_result,
                set,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450UnpackSnorm4x8 {
                id_result_type,
                id_result,
                set,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::GLSLStd450UnpackUnorm4x8 {
                id_result_type,
                id_result,
                set,
                p,
            } => self.visit_simple_instruction(id_result, iter::once(p)),
            Instruction::GLSLStd450UnpackDouble2x32 {
                id_result_type,
                id_result,
                set,
                v,
            } => self.visit_simple_instruction(id_result, iter::once(v)),
            Instruction::GLSLStd450Length {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450Distance {
                id_result_type,
                id_result,
                set,
                p0,
                p1,
            } => self.visit_simple_instruction(id_result, &[p0, p1]),
            Instruction::GLSLStd450Cross {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450Normalize {
                id_result_type,
                id_result,
                set,
                x,
            } => self.visit_simple_instruction(id_result, iter::once(x)),
            Instruction::GLSLStd450FaceForward {
                id_result_type,
                id_result,
                set,
                n,
                i,
                nref,
            } => self.visit_simple_instruction(id_result, &[n, i, nref]),
            Instruction::GLSLStd450Reflect {
                id_result_type,
                id_result,
                set,
                i,
                n,
            } => self.visit_simple_instruction(id_result, &[i, n]),
            Instruction::GLSLStd450Refract {
                id_result_type,
                id_result,
                set,
                i,
                n,
                eta,
            } => self.visit_simple_instruction(id_result, &[i, n, eta]),
            Instruction::GLSLStd450FindILsb {
                id_result_type,
                id_result,
                set,
                value,
            } => self.visit_simple_instruction(id_result, iter::once(value)),
            Instruction::GLSLStd450FindSMsb {
                id_result_type,
                id_result,
                set,
                value,
            } => self.visit_simple_instruction(id_result, iter::once(value)),
            Instruction::GLSLStd450FindUMsb {
                id_result_type,
                id_result,
                set,
                value,
            } => self.visit_simple_instruction(id_result, iter::once(value)),
            Instruction::GLSLStd450InterpolateAtCentroid {
                id_result_type,
                id_result,
                set,
                interpolant,
            } => unimplemented!(),
            Instruction::GLSLStd450InterpolateAtSample {
                id_result_type,
                id_result,
                set,
                interpolant,
                sample,
            } => unimplemented!(),
            Instruction::GLSLStd450InterpolateAtOffset {
                id_result_type,
                id_result,
                set,
                interpolant,
                offset,
            } => unimplemented!(),
            Instruction::GLSLStd450NMin {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450NMax {
                id_result_type,
                id_result,
                set,
                x,
                y,
            } => self.visit_simple_instruction(id_result, &[x, y]),
            Instruction::GLSLStd450NClamp {
                id_result_type,
                id_result,
                set,
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
            unimplemented!("self.cfg.structure_tree().basic_blocks_in_order().collect()");
        loop {
            self.any_changes = false;
            for &basic_block in basic_blocks.iter() {
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
