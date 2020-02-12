// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{IntegerType, Internable, Interned};

impl_struct_with_default_from_to_text! {
    /// the configuration of a target machine
    #[name_keyword = target_properties]
    #[from_text(state <'g> Interned<'g, TargetProperties>, retval => Ok(retval.intern(state.global_state())))]
    #[derive(Clone, Eq, PartialEq, Hash)]
    pub struct TargetProperties {
        /// the underlying integer type of a data pointer
        data_pointer_underlying_type: IntegerType = IntegerType::Int64,
        /// the underlying integer type of a function pointer
        function_pointer_underlying_type: IntegerType = IntegerType::Int64,
    }
}

#[cfg(test)]
mod tests {
    use crate::{prelude::*, IntegerType, TargetProperties};
    use alloc::string::ToString;

    #[test]
    fn test_target_properties() {
        let global_state = GlobalState::new();
        macro_rules! test_target_properties {
            ($global_state:ident, $text:expr, $value:expr, $formatted_text:expr) => {
                let parsed_value = TargetProperties::parse("", $text, &$global_state).unwrap();
                let expected_value = $value.intern(&$global_state);
                assert_eq!(parsed_value, expected_value);
                let text = expected_value.display().to_string();
                assert_eq!($formatted_text, text);
            };
            ($global_state:ident, $text:expr, $value:expr) => {
                test_target_properties!($global_state, $text, $value, $text);
            };
        }

        test_target_properties!(
            global_state,
            "target_properties {}",
            TargetProperties::default(),
            concat!(
                "target_properties {\n",
                "    data_pointer_underlying_type: i64,\n",
                "    function_pointer_underlying_type: i64,\n",
                "}"
            )
        );
        test_target_properties!(
            global_state,
            "target_properties {data_pointer_underlying_type: i32,}",
            TargetProperties {
                data_pointer_underlying_type: IntegerType::Int32,
                function_pointer_underlying_type: IntegerType::Int64,
            },
            concat!(
                "target_properties {\n",
                "    data_pointer_underlying_type: i32,\n",
                "    function_pointer_underlying_type: i64,\n",
                "}"
            )
        );
        test_target_properties!(
            global_state,
            concat!(
                "target_properties {\n",
                "    data_pointer_underlying_type: i64,\n",
                "    function_pointer_underlying_type: i64,\n",
                "}"
            ),
            TargetProperties {
                data_pointer_underlying_type: IntegerType::Int64,
                function_pointer_underlying_type: IntegerType::Int64,
            }
        );
        test_target_properties!(
            global_state,
            concat!(
                "target_properties {\n",
                "    data_pointer_underlying_type: i32,\n",
                "    function_pointer_underlying_type: i32,\n",
                "}"
            ),
            TargetProperties {
                data_pointer_underlying_type: IntegerType::Int32,
                function_pointer_underlying_type: IntegerType::Int32,
            }
        );
    }
}
