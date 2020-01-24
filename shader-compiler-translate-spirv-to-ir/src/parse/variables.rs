// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::errors::decoration_not_allowed;
use crate::errors::DecorationNotAllowedOnInstruction;
use crate::errors::TranslationResult;
use crate::parse::annotations::DecorationClass;
use crate::parse::annotations::DecorationClassMemoryObjectDeclaration;
use crate::parse::annotations::DecorationClassMemoryObjectDeclarationOrStructMember;
use crate::parse::annotations::DecorationClassObject;
use crate::parse::annotations::DecorationClassVariableOrStructMember;
use crate::parse::ParseInstruction;
use crate::parse::TranslationStateParsingTypesConstantsAndGlobals;
use crate::TranslationStateBase;
use alloc::vec::Vec;
use spirv_parser::DecorationComponent;
use spirv_parser::DecorationLocation;
use spirv_parser::DecorationUniformId;
use spirv_parser::OpVariable;

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

impl MemoryObjectDeclarationOrStructMember {
    pub(crate) fn parse_decorations<I: FnOnce() -> spirv_parser::Instruction>(
        decorations: Vec<DecorationClassMemoryObjectDeclarationOrStructMember>,
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

#[derive(Debug)]
pub(crate) struct VariableOrStructMember {
    pub(crate) invariant: bool,
    pub(crate) location: Option<u32>,
}

impl VariableOrStructMember {
    pub(crate) fn parse_decorations<I: FnOnce() -> spirv_parser::Instruction>(
        decorations: Vec<DecorationClassVariableOrStructMember>,
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

#[derive(Debug)]
pub(crate) struct MemoryObjectDeclaration {
    pub(crate) memory_object_declaration_or_struct_member: MemoryObjectDeclarationOrStructMember,
    pub(crate) restrict: bool,
    pub(crate) aliased: bool,
}

impl MemoryObjectDeclaration {
    pub(crate) fn parse_decorations<I: FnOnce() -> spirv_parser::Instruction>(
        memory_object_declaration_or_struct_member: MemoryObjectDeclarationOrStructMember,
        decorations: Vec<DecorationClassMemoryObjectDeclaration>,
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
        Ok(Self {
            memory_object_declaration_or_struct_member,
            restrict,
            aliased,
        })
    }
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

#[derive(Debug)]
pub(crate) struct SPIRVObject {
    pub(crate) uniformity: Uniformity,
    pub(crate) signed_wrapping: Wrapping,
    pub(crate) unsigned_wrapping: Wrapping,
    pub(crate) fp_contraction_allowed: bool,
}

impl SPIRVObject {
    pub(crate) fn parse_decorations<I: FnOnce() -> spirv_parser::Instruction>(
        decorations: Vec<DecorationClassObject>,
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

#[derive(Debug)]
pub(crate) struct SPIRVVariable {}

impl ParseInstruction for OpVariable {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let OpVariable {
            id_result_type,
            id_result,
            storage_class,
            initializer,
        } = *self;
        let result_type = state.get_type(id_result_type.0)?.clone();
        let mut memory_object_declaration_or_struct_member_decorations = Vec::new();
        let mut memory_object_declaration_decorations = Vec::new();
        let mut variable_or_struct_member_decorations = Vec::new();
        let mut object_decorations = Vec::new();
        for decoration in state.take_decorations(id_result)? {
            match decoration {
                DecorationClass::Ignored(_) => {}
                DecorationClass::Invalid(_) => {
                    return Err(DecorationNotAllowedOnInstruction {
                        decoration: decoration.into(),
                        instruction: self.clone().into(),
                    }
                    .into());
                }
                DecorationClass::MemoryObjectDeclaration(v) => {
                    memory_object_declaration_decorations.push(v);
                }
                DecorationClass::MemoryObjectDeclarationOrStructMember(v) => {
                    memory_object_declaration_or_struct_member_decorations.push(v);
                }
                DecorationClass::Variable(_) => todo!(),
                DecorationClass::VariableOrStructMember(v) => {
                    variable_or_struct_member_decorations.push(v);
                }
                DecorationClass::Misc(_) => todo!(),
                DecorationClass::Object(v) => {
                    object_decorations.push(v);
                }
                DecorationClass::Struct(_) | DecorationClass::StructMember(_) => {
                    return Err(DecorationNotAllowedOnInstruction {
                        decoration: decoration.into(),
                        instruction: self.clone().into(),
                    }
                    .into());
                }
            }
        }
        let memory_object_declaration_or_struct_member =
            MemoryObjectDeclarationOrStructMember::parse_decorations(
                memory_object_declaration_or_struct_member_decorations,
                None,
                || self.clone().into(),
            )?;
        let memory_object_declaration = MemoryObjectDeclaration::parse_decorations(
            memory_object_declaration_or_struct_member,
            memory_object_declaration_decorations,
            None,
            || self.clone().into(),
        )?;
        let variable_or_struct_member = VariableOrStructMember::parse_decorations(
            variable_or_struct_member_decorations,
            None,
            || self.clone().into(),
        )?;
        let object =
            SPIRVObject::parse_decorations(object_decorations, None, || self.clone().into())?;
        todo!()
    }
    fn parse_in_function_body<'g, 'i>(
        &'i self,
        _state: &mut TranslationStateBase<'g, 'i>,
    ) -> TranslationResult<()> {
        todo!()
    }
}
