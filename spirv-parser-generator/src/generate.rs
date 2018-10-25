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
        stringify!(
            use std::result;
            use std::error;
            use std::fmt;
            use std::mem;
            use std::str::Utf8Error;
            use std::string::FromUtf8Error;

            trait SPIRVParse: Sized {
                fn spirv_parse<'a>(words: &'a [u32], parse_state: &mut ParseState)
                    -> Result<(Self, &'a [u32])>;
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

            struct ByteIterator<'a> {
                current_word: [u8; 4],
                bytes_left_in_current_word: usize,
                words: &'a [u32],
            }

            impl<'a> ByteIterator<'a> {
                fn new(words: &'a [u32]) -> Self {
                    Self {
                        current_word: [0; 4],
                        bytes_left_in_current_word: 0,
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
                    if self.bytes_left_in_current_word == 0 {
                        let (&current_word, words) = self.words.split_first()?;
                        self.words = words;
                        self.current_word = unsafe { mem::transmute(current_word.to_le()) };
                        self.bytes_left_in_current_word = self.current_word.len();
                    }
                    let byte = self.current_word[self.bytes_left_in_current_word];
                    self.bytes_left_in_current_word -= 1;
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
                    Ok((((high as u64) << 32) | low as u64, words))
                }
            }

            impl SPIRVParse for IdRef {
                fn spirv_parse<'a>(
                    words: &'a [u32],
                    parse_state: &mut ParseState,
                ) -> Result<(Self, &'a [u32])> {
                    let (value, words) = u32::spirv_parse(words, parse_state)?;
                    if value == 0 || value >= parse_state.bound {
                        Err(Error::IdOutOfBounds(value))
                    } else {
                        Ok((IdRef(value), words))
                    }
                }
            }
        )
    )?;
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
                let kind_id = new_id(kind, CamelCase);
                let mut enumerant_members = Vec::new();
                let mut enumerant_member_names = Vec::new();
                let mut enumerant_items = Vec::new();
                let mut enumerant_parse_operations = Vec::new();
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
                    let enumerant_value = enumerant.value;
                    enumerant_parse_operations.push(quote!{
                        let #member_name = if (mask & #enumerant_value) == 0 {
                            mask &= !#enumerant_value;
                            unimplemented!()
                        } else {
                            None
                        };
                    })
                }
                writeln!(
                    &mut out,
                    "{}",
                    quote!{
                        #[derive(Clone, Debug, Default)]
                        pub struct #kind_id {
                            #(#enumerant_members),*
                        }
                        #(#enumerant_items)*
                    }
                )?;
                let parse_body = quote!{
                    let (mask, words) = words.split_first().ok_or(Error::InstructionPrematurelyEnded)?;
                    let mut mask = *mask;
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
                    quote!{
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
                    let parameters = enumerant.parameters.iter().map(|parameter| {
                        let name = new_id(parameter.name.as_ref().unwrap(), SnakeCase);
                        let kind = new_id(&parameter.kind, CamelCase);
                        quote!{
                            #name: #kind,
                        }
                    });
                    generated_enumerants.push(quote!{
                        #name {
                            #(#parameters)*
                        }
                    });
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
                writeln!(
                    &mut out,
                    "{}",
                    quote!{
                        impl SPIRVParse for #kind_id {
                            fn spirv_parse<'a>(
                                words: &'a [u32],
                                parse_state: &mut ParseState,
                            ) -> Result<(Self, &'a [u32])> {
                                unimplemented!()
                            }
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
                if *kind != ast::Kind::IdRef {
                    writeln!(
                        &mut out,
                        "{}",
                        quote!{
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
                }
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
        let mut instruction_parse_cases = Vec::new();
        for instruction in core_instructions.iter() {
            let opcode = instruction.opcode;
            let opname = new_id(remove_initial_op(instruction.opname.as_ref()), CamelCase);
            instruction_parse_cases.push(match &instruction.opname {
                ast::InstructionName::OpSpecConstantOp => {
                    quote!{#opcode => {
                        let (operation, words) = OpSpecConstantOp::spirv_parse(words, parse_state)?;
                        if words.is_empty() {
                            Ok(Instruction::#opname { operation })
                        } else {
                            Err(Error::InstructionTooLong)
                        }
                    }}
                }
                _ => {
                    let mut parse_operations = Vec::new();
                    let mut operand_names = Vec::new();
                    for operand in &instruction.operands {
                        let kind = new_id(&operand.kind, CamelCase);
                        let name = new_id(operand.name.as_ref().unwrap(), SnakeCase);
                        let kind = match operand.quantifier {
                            None => quote!{#kind},
                            Some(ast::Quantifier::Optional) => quote!{Option::<#kind>},
                            Some(ast::Quantifier::Variadic) => quote!{Vec::<#kind>},
                        };
                        parse_operations.push(quote!{
                            let (#name, words) = #kind::spirv_parse(words, parse_state)?;
                        });
                        operand_names.push(name);
                    }
                    quote!{#opcode => {
                        #(#parse_operations)*
                        if words.is_empty() {
                            Ok(Instruction::#opname {
                                #(#operand_names,)*
                            })
                        } else {
                            Err(Error::InstructionTooLong)
                        }
                    }}
                }
            });
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
            }
        )?;
        writeln!(
            &mut out,
            "{}",
            quote!{
                #[derive(Clone, Debug)]
                pub enum Instruction {
                    #(#instruction_enumerants,)*
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

                #[derive(Clone, Debug)]
                pub enum Error {
                    MissingHeader,
                    InvalidHeader,
                    UnsupportedVersion(u32, u32),
                    ZeroInstructionLength,
                    SourcePrematurelyEnded,
                    UnknownOpcode(u16),
                    Utf8Error(Utf8Error),
                    InstructionPrematurelyEnded,
                    InvalidStringTermination,
                    InstructionTooLong,
                    InvalidEnumValue,
                    IdOutOfBounds(u32),
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
                            Error::Utf8Error(error) => fmt::Display::fmt(&error, f),
                            Error::InstructionPrematurelyEnded => write!(f, "SPIR-V instruction prematurely ended"),
                            Error::InvalidStringTermination => write!(f, "SPIR-V LiteralString has an invalid termination word"),
                            Error::InstructionTooLong => write!(f, "SPIR-V instruction is too long"),
                            Error::InvalidEnumValue => write!(f, "enum has invalid value"),
                            Error::IdOutOfBounds(id) => write!(f, "id is out of bounds: {}", id),
                        }
                    }
                }

                impl error::Error for Error {}

                type Result<T> = result::Result<T, Error>;

                #[derive(Clone, Debug)]
                struct ParseState {
                    bound: u32,
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
                        Ok(Self {
                            words,
                            header,
                            parse_state: ParseState {
                                bound: header.bound,
                            },
                        })
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
            quote!{
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
            quote!{
                impl SPIRVParse for OpSpecConstantOp {
                    fn spirv_parse<'a>(
                        words: &'a [u32],
                        parse_state: &mut ParseState
                    ) -> Result<(Self, &'a [u32])> {
                        let (id_result_type, words) = IdResultType::spirv_parse(words, parse_state)?;
                        let (id_result, words) = IdResult::spirv_parse(words, parse_state)?;
                        let (opcode, words) = u32::spirv_parse(words, parse_state)?;
                        unimplemented!()
                    }
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
