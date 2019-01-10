// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use crate::util::NameFormat::*;
use crate::util::WordIterator;
use proc_macro2::TokenStream;
use quote::ToTokens;
use serde::de::{self, Deserialize, Deserializer};
use std::borrow::Cow;
use std::fmt;
use std::mem;
use serde_derive::Deserialize;

#[derive(Copy, Clone)]
pub struct QuotedInteger(pub u32);

impl ToTokens for QuotedInteger {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl fmt::Display for QuotedInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#06X}", self.0)
    }
}

impl fmt::Debug for QuotedInteger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct DisplayQuotedInteger(QuotedInteger);
        impl fmt::Debug for DisplayQuotedInteger {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
        f.debug_tuple("QuotedInteger")
            .field(&DisplayQuotedInteger(*self))
            .finish()
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
        if digits.len() > 8 {
            return Err(de::Error::custom(
                "invalid quoted integer -- too many hexadecimal digits",
            ));
        }
        Ok(QuotedInteger(u32::from_str_radix(digits, radix).unwrap()))
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

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct InstructionOperand {
    pub kind: Kind,
    pub name: Option<String>,
    pub quantifier: Option<Quantifier>,
}

impl InstructionOperand {
    pub fn fixup(&mut self) -> Result<(), crate::Error> {
        if let Some(name) = self.name.take() {
            let substitute_name = match &*name {
                "'Member 0 type', +\n'member 1 type', +\n..." => Some("Member Types"),
                "'Parameter 0 Type', +\n'Parameter 1 Type', +\n..." => Some("Parameter Types"),
                "'Argument 0', +\n'Argument 1', +\n..." => Some("Arguments"),
                "'Operand 1', +\n'Operand 2', +\n..." => Some("Operands"),
                _ => None,
            };
            self.name = Some(substitute_name.map(String::from).unwrap_or(name));
        } else {
            self.name = Some(
                SnakeCase
                    .name_from_words(WordIterator::new(self.kind.as_ref()))
                    .ok_or(crate::Error::DeducingNameForInstructionOperandFailed)?,
            );
        }
        self.kind.set_bit_width(BitWidth::Bits32);
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum InstructionName {
    OpSwitch,
    OpSwitch32,
    OpSwitch64,
    OpConstant,
    OpConstant32,
    OpConstant64,
    OpSpecConstant,
    OpSpecConstant32,
    OpSpecConstant64,
    OpSpecConstantOp,
    OpAccessChain,
    OpBitcast,
    OpBitwiseAnd,
    OpBitwiseOr,
    OpBitwiseXor,
    OpCompositeExtract,
    OpCompositeInsert,
    OpConvertFToS,
    OpConvertFToU,
    OpConvertPtrToU,
    OpConvertSToF,
    OpConvertUToF,
    OpConvertUToPtr,
    OpFAdd,
    OpFConvert,
    OpFDiv,
    OpFMod,
    OpFMul,
    OpFNegate,
    OpFRem,
    OpFSub,
    OpGenericCastToPtr,
    OpIAdd,
    OpIEqual,
    OpIMul,
    OpINotEqual,
    OpISub,
    OpInBoundsAccessChain,
    OpInBoundsPtrAccessChain,
    OpLogicalAnd,
    OpLogicalEqual,
    OpLogicalNot,
    OpLogicalNotEqual,
    OpLogicalOr,
    OpNot,
    OpPtrAccessChain,
    OpPtrCastToGeneric,
    OpQuantizeToF16,
    OpSConvert,
    OpSDiv,
    OpSGreaterThan,
    OpSGreaterThanEqual,
    OpSLessThan,
    OpSLessThanEqual,
    OpSMod,
    OpSNegate,
    OpSRem,
    OpSelect,
    OpShiftLeftLogical,
    OpShiftRightArithmetic,
    OpShiftRightLogical,
    OpUConvert,
    OpUDiv,
    OpUGreaterThan,
    OpUGreaterThanEqual,
    OpULessThan,
    OpULessThanEqual,
    OpUMod,
    OpVectorShuffle,
    OpTypeInt,
    OpTypeFloat,
    OpExtInstImport,
    OpExtInst,
    Other(String),
}

pub const OP_SPEC_CONSTANT_OP_SUPPORTED_INSTRUCTIONS: &[InstructionName] = &[
    InstructionName::OpAccessChain,
    InstructionName::OpBitcast,
    InstructionName::OpBitwiseAnd,
    InstructionName::OpBitwiseOr,
    InstructionName::OpBitwiseXor,
    InstructionName::OpCompositeExtract,
    InstructionName::OpCompositeInsert,
    InstructionName::OpConvertFToS,
    InstructionName::OpConvertFToU,
    InstructionName::OpConvertPtrToU,
    InstructionName::OpConvertSToF,
    InstructionName::OpConvertUToF,
    InstructionName::OpConvertUToPtr,
    InstructionName::OpFAdd,
    InstructionName::OpFConvert,
    InstructionName::OpFDiv,
    InstructionName::OpFMod,
    InstructionName::OpFMul,
    InstructionName::OpFNegate,
    InstructionName::OpFRem,
    InstructionName::OpFSub,
    InstructionName::OpGenericCastToPtr,
    InstructionName::OpIAdd,
    InstructionName::OpIEqual,
    InstructionName::OpIMul,
    InstructionName::OpINotEqual,
    InstructionName::OpISub,
    InstructionName::OpInBoundsAccessChain,
    InstructionName::OpInBoundsPtrAccessChain,
    InstructionName::OpLogicalAnd,
    InstructionName::OpLogicalEqual,
    InstructionName::OpLogicalNot,
    InstructionName::OpLogicalNotEqual,
    InstructionName::OpLogicalOr,
    InstructionName::OpNot,
    InstructionName::OpPtrAccessChain,
    InstructionName::OpPtrCastToGeneric,
    InstructionName::OpQuantizeToF16,
    InstructionName::OpSConvert,
    InstructionName::OpSDiv,
    InstructionName::OpSGreaterThan,
    InstructionName::OpSGreaterThanEqual,
    InstructionName::OpSLessThan,
    InstructionName::OpSLessThanEqual,
    InstructionName::OpSMod,
    InstructionName::OpSNegate,
    InstructionName::OpSRem,
    InstructionName::OpSelect,
    InstructionName::OpShiftLeftLogical,
    InstructionName::OpShiftRightArithmetic,
    InstructionName::OpShiftRightLogical,
    InstructionName::OpUConvert,
    InstructionName::OpUDiv,
    InstructionName::OpUGreaterThan,
    InstructionName::OpUGreaterThanEqual,
    InstructionName::OpULessThan,
    InstructionName::OpULessThanEqual,
    InstructionName::OpUMod,
    InstructionName::OpVectorShuffle,
];

impl Default for InstructionName {
    fn default() -> Self {
        InstructionName::Other(String::new())
    }
}

impl From<String> for InstructionName {
    fn from(v: String) -> Self {
        match &*v {
            "OpSwitch" => return InstructionName::OpSwitch,
            "OpConstant" => return InstructionName::OpConstant,
            "OpSpecConstant" => return InstructionName::OpSpecConstant,
            "OpSpecConstantOp" => return InstructionName::OpSpecConstantOp,
            "OpAccessChain" => return InstructionName::OpAccessChain,
            "OpBitcast" => return InstructionName::OpBitcast,
            "OpBitwiseAnd" => return InstructionName::OpBitwiseAnd,
            "OpBitwiseOr" => return InstructionName::OpBitwiseOr,
            "OpBitwiseXor" => return InstructionName::OpBitwiseXor,
            "OpCompositeExtract" => return InstructionName::OpCompositeExtract,
            "OpCompositeInsert" => return InstructionName::OpCompositeInsert,
            "OpConvertFToS" => return InstructionName::OpConvertFToS,
            "OpConvertFToU" => return InstructionName::OpConvertFToU,
            "OpConvertPtrToU" => return InstructionName::OpConvertPtrToU,
            "OpConvertSToF" => return InstructionName::OpConvertSToF,
            "OpConvertUToF" => return InstructionName::OpConvertUToF,
            "OpConvertUToPtr" => return InstructionName::OpConvertUToPtr,
            "OpFAdd" => return InstructionName::OpFAdd,
            "OpFConvert" => return InstructionName::OpFConvert,
            "OpFDiv" => return InstructionName::OpFDiv,
            "OpFMod" => return InstructionName::OpFMod,
            "OpFMul" => return InstructionName::OpFMul,
            "OpFNegate" => return InstructionName::OpFNegate,
            "OpFRem" => return InstructionName::OpFRem,
            "OpFSub" => return InstructionName::OpFSub,
            "OpGenericCastToPtr" => return InstructionName::OpGenericCastToPtr,
            "OpIAdd" => return InstructionName::OpIAdd,
            "OpIEqual" => return InstructionName::OpIEqual,
            "OpIMul" => return InstructionName::OpIMul,
            "OpINotEqual" => return InstructionName::OpINotEqual,
            "OpISub" => return InstructionName::OpISub,
            "OpInBoundsAccessChain" => return InstructionName::OpInBoundsAccessChain,
            "OpInBoundsPtrAccessChain" => return InstructionName::OpInBoundsPtrAccessChain,
            "OpLogicalAnd" => return InstructionName::OpLogicalAnd,
            "OpLogicalEqual" => return InstructionName::OpLogicalEqual,
            "OpLogicalNot" => return InstructionName::OpLogicalNot,
            "OpLogicalNotEqual" => return InstructionName::OpLogicalNotEqual,
            "OpLogicalOr" => return InstructionName::OpLogicalOr,
            "OpNot" => return InstructionName::OpNot,
            "OpPtrAccessChain" => return InstructionName::OpPtrAccessChain,
            "OpPtrCastToGeneric" => return InstructionName::OpPtrCastToGeneric,
            "OpQuantizeToF16" => return InstructionName::OpQuantizeToF16,
            "OpSConvert" => return InstructionName::OpSConvert,
            "OpSDiv" => return InstructionName::OpSDiv,
            "OpSGreaterThan" => return InstructionName::OpSGreaterThan,
            "OpSGreaterThanEqual" => return InstructionName::OpSGreaterThanEqual,
            "OpSLessThan" => return InstructionName::OpSLessThan,
            "OpSLessThanEqual" => return InstructionName::OpSLessThanEqual,
            "OpSMod" => return InstructionName::OpSMod,
            "OpSNegate" => return InstructionName::OpSNegate,
            "OpSRem" => return InstructionName::OpSRem,
            "OpSelect" => return InstructionName::OpSelect,
            "OpShiftLeftLogical" => return InstructionName::OpShiftLeftLogical,
            "OpShiftRightArithmetic" => return InstructionName::OpShiftRightArithmetic,
            "OpShiftRightLogical" => return InstructionName::OpShiftRightLogical,
            "OpUConvert" => return InstructionName::OpUConvert,
            "OpUDiv" => return InstructionName::OpUDiv,
            "OpUGreaterThan" => return InstructionName::OpUGreaterThan,
            "OpUGreaterThanEqual" => return InstructionName::OpUGreaterThanEqual,
            "OpULessThan" => return InstructionName::OpULessThan,
            "OpULessThanEqual" => return InstructionName::OpULessThanEqual,
            "OpUMod" => return InstructionName::OpUMod,
            "OpVectorShuffle" => return InstructionName::OpVectorShuffle,
            "OpTypeInt" => return InstructionName::OpTypeInt,
            "OpTypeFloat" => return InstructionName::OpTypeFloat,
            "OpExtInstImport" => return InstructionName::OpExtInstImport,
            "OpExtInst" => return InstructionName::OpExtInst,
            _ => {}
        }
        InstructionName::Other(v)
    }
}

impl AsRef<str> for InstructionName {
    fn as_ref(&self) -> &str {
        match self {
            InstructionName::OpSwitch => "OpSwitch",
            InstructionName::OpSwitch32 => "OpSwitch32",
            InstructionName::OpSwitch64 => "OpSwitch64",
            InstructionName::OpConstant => "OpConstant",
            InstructionName::OpConstant32 => "OpConstant32",
            InstructionName::OpConstant64 => "OpConstant64",
            InstructionName::OpSpecConstant => "OpSpecConstant",
            InstructionName::OpSpecConstant32 => "OpSpecConstant32",
            InstructionName::OpSpecConstant64 => "OpSpecConstant64",
            InstructionName::OpSpecConstantOp => "OpSpecConstantOp",
            InstructionName::OpAccessChain => "OpAccessChain",
            InstructionName::OpBitcast => "OpBitcast",
            InstructionName::OpBitwiseAnd => "OpBitwiseAnd",
            InstructionName::OpBitwiseOr => "OpBitwiseOr",
            InstructionName::OpBitwiseXor => "OpBitwiseXor",
            InstructionName::OpCompositeExtract => "OpCompositeExtract",
            InstructionName::OpCompositeInsert => "OpCompositeInsert",
            InstructionName::OpConvertFToS => "OpConvertFToS",
            InstructionName::OpConvertFToU => "OpConvertFToU",
            InstructionName::OpConvertPtrToU => "OpConvertPtrToU",
            InstructionName::OpConvertSToF => "OpConvertSToF",
            InstructionName::OpConvertUToF => "OpConvertUToF",
            InstructionName::OpConvertUToPtr => "OpConvertUToPtr",
            InstructionName::OpFAdd => "OpFAdd",
            InstructionName::OpFConvert => "OpFConvert",
            InstructionName::OpFDiv => "OpFDiv",
            InstructionName::OpFMod => "OpFMod",
            InstructionName::OpFMul => "OpFMul",
            InstructionName::OpFNegate => "OpFNegate",
            InstructionName::OpFRem => "OpFRem",
            InstructionName::OpFSub => "OpFSub",
            InstructionName::OpGenericCastToPtr => "OpGenericCastToPtr",
            InstructionName::OpIAdd => "OpIAdd",
            InstructionName::OpIEqual => "OpIEqual",
            InstructionName::OpIMul => "OpIMul",
            InstructionName::OpINotEqual => "OpINotEqual",
            InstructionName::OpISub => "OpISub",
            InstructionName::OpInBoundsAccessChain => "OpInBoundsAccessChain",
            InstructionName::OpInBoundsPtrAccessChain => "OpInBoundsPtrAccessChain",
            InstructionName::OpLogicalAnd => "OpLogicalAnd",
            InstructionName::OpLogicalEqual => "OpLogicalEqual",
            InstructionName::OpLogicalNot => "OpLogicalNot",
            InstructionName::OpLogicalNotEqual => "OpLogicalNotEqual",
            InstructionName::OpLogicalOr => "OpLogicalOr",
            InstructionName::OpNot => "OpNot",
            InstructionName::OpPtrAccessChain => "OpPtrAccessChain",
            InstructionName::OpPtrCastToGeneric => "OpPtrCastToGeneric",
            InstructionName::OpQuantizeToF16 => "OpQuantizeToF16",
            InstructionName::OpSConvert => "OpSConvert",
            InstructionName::OpSDiv => "OpSDiv",
            InstructionName::OpSGreaterThan => "OpSGreaterThan",
            InstructionName::OpSGreaterThanEqual => "OpSGreaterThanEqual",
            InstructionName::OpSLessThan => "OpSLessThan",
            InstructionName::OpSLessThanEqual => "OpSLessThanEqual",
            InstructionName::OpSMod => "OpSMod",
            InstructionName::OpSNegate => "OpSNegate",
            InstructionName::OpSRem => "OpSRem",
            InstructionName::OpSelect => "OpSelect",
            InstructionName::OpShiftLeftLogical => "OpShiftLeftLogical",
            InstructionName::OpShiftRightArithmetic => "OpShiftRightArithmetic",
            InstructionName::OpShiftRightLogical => "OpShiftRightLogical",
            InstructionName::OpUConvert => "OpUConvert",
            InstructionName::OpUDiv => "OpUDiv",
            InstructionName::OpUGreaterThan => "OpUGreaterThan",
            InstructionName::OpUGreaterThanEqual => "OpUGreaterThanEqual",
            InstructionName::OpULessThan => "OpULessThan",
            InstructionName::OpULessThanEqual => "OpULessThanEqual",
            InstructionName::OpUMod => "OpUMod",
            InstructionName::OpVectorShuffle => "OpVectorShuffle",
            InstructionName::OpTypeInt => "OpTypeInt",
            InstructionName::OpTypeFloat => "OpTypeFloat",
            InstructionName::OpExtInstImport => "OpExtInstImport",
            InstructionName::OpExtInst => "OpExtInst",
            InstructionName::Other(v) => v,
        }
    }
}

impl<'de> Deserialize<'de> for InstructionName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(deserializer)?))
    }
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Instruction {
    pub opname: InstructionName,
    pub opcode: u16,
    #[serde(default)]
    pub operands: Vec<InstructionOperand>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub version: SPIRVVersion,
}

impl Instruction {
    pub fn fixup(&mut self) -> Result<(), crate::Error> {
        for operand in self.operands.iter_mut() {
            operand.fixup()?;
        }
        if self.opname == InstructionName::OpExtInst {
            assert_eq!(self.operands.len(), 5);
            assert_eq!(self.operands[4].kind, Kind::IdRef);
            assert_eq!(self.operands[4].quantifier, Some(Quantifier::Variadic));
            self.operands[4].kind = Kind::Literal(LiteralKind::LiteralInteger32);
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExtensionInstruction {
    pub opname: String,
    pub opcode: u32,
    #[serde(default)]
    pub operands: Vec<InstructionOperand>,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

impl ExtensionInstruction {
    pub fn fixup(&mut self) -> Result<(), crate::Error> {
        for operand in self.operands.iter_mut() {
            operand.fixup()?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct BitwiseEnumerantParameter {
    pub kind: Kind,
}

impl BitwiseEnumerantParameter {
    pub fn fixup(&mut self) -> Result<(), crate::Error> {
        self.kind.set_bit_width(BitWidth::Bits32);
        Ok(())
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ValueEnumerantParameter {
    pub kind: Kind,
    pub name: Option<String>,
}

impl ValueEnumerantParameter {
    pub fn fixup(&mut self) -> Result<(), crate::Error> {
        if self.name.is_none() {
            self.name = Some(
                SnakeCase
                    .name_from_words(WordIterator::new(self.kind.as_ref()))
                    .ok_or(crate::Error::DeducingNameForEnumerantParameterFailed)?,
            );
        }
        self.kind.set_bit_width(BitWidth::Bits32);
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Enumerant<Value, EnumerantParameter> {
    pub enumerant: String,
    pub value: Value,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<EnumerantParameter>,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub version: SPIRVVersion,
}

impl Enumerant<u32, ValueEnumerantParameter> {
    pub fn fixup(&mut self) -> Result<(), crate::Error> {
        for parameter in self.parameters.iter_mut() {
            parameter.fixup()?;
        }
        Ok(())
    }
}

impl Enumerant<QuotedInteger, BitwiseEnumerantParameter> {
    pub fn fixup(&mut self) -> Result<(), crate::Error> {
        for parameter in self.parameters.iter_mut() {
            parameter.fixup()?;
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Kind {
    Literal(LiteralKind),
    IdRef,
    IdResult,
    IdResultType,
    PairLiteralIntegerIdRef,
    PairLiteralInteger32IdRef,
    PairLiteralInteger64IdRef,
    Other(String),
}

impl Kind {
    pub fn set_bit_width(&mut self, bit_width: BitWidth) {
        match (self, bit_width) {
            (Kind::Literal(literal), bit_width) => literal.set_bit_width(bit_width),
            (this @ Kind::PairLiteralIntegerIdRef, BitWidth::Bits32) => {
                *this = Kind::PairLiteralInteger32IdRef
            }
            (this @ Kind::PairLiteralIntegerIdRef, BitWidth::Bits64) => {
                *this = Kind::PairLiteralInteger64IdRef
            }
            (Kind::IdRef, _)
            | (Kind::IdResult, _)
            | (Kind::IdResultType, _)
            | (Kind::PairLiteralInteger32IdRef, _)
            | (Kind::PairLiteralInteger64IdRef, _)
            | (Kind::Other(_), _) => {}
        }
    }
}

impl Default for Kind {
    fn default() -> Self {
        Kind::Other(String::new())
    }
}

impl<'a> From<Cow<'a, str>> for Kind {
    fn from(v: Cow<'a, str>) -> Self {
        if let Some(v) = LiteralKind::from_str(&v) {
            Kind::Literal(v)
        } else if v == "IdRef" {
            Kind::IdRef
        } else if v == "IdResult" {
            Kind::IdResult
        } else if v == "IdResultType" {
            Kind::IdResultType
        } else if v == "PairLiteralIntegerIdRef" {
            Kind::PairLiteralIntegerIdRef
        } else {
            Kind::Other(v.into_owned())
        }
    }
}

impl<'a> From<&'a str> for Kind {
    fn from(v: &'a str) -> Self {
        Kind::from(Cow::Borrowed(v))
    }
}

impl From<String> for Kind {
    fn from(v: String) -> Self {
        Kind::from(Cow::Owned(v))
    }
}

impl AsRef<str> for Kind {
    fn as_ref(&self) -> &str {
        match self {
            Kind::Literal(v) => v.as_ref(),
            Kind::IdRef => "IdRef",
            Kind::IdResult => "IdResult",
            Kind::IdResultType => "IdResultType",
            Kind::PairLiteralIntegerIdRef => "PairLiteralIntegerIdRef",
            Kind::PairLiteralInteger32IdRef => "PairLiteralInteger32IdRef",
            Kind::PairLiteralInteger64IdRef => "PairLiteralInteger64IdRef",
            Kind::Other(v) => v,
        }
    }
}

impl<'de> Deserialize<'de> for Kind {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::from(String::deserialize(deserializer)?))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Deserialize)]
pub enum LiteralKind {
    LiteralInteger,
    #[serde(skip_deserializing)]
    LiteralInteger32,
    #[serde(skip_deserializing)]
    LiteralInteger64,
    LiteralString,
    LiteralContextDependentNumber,
    #[serde(skip_deserializing)]
    LiteralContextDependentNumber32,
    #[serde(skip_deserializing)]
    LiteralContextDependentNumber64,
    LiteralExtInstInteger,
    LiteralSpecConstantOpInteger,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum BitWidth {
    Bits32,
    Bits64,
}

impl LiteralKind {
    pub fn from_str<T: AsRef<str>>(v: T) -> Option<Self> {
        match v.as_ref() {
            "LiteralInteger" => Some(LiteralKind::LiteralInteger),
            "LiteralString" => Some(LiteralKind::LiteralString),
            "LiteralContextDependentNumber" => Some(LiteralKind::LiteralContextDependentNumber),
            "LiteralExtInstInteger" => Some(LiteralKind::LiteralExtInstInteger),
            "LiteralSpecConstantOpInteger" => Some(LiteralKind::LiteralSpecConstantOpInteger),
            _ => None,
        }
    }
    pub fn set_bit_width(&mut self, bit_width: BitWidth) {
        *self = match (*self, bit_width) {
            (LiteralKind::LiteralInteger, BitWidth::Bits32) => LiteralKind::LiteralInteger32,
            (LiteralKind::LiteralInteger, BitWidth::Bits64) => LiteralKind::LiteralInteger64,
            (LiteralKind::LiteralContextDependentNumber, BitWidth::Bits32) => {
                LiteralKind::LiteralContextDependentNumber32
            }
            (LiteralKind::LiteralContextDependentNumber, BitWidth::Bits64) => {
                LiteralKind::LiteralContextDependentNumber64
            }
            (LiteralKind::LiteralInteger32, _)
            | (LiteralKind::LiteralInteger64, _)
            | (LiteralKind::LiteralString, _)
            | (LiteralKind::LiteralContextDependentNumber32, _)
            | (LiteralKind::LiteralContextDependentNumber64, _)
            | (LiteralKind::LiteralExtInstInteger, _)
            | (LiteralKind::LiteralSpecConstantOpInteger, _) => return,
        }
    }
}

impl AsRef<str> for LiteralKind {
    fn as_ref(&self) -> &str {
        match self {
            LiteralKind::LiteralInteger => "LiteralInteger",
            LiteralKind::LiteralInteger32 => "LiteralInteger32",
            LiteralKind::LiteralInteger64 => "LiteralInteger64",
            LiteralKind::LiteralString => "LiteralString",
            LiteralKind::LiteralContextDependentNumber => "LiteralContextDependentNumber",
            LiteralKind::LiteralContextDependentNumber32 => "LiteralContextDependentNumber32",
            LiteralKind::LiteralContextDependentNumber64 => "LiteralContextDependentNumber64",
            LiteralKind::LiteralExtInstInteger => "LiteralExtInstInteger",
            LiteralKind::LiteralSpecConstantOpInteger => "LiteralSpecConstantOpInteger",
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(tag = "category")]
pub enum OperandKind {
    BitEnum {
        kind: Kind,
        enumerants: Vec<Enumerant<QuotedInteger, BitwiseEnumerantParameter>>,
    },
    ValueEnum {
        kind: Kind,
        enumerants: Vec<Enumerant<u32, ValueEnumerantParameter>>,
    },
    Id {
        kind: Kind,
        doc: Option<String>,
    },
    Literal {
        kind: LiteralKind,
        doc: Option<String>,
    },
    Composite {
        kind: Kind,
        bases: Vec<Kind>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct CoreGrammar {
    pub copyright: Vec<String>,
    pub magic_number: QuotedInteger,
    pub major_version: u32,
    pub minor_version: u32,
    pub revision: u32,
    pub instructions: Vec<Instruction>,
    pub operand_kinds: Vec<OperandKind>,
}

impl CoreGrammar {
    pub fn fixup(&mut self) -> Result<(), crate::Error> {
        let instructions = mem::replace(&mut self.instructions, Vec::new());
        for mut instruction in instructions {
            if instruction.version == SPIRVVersion::None {
                continue;
            }
            let (opname_32, opname_64) = match instruction.opname {
                InstructionName::OpSwitch => {
                    (InstructionName::OpSwitch32, InstructionName::OpSwitch64)
                }
                InstructionName::OpConstant => {
                    (InstructionName::OpConstant32, InstructionName::OpConstant64)
                }
                InstructionName::OpSpecConstant => (
                    InstructionName::OpSpecConstant32,
                    InstructionName::OpSpecConstant64,
                ),
                opname => {
                    instruction.opname = opname;
                    instruction.fixup()?;
                    self.instructions.push(instruction);
                    continue;
                }
            };
            instruction.opname = InstructionName::default();
            let mut op_32 = Instruction {
                opname: opname_32,
                ..instruction.clone()
            };
            for operand in op_32.operands.iter_mut() {
                operand.kind.set_bit_width(BitWidth::Bits32);
            }
            op_32.fixup()?;
            self.instructions.push(op_32);
            let mut op_64 = Instruction {
                opname: opname_64,
                ..instruction
            };
            for operand in op_64.operands.iter_mut() {
                operand.kind.set_bit_width(BitWidth::Bits64);
            }
            op_64.fixup()?;
            self.instructions.push(op_64);
        }
        let operand_kinds = mem::replace(&mut self.operand_kinds, Vec::new());
        for operand_kind in operand_kinds {
            match operand_kind {
                OperandKind::BitEnum {
                    kind,
                    mut enumerants,
                } => {
                    enumerants.retain(|enumerant| enumerant.version != SPIRVVersion::None);
                    for enumerant in enumerants.iter_mut() {
                        enumerant.fixup()?;
                    }
                    self.operand_kinds
                        .push(OperandKind::BitEnum { kind, enumerants });
                }
                OperandKind::ValueEnum {
                    kind,
                    mut enumerants,
                } => {
                    enumerants.retain(|enumerant| enumerant.version != SPIRVVersion::None);
                    for enumerant in enumerants.iter_mut() {
                        enumerant.fixup()?;
                    }
                    enumerants.sort_by_key(|enumerant| enumerant.value);
                    enumerants.dedup_by_key(|enumerant| enumerant.value);
                    self.operand_kinds
                        .push(OperandKind::ValueEnum { kind, enumerants });
                }
                OperandKind::Composite { kind, mut bases } => match kind {
                    Kind::PairLiteralIntegerIdRef => {
                        let mut bases_32 = bases.clone();
                        let mut bases_64 = bases;
                        for base in bases_32.iter_mut() {
                            base.set_bit_width(BitWidth::Bits32);
                        }
                        for base in bases_64.iter_mut() {
                            base.set_bit_width(BitWidth::Bits64);
                        }
                        self.operand_kinds.push(OperandKind::Composite {
                            kind: Kind::PairLiteralInteger32IdRef,
                            bases: bases_32,
                        });
                        self.operand_kinds.push(OperandKind::Composite {
                            kind: Kind::PairLiteralInteger64IdRef,
                            bases: bases_64,
                        });
                    }
                    kind => {
                        for base in bases.iter_mut() {
                            base.set_bit_width(BitWidth::Bits32);
                        }
                        self.operand_kinds
                            .push(OperandKind::Composite { kind, bases });
                    }
                },
                OperandKind::Literal { kind, doc } => match kind {
                    LiteralKind::LiteralInteger => {
                        self.operand_kinds.push(OperandKind::Literal {
                            kind: LiteralKind::LiteralInteger32,
                            doc: doc.clone(),
                        });
                        self.operand_kinds.push(OperandKind::Literal {
                            kind: LiteralKind::LiteralInteger64,
                            doc,
                        });
                    }
                    LiteralKind::LiteralContextDependentNumber => {
                        self.operand_kinds.push(OperandKind::Literal {
                            kind: LiteralKind::LiteralContextDependentNumber32,
                            doc: doc.clone(),
                        });
                        self.operand_kinds.push(OperandKind::Literal {
                            kind: LiteralKind::LiteralContextDependentNumber64,
                            doc,
                        });
                    }
                    kind => self.operand_kinds.push(OperandKind::Literal { kind, doc }),
                },
                OperandKind::Id { kind, doc } => {
                    self.operand_kinds.push(OperandKind::Id { kind, doc })
                }
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ExtensionInstructionSet {
    pub copyright: Vec<String>,
    pub version: u32,
    pub revision: u32,
    pub instructions: Vec<ExtensionInstruction>,
}

impl ExtensionInstructionSet {
    pub fn fixup(&mut self) -> Result<(), crate::Error> {
        for instruction in self.instructions.iter_mut() {
            instruction.fixup()?;
        }
        Ok(())
    }
}
