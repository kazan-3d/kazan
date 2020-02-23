// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    cfg::CFGBlockId,
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
        functions::TranslationStateParsingFunctionBody,
        translate_structure_tree::TranslationStateParsingFunctionBodyBlock, ParseInstruction,
        TranslationStateParseBaseTypesConstantsAndGlobals,
        TranslationStateParsingTypesConstantsAndGlobals,
    },
    types::{structs::StructKind, GenericSPIRVType, PointerType, PointerTypeData},
    values::{SPIRVVariable, SPIRVVariableData},
};
use alloc::vec::Vec;
use once_cell::unsync::OnceCell;
use spirv_parser::{
    BuiltIn, DecorationBinding, DecorationBuiltIn, DecorationDescriptorSet, DecorationIndex,
    DecorationInputAttachmentIndex, IdResult, IdResultType, OpVariable, StorageClass,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum VariableScope {
    Global,
    Function,
}

struct ParsedVariable<'g> {
    id_result: IdResult,
    id_result_type: IdResultType,
    storage_class: StorageClass,
    initializer: Option<spirv_parser::IdRef>,
    result_type: PointerType<'g>,
    blend_equation_input_index: Option<u32>,
    binding_point: Option<u32>,
    descriptor_set: Option<u32>,
    input_attachment_index: Option<u32>,
    built_in: Option<BuiltIn>,
    memory_object_declaration_or_struct_member: MemoryObjectDeclarationOrStructMember,
    memory_object_declaration: MemoryObjectDeclaration,
    variable_or_struct_member: VariableOrStructMember,
    object: SPIRVObject,
}

fn parse_variable<'g, 'i>(
    instruction: &'i OpVariable,
    state: &mut TranslationStateParseBaseTypesConstantsAndGlobals<'g, 'i>,
    variable_scope: VariableScope,
) -> TranslationResult<ParsedVariable<'g>> {
    let OpVariable {
        id_result_type,
        id_result,
        storage_class,
        initializer,
    } = *instruction;
    let mut result_type = state
        .get_type(id_result_type.0)?
        .pointer()
        .ok_or_else(|| VariableResultTypeMustBePointer {
            instruction: instruction.clone().into(),
        })?
        .clone();
    result_type
        .get()
        .ok_or_else(|| VariableResultTypeMustBePointer {
            instruction: instruction.clone().into(),
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
                    instruction: instruction.clone().into(),
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
                DecorationClassVariable::InputAttachmentIndex(DecorationInputAttachmentIndex {
                    attachment_index,
                }) => input_attachment_index = Some(attachment_index),
                // PhysicalStorageBufferAddresses
                DecorationClassVariable::RestrictPointer(_)
                | DecorationClassVariable::AliasedPointer(_) => {
                    return Err(DecorationNotAllowedOnInstruction {
                        decoration: decoration.into(),
                        instruction: instruction.clone().into(),
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
                    pointee_type_id,
                    storage_class,
                    array_stride,
                } = *result_type.get().expect("known to be Some");
                let pointee_type = pointee_type.get_relaxed_precision_type().ok_or_else(|| {
                    RelaxedPrecisionDecorationNotAllowed {
                        instruction: instruction.clone().into(),
                    }
                })?;
                result_type = PointerType::new(
                    result_type.id(),
                    PointerTypeData {
                        pointee_type,
                        pointee_type_id,
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
                    instruction: instruction.clone().into(),
                }
                .into());
            }
        }
    }
    let memory_object_declaration_or_struct_member =
        MemoryObjectDeclarationOrStructMember::parse_decorations(
            memory_object_declaration_or_struct_member_decorations,
            None,
            || instruction.clone().into(),
        )?;
    let memory_object_declaration = MemoryObjectDeclaration::parse_decorations(
        memory_object_declaration_decorations,
        None,
        || instruction.clone().into(),
    )?;
    let variable_or_struct_member = VariableOrStructMember::parse_decorations(
        variable_or_struct_member_decorations,
        None,
        || instruction.clone().into(),
    )?;
    let object =
        SPIRVObject::parse_decorations(object_decorations, None, || instruction.clone().into())?;
    Ok(ParsedVariable {
        id_result,
        id_result_type,
        storage_class,
        initializer,
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
    })
}

impl ParseInstruction for OpVariable {
    fn parse_in_types_constants_globals_section<'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingTypesConstantsAndGlobals<'g, 'i>,
    ) -> TranslationResult<()> {
        let ParsedVariable {
            id_result,
            id_result_type,
            storage_class,
            initializer,
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
        } = parse_variable(self, state, VariableScope::Global)?;
        let ir_value = if state.entry_point_interface_variables.contains(&id_result.0) {
            match storage_class {
                StorageClass::UniformConstant(_) => todo!(),
                StorageClass::Uniform(_) => todo!(),
                StorageClass::Input(_) | StorageClass::Output(_) => {
                    if let Some(built_in) = built_in {
                        todo!();
                    } else {
                        match result_type
                            .get()
                            .expect("known to be Some")
                            .pointee_type
                            .struct_type()
                        {
                            Some(struct_type) if struct_type.kind == StructKind::BuiltIns => {
                                todo!();
                            }
                            _ => {
                                todo!();
                            }
                        }
                    }
                }
                StorageClass::Workgroup(_) => todo!(),
                StorageClass::CrossWorkgroup(_) => todo!(),
                StorageClass::Private(_) => todo!(),
                StorageClass::Function(_) => todo!(),
                StorageClass::Generic(_) => todo!(),
                StorageClass::PushConstant(_) => todo!(),
                StorageClass::AtomicCounter(_) => todo!(),
                StorageClass::Image(_) => todo!(),
                StorageClass::StorageBuffer(_) => todo!(),
                StorageClass::PhysicalStorageBuffer(_) => todo!(),
            }
        } else {
            shader_compiler_ir::ValueUse::from_const(
                shader_compiler_ir::DataPointerType.null(),
                state.get_or_make_debug_name(id_result.0)?,
                state.global_state,
            )
        };
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
                ir_value,
            }),
        )
    }
    fn parse_in_function_body_prepass<'f, 'g, 'i>(
        &'i self,
        _state: &mut TranslationStateParsingFunctionBody<'f, 'g, 'i>,
        _block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        Ok(())
    }
    fn parse_in_function_body_reachable<'b, 'f, 'g, 'i>(
        &'i self,
        state: &mut TranslationStateParsingFunctionBodyBlock<'b, 'f, 'g, 'i>,
        block_id: CFGBlockId,
    ) -> TranslationResult<()> {
        let ParsedVariable {
            id_result,
            id_result_type,
            storage_class,
            initializer,
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
        } = parse_variable(self, state, VariableScope::Global)?;
        let result_type_pointee = result_type.get().expect("known to be Some");
        let variable_type = result_type_pointee.pointee_type.get_nonvoid_ir_type(
            state.global_state,
            result_type_pointee.pointee_type_id,
            || self.clone().into(),
        )?;
        let alignment = result_type_pointee.pointee_type.get_alignment(
            state.target_properties,
            state.global_state,
            result_type_pointee.pointee_type_id,
            || self.clone().into(),
        )?;
        let variable = shader_compiler_ir::Variable {
            variable_type,
            alignment,
            pointer: shader_compiler_ir::ValueDefinition::new(
                shader_compiler_ir::DataPointerType,
                state.get_or_make_debug_name(id_result.0)?,
                state.global_state,
            ),
        };
        let ir_value = shader_compiler_ir::ValueUse::new(variable.pointer.value());
        state.local_variables.push(variable);
        if let Some(initializer) = initializer {
            todo!()
        }
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
                ir_value,
            }),
        )
    }
}
