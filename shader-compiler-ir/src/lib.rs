// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![warn(missing_docs)]

//! Shader Compiler Intermediate Representation

pub use once_cell::unsync::OnceCell;
use std::cell::Cell;
use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr::NonNull;
use typed_arena::Arena;

pub struct GlobalState<'g> {
    string_byte_arena: Arena<u8>,
    string_hashtable: RefCell<HashSet<&'g str>>,
    location_arena: Arena<Location<'g>>,
    location_hashtable: RefCell<HashSet<&'g Location<'g>>>,
    type_arena: Arena<Type<'g>>,
    type_hashtable: RefCell<HashSet<&'g Type<'g>>>,
    const_arena: Arena<Const<'g>>,
    const_hashtable: RefCell<HashSet<&'g Const<'g>>>,
    value_arena: Arena<Value<'g>>,
    block_arena: Arena<Block<'g>>,
    loop_arena: Arena<Loop<'g>>,
}

impl<'g> GlobalState<'g> {
    /// create a new `GlobalState`
    pub fn new() -> Self {
        Self {
            string_byte_arena: Arena::new(),
            string_hashtable: RefCell::new(HashSet::new()),
            location_arena: Arena::new(),
            location_hashtable: RefCell::new(HashSet::new()),
            type_arena: Arena::new(),
            type_hashtable: RefCell::new(HashSet::new()),
            const_arena: Arena::new(),
            const_hashtable: RefCell::new(HashSet::new()),
            value_arena: Arena::new(),
            block_arena: Arena::new(),
            loop_arena: Arena::new(),
        }
    }
}

impl<'g> Default for GlobalState<'g> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
pub struct Private {
    _private: (),
}

impl Private {
    const fn new() -> Self {
        Self { _private: () }
    }
}

#[repr(transparent)]
pub struct IdRef<'g, T>(&'g T);

impl<'g, T: fmt::Debug> fmt::Debug for IdRef<'g, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Omitted;
        impl fmt::Debug for Omitted {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.pad("<omitted>")
            }
        }
        struct NumericId(u64);
        impl fmt::Debug for NumericId {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "#{}", self.0)
            }
        }
        #[derive(Default)]
        struct InProgressIds {
            map: HashMap<NonNull<u8>, u64>,
            next_numeric_id: u64,
        }
        thread_local! {
            static IN_PROGRESS_IDS: RefCell<InProgressIds> = RefCell::default();
        }
        struct RemoveOnDrop(NonNull<u8>);
        impl Drop for RemoveOnDrop {
            fn drop(&mut self) {
                let _ = IN_PROGRESS_IDS
                    .try_with(|in_progress_ids| in_progress_ids.borrow_mut().map.remove(&self.0));
            }
        }
        let id = self.id().cast();
        let (inserted, numeric_id) = IN_PROGRESS_IDS.with(|in_progress_ids| {
            let mut in_progress_ids = in_progress_ids.borrow_mut();
            let InProgressIds {
                map,
                next_numeric_id,
            } = &mut *in_progress_ids;
            if map.is_empty() {
                *next_numeric_id = 1;
            }
            match map.entry(id) {
                Entry::Vacant(entry) => {
                    let numeric_id = *next_numeric_id;
                    *next_numeric_id += 1;
                    entry.insert(numeric_id);
                    (true, numeric_id)
                }
                Entry::Occupied(entry) => (false, *entry.get()),
            }
        });
        let remove_on_drop = if inserted {
            Some(RemoveOnDrop(id))
        } else {
            None
        };
        let mut debug_helper = f.debug_tuple("IdRef");
        debug_helper.field(&NumericId(numeric_id));
        if inserted {
            debug_helper.field(self.get());
        } else {
            debug_helper.field(&Omitted);
        }
        let retval = debug_helper.finish();
        std::mem::drop(remove_on_drop);
        retval
    }
}

impl<'g, T> IdRef<'g, T> {
    pub fn id(self) -> NonNull<T> {
        self.0.into()
    }
    pub fn get(self) -> &'g T {
        self.0
    }
}

impl<'g, T> Deref for IdRef<'g, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<'g, T> Eq for IdRef<'g, T> {}

impl<'g, T> Copy for IdRef<'g, T> {}

impl<'g, T> Clone for IdRef<'g, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'g, T> PartialEq for IdRef<'g, T> {
    fn eq(&self, rhs: &IdRef<'g, T>) -> bool {
        self.id() == rhs.id()
    }
}

impl<'g, T> Hash for IdRef<'g, T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.id().hash(h)
    }
}

/// allocate value from `GlobalState`
pub trait Allocate<'g, T> {
    #[doc(hidden)]
    fn alloc_private(&'g self, _private: Private, value: T) -> &'g T;
    /// allocate value from `GlobalState`
    #[must_use]
    fn alloc(&'g self, value: T) -> IdRef<'g, T> {
        IdRef(self.alloc_private(Private::new(), value))
    }
}

#[repr(transparent)]
pub struct Interned<'g, T: ?Sized + Eq + Hash>(&'g T);

impl<T: ?Sized + Eq + Hash> Eq for Interned<'_, T> {}

impl<T: ?Sized + Eq + Hash> Copy for Interned<'_, T> {}

impl<T: ?Sized + Eq + Hash> Clone for Interned<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized + Eq + Hash> PartialEq for Interned<'_, T> {
    fn eq(&self, rhs: &Self) -> bool {
        std::ptr::eq(self.0, rhs.0)
    }
}

impl<T: ?Sized + Eq + Hash> Hash for Interned<'_, T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        (self.0 as *const T).hash(h)
    }
}

impl<T: ?Sized + Eq + Hash> Deref for Interned<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T: ?Sized + Eq + Hash> AsRef<T> for Interned<'_, T> {
    fn as_ref(&self) -> &T {
        self.0
    }
}

impl<'g, T: ?Sized + Eq + Hash> Interned<'g, T> {
    pub fn get(self) -> &'g T {
        self.0
    }
}

impl<T: ?Sized + Eq + Hash + fmt::Debug> fmt::Debug for Interned<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ?Sized + Eq + Hash + fmt::Display> fmt::Display for Interned<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub trait Intern<'g, T: ?Sized + Eq + Hash> {
    #[doc(hidden)]
    fn intern_alloc(&'g self, _private: Private, value: &T) -> &'g T;
    #[doc(hidden)]
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g T>>;
    #[must_use]
    fn intern<'a>(&'g self, value: &'a T) -> Interned<'g, T> {
        let mut hashtable = self.get_hashtable(Private::new()).borrow_mut();
        if let Some(retval) = hashtable.get(value) {
            Interned(retval)
        } else {
            let retval = self.intern_alloc(Private::new(), value);
            let inserted = hashtable.insert(retval);
            assert!(inserted);
            Interned(retval)
        }
    }
}

impl<'g> Intern<'g, str> for GlobalState<'g> {
    fn intern_alloc(&'g self, _private: Private, value: &str) -> &'g str {
        self.string_byte_arena.alloc_str(value)
    }
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g str>> {
        &self.string_hashtable
    }
}

/// a debug location
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Location<'g> {
    /// the source file name
    pub file: Interned<'g, str>,
    /// the line number
    pub line: u32,
    /// the column number
    pub column: u32,
}

impl<'g> Intern<'g, Location<'g>> for GlobalState<'g> {
    fn intern_alloc(&'g self, _private: Private, value: &Location<'g>) -> &'g Location<'g> {
        self.location_arena.alloc(*value)
    }
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g Location<'g>>> {
        &self.location_hashtable
    }
}

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
pub(crate) trait CodeIO<'g> {
    /// the list of SSA value definitions that are the results of executing `self`, or `Uninhabited` if `self` doesn't return
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]>;
    /// the list of SSA values that are the arguments for `self`
    fn arguments(&self) -> &[ValueUse<'g>];
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum SimpleInstructionKind {}

#[derive(Debug)]
pub struct BreakBlock<'g> {
    pub block: IdRef<'g, Block<'g>>,
    pub block_results: Vec<ValueUse<'g>>,
}

impl<'g> CodeIO<'g> for BreakBlock<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        Uninhabited
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &self.block_results
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct LoopHeader<'g> {
    pub argument_definitions: Vec<ValueDefinition<'g>>,
}

impl<'g> CodeIO<'g> for LoopHeader<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        Inhabited(&self.argument_definitions)
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &[]
    }
}

#[derive(Debug)]
pub struct Block<'g> {
    pub body: OnceCell<Vec<Instruction<'g>>>,
    pub result_definitions: Inhabitable<Vec<ValueDefinition<'g>>>,
}

impl<'g> Allocate<'g, Block<'g>> for GlobalState<'g> {
    fn alloc_private(&'g self, _private: Private, value: Block<'g>) -> &'g Block<'g> {
        self.block_arena.alloc(value)
    }
}

impl<'g> Block<'g> {
    pub fn id(&'g self) -> NonNull<Self> {
        self.into()
    }
}

impl<'g> CodeIO<'g> for Block<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        self.result_definitions.as_deref()
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &[]
    }
}

#[derive(Debug)]
pub struct Loop<'g> {
    pub arguments: Vec<ValueUse<'g>>,
    pub header: LoopHeader<'g>,
    pub body: IdRef<'g, Block<'g>>,
}

impl<'g> Loop<'g> {
    pub fn id(&'g self) -> NonNull<Self> {
        self.into()
    }
}

impl<'g> Allocate<'g, Loop<'g>> for GlobalState<'g> {
    fn alloc_private(&'g self, _private: Private, value: Loop<'g>) -> &'g Loop<'g> {
        self.loop_arena.alloc(value)
    }
}

impl<'g> CodeIO<'g> for Loop<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        self.body.results()
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &self.arguments
    }
}

#[derive(Debug)]
pub struct ContinueLoop<'g> {
    pub target_loop: IdRef<'g, Loop<'g>>,
    pub block_arguments: Vec<ValueUse<'g>>,
}

impl<'g> CodeIO<'g> for ContinueLoop<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        Uninhabited
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &self.block_arguments
    }
}

#[derive(Debug)]
pub struct BinaryALUInstruction<'g> {
    pub arguments: [ValueUse<'g>; 2],
    pub result: ValueDefinition<'g>,
}

impl<'g> CodeIO<'g> for BinaryALUInstruction<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        Inhabited(std::slice::from_ref(&self.result))
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &self.arguments
    }
}

#[derive(Debug)]
pub enum SimpleInstruction<'g> {
    Add(BinaryALUInstruction<'g>),
    // TODO: implement
}

impl<'g> CodeIO<'g> for SimpleInstruction<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        match self {
            SimpleInstruction::Add(binary_alu_instruction) => binary_alu_instruction.results(),
        }
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        match self {
            SimpleInstruction::Add(binary_alu_instruction) => binary_alu_instruction.arguments(),
        }
    }
}

#[derive(Debug)]
pub struct BranchInstruction<'g> {
    pub variable: ValueUse<'g>,
    pub targets: Vec<(Interned<'g, Const<'g>>, BreakBlock<'g>)>,
}

impl<'g> CodeIO<'g> for BranchInstruction<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        Inhabited(&[])
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        std::slice::from_ref(&self.variable)
    }
}

/// variable part of `Instruction`
#[derive(Debug)]
pub enum InstructionData<'g> {
    Simple(SimpleInstruction<'g>),
    Block(IdRef<'g, Block<'g>>),
    Loop(IdRef<'g, Loop<'g>>),
    ContinueLoop(ContinueLoop<'g>),
    BreakBlock(BreakBlock<'g>),
    Branch(BranchInstruction<'g>),
}

impl<'g> CodeIO<'g> for InstructionData<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        match self {
            InstructionData::Simple(v) => v.results(),
            InstructionData::Block(v) => v.results(),
            InstructionData::Loop(v) => v.results(),
            InstructionData::ContinueLoop(v) => v.results(),
            InstructionData::BreakBlock(v) => v.results(),
            InstructionData::Branch(v) => v.results(),
        }
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        match self {
            InstructionData::Simple(v) => v.arguments(),
            InstructionData::Block(v) => v.arguments(),
            InstructionData::Loop(v) => v.arguments(),
            InstructionData::ContinueLoop(v) => v.arguments(),
            InstructionData::BreakBlock(v) => v.arguments(),
            InstructionData::Branch(v) => v.arguments(),
        }
    }
}

#[derive(Debug)]
pub struct Instruction<'g> {
    pub location: Option<Interned<'g, Location<'g>>>,
    pub data: InstructionData<'g>,
}

impl<'g> CodeIO<'g> for Instruction<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        self.data.results()
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        self.data.arguments()
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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Void {}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum OpaqueType<'g> {
    // TODO: implement
    #[doc(hidden)]
    _Unimplemented(&'g (), Void),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Type<'g> {
    Integer {
        integer_type: IntegerType,
    },
    Float {
        float_type: FloatType,
    },
    Bool,
    Pointer {
        pointee: Interned<'g, Type<'g>>,
    },
    Vector {
        len: usize,
        element: Interned<'g, Type<'g>>,
    },
    Matrix {
        columns: usize,
        rows: usize,
        element: Interned<'g, Type<'g>>,
    },
    VariableVector {
        element: Interned<'g, Type<'g>>,
    },
    Opaque {
        opaque_type: OpaqueType<'g>,
    },
}

impl<'g> Intern<'g, Type<'g>> for GlobalState<'g> {
    fn intern_alloc(&'g self, _private: Private, value: &Type<'g>) -> &'g Type<'g> {
        self.type_arena.alloc(value.clone())
    }
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g Type<'g>>> {
        &self.type_hashtable
    }
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
pub struct ConstVector<'g> {
    element_type: Interned<'g, Type<'g>>,
    elements: Vec<Interned<'g, Const<'g>>>,
}

impl<'g> ConstVector<'g> {
    pub fn new(elements: Vec<Interned<'g, Const<'g>>>, global_state: &'g GlobalState<'g>) -> Self {
        let mut iter = elements.iter();
        let element_type = iter
            .next()
            .expect("vector must have non-zero size")
            .get()
            .get_type(global_state);
        for element in iter {
            assert_eq!(
                element.get().get_type(global_state),
                element_type,
                "vector must have consistent type"
            );
        }
        ConstVector {
            element_type,
            elements,
        }
    }
    pub fn element_type(&self) -> Interned<'g, Type<'g>> {
        self.element_type
    }
    pub fn elements(&self) -> &[Interned<'g, Const<'g>>] {
        &self.elements
    }
    pub fn get_type(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type> {
        global_state.intern(&Type::Vector {
            element: self.element_type,
            len: self.elements.len(),
        })
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Const<'g> {
    Integer(ConstInteger),
    Float(ConstFloat),
    Bool(bool),
    Vector(ConstVector<'g>),
    // FIXME: add Matrix
    Undef(Interned<'g, Type<'g>>),
}

impl<'g> Intern<'g, Const<'g>> for GlobalState<'g> {
    fn intern_alloc(&'g self, _private: Private, value: &Const<'g>) -> &'g Const<'g> {
        self.const_arena.alloc(value.clone())
    }
    fn get_hashtable(&'g self, _private: Private) -> &'g RefCell<HashSet<&'g Const<'g>>> {
        &self.const_hashtable
    }
}

impl<'g> Const<'g> {
    pub fn get_type(&self, global_state: &'g GlobalState<'g>) -> Interned<'g, Type> {
        match *self {
            Const::Integer(ConstInteger { integer_type, .. }) => {
                global_state.intern(&Type::Integer { integer_type })
            }
            Const::Float(const_float) => global_state.intern(&Type::Float {
                float_type: const_float.get_type(),
            }),
            Const::Bool(_) => global_state.intern(&Type::Bool),
            Const::Vector(ref const_vector) => const_vector.get_type(global_state),
            Const::Undef(ref retval) => retval.clone(),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ValueDefinition<'g> {
    value: IdRef<'g, Value<'g>>,
}

impl<'g> ValueDefinition<'g> {
    pub fn new(
        value_type: Interned<'g, Type<'g>>,
        name: Interned<'g, str>,
        global_state: &'g GlobalState<'g>,
    ) -> ValueDefinition<'g> {
        ValueDefinition {
            value: global_state.alloc(Value {
                value_type,
                name,
                const_value: Cell::new(None),
            }),
        }
    }
    pub fn define_as_const(
        self,
        const_value: Interned<'g, Const<'g>>,
        global_state: &'g GlobalState<'g>,
    ) -> IdRef<'g, Value<'g>> {
        let Self { value } = self;
        assert_eq!(value.value_type, const_value.get().get_type(global_state));
        assert!(
            value.const_value.replace(Some(const_value)).is_none(),
            "invalid Value state"
        );
        value
    }
    pub fn value(&self) -> IdRef<'g, Value<'g>> {
        self.value
    }
}

impl<'g> Deref for ValueDefinition<'g> {
    type Target = IdRef<'g, Value<'g>>;
    fn deref(&self) -> &IdRef<'g, Value<'g>> {
        &self.value
    }
}

#[derive(Debug)]
pub struct Value<'g> {
    pub value_type: Interned<'g, Type<'g>>,
    pub name: Interned<'g, str>,
    pub const_value: Cell<Option<Interned<'g, Const<'g>>>>,
}

impl<'g> Allocate<'g, Value<'g>> for GlobalState<'g> {
    fn alloc_private(&'g self, _private: Private, value: Value<'g>) -> &'g Value<'g> {
        self.value_arena.alloc(value)
    }
}

impl<'g> Value<'g> {
    pub fn id(&'g self) -> NonNull<Self> {
        self.into()
    }
    pub fn from_const(
        const_value: Interned<'g, Const<'g>>,
        name: Interned<'g, str>,
        global_state: &'g GlobalState<'g>,
    ) -> IdRef<'g, Value<'g>> {
        global_state.alloc(Value {
            name,
            value_type: const_value.get().get_type(global_state),
            const_value: Cell::new(Some(const_value)),
        })
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ValueUse<'g> {
    value: IdRef<'g, Value<'g>>,
}

impl<'g> ValueUse<'g> {
    pub fn new(value: IdRef<'g, Value<'g>>) -> Self {
        Self { value }
    }
    pub fn value(&self) -> IdRef<'g, Value<'g>> {
        self.value
    }
}

impl<'g> Deref for ValueUse<'g> {
    type Target = IdRef<'g, Value<'g>>;
    fn deref(&self) -> &IdRef<'g, Value<'g>> {
        &self.value
    }
}
