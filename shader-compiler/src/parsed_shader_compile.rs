// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use super::{Context, IdKind, IdProperties, ParsedShader, ParsedShaderFunction};
use spirv_parser::{FunctionControl, IdRef, IdResult, IdResultType, Instruction};
use std::collections::hash_map;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub(crate) trait ParsedShaderCompile {
    fn compile<'a, C: shader_compiler_backend::Context<'a>>(
        self,
        frontend_context: &mut Context,
        backend_context: &C,
        module: &mut C::Module,
    ) -> C::Function;
}

struct Worklist<T> {
    set: HashSet<T>,
    list: Vec<T>,
}

impl<T: Eq + Hash + Clone> Worklist<T> {
    fn get_next(&mut self) -> Option<T> {
        self.list.pop()
    }
    fn add(&mut self, v: T) -> bool {
        if self.set.insert(v.clone()) {
            self.list.push(v);
            true
        } else {
            false
        }
    }
}

impl<T: Eq + Hash + Clone> Default for Worklist<T> {
    fn default() -> Self {
        Self {
            set: HashSet::new(),
            list: Vec::new(),
        }
    }
}

impl ParsedShaderCompile for ParsedShader {
    fn compile<'a, C: shader_compiler_backend::Context<'a>>(
        self,
        frontend_context: &mut Context,
        backend_context: &C,
        module: &mut C::Module,
    ) -> C::Function {
        let ParsedShader {
            mut ids,
            main_function_id,
            interface_variables,
            execution_modes,
            workgroup_size,
        } = self;
        let mut reachable_functions = HashMap::new();
        let mut reachable_function_worklist = Worklist::default();
        reachable_function_worklist.add(main_function_id);
        while let Some(function_id) = reachable_function_worklist.get_next() {
            let function = match &mut ids[function_id].kind {
                IdKind::Function(function) => function.take().unwrap(),
                _ => unreachable!("id is not a function"),
            };
            let mut function = match reachable_functions.entry(function_id) {
                hash_map::Entry::Vacant(entry) => entry.insert(function),
                _ => unreachable!(),
            };
            let (function_instruction, instructions) = function
                .instructions
                .split_first()
                .expect("missing OpFunction");
            struct FunctionInstruction {
                id_result_type: IdResultType,
                id_result: IdResult,
                function_control: FunctionControl,
                function_type: IdRef,
            }
            let function_instruction = match *function_instruction {
                Instruction::Function {
                    id_result_type,
                    id_result,
                    ref function_control,
                    function_type,
                } => FunctionInstruction {
                    id_result_type,
                    id_result,
                    function_control: function_control.clone(),
                    function_type,
                },
                _ => unreachable!("missing OpFunction"),
            };
            let mut current_basic_block: Option<IdRef> = None;
            for instruction in instructions {
                if let Some(basic_block) = current_basic_block {
                    match instruction {
                        _ => unimplemented!("unimplemented instruction:\n{}", instruction),
                    }
                } else {
                    match instruction {
                        Instruction::Label { id_result } => {
                            ids[id_result.0].assert_no_decorations(id_result.0);
                            current_basic_block = Some(id_result.0);
                        }
                        _ => unimplemented!("unimplemented instruction:\n{}", instruction),
                    }
                }
            }
        }
        unimplemented!()
    }
}
