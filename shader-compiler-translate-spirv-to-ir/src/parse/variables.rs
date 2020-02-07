// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::cfg::CFGBlockId;
use crate::{
    decorations::{
        DecorationAspect, DecorationClass, DecorationClassMisc, DecorationClassVariable,
        MemoryObjectDeclaration, MemoryObjectDeclarationOrStructMember, SPIRVObject,
        VariableOrStructMember,
    },
    errors::{
        DecorationNotAllowedOnInstruction, RelaxedPrecisionDecorationNotAllowed, TranslationResult,
        VariableResultTypeMustBePointer,
    },
    parse::{
        functions::TranslationStateParsingFunctionBody, ParseInstruction,
        TranslationStateParsingTypesConstantsAndGlobals,
    },
    types::{GenericSPIRVType, PointerType, PointerTypeData},
    values::{SPIRVVariable, SPIRVVariableData},
};
use alloc::vec::Vec;
use spirv_parser::{
    DecorationBinding, DecorationBuiltIn, DecorationDescriptorSet, DecorationIndex,
    DecorationInputAttachmentIndex, OpVariable,
};

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
        let mut result_type = state
            .get_type(id_result_type.0)?
            .pointer()
            .ok_or_else(|| VariableResultTypeMustBePointer {
                instruction: self.clone().into(),
            })?
            .clone();
        result_type
            .get()
            .ok_or_else(|| VariableResultTypeMustBePointer {
                instruction: self.clone().into(),
            })?;
        let mut memory_object_declaration_or_struct_member_decorations = Vec::new();
        let mut memory_object_declaration_decorations = Vec::new();
        let mut variable_or_struct_member_decorations = Vec::new();
        let mut object_decorations = Vec::new();
        let mut blend_equation_input_index = None;
        let mut binding_point = None;
        let mut descriptor_set = None;
        let mut input_attachment_index = None;
        let mut built_in = None;
        for decoration in state.take_decorations(id_result)? {
            match decoration {
                DecorationClass::Ignored(_) => {}
                DecorationClass::Invalid(_)
                | DecorationClass::Misc(DecorationClassMisc::SpecId(_))
                | DecorationClass::Misc(DecorationClassMisc::FPRoundingMode(_))
                | DecorationClass::Misc(DecorationClassMisc::ArrayStride(_)) => {
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
                DecorationClass::Variable(decoration) => match decoration {
                    DecorationClassVariable::Index(DecorationIndex { index }) => {
                        blend_equation_input_index = Some(index)
                    }
                    DecorationClassVariable::Binding(DecorationBinding { binding_point: v }) => {
                        binding_point = Some(v)
                    }
                    DecorationClassVariable::DescriptorSet(DecorationDescriptorSet {
                        descriptor_set: v,
                    }) => descriptor_set = Some(v),
                    DecorationClassVariable::InputAttachmentIndex(
                        DecorationInputAttachmentIndex { attachment_index },
                    ) => input_attachment_index = Some(attachment_index),
                    // PhysicalStorageBufferAddresses
                    DecorationClassVariable::RestrictPointer(_)
                    | DecorationClassVariable::AliasedPointer(_) => {
                        return Err(DecorationNotAllowedOnInstruction {
                            decoration: decoration.into(),
                            instruction: self.clone().into(),
                        }
                        .into());
                    }
                },
                DecorationClass::VariableOrStructMember(v) => {
                    variable_or_struct_member_decorations.push(v);
                }
                DecorationClass::RelaxedPrecision(_) => {
                    let PointerTypeData {
                        ref pointee_type,
                        storage_class,
                        array_stride,
                    } = *result_type.get().expect("known to be Some");
                    let pointee_type =
                        pointee_type.get_relaxed_precision_type().ok_or_else(|| {
                            RelaxedPrecisionDecorationNotAllowed {
                                instruction: self.clone().into(),
                            }
                        })?;
                    result_type = PointerType::new(
                        result_type.id(),
                        PointerTypeData {
                            pointee_type,
                            storage_class,
                            array_stride,
                        },
                    );
                }
                DecorationClass::Misc(DecorationClassMisc::BuiltIn(DecorationBuiltIn {
                    built_in: v,
                })) => built_in = Some(v),
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
        state.define_value(
            id_result,
            SPIRVVariable::new(SPIRVVariableData {
                result_type,
                blend_equation_input_index,
                binding_point,
                descriptor_set,
                input_attachment_index,
                built_in,
                memory_object_declaration_or_struct_member,
                memory_object_declaration,
                variable_or_struct_member,
                object,
                storage_class,
                initializer,
            }),
        )
    }
    fn parse_in_function_body_generic<'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        todo!()
    }
}
