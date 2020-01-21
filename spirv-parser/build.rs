// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#[cfg(feature = "spirv-parser-generator")]
use spirv_parser_generator::*;
#[cfg(feature = "spirv-parser-generator")]
use std::{env, io, path::Path};

#[cfg(feature = "spirv-parser-generator")]
fn main() -> Result<(), io::Error> {
    Input::with_default_paths(&[
        ExtensionInstructionSet::OpenCLStd,
        ExtensionInstructionSet::GLSLStd450,
    ])
    .generate()?
    .write_to_file(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/generated_parser.rs"))?;
    Ok(())
}

#[cfg(not(feature = "spirv-parser-generator"))]
fn main() {}
