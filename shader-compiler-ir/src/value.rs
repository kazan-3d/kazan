// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

//! intermediate representation for SSA values

use crate::interned_string::InternedString;
use crate::types::Type;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct ValueDefinition {
    value: Value,
}

impl ValueDefinition {
    pub fn value(&self) -> Value {
        self.value.clone()
    }
    pub fn new(data: ValueData) -> Self {
        ValueDefinition {
            value: Value {
                data: Rc::new(data),
            },
        }
    }
}

impl Deref for ValueDefinition {
    type Target = ValueData;
    fn deref(&self) -> &ValueData {
        &*self.value
    }
}

#[derive(Clone, Debug)]
pub struct ValueData {
    pub value_type: Type,
    pub name: InternedString,
}

#[derive(Clone)]
pub struct Value {
    data: Rc<ValueData>,
}

impl Value {
    pub fn id(&self) -> *const ValueData {
        &*self.data
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.id().hash(hasher)
    }
}

impl PartialEq for Value {
    fn eq(&self, rhs: &Self) -> bool {
        self.id() == rhs.id()
    }
}

impl Eq for Value {}

impl Deref for Value {
    type Target = ValueData;
    fn deref(&self) -> &ValueData {
        &*self.data
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        macro_rules! debug_fields {
            (
                $($field:ident,)+
            ) => {
                {
                    let ValueData {
                        $($field,)+
                    } = &**self;
                    f.debug_struct("Value")
                    .field("id", &self.id())
                    $(.field(stringify!($field), $field))+
                    .finish()
                }
            };
        }
        debug_fields! {
            value_type,
            name,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct ValueUse {
    pub value: Value,
}
