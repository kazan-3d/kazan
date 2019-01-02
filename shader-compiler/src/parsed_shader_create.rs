// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2019 Jacob Lifshay

use crate::{
    ArrayType, BuiltInVariable, Constant, Context, FrontendType, IdKind, IdProperties, Ids,
    MemberDecoration, ParsedShader, ParsedShaderFunction, PointerType, ScalarConstant, ScalarType,
    ShaderEntryPoint, ShaderStageCreateInfo, StructId, StructMember, StructType, Undefable,
    UniformVariable, VectorConstant, VectorType,
};
use spirv_parser::{BuiltIn, Decoration, ExecutionModel, IdRef, Instruction, StorageClass};
use std::mem;
use std::rc::Rc;

#[allow(clippy::cyclomatic_complexity)]
pub(super) fn create<'a, C: shader_compiler_backend::Context<'a>>(
    context: &mut Context,
    stage_info: ShaderStageCreateInfo,
    execution_model: ExecutionModel,
) -> ParsedShader<'a, C> {
    let parser = spirv_parser::Parser::start(stage_info.code).unwrap();
    let header = *parser.header();
    assert_eq!(header.instruction_schema, 0);
    assert_eq!(header.version.0, 1);
    assert!(header.version.1 <= 3);
    let instructions: Vec<_> = parser.map(Result::unwrap).collect();
    println!("Parsing Shader:");
    print!("{}", header);
    for instruction in instructions.iter() {
        print!("{}", instruction);
    }
    let mut ids = Ids((0..header.bound)
        .map(|_| IdProperties {
            kind: IdKind::Undefined,
            decorations: Vec::new(),
            member_decorations: Vec::new(),
        })
        .collect());
    let mut entry_point = None;
    let mut current_function: Option<(IdRef, ParsedShaderFunction)> = None;
    let mut execution_modes = Vec::new();
    let mut workgroup_size = None;
    for instruction in instructions {
        match current_function {
            Some(mut function) => {
                current_function = match instruction {
                    instruction @ Instruction::FunctionEnd {} => {
                        function.1.instructions.push(instruction);
                        ids[function.0].set_kind(IdKind::Function(Some(function.1)));
                        None
                    }
                    instruction => {
                        function.1.instructions.push(instruction);
                        Some(function)
                    }
                };
                continue;
            }
            None => current_function = None,
        }
        match instruction {
            Instruction::Function {
                id_result_type,
                id_result,
                function_control,
                function_type,
            } => {
                ids[id_result.0].assert_no_member_decorations(id_result.0);
                let decorations = ids[id_result.0].decorations.clone();
                current_function = Some((
                    id_result.0,
                    ParsedShaderFunction {
                        instructions: vec![Instruction::Function {
                            id_result_type,
                            id_result,
                            function_control,
                            function_type,
                        }],
                        decorations,
                    },
                ));
            }
            Instruction::EntryPoint {
                execution_model: current_execution_model,
                entry_point: main_function_id,
                name,
                interface,
            } => {
                if execution_model == current_execution_model && name == stage_info.entry_point_name
                {
                    assert!(entry_point.is_none());
                    entry_point = Some(ShaderEntryPoint {
                        main_function_id,
                        interface_variables: interface.clone(),
                    });
                }
            }
            Instruction::ExecutionMode {
                entry_point: entry_point_id,
                mode,
            }
            | Instruction::ExecutionModeId {
                entry_point: entry_point_id,
                mode,
            } => {
                if entry_point_id == entry_point.as_ref().unwrap().main_function_id {
                    execution_modes.push(mode);
                }
            }
            Instruction::Decorate { target, decoration }
            | Instruction::DecorateId { target, decoration } => {
                ids[target].decorations.push(decoration);
            }
            Instruction::MemberDecorate {
                structure_type,
                member,
                decoration,
            } => {
                ids[structure_type]
                    .member_decorations
                    .push(MemberDecoration { member, decoration });
            }
            Instruction::DecorationGroup { id_result } => {
                ids[id_result.0].set_kind(IdKind::DecorationGroup);
            }
            Instruction::GroupDecorate {
                decoration_group,
                targets,
            } => {
                let decorations = ids[decoration_group].decorations.clone();
                for target in targets {
                    ids[target]
                        .decorations
                        .extend(decorations.iter().map(Clone::clone));
                }
            }
            Instruction::GroupMemberDecorate {
                decoration_group,
                targets,
            } => {
                let decorations = ids[decoration_group].decorations.clone();
                for target in targets {
                    ids[target.0]
                        .member_decorations
                        .extend(decorations.iter().map(|decoration| MemberDecoration {
                            member: target.1,
                            decoration: decoration.clone(),
                        }));
                }
            }
            Instruction::TypeFunction {
                id_result,
                return_type,
                parameter_types,
            } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                let kind = IdKind::FunctionType {
                    return_type: ids[return_type].get_type().map(Clone::clone),
                    arguments: parameter_types
                        .iter()
                        .map(|argument| ids[*argument].get_nonvoid_type().clone())
                        .collect(),
                };
                ids[id_result.0].set_kind(kind);
            }
            Instruction::TypeVoid { id_result } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                ids[id_result.0].set_kind(IdKind::VoidType);
            }
            Instruction::TypeBool { id_result } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                ids[id_result.0].set_kind(IdKind::Type(Rc::new(FrontendType::Scalar(
                    ScalarType::Bool,
                ))));
            }
            Instruction::TypeInt {
                id_result,
                width,
                signedness,
            } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                ids[id_result.0].set_kind(IdKind::Type(Rc::new(FrontendType::Scalar(
                    match (width, signedness != 0) {
                        (8, false) => ScalarType::U8,
                        (8, true) => ScalarType::I8,
                        (16, false) => ScalarType::U16,
                        (16, true) => ScalarType::I16,
                        (32, false) => ScalarType::U32,
                        (32, true) => ScalarType::I32,
                        (64, false) => ScalarType::U64,
                        (64, true) => ScalarType::I64,
                        (width, signedness) => unreachable!(
                            "unsupported int type: {}{}",
                            if signedness { "i" } else { "u" },
                            width
                        ),
                    },
                ))));
            }
            Instruction::TypeFloat { id_result, width } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                ids[id_result.0].set_kind(IdKind::Type(Rc::new(FrontendType::Scalar(
                    match width {
                        16 => ScalarType::F16,
                        32 => ScalarType::F32,
                        64 => ScalarType::F64,
                        _ => unreachable!("unsupported float type: f{}", width),
                    },
                ))));
            }
            Instruction::TypeVector {
                id_result,
                component_type,
                component_count,
            } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                let element = ids[component_type].get_nonvoid_type().get_scalar().clone();
                ids[id_result.0].set_kind(IdKind::Type(Rc::new(FrontendType::Vector(
                    VectorType {
                        element,
                        element_count: component_count as usize,
                    },
                ))));
            }
            Instruction::TypeForwardPointer { pointer_type, .. } => {
                ids[pointer_type].set_kind(IdKind::ForwardPointer(Rc::new(FrontendType::Scalar(
                    ScalarType::Pointer(PointerType::unresolved()),
                ))));
            }
            Instruction::TypePointer {
                id_result,
                type_: pointee,
                ..
            } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                let pointee = ids[pointee].get_type().map(Clone::clone);
                let pointer = match mem::replace(&mut ids[id_result.0].kind, IdKind::Undefined) {
                    IdKind::Undefined => Rc::new(FrontendType::Scalar(ScalarType::Pointer(
                        PointerType::new(context, pointee),
                    ))),
                    IdKind::ForwardPointer(pointer) => {
                        if let FrontendType::Scalar(ScalarType::Pointer(pointer)) = &*pointer {
                            pointer.resolve(context, pointee);
                        } else {
                            unreachable!();
                        }
                        pointer
                    }
                    _ => unreachable!("duplicate id"),
                };
                ids[id_result.0].set_kind(IdKind::Type(pointer));
            }
            Instruction::TypeStruct {
                id_result,
                member_types,
            } => {
                let decorations = ids[id_result.0].decorations.clone();
                let struct_type = {
                    let mut members: Vec<_> = member_types
                        .into_iter()
                        .map(|member_type| StructMember {
                            decorations: Vec::new(),
                            member_type: match ids[member_type].kind {
                                IdKind::Type(ref t) => t.clone(),
                                IdKind::ForwardPointer(ref t) => t.clone(),
                                _ => unreachable!("invalid struct member type"),
                            },
                        })
                        .collect();
                    for member_decoration in &ids[id_result.0].member_decorations {
                        members[member_decoration.member as usize]
                            .decorations
                            .push(member_decoration.decoration.clone());
                    }
                    StructType {
                        id: StructId::new(context),
                        decorations,
                        members,
                    }
                };
                ids[id_result.0].set_kind(IdKind::Type(Rc::new(FrontendType::Struct(struct_type))));
            }
            Instruction::TypeRuntimeArray {
                id_result,
                element_type,
            } => {
                ids[id_result.0].assert_no_member_decorations(id_result.0);
                let decorations = ids[id_result.0].decorations.clone();
                let element = ids[element_type].get_nonvoid_type().clone();
                ids[id_result.0].set_kind(IdKind::Type(Rc::new(FrontendType::Array(ArrayType {
                    decorations,
                    element,
                    element_count: None,
                }))));
            }
            Instruction::Variable {
                id_result_type,
                id_result,
                storage_class,
                initializer,
            } => {
                ids[id_result.0].assert_no_member_decorations(id_result.0);
                if let Some(built_in) =
                    ids[id_result.0]
                        .decorations
                        .iter()
                        .find_map(|decoration| match *decoration {
                            Decoration::BuiltIn { built_in } => Some(built_in),
                            _ => None,
                        })
                {
                    let built_in_variable = match built_in {
                        BuiltIn::GlobalInvocationId => {
                            for decoration in &ids[id_result.0].decorations {
                                match decoration {
                                    Decoration::BuiltIn { .. } => {}
                                    _ => unimplemented!(
                                        "unimplemented decoration on {:?}: {:?}",
                                        built_in,
                                        decoration
                                    ),
                                }
                            }
                            assert!(initializer.is_none());
                            BuiltInVariable { built_in }
                        }
                        _ => unimplemented!("unimplemented built-in: {:?}", built_in),
                    };
                    assert_eq!(
                        built_in_variable.get_type(context),
                        ids[id_result_type.0]
                            .get_nonvoid_type()
                            .get_nonvoid_pointee()
                    );
                    ids[id_result.0].set_kind(IdKind::BuiltInVariable(built_in_variable));
                } else {
                    let variable_type = ids[id_result_type.0].get_nonvoid_type().clone();
                    match storage_class {
                        StorageClass::Uniform => {
                            let mut descriptor_set = None;
                            let mut binding = None;
                            for decoration in &ids[id_result.0].decorations {
                                match *decoration {
                                    Decoration::DescriptorSet { descriptor_set: v } => {
                                        assert!(
                                            descriptor_set.is_none(),
                                            "duplicate DescriptorSet decoration"
                                        );
                                        descriptor_set = Some(v);
                                    }
                                    Decoration::Binding { binding_point: v } => {
                                        assert!(binding.is_none(), "duplicate Binding decoration");
                                        binding = Some(v);
                                    }
                                    _ => unimplemented!(
                                        "unimplemented decoration on uniform variable: {:?}",
                                        decoration
                                    ),
                                }
                            }
                            let descriptor_set = descriptor_set
                                .expect("uniform variable is missing DescriptorSet decoration");
                            let binding =
                                binding.expect("uniform variable is missing Binding decoration");
                            assert!(initializer.is_none());
                            ids[id_result.0].set_kind(IdKind::UniformVariable(UniformVariable {
                                binding,
                                descriptor_set,
                                variable_type,
                            }));
                        }
                        StorageClass::Input => unimplemented!(),
                        _ => unimplemented!(
                            "unimplemented OpVariable StorageClass: {:?}",
                            storage_class
                        ),
                    }
                }
            }
            Instruction::Constant32 {
                id_result_type,
                id_result,
                value,
            } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                #[allow(clippy::cast_lossless)]
                let constant = match **ids[id_result_type.0].get_nonvoid_type() {
                    FrontendType::Scalar(ScalarType::U8) => {
                        let converted_value = value as u8;
                        assert_eq!(converted_value as u32, value);
                        Constant::Scalar(ScalarConstant::U8(Undefable::Defined(converted_value)))
                    }
                    FrontendType::Scalar(ScalarType::U16) => {
                        let converted_value = value as u16;
                        assert_eq!(converted_value as u32, value);
                        Constant::Scalar(ScalarConstant::U16(Undefable::Defined(converted_value)))
                    }
                    FrontendType::Scalar(ScalarType::U32) => {
                        Constant::Scalar(ScalarConstant::U32(Undefable::Defined(value)))
                    }
                    FrontendType::Scalar(ScalarType::I8) => {
                        let converted_value = value as i8;
                        assert_eq!(converted_value as u32, value);
                        Constant::Scalar(ScalarConstant::I8(Undefable::Defined(converted_value)))
                    }
                    FrontendType::Scalar(ScalarType::I16) => {
                        let converted_value = value as i16;
                        assert_eq!(converted_value as u32, value);
                        Constant::Scalar(ScalarConstant::I16(Undefable::Defined(converted_value)))
                    }
                    FrontendType::Scalar(ScalarType::I32) => {
                        Constant::Scalar(ScalarConstant::I32(Undefable::Defined(value as i32)))
                    }
                    FrontendType::Scalar(ScalarType::F16) => {
                        let converted_value = value as u16;
                        assert_eq!(converted_value as u32, value);
                        Constant::Scalar(ScalarConstant::F16(Undefable::Defined(converted_value)))
                    }
                    FrontendType::Scalar(ScalarType::F32) => Constant::Scalar(ScalarConstant::F32(
                        Undefable::Defined(f32::from_bits(value)),
                    )),
                    _ => unreachable!("invalid type"),
                };
                ids[id_result.0].set_kind(IdKind::Constant(Rc::new(constant)));
            }
            Instruction::Constant64 {
                id_result_type,
                id_result,
                value,
            } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                let constant = match **ids[id_result_type.0].get_nonvoid_type() {
                    FrontendType::Scalar(ScalarType::U64) => {
                        Constant::Scalar(ScalarConstant::U64(Undefable::Defined(value)))
                    }
                    FrontendType::Scalar(ScalarType::I64) => {
                        Constant::Scalar(ScalarConstant::I64(Undefable::Defined(value as i64)))
                    }
                    FrontendType::Scalar(ScalarType::F64) => Constant::Scalar(ScalarConstant::F64(
                        Undefable::Defined(f64::from_bits(value)),
                    )),
                    _ => unreachable!("invalid type"),
                };
                ids[id_result.0].set_kind(IdKind::Constant(Rc::new(constant)));
            }
            Instruction::ConstantFalse {
                id_result_type,
                id_result,
            } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                let constant = match **ids[id_result_type.0].get_nonvoid_type() {
                    FrontendType::Scalar(ScalarType::Bool) => {
                        Constant::Scalar(ScalarConstant::Bool(Undefable::Defined(false)))
                    }
                    _ => unreachable!("invalid type"),
                };
                ids[id_result.0].set_kind(IdKind::Constant(Rc::new(constant)));
            }
            Instruction::ConstantTrue {
                id_result_type,
                id_result,
            } => {
                ids[id_result.0].assert_no_decorations(id_result.0);
                let constant = match **ids[id_result_type.0].get_nonvoid_type() {
                    FrontendType::Scalar(ScalarType::Bool) => {
                        Constant::Scalar(ScalarConstant::Bool(Undefable::Defined(true)))
                    }
                    _ => unreachable!("invalid type"),
                };
                ids[id_result.0].set_kind(IdKind::Constant(Rc::new(constant)));
            }
            Instruction::ConstantComposite {
                id_result_type,
                id_result,
                constituents,
            } => {
                let constant = match **ids[id_result_type.0].get_nonvoid_type() {
                    FrontendType::Vector(VectorType {
                        ref element,
                        element_count,
                    }) => {
                        assert_eq!(element_count, constituents.len());
                        let constituents = constituents
                            .iter()
                            .map(|id| *ids[*id].get_constant().get_scalar());
                        match *element {
                            ScalarType::U8 => {
                                VectorConstant::U8(constituents.map(|v| v.get_u8()).collect())
                            }
                            ScalarType::U16 => {
                                VectorConstant::U16(constituents.map(|v| v.get_u16()).collect())
                            }
                            ScalarType::U32 => {
                                VectorConstant::U32(constituents.map(|v| v.get_u32()).collect())
                            }
                            ScalarType::U64 => {
                                VectorConstant::U64(constituents.map(|v| v.get_u64()).collect())
                            }
                            ScalarType::I8 => {
                                VectorConstant::I8(constituents.map(|v| v.get_i8()).collect())
                            }
                            ScalarType::I16 => {
                                VectorConstant::I16(constituents.map(|v| v.get_i16()).collect())
                            }
                            ScalarType::I32 => {
                                VectorConstant::I32(constituents.map(|v| v.get_i32()).collect())
                            }
                            ScalarType::I64 => {
                                VectorConstant::I64(constituents.map(|v| v.get_i64()).collect())
                            }
                            ScalarType::F16 => {
                                VectorConstant::F16(constituents.map(|v| v.get_f16()).collect())
                            }
                            ScalarType::F32 => {
                                VectorConstant::F32(constituents.map(|v| v.get_f32()).collect())
                            }
                            ScalarType::F64 => {
                                VectorConstant::F64(constituents.map(|v| v.get_f64()).collect())
                            }
                            ScalarType::Bool => {
                                VectorConstant::Bool(constituents.map(|v| v.get_bool()).collect())
                            }
                            ScalarType::Pointer(_) => unimplemented!(),
                        }
                    }
                    _ => unimplemented!(),
                };
                for decoration in &ids[id_result.0].decorations {
                    match decoration {
                        Decoration::BuiltIn {
                            built_in: BuiltIn::WorkgroupSize,
                        } => {
                            assert!(
                                workgroup_size.is_none(),
                                "duplicate WorkgroupSize decorations"
                            );
                            workgroup_size = match constant {
                                VectorConstant::U32(ref v) => {
                                    assert_eq!(
                                        v.len(),
                                        3,
                                        "invalid type for WorkgroupSize built-in"
                                    );
                                    Some((v[0].unwrap(), v[1].unwrap(), v[2].unwrap()))
                                }
                                _ => unreachable!("invalid type for WorkgroupSize built-in"),
                            };
                        }
                        _ => unimplemented!(
                            "unimplemented decoration on constant {:?}: {:?}",
                            Constant::Vector(constant),
                            decoration
                        ),
                    }
                }
                ids[id_result.0].assert_no_member_decorations(id_result.0);
                ids[id_result.0].set_kind(IdKind::Constant(Rc::new(Constant::Vector(constant))));
            }
            Instruction::MemoryModel {
                addressing_model,
                memory_model,
            } => {
                assert_eq!(addressing_model, spirv_parser::AddressingModel::Logical);
                assert_eq!(memory_model, spirv_parser::MemoryModel::GLSL450);
            }
            Instruction::Capability { .. }
            | Instruction::ExtInstImport { .. }
            | Instruction::Source { .. }
            | Instruction::SourceExtension { .. }
            | Instruction::Name { .. }
            | Instruction::MemberName { .. } => {}
            Instruction::SpecConstant32 { .. } => unimplemented!(),
            Instruction::SpecConstant64 { .. } => unimplemented!(),
            Instruction::SpecConstantTrue { .. } => unimplemented!(),
            Instruction::SpecConstantFalse { .. } => unimplemented!(),
            Instruction::SpecConstantOp { .. } => unimplemented!(),
            instruction => unimplemented!("unimplemented instruction:\n{}", instruction),
        }
    }
    assert!(
        current_function.is_none(),
        "missing terminating OpFunctionEnd"
    );
    let ShaderEntryPoint {
        main_function_id,
        interface_variables,
    } = entry_point.unwrap();
    ParsedShader {
        ids,
        main_function_id,
        interface_variables,
        execution_modes,
        workgroup_size,
    }
}
