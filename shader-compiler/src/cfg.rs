// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use spirv_parser::{IdRef, IdResult, Instruction};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::mem;
use std::rc::{Rc, Weak};

pub(crate) trait GenericNode: Clone + fmt::Debug {
    fn instructions(&self) -> &Vec<Instruction>;
    fn to_node(this: Rc<Self>) -> Node;
    fn label(&self) -> IdRef;
}

#[derive(Clone, Debug)]
pub(crate) struct SimpleNode {
    pub(crate) label: IdRef,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) next: Node,
}

impl GenericNode for SimpleNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(this: Rc<Self>) -> Node {
        Node::Simple(this)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SwitchDefault {
    pub(crate) default_case: Node,
    pub(crate) after_default_cases: Vec<Node>,
}

#[derive(Clone, Debug)]
pub(crate) struct SwitchNode {
    pub(crate) label: IdRef,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) before_default_cases: Vec<Node>,
    pub(crate) default: Option<SwitchDefault>,
    pub(crate) next: Node,
}

impl GenericNode for SwitchNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(this: Rc<Self>) -> Node {
        Node::Switch(this)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SwitchFallthroughNode {
    pub(crate) label: IdRef,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) switch: RefCell<Weak<SwitchNode>>,
    pub(crate) target_label: IdRef,
}

impl GenericNode for SwitchFallthroughNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(this: Rc<Self>) -> Node {
        Node::SwitchFallthrough(this)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SwitchMergeNode {
    pub(crate) label: IdRef,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) switch: RefCell<Weak<SwitchNode>>,
}

impl GenericNode for SwitchMergeNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(this: Rc<Self>) -> Node {
        Node::SwitchMerge(this)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ConditionNode {
    pub(crate) label: IdRef,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) true_node: Option<Node>,
    pub(crate) false_node: Option<Node>,
    pub(crate) next: Node,
}

impl GenericNode for ConditionNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(this: Rc<Self>) -> Node {
        Node::Condition(this)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ConditionMergeNode {
    pub(crate) label: IdRef,
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) condition_node: RefCell<Weak<ConditionNode>>,
}

impl GenericNode for ConditionMergeNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(this: Rc<Self>) -> Node {
        Node::ConditionMerge(this)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ReturnNode {
    pub(crate) label: IdRef,
    pub(crate) instructions: Vec<Instruction>,
}

impl GenericNode for ReturnNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(this: Rc<Self>) -> Node {
        Node::Return(this)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) struct DiscardNode {
    pub(crate) label: IdRef,
    pub(crate) instructions: Vec<Instruction>,
}

impl GenericNode for DiscardNode {
    fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }
    fn to_node(this: Rc<Self>) -> Node {
        Node::Discard(this)
    }
    fn label(&self) -> IdRef {
        self.label
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Node {
    Simple(Rc<SimpleNode>),
    Return(Rc<ReturnNode>),
    Discard(Rc<DiscardNode>),
    Switch(Rc<SwitchNode>),
    SwitchFallthrough(Rc<SwitchFallthroughNode>),
    SwitchMerge(Rc<SwitchMergeNode>),
    Condition(Rc<ConditionNode>),
    ConditionMerge(Rc<ConditionMergeNode>),
}

impl<T: GenericNode> From<Rc<T>> for Node {
    fn from(v: Rc<T>) -> Node {
        GenericNode::to_node(v)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum SwitchCaseKind {
    Default,
    Normal,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum BlockKind {
    Unknown,
    ConditionMerge,
    LoopMerge,
    LoopContinue,
    SwitchCase(SwitchCaseKind),
    SwitchMerge,
}

struct BasicBlock<'a> {
    label_id: IdRef,
    label_line_instructions: &'a [Instruction],
    instructions: &'a [Instruction],
    kind: RefCell<BlockKind>,
}

impl<'a> BasicBlock<'a> {
    fn get_instructions(&self) -> Vec<Instruction> {
        let mut retval: Vec<Instruction> =
            Vec::with_capacity(self.label_line_instructions.len() + 1 + self.instructions.len());
        retval.extend(self.label_line_instructions.iter().map(Clone::clone));
        retval.push(Instruction::Label {
            id_result: IdResult(self.label_id),
        });
        retval.extend(self.instructions.iter().map(Clone::clone));
        retval
    }
    fn set_kind(&self, kind: BlockKind) {
        match self.kind.replace(kind) {
            BlockKind::Unknown => {}
            kind => unreachable!("block kind already set to {:?}", kind),
        }
    }
}

impl<'a> fmt::Debug for BasicBlock<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BasicBlock:\n")?;
        for instruction in self.get_instructions() {
            write!(f, "{}", instruction)?;
        }
        Ok(())
    }
}

struct ParseStateCondition {
    merges: Vec<Rc<ConditionMergeNode>>,
    merge_label: IdRef,
}

struct ParseStateSwitch {
    fallthrough_to_default: Option<Rc<SwitchFallthroughNode>>,
    fallthroughs: Vec<Rc<SwitchFallthroughNode>>,
    default_label: IdRef,
    next_case: Option<IdRef>,
    merges: Vec<Rc<SwitchMergeNode>>,
    merge_label: IdRef,
}

struct ParseState {
    condition: Option<ParseStateCondition>,
    switch: Option<ParseStateSwitch>,
}

fn get_basic_block<'a, 'b>(
    basic_blocks: &'b HashMap<IdRef, BasicBlock<'a>>,
    label_id: IdRef,
) -> &'b BasicBlock<'a> {
    basic_blocks
        .get(&label_id)
        .unwrap_or_else(|| unreachable!("label not found: {}", label_id))
}

impl ParseState {
    fn push_condition(&mut self, condition: ParseStateCondition) -> Option<ParseStateCondition> {
        mem::replace(&mut self.condition, Some(condition))
    }
    fn pop_condition(&mut self, old_condition: Option<ParseStateCondition>) -> ParseStateCondition {
        mem::replace(&mut self.condition, old_condition).unwrap()
    }
    fn push_switch(&mut self, switch: ParseStateSwitch) -> Option<ParseStateSwitch> {
        mem::replace(&mut self.switch, Some(switch))
    }
    fn pop_switch(&mut self, old_switch: Option<ParseStateSwitch>) -> ParseStateSwitch {
        mem::replace(&mut self.switch, old_switch).unwrap()
    }
    fn get_switch(&mut self) -> &mut ParseStateSwitch {
        self.switch.as_mut().unwrap()
    }
    fn parse(&mut self, basic_blocks: &HashMap<IdRef, BasicBlock>, label_id: IdRef) -> Node {
        let basic_block = get_basic_block(basic_blocks, label_id);
        let (terminating_instruction, instructions_without_terminator) = basic_block
            .instructions
            .split_last()
            .expect("missing terminating instruction");
        let control_header_instruction = instructions_without_terminator.last();
        match (terminating_instruction, control_header_instruction) {
            (
                &Instruction::Branch { target_label },
                Some(&Instruction::LoopMerge {
                    merge_block,
                    continue_target,
                    ..
                }),
            ) => unimplemented!(),
            (&Instruction::Branch { target_label }, _) => {
                let kind = *get_basic_block(basic_blocks, target_label).kind.borrow();
                match kind {
                    BlockKind::Unknown => {
                        let next = self.parse(basic_blocks, target_label);
                        Rc::new(SimpleNode {
                            label: label_id,
                            instructions: basic_block.get_instructions(),
                            next,
                        })
                        .into()
                    }
                    BlockKind::ConditionMerge => {
                        let mut condition = self
                            .condition
                            .as_mut()
                            .expect("invalid branch to merge block");
                        assert_eq!(
                            target_label, condition.merge_label,
                            "invalid branch to merge block"
                        );
                        let retval = Rc::new(ConditionMergeNode {
                            label: label_id,
                            instructions: basic_block.get_instructions(),
                            condition_node: Default::default(),
                        });
                        condition.merges.push(retval.clone());
                        retval.into()
                    }
                    BlockKind::LoopMerge => unimplemented!(),
                    BlockKind::LoopContinue => unimplemented!(),
                    BlockKind::SwitchCase(kind) => {
                        let mut switch = self.get_switch();
                        let expected_target_label = match kind {
                            SwitchCaseKind::Normal => {
                                switch.next_case.unwrap_or(switch.default_label)
                            }
                            SwitchCaseKind::Default => switch.default_label,
                        };
                        assert_eq!(
                            target_label, expected_target_label,
                            "invalid branch to next switch case"
                        );
                        unimplemented!()
                    }
                    BlockKind::SwitchMerge => {
                        assert_eq!(
                            target_label,
                            self.get_switch().merge_label,
                            "invalid branch to merge block"
                        );
                        let retval = Rc::new(SwitchMergeNode {
                            label: label_id,
                            instructions: basic_block.get_instructions(),
                            switch: Default::default(),
                        });
                        self.get_switch().merges.push(retval.clone());
                        retval.into()
                    }
                }
            }
            (
                &Instruction::BranchConditional {
                    true_label,
                    false_label,
                    ..
                },
                Some(&Instruction::LoopMerge {
                    merge_block,
                    continue_target,
                    ..
                }),
            ) => unimplemented!(),
            (
                &Instruction::BranchConditional {
                    true_label,
                    false_label,
                    ..
                },
                Some(&Instruction::SelectionMerge { merge_block, .. }),
            ) => {
                get_basic_block(basic_blocks, merge_block).set_kind(BlockKind::ConditionMerge);
                let old_condition = self.push_condition(ParseStateCondition {
                    merge_label: merge_block,
                    merges: Vec::new(),
                });
                let true_node = if true_label != merge_block {
                    Some(self.parse(basic_blocks, true_label))
                } else {
                    None
                };
                let false_node = if false_label != merge_block {
                    Some(self.parse(basic_blocks, false_label))
                } else {
                    None
                };
                let condition = self.pop_condition(old_condition);
                let next = self.parse(basic_blocks, merge_block);
                let retval = Rc::new(ConditionNode {
                    label: label_id,
                    instructions: basic_block.get_instructions(),
                    true_node,
                    false_node,
                    next,
                });
                for merge in condition.merges {
                    merge.condition_node.replace(Rc::downgrade(&retval));
                }
                retval.into()
            }
            (&Instruction::BranchConditional { .. }, _) => {
                unreachable!("missing merge instruction")
            }
            (
                &Instruction::Switch32 {
                    default,
                    target: ref targets,
                    ..
                },
                Some(&Instruction::SelectionMerge { merge_block, .. }),
            ) => {
                unimplemented!();
            }
            (
                &Instruction::Switch64 {
                    default: default_label,
                    target: ref targets,
                    ..
                },
                Some(&Instruction::SelectionMerge { merge_block, .. }),
            ) => {
                get_basic_block(basic_blocks, merge_block).set_kind(BlockKind::SwitchMerge);
                for &(_, target) in targets {
                    if target != merge_block {
                        get_basic_block(basic_blocks, target)
                            .set_kind(BlockKind::SwitchCase(SwitchCaseKind::Normal));
                    }
                }
                if default_label != merge_block {
                    get_basic_block(basic_blocks, default_label)
                        .set_kind(BlockKind::SwitchCase(SwitchCaseKind::Default));
                }
                let old_switch = self.push_switch(ParseStateSwitch {
                    default_label: default_label,
                    fallthrough_to_default: None,
                    merge_label: merge_block,
                    fallthroughs: vec![],
                    merges: vec![],
                    next_case: None,
                });
                let default = if default_label != merge_block {
                    Some(self.parse(basic_blocks, default_label))
                } else {
                    None
                };
                let mut default_fallthrough = None;
                for i in self.get_switch().fallthroughs.drain(..) {
                    assert!(
                        default_fallthrough.is_none(),
                        "multiple fallthroughs from default case"
                    );
                    default_fallthrough = Some(i);
                }
                let mut cases = Vec::with_capacity(targets.len());
                for (index, &(_, target)) in targets.iter().enumerate() {
                    self.get_switch().next_case = targets.get(index + 1).map(|v| v.1);
                    cases.push(self.parse(basic_blocks, target));
                }
                let switch = self.pop_switch(old_switch);
                let (before_default_cases, default) = if let Some(default) = default {
                    if let Some(fallthrough_to_default) = &switch.fallthrough_to_default {
                        // FIXME: handle default_fallthrough
                        unimplemented!()
                    } else if let Some(default_fallthrough) = &default_fallthrough {
                        unimplemented!()
                    } else {
                        (
                            cases,
                            Some(SwitchDefault {
                                default_case: default,
                                after_default_cases: vec![],
                            }),
                        )
                    }
                } else {
                    (cases, None)
                };
                let next = self.parse(basic_blocks, merge_block);
                let retval = Rc::new(SwitchNode {
                    label: label_id,
                    instructions: basic_block.get_instructions(),
                    before_default_cases,
                    default,
                    next,
                });
                if let Some(default_fallthrough) = default_fallthrough {
                    default_fallthrough.switch.replace(Rc::downgrade(&retval));
                }
                if let Some(fallthrough_to_default) = switch.fallthrough_to_default {
                    fallthrough_to_default
                        .switch
                        .replace(Rc::downgrade(&retval));
                }
                for fallthrough in switch.fallthroughs {
                    fallthrough.switch.replace(Rc::downgrade(&retval));
                }
                for merge in switch.merges {
                    merge.switch.replace(Rc::downgrade(&retval));
                }
                retval.into()
            }
            (&Instruction::Switch32 { .. }, _) => unreachable!("missing merge instruction"),
            (&Instruction::Switch64 { .. }, _) => unreachable!("missing merge instruction"),
            (&Instruction::Kill {}, _) => Rc::new(DiscardNode {
                label: label_id,
                instructions: basic_block.get_instructions(),
            })
            .into(),
            (&Instruction::Return {}, _) => Rc::new(ReturnNode {
                label: label_id,
                instructions: basic_block.get_instructions(),
            })
            .into(),
            (&Instruction::ReturnValue { .. }, _) => Rc::new(ReturnNode {
                label: label_id,
                instructions: basic_block.get_instructions(),
            })
            .into(),
            (&Instruction::Unreachable {}, _) => unimplemented!(),
            _ => unreachable!(
                "invalid basic block terminating instruction:\n{}",
                terminating_instruction
            ),
        }
    }
}

pub(crate) fn create_cfg(mut input_instructions: &[Instruction]) -> Node {
    let mut basic_blocks = HashMap::new();
    let mut first_block = None;
    'split_into_blocks: while !input_instructions.is_empty() {
        let (label_id, label_line_instructions) = 'find_label: loop {
            for (i, instruction) in input_instructions.iter().enumerate() {
                match instruction {
                    Instruction::Label { id_result } => {
                        break 'find_label (id_result.0, &input_instructions[..i]);
                    }
                    Instruction::NoLine {} | Instruction::Line { .. } => {}
                    _ => break,
                }
            }
            unreachable!("missing OpLabel")
        };
        if first_block.is_none() {
            first_block = Some(label_id);
        }
        for i in 0..input_instructions.len() {
            match &input_instructions[i] {
                Instruction::Branch { .. }
                | Instruction::BranchConditional { .. }
                | Instruction::Switch32 { .. }
                | Instruction::Switch64 { .. }
                | Instruction::Kill { .. }
                | Instruction::Return { .. }
                | Instruction::ReturnValue { .. }
                | Instruction::Unreachable { .. } => {
                    let (instructions, rest) = input_instructions.split_at(i + 1);
                    input_instructions = rest;
                    let previous = basic_blocks.insert(
                        label_id,
                        BasicBlock {
                            label_line_instructions,
                            label_id,
                            instructions,
                            kind: RefCell::new(BlockKind::Unknown),
                        },
                    );
                    assert!(previous.is_none(), "duplicate OpLabel: {}", label_id);
                    continue 'split_into_blocks;
                }
                _ => {}
            }
        }
        unreachable!("missing terminating instruction");
    }
    let first_block = first_block.expect("missing OpLabel");
    ParseState {
        condition: None,
        switch: None,
    }
    .parse(&basic_blocks, first_block)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IdFactory(u32);

    impl IdFactory {
        fn new() -> IdFactory {
            IdFactory(1)
        }
        fn next(&mut self) -> IdRef {
            let retval = IdRef(self.0);
            self.0 += 1;
            retval
        }
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    enum SerializedCFGElement {
        Simple,
        Return,
        Discard,
        Switch,
        SwitchCase,
        SwitchDefaultCase,
        SwitchEnd,
        SwitchFallthrough,
        SwitchMerge,
        Condition,
        ConditionTrue,
        ConditionFalse,
        ConditionEnd,
        ConditionMerge,
    }

    trait SerializeCFG {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>);
        fn serialize_cfg_into_vec(&self) -> Vec<SerializedCFGElement> {
            let mut retval = Vec::new();
            self.serialize_cfg(&mut retval);
            retval
        }
    }

    impl<T: SerializeCFG> SerializeCFG for Rc<T> {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            (**self).serialize_cfg(output)
        }
    }

    impl<'a, T: SerializeCFG> SerializeCFG for &'a T {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            (**self).serialize_cfg(output)
        }
    }

    impl SerializeCFG for SimpleNode {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            output.push(SerializedCFGElement::Simple);
            self.next.serialize_cfg(output)
        }
    }

    impl SerializeCFG for ReturnNode {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            output.push(SerializedCFGElement::Return);
        }
    }

    impl SerializeCFG for DiscardNode {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            output.push(SerializedCFGElement::Discard);
        }
    }

    impl SerializeCFG for SwitchNode {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            output.push(SerializedCFGElement::Switch);
            for case in &self.before_default_cases {
                output.push(SerializedCFGElement::SwitchCase);
                case.serialize_cfg(output);
            }
            if let Some(default) = &self.default {
                output.push(SerializedCFGElement::SwitchDefaultCase);
                default.default_case.serialize_cfg(output);
                for case in &default.after_default_cases {
                    output.push(SerializedCFGElement::SwitchCase);
                    case.serialize_cfg(output);
                }
            }
            output.push(SerializedCFGElement::SwitchEnd);
            self.next.serialize_cfg(output);
        }
    }

    impl SerializeCFG for SwitchFallthroughNode {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            output.push(SerializedCFGElement::SwitchFallthrough);
        }
    }

    impl SerializeCFG for SwitchMergeNode {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            output.push(SerializedCFGElement::SwitchMerge);
        }
    }

    impl SerializeCFG for ConditionNode {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            output.push(SerializedCFGElement::Condition);
            if let Some(true_node) = &self.true_node {
                output.push(SerializedCFGElement::ConditionTrue);
                true_node.serialize_cfg(output);
            }
            if let Some(false_node) = &self.false_node {
                output.push(SerializedCFGElement::ConditionFalse);
                false_node.serialize_cfg(output);
            }
            output.push(SerializedCFGElement::ConditionEnd);
            self.next.serialize_cfg(output)
        }
    }

    impl SerializeCFG for ConditionMergeNode {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            output.push(SerializedCFGElement::ConditionMerge);
        }
    }

    impl SerializeCFG for Node {
        fn serialize_cfg(&self, output: &mut Vec<SerializedCFGElement>) {
            match self {
                Node::Simple(v) => v.serialize_cfg(output),
                Node::Return(v) => v.serialize_cfg(output),
                Node::Discard(v) => v.serialize_cfg(output),
                Node::Switch(v) => v.serialize_cfg(output),
                Node::SwitchFallthrough(v) => v.serialize_cfg(output),
                Node::SwitchMerge(v) => v.serialize_cfg(output),
                Node::Condition(v) => v.serialize_cfg(output),
                Node::ConditionMerge(v) => v.serialize_cfg(output),
            }
        }
    }

    fn test_cfg(instructions: &[Instruction], expected: &[SerializedCFGElement]) {
        println!("instructions:");
        for instruction in instructions {
            print!("{}", instruction);
        }
        println!();
        let cfg = create_cfg(&instructions);
        assert_eq!(&*cfg.serialize_cfg_into_vec(), expected);
    }

    #[test]
    fn test_cfg_return() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label1 = id_factory.next();
        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label1),
        });
        instructions.push(Instruction::Return);

        test_cfg(&instructions, &[SerializedCFGElement::Return]);
    }

    #[test]
    fn test_cfg_return_value() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label1 = id_factory.next();
        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label1),
        });
        instructions.push(Instruction::ReturnValue {
            value: id_factory.next(),
        });

        test_cfg(&instructions, &[SerializedCFGElement::Return]);
    }

    #[test]
    fn test_cfg_simple_discard() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label1 = id_factory.next();
        let label2 = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label1),
        });
        instructions.push(Instruction::Branch {
            target_label: label2,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label2),
        });
        instructions.push(Instruction::Kill);

        test_cfg(
            &instructions,
            &[SerializedCFGElement::Simple, SerializedCFGElement::Discard],
        );
    }

    #[test]
    fn test_cfg_conditional_none_none() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_endif = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_endif,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::BranchConditional {
            condition: id_factory.next(),
            true_label: label_endif,
            false_label: label_endif,
            branch_weights: vec![],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_endif),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Condition,
                SerializedCFGElement::ConditionEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_conditional_merge_none() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_then = id_factory.next();
        let label_endif = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_endif,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::BranchConditional {
            condition: id_factory.next(),
            true_label: label_then,
            false_label: label_endif,
            branch_weights: vec![],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_then),
        });
        instructions.push(Instruction::Branch {
            target_label: label_endif,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_endif),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Condition,
                SerializedCFGElement::ConditionTrue,
                SerializedCFGElement::ConditionMerge,
                SerializedCFGElement::ConditionEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_conditional_return_merge() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_then = id_factory.next();
        let label_else = id_factory.next();
        let label_endif = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_endif,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::BranchConditional {
            condition: id_factory.next(),
            true_label: label_then,
            false_label: label_else,
            branch_weights: vec![],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_then),
        });
        instructions.push(Instruction::Return);

        instructions.push(Instruction::Label {
            id_result: IdResult(label_else),
        });
        instructions.push(Instruction::Branch {
            target_label: label_endif,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_endif),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Condition,
                SerializedCFGElement::ConditionTrue,
                SerializedCFGElement::Return,
                SerializedCFGElement::ConditionFalse,
                SerializedCFGElement::ConditionMerge,
                SerializedCFGElement::ConditionEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_switch_default_break() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_default = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_merge,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::Switch64 {
            selector: id_factory.next(),
            default: label_default,
            target: vec![],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_default),
        });
        instructions.push(Instruction::Branch {
            target_label: label_merge,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Switch,
                SerializedCFGElement::SwitchDefaultCase,
                SerializedCFGElement::SwitchMerge,
                SerializedCFGElement::SwitchEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_switch_return_default_break() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_case1 = id_factory.next();
        let label_default = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_merge,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::Switch64 {
            selector: id_factory.next(),
            default: label_default,
            target: vec![(0, label_case1)],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_case1),
        });
        instructions.push(Instruction::Return);

        instructions.push(Instruction::Label {
            id_result: IdResult(label_default),
        });
        instructions.push(Instruction::Branch {
            target_label: label_merge,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Switch,
                SerializedCFGElement::SwitchCase,
                SerializedCFGElement::Return,
                SerializedCFGElement::SwitchDefaultCase,
                SerializedCFGElement::SwitchMerge,
                SerializedCFGElement::SwitchEnd,
                SerializedCFGElement::Return,
            ],
        );
    }

    #[test]
    fn test_cfg_switch_fallthrough_default_break() {
        let mut id_factory = IdFactory::new();
        let mut instructions = Vec::new();

        let label_start = id_factory.next();
        let label_case1 = id_factory.next();
        let label_default = id_factory.next();
        let label_merge = id_factory.next();

        instructions.push(Instruction::NoLine);
        instructions.push(Instruction::Label {
            id_result: IdResult(label_start),
        });
        instructions.push(Instruction::SelectionMerge {
            merge_block: label_merge,
            selection_control: spirv_parser::SelectionControl::default(),
        });
        instructions.push(Instruction::Switch64 {
            selector: id_factory.next(),
            default: label_default,
            target: vec![(0, label_case1)],
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_case1),
        });
        instructions.push(Instruction::Branch {
            target_label: label_default,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_default),
        });
        instructions.push(Instruction::Branch {
            target_label: label_merge,
        });

        instructions.push(Instruction::Label {
            id_result: IdResult(label_merge),
        });
        instructions.push(Instruction::Return);

        test_cfg(
            &instructions,
            &[
                SerializedCFGElement::Switch,
                SerializedCFGElement::SwitchCase,
                SerializedCFGElement::Return,
                SerializedCFGElement::SwitchDefaultCase,
                SerializedCFGElement::SwitchMerge,
                SerializedCFGElement::SwitchEnd,
                SerializedCFGElement::Return,
            ],
        );
    }
}
