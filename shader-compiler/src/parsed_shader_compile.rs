// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use super::{Context, IdKind, Ids, ParsedShader, ParsedShaderFunction};
use shader_compiler_backend::{
    types::TypeBuilder, BuildableBasicBlock, DetachedBuilder, Function, Module,
};
use spirv_parser::Decoration;
use spirv_parser::{FunctionControl, IdRef, IdResult, IdResultType, Instruction};
use std::cell::Cell;
use std::collections::hash_map;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;

pub(crate) trait ParsedShaderCompile<'ctx, C: shader_compiler_backend::Context<'ctx>> {
    fn compile(
        self,
        frontend_context: &mut Context,
        backend_context: &'ctx C,
        module: &mut C::Module,
        function_name_prefix: &str,
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

struct FunctionInstruction {
    id_result_type: IdResultType,
    id_result: IdResult,
    function_control: FunctionControl,
    function_type: IdRef,
}

struct FunctionState<'ctx, C: shader_compiler_backend::Context<'ctx>> {
    function_instruction: FunctionInstruction,
    instructions: Vec<Instruction>,
    decorations: Vec<Decoration>,
    backend_function: Cell<Option<C::Function>>,
    backend_function_value: C::Value,
}

struct GetOrAddFunctionState<'ctx, 'tb, 'fnp, C: shader_compiler_backend::Context<'ctx>>
where
    C::TypeBuilder: 'tb,
{
    reachable_functions: HashMap<IdRef, Rc<FunctionState<'ctx, C>>>,
    type_builder: &'tb C::TypeBuilder,
    function_name_prefix: &'fnp str,
}

impl<'ctx, 'tb, 'fnp, C: shader_compiler_backend::Context<'ctx>>
    GetOrAddFunctionState<'ctx, 'tb, 'fnp, C>
{
    fn call(
        &mut self,
        reachable_functions_worklist: &mut Vec<IdRef>,
        ids: &mut Ids<'ctx, C>,
        module: &mut C::Module,
        function_id: IdRef,
    ) -> Rc<FunctionState<'ctx, C>> {
        match self.reachable_functions.entry(function_id) {
            hash_map::Entry::Occupied(v) => v.get().clone(),
            hash_map::Entry::Vacant(v) => {
                reachable_functions_worklist.push(function_id);
                let ParsedShaderFunction {
                    instructions,
                    decorations,
                } = match &mut ids[function_id].kind {
                    IdKind::Function(function) => function.take().unwrap(),
                    _ => unreachable!("id is not a function"),
                };
                let function_instruction = match instructions.get(0) {
                    Some(&Instruction::Function {
                        id_result_type,
                        id_result,
                        ref function_control,
                        function_type,
                    }) => FunctionInstruction {
                        id_result_type,
                        id_result,
                        function_control: function_control.clone(),
                        function_type,
                    },
                    _ => unreachable!("missing OpFunction"),
                };
                for decoration in &decorations {
                    match decoration {
                        _ => unreachable!(
                            "unimplemented function decoration: {:?} on {}",
                            decoration, function_id
                        ),
                    }
                }
                let function_type = match &ids[function_instruction.function_type].kind {
                    IdKind::FunctionType {
                        return_type,
                        arguments,
                    } => {
                        let return_type = match return_type {
                            None => None,
                            Some(v) => unimplemented!(),
                        };
                        let arguments: Vec<_> = arguments
                            .iter()
                            .enumerate()
                            .map(|(argument_index, argument)| unimplemented!())
                            .collect();
                        self.type_builder.build_function(&arguments, return_type)
                    }
                    _ => unreachable!("not a function type"),
                };
                let backend_function = module.add_function(
                    &format!("{}{}", self.function_name_prefix, function_id.0),
                    function_type,
                );
                let backend_function_value = backend_function.as_value();
                v.insert(Rc::new(FunctionState {
                    function_instruction,
                    instructions,
                    decorations,
                    backend_function: Cell::new(Some(backend_function)),
                    backend_function_value,
                }))
                .clone()
            }
        }
    }
}

impl<'ctx, C: shader_compiler_backend::Context<'ctx>> ParsedShaderCompile<'ctx, C>
    for ParsedShader<'ctx, C>
{
    fn compile(
        self,
        frontend_context: &mut Context,
        backend_context: &'ctx C,
        module: &mut C::Module,
        function_name_prefix: &str,
    ) -> C::Function {
        let ParsedShader {
            mut ids,
            main_function_id,
            interface_variables,
            execution_modes,
            workgroup_size,
        } = self;
        let type_builder = backend_context.create_type_builder();
        let mut reachable_functions_worklist = Vec::new();
        let mut get_or_add_function_state = GetOrAddFunctionState {
            reachable_functions: HashMap::new(),
            type_builder: &type_builder,
            function_name_prefix,
        };
        let mut get_or_add_function = |reachable_functions_worklist: &mut Vec<IdRef>,
                                       ids: &mut Ids<'ctx, C>,
                                       module: &mut C::Module,
                                       function_id: IdRef| {
            get_or_add_function_state.call(reachable_functions_worklist, ids, module, function_id)
        };
        let get_or_add_basic_block =
            |ids: &mut Ids<'ctx, C>, label_id: IdRef, backend_function: &mut C::Function| {
                if let IdKind::BasicBlock { basic_block, .. } = &ids[label_id].kind {
                    return basic_block.clone();
                }
                let buildable_basic_block =
                    backend_function.append_new_basic_block(Some(&format!("L{}", label_id.0)));
                let basic_block = buildable_basic_block.as_basic_block();
                ids[label_id].set_kind(IdKind::BasicBlock {
                    buildable_basic_block: Some(buildable_basic_block),
                    basic_block: basic_block.clone(),
                });
                basic_block
            };
        get_or_add_function(
            &mut reachable_functions_worklist,
            &mut ids,
            module,
            main_function_id,
        );
        while let Some(function_id) = reachable_functions_worklist.pop() {
            let function_state = get_or_add_function(
                &mut reachable_functions_worklist,
                &mut ids,
                module,
                function_id,
            );
            let mut backend_function = function_state.backend_function.replace(None).unwrap();
            enum BasicBlockState<'ctx, C: shader_compiler_backend::Context<'ctx>> {
                Detached {
                    builder: C::DetachedBuilder,
                },
                Attached {
                    builder: C::AttachedBuilder,
                    current_label: IdRef,
                },
            }
            let mut current_basic_block: BasicBlockState<C> = BasicBlockState::Detached {
                builder: backend_context.create_builder(),
            };
            for instruction in &function_state.instructions {
                match current_basic_block {
                    BasicBlockState::Attached {
                        builder,
                        current_label,
                    } => match instruction {
                        _ => unimplemented!("unimplemented instruction:\n{}", instruction),
                    },
                    BasicBlockState::Detached { builder } => match instruction {
                        Instruction::Function { .. } => {
                            current_basic_block = BasicBlockState::Detached { builder };
                        }
                        Instruction::Label { id_result } => {
                            ids[id_result.0].assert_no_decorations(id_result.0);
                            get_or_add_basic_block(&mut ids, id_result.0, &mut backend_function);
                            let buildable_basic_block = match ids[id_result.0].kind {
                                IdKind::BasicBlock {
                                    ref mut buildable_basic_block,
                                    ..
                                } => buildable_basic_block.take().expect("duplicate OpLabel"),
                                _ => unreachable!(),
                            };
                            current_basic_block = BasicBlockState::Attached {
                                builder: builder.attach(buildable_basic_block),
                                current_label: id_result.0,
                            };
                        }
                        _ => unimplemented!("unimplemented instruction:\n{}", instruction),
                    },
                }
            }
        }
        unimplemented!()
    }
}