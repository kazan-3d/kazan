// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use ast;
use proc_macro2;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use util::{self, NameFormat::*};
use which;
use Error;
use Options;

#[derive(Debug)]
enum FormatError {
    IOError(io::Error),
    WhichError(which::Error),
    RustFmtFailed(ExitStatus),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FormatError::IOError(v) => fmt::Display::fmt(v, f),
            FormatError::WhichError(v) => fmt::Display::fmt(v, f),
            FormatError::RustFmtFailed(v) => write!(f, "rustfmt failed: {:?}", v),
        }
    }
}

impl From<which::Error> for FormatError {
    fn from(v: which::Error) -> Self {
        FormatError::WhichError(v)
    }
}

impl From<io::Error> for FormatError {
    fn from(v: io::Error) -> Self {
        FormatError::IOError(v)
    }
}

fn format_source<'a>(options: &Options, source: &'a str) -> Result<Cow<'a, str>, FormatError> {
    if !options.run_rustfmt {
        return Ok(Cow::Borrowed(source));
    }
    let rustfmt_path = match options.rustfmt_path.clone() {
        Some(v) => v,
        None => which::which("rustfmt")?,
    };
    let mut command = Command::new(rustfmt_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let stdin = command.stdin.take().unwrap();
    let reader_thread = thread::spawn(move || -> io::Result<(String, Child)> {
        let mut output = String::new();
        command.stdout.take().unwrap().read_to_string(&mut output)?;
        Ok((output, command))
    });
    { stdin }.write_all(source.as_bytes())?;
    let (output, mut command) = reader_thread.join().unwrap()?;
    let exit_status = command.wait()?;
    if exit_status.success() {
        Ok(Cow::Owned(output))
    } else {
        Err(FormatError::RustFmtFailed(exit_status))
    }
}

fn remove_initial_op(name: &str) -> &str {
    const INITIAL_OP: &str = "Op";
    assert!(name.starts_with(INITIAL_OP));
    &name[INITIAL_OP.len()..]
}

fn new_id<T: AsRef<str>>(name: T, name_format: util::NameFormat) -> proc_macro2::Ident {
    proc_macro2::Ident::new(
        &name_format
            .name_from_words(util::WordIterator::new(name.as_ref()))
            .unwrap(),
        proc_macro2::Span::call_site(),
    )
}

fn new_enumerant_id<T1: AsRef<str>, T2: AsRef<str>>(
    enum_name: T1,
    enumerant_name: T2,
) -> proc_macro2::Ident {
    let enumerant_name_words = util::WordIterator::new(enumerant_name.as_ref());
    let enumerant_name_first_word = enumerant_name_words.clone().next();
    let name = if enumerant_name_first_word
        .map(str::chars)
        .as_mut()
        .and_then(Iterator::next)
        .filter(char::is_ascii_digit)
        .is_some()
    {
        CamelCase
            .name_from_words(
                util::WordIterator::new(enum_name.as_ref()).chain(enumerant_name_words),
            )
            .unwrap()
    } else {
        CamelCase.name_from_words(enumerant_name_words).unwrap()
    };
    proc_macro2::Ident::new(&name, proc_macro2::Span::call_site())
}

fn new_combined_id<I: IntoIterator>(names: I, name_format: util::NameFormat) -> proc_macro2::Ident
where
    I::Item: AsRef<str>,
{
    let names: Vec<I::Item> = names.into_iter().collect();
    proc_macro2::Ident::new(
        &name_format
            .name_from_words(
                names
                    .iter()
                    .map(AsRef::as_ref)
                    .flat_map(util::WordIterator::new),
            )
            .unwrap(),
        proc_macro2::Span::call_site(),
    )
}

#[cfg_attr(feature = "cargo-clippy", allow(clippy::cyclomatic_complexity))]
pub(crate) fn generate(
    core_grammar: ast::CoreGrammar,
    parsed_extension_instruction_sets: HashMap<
        super::ExtensionInstructionSet,
        ast::ExtensionInstructionSet,
    >,
    options: &Options,
) -> Result<String, Error> {
    let mut out = Vec::new();
    let ast::CoreGrammar {
        copyright: core_grammar_copyright,
        magic_number,
        major_version,
        minor_version,
        revision: core_revision,
        instructions: core_instructions,
        operand_kinds,
    } = core_grammar;
    writeln!(&mut out, "// automatically generated file")?;
    writeln!(&mut out, "//")?;
    for i in &core_grammar_copyright {
        assert_eq!(i.find('\r'), None);
        assert_eq!(i.find('\n'), None);
        if i == "" {
            writeln!(&mut out, "//");
        } else {
            writeln!(&mut out, "// {}", i);
        }
    }
    writeln!(
        &mut out,
        "{}",
        quote!{
            pub const MAGIC_NUMBER: u32 = #magic_number;
            pub const MAJOR_VERSION: u32 = #major_version;
            pub const MINOR_VERSION: u32 = #minor_version;
            pub const REVISION: u32 = #core_revision;
        }
    )?;
    for operand_kind in &operand_kinds {
        match operand_kind {
            ast::OperandKind::BitEnum { kind, enumerants } => {
                let mut enumerant_members = Vec::new();
                let mut enumerant_member_names = Vec::new();
                let mut enumerant_items = Vec::new();
                for enumerant in enumerants {
                    if enumerant.value.0 == 0 {
                        continue;
                    }
                    let member_name = new_id(&enumerant.enumerant, SnakeCase);
                    enumerant_member_names.push(member_name.clone());
                    let type_name =
                        new_combined_id(&[kind.as_ref(), &enumerant.enumerant], CamelCase);
                    if enumerant.parameters.is_empty() {
                        enumerant_items.push(quote!{
                            #[derive(Clone, Debug, Default)]
                            pub struct #type_name;
                        });
                    } else {
                        let parameters = enumerant.parameters.iter().map(|parameter| {
                            let kind = new_id(&parameter.kind, CamelCase);
                            quote!{
                                pub #kind,
                            }
                        });
                        enumerant_items.push(quote!{
                            #[derive(Clone, Debug, Default)]
                            pub struct #type_name(#(#parameters)*);
                        });
                    }
                    enumerant_members.push(quote!{
                        pub #member_name: Option<#type_name>
                    });
                }
                let kind_id = new_id(kind, CamelCase);
                writeln!(
                    &mut out,
                    "{}",
                    quote!{
                        #[derive(Clone, Debug, Default)]
                        pub struct #kind_id {
                            #(#enumerant_members),*
                        }
                        impl #kind_id {
                            pub fn new() -> Self {
                                Self {
                                    #(#enumerant_member_names: None,)*
                                }
                            }
                        }
                        #(#enumerant_items)*
                    }
                )?;
            }
            ast::OperandKind::ValueEnum { kind, enumerants } => {
                let kind_id = new_id(&kind, CamelCase);
                let mut generated_enumerants = Vec::new();
                for enumerant in enumerants {
                    let name = new_enumerant_id(&kind, &enumerant.enumerant);
                    if enumerant.parameters.is_empty() {
                        generated_enumerants.push(quote!{#name});
                        continue;
                    }
                }
                writeln!(
                    &mut out,
                    "{}",
                    quote!{
                        #[derive(Clone, Debug)]
                        pub enum #kind_id {
                            #(#generated_enumerants,)*
                        }
                    }
                )?;
            }
            ast::OperandKind::Id { kind, doc: _ } => {
                let base = if *kind == ast::Kind::IdRef {
                    quote!{u32}
                } else {
                    quote!{IdRef}
                };
                let kind_id = new_id(kind, CamelCase);
                writeln!(
                    &mut out,
                    "{}",
                    quote!{
                        #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
                        #[repr(transparent)]
                        pub struct #kind_id(pub #base);
                    }
                )?;
            }
            ast::OperandKind::Literal { kind, doc: _ } => {
                let kind_id = new_id(kind, CamelCase);
                writeln!(
                    &mut out,
                    "{}",
                    match kind {
                        ast::LiteralKind::LiteralInteger
                        | ast::LiteralKind::LiteralContextDependentNumber => unreachable!(),
                        ast::LiteralKind::LiteralInteger32
                        | ast::LiteralKind::LiteralContextDependentNumber32 => {
                            quote!{pub type #kind_id = u32;}
                        }
                        ast::LiteralKind::LiteralInteger64
                        | ast::LiteralKind::LiteralContextDependentNumber64 => {
                            quote!{pub type #kind_id = u64;}
                        }
                        ast::LiteralKind::LiteralString => quote!{pub type #kind_id = String;},
                        ast::LiteralKind::LiteralExtInstInteger => {
                            quote!{pub type #kind_id = u32;}
                        }
                        ast::LiteralKind::LiteralSpecConstantOpInteger => continue,
                    }
                )?;
            }
            ast::OperandKind::Composite { kind, bases } => {
                let kind = new_id(kind, CamelCase);
                let bases = bases.into_iter().map(|base| new_id(base, CamelCase));
                writeln!(&mut out, "{}", quote!{pub type #kind = (#(#bases),*);})?;
            }
        }
    }
    {
        let mut instruction_enumerants = Vec::new();
        let mut spec_constant_op_instruction_enumerants = Vec::new();
        for instruction in core_instructions.iter() {
            let opname = new_id(remove_initial_op(instruction.opname.as_ref()), CamelCase);
            let instruction_enumerant =
                if instruction.opname == ast::InstructionName::OpSpecConstantOp {
                    quote!{
                        #opname {
                            operation: OpSpecConstantOp,
                        }
                    }
                } else if instruction.operands.is_empty() {
                    quote!{#opname}
                } else {
                    let mut fields = Vec::new();
                    for operand in instruction.operands.iter() {
                        let kind = new_id(&operand.kind, CamelCase);
                        let name = new_id(operand.name.as_ref().unwrap(), SnakeCase);
                        let kind = match &operand.quantifier {
                            None => quote!{#kind},
                            Some(ast::Quantifier::Optional) => quote!{Option<#kind>},
                            Some(ast::Quantifier::Variadic) => quote!{Vec<#kind>},
                        };
                        fields.push(quote!{#name: #kind});
                    }
                    quote!{
                        #opname {
                            #(#fields,)*
                        }
                    }
                };
            if ast::OP_SPEC_CONSTANT_OP_SUPPORTED_INSTRUCTIONS.contains(&instruction.opname) {
                spec_constant_op_instruction_enumerants.push(instruction_enumerant.clone());
            }
            instruction_enumerants.push(instruction_enumerant);
        }
        writeln!(
            &mut out,
            "{}",
            quote!{
                #[derive(Clone, Debug)]
                pub enum OpSpecConstantOp {
                    #(#spec_constant_op_instruction_enumerants,)*
                }
                #[derive(Clone, Debug)]
                pub enum Instruction {
                    #(#instruction_enumerants,)*
                }
            }
        )?;
    }
    let source = String::from_utf8(out).unwrap();
    let source = match format_source(&options, &source) {
        Ok(source) => source.into_owned(),
        Err(error) => {
            eprintln!("formatting source failed: {}", error);
            source.clone()
        }
    };
    Ok(source)
}
