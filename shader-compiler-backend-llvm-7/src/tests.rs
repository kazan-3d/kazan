// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#[cfg(test)]
// we have a tests module inside a tests module to have rls parse this tests.rs file
#[allow(clippy::module_inception)]
mod tests {
    use shader_compiler_backend::types::TypeBuilder;
    use shader_compiler_backend::*;
    use std::mem;

    fn make_compiler() -> impl Compiler {
        crate::LLVM_7_SHADER_COMPILER
    }

    #[test]
    fn test_basic() {
        type GeneratedFunctionType = unsafe extern "C" fn(u32);
        #[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
        enum FunctionKey {
            Function,
        }
        struct Test;
        impl CompilerUser for Test {
            type FunctionKey = FunctionKey;
            type Error = String;
            fn create_error(message: String) -> String {
                message
            }
            fn run<'a, C: Context<'a>>(
                self,
                context: &'a C,
            ) -> Result<CompileInputs<'a, C, FunctionKey>, String> {
                let type_builder = context.create_type_builder();
                let mut module = context.create_module("test_module");
                let mut function = module.add_function(
                    "test_function",
                    type_builder.build::<GeneratedFunctionType>(),
                );
                let builder = context.create_builder();
                let builder = builder.attach(function.append_new_basic_block(None));
                builder.build_return(None);
                let module = module.verify().unwrap();
                Ok(CompileInputs {
                    module,
                    callable_functions: vec![(FunctionKey::Function, function)]
                        .into_iter()
                        .collect(),
                })
            }
        }
        let compiled_code = make_compiler().run(Test, Default::default()).unwrap();
        let function = compiled_code.get(&FunctionKey::Function).unwrap();
        unsafe {
            let function: GeneratedFunctionType = mem::transmute(function);
            function(0);
        }
    }

    #[test]
    fn test_names() {
        const NAMES: &[&str] = &["main", "abc123-$._"];
        type GeneratedFunctionType = unsafe extern "C" fn(u32);
        #[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
        struct Test;
        impl CompilerUser for Test {
            type FunctionKey = String;
            type Error = String;
            fn create_error(message: String) -> String {
                message
            }
            fn run<'a, C: Context<'a>>(
                self,
                context: &'a C,
            ) -> Result<CompileInputs<'a, C, String>, String> {
                let type_builder = context.create_type_builder();
                let mut module = context.create_module("test_module");
                let mut functions = Vec::new();
                let mut detached_builder = context.create_builder();
                for &name in NAMES {
                    let mut function =
                        module.add_function(name, type_builder.build::<GeneratedFunctionType>());
                    let builder = detached_builder.attach(function.append_new_basic_block(None));
                    detached_builder = builder.build_return(None);
                    functions.push((name.to_string(), function));
                }
                let module = module.verify().unwrap();
                Ok(CompileInputs {
                    module,
                    callable_functions: functions.into_iter().collect(),
                })
            }
        }
        let compiled_code = make_compiler().run(Test, Default::default()).unwrap();
        let function = compiled_code.get(&"main".to_string()).unwrap();
        unsafe {
            let function: GeneratedFunctionType = mem::transmute(function);
            function(0);
        }
    }
}
