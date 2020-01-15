// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextScopeId;
use crate::text::FromTextState;
use crate::text::FromTextSymbol;
use crate::text::FromTextSymbolsState;
use crate::text::FromTextSymbolsStateBase;
use crate::text::NamedId;
use crate::text::NewOrOld;
use crate::text::Punctuation;
use crate::text::ToTextState;
use crate::text::TokenKind;
use crate::Allocate;
use crate::IdRef;
use crate::OnceCell;
use core::fmt;
use core::ops::Deref;

/// the definition of a SSA value -- the point at which the value is assigned to
#[derive(Eq, PartialEq, Hash)]
pub struct ValueDefinition<'g> {
    value: IdRef<'g, Value<'g>>,
}

impl<'g> ValueDefinition<'g> {
    /// create a new `ValueDefinition`.
    /// `name` doesn't need to be unique.
    pub fn new(
        value_type: impl Internable<'g, Interned = Type<'g>>,
        name: impl Internable<'g, Interned = str>,
        global_state: &'g GlobalState<'g>,
    ) -> ValueDefinition<'g> {
        ValueDefinition {
            value: global_state.alloc(Value {
                value_type: value_type.intern(global_state),
                name: name.intern(global_state),
                const_value: OnceCell::new(),
            }),
        }
    }
    /// permanently assign a constant to this IR value.
    ///
    /// # Panics
    ///
    /// panics if `const_value` doesn't have the same IR type as this IR value.
    pub fn define_as_const(
        self,
        const_value: impl Internable<'g, Interned = Const<'g>>,
        global_state: &'g GlobalState<'g>,
    ) -> IdRef<'g, Value<'g>> {
        let Self { value } = self;
        let const_value = const_value.intern(global_state);
        assert_eq!(value.value_type, const_value.get().get_type(global_state));
        value
            .const_value
            .set(const_value)
            .ok()
            .expect("invalid Value state");
        value
    }
    /// get the contained `Value`
    pub fn value(&self) -> IdRef<'g, Value<'g>> {
        self.value
    }
}

impl<'g> Deref for ValueDefinition<'g> {
    type Target = IdRef<'g, Value<'g>>;
    fn deref(&self) -> &IdRef<'g, Value<'g>> {
        &self.value
    }
}

/// the data for a IR value
pub struct Value<'g> {
    /// the IR type
    pub value_type: Interned<'g, Type<'g>>,
    /// the name -- doesn't need to be unique
    pub name: Interned<'g, str>,
    const_value: OnceCell<Interned<'g, Const<'g>>>,
}

impl<'g> Id<'g> for Value<'g> {}

impl<'g> Value<'g> {
    /// the constant value of `self`, if `self` is known to be a constant
    pub fn const_value(&self) -> Option<Interned<'g, Const<'g>>> {
        self.const_value.get().copied()
    }
    /// create a new constant IR value.
    /// `name` doesn't need to be unique.
    pub fn from_const(
        const_value: impl Internable<'g, Interned = Const<'g>>,
        name: impl Internable<'g, Interned = str>,
        global_state: &'g GlobalState<'g>,
    ) -> IdRef<'g, Value<'g>> {
        let const_value = const_value.intern(global_state);
        global_state.alloc(Value {
            name: name.intern(global_state),
            value_type: const_value.get().get_type(global_state),
            const_value: OnceCell::from(const_value),
        })
    }
}

/// a use of an IR value
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ValueUse<'g> {
    value: IdRef<'g, Value<'g>>,
}

impl<'g> ValueUse<'g> {
    /// create a new `ValueUse`
    pub fn new(value: IdRef<'g, Value<'g>>) -> Self {
        Self { value }
    }
    /// create a `ValueUse` for a new constant IR value.
    /// `name` doesn't need to be unique.
    pub fn from_const(
        const_value: impl Internable<'g, Interned = Const<'g>>,
        name: impl Internable<'g, Interned = str>,
        global_state: &'g GlobalState<'g>,
    ) -> ValueUse<'g> {
        Self {
            value: Value::from_const(const_value, name, global_state),
        }
    }
    /// get the contained `Value`
    pub fn value(&self) -> IdRef<'g, Value<'g>> {
        self.value
    }
}

impl<'g> Deref for ValueUse<'g> {
    type Target = IdRef<'g, Value<'g>>;
    fn deref(&self) -> &IdRef<'g, Value<'g>> {
        &self.value
    }
}

impl<'g> FromText<'g> for ValueDefinition<'g> {
    type Parsed = ValueDefinition<'g>;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<ValueDefinition<'g>, FromTextError> {
        let name_location = state.peek_token()?.span;
        let name = NamedId::from_text(state)?;
        state.parse_punct_token_or_error(Punctuation::Colon, "missing ':'")?;
        let value_type = Type::from_text(state)?;
        let retval = Self::new(value_type, name.name, state.global_state());
        let scope = state.push_new_nested_scope();
        if state
            .insert_symbol(
                name,
                FromTextSymbol {
                    value: retval.value(),
                    scope,
                },
            )
            .is_err()
        {
            state.error_at(name_location, "value defined previously")?;
        }
        Ok(retval)
    }
}

impl<'g> FromText<'g> for ValueUse<'g> {
    type Parsed = Self;
    fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Self, FromTextError> {
        let name_location = state.peek_token()?.span;
        let name = NamedId::from_text(state)?;
        if let TokenKind::Punct(Punctuation::Colon) = state.peek_token()?.kind {
            state.parse_token()?;
            let const_value = Const::from_text(state)?;
            let retval = ValueUse::from_const(const_value, name.name, state.global_state());
            if state
                .insert_symbol(
                    name,
                    FromTextSymbol {
                        value: retval.value(),
                        scope: FromTextScopeId::ROOT,
                    },
                )
                .is_err()
            {
                state.error_at(name_location, "value defined previously")?;
            }
            Ok(retval)
        } else if let Some(FromTextSymbol { value, scope }) = state.get_symbol(name) {
            if state.is_scope_visible(scope) {
                Ok(ValueUse::new(value))
            } else {
                state.error_at(name_location, "value not in scope")?.into()
            }
        } else {
            state.error_at(name_location, "name not found")?.into()
        }
    }
}

impl_display_as_to_text!(<'g> ValueDefinition<'g>);

impl<'g> ToText<'g> for ValueDefinition<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let name = state.get_value_named_id(self.value());
        let name = state.check_name_definition(name, "value definition must be written first");
        name.to_text(state)?;
        write!(state, " : ")?;
        self.value().value_type.to_text(state)
    }
}

impl_display_as_to_text!(<'g> ValueUse<'g>);

impl<'g> ToText<'g> for ValueUse<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        let name = state.get_value_named_id(self.value());
        if let (Some(const_value), NewOrOld::New(name)) = (self.value().const_value.get(), &name) {
            name.to_text(state)?;
            write!(state, " : ")?;
            const_value.to_text(state)
        } else {
            let name = state.check_name_use(name, "value definition must be written first");
            name.to_text(state)
        }
    }
}
