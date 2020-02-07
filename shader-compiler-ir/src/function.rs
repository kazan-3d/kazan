// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{
    prelude::*,
    text::{
        FromTextError, FromTextScopeId, FromTextState, FromTextSymbol, FromTextSymbolsState,
        FromTextSymbolsStateBase, Keyword, NamedId, Punctuation, ToTextState, TokenKind,
    },
    Allocate, DataPointerType, FunctionPointerType, IdRef, OnceCell, ParsedBlockNameDefinition,
};
use alloc::vec::Vec;
use core::{fmt, ops::Deref};

/// the function entry, holds the `ValueDefinition`s for the function's arguments
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

/// function inlining hint
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum InliningHint {
    /// no request to inline or not.
    None,
    /// strong request to inline this function, to the extent possible.
    ///
    /// This is the equivalent of LLVM's `alwaysinline`.
    Inline,
    /// strong request to not inline this function, to the extent possible
    ///
    /// This is different than LLVM's `noinline` because inlining is still permitted, though discouraged.
    DontInline,
}

impl Default for InliningHint {
    fn default() -> Self {
        InliningHint::None
    }
}

impl_display_as_to_text!(InliningHint);

impl<'g> FromText<'g> for InliningHint {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let retval = match state.peek_token()?.kind.keyword() {
            Some(Keyword::None) => InliningHint::None,
            Some(Keyword::Inline) => InliningHint::Inline,
            Some(Keyword::DontInline) => InliningHint::DontInline,
            _ => state
                .error_at_peek_token(
                    "expected inlining hint (one of `none`, `inline`, or `dont_inline`)",
                )?
                .into(),
        };
        state.parse_token()?;
        Ok(retval)
    }
}

impl<'g> ToText<'g> for InliningHint {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            InliningHint::None => write!(state, "none"),
            InliningHint::Inline => write!(state, "inline"),
            InliningHint::DontInline => write!(state, "dont_inline"),
        }
    }
}

/// function side-effects
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum FunctionSideEffects {
    /// The function is a normal function -- can read and modify all visible memory and have side-effects.
    Normal,
    /// Compiler can assume this function has no side effect, but might read global
    /// memory or read through dereferenced function parameters. Always computes the
    /// same result for the same argument values.
    ///
    /// Same as SPIR-V's `FunctionControl::Pure` or LLVM's `readonly`
    Pure,
    /// Compiler can assume this function has no side effects, and will not access
    /// global memory or dereference function parameters. Always computes the same
    /// result for the same argument values.
    ///
    /// Same as SPIR-V's `FunctionControl::Const` or LLVM's `readnone`
    Const,
}

impl Default for FunctionSideEffects {
    fn default() -> Self {
        FunctionSideEffects::Normal
    }
}

impl_display_as_to_text!(FunctionSideEffects);

impl<'g> FromText<'g> for FunctionSideEffects {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let retval = match state.peek_token()?.kind.keyword() {
            Some(Keyword::Normal) => FunctionSideEffects::Normal,
            Some(Keyword::Pure) => FunctionSideEffects::Pure,
            Some(Keyword::Const) => FunctionSideEffects::Const,
            _ => state
                .error_at_peek_token(
                    "expected function side-effects (one of `normal`, `pure`, or `const`)",
                )?
                .into(),
        };
        state.parse_token()?;
        Ok(retval)
    }
}

impl<'g> ToText<'g> for FunctionSideEffects {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match self {
            FunctionSideEffects::Normal => write!(state, "normal"),
            FunctionSideEffects::Pure => write!(state, "pure"),
            FunctionSideEffects::Const => write!(state, "const"),
        }
    }
}

impl_struct_with_default_from_to_text! {
    /// optimization hints for a function
    #[name_keyword = hints]
    #[from_text(state <'g> FunctionHints, retval => Ok(retval))]
    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    pub struct FunctionHints {
        /// function inlining hint
        inlining_hint: InliningHint = InliningHint::default(),
        /// function side-effects
        side_effects: FunctionSideEffects = FunctionSideEffects::default(),
    }
}

/// a variable
pub struct Variable<'g> {
    /// the type of the variable
    pub variable_type: Interned<'g, Type<'g>>,
    /// the `ValueDefinition` that points to the variable
    pub pointer: ValueDefinition<'g>,
}

impl_display_as_to_text!(<'g> Variable<'g>);

impl<'g> FromText<'g> for Variable<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let variable_type = Type::from_text(state)?;
        state.parse_punct_token_or_error(
            Punctuation::Arrow,
            "missing arrow (`->`) between variable type and value definition",
        )?;
        let pointer_location = state.peek_token()?.span;
        let pointer: ValueDefinition<'g> = ValueDefinition::from_text(state)?;
        if pointer.value_type != DataPointerType.intern(state.global_state()) {
            state.error_at(
                pointer_location,
                "invalid variable value definition type: must be data_ptr",
            )?;
        }
        Ok(Variable {
            variable_type,
            pointer,
        })
    }
}

impl<'g> ToText<'g> for Variable<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let Variable {
            variable_type,
            pointer,
        } = self;
        variable_type.to_text(state)?;
        write!(state, " -> ")?;
        pointer.to_text(state)
    }
}

/// the struct storing the data for a `Function`
pub struct FunctionData<'g> {
    /// the name of the `Function` -- doesn't need to be unique
    pub name: Interned<'g, str>,
    /// optimization hints for the `Function`
    pub hints: FunctionHints,
    /// the type of a pointer to this function
    pub function_type: FunctionPointerType<'g>,
    /// the function entry, holds the `ValueDefinition`s for the function's arguments
    pub entry: FunctionEntry<'g>,
    /// the local variables of this function
    pub local_variables: OnceCell<Vec<Variable<'g>>>,
    /// The body of the function, the function returns when `body` finishes.
    /// The values defined in `body.result_definitions` are the return values.
    pub body: Block<'g>,
}

impl<'g> FunctionData<'g> {
    /// Sets the local variables of `self` to the passed-in value.
    ///
    /// # Panics
    ///
    /// Panics if the local variables were already set.
    pub fn set_local_variables(&self, local_variables: Vec<Variable<'g>>) {
        #![allow(clippy::ok_expect)]
        self.local_variables
            .set(local_variables)
            .ok()
            .expect("function local variables already set");
    }
}

impl<'g> Id<'g> for FunctionData<'g> {}

/// a reference to a `Function`
#[derive(Clone, Eq, PartialEq, Hash)]
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
#[derive(Eq, PartialEq, Hash)]
pub struct Function<'g> {
    value: IdRef<'g, FunctionData<'g>>,
}

impl<'g> Function<'g> {
    /// create a new `Function`. the name doesn't need to be unique.
    /// the type is made from `argument_definitions` and from `body.result_definitions`.
    pub fn new(
        name: impl Internable<'g, Interned = str>,
        hints: FunctionHints,
        argument_definitions: Vec<ValueDefinition<'g>>,
        local_variables: Option<Vec<Variable<'g>>>,
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
                hints,
                function_type,
                entry: FunctionEntry {
                    argument_definitions,
                },
                local_variables: local_variables.map_or_else(OnceCell::new, OnceCell::from),
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
        let missing_opening_brace = "missing opening curly brace: '{'";
        state.parse_parenthesized(
            Punctuation::LCurlyBrace,
            missing_opening_brace,
            Punctuation::RCurlyBrace,
            missing_closing_brace,
            |state| -> Result<_, _> {
                let hints = FunctionHints::from_text(state)?;
                let local_variables = state.parse_parenthesized(
                    Punctuation::LCurlyBrace,
                    missing_opening_brace,
                    Punctuation::RCurlyBrace,
                    missing_closing_brace,
                    |state| -> Result<_, _> {
                        let mut local_variables = Vec::new();
                        loop {
                            match state.peek_token()?.kind {
                                TokenKind::EndOfFile
                                | TokenKind::Punct(Punctuation::RCurlyBrace) => break,
                                _ => {
                                    local_variables.push(Variable::from_text(state)?);
                                    state.parse_punct_token_or_error(
                                        Punctuation::Semicolon,
                                        "missing terminating semicolon: ';'",
                                    )?;
                                }
                            }
                        }
                        Ok(local_variables)
                    },
                )?;
                let block_name = ParsedBlockNameDefinition::from_text(state)?;
                let block =
                    Block::without_body(block_name.name, result_definitions, state.global_state());
                let block_value = block.value();
                let function = Function::new(
                    name.name,
                    hints,
                    argument_definitions,
                    Some(local_variables),
                    block,
                    state.global_state(),
                );
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
            hints,
            function_type,
            entry: FunctionEntry {
                argument_definitions,
            },
            local_variables,
            body,
        } = &***self;
        let local_variables = local_variables
            .get()
            .expect("function local variables not set");
        argument_definitions.to_text(state)?;
        write!(state, " -> ")?;
        function_type.returns.to_text(state)?;
        writeln!(state, " {{")?;
        state.indent(|state| {
            hints.to_text(state)?;
            writeln!(state)?;
            writeln!(state, "{{")?;
            state.indent(|state| {
                for local_variable in local_variables {
                    local_variable.to_text(state)?;
                    writeln!(state, ";")?;
                }
                Ok(())
            })?;
            writeln!(state, "}}")?;
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
    use crate::{ContinueLoop, IntegerType};
    use alloc::string::ToString;

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
        let function1 = Function::new(
            "function1",
            FunctionHints {
                inlining_hint: InliningHint::Inline,
                side_effects: FunctionSideEffects::Const,
            },
            vec![],
            Some(vec![]),
            block1,
            global_state,
        );
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
                "    hints {\n",
                "        inlining_hint: inline,\n",
                "        side_effects: const,\n",
                "    }\n",
                "    {\n",
                "    }\n",
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
        let function1 = Function::new(
            "function1",
            FunctionHints::default(),
            vec![],
            Some(vec![Variable {
                variable_type: IntegerType::Int32.intern(global_state),
                pointer: ValueDefinition::new(DataPointerType, "local_var1", global_state),
            }]),
            block1,
            global_state,
        );
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
                "    hints {\n",
                "        inlining_hint: none,\n",
                "        side_effects: normal,\n",
                "    }\n",
                "    {\n",
                "        i32 -> local_var1 : data_ptr;\n",
                "    }\n",
                "    block1 {\n",
                "        loop loop1[\"\"0 : fn function1] -> ! {\n",
                "            -> [loop_var : fn[] -> !];\n",
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
