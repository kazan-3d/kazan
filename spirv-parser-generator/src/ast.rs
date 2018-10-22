// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use serde::de::{self, Deserialize, Deserializer};
use std::fmt;
use util::NameFormat::*;
use util::WordIterator;

#[derive(Copy, Clone)]
pub enum QuotedInteger {
    U16Hex(u16),
    U32Hex(u32),
}

impl fmt::Display for QuotedInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            QuotedInteger::U16Hex(v) => write!(f, "{:#06X}", v),
            QuotedInteger::U32Hex(v) => write!(f, "{:#010X}", v),
        }
    }
}

impl fmt::Debug for QuotedInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct DisplayQuotedInteger(self::QuotedInteger);
        impl fmt::Debug for DisplayQuotedInteger {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
        #[derive(Debug)]
        struct QuotedInteger(DisplayQuotedInteger);
        QuotedInteger(DisplayQuotedInteger(*self)).fmt(f)
    }
}

impl<'de> Deserialize<'de> for QuotedInteger {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        let prefix = "0x";
        if !s.starts_with(prefix) {
            return Err(de::Error::custom(format!(
                "invalid quoted integer -- must start with {:?}",
                prefix
            )));
        }
        let digits = s.split_at(prefix.len()).1;
        let radix = 0x10;
        if digits.find(|c: char| !c.is_digit(radix)).is_some() {
            return Err(de::Error::custom(
                "invalid quoted integer -- not a hexadecimal digit",
            ));
        }
        let retval = match digits.len() {
            4 => QuotedInteger::U16Hex(u16::from_str_radix(digits, radix).unwrap()),
            8 => QuotedInteger::U32Hex(u32::from_str_radix(digits, radix).unwrap()),
            _ => {
                return Err(de::Error::custom(
                    "invalid quoted integer -- wrong number of hex digits",
                ));
            }
        };
        Ok(retval)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SPIRVVersion {
    Any,
    None,
    AtLeast { major: u32, minor: u32 },
}

impl Default for SPIRVVersion {
    fn default() -> Self {
        SPIRVVersion::Any
    }
}

impl<'de> Deserialize<'de> for SPIRVVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        if s == "None" {
            return Ok(SPIRVVersion::None);
        }
        let dot_pos = s
            .find('.')
            .ok_or_else(|| de::Error::custom("invalid SPIR-V version -- no decimal place"))?;
        let (major_digits, minor_digits) = s.split_at(dot_pos);
        let minor_digits = minor_digits.split_at(1).1;
        let parse_digits = |digits: &str| -> Result<u32, D::Error> {
            if digits == "" {
                return Err(de::Error::custom(
                    "invalid SPIR-V version -- expected a decimal digit",
                ));
            }
            if digits.find(|c: char| !c.is_ascii_digit()).is_some() {
                return Err(de::Error::custom(
                    "invalid SPIR-V version -- expected a decimal digit",
                ));
            }
            if digits.len() > 5 {
                return Err(de::Error::custom(
                    "invalid SPIR-V version -- too many digits",
                ));
            }
            Ok(digits.parse().unwrap())
        };
        let major = parse_digits(major_digits)?;
        let minor = parse_digits(minor_digits)?;
        Ok(SPIRVVersion::AtLeast { major, minor })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub enum Quantifier {
    #[serde(rename = "?")]
    Optional,
    #[serde(rename = "*")]
    Variadic,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstructionOperand {
    kind: String,
    name: Option<String>,
    quantifier: Option<Quantifier>,
}

impl InstructionOperand {
    pub fn guess_name(&mut self) -> Result<(), ::Error> {
        if self.name.is_none() {
            self.name = Some(
                SnakeCase
                    .name_from_words(WordIterator::new(&self.kind))
                    .ok_or(::Error::DeducingNameForInstructionOperandFailed)?,
            );
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    opname: String,
    opcode: u16,
    #[serde(default)]
    operands: Vec<InstructionOperand>,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    version: SPIRVVersion,
}

impl Instruction {
    pub fn guess_names(&mut self) -> Result<(), ::Error> {
        for operand in self.operands.iter_mut() {
            operand.guess_name()?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExtensionInstruction {
    opname: String,
    opcode: u16,
    #[serde(default)]
    operands: Vec<InstructionOperand>,
    #[serde(default)]
    capabilities: Vec<String>,
}

impl ExtensionInstruction {
    pub fn guess_names(&mut self) -> Result<(), ::Error> {
        for operand in self.operands.iter_mut() {
            operand.guess_name()?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EnumerantParameter {
    kind: String,
    name: Option<String>,
}

impl EnumerantParameter {
    pub fn guess_name(&mut self) -> Result<(), ::Error> {
        if self.name.is_none() {
            self.name = Some(
                SnakeCase
                    .name_from_words(WordIterator::new(&self.kind))
                    .ok_or(::Error::DeducingNameForEnumerantParameterFailed)?,
            );
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Enumerant<Value> {
    enumerant: String,
    value: Value,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    parameters: Vec<EnumerantParameter>,
    #[serde(default)]
    extensions: Vec<String>,
    #[serde(default)]
    version: SPIRVVersion,
}

impl<Value> Enumerant<Value> {
    pub fn guess_names(&mut self) -> Result<(), ::Error> {
        for parameter in self.parameters.iter_mut() {
            parameter.guess_name()?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(tag = "category")]
pub enum OperandKind {
    BitEnum {
        kind: String,
        enumerants: Vec<Enumerant<QuotedInteger>>,
    },
    ValueEnum {
        kind: String,
        enumerants: Vec<Enumerant<u32>>,
    },
    Id {
        kind: String,
        doc: Option<String>,
    },
    Literal {
        kind: String,
        doc: Option<String>,
    },
    Composite {
        kind: String,
        bases: Vec<String>,
    },
}

impl OperandKind {
    pub fn guess_names(&mut self) -> Result<(), ::Error> {
        match self {
            OperandKind::BitEnum { enumerants, .. } => {
                for enumerant in enumerants.iter_mut() {
                    enumerant.guess_names()?;
                }
            }
            OperandKind::ValueEnum { enumerants, .. } => {
                for enumerant in enumerants.iter_mut() {
                    enumerant.guess_names()?;
                }
            }
            OperandKind::Id { .. }
            | OperandKind::Literal { .. }
            | OperandKind::Composite { .. } => {}
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CoreGrammar {
    copyright: Vec<String>,
    magic_number: QuotedInteger,
    major_version: u16,
    minor_version: u16,
    revision: u32,
    instructions: Vec<Instruction>,
    operand_kinds: Vec<OperandKind>,
}

impl CoreGrammar {
    pub fn guess_names(&mut self) -> Result<(), ::Error> {
        for instruction in self.instructions.iter_mut() {
            instruction.guess_names()?;
        }
        for operand_kind in self.operand_kinds.iter_mut() {
            operand_kind.guess_names()?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExtensionInstructionSet {
    copyright: Vec<String>,
    version: u32,
    revision: u32,
    instructions: Vec<ExtensionInstruction>,
}

impl ExtensionInstructionSet {
    pub fn guess_names(&mut self) -> Result<(), ::Error> {
        for instruction in self.instructions.iter_mut() {
            instruction.guess_names()?;
        }
        Ok(())
    }
}
