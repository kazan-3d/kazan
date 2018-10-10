// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay
#![deny(missing_docs)]

//! Shader Compiler for Kazan

#[macro_use]
pub mod backend;

#[cfg(test)]
mod test {
    #![allow(dead_code)]

    buildable_struct!{
        struct S1 {
        }
    }

    buildable_struct!{
        pub struct S2 {
            v: u32,
        }
    }

    buildable_struct!{
        struct S3 {
            p: *mut S2,
            v: ::backend::VecNx4<f32>,
        }
    }
}
