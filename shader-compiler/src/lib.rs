// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

mod cfg;
mod instruction_properties;
mod lattice;
mod parsed_shader_compile;
mod parsed_shader_create;
mod uniformity;

use crate::parsed_shader_compile::ParsedShaderCompile;
use shader_compiler_backend::Module;
use spirv_parser::{BuiltIn, Decoration, ExecutionMode, ExecutionModel, IdRef, Instruction};
use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum CompiledFunctionKey {
    ComputeShaderEntrypoint,
}

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
    use crate::{Context, FrontendType};
    use std::cell::RefCell;
    use std::fmt;
    use std::hash::{Hash, Hasher};
    use std::rc::{Rc, Weak};

    #[derive(Default)]
    pub struct ContextTypes(Vec<Rc<FrontendType>>);

    #[derive(Clone, Debug)]
    enum PointerTypeState {
        Void,
        Normal(Weak<FrontendType>),
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
        pub fn new(context: &mut Context, pointee: Option<Rc<FrontendType>>) -> Self {
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
        pub fn resolve(&self, context: &mut Context, new_pointee: Option<Rc<FrontendType>>) {
            let mut pointee = self.pointee.borrow_mut();
            match &*pointee {
                PointerTypeState::Unresolved => {}
                _ => unreachable!("pointer already resolved"),
            }
            *pointee = Self::new(context, new_pointee).pointee.into_inner();
        }
        pub fn pointee(&self) -> Option<Rc<FrontendType>> {
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

pub use crate::pointer_type::PointerType;

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
    pub member_type: Rc<FrontendType>,
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
    pub element: Rc<FrontendType>,
    pub element_count: Option<usize>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum FrontendType {
    Scalar(ScalarType),
    Vector(VectorType),
    Struct(StructType),
    Array(ArrayType),
}

impl FrontendType {
    pub fn is_pointer(&self) -> bool {
        if let FrontendType::Scalar(ScalarType::Pointer(_)) = self {
            true
        } else {
            false
        }
    }
    pub fn is_scalar(&self) -> bool {
        if let FrontendType::Scalar(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_vector(&self) -> bool {
        if let FrontendType::Vector(_) = self {
            true
        } else {
            false
        }
    }
    pub fn get_pointee(&self) -> Option<Rc<FrontendType>> {
        if let FrontendType::Scalar(ScalarType::Pointer(pointer)) = self {
            pointer.pointee()
        } else {
            unreachable!("not a pointer")
        }
    }
    pub fn get_nonvoid_pointee(&self) -> Rc<FrontendType> {
        self.get_pointee().expect("void is not allowed here")
    }
    pub fn get_scalar(&self) -> &ScalarType {
        if let FrontendType::Scalar(scalar) = self {
            scalar
        } else {
            unreachable!("not a scalar type")
        }
    }
    pub fn get_vector(&self) -> &VectorType {
        if let FrontendType::Vector(vector) = self {
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
    pub fn get_type(self) -> FrontendType {
        FrontendType::Scalar(self.get_scalar_type())
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
    pub fn get_type(&self) -> FrontendType {
        FrontendType::Vector(VectorType {
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
    pub fn get_type(&self) -> FrontendType {
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
    fn get_type(&self, _context: &mut Context) -> Rc<FrontendType> {
        match self.built_in {
            BuiltIn::GlobalInvocationId => Rc::new(FrontendType::Vector(VectorType {
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
    variable_type: Rc<FrontendType>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum CrossLaneBehavior {
    Uniform,
    Nonuniform,
}

#[derive(Debug)]
struct FrontendValue<'a, C: shader_compiler_backend::Context<'a>> {
    frontend_type: Rc<FrontendType>,
    backend_value: Option<C::Value>,
    cross_lane_behavior: CrossLaneBehavior,
}

#[derive(Debug)]
enum IdKind<'a, C: shader_compiler_backend::Context<'a>> {
    Undefined,
    DecorationGroup,
    Type(Rc<FrontendType>),
    VoidType,
    FunctionType {
        return_type: Option<Rc<FrontendType>>,
        arguments: Vec<Rc<FrontendType>>,
    },
    ForwardPointer(Rc<FrontendType>),
    BuiltInVariable(BuiltInVariable),
    Constant(Rc<Constant>),
    UniformVariable(UniformVariable),
    Function(Option<ParsedShaderFunction>),
    BasicBlock {
        basic_block: C::BasicBlock,
        buildable_basic_block: Option<C::BuildableBasicBlock>,
    },
    Value(FrontendValue<'a, C>),
}

#[derive(Debug)]
struct IdProperties<'a, C: shader_compiler_backend::Context<'a>> {
    kind: IdKind<'a, C>,
    decorations: Vec<Decoration>,
    member_decorations: Vec<MemberDecoration>,
}

impl<'a, C: shader_compiler_backend::Context<'a>> IdProperties<'a, C> {
    fn is_empty(&self) -> bool {
        match self.kind {
            IdKind::Undefined => {}
            _ => return false,
        }
        self.decorations.is_empty() && self.member_decorations.is_empty()
    }
    fn set_kind(&mut self, kind: IdKind<'a, C>) {
        match &self.kind {
            IdKind::Undefined => {}
            _ => unreachable!("duplicate id"),
        }
        self.kind = kind;
    }
    fn get_type(&self) -> Option<&Rc<FrontendType>> {
        match &self.kind {
            IdKind::Type(t) => Some(t),
            IdKind::VoidType => None,
            _ => unreachable!("id is not type"),
        }
    }
    fn get_nonvoid_type(&self) -> &Rc<FrontendType> {
        self.get_type().expect("void is not allowed here")
    }
    fn get_constant(&self) -> &Rc<Constant> {
        match &self.kind {
            IdKind::Constant(c) => c,
            _ => unreachable!("id is not a constant"),
        }
    }
    fn get_value(&self) -> &FrontendValue<'a, C> {
        match &self.kind {
            IdKind::Value(retval) => retval,
            _ => unreachable!("id is not a value"),
        }
    }
    fn get_value_mut(&mut self) -> &mut FrontendValue<'a, C> {
        match &mut self.kind {
            IdKind::Value(retval) => retval,
            _ => unreachable!("id is not a value"),
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

struct Ids<'a, C: shader_compiler_backend::Context<'a>>(Vec<IdProperties<'a, C>>);

impl<'a, C: shader_compiler_backend::Context<'a>> Ids<'a, C> {
    pub fn iter(&self) -> impl Iterator<Item = (IdRef, &IdProperties<'a, C>)> {
        (1..self.0.len()).map(move |index| (IdRef(index as u32), &self.0[index]))
    }
}

impl<'a, C: shader_compiler_backend::Context<'a>> fmt::Debug for Ids<'a, C> {
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

impl<'a, C: shader_compiler_backend::Context<'a>> Index<IdRef> for Ids<'a, C> {
    type Output = IdProperties<'a, C>;
    fn index<'b>(&'b self, index: IdRef) -> &'b IdProperties<'a, C> {
        &self.0[index.0 as usize]
    }
}

impl<'a, C: shader_compiler_backend::Context<'a>> IndexMut<IdRef> for Ids<'a, C> {
    fn index_mut(&mut self, index: IdRef) -> &mut IdProperties<'a, C> {
        &mut self.0[index.0 as usize]
    }
}

struct ParsedShaderFunction {
    instructions: Vec<Instruction>,
    decorations: Vec<Decoration>,
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
struct ParsedShader<'a, C: shader_compiler_backend::Context<'a>> {
    ids: Ids<'a, C>,
    main_function_id: IdRef,
    interface_variables: Vec<IdRef>,
    execution_modes: Vec<ExecutionMode>,
    workgroup_size: Option<(u32, u32, u32)>,
}

struct ShaderEntryPoint {
    main_function_id: IdRef,
    interface_variables: Vec<IdRef>,
}

impl<'a, C: shader_compiler_backend::Context<'a>> ParsedShader<'a, C> {
    fn create(
        context: &mut Context,
        stage_info: ShaderStageCreateInfo,
        execution_model: ExecutionModel,
    ) -> Self {
        parsed_shader_create::create(context, stage_info, execution_model)
    }
}

#[derive(Clone, Debug)]
pub struct GenericPipelineOptions {
    pub optimization_mode: shader_compiler_backend::OptimizationMode,
}

#[derive(Clone, Debug)]
pub enum DescriptorLayout {
    Sampler { count: usize },
    CombinedImageSampler { count: usize },
    SampledImage { count: usize },
    StorageImage { count: usize },
    UniformTexelBuffer { count: usize },
    StorageTexelBuffer { count: usize },
    UniformBuffer { count: usize },
    StorageBuffer { count: usize },
    UniformBufferDynamic { count: usize },
    StorageBufferDynamic { count: usize },
    InputAttachment { count: usize },
}

#[derive(Clone, Debug)]
pub struct DescriptorSetLayout {
    pub bindings: Vec<Option<DescriptorLayout>>,
}

#[derive(Clone, Debug)]
pub struct PipelineLayout {
    pub push_constants_size: usize,
    pub descriptor_sets: Vec<DescriptorSetLayout>,
}

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
    pub fn new<C: shader_compiler_backend::Compiler>(
        options: &ComputePipelineOptions,
        compute_shader_stage: ShaderStageCreateInfo,
        pipeline_layout: PipelineLayout,
        backend_compiler: C,
    ) -> ComputePipeline {
        let mut frontend_context = Context::default();
        struct CompilerUser<'a> {
            frontend_context: Context,
            compute_shader_stage: ShaderStageCreateInfo<'a>,
        }
        #[derive(Debug)]
        enum CompileError {}
        impl<'cu> shader_compiler_backend::CompilerUser for CompilerUser<'cu> {
            type FunctionKey = CompiledFunctionKey;
            type Error = CompileError;
            fn create_error(message: String) -> CompileError {
                panic!("compile error: {}", message)
            }
            fn run<'a, C: shader_compiler_backend::Context<'a>>(
                self,
                context: &'a C,
            ) -> Result<
                shader_compiler_backend::CompileInputs<'a, C, CompiledFunctionKey>,
                CompileError,
            > {
                let backend_context = context;
                let CompilerUser {
                    mut frontend_context,
                    compute_shader_stage,
                } = self;
                let parsed_shader = ParsedShader::create(
                    &mut frontend_context,
                    compute_shader_stage,
                    ExecutionModel::GLCompute,
                );
                let mut module = backend_context.create_module("");
                let function = parsed_shader.compile(
                    &mut frontend_context,
                    backend_context,
                    &mut module,
                    "fn_",
                );
                Ok(shader_compiler_backend::CompileInputs {
                    module: module.verify().unwrap(),
                    callable_functions: iter::once((
                        CompiledFunctionKey::ComputeShaderEntrypoint,
                        function,
                    ))
                    .collect(),
                })
            }
        }
        let compile_results = backend_compiler
            .run(
                CompilerUser {
                    frontend_context,
                    compute_shader_stage,
                },
                shader_compiler_backend::CompilerIndependentConfig {
                    optimization_mode: options.generic_options.optimization_mode,
                }
                .into(),
            )
            .unwrap();
        unimplemented!()
    }
}
