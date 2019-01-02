// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2019 Jacob Lifshay

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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum InstructionClass {
    /// misc module-level instructions
    ModuleLevel,
    /// OpLine and OpNoLine
    DebugLine,
    /// OpLabel
    Label,
    /// Debug Info
    Debug,
    /// type definition
    Type,
    /// constant definition
    Const,
    /// specialization constant
    SpecConst,
    /// OpUndef
    Undef,
    /// decoration
    Decoration,
    /// OpPhi
    Phi,
    /// simple operations
    /// includes only side-effect free operations
    /// basically all instructions that only calculate a value
    Simple,
    /// OpSelectionMerge and OpLoopMerge
    StructuredControlFlow,
    /// OpExtInst and OpExtInstImport
    ExtInst,
    /// function declaration and definition instructions
    Function,
    /// operations that calculate screen-space derivatives
    Derivative,
    /// glsl interpolateAt* functions
    InterpolateAt,
    /// basic block terminating instruction
    BlockTerminator,
    /// OpNop
    Nop,
    /// OpVariable
    Variable,
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
    pub fn class(self) -> InstructionClass {
        use self::InstructionClass::*;
        match self.instruction {
            Instruction::Nop => Nop,
            Instruction::Undef { .. } => Undef,
            Instruction::SourceContinued { .. }
            | Instruction::Source { .. }
            | Instruction::SourceExtension { .. }
            | Instruction::Name { .. }
            | Instruction::MemberName { .. }
            | Instruction::String { .. } => Debug,
            Instruction::Line { .. } => DebugLine,
            Instruction::Extension { .. } => ModuleLevel,
            Instruction::ExtInstImport { .. } | Instruction::ExtInst { .. } => ExtInst,
            Instruction::MemoryModel { .. }
            | Instruction::EntryPoint { .. }
            | Instruction::ExecutionMode { .. }
            | Instruction::Capability { .. } => ModuleLevel,
            Instruction::TypeVoid { .. }
            | Instruction::TypeBool { .. }
            | Instruction::TypeInt { .. }
            | Instruction::TypeFloat { .. }
            | Instruction::TypeVector { .. }
            | Instruction::TypeMatrix { .. }
            | Instruction::TypeImage { .. }
            | Instruction::TypeSampler { .. }
            | Instruction::TypeSampledImage { .. }
            | Instruction::TypeArray { .. }
            | Instruction::TypeRuntimeArray { .. }
            | Instruction::TypeStruct { .. }
            | Instruction::TypeOpaque { .. }
            | Instruction::TypePointer { .. }
            | Instruction::TypeFunction { .. }
            | Instruction::TypeEvent { .. }
            | Instruction::TypeDeviceEvent { .. }
            | Instruction::TypeReserveId { .. }
            | Instruction::TypeQueue { .. }
            | Instruction::TypePipe { .. }
            | Instruction::TypeForwardPointer { .. } => Type,
            Instruction::ConstantTrue { .. }
            | Instruction::ConstantFalse { .. }
            | Instruction::Constant32 { .. }
            | Instruction::Constant64 { .. }
            | Instruction::ConstantComposite { .. }
            | Instruction::ConstantSampler { .. }
            | Instruction::ConstantNull { .. } => Const,
            Instruction::SpecConstantTrue { .. }
            | Instruction::SpecConstantFalse { .. }
            | Instruction::SpecConstant32 { .. }
            | Instruction::SpecConstant64 { .. }
            | Instruction::SpecConstantComposite { .. }
            | Instruction::SpecConstantOp { .. } => SpecConst,
            Instruction::Function { .. }
            | Instruction::FunctionParameter { .. }
            | Instruction::FunctionEnd => Function,
            Instruction::FunctionCall { .. } => unimplemented!(),
            Instruction::Variable { .. } => Variable,
            Instruction::ImageTexelPointer { .. } => Simple,
            Instruction::Load { .. } => unimplemented!(),
            Instruction::Store { .. } => unimplemented!(),
            Instruction::CopyMemory { .. } => unimplemented!(),
            Instruction::CopyMemorySized { .. } => unimplemented!(),
            Instruction::AccessChain { .. }
            | Instruction::InBoundsAccessChain { .. }
            | Instruction::PtrAccessChain { .. }
            | Instruction::ArrayLength { .. }
            | Instruction::GenericPtrMemSemantics { .. }
            | Instruction::InBoundsPtrAccessChain { .. } => Simple,
            Instruction::Decorate { .. }
            | Instruction::MemberDecorate { .. }
            | Instruction::DecorationGroup { .. }
            | Instruction::GroupDecorate { .. }
            | Instruction::GroupMemberDecorate { .. } => Decoration,
            Instruction::VectorExtractDynamic { .. }
            | Instruction::VectorInsertDynamic { .. }
            | Instruction::VectorShuffle { .. }
            | Instruction::CompositeConstruct { .. }
            | Instruction::CompositeExtract { .. }
            | Instruction::CompositeInsert { .. }
            | Instruction::CopyObject { .. }
            | Instruction::Transpose { .. }
            | Instruction::SampledImage { .. } => Simple,
            Instruction::ImageSampleImplicitLod { .. } => unimplemented!(),
            Instruction::ImageSampleExplicitLod { .. } => unimplemented!(),
            Instruction::ImageSampleDrefImplicitLod { .. } => unimplemented!(),
            Instruction::ImageSampleDrefExplicitLod { .. } => unimplemented!(),
            Instruction::ImageSampleProjImplicitLod { .. } => unimplemented!(),
            Instruction::ImageSampleProjExplicitLod { .. } => unimplemented!(),
            Instruction::ImageSampleProjDrefImplicitLod { .. } => unimplemented!(),
            Instruction::ImageSampleProjDrefExplicitLod { .. } => unimplemented!(),
            Instruction::ImageFetch { .. } => unimplemented!(),
            Instruction::ImageGather { .. } => unimplemented!(),
            Instruction::ImageDrefGather { .. } => unimplemented!(),
            Instruction::ImageRead { .. } => unimplemented!(),
            Instruction::ImageWrite { .. } => unimplemented!(),
            Instruction::Image { .. } => Simple,
            Instruction::ImageQueryFormat { .. } => unimplemented!(),
            Instruction::ImageQueryOrder { .. } => unimplemented!(),
            Instruction::ImageQuerySizeLod { .. } => unimplemented!(),
            Instruction::ImageQuerySize { .. } => unimplemented!(),
            Instruction::ImageQueryLod { .. } => unimplemented!(),
            Instruction::ImageQueryLevels { .. } => unimplemented!(),
            Instruction::ImageQuerySamples { .. } => unimplemented!(),
            Instruction::ConvertFToU { .. }
            | Instruction::ConvertFToS { .. }
            | Instruction::ConvertSToF { .. }
            | Instruction::ConvertUToF { .. }
            | Instruction::UConvert { .. }
            | Instruction::SConvert { .. }
            | Instruction::FConvert { .. }
            | Instruction::QuantizeToF16 { .. }
            | Instruction::ConvertPtrToU { .. }
            | Instruction::SatConvertSToU { .. }
            | Instruction::SatConvertUToS { .. }
            | Instruction::ConvertUToPtr { .. }
            | Instruction::PtrCastToGeneric { .. }
            | Instruction::GenericCastToPtr { .. }
            | Instruction::GenericCastToPtrExplicit { .. }
            | Instruction::Bitcast { .. }
            | Instruction::SNegate { .. }
            | Instruction::FNegate { .. }
            | Instruction::IAdd { .. }
            | Instruction::FAdd { .. }
            | Instruction::ISub { .. }
            | Instruction::FSub { .. }
            | Instruction::IMul { .. }
            | Instruction::FMul { .. }
            | Instruction::UDiv { .. }
            | Instruction::SDiv { .. }
            | Instruction::FDiv { .. }
            | Instruction::UMod { .. }
            | Instruction::SRem { .. }
            | Instruction::SMod { .. }
            | Instruction::FRem { .. }
            | Instruction::FMod { .. }
            | Instruction::VectorTimesScalar { .. }
            | Instruction::MatrixTimesScalar { .. }
            | Instruction::VectorTimesMatrix { .. }
            | Instruction::MatrixTimesVector { .. }
            | Instruction::MatrixTimesMatrix { .. }
            | Instruction::OuterProduct { .. }
            | Instruction::Dot { .. }
            | Instruction::IAddCarry { .. }
            | Instruction::ISubBorrow { .. }
            | Instruction::UMulExtended { .. }
            | Instruction::SMulExtended { .. }
            | Instruction::Any { .. }
            | Instruction::All { .. }
            | Instruction::IsNan { .. }
            | Instruction::IsInf { .. }
            | Instruction::IsFinite { .. }
            | Instruction::IsNormal { .. }
            | Instruction::SignBitSet { .. }
            | Instruction::LessOrGreater { .. }
            | Instruction::Ordered { .. }
            | Instruction::Unordered { .. }
            | Instruction::LogicalEqual { .. }
            | Instruction::LogicalNotEqual { .. }
            | Instruction::LogicalOr { .. }
            | Instruction::LogicalAnd { .. }
            | Instruction::LogicalNot { .. }
            | Instruction::Select { .. }
            | Instruction::IEqual { .. }
            | Instruction::INotEqual { .. }
            | Instruction::UGreaterThan { .. }
            | Instruction::SGreaterThan { .. }
            | Instruction::UGreaterThanEqual { .. }
            | Instruction::SGreaterThanEqual { .. }
            | Instruction::ULessThan { .. }
            | Instruction::SLessThan { .. }
            | Instruction::ULessThanEqual { .. }
            | Instruction::SLessThanEqual { .. }
            | Instruction::FOrdEqual { .. }
            | Instruction::FUnordEqual { .. }
            | Instruction::FOrdNotEqual { .. }
            | Instruction::FUnordNotEqual { .. }
            | Instruction::FOrdLessThan { .. }
            | Instruction::FUnordLessThan { .. }
            | Instruction::FOrdGreaterThan { .. }
            | Instruction::FUnordGreaterThan { .. }
            | Instruction::FOrdLessThanEqual { .. }
            | Instruction::FUnordLessThanEqual { .. }
            | Instruction::FOrdGreaterThanEqual { .. }
            | Instruction::FUnordGreaterThanEqual { .. }
            | Instruction::ShiftRightLogical { .. }
            | Instruction::ShiftRightArithmetic { .. }
            | Instruction::ShiftLeftLogical { .. }
            | Instruction::BitwiseOr { .. }
            | Instruction::BitwiseXor { .. }
            | Instruction::BitwiseAnd { .. }
            | Instruction::Not { .. }
            | Instruction::BitFieldInsert { .. }
            | Instruction::BitFieldSExtract { .. }
            | Instruction::BitFieldUExtract { .. }
            | Instruction::BitReverse { .. }
            | Instruction::BitCount { .. } => Simple,
            Instruction::DPdx { .. }
            | Instruction::DPdy { .. }
            | Instruction::Fwidth { .. }
            | Instruction::DPdxFine { .. }
            | Instruction::DPdyFine { .. }
            | Instruction::FwidthFine { .. }
            | Instruction::DPdxCoarse { .. }
            | Instruction::DPdyCoarse { .. }
            | Instruction::FwidthCoarse { .. } => Derivative,
            Instruction::EmitVertex => unimplemented!(),
            Instruction::EndPrimitive => unimplemented!(),
            Instruction::EmitStreamVertex { .. } => unimplemented!(),
            Instruction::EndStreamPrimitive { .. } => unimplemented!(),
            Instruction::ControlBarrier { .. } => unimplemented!(),
            Instruction::MemoryBarrier { .. } => unimplemented!(),
            Instruction::AtomicLoad { .. } => unimplemented!(),
            Instruction::AtomicStore { .. } => unimplemented!(),
            Instruction::AtomicExchange { .. } => unimplemented!(),
            Instruction::AtomicCompareExchange { .. } => unimplemented!(),
            Instruction::AtomicCompareExchangeWeak { .. } => unimplemented!(),
            Instruction::AtomicIIncrement { .. } => unimplemented!(),
            Instruction::AtomicIDecrement { .. } => unimplemented!(),
            Instruction::AtomicIAdd { .. } => unimplemented!(),
            Instruction::AtomicISub { .. } => unimplemented!(),
            Instruction::AtomicSMin { .. } => unimplemented!(),
            Instruction::AtomicUMin { .. } => unimplemented!(),
            Instruction::AtomicSMax { .. } => unimplemented!(),
            Instruction::AtomicUMax { .. } => unimplemented!(),
            Instruction::AtomicAnd { .. } => unimplemented!(),
            Instruction::AtomicOr { .. } => unimplemented!(),
            Instruction::AtomicXor { .. } => unimplemented!(),
            Instruction::Phi { .. } => Phi,
            Instruction::LoopMerge { .. } | Instruction::SelectionMerge { .. } => {
                StructuredControlFlow
            }
            Instruction::Label { .. } => Label,
            Instruction::Branch { .. }
            | Instruction::BranchConditional { .. }
            | Instruction::Switch32 { .. }
            | Instruction::Switch64 { .. }
            | Instruction::Kill
            | Instruction::Return
            | Instruction::ReturnValue { .. }
            | Instruction::Unreachable => BlockTerminator,
            Instruction::LifetimeStart { .. } => unimplemented!(),
            Instruction::LifetimeStop { .. } => unimplemented!(),
            Instruction::GroupAsyncCopy { .. } => unimplemented!(),
            Instruction::GroupWaitEvents { .. } => unimplemented!(),
            Instruction::GroupAll { .. } => unimplemented!(),
            Instruction::GroupAny { .. } => unimplemented!(),
            Instruction::GroupBroadcast { .. } => unimplemented!(),
            Instruction::GroupIAdd { .. } => unimplemented!(),
            Instruction::GroupFAdd { .. } => unimplemented!(),
            Instruction::GroupFMin { .. } => unimplemented!(),
            Instruction::GroupUMin { .. } => unimplemented!(),
            Instruction::GroupSMin { .. } => unimplemented!(),
            Instruction::GroupFMax { .. } => unimplemented!(),
            Instruction::GroupUMax { .. } => unimplemented!(),
            Instruction::GroupSMax { .. } => unimplemented!(),
            Instruction::ReadPipe { .. } => unimplemented!(),
            Instruction::WritePipe { .. } => unimplemented!(),
            Instruction::ReservedReadPipe { .. } => unimplemented!(),
            Instruction::ReservedWritePipe { .. } => unimplemented!(),
            Instruction::ReserveReadPipePackets { .. } => unimplemented!(),
            Instruction::ReserveWritePipePackets { .. } => unimplemented!(),
            Instruction::CommitReadPipe { .. } => unimplemented!(),
            Instruction::CommitWritePipe { .. } => unimplemented!(),
            Instruction::IsValidReserveId { .. } => unimplemented!(),
            Instruction::GetNumPipePackets { .. } => unimplemented!(),
            Instruction::GetMaxPipePackets { .. } => unimplemented!(),
            Instruction::GroupReserveReadPipePackets { .. } => unimplemented!(),
            Instruction::GroupReserveWritePipePackets { .. } => unimplemented!(),
            Instruction::GroupCommitReadPipe { .. } => unimplemented!(),
            Instruction::GroupCommitWritePipe { .. } => unimplemented!(),
            Instruction::EnqueueMarker { .. } => unimplemented!(),
            Instruction::EnqueueKernel { .. } => unimplemented!(),
            Instruction::GetKernelNDrangeSubGroupCount { .. } => unimplemented!(),
            Instruction::GetKernelNDrangeMaxSubGroupSize { .. } => unimplemented!(),
            Instruction::GetKernelWorkGroupSize { .. } => unimplemented!(),
            Instruction::GetKernelPreferredWorkGroupSizeMultiple { .. } => unimplemented!(),
            Instruction::RetainEvent { .. } => unimplemented!(),
            Instruction::ReleaseEvent { .. } => unimplemented!(),
            Instruction::CreateUserEvent { .. } => unimplemented!(),
            Instruction::IsValidEvent { .. } => unimplemented!(),
            Instruction::SetUserEventStatus { .. } => unimplemented!(),
            Instruction::CaptureEventProfilingInfo { .. } => unimplemented!(),
            Instruction::GetDefaultQueue { .. } => unimplemented!(),
            Instruction::BuildNDRange { .. } => unimplemented!(),
            Instruction::ImageSparseSampleImplicitLod { .. } => unimplemented!(),
            Instruction::ImageSparseSampleExplicitLod { .. } => unimplemented!(),
            Instruction::ImageSparseSampleDrefImplicitLod { .. } => unimplemented!(),
            Instruction::ImageSparseSampleDrefExplicitLod { .. } => unimplemented!(),
            Instruction::ImageSparseFetch { .. } => unimplemented!(),
            Instruction::ImageSparseGather { .. } => unimplemented!(),
            Instruction::ImageSparseDrefGather { .. } => unimplemented!(),
            Instruction::ImageSparseTexelsResident { .. } => unimplemented!(),
            Instruction::NoLine => DebugLine,
            Instruction::AtomicFlagTestAndSet { .. } => unimplemented!(),
            Instruction::AtomicFlagClear { .. } => unimplemented!(),
            Instruction::ImageSparseRead { .. } => unimplemented!(),
            Instruction::SizeOf { .. } => Simple,
            Instruction::TypePipeStorage { .. } => Type,
            Instruction::ConstantPipeStorage { .. } => unimplemented!(),
            Instruction::CreatePipeFromPipeStorage { .. } => unimplemented!(),
            Instruction::GetKernelLocalSizeForSubgroupCount { .. } => unimplemented!(),
            Instruction::GetKernelMaxNumSubgroups { .. } => unimplemented!(),
            Instruction::TypeNamedBarrier { .. } => Type,
            Instruction::NamedBarrierInitialize { .. } => unimplemented!(),
            Instruction::MemoryNamedBarrier { .. } => unimplemented!(),
            Instruction::ModuleProcessed { .. } => Debug,
            Instruction::ExecutionModeId { .. } => ModuleLevel,
            Instruction::DecorateId { .. } => Decoration,
            Instruction::GroupNonUniformElect { .. } => unimplemented!(),
            Instruction::GroupNonUniformAll { .. } => unimplemented!(),
            Instruction::GroupNonUniformAny { .. } => unimplemented!(),
            Instruction::GroupNonUniformAllEqual { .. } => unimplemented!(),
            Instruction::GroupNonUniformBroadcast { .. } => unimplemented!(),
            Instruction::GroupNonUniformBroadcastFirst { .. } => unimplemented!(),
            Instruction::GroupNonUniformBallot { .. } => unimplemented!(),
            Instruction::GroupNonUniformInverseBallot { .. } => unimplemented!(),
            Instruction::GroupNonUniformBallotBitExtract { .. } => unimplemented!(),
            Instruction::GroupNonUniformBallotBitCount { .. } => unimplemented!(),
            Instruction::GroupNonUniformBallotFindLSB { .. } => unimplemented!(),
            Instruction::GroupNonUniformBallotFindMSB { .. } => unimplemented!(),
            Instruction::GroupNonUniformShuffle { .. } => unimplemented!(),
            Instruction::GroupNonUniformShuffleXor { .. } => unimplemented!(),
            Instruction::GroupNonUniformShuffleUp { .. } => unimplemented!(),
            Instruction::GroupNonUniformShuffleDown { .. } => unimplemented!(),
            Instruction::GroupNonUniformIAdd { .. } => unimplemented!(),
            Instruction::GroupNonUniformFAdd { .. } => unimplemented!(),
            Instruction::GroupNonUniformIMul { .. } => unimplemented!(),
            Instruction::GroupNonUniformFMul { .. } => unimplemented!(),
            Instruction::GroupNonUniformSMin { .. } => unimplemented!(),
            Instruction::GroupNonUniformUMin { .. } => unimplemented!(),
            Instruction::GroupNonUniformFMin { .. } => unimplemented!(),
            Instruction::GroupNonUniformSMax { .. } => unimplemented!(),
            Instruction::GroupNonUniformUMax { .. } => unimplemented!(),
            Instruction::GroupNonUniformFMax { .. } => unimplemented!(),
            Instruction::GroupNonUniformBitwiseAnd { .. } => unimplemented!(),
            Instruction::GroupNonUniformBitwiseOr { .. } => unimplemented!(),
            Instruction::GroupNonUniformBitwiseXor { .. } => unimplemented!(),
            Instruction::GroupNonUniformLogicalAnd { .. } => unimplemented!(),
            Instruction::GroupNonUniformLogicalOr { .. } => unimplemented!(),
            Instruction::GroupNonUniformLogicalXor { .. } => unimplemented!(),
            Instruction::GroupNonUniformQuadBroadcast { .. } => unimplemented!(),
            Instruction::GroupNonUniformQuadSwap { .. } => unimplemented!(),
            Instruction::ReportIntersectionNV { .. } => unimplemented!(),
            Instruction::IgnoreIntersectionNV => unimplemented!(),
            Instruction::TerminateRayNV => unimplemented!(),
            Instruction::TraceNV { .. } => unimplemented!(),
            Instruction::TypeAccelerationStructureNV { .. } => Type,
            Instruction::ExecuteCallableNV { .. } => unimplemented!(),
            Instruction::OpenCLStdAcos { .. }
            | Instruction::OpenCLStdAcosh { .. }
            | Instruction::OpenCLStdAcospi { .. }
            | Instruction::OpenCLStdAsin { .. }
            | Instruction::OpenCLStdAsinh { .. }
            | Instruction::OpenCLStdAsinpi { .. }
            | Instruction::OpenCLStdAtan { .. }
            | Instruction::OpenCLStdAtan2 { .. }
            | Instruction::OpenCLStdAtanh { .. }
            | Instruction::OpenCLStdAtanpi { .. }
            | Instruction::OpenCLStdAtan2pi { .. }
            | Instruction::OpenCLStdCbrt { .. }
            | Instruction::OpenCLStdCeil { .. }
            | Instruction::OpenCLStdCopysign { .. }
            | Instruction::OpenCLStdCos { .. }
            | Instruction::OpenCLStdCosh { .. }
            | Instruction::OpenCLStdCospi { .. }
            | Instruction::OpenCLStdErfc { .. }
            | Instruction::OpenCLStdErf { .. }
            | Instruction::OpenCLStdExp { .. }
            | Instruction::OpenCLStdExp2 { .. }
            | Instruction::OpenCLStdExp10 { .. }
            | Instruction::OpenCLStdExpm1 { .. }
            | Instruction::OpenCLStdFabs { .. }
            | Instruction::OpenCLStdFdim { .. }
            | Instruction::OpenCLStdFloor { .. }
            | Instruction::OpenCLStdFma { .. }
            | Instruction::OpenCLStdFmax { .. }
            | Instruction::OpenCLStdFmin { .. }
            | Instruction::OpenCLStdFmod { .. }
            | Instruction::OpenCLStdFract { .. }
            | Instruction::OpenCLStdFrexp { .. }
            | Instruction::OpenCLStdHypot { .. }
            | Instruction::OpenCLStdIlogb { .. }
            | Instruction::OpenCLStdLdexp { .. }
            | Instruction::OpenCLStdLgamma { .. }
            | Instruction::OpenCLStdLgammaR { .. }
            | Instruction::OpenCLStdLog { .. }
            | Instruction::OpenCLStdLog2 { .. }
            | Instruction::OpenCLStdLog10 { .. }
            | Instruction::OpenCLStdLog1p { .. }
            | Instruction::OpenCLStdLogb { .. }
            | Instruction::OpenCLStdMad { .. }
            | Instruction::OpenCLStdMaxmag { .. }
            | Instruction::OpenCLStdMinmag { .. }
            | Instruction::OpenCLStdModf { .. }
            | Instruction::OpenCLStdNan { .. }
            | Instruction::OpenCLStdNextafter { .. }
            | Instruction::OpenCLStdPow { .. }
            | Instruction::OpenCLStdPown { .. }
            | Instruction::OpenCLStdPowr { .. }
            | Instruction::OpenCLStdRemainder { .. }
            | Instruction::OpenCLStdRemquo { .. }
            | Instruction::OpenCLStdRint { .. }
            | Instruction::OpenCLStdRootn { .. }
            | Instruction::OpenCLStdRound { .. }
            | Instruction::OpenCLStdRsqrt { .. }
            | Instruction::OpenCLStdSin { .. }
            | Instruction::OpenCLStdSincos { .. }
            | Instruction::OpenCLStdSinh { .. }
            | Instruction::OpenCLStdSinpi { .. }
            | Instruction::OpenCLStdSqrt { .. }
            | Instruction::OpenCLStdTan { .. }
            | Instruction::OpenCLStdTanh { .. }
            | Instruction::OpenCLStdTanpi { .. }
            | Instruction::OpenCLStdTgamma { .. }
            | Instruction::OpenCLStdTrunc { .. }
            | Instruction::OpenCLStdHalfCos { .. }
            | Instruction::OpenCLStdHalfDivide { .. }
            | Instruction::OpenCLStdHalfExp { .. }
            | Instruction::OpenCLStdHalfExp2 { .. }
            | Instruction::OpenCLStdHalfExp10 { .. }
            | Instruction::OpenCLStdHalfLog { .. }
            | Instruction::OpenCLStdHalfLog2 { .. }
            | Instruction::OpenCLStdHalfLog10 { .. }
            | Instruction::OpenCLStdHalfPowr { .. }
            | Instruction::OpenCLStdHalfRecip { .. }
            | Instruction::OpenCLStdHalfRsqrt { .. }
            | Instruction::OpenCLStdHalfSin { .. }
            | Instruction::OpenCLStdHalfSqrt { .. }
            | Instruction::OpenCLStdHalfTan { .. }
            | Instruction::OpenCLStdNativeCos { .. }
            | Instruction::OpenCLStdNativeDivide { .. }
            | Instruction::OpenCLStdNativeExp { .. }
            | Instruction::OpenCLStdNativeExp2 { .. }
            | Instruction::OpenCLStdNativeExp10 { .. }
            | Instruction::OpenCLStdNativeLog { .. }
            | Instruction::OpenCLStdNativeLog2 { .. }
            | Instruction::OpenCLStdNativeLog10 { .. }
            | Instruction::OpenCLStdNativePowr { .. }
            | Instruction::OpenCLStdNativeRecip { .. }
            | Instruction::OpenCLStdNativeRsqrt { .. }
            | Instruction::OpenCLStdNativeSin { .. }
            | Instruction::OpenCLStdNativeSqrt { .. }
            | Instruction::OpenCLStdNativeTan { .. }
            | Instruction::OpenCLStdSAbs { .. }
            | Instruction::OpenCLStdSAbsDiff { .. }
            | Instruction::OpenCLStdSAddSat { .. }
            | Instruction::OpenCLStdUAddSat { .. }
            | Instruction::OpenCLStdSHadd { .. }
            | Instruction::OpenCLStdUHadd { .. }
            | Instruction::OpenCLStdSRhadd { .. }
            | Instruction::OpenCLStdURhadd { .. }
            | Instruction::OpenCLStdSClamp { .. }
            | Instruction::OpenCLStdUClamp { .. }
            | Instruction::OpenCLStdClz { .. }
            | Instruction::OpenCLStdCtz { .. }
            | Instruction::OpenCLStdSMadHi { .. }
            | Instruction::OpenCLStdUMadSat { .. }
            | Instruction::OpenCLStdSMadSat { .. }
            | Instruction::OpenCLStdSMax { .. }
            | Instruction::OpenCLStdUMax { .. }
            | Instruction::OpenCLStdSMin { .. }
            | Instruction::OpenCLStdUMin { .. }
            | Instruction::OpenCLStdSMulHi { .. }
            | Instruction::OpenCLStdRotate { .. }
            | Instruction::OpenCLStdSSubSat { .. }
            | Instruction::OpenCLStdUSubSat { .. }
            | Instruction::OpenCLStdUUpsample { .. }
            | Instruction::OpenCLStdSUpsample { .. }
            | Instruction::OpenCLStdPopcount { .. }
            | Instruction::OpenCLStdSMad24 { .. }
            | Instruction::OpenCLStdUMad24 { .. }
            | Instruction::OpenCLStdSMul24 { .. }
            | Instruction::OpenCLStdUMul24 { .. }
            | Instruction::OpenCLStdUAbs { .. }
            | Instruction::OpenCLStdUAbsDiff { .. }
            | Instruction::OpenCLStdUMulHi { .. }
            | Instruction::OpenCLStdUMadHi { .. }
            | Instruction::OpenCLStdFclamp { .. }
            | Instruction::OpenCLStdDegrees { .. }
            | Instruction::OpenCLStdFmaxCommon { .. }
            | Instruction::OpenCLStdFminCommon { .. }
            | Instruction::OpenCLStdMix { .. }
            | Instruction::OpenCLStdRadians { .. }
            | Instruction::OpenCLStdStep { .. }
            | Instruction::OpenCLStdSmoothstep { .. }
            | Instruction::OpenCLStdSign { .. }
            | Instruction::OpenCLStdCross { .. }
            | Instruction::OpenCLStdDistance { .. }
            | Instruction::OpenCLStdLength { .. }
            | Instruction::OpenCLStdNormalize { .. }
            | Instruction::OpenCLStdFastDistance { .. }
            | Instruction::OpenCLStdFastLength { .. }
            | Instruction::OpenCLStdFastNormalize { .. }
            | Instruction::OpenCLStdBitselect { .. }
            | Instruction::OpenCLStdSelect { .. } => Simple,
            Instruction::OpenCLStdVloadn { .. } => unimplemented!(),
            Instruction::OpenCLStdVstoren { .. } => unimplemented!(),
            Instruction::OpenCLStdVloadHalf { .. } => unimplemented!(),
            Instruction::OpenCLStdVloadHalfn { .. } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalf { .. } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalfR { .. } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalfn { .. } => unimplemented!(),
            Instruction::OpenCLStdVstoreHalfnR { .. } => unimplemented!(),
            Instruction::OpenCLStdVloadaHalfn { .. } => unimplemented!(),
            Instruction::OpenCLStdVstoreaHalfn { .. } => unimplemented!(),
            Instruction::OpenCLStdVstoreaHalfnR { .. } => unimplemented!(),
            Instruction::OpenCLStdShuffle { .. } | Instruction::OpenCLStdShuffle2 { .. } => Simple,
            Instruction::OpenCLStdPrintf { .. } => unimplemented!(),
            Instruction::OpenCLStdPrefetch { .. } => unimplemented!(),
            Instruction::GLSLStd450Round { .. }
            | Instruction::GLSLStd450RoundEven { .. }
            | Instruction::GLSLStd450Trunc { .. }
            | Instruction::GLSLStd450FAbs { .. }
            | Instruction::GLSLStd450SAbs { .. }
            | Instruction::GLSLStd450FSign { .. }
            | Instruction::GLSLStd450SSign { .. }
            | Instruction::GLSLStd450Floor { .. }
            | Instruction::GLSLStd450Ceil { .. }
            | Instruction::GLSLStd450Fract { .. }
            | Instruction::GLSLStd450Radians { .. }
            | Instruction::GLSLStd450Degrees { .. }
            | Instruction::GLSLStd450Sin { .. }
            | Instruction::GLSLStd450Cos { .. }
            | Instruction::GLSLStd450Tan { .. }
            | Instruction::GLSLStd450Asin { .. }
            | Instruction::GLSLStd450Acos { .. }
            | Instruction::GLSLStd450Atan { .. }
            | Instruction::GLSLStd450Sinh { .. }
            | Instruction::GLSLStd450Cosh { .. }
            | Instruction::GLSLStd450Tanh { .. }
            | Instruction::GLSLStd450Asinh { .. }
            | Instruction::GLSLStd450Acosh { .. }
            | Instruction::GLSLStd450Atanh { .. }
            | Instruction::GLSLStd450Atan2 { .. }
            | Instruction::GLSLStd450Pow { .. }
            | Instruction::GLSLStd450Exp { .. }
            | Instruction::GLSLStd450Log { .. }
            | Instruction::GLSLStd450Exp2 { .. }
            | Instruction::GLSLStd450Log2 { .. }
            | Instruction::GLSLStd450Sqrt { .. }
            | Instruction::GLSLStd450InverseSqrt { .. }
            | Instruction::GLSLStd450Determinant { .. }
            | Instruction::GLSLStd450MatrixInverse { .. }
            | Instruction::GLSLStd450Modf { .. }
            | Instruction::GLSLStd450ModfStruct { .. }
            | Instruction::GLSLStd450FMin { .. }
            | Instruction::GLSLStd450UMin { .. }
            | Instruction::GLSLStd450SMin { .. }
            | Instruction::GLSLStd450FMax { .. }
            | Instruction::GLSLStd450UMax { .. }
            | Instruction::GLSLStd450SMax { .. }
            | Instruction::GLSLStd450FClamp { .. }
            | Instruction::GLSLStd450UClamp { .. }
            | Instruction::GLSLStd450SClamp { .. }
            | Instruction::GLSLStd450FMix { .. }
            | Instruction::GLSLStd450IMix { .. }
            | Instruction::GLSLStd450Step { .. }
            | Instruction::GLSLStd450SmoothStep { .. }
            | Instruction::GLSLStd450Fma { .. }
            | Instruction::GLSLStd450Frexp { .. }
            | Instruction::GLSLStd450FrexpStruct { .. }
            | Instruction::GLSLStd450Ldexp { .. }
            | Instruction::GLSLStd450PackSnorm4x8 { .. }
            | Instruction::GLSLStd450PackUnorm4x8 { .. }
            | Instruction::GLSLStd450PackSnorm2x16 { .. }
            | Instruction::GLSLStd450PackUnorm2x16 { .. }
            | Instruction::GLSLStd450PackHalf2x16 { .. }
            | Instruction::GLSLStd450PackDouble2x32 { .. }
            | Instruction::GLSLStd450UnpackSnorm2x16 { .. }
            | Instruction::GLSLStd450UnpackUnorm2x16 { .. }
            | Instruction::GLSLStd450UnpackHalf2x16 { .. }
            | Instruction::GLSLStd450UnpackSnorm4x8 { .. }
            | Instruction::GLSLStd450UnpackUnorm4x8 { .. }
            | Instruction::GLSLStd450UnpackDouble2x32 { .. }
            | Instruction::GLSLStd450Length { .. }
            | Instruction::GLSLStd450Distance { .. }
            | Instruction::GLSLStd450Cross { .. }
            | Instruction::GLSLStd450Normalize { .. }
            | Instruction::GLSLStd450FaceForward { .. }
            | Instruction::GLSLStd450Reflect { .. }
            | Instruction::GLSLStd450Refract { .. }
            | Instruction::GLSLStd450FindILsb { .. }
            | Instruction::GLSLStd450FindSMsb { .. }
            | Instruction::GLSLStd450FindUMsb { .. } => Simple,
            Instruction::GLSLStd450InterpolateAtCentroid { .. }
            | Instruction::GLSLStd450InterpolateAtSample { .. }
            | Instruction::GLSLStd450InterpolateAtOffset { .. } => InterpolateAt,
            Instruction::GLSLStd450NMin { .. }
            | Instruction::GLSLStd450NMax { .. }
            | Instruction::GLSLStd450NClamp { .. } => Simple,
        }
    }
}
