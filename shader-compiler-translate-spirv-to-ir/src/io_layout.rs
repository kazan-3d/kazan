// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{errors::InvalidVectorComponentCount, types::IntegerType, TranslationResult};
use alloc::{boxed::Box, rc::Rc, vec::Vec};
use core::cell::Cell;
use shader_compiler_ir::{Alignment, FloatType, ValueUse};

pub(crate) const COMPONENT_SIZE_IN_BYTES: u32 = 4;
pub(crate) const LOCATION_SIZE_IN_COMPONENTS: u32 = 4;
pub(crate) const LOCATION_SIZE_IN_BYTES: u32 =
    COMPONENT_SIZE_IN_BYTES * LOCATION_SIZE_IN_COMPONENTS;

pub(crate) fn io_interface_block_alignment() -> Alignment {
    Alignment::new(LOCATION_SIZE_IN_BYTES).expect("known to be a valid alignment")
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) enum IOLayoutScalar {
    Integer(IntegerType),
    Float(FloatType),
}

impl From<IntegerType> for IOLayoutScalar {
    fn from(v: IntegerType) -> Self {
        Self::Integer(v)
    }
}

impl From<FloatType> for IOLayoutScalar {
    fn from(v: FloatType) -> Self {
        Self::Float(v)
    }
}

impl IOLayoutScalar {
    pub(crate) fn io_component_count(self) -> u32 {
        use shader_compiler_ir::IntegerType as IRIntegerType;
        match self {
            Self::Integer(integer_type) => match integer_type.ir_type {
                IRIntegerType::Int8
                | IRIntegerType::Int16
                | IRIntegerType::Int32
                | IRIntegerType::RelaxedInt32 => 1,
                IRIntegerType::Int64 => 2,
            },
            Self::Float(FloatType::Float16)
            | Self::Float(FloatType::Float32)
            | Self::Float(FloatType::RelaxedFloat32) => 1,
            Self::Float(FloatType::Float64) => 2,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct IOLayoutVector {
    pub(crate) component_type: IOLayoutScalar,
    pub(crate) component_count: u32,
}

impl From<IOLayoutScalar> for IOLayoutVector {
    fn from(component_type: IOLayoutScalar) -> Self {
        Self {
            component_type,
            component_count: 1,
        }
    }
}

impl IOLayoutVector {
    pub(crate) fn new(
        component_type: IOLayoutScalar,
        component_count: u32,
    ) -> TranslationResult<Self> {
        match component_count {
            1 | 2 | 3 | 4 => Ok(Self {
                component_type,
                component_count,
            }),
            _ => Err(InvalidVectorComponentCount { component_count }.into()),
        }
    }
    pub(crate) fn location_count(self) -> u32 {
        match (
            self.component_count,
            self.component_type.io_component_count(),
        ) {
            (1, 1) | (2, 1) | (3, 1) | (4, 1) => 1,
            (1, 2) | (2, 2) => 1,
            (3, 2) | (4, 2) => 2,
            _ => unreachable!("invalid IOLayoutVector: {:?}", self),
        }
    }
    pub(crate) fn is_valid_start_io_component(self, start_io_component: u32) -> bool {
        match (
            self.component_type.io_component_count(),
            self.component_count,
        ) {
            (1, 1) => start_io_component < 4,
            (1, 2) => start_io_component < 3,
            (1, 3) => start_io_component < 2,
            (1, 4) => start_io_component == 0,
            (2, 1) => start_io_component == 0 || start_io_component == 2,
            (2, 2) | (2, 3) | (2, 4) => start_io_component == 0,
            _ => unreachable!("invalid IOLayoutVector: {:?}", self),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct IOLayoutVectorAtComponent {
    pub(crate) vector_type: IOLayoutVector,
    pub(crate) start_io_component: u32,
}

impl IOLayoutVectorAtComponent {
    pub(crate) fn new(vector_type: IOLayoutVector, start_io_component: u32) -> Result<Self, ()> {
        if vector_type.is_valid_start_io_component(start_io_component) {
            Ok(Self {
                vector_type,
                start_io_component,
            })
        } else {
            Err(())
        }
    }
    pub(crate) fn location_count(self) -> u32 {
        self.vector_type.location_count()
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct IOLayoutMatrix {
    pub(crate) column_type: IOLayoutVector,
    pub(crate) column_count: u32,
}

impl IOLayoutMatrix {
    pub(crate) fn new(column_type: IOLayoutVector, column_count: u32) -> Result<Self, ()> {
        if (2..=4).contains(&column_count) {
            Ok(Self {
                column_type,
                column_count,
            })
        } else {
            Err(())
        }
    }
    pub(crate) fn location_count(self) -> u32 {
        self.column_type.location_count() * self.column_count
    }
}

#[derive(Clone, Debug)]
pub(crate) enum IOLayoutArrayElement {
    Vector(IOLayoutVector),
    Matrix(IOLayoutMatrix),
    Array(Box<IOLayoutArray>),
}

impl From<IOLayoutVector> for IOLayoutArrayElement {
    fn from(v: IOLayoutVector) -> Self {
        Self::Vector(v)
    }
}

impl From<IOLayoutMatrix> for IOLayoutArrayElement {
    fn from(v: IOLayoutMatrix) -> Self {
        Self::Matrix(v)
    }
}

impl From<Box<IOLayoutArray>> for IOLayoutArrayElement {
    fn from(v: Box<IOLayoutArray>) -> Self {
        Self::Array(v)
    }
}

impl From<IOLayoutArray> for IOLayoutArrayElement {
    fn from(v: IOLayoutArray) -> Self {
        Box::new(v).into()
    }
}

impl IOLayoutArrayElement {
    pub(crate) fn location_count(&self) -> u32 {
        match self {
            Self::Vector(v) => v.location_count(),
            Self::Matrix(v) => v.location_count(),
            Self::Array(v) => v.location_count(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct IOLayoutArray {
    pub(crate) element_type: IOLayoutArrayElement,
    pub(crate) len: u32,
}

impl IOLayoutArray {
    pub(crate) fn new(element_type: IOLayoutArrayElement, len: u32) -> Self {
        Self { element_type, len }
    }
    pub(crate) fn location_count(&self) -> u32 {
        self.element_type.location_count() * self.len
    }
}

#[derive(Clone, Debug)]
pub(crate) enum IOLayoutNonStructType {
    Vector(IOLayoutVectorAtComponent),
    Matrix(IOLayoutMatrix),
    Array(IOLayoutArray),
}

impl From<IOLayoutVectorAtComponent> for IOLayoutNonStructType {
    fn from(v: IOLayoutVectorAtComponent) -> Self {
        Self::Vector(v)
    }
}

impl From<IOLayoutMatrix> for IOLayoutNonStructType {
    fn from(v: IOLayoutMatrix) -> Self {
        Self::Matrix(v)
    }
}

impl From<IOLayoutArray> for IOLayoutNonStructType {
    fn from(v: IOLayoutArray) -> Self {
        Self::Array(v)
    }
}

impl IOLayoutNonStructType {
    pub(crate) fn location_count(&self) -> u32 {
        match self {
            Self::Vector(v) => v.location_count(),
            Self::Matrix(v) => v.location_count(),
            Self::Array(v) => v.location_count(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct IOLayoutNonStruct {
    pub(crate) start_location: u32,
    pub(crate) io_layout_type: IOLayoutNonStructType,
    pub(crate) interface_block_field_index: Cell<Option<u32>>,
}

#[derive(Clone, Debug)]
pub(crate) struct IOLayoutStruct {
    pub(crate) members: Vec<IOLayout>,
}

#[derive(Clone, Debug)]
pub(crate) enum IOLayout {
    NonStruct(Rc<IOLayoutNonStruct>),
    Struct(Rc<IOLayoutStruct>),
}

impl From<Rc<IOLayoutNonStruct>> for IOLayout {
    fn from(v: Rc<IOLayoutNonStruct>) -> Self {
        Self::NonStruct(v)
    }
}

impl From<Rc<IOLayoutStruct>> for IOLayout {
    fn from(v: Rc<IOLayoutStruct>) -> Self {
        Self::Struct(v)
    }
}

impl From<IOLayoutNonStruct> for IOLayout {
    fn from(v: IOLayoutNonStruct) -> Self {
        Rc::new(v).into()
    }
}

impl From<IOLayoutStruct> for IOLayout {
    fn from(v: IOLayoutStruct) -> Self {
        Rc::new(v).into()
    }
}
