// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

#![cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]

include!(concat!(env!("OUT_DIR"), "/generated_parser.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    use std::slice;

    #[test]
    fn parse_test() {
        const BYTES: &[u8] = include_bytes!("../test_inputs/test.spv");
        assert_eq!(BYTES.len() % mem::size_of::<u32>(), 0);
        let mut words: Vec<u32> = Vec::new();
        words.resize(BYTES.len() / mem::size_of::<u32>(), 0);
        unsafe {
            let bytes = slice::from_raw_parts_mut(
                words.as_mut_ptr() as *mut u8,
                words.len() * mem::size_of::<u32>(),
            );
            bytes.copy_from_slice(BYTES);
        }
        assert!(!words.is_empty());
        if words[0].swap_bytes() == MAGIC_NUMBER {
            for word in words.iter_mut() {
                *word = word.swap_bytes();
            }
        }
        let parser = Parser::start(&words).unwrap();
        println!("{}", parser.header());
        for instruction in parser.map(Result::unwrap) {
            println!("{:?}", instruction);
        }
    }
}
