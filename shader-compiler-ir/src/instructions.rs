// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::BreakBlock;
use crate::ContinueLoop;

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

impl<'g> From<SimpleInstruction<'g>> for InstructionData<'g> {
    fn from(v: SimpleInstruction<'g>) -> InstructionData<'g> {
        InstructionData::Simple(v)
    }
}

impl<'g> From<ContinueLoop<'g>> for InstructionData<'g> {
    fn from(v: ContinueLoop<'g>) -> InstructionData<'g> {
        InstructionData::ContinueLoop(v)
    }
}

impl<'g> From<BreakBlock<'g>> for InstructionData<'g> {
    fn from(v: BreakBlock<'g>) -> InstructionData<'g> {
        InstructionData::BreakBlock(v)
    }
}

impl<'g> From<BranchInstruction<'g>> for InstructionData<'g> {
    fn from(v: BranchInstruction<'g>) -> InstructionData<'g> {
        InstructionData::Branch(v)
    }
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

impl<'g> Instruction<'g> {
    pub fn new(
        location: Option<Interned<'g, Location<'g>>>,
        data: impl Into<InstructionData<'g>>,
    ) -> Self {
        Self {
            location: location,
            data: data.into(),
        }
    }
    pub fn with_location(
        location: Interned<'g, Location<'g>>,
        data: impl Into<InstructionData<'g>>,
    ) -> Self {
        Self {
            location: Some(location),
            data: data.into(),
        }
    }
    pub fn with_internable_location(
        location: impl Internable<'g, Interned = Location<'g>>,
        data: impl Into<InstructionData<'g>>,
        global_state: &'g GlobalState<'g>,
    ) -> Self {
        Self {
            location: Some(location.intern(global_state)),
            data: data.into(),
        }
    }
    pub fn without_location(data: impl Into<InstructionData<'g>>) -> Self {
        Self {
            location: None,
            data: data.into(),
        }
    }
}

impl<'g> CodeIO<'g> for Instruction<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        self.data.results()
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        self.data.arguments()
    }
}
