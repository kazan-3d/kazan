// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![warn(missing_docs)]

//! Shader Compiler Intermediate Representation

mod consts;
mod debug_info;
mod global_state;
mod types;
mod values;

pub mod from_text;
pub mod prelude;

pub use crate::consts::*;
pub use crate::debug_info::*;
pub use crate::from_text::FromText;
pub use crate::global_state::*;
pub use crate::types::*;
pub use crate::values::*;
pub use once_cell::unsync::OnceCell;
use std::hash::Hash;

/// code structure input/output
pub trait CodeIO<'g> {
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

impl<'g> Id<'g> for Block<'g> {}

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

impl<'g> Id<'g> for Loop<'g> {}

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
