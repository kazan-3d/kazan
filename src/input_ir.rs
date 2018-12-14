// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

mod parser;

use petgraph::prelude::*;
use petgraph::stable_graph::{IndexType, NodeIndex, StableGraph};
use std::error;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(transparent)]
pub struct CFGIndexType(usize);

unsafe impl IndexType for CFGIndexType {
    fn new(v: usize) -> Self {
        CFGIndexType(v)
    }
    fn index(&self) -> usize {
        self.0
    }
    fn max() -> Self {
        CFGIndexType(usize::max_value())
    }
}

pub type CFGNodeIndex = NodeIndex<CFGIndexType>;

pub type CFG = StableGraph<BasicBlock, (), Directed, CFGIndexType>;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VirtualRegister(usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum IntCompareCondition {
    Eq,
    NE,
    ULT,
    ULE,
    UGT,
    UGE,
    SLT,
    SLE,
    SGT,
    SGE,
}

impl fmt::Display for IntCompareCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl IntCompareCondition {
    pub fn name(self) -> &'static str {
        match self {
            IntCompareCondition::Eq => "eq",
            IntCompareCondition::NE => "ne",
            IntCompareCondition::ULT => "ult",
            IntCompareCondition::ULE => "ule",
            IntCompareCondition::UGT => "ugt",
            IntCompareCondition::UGE => "uge",
            IntCompareCondition::SLT => "slt",
            IntCompareCondition::SLE => "sle",
            IntCompareCondition::SGT => "sgt",
            IntCompareCondition::SGE => "sge",
        }
    }
}

#[derive(Clone, Debug)]
pub enum CompareConditionParseError {
    Float,
    Int,
}

impl fmt::Display for CompareConditionParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "invalid {} compare condition",
            match self {
                CompareConditionParseError::Float => "float",
                CompareConditionParseError::Int => "integer",
            }
        )
    }
}

impl error::Error for CompareConditionParseError {}

impl FromStr for IntCompareCondition {
    type Err = CompareConditionParseError;
    fn from_str(s: &str) -> Result<Self, CompareConditionParseError> {
        match s {
            "eq" => Ok(IntCompareCondition::Eq),
            "ne" => Ok(IntCompareCondition::NE),
            "ult" => Ok(IntCompareCondition::ULT),
            "ule" => Ok(IntCompareCondition::ULE),
            "ugt" => Ok(IntCompareCondition::UGT),
            "uge" => Ok(IntCompareCondition::UGE),
            "slt" => Ok(IntCompareCondition::SLT),
            "sle" => Ok(IntCompareCondition::SLE),
            "sgt" => Ok(IntCompareCondition::SGT),
            "sge" => Ok(IntCompareCondition::SGE),
            _ => Err(CompareConditionParseError::Int),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FpCompareCondition {
    less: bool,
    equal: bool,
    greater: bool,
    unordered: bool,
}

impl FpCompareCondition {
    pub const NEVER: FpCompareCondition = FpCompareCondition {
        less: false,
        equal: false,
        greater: false,
        unordered: false,
    };
    pub const UNORDERED: FpCompareCondition = FpCompareCondition {
        less: false,
        equal: false,
        greater: false,
        unordered: true,
    };
    pub const LT: FpCompareCondition = FpCompareCondition {
        less: true,
        equal: false,
        greater: false,
        unordered: false,
    };
    pub const LT_OR_UNORDERED: FpCompareCondition = FpCompareCondition {
        less: true,
        equal: false,
        greater: false,
        unordered: true,
    };
    pub const EQ: FpCompareCondition = FpCompareCondition {
        less: false,
        equal: true,
        greater: false,
        unordered: false,
    };
    pub const EQ_OR_UNORDERED: FpCompareCondition = FpCompareCondition {
        less: false,
        equal: true,
        greater: false,
        unordered: true,
    };
    pub const LE: FpCompareCondition = FpCompareCondition {
        less: true,
        equal: true,
        greater: false,
        unordered: false,
    };
    pub const LE_OR_UNORDERED: FpCompareCondition = FpCompareCondition {
        less: true,
        equal: true,
        greater: false,
        unordered: true,
    };
    pub const GT: FpCompareCondition = FpCompareCondition {
        less: false,
        equal: false,
        greater: true,
        unordered: false,
    };
    pub const GT_OR_UNORDERED: FpCompareCondition = FpCompareCondition {
        less: false,
        equal: false,
        greater: true,
        unordered: true,
    };
    pub const NE_AND_ORDERED: FpCompareCondition = FpCompareCondition {
        less: true,
        equal: false,
        greater: true,
        unordered: false,
    };
    pub const NE: FpCompareCondition = FpCompareCondition {
        less: true,
        equal: false,
        greater: true,
        unordered: true,
    };
    pub const GE: FpCompareCondition = FpCompareCondition {
        less: false,
        equal: true,
        greater: true,
        unordered: false,
    };
    pub const GE_OR_UNORDERED: FpCompareCondition = FpCompareCondition {
        less: false,
        equal: true,
        greater: true,
        unordered: true,
    };
    pub const ORDERED: FpCompareCondition = FpCompareCondition {
        less: true,
        equal: true,
        greater: true,
        unordered: false,
    };
    pub const ALWAYS: FpCompareCondition = FpCompareCondition {
        less: true,
        equal: true,
        greater: true,
        unordered: true,
    };
    pub fn name(self) -> &'static str {
        match self {
            FpCompareCondition::NEVER => "never",
            FpCompareCondition::UNORDERED => "uno",
            FpCompareCondition::LT => "olt",
            FpCompareCondition::LT_OR_UNORDERED => "ult",
            FpCompareCondition::EQ => "oeq",
            FpCompareCondition::EQ_OR_UNORDERED => "ueq",
            FpCompareCondition::LE => "ole",
            FpCompareCondition::LE_OR_UNORDERED => "ule",
            FpCompareCondition::GT => "ogt",
            FpCompareCondition::GT_OR_UNORDERED => "ugt",
            FpCompareCondition::NE_AND_ORDERED => "one",
            FpCompareCondition::NE => "une",
            FpCompareCondition::GE => "oge",
            FpCompareCondition::GE_OR_UNORDERED => "uge",
            FpCompareCondition::ORDERED => "ord",
            FpCompareCondition::ALWAYS => "always",
        }
    }
}

impl fmt::Display for FpCompareCondition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl FromStr for FpCompareCondition {
    type Err = CompareConditionParseError;
    fn from_str(s: &str) -> Result<Self, CompareConditionParseError> {
        match s {
            "never" => Ok(FpCompareCondition::NEVER),
            "uno" => Ok(FpCompareCondition::UNORDERED),
            "olt" => Ok(FpCompareCondition::LT),
            "ult" => Ok(FpCompareCondition::LT_OR_UNORDERED),
            "oeq" => Ok(FpCompareCondition::EQ),
            "ueq" => Ok(FpCompareCondition::EQ_OR_UNORDERED),
            "ole" => Ok(FpCompareCondition::LE),
            "ule" => Ok(FpCompareCondition::LE_OR_UNORDERED),
            "ogt" => Ok(FpCompareCondition::GT),
            "ugt" => Ok(FpCompareCondition::GT_OR_UNORDERED),
            "one" => Ok(FpCompareCondition::NE_AND_ORDERED),
            "une" => Ok(FpCompareCondition::NE),
            "oge" => Ok(FpCompareCondition::GE),
            "uge" => Ok(FpCompareCondition::GE_OR_UNORDERED),
            "ord" => Ok(FpCompareCondition::ORDERED),
            "always" => Ok(FpCompareCondition::ALWAYS),
            _ => Err(CompareConditionParseError::Float),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum AluOperation {
    FNeg,
    FAdd,
    FFMA,
    UIToFp,
    FpToUI,
    SIToFp,
    FpToSI,
    INeg,
    IAdd,
    ICmp(IntCompareCondition),
    FCmp(FpCompareCondition),
    Bitcast,
    Trunc,
    ZExt,
    SExt,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FpType {
    F16,
    F32,
    F64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum IntegerType {
    I8,
    I16,
    I32,
    I64,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ScalarType {
    Int(IntegerType),
    Fp(FpType),
    Bool,
}

#[derive(Clone, Debug)]
pub struct ScalarTypeParseError;

impl fmt::Display for ScalarTypeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt("invalid scalar type", f)
    }
}

impl error::Error for ScalarTypeParseError {}

impl FromStr for ScalarType {
    type Err = ScalarTypeParseError;
    fn from_str(s: &str) -> Result<Self, ScalarTypeParseError> {
        match s {
            "f16" => Ok(ScalarType::Fp(FpType::F16)),
            "f32" => Ok(ScalarType::Fp(FpType::F32)),
            "f64" => Ok(ScalarType::Fp(FpType::F64)),
            "i8" => Ok(ScalarType::Int(IntegerType::I8)),
            "i16" => Ok(ScalarType::Int(IntegerType::I16)),
            "i32" => Ok(ScalarType::Int(IntegerType::I32)),
            "i64" => Ok(ScalarType::Int(IntegerType::I64)),
            "bool" => Ok(ScalarType::Bool),
            _ => Err(ScalarTypeParseError),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Type {
    scalar_type: ScalarType,
    register_count: usize,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Instruction {
    Move {
        result: VirtualRegister,
        result_type: Type,
        source: VirtualRegister,
    },
    Alu {
        result: VirtualRegister,
        result_type: Type,
        operation: AluOperation,
        sources: Vec<(VirtualRegister, Type)>,
    },
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum TerminatingInstruction<LabelType = CFGNodeIndex> {
    Branch {
        target: LabelType,
    },
    Switch {
        selector: VirtualRegister,
        selector_type: IntegerType,
        default: LabelType,
        cases: Vec<(u64, LabelType)>,
    },
    Unreachable,
    Return,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Phi<LabelType = CFGNodeIndex> {
    result: VirtualRegister,
    result_type: Type,
    sources: Vec<(LabelType, VirtualRegister)>,
}

#[derive(Clone, Debug)]
pub struct BasicBlock<LabelType = CFGNodeIndex> {
    phis: Vec<Phi<LabelType>>,
    instructions: Vec<Instruction>,
    terminating_instruction: TerminatingInstruction<LabelType>,
}

pub use self::parser::{parse, ParseError, TextPosition, TextRange};
