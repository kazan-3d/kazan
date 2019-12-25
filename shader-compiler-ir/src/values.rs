// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use std::cell::Cell;
use std::ops::Deref;

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ValueDefinition<'g> {
    value: IdRef<'g, Value<'g>>,
}

impl<'g> ValueDefinition<'g> {
    pub fn new(
        value_type: Interned<'g, Type<'g>>,
        name: Interned<'g, str>,
        global_state: &'g GlobalState<'g>,
    ) -> ValueDefinition<'g> {
        ValueDefinition {
            value: global_state.alloc(Value {
                value_type,
                name,
                const_value: Cell::new(None),
            }),
        }
    }
    pub fn define_as_const(
        self,
        const_value: Interned<'g, Const<'g>>,
        global_state: &'g GlobalState<'g>,
    ) -> IdRef<'g, Value<'g>> {
        let Self { value } = self;
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
        const_value: Interned<'g, Const<'g>>,
        name: Interned<'g, str>,
        global_state: &'g GlobalState<'g>,
    ) -> IdRef<'g, Value<'g>> {
        global_state.alloc(Value {
            name,
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
