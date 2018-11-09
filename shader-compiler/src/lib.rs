// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

#[macro_use]
extern crate shader_compiler_backend;
extern crate spirv_parser;

use spirv_parser::{
    BuiltIn, Decoration, ExecutionMode, ExecutionModel, IdRef, Instruction, StorageClass,
};
use std::error;
use std::fmt;
use std::mem;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

#[derive(Default)]
pub struct Context {
    types: pointer_type::ContextTypes,
}

mod pointer_type {
    use super::{Context, Type};
    use std::cell::RefCell;
    use std::hash::{Hash, Hasher};
    use std::rc::{Rc, Weak};

    #[derive(Default)]
    pub struct ContextTypes(Vec<Rc<Type>>);

    #[derive(Clone, Debug)]
    enum PointerTypeState {
        Void,
        Normal(Weak<Type>),
        Unresolved,
    }

    #[derive(Clone, Debug)]
    pub struct PointerType {
        pointee: RefCell<PointerTypeState>,
    }

    impl PointerType {
        pub fn new(context: &mut Context, pointee: Option<Rc<Type>>) -> Self {
            Self {
                pointee: RefCell::new(match pointee {
                    Some(pointee) => {
                        let weak = Rc::downgrade(&pointee);
                        context.types.0.push(pointee);
                        PointerTypeState::Normal(weak)
                    }
                    None => PointerTypeState::Void,
                }),
            }
        }
        pub fn new_void() -> Self {
            Self {
                pointee: RefCell::new(PointerTypeState::Void),
            }
        }
        pub fn unresolved() -> Self {
            Self {
                pointee: RefCell::new(PointerTypeState::Unresolved),
            }
        }
        pub fn resolve(&self, context: &mut Context, new_pointee: Option<Rc<Type>>) {
            let mut pointee = self.pointee.borrow_mut();
            match &*pointee {
                PointerTypeState::Unresolved => {}
                _ => unreachable!("pointer already resolved"),
            }
            *pointee = Self::new(context, new_pointee).pointee.into_inner();
        }
        pub fn pointee(&self) -> Option<Rc<Type>> {
            match *self.pointee.borrow() {
                PointerTypeState::Normal(ref pointee) => Some(
                    pointee
                        .upgrade()
                        .expect("PointerType is not valid after the associated Context is dropped"),
                ),
                PointerTypeState::Void => None,
                PointerTypeState::Unresolved => {
                    unreachable!("pointee() called on unresolved pointer")
                }
            }
        }
    }

    impl PartialEq for PointerType {
        fn eq(&self, rhs: &Self) -> bool {
            self.pointee() == rhs.pointee()
        }
    }

    impl Eq for PointerType {}

    impl Hash for PointerType {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            self.pointee().hash(hasher);
        }
    }
}

pub use pointer_type::PointerType;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ScalarType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F16,
    F32,
    F64,
    Bool,
    Pointer(PointerType),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Type {
    Scalar(ScalarType),
    Vector {
        element: ScalarType,
        element_count: usize,
    },
}

#[derive(Debug)]
pub struct NotAPointer;

impl fmt::Display for NotAPointer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "not a pointer")
    }
}

impl error::Error for NotAPointer {}

impl Type {
    pub fn is_pointer(&self) -> bool {
        if let Type::Scalar(ScalarType::Pointer(_)) = self {
            true
        } else {
            false
        }
    }
    pub fn get_pointee(&self) -> Result<Option<Rc<Type>>, NotAPointer> {
        if let Type::Scalar(ScalarType::Pointer(pointer)) = self {
            Ok(pointer.pointee())
        } else {
            Err(NotAPointer)
        }
    }
    pub fn get_nonvoid_pointee(&self) -> Rc<Type> {
        self.get_pointee()
            .unwrap()
            .expect("void is not allowed here")
    }
}

#[derive(Clone, Debug)]
pub enum Constant {
    Undef(Rc<Type>),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F16(u16),
    F32(f32),
    F64(f64),
    Bool(bool),
}

impl Constant {
    pub fn get_type(&self) -> &Type {
        match self {
            Constant::Undef(t) => &*t,
            Constant::U8(_) => &Type::Scalar(ScalarType::U8),
            Constant::U16(_) => &Type::Scalar(ScalarType::U16),
            Constant::U32(_) => &Type::Scalar(ScalarType::U32),
            Constant::U64(_) => &Type::Scalar(ScalarType::U64),
            Constant::I8(_) => &Type::Scalar(ScalarType::I8),
            Constant::I16(_) => &Type::Scalar(ScalarType::I16),
            Constant::I32(_) => &Type::Scalar(ScalarType::I32),
            Constant::I64(_) => &Type::Scalar(ScalarType::I64),
            Constant::F16(_) => &Type::Scalar(ScalarType::F16),
            Constant::F32(_) => &Type::Scalar(ScalarType::F32),
            Constant::F64(_) => &Type::Scalar(ScalarType::F64),
            Constant::Bool(_) => &Type::Scalar(ScalarType::Bool),
        }
    }
}

#[derive(Debug, Clone)]
struct MemberDecoration {
    member: u32,
    decoration: Decoration,
}

#[derive(Debug, Clone)]
struct BuiltInVariable {
    built_in: BuiltIn,
}

impl BuiltInVariable {
    fn get_type(&self, _context: &mut Context) -> Rc<Type> {
        match self.built_in {
            BuiltIn::GlobalInvocationId => Rc::new(Type::Vector {
                element: ScalarType::U32,
                element_count: 3,
            }),
            _ => unreachable!("unknown built-in"),
        }
    }
}

#[derive(Debug)]
enum IdKind {
    Undefined,
    DecorationGroup,
    Type(Rc<Type>),
    VoidType,
    FunctionType {
        return_type: Option<Rc<Type>>,
        arguments: Vec<Rc<Type>>,
    },
    ForwardPointer(Rc<Type>),
    BuiltInVariable(BuiltInVariable),
    Constant(Constant),
}

#[derive(Debug)]
struct IdProperties {
    kind: IdKind,
    decorations: Vec<Decoration>,
    member_decorations: Vec<MemberDecoration>,
}

impl IdProperties {
    fn set_kind(&mut self, kind: IdKind) {
        match &self.kind {
            IdKind::Undefined => {}
            _ => unreachable!("duplicate id"),
        }
        self.kind = kind;
    }
    fn get_type(&self) -> Option<Rc<Type>> {
        match &self.kind {
            IdKind::Type(t) => Some(t.clone()),
            IdKind::VoidType => None,
            _ => unreachable!("id is not type"),
        }
    }
    fn get_nonvoid_type(&self) -> Rc<Type> {
        self.get_type().expect("void is not allowed here")
    }
    fn assert_no_member_decorations(&self, id: IdRef) {
        for member_decoration in &self.member_decorations {
            unreachable!(
                "member decoration not allowed on {}: {:?}",
                id, member_decoration
            );
        }
    }
    fn assert_no_decorations(&self, id: IdRef) {
        self.assert_no_member_decorations(id);
        for decoration in &self.decorations {
            unreachable!("decoration not allowed on {}: {:?}", id, decoration);
        }
    }
}

#[derive(Debug)]
struct Ids(Vec<IdProperties>);

impl Index<IdRef> for Ids {
    type Output = IdProperties;
    fn index(&self, index: IdRef) -> &IdProperties {
        &self.0[index.0 as usize]
    }
}

impl IndexMut<IdRef> for Ids {
    fn index_mut(&mut self, index: IdRef) -> &mut IdProperties {
        &mut self.0[index.0 as usize]
    }
}

struct ParsedShaderFunction {
    instructions: Vec<Instruction>,
}

#[allow(dead_code)]
struct ParsedShader {
    ids: Ids,
    functions: Vec<ParsedShaderFunction>,
    main_function_id: IdRef,
    interface_variables: Vec<IdRef>,
    execution_modes: Vec<ExecutionMode>,
}

struct ShaderEntryPoint {
    main_function_id: IdRef,
    interface_variables: Vec<IdRef>,
}

impl ParsedShader {
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cyclomatic_complexity))]
    fn create(
        context: &mut Context,
        stage_info: ShaderStageCreateInfo,
        execution_model: ExecutionModel,
    ) -> Self {
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
        let mut current_function: Option<ParsedShaderFunction> = None;
        let mut functions = Vec::new();
        let mut execution_modes = Vec::new();
        for instruction in instructions {
            match current_function {
                Some(mut function) => {
                    current_function = match instruction {
                        instruction @ Instruction::FunctionEnd {} => {
                            function.instructions.push(instruction);
                            functions.push(function);
                            None
                        }
                        instruction => {
                            function.instructions.push(instruction);
                            Some(function)
                        }
                    };
                    continue;
                }
                None => current_function = None,
            }
            match instruction {
                instruction @ Instruction::Function { .. } => {
                    current_function = Some(ParsedShaderFunction {
                        instructions: vec![instruction],
                    });
                }
                Instruction::EntryPoint {
                    execution_model: current_execution_model,
                    entry_point: main_function_id,
                    name,
                    interface,
                } => {
                    if execution_model == current_execution_model
                        && name == stage_info.entry_point_name
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
                        return_type: ids[return_type].get_type(),
                        arguments: parameter_types
                            .iter()
                            .map(|argument| {
                                ids[*argument]
                                    .get_type()
                                    .expect("void is not allowed as a function argument")
                            })
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
                    ids[id_result.0]
                        .set_kind(IdKind::Type(Rc::new(Type::Scalar(ScalarType::Bool))));
                }
                Instruction::TypeInt {
                    id_result,
                    width,
                    signedness,
                } => {
                    ids[id_result.0].assert_no_decorations(id_result.0);
                    ids[id_result.0].set_kind(IdKind::Type(Rc::new(Type::Scalar(
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
                    ids[id_result.0].set_kind(IdKind::Type(Rc::new(Type::Scalar(match width {
                        16 => ScalarType::F16,
                        32 => ScalarType::F32,
                        64 => ScalarType::F64,
                        _ => unreachable!("unsupported float type: f{}", width),
                    }))));
                }
                Instruction::TypeVector {
                    id_result,
                    component_type,
                    component_count,
                } => {
                    ids[id_result.0].assert_no_decorations(id_result.0);
                    let element = match &*ids[component_type]
                        .get_type()
                        .expect("void is not a valid vector element type")
                    {
                        Type::Scalar(v) => v.clone(),
                        _ => unreachable!("vector element type must be a scalar"),
                    };
                    ids[id_result.0].set_kind(IdKind::Type(Rc::new(Type::Vector {
                        element,
                        element_count: component_count as usize,
                    })));
                }
                Instruction::TypeForwardPointer { pointer_type, .. } => {
                    ids[pointer_type].set_kind(IdKind::ForwardPointer(Rc::new(Type::Scalar(
                        ScalarType::Pointer(PointerType::unresolved()),
                    ))));
                }
                Instruction::TypePointer {
                    id_result,
                    type_: pointee,
                    ..
                } => {
                    ids[id_result.0].assert_no_decorations(id_result.0);
                    let pointee = ids[pointee].get_type();
                    let pointer = match mem::replace(&mut ids[id_result.0].kind, IdKind::Undefined)
                    {
                        IdKind::Undefined => Rc::new(Type::Scalar(ScalarType::Pointer(
                            PointerType::new(context, pointee),
                        ))),
                        IdKind::ForwardPointer(pointer) => {
                            if let Type::Scalar(ScalarType::Pointer(pointer)) = &*pointer {
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
                            }) {
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
                        match storage_class {
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
                    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cast_lossless))]
                    let constant = match &*ids[id_result_type.0].get_nonvoid_type() {
                        Type::Scalar(ScalarType::U8) => {
                            let converted_value = value as u8;
                            assert_eq!(converted_value as u32, value);
                            Constant::U8(converted_value)
                        }
                        Type::Scalar(ScalarType::U16) => {
                            let converted_value = value as u16;
                            assert_eq!(converted_value as u32, value);
                            Constant::U16(converted_value)
                        }
                        Type::Scalar(ScalarType::U32) => Constant::U32(value),
                        Type::Scalar(ScalarType::I8) => {
                            let converted_value = value as i8;
                            assert_eq!(converted_value as u32, value);
                            Constant::I8(converted_value)
                        }
                        Type::Scalar(ScalarType::I16) => {
                            let converted_value = value as i16;
                            assert_eq!(converted_value as u32, value);
                            Constant::I16(converted_value)
                        }
                        Type::Scalar(ScalarType::I32) => Constant::I32(value as i32),
                        Type::Scalar(ScalarType::F16) => {
                            let converted_value = value as u16;
                            assert_eq!(converted_value as u32, value);
                            Constant::F16(converted_value)
                        }
                        Type::Scalar(ScalarType::F32) => Constant::F32(f32::from_bits(value)),
                        _ => unreachable!("invalid type"),
                    };
                    ids[id_result.0].set_kind(IdKind::Constant(constant));
                }
                Instruction::Constant64 {
                    id_result_type,
                    id_result,
                    value,
                } => {
                    ids[id_result.0].assert_no_decorations(id_result.0);
                    let constant = match &*ids[id_result_type.0].get_nonvoid_type() {
                        Type::Scalar(ScalarType::U64) => Constant::U64(value),
                        Type::Scalar(ScalarType::I64) => Constant::I64(value as i64),
                        Type::Scalar(ScalarType::F64) => Constant::F64(f64::from_bits(value)),
                        _ => unreachable!("invalid type"),
                    };
                    ids[id_result.0].set_kind(IdKind::Constant(constant));
                }
                Instruction::ConstantFalse {
                    id_result_type,
                    id_result,
                } => {
                    ids[id_result.0].assert_no_decorations(id_result.0);
                    let constant = match &*ids[id_result_type.0].get_nonvoid_type() {
                        Type::Scalar(ScalarType::Bool) => Constant::Bool(false),
                        _ => unreachable!("invalid type"),
                    };
                    ids[id_result.0].set_kind(IdKind::Constant(constant));
                }
                Instruction::ConstantTrue {
                    id_result_type,
                    id_result,
                } => {
                    ids[id_result.0].assert_no_decorations(id_result.0);
                    let constant = match &*ids[id_result_type.0].get_nonvoid_type() {
                        Type::Scalar(ScalarType::Bool) => Constant::Bool(true),
                        _ => unreachable!("invalid type"),
                    };
                    ids[id_result.0].set_kind(IdKind::Constant(constant));
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
            functions,
            main_function_id,
            interface_variables,
            execution_modes,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GenericPipelineOptions {
    pub optimization_mode: shader_compiler_backend::OptimizationMode,
}

#[derive(Debug)]
pub struct PipelineLayout {}

#[derive(Debug)]
pub struct ComputePipeline {}

#[derive(Clone, Debug)]
pub struct ComputePipelineOptions {
    pub generic_options: GenericPipelineOptions,
}

#[derive(Copy, Clone, Debug)]
pub struct Specialization<'a> {
    pub id: u32,
    pub bytes: &'a [u8],
}

#[derive(Copy, Clone, Debug)]
pub struct ShaderStageCreateInfo<'a> {
    pub code: &'a [u32],
    pub entry_point_name: &'a str,
    pub specializations: &'a [Specialization<'a>],
}

impl ComputePipeline {
    pub fn new(
        _options: &ComputePipelineOptions,
        compute_shader_stage: ShaderStageCreateInfo,
    ) -> ComputePipeline {
        let mut context = Context::default();
        let _parsed_shader = ParsedShader::create(
            &mut context,
            compute_shader_stage,
            ExecutionModel::GLCompute,
        );
        unimplemented!()
    }
}
