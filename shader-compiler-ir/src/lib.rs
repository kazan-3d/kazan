// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![warn(missing_docs)]

//! Shader Compiler Intermediate Representation

mod debug;
mod global_state;
mod interned_string;

pub use crate::debug::Location;
pub use crate::debug::LocationData;
pub use crate::global_state::GlobalState;
pub use crate::interned_string::InternedString;
use std::cell::RefCell;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;
use std::rc::Weak;

/// if a type or value `T` is inhabited (is reachable)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Inhabitable<T> {
    /// type or value `T` is inhabited (is reachable)
    Inhabited(T),
    /// uninhabited (unreachable)
    Uninhabited,
}

pub use Inhabitable::*;

impl<T> Inhabitable<T> {
    /// like `Option::as_ref`
    pub fn as_ref(&self) -> Inhabitable<&T> {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    /// like `Option::as_mut`
    pub fn as_mut(&mut self) -> Inhabitable<&mut T> {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    /// like `Option::map`
    pub fn map<F: FnOnce(T) -> R, R>(self, f: F) -> Inhabitable<R> {
        match self {
            Inhabited(v) => Inhabited(f(v)),
            Uninhabited => Uninhabited,
        }
    }
    /// like `Option::as_deref`
    pub fn as_deref(&self) -> Inhabitable<&T::Target>
    where
        T: Deref,
    {
        self.as_ref().map(|v| &**v)
    }
    /// like `Option::as_deref_mut`
    pub fn as_deref_mut(&mut self) -> Inhabitable<&mut T::Target>
    where
        T: DerefMut,
    {
        self.as_mut().map(|v| &mut **v)
    }
    /// return `Some` if `self` is `Inhabited`
    pub fn inhabited(self) -> Option<T> {
        match self {
            Inhabited(v) => Some(v),
            Uninhabited => None,
        }
    }
}

/// code structure input/output
pub(crate) trait CodeIO {
    /// the list of SSA value definitions that are the results of executing `self`, or `Uninhabited` if `self` doesn't return
    fn results(&self) -> Inhabitable<&[ValueDefinition]>;
    /// the list of SSA values that are the arguments for `self`
    fn arguments(&self) -> &[ValueUse];
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum SimpleInstructionKind {}

#[derive(Debug)]
pub struct BreakBlock {
    pub block: Weak<Block>,
    pub block_results: Vec<ValueUse>,
}

impl CodeIO for BreakBlock {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        Uninhabited
    }
    fn arguments(&self) -> &[ValueUse] {
        &self.block_results
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct LoopHeader {
    pub argument_definitions: Vec<ValueDefinition>,
}

impl CodeIO for LoopHeader {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        Inhabited(&self.argument_definitions)
    }
    fn arguments(&self) -> &[ValueUse] {
        &[]
    }
}

#[derive(Debug)]
pub struct Block {
    pub body: Vec<Instruction>,
    pub result_definitions: Inhabitable<Vec<ValueDefinition>>,
}

impl CodeIO for Block {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        self.result_definitions.as_deref()
    }
    fn arguments(&self) -> &[ValueUse] {
        &[]
    }
}

#[derive(Debug)]
pub struct Loop {
    pub arguments: Vec<ValueUse>,
    pub header: LoopHeader,
    pub body: Rc<Block>,
}

impl CodeIO for Loop {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        self.body.results()
    }
    fn arguments(&self) -> &[ValueUse] {
        &self.arguments
    }
}

#[derive(Debug)]
pub struct ContinueLoop {
    pub target_loop: Weak<Loop>,
    pub block_arguments: Vec<ValueUse>,
}

impl CodeIO for ContinueLoop {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        Uninhabited
    }
    fn arguments(&self) -> &[ValueUse] {
        &self.block_arguments
    }
}

#[derive(Debug)]
pub enum SimpleInstruction {}

impl CodeIO for SimpleInstruction {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        match *self {}
    }
    fn arguments(&self) -> &[ValueUse] {
        match *self {}
    }
}

/// variable part of `Instruction`
#[derive(Debug)]
pub enum InstructionData {
    Simple(SimpleInstruction),
    Block(Rc<Block>),
    Loop(Rc<Loop>),
    ContinueLoop(ContinueLoop),
    BreakBlock(BreakBlock),
}

impl CodeIO for InstructionData {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        match self {
            InstructionData::Simple(v) => v.results(),
            InstructionData::Block(v) => v.results(),
            InstructionData::Loop(v) => v.results(),
            InstructionData::ContinueLoop(v) => v.results(),
            InstructionData::BreakBlock(v) => v.results(),
        }
    }
    fn arguments(&self) -> &[ValueUse] {
        match self {
            InstructionData::Simple(v) => v.arguments(),
            InstructionData::Block(v) => v.arguments(),
            InstructionData::Loop(v) => v.arguments(),
            InstructionData::ContinueLoop(v) => v.arguments(),
            InstructionData::BreakBlock(v) => v.arguments(),
        }
    }
}

#[derive(Debug)]
pub struct Instruction {
    pub location: Location,
    pub data: InstructionData,
}

impl CodeIO for Instruction {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        self.data.results()
    }
    fn arguments(&self) -> &[ValueUse] {
        self.data.arguments()
    }
}

#[derive(Clone)]
pub struct Type(Rc<TypeData>);

impl PartialEq for Type {
    fn eq(&self, rhs: &Self) -> bool {
        Rc::ptr_eq(&self.0, &rhs.0)
    }
}

impl Eq for Type {}

impl Hash for Type {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (&*self.0 as *const TypeData).hash(hasher)
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl Deref for Type {
    type Target = TypeData;
    fn deref(&self) -> &TypeData {
        &*self.0
    }
}

impl Type {
    pub fn get(value: &TypeData, global_state: &GlobalState) -> Type {
        Type(global_state.type_interner.intern(value))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum IntegerType {
    Int8,
    Int16,
    Int32,
    Int64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FloatType {
    Float16,
    Float32,
    Float64,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum OpaqueType {}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TypeData {
    Integer {
        integer_type: IntegerType,
    },
    Float {
        float_type: FloatType,
    },
    Bool,
    Pointer {
        pointee: Type,
    },
    Vector {
        size: usize,
        element: Type,
    },
    Matrix {
        columns: usize,
        rows: usize,
        element: Type,
    },
    VariableVector {
        element: Type,
    },
    Opaque {
        opaque_type: OpaqueType,
    },
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ConstInteger {
    pub value: u64,
    pub integer_type: IntegerType,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConstFloat {
    Float16 { bits: u16 },
    Float32 { bits: u32 },
    Float64 { bits: u64 },
}

impl ConstFloat {
    pub fn get_type(self) -> FloatType {
        match self {
            ConstFloat::Float16 { .. } => FloatType::Float16,
            ConstFloat::Float32 { .. } => FloatType::Float32,
            ConstFloat::Float64 { .. } => FloatType::Float64,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ConstData {
    Integer(ConstInteger),
    Float(ConstFloat),
    Bool(bool),
    Vector {
        size: usize,
        element: Const,
    },
    Matrix {
        columns: usize,
        rows: usize,
        element: Const,
    },
    Undef(Type),
}

impl ConstData {
    pub fn get_type(&self, global_state: &GlobalState) -> Type {
        match *self {
            ConstData::Integer(ConstInteger { integer_type, .. }) => {
                Type::get(&TypeData::Integer { integer_type }, global_state)
            }
            ConstData::Float(const_float) => Type::get(
                &TypeData::Float {
                    float_type: const_float.get_type(),
                },
                global_state,
            ),
            ConstData::Bool(_) => Type::get(&TypeData::Bool, global_state),
            ConstData::Vector { size, ref element } => Type::get(
                &TypeData::Vector {
                    size,
                    element: element.get_type().clone(),
                },
                global_state,
            ),
            ConstData::Matrix {
                columns,
                rows,
                ref element,
            } => Type::get(
                &TypeData::Matrix {
                    columns,
                    rows,
                    element: element.get_type().clone(),
                },
                global_state,
            ),
            ConstData::Undef(ref retval) => retval.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Const {
    data: Rc<ConstData>,
    const_type: Type,
}

impl PartialEq for Const {
    fn eq(&self, rhs: &Self) -> bool {
        Rc::ptr_eq(&self.data, &rhs.data)
    }
}

impl Eq for Const {}

impl Hash for Const {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        (&*self.data as *const ConstData).hash(hasher)
    }
}

impl fmt::Debug for Const {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl Deref for Const {
    type Target = ConstData;
    fn deref(&self) -> &ConstData {
        &*self.data
    }
}

impl Const {
    pub fn get(value: &ConstData, global_state: &GlobalState) -> Const {
        let data = global_state.const_interner.intern(value);
        let const_type = value.get_type(global_state);
        Const { data, const_type }
    }
    pub fn get_type(&self) -> &Type {
        &self.const_type
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ValueDefinition {
    value: Value,
}

impl ValueDefinition {
    pub fn value(&self) -> Value {
        self.value.clone()
    }
    pub fn new(name: InternedString, value_type: Type) -> Self {
        ValueDefinition {
            value: Value {
                data: Rc::new(ValueData {
                    name,
                    value_type,
                    const_value: RefCell::new(None),
                }),
            },
        }
    }
    pub fn define_as_const(self, const_value: Const) -> Value {
        let Self { value } = self;
        assert_eq!(value.value_type, *const_value.get_type());
        *value.const_value.borrow_mut() = Some(const_value);
        value
    }
}

impl Deref for ValueDefinition {
    type Target = ValueData;
    fn deref(&self) -> &ValueData {
        &*self.value
    }
}

#[derive(Clone, Debug)]
pub struct ValueData {
    pub value_type: Type,
    pub name: InternedString,
    const_value: RefCell<Option<Const>>,
}

#[derive(Clone)]
pub struct Value {
    data: Rc<ValueData>,
}

impl Value {
    pub fn id(&self) -> *const ValueData {
        &*self.data
    }
    pub fn from_const(const_value: Const, name: InternedString) -> Value {
        Value {
            data: Rc::new(ValueData {
                name,
                value_type: const_value.get_type().clone(),
                const_value: RefCell::new(Some(const_value)),
            }),
        }
    }
    pub fn const_value(&self) -> Option<Const> {
        self.const_value.borrow().clone()
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id().hash(hasher)
    }
}

impl PartialEq for Value {
    fn eq(&self, rhs: &Self) -> bool {
        self.id() == rhs.id()
    }
}

impl Eq for Value {}

impl Deref for Value {
    type Target = ValueData;
    fn deref(&self) -> &ValueData {
        &*self.data
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        macro_rules! debug_fields {
            (
                $($field:ident,)+
            ) => {
                {
                    let ValueData {
                        $($field,)+
                        const_value,
                    } = &**self;
                    f.debug_struct("Value")
                    .field("id", &self.id())
                    $(.field(stringify!($field), $field))+
                    .field("const_value", &*const_value.borrow())
                    .finish()
                }
            };
        }
        debug_fields! {
            value_type,
            name,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ValueUse {
    value: Value,
}

impl ValueUse {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
    pub fn value(&self) -> Value {
        self.value.clone()
    }
}

impl Deref for ValueUse {
    type Target = ValueData;
    fn deref(&self) -> &ValueData {
        &*self.value
    }
}
