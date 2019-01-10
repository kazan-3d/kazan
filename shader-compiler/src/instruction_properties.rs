// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

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
        macro_rules! result_and_type {
            {$instruction:expr; $($none:ident),*; $($result_without_type:ident),*; $($result_and_type:ident),*} => {
                match $instruction {
                    $(Instruction::$none {..})|* => None,
                    $(Instruction::$result_without_type{id_result, ..})|* => Some((id_result, None)),
                    $(Instruction::$result_and_type{id_result_type, id_result, ..})|* => Some((id_result, Some(id_result_type))),
                }
            };
        }
        result_and_type! {
            *self.instruction;
            // no IdResult or IdResultType
            Nop,
            SourceContinued,
            Source,
            SourceExtension,
            Name,
            MemberName,
            Line,
            Extension,
            MemoryModel,
            EntryPoint,
            ExecutionMode,
            Capability,
            TypeForwardPointer,
            SpecConstantOp,
            FunctionEnd,
            Store,
            CopyMemory,
            CopyMemorySized,
            Decorate,
            MemberDecorate,
            GroupDecorate,
            GroupMemberDecorate,
            ImageWrite,
            EmitVertex,
            EndPrimitive,
            EmitStreamVertex,
            EndStreamPrimitive,
            ControlBarrier,
            MemoryBarrier,
            AtomicStore,
            LoopMerge,
            SelectionMerge,
            Branch,
            BranchConditional,
            Switch32,
            Switch64,
            Kill,
            Return,
            ReturnValue,
            Unreachable,
            LifetimeStart,
            LifetimeStop,
            GroupWaitEvents,
            CommitReadPipe,
            CommitWritePipe,
            GroupCommitReadPipe,
            GroupCommitWritePipe,
            RetainEvent,
            ReleaseEvent,
            SetUserEventStatus,
            CaptureEventProfilingInfo,
            NoLine,
            AtomicFlagClear,
            MemoryNamedBarrier,
            ModuleProcessed,
            ExecutionModeId,
            DecorateId,
            IgnoreIntersectionNV,
            TerminateRayNV,
            TraceNV,
            ExecuteCallableNV;

            // IdResult but no IdResultType
            String,
            ExtInstImport,
            TypeVoid,
            TypeBool,
            TypeInt,
            TypeFloat,
            TypeVector,
            TypeMatrix,
            TypeImage,
            TypeSampler,
            TypeSampledImage,
            TypeArray,
            TypeRuntimeArray,
            TypeStruct,
            TypeOpaque,
            TypePointer,
            TypeFunction,
            TypeEvent,
            TypeDeviceEvent,
            TypeReserveId,
            TypeQueue,
            TypePipe,
            DecorationGroup,
            Label,
            TypePipeStorage,
            TypeNamedBarrier,
            TypeAccelerationStructureNV;

            // IdResult and IdResultType
            Undef,
            ExtInst,
            ConstantTrue,
            ConstantFalse,
            Constant32,
            Constant64,
            ConstantComposite,
            ConstantSampler,
            ConstantNull,
            SpecConstantTrue,
            SpecConstantFalse,
            SpecConstant32,
            SpecConstant64,
            SpecConstantComposite,
            Function,
            FunctionParameter,
            FunctionCall,
            Variable,
            ImageTexelPointer,
            Load,
            AccessChain,
            InBoundsAccessChain,
            PtrAccessChain,
            ArrayLength,
            GenericPtrMemSemantics,
            InBoundsPtrAccessChain,
            VectorExtractDynamic,
            VectorInsertDynamic,
            VectorShuffle,
            CompositeConstruct,
            CompositeExtract,
            CompositeInsert,
            CopyObject,
            Transpose,
            SampledImage,
            ImageSampleImplicitLod,
            ImageSampleExplicitLod,
            ImageSampleDrefImplicitLod,
            ImageSampleDrefExplicitLod,
            ImageSampleProjImplicitLod,
            ImageSampleProjExplicitLod,
            ImageSampleProjDrefImplicitLod,
            ImageSampleProjDrefExplicitLod,
            ImageFetch,
            ImageGather,
            ImageDrefGather,
            ImageRead,
            Image,
            ImageQueryFormat,
            ImageQueryOrder,
            ImageQuerySizeLod,
            ImageQuerySize,
            ImageQueryLod,
            ImageQueryLevels,
            ImageQuerySamples,
            ConvertFToU,
            ConvertFToS,
            ConvertSToF,
            ConvertUToF,
            UConvert,
            SConvert,
            FConvert,
            QuantizeToF16,
            ConvertPtrToU,
            SatConvertSToU,
            SatConvertUToS,
            ConvertUToPtr,
            PtrCastToGeneric,
            GenericCastToPtr,
            GenericCastToPtrExplicit,
            Bitcast,
            SNegate,
            FNegate,
            IAdd,
            FAdd,
            ISub,
            FSub,
            IMul,
            FMul,
            UDiv,
            SDiv,
            FDiv,
            UMod,
            SRem,
            SMod,
            FRem,
            FMod,
            VectorTimesScalar,
            MatrixTimesScalar,
            VectorTimesMatrix,
            MatrixTimesVector,
            MatrixTimesMatrix,
            OuterProduct,
            Dot,
            IAddCarry,
            ISubBorrow,
            UMulExtended,
            SMulExtended,
            Any,
            All,
            IsNan,
            IsInf,
            IsFinite,
            IsNormal,
            SignBitSet,
            LessOrGreater,
            Ordered,
            Unordered,
            LogicalEqual,
            LogicalNotEqual,
            LogicalOr,
            LogicalAnd,
            LogicalNot,
            Select,
            IEqual,
            INotEqual,
            UGreaterThan,
            SGreaterThan,
            UGreaterThanEqual,
            SGreaterThanEqual,
            ULessThan,
            SLessThan,
            ULessThanEqual,
            SLessThanEqual,
            FOrdEqual,
            FUnordEqual,
            FOrdNotEqual,
            FUnordNotEqual,
            FOrdLessThan,
            FUnordLessThan,
            FOrdGreaterThan,
            FUnordGreaterThan,
            FOrdLessThanEqual,
            FUnordLessThanEqual,
            FOrdGreaterThanEqual,
            FUnordGreaterThanEqual,
            ShiftRightLogical,
            ShiftRightArithmetic,
            ShiftLeftLogical,
            BitwiseOr,
            BitwiseXor,
            BitwiseAnd,
            Not,
            BitFieldInsert,
            BitFieldSExtract,
            BitFieldUExtract,
            BitReverse,
            BitCount,
            DPdx,
            DPdy,
            Fwidth,
            DPdxFine,
            DPdyFine,
            FwidthFine,
            DPdxCoarse,
            DPdyCoarse,
            FwidthCoarse,
            AtomicLoad,
            AtomicExchange,
            AtomicCompareExchange,
            AtomicCompareExchangeWeak,
            AtomicIIncrement,
            AtomicIDecrement,
            AtomicIAdd,
            AtomicISub,
            AtomicSMin,
            AtomicUMin,
            AtomicSMax,
            AtomicUMax,
            AtomicAnd,
            AtomicOr,
            AtomicXor,
            Phi,
            GroupAsyncCopy,
            GroupAll,
            GroupAny,
            GroupBroadcast,
            GroupIAdd,
            GroupFAdd,
            GroupFMin,
            GroupUMin,
            GroupSMin,
            GroupFMax,
            GroupUMax,
            GroupSMax,
            ReadPipe,
            WritePipe,
            ReservedReadPipe,
            ReservedWritePipe,
            ReserveReadPipePackets,
            ReserveWritePipePackets,
            IsValidReserveId,
            GetNumPipePackets,
            GetMaxPipePackets,
            GroupReserveReadPipePackets,
            GroupReserveWritePipePackets,
            EnqueueMarker,
            EnqueueKernel,
            GetKernelNDrangeSubGroupCount,
            GetKernelNDrangeMaxSubGroupSize,
            GetKernelWorkGroupSize,
            GetKernelPreferredWorkGroupSizeMultiple,
            CreateUserEvent,
            IsValidEvent,
            GetDefaultQueue,
            BuildNDRange,
            ImageSparseSampleImplicitLod,
            ImageSparseSampleExplicitLod,
            ImageSparseSampleDrefImplicitLod,
            ImageSparseSampleDrefExplicitLod,
            ImageSparseFetch,
            ImageSparseGather,
            ImageSparseDrefGather,
            ImageSparseTexelsResident,
            AtomicFlagTestAndSet,
            ImageSparseRead,
            SizeOf,
            ConstantPipeStorage,
            CreatePipeFromPipeStorage,
            GetKernelLocalSizeForSubgroupCount,
            GetKernelMaxNumSubgroups,
            NamedBarrierInitialize,
            GroupNonUniformElect,
            GroupNonUniformAll,
            GroupNonUniformAny,
            GroupNonUniformAllEqual,
            GroupNonUniformBroadcast,
            GroupNonUniformBroadcastFirst,
            GroupNonUniformBallot,
            GroupNonUniformInverseBallot,
            GroupNonUniformBallotBitExtract,
            GroupNonUniformBallotBitCount,
            GroupNonUniformBallotFindLSB,
            GroupNonUniformBallotFindMSB,
            GroupNonUniformShuffle,
            GroupNonUniformShuffleXor,
            GroupNonUniformShuffleUp,
            GroupNonUniformShuffleDown,
            GroupNonUniformIAdd,
            GroupNonUniformFAdd,
            GroupNonUniformIMul,
            GroupNonUniformFMul,
            GroupNonUniformSMin,
            GroupNonUniformUMin,
            GroupNonUniformFMin,
            GroupNonUniformSMax,
            GroupNonUniformUMax,
            GroupNonUniformFMax,
            GroupNonUniformBitwiseAnd,
            GroupNonUniformBitwiseOr,
            GroupNonUniformBitwiseXor,
            GroupNonUniformLogicalAnd,
            GroupNonUniformLogicalOr,
            GroupNonUniformLogicalXor,
            GroupNonUniformQuadBroadcast,
            GroupNonUniformQuadSwap,
            ReportIntersectionNV,
            OpenCLStdAcos,
            OpenCLStdAcosh,
            OpenCLStdAcospi,
            OpenCLStdAsin,
            OpenCLStdAsinh,
            OpenCLStdAsinpi,
            OpenCLStdAtan,
            OpenCLStdAtan2,
            OpenCLStdAtanh,
            OpenCLStdAtanpi,
            OpenCLStdAtan2pi,
            OpenCLStdCbrt,
            OpenCLStdCeil,
            OpenCLStdCopysign,
            OpenCLStdCos,
            OpenCLStdCosh,
            OpenCLStdCospi,
            OpenCLStdErfc,
            OpenCLStdErf,
            OpenCLStdExp,
            OpenCLStdExp2,
            OpenCLStdExp10,
            OpenCLStdExpm1,
            OpenCLStdFabs,
            OpenCLStdFdim,
            OpenCLStdFloor,
            OpenCLStdFma,
            OpenCLStdFmax,
            OpenCLStdFmin,
            OpenCLStdFmod,
            OpenCLStdFract,
            OpenCLStdFrexp,
            OpenCLStdHypot,
            OpenCLStdIlogb,
            OpenCLStdLdexp,
            OpenCLStdLgamma,
            OpenCLStdLgammaR,
            OpenCLStdLog,
            OpenCLStdLog2,
            OpenCLStdLog10,
            OpenCLStdLog1p,
            OpenCLStdLogb,
            OpenCLStdMad,
            OpenCLStdMaxmag,
            OpenCLStdMinmag,
            OpenCLStdModf,
            OpenCLStdNan,
            OpenCLStdNextafter,
            OpenCLStdPow,
            OpenCLStdPown,
            OpenCLStdPowr,
            OpenCLStdRemainder,
            OpenCLStdRemquo,
            OpenCLStdRint,
            OpenCLStdRootn,
            OpenCLStdRound,
            OpenCLStdRsqrt,
            OpenCLStdSin,
            OpenCLStdSincos,
            OpenCLStdSinh,
            OpenCLStdSinpi,
            OpenCLStdSqrt,
            OpenCLStdTan,
            OpenCLStdTanh,
            OpenCLStdTanpi,
            OpenCLStdTgamma,
            OpenCLStdTrunc,
            OpenCLStdHalfCos,
            OpenCLStdHalfDivide,
            OpenCLStdHalfExp,
            OpenCLStdHalfExp2,
            OpenCLStdHalfExp10,
            OpenCLStdHalfLog,
            OpenCLStdHalfLog2,
            OpenCLStdHalfLog10,
            OpenCLStdHalfPowr,
            OpenCLStdHalfRecip,
            OpenCLStdHalfRsqrt,
            OpenCLStdHalfSin,
            OpenCLStdHalfSqrt,
            OpenCLStdHalfTan,
            OpenCLStdNativeCos,
            OpenCLStdNativeDivide,
            OpenCLStdNativeExp,
            OpenCLStdNativeExp2,
            OpenCLStdNativeExp10,
            OpenCLStdNativeLog,
            OpenCLStdNativeLog2,
            OpenCLStdNativeLog10,
            OpenCLStdNativePowr,
            OpenCLStdNativeRecip,
            OpenCLStdNativeRsqrt,
            OpenCLStdNativeSin,
            OpenCLStdNativeSqrt,
            OpenCLStdNativeTan,
            OpenCLStdSAbs,
            OpenCLStdSAbsDiff,
            OpenCLStdSAddSat,
            OpenCLStdUAddSat,
            OpenCLStdSHadd,
            OpenCLStdUHadd,
            OpenCLStdSRhadd,
            OpenCLStdURhadd,
            OpenCLStdSClamp,
            OpenCLStdUClamp,
            OpenCLStdClz,
            OpenCLStdCtz,
            OpenCLStdSMadHi,
            OpenCLStdUMadSat,
            OpenCLStdSMadSat,
            OpenCLStdSMax,
            OpenCLStdUMax,
            OpenCLStdSMin,
            OpenCLStdUMin,
            OpenCLStdSMulHi,
            OpenCLStdRotate,
            OpenCLStdSSubSat,
            OpenCLStdUSubSat,
            OpenCLStdUUpsample,
            OpenCLStdSUpsample,
            OpenCLStdPopcount,
            OpenCLStdSMad24,
            OpenCLStdUMad24,
            OpenCLStdSMul24,
            OpenCLStdUMul24,
            OpenCLStdUAbs,
            OpenCLStdUAbsDiff,
            OpenCLStdUMulHi,
            OpenCLStdUMadHi,
            OpenCLStdFclamp,
            OpenCLStdDegrees,
            OpenCLStdFmaxCommon,
            OpenCLStdFminCommon,
            OpenCLStdMix,
            OpenCLStdRadians,
            OpenCLStdStep,
            OpenCLStdSmoothstep,
            OpenCLStdSign,
            OpenCLStdCross,
            OpenCLStdDistance,
            OpenCLStdLength,
            OpenCLStdNormalize,
            OpenCLStdFastDistance,
            OpenCLStdFastLength,
            OpenCLStdFastNormalize,
            OpenCLStdBitselect,
            OpenCLStdSelect,
            OpenCLStdVloadn,
            OpenCLStdVstoren,
            OpenCLStdVloadHalf,
            OpenCLStdVloadHalfn,
            OpenCLStdVstoreHalf,
            OpenCLStdVstoreHalfR,
            OpenCLStdVstoreHalfn,
            OpenCLStdVstoreHalfnR,
            OpenCLStdVloadaHalfn,
            OpenCLStdVstoreaHalfn,
            OpenCLStdVstoreaHalfnR,
            OpenCLStdShuffle,
            OpenCLStdShuffle2,
            OpenCLStdPrintf,
            OpenCLStdPrefetch,
            GLSLStd450Round,
            GLSLStd450RoundEven,
            GLSLStd450Trunc,
            GLSLStd450FAbs,
            GLSLStd450SAbs,
            GLSLStd450FSign,
            GLSLStd450SSign,
            GLSLStd450Floor,
            GLSLStd450Ceil,
            GLSLStd450Fract,
            GLSLStd450Radians,
            GLSLStd450Degrees,
            GLSLStd450Sin,
            GLSLStd450Cos,
            GLSLStd450Tan,
            GLSLStd450Asin,
            GLSLStd450Acos,
            GLSLStd450Atan,
            GLSLStd450Sinh,
            GLSLStd450Cosh,
            GLSLStd450Tanh,
            GLSLStd450Asinh,
            GLSLStd450Acosh,
            GLSLStd450Atanh,
            GLSLStd450Atan2,
            GLSLStd450Pow,
            GLSLStd450Exp,
            GLSLStd450Log,
            GLSLStd450Exp2,
            GLSLStd450Log2,
            GLSLStd450Sqrt,
            GLSLStd450InverseSqrt,
            GLSLStd450Determinant,
            GLSLStd450MatrixInverse,
            GLSLStd450Modf,
            GLSLStd450ModfStruct,
            GLSLStd450FMin,
            GLSLStd450UMin,
            GLSLStd450SMin,
            GLSLStd450FMax,
            GLSLStd450UMax,
            GLSLStd450SMax,
            GLSLStd450FClamp,
            GLSLStd450UClamp,
            GLSLStd450SClamp,
            GLSLStd450FMix,
            GLSLStd450IMix,
            GLSLStd450Step,
            GLSLStd450SmoothStep,
            GLSLStd450Fma,
            GLSLStd450Frexp,
            GLSLStd450FrexpStruct,
            GLSLStd450Ldexp,
            GLSLStd450PackSnorm4x8,
            GLSLStd450PackUnorm4x8,
            GLSLStd450PackSnorm2x16,
            GLSLStd450PackUnorm2x16,
            GLSLStd450PackHalf2x16,
            GLSLStd450PackDouble2x32,
            GLSLStd450UnpackSnorm2x16,
            GLSLStd450UnpackUnorm2x16,
            GLSLStd450UnpackHalf2x16,
            GLSLStd450UnpackSnorm4x8,
            GLSLStd450UnpackUnorm4x8,
            GLSLStd450UnpackDouble2x32,
            GLSLStd450Length,
            GLSLStd450Distance,
            GLSLStd450Cross,
            GLSLStd450Normalize,
            GLSLStd450FaceForward,
            GLSLStd450Reflect,
            GLSLStd450Refract,
            GLSLStd450FindILsb,
            GLSLStd450FindSMsb,
            GLSLStd450FindUMsb,
            GLSLStd450InterpolateAtCentroid,
            GLSLStd450InterpolateAtSample,
            GLSLStd450InterpolateAtOffset,
            GLSLStd450NMin,
            GLSLStd450NMax,
            GLSLStd450NClamp
        }
    }
    pub fn result(self) -> Option<IdResult> {
        Some(self.result_and_type()?.0)
    }
    pub fn result_type(self) -> Option<IdResultType> {
        self.result_and_type()?.1
    }
}
