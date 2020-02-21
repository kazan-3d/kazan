// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::{text::FromText, GlobalState, Module};
use alloc::string::{String, ToString};
use core::fmt;

#[doc(hidden)]
pub fn assert_ir_matches_file_helper<Fail: FnOnce(fmt::Arguments<'_>)>(
    ir: &Module<'_>,
    file_text: &'static str,
    file_name: &'static str,
    fail: Fail,
) {
    let file_text: String = file_text
        .chars()
        .filter(|&ch| match ch {
            '\r' => false,       // skip CR in CRLF to convert to LF
            '\u{FEFF}' => false, // skip BOM
            _ => true,
        })
        .collect();
    let file_text = file_text.trim_end_matches('\n');
    let ir_text = ir.to_string();
    if ir_text != file_text {
        fail(format_args!(
            "ir doesn't match contents of {:?}:\n{}",
            file_name, ir_text
        ));
    }
}

/// Asserts that the provided IR `Module` matches the contents of the file `file_name`.
///
/// The IR is converted to text, then matched against the text from the file loaded using `include_str!`.
///
/// This will `panic!` if the IR doesn't match.
#[macro_export]
macro_rules! assert_ir_matches_file {
    ($ir:expr, $file_name:expr $(,)*) => {
        $crate::macros::assert_ir_matches_file_helper(
            &$ir,
            include_str!($file_name),
            $file_name,
            |fail_msg| panic!("{}", fail_msg),
        )
    };
}

#[doc(hidden)]
pub fn include_ir_file_helper<'g, Fail: FnOnce(fmt::Arguments<'_>) -> Module<'g>>(
    global_state: &'g GlobalState<'g>,
    file_text: &'static str,
    file_name: &'static str,
    fail: Fail,
) -> Module<'g> {
    match Module::parse(file_name, file_text, global_state) {
        Ok(retval) => retval,
        Err(error) => fail(format_args!("failed to parse IR into Module: {}", error)),
    }
}

/// Include the contents of the file `file_name` parsed into a `Module`.
///
/// The file loaded using `include_str!`, then parsed using `Module::parse`.
///
/// # Panics
///
/// Panics at run-time if the file can't be parsed as a valid `Module`.
#[macro_export]
macro_rules! include_ir_file {
    ($global_state:expr, $file_name:expr $(,)*) => {
        $crate::macros::include_ir_file_helper(
            $global_state,
            include_str!($file_name),
            $file_name,
            |fail_msg| panic!("{}", fail_msg),
        )
    };
}

#[cfg(test)]
mod tests {
    use crate::GlobalState;

    #[test]
    fn test_assert_ir_matches_file() {
        let global_state = GlobalState::new();
        let value = include_ir_file!(
            &global_state,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_data/test_assert_ir_matches_file.kazan-ir"
            )
        );
        assert_ir_matches_file!(
            value,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/test_data/test_assert_ir_matches_file.kazan-ir"
            )
        );
    }
}
