// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use crate::input_ir::{
    AluOperation, Instruction, Phi, ScalarType, TerminatingInstruction, Type, VirtualRegister, CFG,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::{Rc, Weak};
use std::str::FromStr;

#[derive(Copy, Clone, Debug)]
struct ParserInput<'a> {
    input_str: &'a str,
    current_position: TextPosition,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TextPosition {
    pub line_number: usize,
    pub byte_index: usize,
    pub line_start_byte_index: usize,
}

impl TextPosition {
    pub fn column(self) -> usize {
        self.byte_index - self.line_start_byte_index + 1
    }
}

impl fmt::Display for TextPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line_number, self.column())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct TextRange {
    pub start: TextPosition,
    pub end: TextPosition,
}

impl fmt::Display for TextRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.start, f)
    }
}

struct TextAndRange<'a> {
    text: &'a str,
    range: TextRange,
}

impl<'a> ParserInput<'a> {
    fn new(input_str: &'a str) -> Self {
        Self {
            input_str,
            current_position: TextPosition {
                byte_index: 0,
                line_number: 1,
                line_start_byte_index: 0,
            },
        }
    }
    fn peek(mut self) -> Option<char> {
        self.get()
    }
    fn get(&mut self) -> Option<char> {
        let mut chars = self.input_str[self.current_position.byte_index..].chars();
        let retval = chars.next();
        self.current_position.byte_index = self.input_str.len() - chars.as_str().len();
        if (retval == Some('\r') && chars.next() != Some('\n')) || retval == Some('\n') {
            self.current_position.line_number += 1;
            self.current_position.line_start_byte_index = self.current_position.byte_index;
        }
        retval
    }
    fn range(self, start_position: TextPosition) -> TextAndRange<'a> {
        TextAndRange {
            text: &self.input_str[start_position.byte_index..self.current_position.byte_index],
            range: TextRange {
                start: start_position,
                end: self.current_position,
            },
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
enum TokenKind {
    EndOfFile,
    Identifier,
    Integer,
    Colon,
    Percent,
    NewLine,
    LBrace,
    RBrace,
}

#[derive(Copy, Clone, Debug)]
struct Token<'a> {
    range: TextRange,
    kind: TokenKind,
    text: &'a str,
    numeric_value: Option<u64>,
}

#[derive(Clone, Debug)]
struct Tokenizer<'a> {
    input: ParserInput<'a>,
    next_result: Option<Result<Token<'a>, ParseError>>,
}

impl<'a> Tokenizer<'a> {
    fn get(&mut self) -> Result<Token<'a>, ParseError> {
        if let Some(result) = self.next_result.take() {
            return result;
        }
        loop {
            let start_position = self.input.current_position;
            let ch = match self.input.peek() {
                Some(v) => v,
                None => {
                    return Ok(Token {
                        range: self.input.range(start_position).range,
                        kind: TokenKind::EndOfFile,
                        text: "",
                        numeric_value: None,
                    });
                }
            };
            if ch == '\r' || ch == '\n' {
                self.input.get();
                if ch == '\r' && self.input.peek() == Some('\n') {
                    self.input.get();
                }
                return Ok(Token {
                    range: self.input.range(start_position).range,
                    kind: TokenKind::NewLine,
                    text: "\n",
                    numeric_value: None,
                });
            } else if ch.is_ascii_whitespace() {
                self.input.get();
            } else if ch == '#' || ch == ';' {
                while let Some(ch) = self.input.peek() {
                    if ch == '\r' || ch == '\n' {
                        break;
                    }
                    self.input.get();
                }
            } else {
                break;
            }
        }
        let start_position = self.input.current_position;
        let token = |input: ParserInput<'a>, kind: TokenKind, numeric_value| {
            let retval = input.range(start_position);
            Ok(Token {
                range: retval.range,
                kind,
                text: retval.text,
                numeric_value,
            })
        };
        match self.input.get().unwrap() {
            ':' => token(self.input, TokenKind::Colon, None),
            '%' => token(self.input, TokenKind::Percent, None),
            '{' => token(self.input, TokenKind::LBrace, None),
            '}' => token(self.input, TokenKind::RBrace, None),
            ch if ch.is_ascii_alphabetic() => {
                while self
                    .input
                    .peek()
                    .map(|v| v.is_ascii_alphanumeric() || v == '_')
                    .unwrap_or(false)
                {
                    self.input.get();
                }
                token(self.input, TokenKind::Identifier, None)
            }
            ch if ch.is_digit(10) => {
                while self.input.peek().map(|v| v.is_digit(10)).unwrap_or(false) {
                    self.input.get();
                }
                let numeric_value = match self.input.range(start_position).text.parse() {
                    Ok(numeric_value) => Some(numeric_value),
                    _ => parse_error(start_position, "number too big")?,
                };
                token(self.input, TokenKind::Integer, numeric_value)
            }
            ch => parse_error(start_position, format!("invalid character: {:?}", ch)),
        }
    }
    fn peek(&mut self) -> Result<Token<'a>, ParseError> {
        let retval = self.get();
        self.next_result = Some(retval.clone());
        retval
    }
    fn get_required_kind<S: ToString, F: FnOnce() -> S>(
        &mut self,
        required_kind: TokenKind,
        failure_message: F,
    ) -> Result<Token<'a>, ParseError> {
        let retval = self.get()?;
        if retval.kind != required_kind {
            parse_error(retval.range.start, failure_message())
        } else {
            Ok(retval)
        }
    }
    fn get_usize<S: ToString, F: FnOnce() -> S>(
        &mut self,
        failure_message: F,
    ) -> Result<(TextRange, usize), ParseError> {
        let Token {
            range,
            numeric_value,
            ..
        } = self.get_required_kind(TokenKind::Integer, failure_message)?;
        match numeric_value {
            Some(numeric_value) if numeric_value <= usize::max_value() as u64 => {
                Ok((range, numeric_value as usize))
            }
            _ => parse_error(range.start, "number too big"),
        }
    }
}

#[derive(Clone, Debug)]
struct Label<'a> {
    name_range: RefCell<TextRange>,
    name: &'a str,
    basic_block: RefCell<Weak<ParseBasicBlock<'a>>>,
}

impl Eq for Label<'_> {}

impl PartialEq for Label<'_> {
    fn eq(&self, rhs: &Self) -> bool {
        self.name == rhs.name
    }
}

impl Hash for Label<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct LabelReference<'a> {
    range: TextRange,
    label: Rc<Label<'a>>,
}

#[derive(Debug)]
struct ParseBasicBlockBody<'a> {
    phis: Vec<Phi<LabelReference<'a>>>,
    instructions: Vec<Instruction>,
    terminating_instruction: TerminatingInstruction<LabelReference<'a>>,
}

#[derive(Debug)]
struct ParseBasicBlock<'a> {
    label: Rc<Label<'a>>,
    body: RefCell<Option<ParseBasicBlockBody<'a>>>,
}

struct Parser<'a> {
    labels: HashMap<&'a str, Rc<Label<'a>>>,
    label_refs: Vec<LabelReference<'a>>,
}

enum ParseInstruction<'a> {
    Instruction(Instruction),
    Phi(Phi<LabelReference<'a>>),
    TerminatingInstruction(TerminatingInstruction<LabelReference<'a>>),
}

impl<'a> Parser<'a> {
    fn parse_label(
        &mut self,
        tokenizer: &mut Tokenizer<'a>,
    ) -> Result<LabelReference<'a>, ParseError> {
        let name = tokenizer.get_required_kind(TokenKind::Identifier, || "missing label")?;
        let retval = LabelReference {
            range: name.range,
            label: self
                .labels
                .entry(name.text)
                .or_insert_with(|| {
                    Rc::new(Label {
                        name_range: RefCell::new(name.range),
                        name: name.text,
                        basic_block: RefCell::new(Weak::new()),
                    })
                })
                .clone(),
        };
        self.label_refs.push(retval.clone());
        Ok(retval)
    }
    fn parse_scalar_type(
        &mut self,
        tokenizer: &mut Tokenizer<'a>,
    ) -> Result<(TextRange, ScalarType), ParseError> {
        let token = tokenizer.get()?;
        if token.kind == TokenKind::Identifier {
            if let Ok(retval) = token.text.parse() {
                return Ok((token.range, retval));
            }
        }
        parse_error(token.range.start, "expected type")
    }
    fn parse_compare_condition<T: FromStr>(
        &mut self,
        tokenizer: &mut Tokenizer<'a>,
    ) -> Result<T, ParseError>
    where
        T::Err: ToString,
    {
        let token =
            tokenizer.get_required_kind(TokenKind::Identifier, || "expected compare condition")?;
        match token.text.parse() {
            Ok(retval) => Ok(retval),
            Err(err) => parse_error(token.range.start, err),
        }
    }
    fn parse_type(
        &mut self,
        tokenizer: &mut Tokenizer<'a>,
    ) -> Result<(TextRange, Type), ParseError> {
        let (mut range, scalar_type) = self.parse_scalar_type(tokenizer)?;
        tokenizer.get_required_kind(TokenKind::LBrace, || "missing {")?;
        let register_count = tokenizer.get_usize(|| "missing register count")?.1;
        range.end = tokenizer
            .get_required_kind(TokenKind::RBrace, || "missing }")?
            .range
            .end;
        Ok((
            range,
            Type {
                scalar_type,
                register_count,
            },
        ))
    }
    fn parse_register(
        &mut self,
        tokenizer: &mut Tokenizer<'a>,
    ) -> Result<VirtualRegister, ParseError> {
        tokenizer.get_required_kind(TokenKind::Percent, || "expected register")?;
        Ok(VirtualRegister(
            tokenizer
                .get_usize(|| "expected virtual register number")?
                .1,
        ))
    }
    fn parse_register_and_type(
        &mut self,
        tokenizer: &mut Tokenizer<'a>,
    ) -> Result<(VirtualRegister, Type), ParseError> {
        let register = self.parse_register(tokenizer)?;
        Ok((register, self.parse_type(tokenizer)?.1))
    }
    fn parse_rest_of_alu_instruction(
        &mut self,
        tokenizer: &mut Tokenizer<'a>,
        result: VirtualRegister,
        result_type: Type,
        operation: AluOperation,
        argument_count: usize,
    ) -> Result<ParseInstruction<'a>, ParseError> {
        let mut sources = Vec::with_capacity(argument_count);
        for _ in 0..argument_count {
            sources.push(self.parse_register_and_type(tokenizer)?);
        }
        Ok(ParseInstruction::Instruction(Instruction::Alu {
            result,
            result_type,
            operation,
            sources,
        }))
    }
    fn parse_instruction(
        &mut self,
        tokenizer: &mut Tokenizer<'a>,
        can_be_phi: bool,
    ) -> Result<ParseInstruction<'a>, ParseError> {
        while tokenizer.peek()?.kind == TokenKind::NewLine {
            tokenizer.get()?;
        }
        let result_register_and_type = if tokenizer.peek()?.kind == TokenKind::Percent {
            Some(self.parse_register_and_type(tokenizer)?)
        } else {
            None
        };
        let opcode = tokenizer.get_required_kind(TokenKind::Identifier, || "expected opcode")?;
        let get_result_register_and_type = || {
            if let Some(result_register_and_type) = result_register_and_type {
                Ok(result_register_and_type)
            } else {
                parse_error(opcode.range.start, "missing result register")
            }
        };
        let no_result_register_and_type = || {
            if result_register_and_type.is_some() {
                parse_error(
                    opcode.range.start,
                    format!("{} instruction has no result", opcode.text),
                )
            } else {
                Ok(())
            }
        };
        let alu_op = |this: &mut Parser<'a>,
                      tokenizer: &mut Tokenizer<'a>,
                      operation: AluOperation,
                      argument_count: usize|
         -> Result<ParseInstruction<'a>, ParseError> {
            let (result, result_type) = get_result_register_and_type()?;
            this.parse_rest_of_alu_instruction(
                tokenizer,
                result,
                result_type,
                operation,
                argument_count,
            )
        };
        let retval = match opcode.text {
            "move" => {
                let (result, result_type) = get_result_register_and_type()?;
                let source = self.parse_register(tokenizer)?;
                ParseInstruction::Instruction(Instruction::Move {
                    result,
                    result_type,
                    source,
                })
            }
            "phi" => {
                let (result, result_type) = get_result_register_and_type()?;
                if !can_be_phi {
                    parse_error(
                        opcode.range.start,
                        "phi instructions must all be at beginning of basic block",
                    )?;
                }
                let mut sources = Vec::new();
                while tokenizer.peek()?.kind == TokenKind::LBrace {
                    tokenizer.get()?;
                    let label = self.parse_label(tokenizer)?;
                    let register = self.parse_register(tokenizer)?;
                    tokenizer.get_required_kind(TokenKind::RBrace, || "missing }")?;
                    sources.push((label, register));
                }
                ParseInstruction::Phi(Phi {
                    result,
                    result_type,
                    sources,
                })
            }
            "branch" => {
                no_result_register_and_type()?;
                let target = self.parse_label(tokenizer)?;
                ParseInstruction::TerminatingInstruction(TerminatingInstruction::Branch { target })
            }
            "switch" => {
                no_result_register_and_type()?;
                let selector = self.parse_register(tokenizer)?;
                let (selector_type_range, selector_type) = self.parse_type(tokenizer)?;
                let selector_type = match selector_type {
                    Type {
                        scalar_type: ScalarType::Int(v),
                        register_count: 1,
                    } => v,
                    _ => parse_error(
                        selector_type_range.start,
                        "switch selector must be a single-register integer",
                    )?,
                };
                let default = self.parse_label(tokenizer)?;
                let mut cases = Vec::new();
                while tokenizer.peek()?.kind == TokenKind::LBrace {
                    tokenizer.get()?;
                    let number = tokenizer
                        .get_required_kind(TokenKind::Integer, || "missing switch match value")?
                        .numeric_value
                        .unwrap();
                    let label = self.parse_label(tokenizer)?;
                    tokenizer.get_required_kind(TokenKind::RBrace, || "missing }")?;
                    cases.push((number, label));
                }
                ParseInstruction::TerminatingInstruction(TerminatingInstruction::Switch {
                    selector,
                    selector_type,
                    default,
                    cases,
                })
            }
            "unreachable" => {
                no_result_register_and_type()?;
                ParseInstruction::TerminatingInstruction(TerminatingInstruction::Unreachable)
            }
            "return" => {
                no_result_register_and_type()?;
                ParseInstruction::TerminatingInstruction(TerminatingInstruction::Return)
            }
            "icmp" => {
                let (result, result_type) = get_result_register_and_type()?;
                let compare_condition = self.parse_compare_condition(tokenizer)?;
                self.parse_rest_of_alu_instruction(
                    tokenizer,
                    result,
                    result_type,
                    AluOperation::ICmp(compare_condition),
                    2,
                )?
            }
            "fcmp" => {
                let (result, result_type) = get_result_register_and_type()?;
                let compare_condition = self.parse_compare_condition(tokenizer)?;
                self.parse_rest_of_alu_instruction(
                    tokenizer,
                    result,
                    result_type,
                    AluOperation::FCmp(compare_condition),
                    2,
                )?
            }
            "fneg" => alu_op(self, tokenizer, AluOperation::FNeg, 1)?,
            "fadd" => alu_op(self, tokenizer, AluOperation::FAdd, 2)?,
            "ffma" => alu_op(self, tokenizer, AluOperation::FFMA, 3)?,
            "uitofp" => alu_op(self, tokenizer, AluOperation::UIToFp, 1)?,
            "sitofp" => alu_op(self, tokenizer, AluOperation::SIToFp, 1)?,
            "fptoui" => alu_op(self, tokenizer, AluOperation::FpToUI, 1)?,
            "fptosi" => alu_op(self, tokenizer, AluOperation::FpToSI, 1)?,
            "ineg" => alu_op(self, tokenizer, AluOperation::INeg, 1)?,
            "iadd" => alu_op(self, tokenizer, AluOperation::IAdd, 2)?,
            "bitcast" => alu_op(self, tokenizer, AluOperation::Bitcast, 1)?,
            "trunc" => alu_op(self, tokenizer, AluOperation::Trunc, 1)?,
            "zext" => alu_op(self, tokenizer, AluOperation::ZExt, 1)?,
            "sext" => alu_op(self, tokenizer, AluOperation::SExt, 1)?,
            _ => parse_error(opcode.range.start, "unknown opcode")?,
        };
        match tokenizer.peek()?.kind {
            TokenKind::EndOfFile | TokenKind::NewLine => {}
            _ => parse_error(
                tokenizer.peek()?.range.start,
                "extra tokens after instruction",
            )?,
        }
        Ok(retval)
    }
    fn parse_basic_block(
        &mut self,
        tokenizer: &mut Tokenizer<'a>,
    ) -> Result<Rc<ParseBasicBlock<'a>>, ParseError> {
        while tokenizer.peek()?.kind == TokenKind::NewLine {
            tokenizer.get()?;
        }
        let label = self.parse_label(tokenizer)?;
        tokenizer.get_required_kind(TokenKind::Colon, || "missing ':' after label name")?;
        if let Some(basic_block) = label.label.basic_block.borrow().upgrade() {
            parse_error(
                label.range.start,
                format!(
                    "label already defined at {}",
                    basic_block.label.name_range.borrow().start
                ),
            )?;
        }
        let retval = Rc::new(ParseBasicBlock {
            label: label.label.clone(),
            body: RefCell::new(None),
        });
        *label.label.basic_block.borrow_mut() = Rc::downgrade(&retval);
        *label.label.name_range.borrow_mut() = label.range;
        let mut phis = Vec::new();
        let mut instructions = Vec::new();
        let terminating_instruction = loop {
            let can_be_phi = instructions.is_empty();
            match self.parse_instruction(tokenizer, can_be_phi)? {
                ParseInstruction::Phi(phi) => {
                    assert!(instructions.is_empty());
                    phis.push(phi);
                }
                ParseInstruction::Instruction(instruction) => instructions.push(instruction),
                ParseInstruction::TerminatingInstruction(terminating_instruction) => {
                    break terminating_instruction;
                }
            }
        };
        *retval.body.borrow_mut() = Some(ParseBasicBlockBody {
            phis,
            instructions,
            terminating_instruction,
        });
        Ok(retval)
    }
    fn parse_cfg(&mut self, tokenizer: &mut Tokenizer<'a>) -> Result<CFG, ParseError> {
        let mut basic_blocks = Vec::new();
        loop {
            basic_blocks.push(self.parse_basic_block(tokenizer)?);
            while tokenizer.peek()?.kind == TokenKind::NewLine {
                tokenizer.get()?;
            }
            if tokenizer.peek()?.kind == TokenKind::EndOfFile {
                break;
            }
        }
        for label_ref in self.label_refs.iter() {
            if label_ref.label.basic_block.borrow().upgrade().is_none() {
                parse_error(
                    label_ref.range.start,
                    format!("undefined label: {}", label_ref.label.name),
                )?;
            }
        }
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct ParseError {
    pub position: TextPosition,
    pub message: String,
}

fn parse_error<T: ToString, R>(position: TextPosition, message: T) -> Result<R, ParseError> {
    let parse_error = ParseError {
        position,
        message: message.to_string(),
    };
    #[cfg(test)]
    panic!("parse error: {}", parse_error);
    #[cfg(not(test))]
    return Err(parse_error);
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: error: {}", self.position, self.message)
    }
}

pub fn parse(input: &str) -> Result<CFG, ParseError> {
    let mut tokenizer = Tokenizer {
        input: ParserInput::new(input),
        next_result: None,
    };
    let retval = Parser {
        labels: HashMap::new(),
        label_refs: Vec::new(),
    }
    .parse_cfg(&mut tokenizer)?;
    let token = tokenizer.get()?;
    if token.kind == TokenKind::EndOfFile {
        Ok(retval)
    } else {
        parse_error(token.range.start, "extra input")?
    }
}
