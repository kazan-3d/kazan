// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

//! intermediate representation for executable code

use crate::debug;
use crate::value::ValueDefinition;
use crate::value::ValueUse;
use std::hash::Hash;
use std::ops::Deref;
use std::ops::DerefMut;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Inhabitable<T> {
    Inhabited(T),
    Uninhabited,
}

pub use Inhabitable::Inhabited;
pub use Inhabitable::Uninhabited;

impl<T> Inhabitable<T> {
    pub fn as_ref(&self) -> Inhabitable<&T> {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    pub fn as_mut(&mut self) -> Inhabitable<&mut T> {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    pub fn as_deref(&self) -> Inhabitable<&T::Target>
    where
        T: Deref,
    {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    pub fn as_deref_mut(&mut self) -> Inhabitable<&mut T::Target>
    where
        T: DerefMut,
    {
        match self {
            Inhabited(v) => Inhabited(v),
            Uninhabited => Uninhabited,
        }
    }
    pub fn inhabited(self) -> Option<T> {
        match self {
            Inhabited(v) => Some(v),
            Uninhabited => None,
        }
    }
}

pub trait CodeIO {
    fn results(&self) -> Inhabitable<&[ValueDefinition]>;
    fn arguments(&self) -> &[ValueUse];
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SimpleInstructionKind {}

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
pub struct BlockHeader {
    pub argument_definitions: Vec<ValueDefinition>,
}

impl CodeIO for BlockHeader {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        Inhabited(&self.argument_definitions)
    }
    fn arguments(&self) -> &[ValueUse] {
        &[]
    }
}

#[derive(Debug)]
pub struct Block {
    pub arguments: Vec<ValueUse>,
    pub header: BlockHeader,
    pub body: Vec<Instruction>,
    pub result_definitions: Inhabitable<Vec<ValueDefinition>>,
}

impl CodeIO for Block {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        self.result_definitions.as_deref()
    }
    fn arguments(&self) -> &[ValueUse] {
        &self.arguments
    }
}

#[derive(Debug)]
pub struct Loop {
    pub block: Rc<Block>,
}

impl CodeIO for Loop {
    fn results(&self) -> Inhabitable<&[ValueDefinition]> {
        self.block.results()
    }
    fn arguments(&self) -> &[ValueUse] {
        self.block.arguments()
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
    pub debug_location: debug::Location,
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
