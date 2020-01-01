// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::OnceCell;

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
