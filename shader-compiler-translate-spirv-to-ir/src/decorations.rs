// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::errors::{decoration_not_allowed, TranslationResult};
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};
use spirv_parser::{DecorationComponent, DecorationLocation, DecorationUniformId};

impl_spirv_enum_partition! {
    /// partitioned form of `Decoration`
    pub(crate) enum DecorationClass(Decoration) {
        RelaxedPrecision(DecorationClassRelaxedPrecision {
            RelaxedPrecision(DecorationRelaxedPrecision),
        }),
        Misc(DecorationClassMisc {
            SpecId(DecorationSpecId),
            BuiltIn(DecorationBuiltIn),
            FPRoundingMode(DecorationFPRoundingMode),
            ArrayStride(DecorationArrayStride),
        }),
        /// decorations on `OpTypeStruct`
        Struct(DecorationClassStruct {
            Block(DecorationBlock),
            BufferBlock(DecorationBufferBlock),
            GLSLShared(DecorationGLSLShared),
            GLSLPacked(DecorationGLSLPacked),
        }),
        /// decorations on struct members
        StructMember(DecorationClassStructMember {
            RowMajor(DecorationRowMajor),
            ColMajor(DecorationColMajor),
            MatrixStride(DecorationMatrixStride),
            Offset(DecorationOffset),
        }),
        /// ignored decorations
        Ignored(DecorationClassIgnored {
            CounterBuffer(DecorationCounterBuffer),
            UserSemantic(DecorationUserSemantic),
        }),
        /// decorations that are not allowed
        Invalid(DecorationClassInvalid {
            CPacked(DecorationCPacked),
            Constant(DecorationConstant),
            SaturatedConversion(DecorationSaturatedConversion),
            FuncParamAttr(DecorationFuncParamAttr),
            FPFastMathMode(DecorationFPFastMathMode),
            LinkageAttributes(DecorationLinkageAttributes),
            Alignment(DecorationAlignment),
            MaxByteOffset(DecorationMaxByteOffset),
            AlignmentId(DecorationAlignmentId),
            MaxByteOffsetId(DecorationMaxByteOffsetId),
        }),
        /// decorations for memory object declarations or struct members
        MemoryObjectDeclarationOrStructMember(DecorationClassMemoryObjectDeclarationOrStructMember {
            NoPerspective(DecorationNoPerspective),
            Flat(DecorationFlat),
            Patch(DecorationPatch),
            Centroid(DecorationCentroid),
            Sample(DecorationSample),
            Volatile(DecorationVolatile),
            Coherent(DecorationCoherent),
            NonWritable(DecorationNonWritable),
            NonReadable(DecorationNonReadable),
            Stream(DecorationStream),
            Component(DecorationComponent),
            XfbBuffer(DecorationXfbBuffer),
            XfbStride(DecorationXfbStride),
        }),
        /// decorations for memory object declarations
        MemoryObjectDeclaration(DecorationClassMemoryObjectDeclaration {
            Restrict(DecorationRestrict),
            Aliased(DecorationAliased),
        }),
        /// decorations for variables or struct members
        VariableOrStructMember(DecorationClassVariableOrStructMember {
            Invariant(DecorationInvariant),
            Location(DecorationLocation),
        }),
        /// decorations for objects
        Object(DecorationClassObject {
            Uniform(DecorationUniform),
            UniformId(DecorationUniformId),
            NoContraction(DecorationNoContraction),
            NoSignedWrap(DecorationNoSignedWrap),
            NoUnsignedWrap(DecorationNoUnsignedWrap),
            NonUniform(DecorationNonUniform),
        }),
        /// decorations for variables
        Variable(DecorationClassVariable {
            Index(DecorationIndex),
            Binding(DecorationBinding),
            DescriptorSet(DecorationDescriptorSet),
            InputAttachmentIndex(DecorationInputAttachmentIndex),
            RestrictPointer(DecorationRestrictPointer),
            AliasedPointer(DecorationAliasedPointer),
        }),
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum InterpolationKind {
    Flat,
    NoPerspective,
    Perspective,
}

#[derive(Debug)]
pub(crate) struct MemoryObjectDeclarationOrStructMember {
    pub(crate) interpolation_kind: InterpolationKind,
    pub(crate) centroid: bool,
    pub(crate) volatile: bool,
    pub(crate) coherent: bool,
    pub(crate) writable: bool,
    pub(crate) readable: bool,
    pub(crate) component: Option<u32>,
}

#[derive(Debug)]
pub(crate) struct VariableOrStructMember {
    pub(crate) invariant: bool,
    pub(crate) location: Option<u32>,
}

#[derive(Debug)]
pub(crate) struct MemoryObjectDeclaration {
    pub(crate) restrict: bool,
    pub(crate) aliased: bool,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum Uniformity {
    Unknown,
    Uniform,
    NonUniform,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum Wrapping {
    Allowed,
    UndefinedBehavior,
}

#[derive(Clone, Debug)]
pub(crate) struct SPIRVObject {
    pub(crate) uniformity: Uniformity,
    pub(crate) signed_wrapping: Wrapping,
    pub(crate) unsigned_wrapping: Wrapping,
    pub(crate) fp_contraction_allowed: bool,
}

pub(crate) trait DecorationAspect: Sized + core::fmt::Debug {
    type DecorationClass: Into<DecorationClass>
        + Into<spirv_parser::Decoration>
        + Clone
        + core::fmt::Debug;
    fn parse_decorations<I: FnOnce() -> spirv_parser::Instruction>(
        decorations: Vec<Self::DecorationClass>,
        member_index: Option<u32>,
        instruction: I,
    ) -> TranslationResult<Self>;
}

impl DecorationAspect for MemoryObjectDeclaration {
    type DecorationClass = DecorationClassMemoryObjectDeclaration;
    fn parse_decorations<I: FnOnce() -> spirv_parser::Instruction>(
        decorations: Vec<Self::DecorationClass>,
        _member_index: Option<u32>,
        _instruction: I,
    ) -> TranslationResult<Self> {
        let mut restrict = false;
        let mut aliased = false;
        use DecorationClassMemoryObjectDeclaration::*;
        for decoration in decorations {
            match decoration {
                Restrict(_) => restrict = true,
                Aliased(_) => aliased = true,
            }
        }
        Ok(Self { restrict, aliased })
    }
}

impl DecorationAspect for MemoryObjectDeclarationOrStructMember {
    type DecorationClass = DecorationClassMemoryObjectDeclarationOrStructMember;
    fn parse_decorations<I: FnOnce() -> spirv_parser::Instruction>(
        decorations: Vec<Self::DecorationClass>,
        member_index: Option<u32>,
        instruction: I,
    ) -> TranslationResult<Self> {
        let mut interpolation_kind = InterpolationKind::Perspective;
        let mut centroid = false;
        let mut volatile = false;
        let mut coherent = false;
        let mut writable = true;
        let mut readable = true;
        let mut component = None;
        use DecorationClassMemoryObjectDeclarationOrStructMember::*;
        for decoration in decorations {
            match decoration {
                NoPerspective(_) => {
                    interpolation_kind = InterpolationKind::NoPerspective;
                }
                Flat(_) => {
                    interpolation_kind = InterpolationKind::Flat;
                }
                Centroid(_) => centroid = true,
                Volatile(_) => volatile = true,
                Coherent(_) => coherent = true,
                NonWritable(_) => writable = false,
                NonReadable(_) => readable = false,
                Component(DecorationComponent { component: v }) => component = Some(v),
                // tessellation
                Patch(_)
                // sample rate shading
                | Sample(_)
                // geometry streams
                | Stream(_)
                // transform feedback
                | XfbBuffer(_) | XfbStride(_) => {
                    return Err(decoration_not_allowed(
                        member_index,
                        decoration.into(),
                        instruction(),
                    ));
                }
            }
        }
        Ok(Self {
            interpolation_kind,
            centroid,
            volatile,
            coherent,
            writable,
            readable,
            component,
        })
    }
}

impl DecorationAspect for VariableOrStructMember {
    type DecorationClass = DecorationClassVariableOrStructMember;
    fn parse_decorations<I: FnOnce() -> spirv_parser::Instruction>(
        decorations: Vec<Self::DecorationClass>,
        _member_index: Option<u32>,
        _instruction: I,
    ) -> TranslationResult<Self> {
        let mut invariant = false;
        let mut location = None;
        use DecorationClassVariableOrStructMember::*;
        for decoration in decorations {
            match decoration {
                Invariant(_) => invariant = true,
                Location(DecorationLocation { location: v }) => location = Some(v),
            }
        }
        Ok(Self {
            invariant,
            location,
        })
    }
}

impl DecorationAspect for SPIRVObject {
    type DecorationClass = DecorationClassObject;
    fn parse_decorations<I: FnOnce() -> spirv_parser::Instruction>(
        decorations: Vec<Self::DecorationClass>,
        _member_index: Option<u32>,
        _instruction: I,
    ) -> TranslationResult<Self> {
        let mut uniformity = Uniformity::Unknown;
        let mut signed_wrapping = Wrapping::Allowed;
        let mut unsigned_wrapping = Wrapping::Allowed;
        let mut fp_contraction_allowed = true;
        use DecorationClassObject::*;
        for decoration in decorations {
            match decoration {
                Uniform(_) => uniformity = Uniformity::Uniform,
                UniformId(DecorationUniformId {
                    execution: _execution,
                }) => {
                    // FIXME: currently ignored
                }
                NoContraction(_) => fp_contraction_allowed = false,
                NoSignedWrap(_) => signed_wrapping = Wrapping::UndefinedBehavior,
                NoUnsignedWrap(_) => unsigned_wrapping = Wrapping::UndefinedBehavior,
                NonUniform(_) => uniformity = Uniformity::NonUniform,
            }
        }
        Ok(Self {
            uniformity,
            signed_wrapping,
            unsigned_wrapping,
            fp_contraction_allowed,
        })
    }
}

pub(crate) trait GetDecorationAspect<A: DecorationAspect> {
    fn get_decoration_aspect_impl(&self) -> &A;
}

impl<A: DecorationAspect, T> GetDecorationAspect<A> for T
where
    Self: Deref,
    <Self as Deref>::Target: GetDecorationAspect<A>,
{
    fn get_decoration_aspect_impl(&self) -> &A {
        (**self).get_decoration_aspect_impl()
    }
}

pub(crate) trait GetDecorationAspectMut<A: DecorationAspect>:
    GetDecorationAspect<A>
{
    fn get_decoration_aspect_mut_impl(&mut self) -> &mut A;
}

impl<A: DecorationAspect, T> GetDecorationAspectMut<A> for T
where
    Self: DerefMut,
    <Self as Deref>::Target: GetDecorationAspectMut<A>,
{
    fn get_decoration_aspect_mut_impl(&mut self) -> &mut A {
        (**self).get_decoration_aspect_mut_impl()
    }
}

pub(crate) trait DecorationAspects {
    fn get_decoration_aspect<T: DecorationAspect>(&self) -> &T
    where
        Self: GetDecorationAspect<T>,
    {
        self.get_decoration_aspect_impl()
    }
    fn get_decoration_aspect_mut<T: DecorationAspect>(&mut self) -> &mut T
    where
        Self: GetDecorationAspectMut<T>,
    {
        self.get_decoration_aspect_mut_impl()
    }
}

impl<T> DecorationAspects for T {}
