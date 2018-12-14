// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

mod input_ir;

fn main() {}

#[cfg(test)]
mod tests {
    use crate::input_ir;
    #[test]
    fn test1() {
        let cfg = input_ir::parse(
            r"#
start:
    %1 f32{32} fadd %2 f32{32} %3 f32{32}
    return
",
        )
        .map_err(|v| v.to_string())
        .unwrap();
    }
}
