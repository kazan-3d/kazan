// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::prelude::*;
use crate::text::FromTextError;
use crate::text::FromTextState;
use crate::text::Keyword;
use crate::text::Punctuation;
use crate::text::ToTextState;
use crate::IntegerType;
use core::fmt;

macro_rules! impl_target_properties {
    ($($(#[doc = $doc:expr])+ $member:ident:$member_ty:ty = $member_init:expr,)+) => {
        /// the configuration of a target machine
        #[derive(Clone, Eq, PartialEq, Hash)]
        pub struct TargetProperties {
            $(
                $(#[doc = $doc])+
                pub $member: $member_ty,
            )+
        }

        impl_display_as_to_text!(TargetProperties);

        impl Default for TargetProperties {
            fn default() -> Self {
                TargetProperties {
                    $(
                        $member: $member_init,
                    )+
                }
            }
        }

        impl<'g> FromText<'g> for TargetProperties {
            type Parsed = Interned<'g, Self>;
            fn from_text(state: &mut FromTextState<'g, '_>) -> Result<Interned<'g, Self>, FromTextError> {
                state
                    .parse_keyword_token_or_error(Keyword::TargetProperties, "missing target_properties")?;
                state.parse_parenthesized(
                    Punctuation::LCurlyBrace,
                    "missing opening curly brace ('{')",
                    Punctuation::RCurlyBrace,
                    "missing closing curly brace ('}')",
                    |state| -> Result<Interned<'g, Self>, FromTextError> {
                        let mut retval = Self::default();
                        struct Specified {
                            $($member: bool,)+
                        }
                        let mut specified = Specified {
                            $($member: false,)+
                        };
                        while let Some(ident) = state.peek_token()?.kind.raw_identifier() {
                            match ident {
                                $(
                                    stringify!($member) => {
                                        if specified.$member {
                                            state.error_at_peek_token(concat!("duplicate field: ", stringify!($member), " already specified"))?;
                                        }
                                        specified.$member = true;
                                        state.parse_token()?;
                                        state.parse_punct_token_or_error(Punctuation::Colon, "missing colon (':') after field name")?;
                                        retval.$member = <$member_ty>::from_text(state)?;
                                    }
                                )+
                                _ => state.error_at_peek_token("unknown field name")?.into(),
                            }
                            state.parse_punct_token_or_error(Punctuation::Comma, "missing comma (',') after field value")?;
                        }
                        Ok(retval.intern(state.global_state()))
                    },
                )
            }
        }

        impl<'g> ToText<'g> for TargetProperties {
            fn to_text(&self, state: &mut ToTextState<'g, '_>) -> fmt::Result {
                writeln!(state, "target_properties {{")?;
                state.indent(|state| -> fmt::Result {
                    $(
                        write!(state, concat!(stringify!($member), ": "))?;
                        self.$member.to_text(state)?;
                        writeln!(state, ",")?;
                    )+
                    Ok(())
                })?;
                write!(state, "}}")
            }
        }
    };
}

impl_target_properties! {
    /// the underlying integer type of a data pointer
    data_pointer_underlying_type: IntegerType = IntegerType::Int64,
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use crate::IntegerType;
    use crate::TargetProperties;
    use alloc::string::ToString;

    #[test]
    fn test_target_properties() {
        let global_state = GlobalState::new();
        macro_rules! test_target_properties {
            ($global_state:ident, $text:literal, $value:expr, $formatted_text:literal) => {
                let parsed_value = TargetProperties::parse("", $text, &$global_state).unwrap();
                let expected_value = $value.intern(&$global_state);
                assert_eq!(parsed_value, expected_value);
                let text = expected_value.display().to_string();
                assert_eq!($formatted_text, text);
            };
            ($global_state:ident, $text:literal, $value:expr) => {
                test_target_properties!($global_state, $text, $value, $text);
            };
        }

        test_target_properties!(
            global_state,
            "target_properties {}",
            TargetProperties::default(),
            "target_properties {\n    data_pointer_underlying_type: i64,\n}"
        );
        test_target_properties!(
            global_state,
            "target_properties {\n    data_pointer_underlying_type: i64,\n}",
            TargetProperties {
                data_pointer_underlying_type: IntegerType::Int64
            }
        );
        test_target_properties!(
            global_state,
            "target_properties {\n    data_pointer_underlying_type: i32,\n}",
            TargetProperties {
                data_pointer_underlying_type: IntegerType::Int32
            }
        );
    }
}
