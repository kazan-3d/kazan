// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextScopeId;
use crate::text::FromTextState;
use crate::text::FromTextSymbol;
use crate::text::FromTextSymbolsState;
use crate::text::FromTextSymbolsStateBase;
use crate::text::Keyword;
use crate::text::NamedId;
use crate::text::Punctuation;
use crate::text::ToTextState;
use crate::Allocate;
use crate::FunctionPointerType;
use crate::ParsedBlockNameDefinition;
use std::fmt;
use std::ops::Deref;

/// the function entry, holds the `ValueDefinition`s for the function's arguments
#[derive(Debug)]
pub struct FunctionEntry<'g> {
    /// the `ValueDefinition`s for the function's arguments
    pub argument_definitions: Vec<ValueDefinition<'g>>,
}

impl<'g> CodeIO<'g> for FunctionEntry<'g> {
    fn results(&self) -> Inhabitable<&[ValueDefinition<'g>]> {
        Inhabited(&self.argument_definitions)
    }
    fn arguments(&self) -> &[ValueUse<'g>] {
        &[]
    }
}

/// the struct storing the data for a `Function`
#[derive(Debug)]
pub struct FunctionData<'g> {
    /// the name of the `Function` -- doesn't need to be unique
    pub name: Interned<'g, str>,
    /// the type of a pointer to this function
    pub function_type: FunctionPointerType<'g>,
    /// the function entry, holds the `ValueDefinition`s for the function's arguments
    pub entry: FunctionEntry<'g>,
    /// The body of the function, the function returns when `body` finishes.
    /// The values defined in `body.result_definitions` are the return values.
    pub body: Block<'g>,
}

impl<'g> Id<'g> for FunctionData<'g> {}

/// a reference to a `Function`
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct FunctionRef<'g> {
    value: IdRef<'g, FunctionData<'g>>,
}

impl<'g> FunctionRef<'g> {
    /// create a new reference to the passed in function
    pub fn new(value: IdRef<'g, FunctionData<'g>>) -> Self {
        Self { value }
    }
    /// get the contained `IdRef<FunctionData>`
    pub fn value(&self) -> IdRef<'g, FunctionData<'g>> {
        self.value
    }
}

/// a function
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct Function<'g> {
    value: IdRef<'g, FunctionData<'g>>,
}

impl<'g> Function<'g> {
    /// create a new `Function`. the name doesn't need to be unique.
    /// the type is made from `argument_definitions` and from `body.result_definitions`.
    pub fn new(
        name: impl Internable<'g, Interned = str>,
        argument_definitions: Vec<ValueDefinition<'g>>,
        body: Block<'g>,
        global_state: &'g GlobalState<'g>,
    ) -> Self {
        let function_type = FunctionPointerType {
            arguments: argument_definitions.iter().map(|v| v.value_type).collect(),
            returns: body
                .result_definitions
                .as_ref()
                .map(|v| v.iter().map(|v| v.value_type).collect()),
        };
        Function {
            value: global_state.alloc(FunctionData {
                name: name.intern(global_state),
                function_type,
                entry: FunctionEntry {
                    argument_definitions,
                },
                body,
            }),
        }
    }
    /// get the contained `IdRef<FunctionData>`
    pub fn value(&self) -> IdRef<'g, FunctionData<'g>> {
        self.value
    }
}

impl<'g> Deref for FunctionRef<'g> {
    type Target = IdRef<'g, FunctionData<'g>>;
    fn deref(&self) -> &IdRef<'g, FunctionData<'g>> {
        &self.value
    }
}

impl<'g> Deref for Function<'g> {
    type Target = IdRef<'g, FunctionData<'g>>;
    fn deref(&self) -> &IdRef<'g, FunctionData<'g>> {
        &self.value
    }
}

impl<'g> FromText<'g> for FunctionRef<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let name_location = state.peek_token()?.span;
        let name = NamedId::from_text(state)?;
        if let Some(FromTextSymbol { value, scope }) = state.get_symbol(name) {
            if state.is_scope_visible(scope) {
                Ok(FunctionRef::new(value))
            } else {
                state
                    .error_at(name_location, "function not in scope")?
                    .into()
            }
        } else {
            state.error_at(name_location, "name not found")?.into()
        }
    }
}

impl_display_as_to_text!(<'g> FunctionRef<'g>);

impl<'g> ToText<'g> for FunctionRef<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let name = state.get_function_named_id(self.value());
        let name = state.check_name_use(name, "function definition must be written first");
        name.to_text(state)
    }
}

impl<'g> FromText<'g> for Function<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        state.parse_keyword_token_or_error(Keyword::Fn, "expected function (`fn`)")?;
        let name_location = state.peek_token()?.span;
        let name = NamedId::from_text(state)?;
        let initial_scope = state.scope_stack_top;
        let argument_definitions = Vec::<ValueDefinition>::from_text(state)?;
        state.parse_punct_token_or_error(Punctuation::Arrow, "missing arrow: '->'")?;
        let return_types = Inhabitable::<Vec<Type>>::from_text(state)?;
        let result_definitions: Inhabitable<Vec<_>> = return_types.as_ref().map(|v| {
            v.iter()
                .map(|v| ValueDefinition::new(v, "", state.global_state()))
                .collect()
        });
        let missing_closing_brace = "missing closing curly brace: '}'";
        state.parse_parenthesized(
            Punctuation::LCurlyBrace,
            "missing opening curly brace: '{'",
            Punctuation::RCurlyBrace,
            missing_closing_brace,
            |state| -> Result<_, _> {
                let block_name = ParsedBlockNameDefinition::from_text(state)?;
                let block =
                    Block::without_body(block_name.name, result_definitions, state.global_state());
                let block_value = block.value();
                let function =
                    Function::new(name.name, argument_definitions, block, state.global_state());
                if state
                    .insert_symbol(
                        name,
                        FromTextSymbol {
                            value: function.value(),
                            scope: FromTextScopeId::ROOT,
                        },
                    )
                    .is_err()
                {
                    state.error_at(name_location, "duplicate function name")?;
                }
                Block::parse_body(block_value, block_name, state)?;
                state.scope_stack_top = initial_scope;
                Ok(function)
            },
        )
    }
}

impl<'g> ToText<'g> for Function<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        write!(state, "fn ")?;
        let name = state.get_function_named_id(self.value());
        let name = state.check_name_definition(name, "function definition must be written first");
        name.to_text(state)?;
        let FunctionData {
            name: _name,
            function_type,
            entry: FunctionEntry {
                argument_definitions,
            },
            body,
        } = &***self;
        argument_definitions.to_text(state)?;
        write!(state, " -> ")?;
        function_type.returns.to_text(state)?;
        writeln!(state, " {{")?;
        state.indent(|state| {
            Block::name_definition_to_text(body.value(), state)?;
            write!(state, " ")?;
            body.body_to_text(state)?;
            writeln!(state)
        })?;
        write!(state, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContinueLoop;

    #[test]
    fn test_from_to_text() {
        let global_state = GlobalState::new();
        let global_state = &global_state;
        macro_rules! test_from_to_text {
            ($global_state:ident, $text:expr, $type:ty, $value:expr, $formatted_text:expr) => {{
                let text = $value.display().to_string();
                assert_eq!($formatted_text, text);
                let parsed_value = <$type>::parse("", $text, $global_state).unwrap();
                let text = parsed_value.display().to_string();
                assert_eq!($formatted_text, text);
            }};
            ($global_state:ident, $text:expr, $type:ty, $const:expr) => {
                test_from_to_text!($global_state, $text, $type, $const, $text);
            };
        }

        let block1 = Block::without_body("block1", Uninhabited, global_state);
        let function1 = Function::new("function1", vec![], block1, global_state);
        let mut block1_body = Vec::new();
        let block2 = Block::without_body("block2", Uninhabited, global_state);
        let mut block2_body = Vec::new();
        let loop1 = Loop::new("loop1", vec![], vec![], block2, global_state);
        block2_body.push(Instruction::without_location(ContinueLoop {
            target_loop: LoopRef::new(loop1.value()),
            loop_arguments: vec![],
        }));
        loop1.body.set_body(block2_body);
        block1_body.push(Instruction::without_location(loop1));
        function1.body.set_body(block1_body);
        test_from_to_text!(
            global_state,
            concat!(
                "fn function1[] -> ! {\n",
                "    block1 {\n",
                "        loop loop1[] -> ! {\n",
                "            -> [];\n",
                "            block2 {\n",
                "                continue loop1[];\n",
                "            }\n",
                "        };\n",
                "    }\n",
                "}"
            ),
            Function,
            function1
        );

        let block1 = Block::without_body("block1", Uninhabited, global_state);
        let function1 = Function::new("function1", vec![], block1, global_state);
        let mut block1_body = Vec::new();
        let block2 = Block::without_body("block2", Uninhabited, global_state);
        let mut block2_body = Vec::new();
        let loop_var_def = ValueDefinition::new(&function1.function_type, "loop_var", global_state);
        let loop_var = loop_var_def.value();
        let loop1 = Loop::new(
            "loop1",
            vec![ValueUse::from_const(
                FunctionRef::new(function1.value()),
                "",
                global_state,
            )],
            vec![loop_var_def],
            block2,
            global_state,
        );
        block2_body.push(Instruction::without_location(ContinueLoop {
            target_loop: LoopRef::new(loop1.value()),
            loop_arguments: vec![ValueUse::new(loop_var)],
        }));
        loop1.body.set_body(block2_body);
        block1_body.push(Instruction::without_location(loop1));
        function1.body.set_body(block1_body);
        test_from_to_text!(
            global_state,
            concat!(
                "fn function1[] -> ! {\n",
                "    block1 {\n",
                "        loop loop1[\"\"0 : fn function1] -> ! {\n",
                "            -> [loop_var : *fn[] -> !];\n",
                "            block2 {\n",
                "                continue loop1[loop_var];\n",
                "            }\n",
                "        };\n",
                "    }\n",
                "}"
            ),
            Function,
            function1
        );
    }
}
