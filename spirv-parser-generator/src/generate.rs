// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::ast;
use crate::util::{self, NameFormat::*};
use crate::Error;
use crate::Options;
use proc_macro2;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::io::{self, Read, Write};
use std::iter;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use which;
use quote::quote;

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

struct ParsedExtensionInstructionSet {
    ast: ast::ExtensionInstructionSet,
    enumerant_name: proc_macro2::Ident,
    spirv_instruction_set_name: &'static str,
}

#[allow(clippy::cyclomatic_complexity)]
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
    let parsed_extension_instruction_sets: Vec<_> = parsed_extension_instruction_sets
        .into_iter()
        .map(|(key, ast)| match key {
            super::ExtensionInstructionSet::GLSLStd450 => ParsedExtensionInstructionSet {
                ast,
                enumerant_name: new_id("GLSLStd450", CamelCase),
                spirv_instruction_set_name: "GLSL.std.450",
            },
            super::ExtensionInstructionSet::OpenCLStd => ParsedExtensionInstructionSet {
                ast,
                enumerant_name: new_id("OpenCLStd", CamelCase),
                spirv_instruction_set_name: "OpenCL.std",
            },
        })
        .collect();
    writeln!(&mut out, "// automatically generated file")?;
    {
        let mut copyright_set = HashSet::new();
        for copyright in iter::once(&core_grammar_copyright).chain(
            parsed_extension_instruction_sets
                .iter()
                .map(|v| &v.ast.copyright),
        ) {
            if !copyright_set.insert(copyright) {
                continue;
            }
            writeln!(&mut out, "//")?;
            for line in copyright.iter() {
                assert_eq!(line.find('\r'), None);
                assert_eq!(line.find('\n'), None);
                if line == "" {
                    writeln!(&mut out, "//")?;
                } else {
                    writeln!(&mut out, "// {}", line)?;
                }
            }
        }
    }
    writeln!(
        &mut out,
        "{}",
        stringify!(
            use std::borrow::Cow;
            use std::error;
            use std::fmt;
            use std::mem;
            use std::ops::Deref;
            use std::result;
            use std::str::Utf8Error;
            use std::string::FromUtf8Error;

            trait SPIRVParse: Sized {
                fn spirv_parse<'a>(words: &'a [u32], parse_state: &mut ParseState)
                    -> Result<(Self, &'a [u32])>;
            }

            trait SPIRVDisplay {
                fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result;
            }

            impl<T: SPIRVParse> SPIRVParse for Option<T> {
                fn spirv_parse<'a>(
                    words: &'a [u32],
                    parse_state: &mut ParseState,
                ) -> Result<(Self, &'a [u32])> {
                    if words.is_empty() {
                        Ok((None, words))
                    } else {
                        let (value, words) = T::spirv_parse(words, parse_state)?;
                        Ok((Some(value), words))
                    }
                }
            }

            impl<T: SPIRVDisplay> SPIRVDisplay for Option<T> {
                fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        Some(v) => v.spirv_display(f),
                        None => Ok(()),
                    }
                }
            }

            impl<T: SPIRVParse> SPIRVParse for Vec<T> {
                fn spirv_parse<'a>(
                    mut words: &'a [u32],
                    parse_state: &mut ParseState,
                ) -> Result<(Self, &'a [u32])> {
                    let mut retval = Vec::new();
                    while !words.is_empty() {
                        let result = T::spirv_parse(words, parse_state)?;
                        words = result.1;
                        retval.push(result.0);
                    }
                    Ok((retval, words))
                }
            }

            impl<T: SPIRVDisplay> SPIRVDisplay for Vec<T> {
                fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    for i in self {
                        i.spirv_display(f)?;
                    }
                    Ok(())
                }
            }

            impl<A: SPIRVParse, B: SPIRVParse> SPIRVParse for (A, B) {
                fn spirv_parse<'a>(
                    words: &'a [u32],
                    parse_state: &mut ParseState,
                ) -> Result<(Self, &'a [u32])> {
                    let (a, words) = A::spirv_parse(words, parse_state)?;
                    let (b, words) = B::spirv_parse(words, parse_state)?;
                    Ok(((a, b), words))
                }
            }

            impl<A: SPIRVDisplay, B: SPIRVDisplay> SPIRVDisplay for (A, B) {
                fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.0.spirv_display(f)?;
                    self.1.spirv_display(f)
                }
            }

            const BYTES_PER_WORD: usize = 4;

            struct ByteIterator<'a> {
                current_word: [u8; BYTES_PER_WORD],
                current_word_index: usize,
                words: &'a [u32],
            }

            impl<'a> ByteIterator<'a> {
                fn new(words: &'a [u32]) -> Self {
                    Self {
                        current_word: [0; BYTES_PER_WORD],
                        current_word_index: BYTES_PER_WORD,
                        words,
                    }
                }
                fn take_unread_words(&mut self) -> &'a [u32] {
                    mem::replace(&mut self.words, &[])
                }
            }

            impl<'a> Iterator for ByteIterator<'a> {
                type Item = u8;
                fn next(&mut self) -> Option<u8> {
                    if self.current_word_index >= BYTES_PER_WORD {
                        let (&current_word, words) = self.words.split_first()?;
                        self.words = words;
                        self.current_word = unsafe { mem::transmute(current_word.to_le()) };
                        self.current_word_index = 0;
                    }
                    let byte = self.current_word[self.current_word_index];
                    self.current_word_index += 1;
                    Some(byte)
                }
            }

            impl SPIRVParse for String {
                fn spirv_parse<'a>(
                    words: &'a [u32],
                    _parse_state: &mut ParseState,
                ) -> Result<(Self, &'a [u32])> {
                    let mut byte_count_excluding_null_terminator = None;
                    for (index, byte) in ByteIterator::new(words).enumerate() {
                        if byte == 0 {
                            byte_count_excluding_null_terminator = Some(index);
                            break;
                        }
                    }
                    let byte_count_excluding_null_terminator =
                        byte_count_excluding_null_terminator.ok_or(Error::InstructionPrematurelyEnded)?;
                    let mut bytes = Vec::with_capacity(byte_count_excluding_null_terminator);
                    let mut byte_iter = ByteIterator::new(words);
                    for _ in 0..byte_count_excluding_null_terminator {
                        let byte = byte_iter.next().unwrap();
                        bytes.push(byte);
                    }
                    let _null_terminator = byte_iter.next().unwrap();
                    let words = byte_iter.take_unread_words();
                    for v in byte_iter {
                        if v != 0 {
                            return Err(Error::InvalidStringTermination);
                        }
                    }
                    assert_eq!(bytes.len(), byte_count_excluding_null_terminator);
                    Ok((String::from_utf8(bytes)?, words))
                }
            }

            impl SPIRVDisplay for String {
                fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, " {:?}", self)
                }
            }

            impl SPIRVParse for u32 {
                fn spirv_parse<'a>(
                    words: &'a [u32],
                    _parse_state: &mut ParseState,
                ) -> Result<(Self, &'a [u32])> {
                    let (&value, words) = words
                        .split_first()
                        .ok_or(Error::InstructionPrematurelyEnded)?;
                    Ok((value, words))
                }
            }

            impl SPIRVDisplay for u32 {
                fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, " {}", self)
                }
            }

            impl SPIRVParse for u64 {
                fn spirv_parse<'a>(
                    words: &'a [u32],
                    _parse_state: &mut ParseState,
                ) -> Result<(Self, &'a [u32])> {
                    let (&low, words) = words
                        .split_first()
                        .ok_or(Error::InstructionPrematurelyEnded)?;
                    let (&high, words) = words
                        .split_first()
                        .ok_or(Error::InstructionPrematurelyEnded)?;
                    Ok(((u64::from(high) << 32) | u64::from(low), words))
                }
            }

            impl SPIRVDisplay for u64 {
                fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, " {}", self)
                }
            }

            impl SPIRVParse for IdRef {
                fn spirv_parse<'a>(
                    words: &'a [u32],
                    parse_state: &mut ParseState,
                ) -> Result<(Self, &'a [u32])> {
                    let (value, words) = u32::spirv_parse(words, parse_state)?;
                    if value == 0 || value as usize >= parse_state.id_states.len() {
                        Err(Error::IdOutOfBounds(value))
                    } else {
                        Ok((IdRef(value), words))
                    }
                }
            }

            impl SPIRVDisplay for IdRef {
                fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, " {}", self)
                }
            }
        )
    )?;
    writeln!(
        &mut out,
        "{}",
        quote! {
            pub const MAGIC_NUMBER: u32 = #magic_number;
            pub const MAJOR_VERSION: u32 = #major_version;
            pub const MINOR_VERSION: u32 = #minor_version;
            pub const REVISION: u32 = #core_revision;
        }
    )?;
    for operand_kind in &operand_kinds {
        match operand_kind {
            ast::OperandKind::BitEnum { kind, enumerants } => {
                let kind_id = new_id(kind, CamelCase);
                let mut enumerant_members = Vec::new();
                let mut enumerant_member_names = Vec::new();
                let mut enumerant_items = Vec::new();
                let mut enumerant_parse_operations = Vec::new();
                let mut enumerant_display_mask_operations = Vec::new();
                let mut enumerant_display_operations = Vec::new();
                let mut none_name = "None";
                for enumerant in enumerants {
                    if enumerant.value.0 == 0 {
                        none_name = enumerant.enumerant.as_ref();
                        continue;
                    }
                    let enumerant_name = &enumerant.enumerant;
                    let member_name = new_id(&enumerant.enumerant, SnakeCase);
                    let member_name = &member_name;
                    enumerant_member_names.push(member_name.clone());
                    let type_name =
                        new_combined_id(&[kind.as_ref(), &enumerant.enumerant], CamelCase);
                    let enumerant_parse_operation;
                    if enumerant.parameters.is_empty() {
                        enumerant_items.push(quote! {
                            #[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
                            pub struct #type_name;
                        });
                        enumerant_parse_operation = quote! {(Some(#type_name), words)};
                        enumerant_display_mask_operations.push(quote! {
                            if self.#member_name.is_some() {
                                if any_members {
                                    write!(f, "|{}", #enumerant_name)?;
                                } else {
                                    write!(f, " {}", #enumerant_name)?;
                                    any_members = true;
                                }
                            }
                        });
                        enumerant_display_operations.push(quote! {});
                    } else {
                        let mut enumerant_parameter_declarations = Vec::new();
                        let mut enumerant_parameter_names = Vec::new();
                        let mut parse_enumerant_members = Vec::new();
                        let mut display_enumerant_members = Vec::new();
                        for (index, parameter) in enumerant.parameters.iter().enumerate() {
                            let name = new_id(format!("parameter_{}", index), SnakeCase);
                            let kind = new_id(&parameter.kind, CamelCase);
                            enumerant_parameter_declarations.push(quote! {
                                pub #kind,
                            });
                            enumerant_parameter_names.push(quote! {
                                #name,
                            });
                            parse_enumerant_members.push(quote! {
                                let (#name, words) = #kind::spirv_parse(words, parse_state)?;
                            });
                            display_enumerant_members.push(quote! {
                                #name.spirv_display(f)?;
                            });
                        }
                        enumerant_items.push(quote! {
                            #[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
                            pub struct #type_name(#(#enumerant_parameter_declarations)*);
                        });
                        let enumerant_parameter_names = &enumerant_parameter_names;
                        enumerant_parse_operation = quote! {
                            #(#parse_enumerant_members)*
                            (Some(#type_name(#(#enumerant_parameter_names)*)), words)
                        };
                        enumerant_display_mask_operations.push(quote! {
                            if self.#member_name.is_some() {
                                if any_members {
                                    write!(f, "|{}", #enumerant_name)?;
                                } else {
                                    write!(f, " {}", #enumerant_name)?;
                                    any_members = true;
                                }
                            }
                        });
                        enumerant_display_operations.push(quote!{
                            if let Some(#type_name(#(#enumerant_parameter_names)*)) = &self.#member_name {
                                #(#display_enumerant_members)*
                            }
                        });
                    };
                    enumerant_members.push(quote! {
                        pub #member_name: Option<#type_name>
                    });
                    let enumerant_value = enumerant.value;
                    enumerant_parse_operations.push(quote! {
                        let (#member_name, words) = if (mask & #enumerant_value) != 0 {
                            mask &= !#enumerant_value;
                            #enumerant_parse_operation
                        } else {
                            (None, words)
                        };
                    })
                }
                writeln!(
                    &mut out,
                    "{}",
                    quote! {
                        #[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
                        pub struct #kind_id {
                            #(#enumerant_members),*
                        }
                        #(#enumerant_items)*
                    }
                )?;
                let parse_body = quote! {
                    let (mut mask, words) = u32::spirv_parse(words, parse_state)?;
                    #(#enumerant_parse_operations)*
                    if mask != 0 {
                        Err(Error::InvalidEnumValue)
                    } else {
                        Ok((Self {
                            #(#enumerant_member_names,)*
                        }, words))
                    }
                };
                writeln!(
                    &mut out,
                    "{}",
                    quote! {
                        impl SPIRVParse for #kind_id {
                            fn spirv_parse<'a>(
                                words: &'a [u32],
                                parse_state: &mut ParseState,
                            ) -> Result<(Self, &'a [u32])> {
                                #parse_body
                            }
                        }
                    }
                )?;
                writeln!(
                    &mut out,
                    "{}",
                    quote! {
                        impl SPIRVDisplay for #kind_id {
                            fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                                let mut any_members = false;
                                #(#enumerant_display_mask_operations)*
                                if !any_members {
                                    write!(f, " {}", #none_name)?;
                                }
                                #(#enumerant_display_operations)*
                                Ok(())
                            }
                        }
                    }
                )?;
            }
            ast::OperandKind::ValueEnum { kind, enumerants } => {
                let mut has_any_parameters = false;
                for enumerant in enumerants {
                    if !enumerant.parameters.is_empty() {
                        has_any_parameters = true;
                    }
                }
                let kind_id = new_id(&kind, CamelCase);
                let mut generated_enumerants = Vec::new();
                let mut enumerant_parse_cases = Vec::new();
                let mut enumerant_display_cases = Vec::new();
                for enumerant in enumerants {
                    let name = new_enumerant_id(&kind, &enumerant.enumerant);
                    let enumerant_value = enumerant.value;
                    let display_name = &enumerant.enumerant;
                    if enumerant.parameters.is_empty() {
                        generated_enumerants.push(quote! {#name});
                        enumerant_parse_cases.push(quote! {
                            #enumerant_value => Ok((#kind_id::#name, words)),
                        });
                        enumerant_display_cases.push(quote! {
                            #kind_id::#name => write!(f, " {}", #display_name),
                        });
                    } else {
                        let mut enumerant_member_declarations = Vec::new();
                        let mut enumerant_member_names = Vec::new();
                        let mut parse_enumerant_members = Vec::new();
                        let mut display_enumerant_members = Vec::new();
                        for parameter in enumerant.parameters.iter() {
                            let name = new_id(parameter.name.as_ref().unwrap(), SnakeCase);
                            let kind = new_id(&parameter.kind, CamelCase);
                            enumerant_member_declarations.push(quote! {
                                #name: #kind,
                            });
                            enumerant_member_names.push(quote! {
                                #name,
                            });
                            parse_enumerant_members.push(quote! {
                                let (#name, words) = #kind::spirv_parse(words, parse_state)?;
                            });
                            display_enumerant_members.push(quote! {
                                #name.spirv_display(f)?;
                            });
                        }
                        generated_enumerants.push(quote! {
                            #name {
                                #(#enumerant_member_declarations)*
                            }
                        });
                        let enumerant_member_names = &enumerant_member_names;
                        enumerant_parse_cases.push(quote! {
                            #enumerant_value => {
                                #(#parse_enumerant_members)*
                                Ok((#kind_id::#name {
                                    #(#enumerant_member_names)*
                                }, words))
                            },
                        });
                        enumerant_display_cases.push(quote! {
                            #kind_id::#name {
                                #(#enumerant_member_names)*
                            } => {
                                write!(f, " {}", #display_name)?;
                                #(#display_enumerant_members)*
                                Ok(())
                            }
                        });
                    }
                }
                let mut derives = vec![
                    quote! {Clone},
                    quote! {Debug},
                    quote! {Eq},
                    quote! {PartialEq},
                    quote! {Hash},
                ];
                if !has_any_parameters {
                    derives.push(quote! {Copy});
                }
                writeln!(
                    &mut out,
                    "{}",
                    quote! {
                        #[derive(#(#derives),*)]
                        pub enum #kind_id {
                            #(#generated_enumerants,)*
                        }
                    }
                )?;
                writeln!(
                    &mut out,
                    "{}",
                    quote! {
                        impl SPIRVParse for #kind_id {
                            fn spirv_parse<'a>(
                                words: &'a [u32],
                                parse_state: &mut ParseState,
                            ) -> Result<(Self, &'a [u32])> {
                                let (enumerant, words) = u32::spirv_parse(words, parse_state)?;
                                match enumerant {
                                    #(#enumerant_parse_cases)*
                                    _ => Err(Error::InvalidEnumValue),
                                }
                            }
                        }
                    }
                )?;
                writeln!(
                    &mut out,
                    "{}",
                    quote! {
                        impl SPIRVDisplay for #kind_id {
                            fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                                match self {
                                    #(#enumerant_display_cases)*
                                }
                            }
                        }
                    }
                )?;
            }
            ast::OperandKind::Id { kind, .. } => {
                let base = if *kind == ast::Kind::IdRef {
                    quote! {u32}
                } else {
                    quote! {IdRef}
                };
                let kind_id = new_id(kind, CamelCase);
                writeln!(
                    &mut out,
                    "{}",
                    quote! {
                        #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
                        #[repr(transparent)]
                        pub struct #kind_id(pub #base);
                    }
                )?;
                if *kind != ast::Kind::IdRef {
                    writeln!(
                        &mut out,
                        "{}",
                        quote! {
                            impl SPIRVParse for #kind_id {
                                fn spirv_parse<'a>(
                                    words: &'a [u32],
                                    parse_state: &mut ParseState,
                                ) -> Result<(Self, &'a [u32])> {
                                    IdRef::spirv_parse(words, parse_state).map(|(value, words)| (#kind_id(value), words))
                                }
                            }
                        }
                    )?;
                    writeln!(
                        &mut out,
                        "{}",
                        quote! {
                            impl fmt::Display for #kind_id {
                                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                                    fmt::Display::fmt(&self.0, f)
                                }
                            }
                        }
                    )?;
                    writeln!(
                        &mut out,
                        "{}",
                        quote! {
                            impl SPIRVDisplay for #kind_id {
                                fn spirv_display(&self, f: &mut fmt::Formatter) -> fmt::Result {
                                    self.0.spirv_display(f)
                                }
                            }
                        }
                    )?;
                }
            }
            ast::OperandKind::Literal { kind, .. } => {
                let kind_id = new_id(kind, CamelCase);
                writeln!(
                    &mut out,
                    "{}",
                    match kind {
                        ast::LiteralKind::LiteralInteger
                        | ast::LiteralKind::LiteralContextDependentNumber => unreachable!(),
                        ast::LiteralKind::LiteralInteger32
                        | ast::LiteralKind::LiteralContextDependentNumber32 => {
                            quote! {pub type #kind_id = u32;}
                        }
                        ast::LiteralKind::LiteralInteger64
                        | ast::LiteralKind::LiteralContextDependentNumber64 => {
                            quote! {pub type #kind_id = u64;}
                        }
                        ast::LiteralKind::LiteralString => quote! {pub type #kind_id = String;},
                        ast::LiteralKind::LiteralExtInstInteger => {
                            quote! {pub type #kind_id = u32;}
                        }
                        ast::LiteralKind::LiteralSpecConstantOpInteger => continue,
                    }
                )?;
            }
            ast::OperandKind::Composite { kind, bases } => {
                let kind = new_id(kind, CamelCase);
                let bases = bases.iter().map(|base| new_id(base, CamelCase));
                writeln!(&mut out, "{}", quote! {pub type #kind = (#(#bases),*);})?;
            }
        }
    }
    {
        let mut instruction_enumerants = Vec::new();
        let mut spec_constant_op_instruction_enumerants = Vec::new();
        let mut instruction_parse_cases = Vec::new();
        let mut instruction_display_cases = Vec::new();
        let mut instruction_spec_constant_parse_cases = Vec::new();
        let mut instruction_spec_constant_display_cases = Vec::new();
        let mut instruction_extension_enumerants = Vec::new();
        let mut instruction_extension_parse_cases = Vec::new();
        let mut instruction_extension_display_cases = Vec::new();
        for parsed_extension_instruction_set in &parsed_extension_instruction_sets {
            let extension_instruction_set = &parsed_extension_instruction_set.enumerant_name;
            for instruction in &parsed_extension_instruction_set.ast.instructions {
                let instruction_enumerant_name = new_combined_id(
                    &[
                        parsed_extension_instruction_set.spirv_instruction_set_name,
                        instruction.opname.as_ref(),
                    ],
                    CamelCase,
                );
                let opcode = instruction.opcode;
                let mut fields = Vec::new();
                for operand in instruction.operands.iter() {
                    let kind = new_id(&operand.kind, CamelCase);
                    let name = new_id(operand.name.as_ref().unwrap(), SnakeCase);
                    let kind = match &operand.quantifier {
                        None => quote! {#kind},
                        Some(ast::Quantifier::Optional) => quote! {Option<#kind>},
                        Some(ast::Quantifier::Variadic) => quote! {Vec<#kind>},
                    };
                    fields.push(quote! {#name: #kind});
                }
                let instruction_extension_enumerant = quote! {
                    #instruction_enumerant_name {
                        id_result_type: IdResultType,
                        id_result: IdResult,
                        set: IdRef,
                        #(#fields,)*
                    }
                };
                instruction_extension_enumerants.push(instruction_extension_enumerant);
                let mut parse_operations = Vec::new();
                let mut display_operations = Vec::new();
                let mut operand_names = Vec::new();
                for operand in &instruction.operands {
                    let kind = new_id(&operand.kind, CamelCase);
                    let name = new_id(operand.name.as_ref().unwrap(), SnakeCase);
                    let kind = match operand.quantifier {
                        None => quote! {#kind},
                        Some(ast::Quantifier::Optional) => quote! {Option::<#kind>},
                        Some(ast::Quantifier::Variadic) => quote! {Vec::<#kind>},
                    };
                    parse_operations.push(quote! {
                        let (#name, words) = #kind::spirv_parse(words, parse_state)?;
                    });
                    display_operations.push(quote! {
                        #name.spirv_display(f)?;
                    });
                    operand_names.push(name);
                }
                let operand_names = &operand_names;
                let body = quote! {
                    #(#parse_operations)*
                    if words.is_empty() {
                        Ok(Instruction::#instruction_enumerant_name {
                            id_result_type,
                            id_result,
                            set,
                            #(#operand_names,)*
                        })
                    } else {
                        Err(Error::InstructionTooLong)
                    }
                };
                let instruction_extension_parse_case = quote! {
                    (ExtensionInstructionSet::#extension_instruction_set, #opcode) => {
                        #body
                    }
                };
                instruction_extension_parse_cases.push(instruction_extension_parse_case);
                let display_opname = &instruction.opname;
                let instruction_extension_display_case = quote! {
                    Instruction::#instruction_enumerant_name {
                        id_result_type,
                        id_result,
                        set,
                        #(#operand_names,)*
                    } => {
                        write!(
                            f,
                            "{}OpExtInst {} {} {}",
                            InstructionIndentAndResult(Some(*id_result)),
                            id_result_type,
                            set,
                            #display_opname,
                        )?;
                        #(#display_operations)*
                        writeln!(f)
                    }
                };
                instruction_extension_display_cases.push(instruction_extension_display_case);
            }
        }
        let instruction_extension_parse_cases = &instruction_extension_parse_cases;
        for instruction in core_instructions.iter() {
            let opcode = instruction.opcode;
            let opname = new_id(remove_initial_op(instruction.opname.as_ref()), CamelCase);
            let display_opname = instruction.opname.as_ref();
            let display_opname_without_initial_op = remove_initial_op(display_opname);
            let instruction_parse_case;
            let instruction_display_case;
            match &instruction.opname {
                ast::InstructionName::OpExtInstImport => {
                    let body = quote! {
                        parse_state.define_id(
                            id_result,
                            IdState::ExtensionInstructionSet(ExtensionInstructionSet::from(&*name)),
                        )?;
                        if words.is_empty() {
                            Ok(Instruction::ExtInstImport { id_result, name })
                        } else {
                            Err(Error::InstructionTooLong)
                        }
                    };
                    instruction_parse_case = quote! {#opcode => {
                        let (id_result, words) = IdResult::spirv_parse(words, parse_state)?;
                        let (name, words) = LiteralString::spirv_parse(words, parse_state)?;
                        #body
                    }};
                    instruction_display_case = quote! {
                        Instruction::ExtInstImport { id_result, name } => {
                            writeln!(f, "{}{} {:?}", InstructionIndentAndResult(Some(*id_result)), #display_opname, name)
                        }
                    };
                }
                ast::InstructionName::OpExtInst => {
                    let body = quote! {
                        let extension_instruction_set;
                        match parse_state.id_states[set.0 as usize].clone() {
                            IdState::ExtensionInstructionSet(ExtensionInstructionSet::Other(_)) => {
                                let (operands, words) = Vec::<LiteralInteger32>::spirv_parse(words, parse_state)?;
                                if words.is_empty() {
                                    return Ok(Instruction::ExtInst {
                                        id_result_type,
                                        id_result,
                                        set,
                                        instruction,
                                        operands,
                                    });
                                } else {
                                    return Err(Error::InstructionTooLong);
                                }
                            }
                            IdState::ExtensionInstructionSet(v) => {
                                extension_instruction_set = v;
                            }
                            _ => return Err(Error::IdIsNotExtInstImport(set)),
                        };
                        match (extension_instruction_set, instruction) {
                            #(#instruction_extension_parse_cases)*
                            (extension_instruction_set, instruction) => Err(Error::UnknownExtensionOpcode(extension_instruction_set, instruction)),
                        }
                    };
                    instruction_parse_case = quote! {
                        #opcode => {
                            let (id_result_type, words) = IdResultType::spirv_parse(words, parse_state)?;
                            let (id_result, words) = IdResult::spirv_parse(words, parse_state)?;
                            parse_state.define_value(id_result_type, id_result)?;
                            let (set, words) = IdRef::spirv_parse(words, parse_state)?;
                            let (instruction, words) = LiteralExtInstInteger::spirv_parse(words, parse_state)?;
                            #body
                        }
                    };
                    instruction_display_case = quote! {
                        Instruction::ExtInst {
                            id_result_type,
                            id_result,
                            set,
                            instruction,
                            operands,
                        } => {
                            write!(f, "{}{}", InstructionIndentAndResult(Some(*id_result)), #display_opname)?;
                            id_result_type.spirv_display(f)?;
                            set.spirv_display(f)?;
                            instruction.spirv_display(f)?;
                            operands.spirv_display(f)?;
                            writeln!(f)
                        }
                    };
                }
                ast::InstructionName::OpTypeInt => {
                    let body = quote! {
                        let (signedness, words) = LiteralInteger32::spirv_parse(words, parse_state)?;
                        let id_state = match width {
                            8 | 16 | 32 => IdState::Type(IdStateType(BitWidth::Width32OrLess)),
                            64 => IdState::Type(IdStateType(BitWidth::Width64)),
                            _ => return Err(Error::UnsupportedIntSize),
                        };
                        parse_state.define_id(id_result, id_state)?;
                        if words.is_empty() {
                            Ok(Instruction::TypeInt {
                                id_result,
                                width,
                                signedness,
                            })
                        } else {
                            Err(Error::InstructionTooLong)
                        }
                    };
                    instruction_parse_case = quote! {
                        #opcode => {
                            let (id_result, words) = IdResult::spirv_parse(words, parse_state)?;
                            let (width, words) = LiteralInteger32::spirv_parse(words, parse_state)?;
                            #body
                        }
                    };
                    instruction_display_case = quote! {
                        Instruction::TypeInt {
                            id_result,
                            width,
                            signedness,
                        } => {
                            write!(
                                f,
                                "{}{}",
                                InstructionIndentAndResult(Some(*id_result)),
                                "OpTypeInt"
                            )?;
                            width.spirv_display(f)?;
                            signedness.spirv_display(f)?;
                            writeln!(f)
                        }
                    };
                }
                ast::InstructionName::OpTypeFloat => {
                    instruction_parse_case = quote! {
                        #opcode => {
                            let (id_result, words) = IdResult::spirv_parse(words, parse_state)?;
                            let (width, words) = LiteralInteger32::spirv_parse(words, parse_state)?;
                            let id_state = match width {
                                16 | 32 => IdState::Type(IdStateType(BitWidth::Width32OrLess)),
                                64 => IdState::Type(IdStateType(BitWidth::Width64)),
                                _ => return Err(Error::UnsupportedFloatSize),
                            };
                            parse_state.define_id(id_result, id_state)?;
                            if words.is_empty() {
                                Ok(Instruction::TypeFloat {
                                    id_result,
                                    width,
                                })
                            } else {
                                Err(Error::InstructionTooLong)
                            }
                        }
                    };
                    instruction_display_case = quote! {
                        Instruction::TypeFloat { id_result, width } => {
                            write!(
                                f,
                                "{}{}",
                                InstructionIndentAndResult(Some(*id_result)),
                                "OpTypeFloat"
                            )?;
                            width.spirv_display(f)?;
                            writeln!(f)
                        }
                    };
                }
                ast::InstructionName::OpSwitch32 => {
                    let body32 = quote! {
                        IdState::Value(IdStateValue(BitWidth::Width32OrLess)) => {
                            let (target, words) = Vec::<PairLiteralInteger32IdRef>::spirv_parse(words, parse_state)?;
                            if words.is_empty() {
                                Ok(Instruction::Switch32 {
                                    selector,
                                    default,
                                    target,
                                })
                            } else {
                                Err(Error::InstructionTooLong)
                            }
                        }
                    };
                    let body64 = quote! {
                        IdState::Value(IdStateValue(BitWidth::Width64)) => {
                            let (target, words) = Vec::<PairLiteralInteger64IdRef>::spirv_parse(words, parse_state)?;
                            if words.is_empty() {
                                Ok(Instruction::Switch64 {
                                    selector,
                                    default,
                                    target,
                                })
                            } else {
                                Err(Error::InstructionTooLong)
                            }
                        }
                    };
                    instruction_parse_case = quote! {
                        #opcode => {
                            let (selector, words) = IdRef::spirv_parse(words, parse_state)?;
                            let (default, words) = IdRef::spirv_parse(words, parse_state)?;
                            match &parse_state.id_states[selector.0 as usize] {
                                #body32
                                #body64
                                _ => Err(Error::SwitchSelectorIsInvalid(selector)),
                            }
                        }
                    };
                    instruction_display_case = quote! {
                        Instruction::Switch32 {
                            selector,
                            default,
                            target,
                        } => {
                            write!(
                                f,
                                "{}{}",
                                InstructionIndentAndResult(None),
                                "OpSwitch"
                            )?;
                            selector.spirv_display(f)?;
                            default.spirv_display(f)?;
                            target.spirv_display(f)?;
                            writeln!(f)
                        }
                        Instruction::Switch64 {
                            selector,
                            default,
                            target,
                        } => {
                            write!(
                                f,
                                "{}{}",
                                InstructionIndentAndResult(None),
                                "OpSwitch"
                            )?;
                            selector.spirv_display(f)?;
                            default.spirv_display(f)?;
                            target.spirv_display(f)?;
                            writeln!(f)
                        }
                    };
                }
                ast::InstructionName::OpSwitch64 => {
                    instruction_parse_case = quote! {};
                    instruction_display_case = quote! {};
                }
                ast::InstructionName::OpConstant32 => {
                    let body32 = quote! {
                        IdStateType(BitWidth::Width32OrLess) => {
                            let (value, words) = LiteralContextDependentNumber32::spirv_parse(words, parse_state)?;
                            if words.is_empty() {
                                Ok(Instruction::Constant32 {
                                    id_result_type,
                                    id_result,
                                    value,
                                })
                            } else {
                                Err(Error::InstructionTooLong)
                            }
                        }
                    };
                    let body64 = quote! {
                        IdStateType(BitWidth::Width64) => {
                            let (value, words) = LiteralContextDependentNumber64::spirv_parse(words, parse_state)?;
                            if words.is_empty() {
                                Ok(Instruction::Constant64 {
                                    id_result_type,
                                    id_result,
                                    value,
                                })
                            } else {
                                Err(Error::InstructionTooLong)
                            }
                        }
                    };
                    instruction_parse_case = quote! {
                        #opcode => {
                            let (id_result_type, words) = IdResultType::spirv_parse(words, parse_state)?;
                            let (id_result, words) = IdResult::spirv_parse(words, parse_state)?;
                            parse_state.define_value(id_result_type, id_result)?;
                            match parse_state.get_type(id_result_type.0)? {
                                #body32
                                #body64
                            }
                        }
                    };
                    instruction_display_case = quote! {
                        Instruction::Constant32 {
                            id_result_type,
                            id_result,
                            value,
                        } => {
                            write!(
                                f,
                                "{}{}",
                                InstructionIndentAndResult(Some(*id_result)),
                                "OpConstant"
                            )?;
                            id_result_type.spirv_display(f)?;
                            writeln!(f, " {:#010X}", value)
                        }
                        Instruction::Constant64 {
                            id_result_type,
                            id_result,
                            value,
                        } => {
                            write!(
                                f,
                                "{}{}",
                                InstructionIndentAndResult(Some(*id_result)),
                                "OpConstant"
                            )?;
                            id_result_type.spirv_display(f)?;
                            writeln!(f, " {:#018X}", value)
                        }
                    };
                }
                ast::InstructionName::OpConstant64 => {
                    instruction_parse_case = quote! {};
                    instruction_display_case = quote! {};
                }
                ast::InstructionName::OpSpecConstant32 => {
                    let body32 = quote! {
                        IdStateType(BitWidth::Width32OrLess) => {
                            let (value, words) = LiteralContextDependentNumber32::spirv_parse(words, parse_state)?;
                            if words.is_empty() {
                                Ok(Instruction::SpecConstant32 {
                                    id_result_type,
                                    id_result,
                                    value,
                                })
                            } else {
                                Err(Error::InstructionTooLong)
                            }
                        }
                    };
                    let body64 = quote! {
                        IdStateType(BitWidth::Width64) => {
                            let (value, words) = LiteralContextDependentNumber64::spirv_parse(words, parse_state)?;
                            if words.is_empty() {
                                Ok(Instruction::SpecConstant64 {
                                    id_result_type,
                                    id_result,
                                    value,
                                })
                            } else {
                                Err(Error::InstructionTooLong)
                            }
                        }
                    };
                    instruction_parse_case = quote! {
                        #opcode => {
                            let (id_result_type, words) = IdResultType::spirv_parse(words, parse_state)?;
                            let (id_result, words) = IdResult::spirv_parse(words, parse_state)?;
                            parse_state.define_value(id_result_type, id_result)?;
                            match parse_state.get_type(id_result_type.0)? {
                                #body32
                                #body64
                            }
                        }
                    };
                    instruction_display_case = quote! {
                        Instruction::SpecConstant32 {
                            id_result_type,
                            id_result,
                            value,
                        } => {
                            write!(
                                f,
                                "{}{}",
                                InstructionIndentAndResult(Some(*id_result)),
                                "OpSpecConstant"
                            )?;
                            id_result_type.spirv_display(f)?;
                            writeln!(f, " {:#010X}", value)
                        }
                        Instruction::SpecConstant64 {
                            id_result_type,
                            id_result,
                            value,
                        } => {
                            write!(
                                f,
                                "{}{}",
                                InstructionIndentAndResult(Some(*id_result)),
                                "OpSpecConstant"
                            )?;
                            id_result_type.spirv_display(f)?;
                            writeln!(f, " {:#018X}", value)
                        }
                    };
                }
                ast::InstructionName::OpSpecConstant64 => {
                    instruction_parse_case = quote! {};
                    instruction_display_case = quote! {};
                }
                ast::InstructionName::OpSpecConstantOp => {
                    instruction_parse_case = quote! {#opcode => {
                        let (operation, words) = OpSpecConstantOp::spirv_parse(words, parse_state)?;
                        if words.is_empty() {
                            Ok(Instruction::#opname { operation })
                        } else {
                            Err(Error::InstructionTooLong)
                        }
                    }};
                    instruction_display_case = quote! {
                        Instruction::#opname { operation } => fmt::Display::fmt(operation, f),
                    };
                }
                _ => {
                    let mut parse_operations = Vec::new();
                    let mut display_operations = Vec::new();
                    let mut operand_names = Vec::new();
                    let mut result_name = None;
                    for operand in &instruction.operands {
                        let kind = new_id(&operand.kind, CamelCase);
                        let name = new_id(operand.name.as_ref().unwrap(), SnakeCase);
                        let kind = match operand.quantifier {
                            None => quote! {#kind},
                            Some(ast::Quantifier::Optional) => quote! {Option::<#kind>},
                            Some(ast::Quantifier::Variadic) => quote! {Vec::<#kind>},
                        };
                        parse_operations.push(quote! {
                            let (#name, words) = #kind::spirv_parse(words, parse_state)?;
                        });
                        operand_names.push(name.clone());
                        if operand.kind == ast::Kind::IdResult {
                            assert_eq!(result_name, None);
                            result_name = Some(name);
                        } else {
                            display_operations.push(quote! {
                                #name.spirv_display(f)?;
                            });
                        }
                    }
                    if let Some([operand1, operand2]) = instruction.operands.get(..2) {
                        if operand1.kind == ast::Kind::IdResultType
                            && operand2.kind == ast::Kind::IdResult
                        {
                            let operand1_name = new_id(operand1.name.as_ref().unwrap(), SnakeCase);
                            let operand2_name = new_id(operand2.name.as_ref().unwrap(), SnakeCase);
                            parse_operations.push(quote! {
                                parse_state.define_value(#operand1_name, #operand2_name)?;
                            });
                        }
                    }
                    let operand_names = &operand_names;
                    instruction_parse_case = quote! {#opcode => {
                        #(#parse_operations)*
                        if words.is_empty() {
                            Ok(Instruction::#opname {
                                #(#operand_names,)*
                            })
                        } else {
                            Err(Error::InstructionTooLong)
                        }
                    }};
                    let result_value = match result_name {
                        None => quote! {None},
                        Some(result_name) => quote! {Some(*#result_name)},
                    };
                    instruction_display_case = quote! {
                        Instruction::#opname { #(#operand_names,)* } => {
                            write!(f, "{}{}", InstructionIndentAndResult(#result_value), #display_opname)?;
                            #(#display_operations)*
                            writeln!(f)
                        }
                    };
                }
            }
            instruction_parse_cases.push(instruction_parse_case);
            instruction_display_cases.push(instruction_display_case);
            let instruction_enumerant =
                if instruction.opname == ast::InstructionName::OpSpecConstantOp {
                    quote! {
                        #opname {
                            operation: OpSpecConstantOp,
                        }
                    }
                } else if instruction.operands.is_empty() {
                    quote! {#opname}
                } else {
                    let mut fields = Vec::new();
                    for operand in instruction.operands.iter() {
                        let kind = new_id(&operand.kind, CamelCase);
                        let name = new_id(operand.name.as_ref().unwrap(), SnakeCase);
                        let kind = match &operand.quantifier {
                            None => quote! {#kind},
                            Some(ast::Quantifier::Optional) => quote! {Option<#kind>},
                            Some(ast::Quantifier::Variadic) => quote! {Vec<#kind>},
                        };
                        fields.push(quote! {#name: #kind});
                    }
                    quote! {
                        #opname {
                            #(#fields,)*
                        }
                    }
                };
            if ast::OP_SPEC_CONSTANT_OP_SUPPORTED_INSTRUCTIONS.contains(&instruction.opname) {
                let opcode = u32::from(opcode);
                spec_constant_op_instruction_enumerants.push(instruction_enumerant.clone());
                let mut parse_operations = Vec::new();
                let mut display_operations = Vec::new();
                let mut operand_names = Vec::new();
                operand_names.push(new_id("id_result_type", SnakeCase));
                operand_names.push(new_id("id_result", SnakeCase));
                for operand in instruction.operands.iter().skip(2) {
                    let kind = new_id(&operand.kind, CamelCase);
                    let name = new_id(operand.name.as_ref().unwrap(), SnakeCase);
                    let kind = match operand.quantifier {
                        None => quote! {#kind},
                        Some(ast::Quantifier::Optional) => quote! {Option::<#kind>},
                        Some(ast::Quantifier::Variadic) => quote! {Vec::<#kind>},
                    };
                    parse_operations.push(quote! {
                        let (#name, words) = #kind::spirv_parse(words, parse_state)?;
                    });
                    display_operations.push(quote! {
                        #name.spirv_display(f)?;
                    });
                    operand_names.push(name);
                }
                if let Some([operand1, operand2]) = instruction.operands.get(..2) {
                    assert_eq!(operand1.kind, ast::Kind::IdResultType);
                    assert_eq!(operand2.kind, ast::Kind::IdResult);
                    let operand1_name = new_id(operand1.name.as_ref().unwrap(), SnakeCase);
                    let operand2_name = new_id(operand2.name.as_ref().unwrap(), SnakeCase);
                    parse_operations.push(quote! {
                        parse_state.define_value(#operand1_name, #operand2_name)?;
                    });
                } else {
                    assert!(
                        false,
                        "spec constant op is missing id_result_type and id_result"
                    );
                }
                let operand_names = &operand_names;
                instruction_spec_constant_parse_cases.push(quote! {#opcode => {
                    #(#parse_operations)*
                    if words.is_empty() {
                        Ok((OpSpecConstantOp::#opname {
                            #(#operand_names,)*
                        }, words))
                    } else {
                        Err(Error::InstructionTooLong)
                    }
                }});
                instruction_spec_constant_display_cases.push(quote!{
                    OpSpecConstantOp::#opname {
                        #(#operand_names,)*
                    } => {
                        write!(f, "{}{}", InstructionIndentAndResult(Some(*id_result)), "OpSpecConstantOp")?;
                        id_result_type.spirv_display(f)?;
                        write!(f, " {}", #display_opname_without_initial_op)?;
                        #(#display_operations)*
                        writeln!(f)
                    }
                });
            }
            instruction_enumerants.push(instruction_enumerant);
        }
        writeln!(
            &mut out,
            "{}",
            quote! {
                #[derive(Clone, Debug)]
                pub enum OpSpecConstantOp {
                    #(#spec_constant_op_instruction_enumerants,)*
                }
            }
        )?;
        writeln!(
            &mut out,
            "{}",
            quote! {
                impl fmt::Display for OpSpecConstantOp {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        match self {
                            #(#instruction_spec_constant_display_cases)*
                        }
                    }
                }
            }
        )?;
        writeln!(
            &mut out,
            "{}",
            quote! {
                #[derive(Clone, Debug)]
                pub enum Instruction {
                    #(#instruction_enumerants,)*
                    #(#instruction_extension_enumerants,)*
                }
            }
        )?;
        writeln!(
            &mut out,
            "{}",
            stringify!(
                #[derive(Copy, Clone, Debug)]
                pub struct Header {
                    pub version: (u32, u32),
                    pub generator: u32,
                    pub bound: u32,
                    pub instruction_schema: u32,
                }

                impl fmt::Display for Header {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        writeln!(f, "; SPIR-V")?;
                        writeln!(f, "; Version: {}.{}", self.version.0, self.version.1)?;
                        writeln!(f, "; Generator: {:#X}", self.generator)?;
                        writeln!(f, "; Bound: {}", self.bound)?;
                        writeln!(f, "; Schema: {}", self.instruction_schema)
                    }
                }

                struct InstructionIndentAndResult(Option<IdResult>);

                impl fmt::Display for InstructionIndentAndResult {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{:>15}", self.0.map(|v| format!("{} = ", v.0)).unwrap_or_default())
                    }
                }

                impl fmt::Display for IdRef {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "%{}", self.0)
                    }
                }

                #[derive(Clone, Debug)]
                pub enum Error {
                    MissingHeader,
                    InvalidHeader,
                    BoundTooBig(u32),
                    UnsupportedVersion(u32, u32),
                    ZeroInstructionLength,
                    SourcePrematurelyEnded,
                    UnknownOpcode(u16),
                    UnknownSpecConstantOpcode(u32),
                    UnknownExtensionOpcode(ExtensionInstructionSet, u32),
                    Utf8Error(Utf8Error),
                    InstructionPrematurelyEnded,
                    InvalidStringTermination,
                    InstructionTooLong,
                    InvalidEnumValue,
                    IdOutOfBounds(u32),
                    IdAlreadyDefined(IdResult),
                    UnsupportedFloatSize,
                    UnsupportedIntSize,
                    UndefinedType(IdRef),
                    SwitchSelectorIsInvalid(IdRef),
                    IdIsNotExtInstImport(IdRef),
                }

                impl From<Utf8Error> for Error {
                    fn from(v: Utf8Error) -> Self {
                        Error::Utf8Error(v)
                    }
                }

                impl From<FromUtf8Error> for Error {
                    fn from(v: FromUtf8Error) -> Self {
                        Error::Utf8Error(v.utf8_error())
                    }
                }

                impl fmt::Display for Error {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        match *self {
                            Error::MissingHeader => write!(f, "SPIR-V source is missing the file header"),
                            Error::InvalidHeader => write!(f, "SPIR-V source has an invalid file header"),
                            Error::BoundTooBig(bound) => write!(
                                f,
                                "SPIR-V source has an invalid file header; the id bound is way bigger than needed: {}",
                                bound,
                            ),
                            Error::UnsupportedVersion(major, minor) => write!(
                                f,
                                "SPIR-V source has an unsupported version: {}.{}",
                                major,
                                minor
                            ),
                            Error::ZeroInstructionLength => write!(f, "SPIR-V instruction has a length of zero"),
                            Error::SourcePrematurelyEnded => write!(f, "SPIR-V source prematurely ended"),
                            Error::UnknownOpcode(opcode) => {
                                write!(f, "SPIR-V instruction has an unknown opcode: {}", opcode)
                            }
                            Error::UnknownSpecConstantOpcode(opcode) => {
                                write!(f, "SPIR-V OpSpecConstantOp instruction has an unknown opcode: {}", opcode)
                            }
                            Error::UnknownExtensionOpcode(ref extension_instruction_set, opcode) => {
                                write!(f, "SPIR-V OpExtInst instruction has an unknown opcode: {} in {}", opcode, extension_instruction_set)
                            }
                            Error::Utf8Error(error) => fmt::Display::fmt(&error, f),
                            Error::InstructionPrematurelyEnded => write!(f, "SPIR-V instruction prematurely ended"),
                            Error::InvalidStringTermination => write!(f, "SPIR-V LiteralString has an invalid termination word"),
                            Error::InstructionTooLong => write!(f, "SPIR-V instruction is too long"),
                            Error::InvalidEnumValue => write!(f, "enum has invalid value"),
                            Error::IdOutOfBounds(id) => write!(f, "id is out of bounds: {}", id),
                            Error::IdAlreadyDefined(id) => write!(f, "id is already defined: {}", id),
                            Error::UnsupportedFloatSize => write!(f, "unsupported float size"),
                            Error::UnsupportedIntSize => write!(f, "unsupported int size"),
                            Error::UndefinedType(id) => write!(f, "undefined type {}", id),
                            Error::SwitchSelectorIsInvalid(id) => write!(f, "Switch selector is invalid: {}", id),
                            Error::IdIsNotExtInstImport(id) => write!(f, "id is not the result of an OpExtInstImport instruction: {}", id),
                        }
                    }
                }

                impl error::Error for Error {}

                type Result<T> = result::Result<T, Error>;

                #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
                enum BitWidth {
                    Width32OrLess,
                    Width64,
                }

                #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
                struct IdStateType(BitWidth);

                #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
                struct IdStateValue(BitWidth);

                #[derive(Clone, Debug, Eq, PartialEq, Hash)]
                enum IdState {
                    Unknown,
                    Type(IdStateType),
                    Value(IdStateValue),
                    ExtensionInstructionSet(ExtensionInstructionSet),
                }

                #[derive(Clone, Debug)]
                struct ParseState {
                    id_states: Vec<IdState>,
                }

                impl ParseState {
                    fn define_id(&mut self, id_result: IdResult, new_id_state: IdState) -> Result<()> {
                        let id_state = &mut self.id_states[(id_result.0).0 as usize];
                        if *id_state != IdState::Unknown {
                            return Err(Error::IdAlreadyDefined(id_result));
                        }
                        *id_state = new_id_state;
                        Ok(())
                    }
                    fn get_type(&self, id: IdRef) -> Result<IdStateType> {
                        if let IdState::Type(retval) = self.id_states[id.0 as usize] {
                            Ok(retval)
                        } else {
                            Err(Error::UndefinedType(id))
                        }
                    }
                    fn define_value(&mut self, id_result_type: IdResultType, id_result: IdResult) -> Result<()> {
                        if let IdState::Type(IdStateType(bit_width)) = self.id_states[(id_result_type.0).0 as usize] {
                            self.define_id(id_result, IdState::Value(IdStateValue(bit_width)))?;
                        }
                        Ok(())
                    }
                }

                #[derive(Clone, Debug)]
                pub struct Parser<'a> {
                    words: &'a [u32],
                    header: Header,
                    parse_state: ParseState,
                }

                fn parse_version(v: u32) -> Result<(u32, u32)> {
                    if (v & 0xFF0000FF) != 0 {
                        return Err(Error::InvalidHeader);
                    }
                    let major = (v >> 16) & 0xFF;
                    let minor = (v >> 8) & 0xFF;
                    Ok((major, minor))
                }

                impl<'a> Parser<'a> {
                    pub fn header(&self) -> &Header {
                        &self.header
                    }
                    pub fn start(mut words: &'a [u32]) -> Result<Self> {
                        let header = words.get(0..5).ok_or(Error::MissingHeader)?;
                        words = &words[5..];
                        let header = match *header {
                            [MAGIC_NUMBER, version, generator, bound, instruction_schema @ 0] if bound >= 1 => {
                                let version = parse_version(version)?;
                                if version.0 != MAJOR_VERSION || version.1 > MINOR_VERSION {
                                    return Err(Error::UnsupportedVersion(version.0, version.1));
                                }
                                Header {
                                    version,
                                    generator,
                                    bound,
                                    instruction_schema,
                                }
                            }
                            _ => return Err(Error::InvalidHeader),
                        };
                        if header.bound as usize > words.len() && header.bound > 0x10000 {
                            Err(Error::BoundTooBig(header.bound))
                        } else {
                            Ok(Self {
                                words,
                                header,
                                parse_state: ParseState {
                                    id_states: vec![IdState::Unknown; header.bound as usize],
                                },
                            })
                        }
                    }
                    fn next_helper(&mut self, length_and_opcode: u32) -> Result<Instruction> {
                        let length = (length_and_opcode >> 16) as usize;
                        let opcode = length_and_opcode as u16;
                        if length == 0 {
                            return Err(Error::ZeroInstructionLength);
                        }
                        let instruction_words = self.words.get(1..length).ok_or(Error::SourcePrematurelyEnded)?;
                        self.words = &self.words[length..];
                        parse_instruction(opcode, instruction_words, &mut self.parse_state)
                    }
                }

                impl<'a> Iterator for Parser<'a> {
                    type Item = Result<Instruction>;
                    fn next(&mut self) -> Option<Result<Instruction>> {
                        let length_and_opcode = self.words.get(0)?;
                        Some(self.next_helper(*length_and_opcode))
                    }
                }
            )
        )?;
        writeln!(
            &mut out,
            "{}",
            quote! {
                fn parse_instruction(opcode: u16, words: &[u32], parse_state: &mut ParseState) -> Result<Instruction> {
                    match opcode {
                        #(#instruction_parse_cases)*
                        opcode => Err(Error::UnknownOpcode(opcode)),
                    }
                }
            }
        )?;
        writeln!(
            &mut out,
            "{}",
            quote! {
                impl fmt::Display for Instruction {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        match self {
                            #(#instruction_display_cases)*
                            #(#instruction_extension_display_cases)*
                        }
                    }
                }
            }
        )?;
        let body = quote! {
            let (id_result_type, words) = IdResultType::spirv_parse(words, parse_state)?;
            let (id_result, words) = IdResult::spirv_parse(words, parse_state)?;
            let (opcode, words) = u32::spirv_parse(words, parse_state)?;
            match opcode {
                #(#instruction_spec_constant_parse_cases)*
                opcode => Err(Error::UnknownSpecConstantOpcode(opcode)),
            }
        };
        writeln!(
            &mut out,
            "{}",
            quote! {
                impl SPIRVParse for OpSpecConstantOp {
                    fn spirv_parse<'a>(
                        words: &'a [u32],
                        parse_state: &mut ParseState
                    ) -> Result<(Self, &'a [u32])> {
                        #body
                    }
                }
            }
        )?;
    }
    {
        let extension_instruction_set_enumerants: Vec<_> = parsed_extension_instruction_sets
            .iter()
            .map(|v| &v.enumerant_name)
            .collect();
        let extension_instruction_set_enumerants = &extension_instruction_set_enumerants;
        let spirv_instruction_set_names: Vec<_> = parsed_extension_instruction_sets
            .iter()
            .map(|v| v.spirv_instruction_set_name)
            .collect();
        let spirv_instruction_set_names = &spirv_instruction_set_names;
        for parsed_extension_instruction_set in parsed_extension_instruction_sets.iter() {
            let version_name = new_combined_id(
                &[
                    parsed_extension_instruction_set.spirv_instruction_set_name,
                    "version",
                ],
                UppercaseSnakeCase,
            );
            let version = parsed_extension_instruction_set.ast.version;
            let revision_name = new_combined_id(
                &[
                    parsed_extension_instruction_set.spirv_instruction_set_name,
                    "revision",
                ],
                UppercaseSnakeCase,
            );
            let revision = parsed_extension_instruction_set.ast.revision;
            writeln!(
                &mut out,
                "{}",
                quote! {
                    pub const #version_name: u32 = #version;
                    pub const #revision_name: u32 = #revision;
                }
            )?;
        }
        writeln!(
            &mut out,
            "{}",
            quote! {
                #[derive(Clone, Eq, PartialEq, Hash, Debug)]
                pub enum ExtensionInstructionSet {
                    #(#extension_instruction_set_enumerants,)*
                    Other(String),
                }
            }
        )?;
        writeln!(
            &mut out,
            "{}",
            quote! {
                impl<'a> From<Cow<'a, str>> for ExtensionInstructionSet {
                    fn from(s: Cow<'a, str>) -> ExtensionInstructionSet {
                        match s.as_ref() {
                            #(#spirv_instruction_set_names => return ExtensionInstructionSet::#extension_instruction_set_enumerants,)*
                            _ => {}
                        }
                        ExtensionInstructionSet::Other(s.into_owned())
                    }
                }
            }
        )?;
        writeln!(
            &mut out,
            "{}",
            quote! {
                impl Deref for ExtensionInstructionSet {
                    type Target = str;
                    fn deref(&self) -> &str {
                        match self {
                            #(ExtensionInstructionSet::#extension_instruction_set_enumerants => #spirv_instruction_set_names,)*
                            ExtensionInstructionSet::Other(s) => &**s,
                        }
                    }
                }
            }
        )?;
        writeln!(
            &mut out,
            "{}",
            stringify!(
                impl AsRef<str> for ExtensionInstructionSet {
                    fn as_ref(&self) -> &str {
                        &**self
                    }
                }

                impl From<ExtensionInstructionSet> for String {
                    fn from(v: ExtensionInstructionSet) -> String {
                        match v {
                            ExtensionInstructionSet::Other(v) => v,
                            v => String::from(v.as_ref()),
                        }
                    }
                }

                impl<'a> From<&'a str> for ExtensionInstructionSet {
                    fn from(s: &'a str) -> Self {
                        Cow::Borrowed(s).into()
                    }
                }

                impl From<String> for ExtensionInstructionSet {
                    fn from(s: String) -> Self {
                        Self::from(Cow::Owned(s))
                    }
                }

                impl fmt::Display for ExtensionInstructionSet {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        let s: &str = &**self;
                        fmt::Display::fmt(s, f)
                    }
                }
            )
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
