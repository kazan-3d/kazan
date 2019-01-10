// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

// allow unneeded_field_pattern to ensure fields aren't accidently missed
#![allow(clippy::unneeded_field_pattern)]

use std::collections::HashMap;
use std::error;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;

mod ast;
mod generate;
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

pub struct Output {
    text: String,
}

impl Output {
    pub fn to_str(&self) -> &str {
        &self.text
    }
    pub fn into_string(self) -> String {
        self.text
    }
    pub fn write<W: io::Write>(&self, mut writer: W) -> Result<(), io::Error> {
        write!(writer, "{}", self.text)
    }
    pub fn write_to_file<T: AsRef<Path>>(&self, path: T) -> Result<(), io::Error> {
        self.write(File::create(path)?)
    }
}

struct Options {
    run_rustfmt: bool,
    rustfmt_path: Option<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            run_rustfmt: true,
            rustfmt_path: None,
        }
    }
}

pub struct Input {
    spirv_core_grammar_json_path: PathBuf,
    extension_instruction_sets: HashMap<ExtensionInstructionSet, PathBuf>,
    options: Options,
}

fn get_spirv_grammar_path<T: AsRef<Path>>(name: T) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../external/SPIRV-Headers/include/spirv/unified1")
        .join(name)
}

impl Input {
    pub fn with_default_paths(extension_instruction_sets: &[ExtensionInstructionSet]) -> Input {
        let mut retval = Self::new(get_spirv_grammar_path("spirv.core.grammar.json"));
        for &extension_instruction_set in extension_instruction_sets {
            retval = retval.add_extension_instruction_set(
                extension_instruction_set,
                get_spirv_grammar_path(extension_instruction_set.get_grammar_json_file_name()),
            );
        }
        retval
    }
    pub fn new<T: AsRef<Path>>(spirv_core_grammar_json_path: T) -> Input {
        Input {
            spirv_core_grammar_json_path: spirv_core_grammar_json_path.as_ref().into(),
            extension_instruction_sets: HashMap::new(),
            options: Options::default(),
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
            options,
        } = self;
        let mut core_grammar: ast::CoreGrammar =
            serde_json::from_reader(File::open(spirv_core_grammar_json_path)?)?;
        core_grammar.fixup()?;
        let mut parsed_extension_instruction_sets: HashMap<
            ExtensionInstructionSet,
            ast::ExtensionInstructionSet,
        > = Default::default();
        for (extension_instruction_set, path) in extension_instruction_sets {
            let mut parsed_extension_instruction_set: ast::ExtensionInstructionSet =
                serde_json::from_reader(File::open(path)?)?;
            parsed_extension_instruction_set.fixup()?;
            assert!(parsed_extension_instruction_sets
                .insert(extension_instruction_set, parsed_extension_instruction_set)
                .is_none());
        }
        Ok(Output {
            text: generate::generate(core_grammar, parsed_extension_instruction_sets, &options)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_core_grammar() -> Result<(), Error> {
        Input::with_default_paths(&[]).generate()?;
        Ok(())
    }

    #[test]
    fn parse_core_grammar_with_opencl() -> Result<(), Error> {
        Input::with_default_paths(&[ExtensionInstructionSet::OpenCLStd]).generate()?;
        Ok(())
    }

    #[test]
    fn parse_core_grammar_with_opencl_and_glsl() -> Result<(), Error> {
        Input::with_default_paths(&[
            ExtensionInstructionSet::OpenCLStd,
            ExtensionInstructionSet::GLSLStd450,
        ])
        .generate()?;
        Ok(())
    }

    #[test]
    fn parse_core_grammar_with_glsl() -> Result<(), Error> {
        Input::with_default_paths(&[ExtensionInstructionSet::GLSLStd450]).generate()?;
        Ok(())
    }
}
