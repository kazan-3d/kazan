// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
use spirv_parser_generator::*;
use std::env;
use std::io;
use std::path::Path;

fn main() -> Result<(), io::Error> {
    Input::with_default_paths(&[
        ExtensionInstructionSet::OpenCLStd,
        ExtensionInstructionSet::GLSLStd450,
    ])
    .generate()?
    .write_to_file(Path::new(&env::var_os("OUT_DIR").unwrap()).join("generated_parser.rs"))?;
    Ok(())
}
