// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;

mod ast;
mod util;

pub const SPIRV_CORE_GRAMMAR_JSON_FILE_NAME: &str = "spirv.core.grammar.json";

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ExtensionInstructionSet {
    GLSLStd450,
    OpenCLStd,
}

impl ExtensionInstructionSet {
    pub fn get_grammar_json_file_name(self) -> &'static str {
        match self {
            ExtensionInstructionSet::GLSLStd450 => "extinst.glsl.std.450.grammar.json",
            ExtensionInstructionSet::OpenCLStd => "extinst.opencl.std.100.grammar.json",
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    JSONError(serde_json::Error),
    DeducingNameForInstructionOperandFailed,
    DeducingNameForEnumerantParameterFailed,
}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Error {
        Error::IOError(v)
    }
}

impl From<serde_json::Error> for Error {
    fn from(v: serde_json::Error) -> Error {
        if let serde_json::error::Category::Io = v.classify() {
            Error::IOError(v.into())
        } else {
            Error::JSONError(v)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IOError(v) => fmt::Display::fmt(v, f),
            Error::JSONError(v) => fmt::Display::fmt(v, f),
            Error::DeducingNameForInstructionOperandFailed => {
                write!(f, "deducing name for InstructionOperand failed")
            }
            Error::DeducingNameForEnumerantParameterFailed => {
                write!(f, "deducing name for EnumerantParameter failed")
            }
        }
    }
}

impl error::Error for Error {}

impl From<Error> for io::Error {
    fn from(error: Error) -> Self {
        match error {
            Error::IOError(v) => v,
            Error::JSONError(v) => v.into(),
            error @ Error::DeducingNameForInstructionOperandFailed
            | error @ Error::DeducingNameForEnumerantParameterFailed => {
                io::Error::new(io::ErrorKind::Other, format!("{}", error))
            }
        }
    }
}

pub struct Output {}

pub struct Input {
    spirv_core_grammar_json_path: PathBuf,
    extension_instruction_sets: HashMap<ExtensionInstructionSet, PathBuf>,
}

impl Input {
    pub fn new<T: AsRef<Path>>(spirv_core_grammar_json_path: T) -> Input {
        Input {
            spirv_core_grammar_json_path: spirv_core_grammar_json_path.as_ref().into(),
            extension_instruction_sets: HashMap::new(),
        }
    }
    pub fn add_extension_instruction_set<T: AsRef<Path>>(
        mut self,
        extension_instruction_set: ExtensionInstructionSet,
        path: T,
    ) -> Self {
        assert!(
            self.extension_instruction_sets
                .insert(extension_instruction_set, path.as_ref().into())
                .is_none(),
            "duplicate extension instruction set: {:?}",
            extension_instruction_set
        );
        self
    }
    pub fn generate(self) -> Result<Output, Error> {
        let Input {
            spirv_core_grammar_json_path,
            extension_instruction_sets,
        } = self;
        let mut core_grammar: ast::CoreGrammar =
            serde_json::from_reader(File::open(spirv_core_grammar_json_path)?)?;
        core_grammar.guess_names()?;
        let mut parsed_extension_instruction_sets: HashMap<
            ExtensionInstructionSet,
            ast::ExtensionInstructionSet,
        > = Default::default();
        for (extension_instruction_set, path) in extension_instruction_sets {
            let mut parsed_extension_instruction_set: ast::ExtensionInstructionSet =
                serde_json::from_reader(File::open(path)?)?;
            parsed_extension_instruction_set.guess_names()?;
            assert!(
                parsed_extension_instruction_sets
                    .insert(extension_instruction_set, parsed_extension_instruction_set)
                    .is_none()
            );
        }
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn get_spirv_grammar_path<T: AsRef<Path>>(name: T) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../external/SPIRV-Headers/include/spirv/unified1")
            .join(name)
    }

    fn create_input(extension_instruction_sets: &[ExtensionInstructionSet]) -> Input {
        let mut retval = Input::new(get_spirv_grammar_path("spirv.core.grammar.json"));
        for &extension_instruction_set in extension_instruction_sets {
            retval = retval.add_extension_instruction_set(
                extension_instruction_set,
                get_spirv_grammar_path(extension_instruction_set.get_grammar_json_file_name()),
            );
        }
        retval
    }

    #[test]
    fn parse_core_grammar() -> Result<(), Error> {
        create_input(&[]).generate()?;
        Ok(())
    }

    #[test]
    fn parse_core_grammar_with_opencl() -> Result<(), Error> {
        create_input(&[ExtensionInstructionSet::OpenCLStd]).generate()?;
        Ok(())
    }

    #[test]
    fn parse_core_grammar_with_opencl_and_glsl() -> Result<(), Error> {
        create_input(&[
            ExtensionInstructionSet::OpenCLStd,
            ExtensionInstructionSet::GLSLStd450,
        ])
        .generate()?;
        Ok(())
    }

    #[test]
    fn parse_core_grammar_with_glsl() -> Result<(), Error> {
        create_input(&[ExtensionInstructionSet::GLSLStd450]).generate()?;
        Ok(())
    }
}
