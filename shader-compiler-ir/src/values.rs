// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextState;
use crate::text::NamedId;
use crate::text::NewOrOld;
use crate::text::Punctuation;
use crate::text::ToTextState;
use crate::text::TokenKind;
use crate::text::ValueAndAvailability;
use std::cell::Cell;
use std::fmt;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ValueDefinition<'g> {
    value: IdRef<'g, Value<'g>>,
}

impl<'g> ValueDefinition<'g> {
    pub fn new(
        value_type: impl Internable<'g, Interned = Type<'g>>,
        name: impl Internable<'g, Interned = str>,
        global_state: &'g GlobalState<'g>,
    ) -> ValueDefinition<'g> {
        ValueDefinition {
            value: global_state.alloc(Value {
                value_type: value_type.intern(global_state),
                name: name.intern(global_state),
                const_value: Cell::new(None),
            }),
        }
    }
    pub fn define_as_const(
        self,
        const_value: impl Internable<'g, Interned = Const<'g>>,
        global_state: &'g GlobalState<'g>,
    ) -> IdRef<'g, Value<'g>> {
        let Self { value } = self;
        let const_value = const_value.intern(global_state);
        assert_eq!(value.value_type, const_value.get().get_type(global_state));
        assert!(
            value.const_value.replace(Some(const_value)).is_none(),
            "invalid Value state"
        );
        value
    }
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

#[derive(Debug)]
pub struct Value<'g> {
    pub value_type: Interned<'g, Type<'g>>,
    pub name: Interned<'g, str>,
    pub const_value: Cell<Option<Interned<'g, Const<'g>>>>,
}

impl<'g> Id<'g> for Value<'g> {}

impl<'g> Value<'g> {
    pub fn from_const(
        const_value: impl Internable<'g, Interned = Const<'g>>,
        name: impl Internable<'g, Interned = str>,
        global_state: &'g GlobalState<'g>,
    ) -> IdRef<'g, Value<'g>> {
        let const_value = const_value.intern(global_state);
        global_state.alloc(Value {
            name: name.intern(global_state),
            value_type: const_value.get().get_type(global_state),
            const_value: Cell::new(Some(const_value)),
        })
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ValueUse<'g> {
    value: IdRef<'g, Value<'g>>,
}

impl<'g> ValueUse<'g> {
    pub fn new(value: IdRef<'g, Value<'g>>) -> Self {
        Self { value }
    }
    pub fn from_const(
        const_value: impl Internable<'g, Interned = Const<'g>>,
        name: impl Internable<'g, Interned = str>,
        global_state: &'g GlobalState<'g>,
    ) -> ValueUse<'g> {
        Self {
            value: Value::from_const(const_value, name, global_state),
        }
    }
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

#[derive(Debug)]
pub struct UnavailableValueDefinition<'g>(Rc<ValueAndAvailability<'g>>);

impl<'g> UnavailableValueDefinition<'g> {
    pub fn mark_available(self) -> ValueDefinition<'g> {
        let value = self.0.mark_available();
        ValueDefinition { value }
    }
}

impl<'g> From<ValueDefinition<'g>> for UnavailableValueDefinition<'g> {
    fn from(v: ValueDefinition<'g>) -> Self {
        UnavailableValueDefinition(ValueAndAvailability::new(v.value(), false))
    }
}

impl<'g> FromText<'g> for ValueDefinition<'g> {
    type Parsed = UnavailableValueDefinition<'g>;
    fn from_text(
        state: &mut FromTextState<'g, '_>,
    ) -> Result<UnavailableValueDefinition<'g>, FromTextError> {
        let name_location = state.peek_token()?.span;
        let name = NamedId::from_text(state)?;
        state.parse_punct_token_or_error(Punctuation::Colon, "missing ':'")?;
        let value_type = Type::from_text(state)?;
        let retval = UnavailableValueDefinition::from(Self::new(
            value_type,
            name.name,
            state.global_state(),
        ));
        if let Err(_) = state.insert_value(name, retval.0.clone()) {
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
            if let Err(_) =
                state.insert_value(name, ValueAndAvailability::new(retval.value(), true))
            {
                state.error_at(name_location, "value defined previously")?;
            }
            Ok(retval)
        } else if let Some(value) = state.get_value(name) {
            if let Some(value) = value.value_if_available() {
                Ok(ValueUse::new(value))
            } else {
                state
                    .error_at(name_location, "value not yet available")?
                    .into()
            }
        } else {
            state.error_at(name_location, "name not found")?.into()
        }
    }
}

impl<'g> ToText<'g> for ValueDefinition<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        if let NewOrOld::New(name) = state.get_value_named_id(self.value()) {
            name.to_text(state)?;
            write!(state, " : ")?;
            self.value().value_type.to_text(state)
        } else {
            panic!("value definition must be written first");
        }
    }
}

impl<'g> ToText<'g> for ValueUse<'g> {
    fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
        match state.get_value_named_id(self.value()) {
            NewOrOld::New(name) => {
                name.to_text(state)?;
                write!(state, " : ")?;
                self.value()
                    .const_value
                    .get()
                    .expect("value definition must be written first")
                    .to_text(state)
            }
            NewOrOld::Old(name) => name.to_text(state),
        }
    }
}
