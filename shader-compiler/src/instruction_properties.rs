// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use spirv_parser::{IdRef, IdResult, IdResultType, Instruction};

#[derive(Copy, Clone, Debug)]
enum TerminatingInstructionTargetsState<'a> {
    None,
    ReturnOrKill,
    Single(IdRef),
    Two(IdRef, IdRef),
    Switch32WithoutDefault(&'a [(u32, IdRef)]),
    Switch32(IdRef, &'a [(u32, IdRef)]),
    Switch64WithoutDefault(&'a [(u64, IdRef)]),
    Switch64(IdRef, &'a [(u64, IdRef)]),
}

#[derive(Clone, Debug)]
pub struct TerminatingInstructionTargets<'a>(TerminatingInstructionTargetsState<'a>);

impl<'a> Iterator for TerminatingInstructionTargets<'a> {
    type Item = IdRef;
    fn next(&mut self) -> Option<IdRef> {
        match self.0 {
            TerminatingInstructionTargetsState::None => None,
            TerminatingInstructionTargetsState::ReturnOrKill => None,
            TerminatingInstructionTargetsState::Single(v) => {
                self.0 = TerminatingInstructionTargetsState::None;
                Some(v)
            }
            TerminatingInstructionTargetsState::Two(v1, v2) => {
                self.0 = TerminatingInstructionTargetsState::Single(v2);
                Some(v1)
            }
            TerminatingInstructionTargetsState::Switch32WithoutDefault(v) => {
                let (first, rest) = v.split_first()?;
                self.0 = TerminatingInstructionTargetsState::Switch32WithoutDefault(rest);
                Some(first.1)
            }
            TerminatingInstructionTargetsState::Switch32(v1, v2) => {
                self.0 = TerminatingInstructionTargetsState::Switch32WithoutDefault(v2);
                Some(v1)
            }
            TerminatingInstructionTargetsState::Switch64WithoutDefault(v) => {
                let (first, rest) = v.split_first()?;
                self.0 = TerminatingInstructionTargetsState::Switch64WithoutDefault(rest);
                Some(first.1)
            }
            TerminatingInstructionTargetsState::Switch64(v1, v2) => {
                self.0 = TerminatingInstructionTargetsState::Switch64WithoutDefault(v2);
                Some(v1)
            }
        }
    }
}

impl<'a> TerminatingInstructionTargets<'a> {
    fn new(instruction: &'a Instruction) -> Option<Self> {
        match *instruction {
            Instruction::Branch { target_label, .. } => Some(TerminatingInstructionTargets(
                TerminatingInstructionTargetsState::Single(target_label),
            )),
            Instruction::BranchConditional {
                true_label,
                false_label,
                ..
            } => Some(TerminatingInstructionTargets(
                TerminatingInstructionTargetsState::Two(true_label, false_label),
            )),
            Instruction::Switch32 {
                default,
                ref target,
                ..
            } => Some(TerminatingInstructionTargets(
                TerminatingInstructionTargetsState::Switch32(default, target),
            )),
            Instruction::Switch64 {
                default,
                ref target,
                ..
            } => Some(TerminatingInstructionTargets(
                TerminatingInstructionTargetsState::Switch64(default, target),
            )),
            Instruction::Kill => Some(TerminatingInstructionTargets(
                TerminatingInstructionTargetsState::ReturnOrKill,
            )),
            Instruction::Return => Some(TerminatingInstructionTargets(
                TerminatingInstructionTargetsState::ReturnOrKill,
            )),
            Instruction::ReturnValue { .. } => Some(TerminatingInstructionTargets(
                TerminatingInstructionTargetsState::ReturnOrKill,
            )),
            Instruction::Unreachable => Some(TerminatingInstructionTargets(
                TerminatingInstructionTargetsState::None,
            )),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InstructionProperties<'a> {
    instruction: &'a Instruction,
}

impl<'a> InstructionProperties<'a> {
    pub fn new(instruction: &'a Instruction) -> Self {
        InstructionProperties { instruction }
    }
    pub fn targets(self) -> Option<TerminatingInstructionTargets<'a>> {
        TerminatingInstructionTargets::new(self.instruction)
    }
    pub fn is_block_terminator(self) -> bool {
        self.targets().is_some()
    }
    pub fn is_debug_line(self) -> bool {
        match self.instruction {
            Instruction::Line { .. } | Instruction::NoLine => true,
            _ => false,
        }
    }
    pub fn instruction(self) -> &'a Instruction {
        self.instruction
    }
    pub fn result_and_type(self) -> Option<(IdResult, Option<IdResultType>)> {
        match *self.instruction {
            Instruction::Undef {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ExtInst {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConstantTrue {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConstantFalse {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Constant32 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Constant64 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConstantComposite {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConstantSampler {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConstantNull {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SpecConstantTrue {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SpecConstantFalse {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SpecConstant32 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SpecConstant64 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SpecConstantComposite {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Function {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FunctionParameter {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FunctionCall {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Variable {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageTexelPointer {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Load {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AccessChain {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::InBoundsAccessChain {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::PtrAccessChain {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ArrayLength {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GenericPtrMemSemantics {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::InBoundsPtrAccessChain {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::VectorExtractDynamic {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::VectorInsertDynamic {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::VectorShuffle {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::CompositeConstruct {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::CompositeExtract {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::CompositeInsert {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::CopyObject {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Transpose {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SampledImage {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSampleImplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSampleExplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSampleDrefImplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSampleDrefExplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSampleProjImplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSampleProjExplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSampleProjDrefImplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSampleProjDrefExplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageFetch {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageGather {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageDrefGather {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageRead {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Image {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageQueryFormat {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageQueryOrder {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageQuerySizeLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageQuerySize {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageQueryLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageQueryLevels {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageQuerySamples {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConvertFToU {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConvertFToS {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConvertSToF {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConvertUToF {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::UConvert {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SConvert {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FConvert {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::QuantizeToF16 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConvertPtrToU {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SatConvertSToU {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SatConvertUToS {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConvertUToPtr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::PtrCastToGeneric {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GenericCastToPtr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GenericCastToPtrExplicit {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Bitcast {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SNegate {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FNegate {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IAdd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FAdd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ISub {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FSub {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IMul {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FMul {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::UDiv {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SDiv {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FDiv {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::UMod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SRem {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SMod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FRem {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FMod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::VectorTimesScalar {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::MatrixTimesScalar {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::VectorTimesMatrix {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::MatrixTimesVector {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::MatrixTimesMatrix {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OuterProduct {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Dot {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IAddCarry {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ISubBorrow {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::UMulExtended {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SMulExtended {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Any {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::All {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IsNan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IsInf {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IsFinite {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IsNormal {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SignBitSet {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::LessOrGreater {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Ordered {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Unordered {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::LogicalEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::LogicalNotEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::LogicalOr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::LogicalAnd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::LogicalNot {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Select {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::INotEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::UGreaterThan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SGreaterThan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::UGreaterThanEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SGreaterThanEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ULessThan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SLessThan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ULessThanEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SLessThanEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FOrdEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FUnordEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FOrdNotEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FUnordNotEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FOrdLessThan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FUnordLessThan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FOrdGreaterThan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FUnordGreaterThan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FOrdLessThanEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FUnordLessThanEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FOrdGreaterThanEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FUnordGreaterThanEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ShiftRightLogical {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ShiftRightArithmetic {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ShiftLeftLogical {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::BitwiseOr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::BitwiseXor {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::BitwiseAnd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Not {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::BitFieldInsert {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::BitFieldSExtract {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::BitFieldUExtract {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::BitReverse {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::BitCount {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::DPdx {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::DPdy {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Fwidth {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::DPdxFine {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::DPdyFine {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FwidthFine {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::DPdxCoarse {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::DPdyCoarse {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::FwidthCoarse {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicLoad {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicExchange {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicCompareExchange {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicCompareExchangeWeak {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicIIncrement {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicIDecrement {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicIAdd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicISub {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicSMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicUMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicSMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicUMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicAnd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicOr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicXor {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::Phi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupAsyncCopy {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupAll {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupAny {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupBroadcast {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupIAdd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupFAdd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupFMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupUMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupSMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupFMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupUMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupSMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ReadPipe {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::WritePipe {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ReservedReadPipe {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ReservedWritePipe {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ReserveReadPipePackets {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ReserveWritePipePackets {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IsValidReserveId {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GetNumPipePackets {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GetMaxPipePackets {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupReserveReadPipePackets {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupReserveWritePipePackets {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::EnqueueMarker {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::EnqueueKernel {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GetKernelNDrangeSubGroupCount {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GetKernelNDrangeMaxSubGroupSize {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GetKernelWorkGroupSize {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GetKernelPreferredWorkGroupSizeMultiple {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::CreateUserEvent {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::IsValidEvent {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GetDefaultQueue {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::BuildNDRange {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSparseSampleImplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSparseSampleExplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSparseSampleDrefImplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSparseSampleDrefExplicitLod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSparseFetch {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSparseGather {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSparseDrefGather {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSparseTexelsResident {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::AtomicFlagTestAndSet {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ImageSparseRead {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::SizeOf {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ConstantPipeStorage {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::CreatePipeFromPipeStorage {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GetKernelLocalSizeForSubgroupCount {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GetKernelMaxNumSubgroups {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::NamedBarrierInitialize {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformElect {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformAll {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformAny {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformAllEqual {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBroadcast {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBroadcastFirst {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBallot {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformInverseBallot {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBallotBitExtract {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBallotBitCount {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBallotFindLSB {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBallotFindMSB {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformShuffle {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformShuffleXor {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformShuffleUp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformShuffleDown {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformIAdd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformFAdd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformIMul {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformFMul {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformSMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformUMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformFMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformSMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformUMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformFMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBitwiseAnd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBitwiseOr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformBitwiseXor {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformLogicalAnd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformLogicalOr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformLogicalXor {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformQuadBroadcast {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GroupNonUniformQuadSwap {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::ReportIntersectionNV {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAcos {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAcosh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAcospi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAsin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAsinh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAsinpi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAtan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAtan2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAtanh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAtanpi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdAtan2pi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdCbrt {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdCeil {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdCopysign {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdCos {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdCosh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdCospi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdErfc {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdErf {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdExp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdExp2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdExp10 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdExpm1 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFabs {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFdim {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFloor {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFma {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFmax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFmin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFmod {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFract {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFrexp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHypot {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdIlogb {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdLdexp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdLgamma {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdLgammaR {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdLog {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdLog2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdLog10 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdLog1p {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdLogb {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdMad {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdMaxmag {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdMinmag {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdModf {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNextafter {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdPow {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdPown {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdPowr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdRemainder {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdRemquo {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdRint {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdRootn {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdRound {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdRsqrt {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSincos {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSinh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSinpi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSqrt {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdTan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdTanh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdTanpi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdTgamma {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdTrunc {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfCos {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfDivide {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfExp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfExp2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfExp10 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfLog {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfLog2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfLog10 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfPowr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfRecip {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfRsqrt {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfSin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfSqrt {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdHalfTan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeCos {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeDivide {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeExp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeExp2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeExp10 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeLog {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeLog2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeLog10 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativePowr {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeRecip {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeRsqrt {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeSin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeSqrt {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNativeTan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSAbs {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSAbsDiff {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSAddSat {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUAddSat {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSHadd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUHadd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSRhadd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdURhadd {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSClamp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUClamp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdClz {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdCtz {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSMadHi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUMadSat {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSMadSat {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSMulHi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdRotate {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSSubSat {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUSubSat {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUUpsample {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSUpsample {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdPopcount {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSMad24 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUMad24 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSMul24 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUMul24 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUAbs {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUAbsDiff {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUMulHi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdUMadHi {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFclamp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdDegrees {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFmaxCommon {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFminCommon {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdMix {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdRadians {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdStep {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSmoothstep {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSign {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdCross {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdDistance {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdLength {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdNormalize {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFastDistance {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFastLength {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdFastNormalize {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdBitselect {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdSelect {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVloadn {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVstoren {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVloadHalf {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVloadHalfn {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVstoreHalf {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVstoreHalfR {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVstoreHalfn {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVstoreHalfnR {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVloadaHalfn {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVstoreaHalfn {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdVstoreaHalfnR {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdShuffle {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdShuffle2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdPrintf {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::OpenCLStdPrefetch {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Round {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450RoundEven {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Trunc {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FAbs {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450SAbs {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FSign {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450SSign {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Floor {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Ceil {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Fract {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Radians {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Degrees {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Sin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Cos {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Tan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Asin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Acos {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Atan {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Sinh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Cosh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Tanh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Asinh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Acosh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Atanh {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Atan2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Pow {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Exp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Log {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Exp2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Log2 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Sqrt {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450InverseSqrt {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Determinant {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450MatrixInverse {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Modf {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450ModfStruct {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450UMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450SMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450UMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450SMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FClamp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450UClamp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450SClamp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FMix {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450IMix {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Step {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450SmoothStep {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Fma {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Frexp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FrexpStruct {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Ldexp {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450PackSnorm4x8 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450PackUnorm4x8 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450PackSnorm2x16 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450PackUnorm2x16 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450PackHalf2x16 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450PackDouble2x32 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450UnpackSnorm2x16 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450UnpackUnorm2x16 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450UnpackHalf2x16 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450UnpackSnorm4x8 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450UnpackUnorm4x8 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450UnpackDouble2x32 {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Length {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Distance {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Cross {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Normalize {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FaceForward {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Reflect {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450Refract {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FindILsb {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FindSMsb {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450FindUMsb {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450InterpolateAtCentroid {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450InterpolateAtSample {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450InterpolateAtOffset {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450NMin {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450NMax {
                id_result_type,
                id_result,
                ..
            }
            | Instruction::GLSLStd450NClamp {
                id_result_type,
                id_result,
                ..
            } => Some((id_result, Some(id_result_type))),
            Instruction::String { id_result, .. }
            | Instruction::ExtInstImport { id_result, .. }
            | Instruction::TypeVoid { id_result, .. }
            | Instruction::TypeBool { id_result, .. }
            | Instruction::TypeInt { id_result, .. }
            | Instruction::TypeFloat { id_result, .. }
            | Instruction::TypeVector { id_result, .. }
            | Instruction::TypeMatrix { id_result, .. }
            | Instruction::TypeImage { id_result, .. }
            | Instruction::TypeSampler { id_result, .. }
            | Instruction::TypeSampledImage { id_result, .. }
            | Instruction::TypeArray { id_result, .. }
            | Instruction::TypeRuntimeArray { id_result, .. }
            | Instruction::TypeStruct { id_result, .. }
            | Instruction::TypeOpaque { id_result, .. }
            | Instruction::TypePointer { id_result, .. }
            | Instruction::TypeFunction { id_result, .. }
            | Instruction::TypeEvent { id_result, .. }
            | Instruction::TypeDeviceEvent { id_result, .. }
            | Instruction::TypeReserveId { id_result, .. }
            | Instruction::TypeQueue { id_result, .. }
            | Instruction::TypePipe { id_result, .. }
            | Instruction::DecorationGroup { id_result, .. }
            | Instruction::Label { id_result, .. }
            | Instruction::TypePipeStorage { id_result, .. }
            | Instruction::TypeNamedBarrier { id_result, .. }
            | Instruction::TypeAccelerationStructureNV { id_result, .. } => Some((id_result, None)),
            Instruction::Nop
            | Instruction::SourceContinued { .. }
            | Instruction::Source { .. }
            | Instruction::SourceExtension { .. }
            | Instruction::Name { .. }
            | Instruction::MemberName { .. }
            | Instruction::Line { .. }
            | Instruction::Extension { .. }
            | Instruction::MemoryModel { .. }
            | Instruction::EntryPoint { .. }
            | Instruction::ExecutionMode { .. }
            | Instruction::Capability { .. }
            | Instruction::TypeForwardPointer { .. }
            | Instruction::SpecConstantOp { .. }
            | Instruction::FunctionEnd
            | Instruction::Store { .. }
            | Instruction::CopyMemory { .. }
            | Instruction::CopyMemorySized { .. }
            | Instruction::Decorate { .. }
            | Instruction::MemberDecorate { .. }
            | Instruction::GroupDecorate { .. }
            | Instruction::GroupMemberDecorate { .. }
            | Instruction::ImageWrite { .. }
            | Instruction::EmitVertex
            | Instruction::EndPrimitive
            | Instruction::EmitStreamVertex { .. }
            | Instruction::EndStreamPrimitive { .. }
            | Instruction::ControlBarrier { .. }
            | Instruction::MemoryBarrier { .. }
            | Instruction::AtomicStore { .. }
            | Instruction::LoopMerge { .. }
            | Instruction::SelectionMerge { .. }
            | Instruction::Branch { .. }
            | Instruction::BranchConditional { .. }
            | Instruction::Switch32 { .. }
            | Instruction::Switch64 { .. }
            | Instruction::Kill
            | Instruction::Return
            | Instruction::ReturnValue { .. }
            | Instruction::Unreachable
            | Instruction::LifetimeStart { .. }
            | Instruction::LifetimeStop { .. }
            | Instruction::GroupWaitEvents { .. }
            | Instruction::CommitReadPipe { .. }
            | Instruction::CommitWritePipe { .. }
            | Instruction::GroupCommitReadPipe { .. }
            | Instruction::GroupCommitWritePipe { .. }
            | Instruction::RetainEvent { .. }
            | Instruction::ReleaseEvent { .. }
            | Instruction::SetUserEventStatus { .. }
            | Instruction::CaptureEventProfilingInfo { .. }
            | Instruction::NoLine
            | Instruction::AtomicFlagClear { .. }
            | Instruction::MemoryNamedBarrier { .. }
            | Instruction::ModuleProcessed { .. }
            | Instruction::ExecutionModeId { .. }
            | Instruction::DecorateId { .. }
            | Instruction::IgnoreIntersectionNV
            | Instruction::TerminateRayNV
            | Instruction::TraceNV { .. }
            | Instruction::ExecuteCallableNV { .. } => None,
        }
    }
    pub fn result(self) -> Option<IdResult> {
        Some(self.result_and_type()?.0)
    }
    pub fn result_type(self) -> Option<IdResultType> {
        self.result_and_type()?.1
    }
}
