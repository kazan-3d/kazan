// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

extern crate shader_compiler_backend;
extern crate spirv_parser;

use spirv_parser::{
    BuiltIn, Decoration, ExecutionMode, ExecutionModel, IdRef, Instruction, StorageClass,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::mem;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

pub struct Context {
    types: pointer_type::ContextTypes,
    next_struct_id: usize,
}

impl Default for Context {
    fn default() -> Context {
        Context {
            types: Default::default(),
            next_struct_id: 0,
        }
    }
}

mod pointer_type {
    use super::{Context, Type};
    use std::cell::RefCell;
    use std::fmt;
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

    #[derive(Clone)]
    pub struct PointerType {
        pointee: RefCell<PointerTypeState>,
    }

    impl fmt::Debug for PointerType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let mut state = f.debug_struct("PointerType");
            if let PointerTypeState::Unresolved = *self.pointee.borrow() {
                state.field("pointee", &PointerTypeState::Unresolved);
            } else {
                state.field("pointee", &self.pointee());
            }
            state.finish()
        }
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
pub struct VectorType {
    pub element: ScalarType,
    pub element_count: usize,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct StructMember {
    pub decorations: Vec<Decoration>,
    pub member_type: Rc<Type>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct StructId(usize);

impl StructId {
    pub fn new(context: &mut Context) -> Self {
        let retval = StructId(context.next_struct_id);
        context.next_struct_id += 1;
        retval
    }
}

#[derive(Clone)]
pub struct StructType {
    pub id: StructId,
    pub decorations: Vec<Decoration>,
    pub members: Vec<StructMember>,
}

impl Eq for StructType {}

impl PartialEq for StructType {
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}

impl Hash for StructType {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.id.hash(h)
    }
}

impl fmt::Debug for StructType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        thread_local! {
            static CURRENTLY_FORMATTING: RefCell<HashSet<StructId>> = RefCell::new(HashSet::new());
        }
        struct CurrentlyFormatting {
            id: StructId,
            was_formatting: bool,
        }
        impl CurrentlyFormatting {
            fn new(id: StructId) -> Self {
                let was_formatting = CURRENTLY_FORMATTING
                    .with(|currently_formatting| !currently_formatting.borrow_mut().insert(id));
                Self { id, was_formatting }
            }
        }
        impl Drop for CurrentlyFormatting {
            fn drop(&mut self) {
                if !self.was_formatting {
                    CURRENTLY_FORMATTING.with(|currently_formatting| {
                        currently_formatting.borrow_mut().remove(&self.id);
                    });
                }
            }
        }
        let currently_formatting = CurrentlyFormatting::new(self.id);
        let mut state = f.debug_struct("StructType");
        state.field("id", &self.id);
        if !currently_formatting.was_formatting {
            state.field("decorations", &self.decorations);
            state.field("members", &self.members);
        }
        state.finish()
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ArrayType {
    pub decorations: Vec<Decoration>,
    pub element: Rc<Type>,
    pub element_count: Option<usize>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Type {
    Scalar(ScalarType),
    Vector(VectorType),
    Struct(StructType),
    Array(ArrayType),
}

impl Type {
    pub fn is_pointer(&self) -> bool {
        if let Type::Scalar(ScalarType::Pointer(_)) = self {
            true
        } else {
            false
        }
    }
    pub fn is_scalar(&self) -> bool {
        if let Type::Scalar(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_vector(&self) -> bool {
        if let Type::Vector(_) = self {
            true
        } else {
            false
        }
    }
    pub fn get_pointee(&self) -> Option<Rc<Type>> {
        if let Type::Scalar(ScalarType::Pointer(pointer)) = self {
            pointer.pointee()
        } else {
            unreachable!("not a pointer")
        }
    }
    pub fn get_nonvoid_pointee(&self) -> Rc<Type> {
        self.get_pointee().expect("void is not allowed here")
    }
    pub fn get_scalar(&self) -> &ScalarType {
        if let Type::Scalar(scalar) = self {
            scalar
        } else {
            unreachable!("not a scalar type")
        }
    }
    pub fn get_vector(&self) -> &VectorType {
        if let Type::Vector(vector) = self {
            vector
        } else {
            unreachable!("not a vector type")
        }
    }
}

/// value that can be either defined or undefined
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Undefable<T> {
    Undefined,
    Defined(T),
}

impl<T> Undefable<T> {
    pub fn unwrap(self) -> T {
        match self {
            Undefable::Undefined => panic!("Undefable::unwrap called on Undefined"),
            Undefable::Defined(v) => v,
        }
    }
}

impl<T> From<T> for Undefable<T> {
    fn from(v: T) -> Undefable<T> {
        Undefable::Defined(v)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ScalarConstant {
    U8(Undefable<u8>),
    U16(Undefable<u16>),
    U32(Undefable<u32>),
    U64(Undefable<u64>),
    I8(Undefable<i8>),
    I16(Undefable<i16>),
    I32(Undefable<i32>),
    I64(Undefable<i64>),
    F16(Undefable<u16>),
    F32(Undefable<f32>),
    F64(Undefable<f64>),
    Bool(Undefable<bool>),
}

macro_rules! define_scalar_vector_constant_impl_without_from {
    ($type:ident, $name:ident, $get_name:ident) => {
        impl ScalarConstant {
            pub fn $get_name(self) -> Undefable<$type> {
                match self {
                    ScalarConstant::$name(v) => v,
                    _ => unreachable!(concat!("expected a constant ", stringify!($type))),
                }
            }
        }
        impl VectorConstant {
            pub fn $get_name(&self) -> &Vec<Undefable<$type>> {
                match self {
                    VectorConstant::$name(v) => v,
                    _ => unreachable!(concat!(
                        "expected a constant vector with ",
                        stringify!($type),
                        " elements"
                    )),
                }
            }
        }
    };
}

macro_rules! define_scalar_vector_constant_impl {
    ($type:ident, $name:ident, $get_name:ident) => {
        define_scalar_vector_constant_impl_without_from!($type, $name, $get_name);
        impl From<Undefable<$type>> for ScalarConstant {
            fn from(v: Undefable<$type>) -> ScalarConstant {
                ScalarConstant::$name(v)
            }
        }
        impl From<Vec<Undefable<$type>>> for VectorConstant {
            fn from(v: Vec<Undefable<$type>>) -> VectorConstant {
                VectorConstant::$name(v)
            }
        }
    };
}

define_scalar_vector_constant_impl!(u8, U8, get_u8);
define_scalar_vector_constant_impl!(u16, U16, get_u16);
define_scalar_vector_constant_impl!(u32, U32, get_u32);
define_scalar_vector_constant_impl!(u64, U64, get_u64);
define_scalar_vector_constant_impl!(i8, I8, get_i8);
define_scalar_vector_constant_impl!(i16, I16, get_i16);
define_scalar_vector_constant_impl!(i32, I32, get_i32);
define_scalar_vector_constant_impl!(i64, I64, get_i64);
define_scalar_vector_constant_impl_without_from!(u16, F16, get_f16);
define_scalar_vector_constant_impl!(f32, F32, get_f32);
define_scalar_vector_constant_impl!(f64, F64, get_f64);
define_scalar_vector_constant_impl!(bool, Bool, get_bool);

impl ScalarConstant {
    pub fn get_type(self) -> Type {
        Type::Scalar(self.get_scalar_type())
    }
    pub fn get_scalar_type(self) -> ScalarType {
        match self {
            ScalarConstant::U8(_) => ScalarType::U8,
            ScalarConstant::U16(_) => ScalarType::U16,
            ScalarConstant::U32(_) => ScalarType::U32,
            ScalarConstant::U64(_) => ScalarType::U64,
            ScalarConstant::I8(_) => ScalarType::I8,
            ScalarConstant::I16(_) => ScalarType::I16,
            ScalarConstant::I32(_) => ScalarType::I32,
            ScalarConstant::I64(_) => ScalarType::I64,
            ScalarConstant::F16(_) => ScalarType::F16,
            ScalarConstant::F32(_) => ScalarType::F32,
            ScalarConstant::F64(_) => ScalarType::F64,
            ScalarConstant::Bool(_) => ScalarType::Bool,
        }
    }
}

#[derive(Clone, Debug)]
pub enum VectorConstant {
    U8(Vec<Undefable<u8>>),
    U16(Vec<Undefable<u16>>),
    U32(Vec<Undefable<u32>>),
    U64(Vec<Undefable<u64>>),
    I8(Vec<Undefable<i8>>),
    I16(Vec<Undefable<i16>>),
    I32(Vec<Undefable<i32>>),
    I64(Vec<Undefable<i64>>),
    F16(Vec<Undefable<u16>>),
    F32(Vec<Undefable<f32>>),
    F64(Vec<Undefable<f64>>),
    Bool(Vec<Undefable<bool>>),
}

impl VectorConstant {
    pub fn get_element_type(&self) -> ScalarType {
        match self {
            VectorConstant::U8(_) => ScalarType::U8,
            VectorConstant::U16(_) => ScalarType::U16,
            VectorConstant::U32(_) => ScalarType::U32,
            VectorConstant::U64(_) => ScalarType::U64,
            VectorConstant::I8(_) => ScalarType::I8,
            VectorConstant::I16(_) => ScalarType::I16,
            VectorConstant::I32(_) => ScalarType::I32,
            VectorConstant::I64(_) => ScalarType::I64,
            VectorConstant::F16(_) => ScalarType::F16,
            VectorConstant::F32(_) => ScalarType::F32,
            VectorConstant::F64(_) => ScalarType::F64,
            VectorConstant::Bool(_) => ScalarType::Bool,
        }
    }
    pub fn get_element_count(&self) -> usize {
        match self {
            VectorConstant::U8(v) => v.len(),
            VectorConstant::U16(v) => v.len(),
            VectorConstant::U32(v) => v.len(),
            VectorConstant::U64(v) => v.len(),
            VectorConstant::I8(v) => v.len(),
            VectorConstant::I16(v) => v.len(),
            VectorConstant::I32(v) => v.len(),
            VectorConstant::I64(v) => v.len(),
            VectorConstant::F16(v) => v.len(),
            VectorConstant::F32(v) => v.len(),
            VectorConstant::F64(v) => v.len(),
            VectorConstant::Bool(v) => v.len(),
        }
    }
    pub fn get_type(&self) -> Type {
        Type::Vector(VectorType {
            element: self.get_element_type(),
            element_count: self.get_element_count(),
        })
    }
}

#[derive(Clone, Debug)]
pub enum Constant {
    Scalar(ScalarConstant),
    Vector(VectorConstant),
}

impl Constant {
    pub fn get_type(&self) -> Type {
        match self {
            Constant::Scalar(v) => v.get_type(),
            Constant::Vector(v) => v.get_type(),
        }
    }
    pub fn get_scalar(&self) -> &ScalarConstant {
        match self {
            Constant::Scalar(v) => v,
            _ => unreachable!("not a scalar constant"),
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
            BuiltIn::GlobalInvocationId => Rc::new(Type::Vector(VectorType {
                element: ScalarType::U32,
                element_count: 3,
            })),
            _ => unreachable!("unknown built-in"),
        }
    }
}

#[derive(Debug, Clone)]
struct UniformVariable {
    binding: u32,
    descriptor_set: u32,
    variable_type: Rc<Type>,
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
    Constant(Rc<Constant>),
    UniformVariable(UniformVariable),
}

#[derive(Debug)]
struct IdProperties {
    kind: IdKind,
    decorations: Vec<Decoration>,
    member_decorations: Vec<MemberDecoration>,
}

impl IdProperties {
    fn is_empty(&self) -> bool {
        match self.kind {
            IdKind::Undefined => {}
            _ => return false,
        }
        self.decorations.is_empty() && self.member_decorations.is_empty()
    }
    fn set_kind(&mut self, kind: IdKind) {
        match &self.kind {
            IdKind::Undefined => {}
            _ => unreachable!("duplicate id"),
        }
        self.kind = kind;
    }
    fn get_type(&self) -> Option<&Rc<Type>> {
        match &self.kind {
            IdKind::Type(t) => Some(t),
            IdKind::VoidType => None,
            _ => unreachable!("id is not type"),
        }
    }
    fn get_nonvoid_type(&self) -> &Rc<Type> {
        self.get_type().expect("void is not allowed here")
    }
    fn get_constant(&self) -> &Rc<Constant> {
        match &self.kind {
            IdKind::Constant(c) => c,
            _ => unreachable!("id is not a constant"),
        }
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

struct Ids(Vec<IdProperties>);

impl fmt::Debug for Ids {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entries(
                self.0
                    .iter()
                    .enumerate()
                    .filter_map(|(id_index, id_properties)| {
                        if id_properties.is_empty() {
                            return None;
                        }
                        Some((IdRef(id_index as u32), id_properties))
                    }),
            )
            .finish()
    }
}

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

impl fmt::Debug for ParsedShaderFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParsedShaderFunction:\n")?;
        for instruction in &self.instructions {
            write!(f, "{}", instruction)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct ParsedShader {
    ids: Ids,
    functions: Vec<ParsedShaderFunction>,
    main_function_id: IdRef,
    interface_variables: Vec<IdRef>,
    execution_modes: Vec<ExecutionMode>,
    workgroup_size: Option<(u32, u32, u32)>,
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
        let mut workgroup_size = None;
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
                    let element = ids[component_type].get_nonvoid_type().get_scalar().clone();
                    ids[id_result.0].set_kind(IdKind::Type(Rc::new(Type::Vector(VectorType {
                        element,
                        element_count: component_count as usize,
                    }))));
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
                    let pointee = ids[pointee].get_type().map(Clone::clone);
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
                    ids[id_result.0].set_kind(IdKind::Type(Rc::new(Type::Struct(struct_type))));
                }
                Instruction::TypeRuntimeArray {
                    id_result,
                    element_type,
                } => {
                    ids[id_result.0].assert_no_member_decorations(id_result.0);
                    let decorations = ids[id_result.0].decorations.clone();
                    let element = ids[element_type].get_nonvoid_type().clone();
                    ids[id_result.0].set_kind(IdKind::Type(Rc::new(Type::Array(ArrayType {
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
                                            assert!(
                                                binding.is_none(),
                                                "duplicate Binding decoration"
                                            );
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
                                let binding = binding
                                    .expect("uniform variable is missing Binding decoration");
                                assert!(initializer.is_none());
                                ids[id_result.0].set_kind(IdKind::UniformVariable(
                                    UniformVariable {
                                        binding,
                                        descriptor_set,
                                        variable_type,
                                    },
                                ));
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
                    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cast_lossless))]
                    let constant = match **ids[id_result_type.0].get_nonvoid_type() {
                        Type::Scalar(ScalarType::U8) => {
                            let converted_value = value as u8;
                            assert_eq!(converted_value as u32, value);
                            Constant::Scalar(ScalarConstant::U8(Undefable::Defined(
                                converted_value,
                            )))
                        }
                        Type::Scalar(ScalarType::U16) => {
                            let converted_value = value as u16;
                            assert_eq!(converted_value as u32, value);
                            Constant::Scalar(ScalarConstant::U16(Undefable::Defined(
                                converted_value,
                            )))
                        }
                        Type::Scalar(ScalarType::U32) => {
                            Constant::Scalar(ScalarConstant::U32(Undefable::Defined(value)))
                        }
                        Type::Scalar(ScalarType::I8) => {
                            let converted_value = value as i8;
                            assert_eq!(converted_value as u32, value);
                            Constant::Scalar(ScalarConstant::I8(Undefable::Defined(
                                converted_value,
                            )))
                        }
                        Type::Scalar(ScalarType::I16) => {
                            let converted_value = value as i16;
                            assert_eq!(converted_value as u32, value);
                            Constant::Scalar(ScalarConstant::I16(Undefable::Defined(
                                converted_value,
                            )))
                        }
                        Type::Scalar(ScalarType::I32) => {
                            Constant::Scalar(ScalarConstant::I32(Undefable::Defined(value as i32)))
                        }
                        Type::Scalar(ScalarType::F16) => {
                            let converted_value = value as u16;
                            assert_eq!(converted_value as u32, value);
                            Constant::Scalar(ScalarConstant::F16(Undefable::Defined(
                                converted_value,
                            )))
                        }
                        Type::Scalar(ScalarType::F32) => Constant::Scalar(ScalarConstant::F32(
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
                        Type::Scalar(ScalarType::U64) => {
                            Constant::Scalar(ScalarConstant::U64(Undefable::Defined(value)))
                        }
                        Type::Scalar(ScalarType::I64) => {
                            Constant::Scalar(ScalarConstant::I64(Undefable::Defined(value as i64)))
                        }
                        Type::Scalar(ScalarType::F64) => Constant::Scalar(ScalarConstant::F64(
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
                        Type::Scalar(ScalarType::Bool) => {
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
                        Type::Scalar(ScalarType::Bool) => {
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
                        Type::Vector(VectorType {
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
                                ScalarType::Bool => VectorConstant::Bool(
                                    constituents.map(|v| v.get_bool()).collect(),
                                ),
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
                    ids[id_result.0]
                        .set_kind(IdKind::Constant(Rc::new(Constant::Vector(constant))));
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
            workgroup_size,
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
        let parsed_shader = ParsedShader::create(
            &mut context,
            compute_shader_stage,
            ExecutionModel::GLCompute,
        );
        println!("parsed_shader:\n{:#?}", parsed_shader);
        unimplemented!()
    }
}
