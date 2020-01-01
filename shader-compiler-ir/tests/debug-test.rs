// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use shader_compiler_ir::prelude::*;
use shader_compiler_ir::{
    BinaryALUInstruction, BranchInstruction, BreakBlock, ContinueLoop, InstructionData,
    IntegerType, LoopHeader, OnceCell, SimpleInstruction,
};

#[test]
fn test_debug() {
    let global_state = GlobalState::default();
    let global_state = &global_state;
    let loop_counter_def = IntegerType::Int32.new_value_definition("loop_counter", global_state);
    let loop_counter = loop_counter_def.value();
    let loop_counter_next_def =
        IntegerType::Int32.new_value_definition("loop_counter_next", global_state);
    let loop_counter_next = loop_counter_next_def.value();
    let loop_start = 0u32;
    let loop_end = 10u32;
    let loop_increment = 1u32;
    let loop_body = global_state.alloc(Block {
        body: OnceCell::new(),
        result_definitions: Inhabited(Vec::new()),
    });
    let loop_ = global_state.alloc(Loop {
        arguments: vec![ValueUse::from_const(loop_start, "loop_start", global_state)],
        header: LoopHeader {
            argument_definitions: vec![loop_counter_def],
        },
        body: loop_body,
    });
    loop_body
        .body
        .set(vec![
            Instruction::with_location(
                Location::new_interned("file1.vertex", 2, 1, global_state),
                BranchInstruction {
                    variable: ValueUse::new(loop_counter),
                    targets: vec![(
                        loop_end.intern(global_state),
                        BreakBlock {
                            block: loop_body,
                            block_results: vec![],
                        },
                    )],
                },
            ),
            Instruction::with_location(
                Location::new_interned("file1.vertex", 3, 1, global_state),
                SimpleInstruction::Add(BinaryALUInstruction {
                    arguments: [
                        ValueUse::new(loop_counter),
                        ValueUse::from_const(loop_increment, "loop_increment", global_state),
                    ],
                    result: loop_counter_next_def,
                }),
            ),
            Instruction::with_location(
                Location::new_interned("file1.vertex", 4, 1, global_state),
                ContinueLoop {
                    target_loop: loop_,
                    block_arguments: vec![ValueUse::new(loop_counter_next)],
                },
            ),
        ])
        .unwrap();
    let entry_block = global_state.alloc(Block {
        body: OnceCell::new(),
        result_definitions: Inhabited(Vec::new()),
    });
    entry_block
        .body
        .set(vec![
            Instruction::with_location(
                Location::new_interned("file1.vertex", 1, 1, global_state),
                InstructionData::Loop(loop_),
            ),
            Instruction::with_location(
                Location::new_interned("file1.vertex", 2, 1, global_state),
                BreakBlock {
                    block: entry_block,
                    block_results: vec![],
                },
            ),
        ])
        .unwrap();
    let expected_code = r#"IdRef(
    #1,
    Block {
        body: OnceCell(
            [
                Instruction {
                    location: Some(
                        Location {
                            file: "file1.vertex",
                            line: 1,
                            column: 1,
                        },
                    ),
                    data: Loop(
                        IdRef(
                            #2,
                            Loop {
                                arguments: [
                                    ValueUse {
                                        value: IdRef(
                                            #3,
                                            Value {
                                                value_type: Integer {
                                                    integer_type: Int32,
                                                },
                                                name: "loop_start",
                                                const_value: Cell {
                                                    value: Some(
                                                        Integer(
                                                            ConstInteger {
                                                                value: 0,
                                                                integer_type: Int32,
                                                            },
                                                        ),
                                                    ),
                                                },
                                            },
                                        ),
                                    },
                                ],
                                header: LoopHeader {
                                    argument_definitions: [
                                        ValueDefinition {
                                            value: IdRef(
                                                #4,
                                                Value {
                                                    value_type: Integer {
                                                        integer_type: Int32,
                                                    },
                                                    name: "loop_counter",
                                                    const_value: Cell {
                                                        value: None,
                                                    },
                                                },
                                            ),
                                        },
                                    ],
                                },
                                body: IdRef(
                                    #5,
                                    Block {
                                        body: OnceCell(
                                            [
                                                Instruction {
                                                    location: Some(
                                                        Location {
                                                            file: "file1.vertex",
                                                            line: 2,
                                                            column: 1,
                                                        },
                                                    ),
                                                    data: Branch(
                                                        BranchInstruction {
                                                            variable: ValueUse {
                                                                value: IdRef(
                                                                    #6,
                                                                    Value {
                                                                        value_type: Integer {
                                                                            integer_type: Int32,
                                                                        },
                                                                        name: "loop_counter",
                                                                        const_value: Cell {
                                                                            value: None,
                                                                        },
                                                                    },
                                                                ),
                                                            },
                                                            targets: [
                                                                (
                                                                    Integer(
                                                                        ConstInteger {
                                                                            value: 10,
                                                                            integer_type: Int32,
                                                                        },
                                                                    ),
                                                                    BreakBlock {
                                                                        block: IdRef(
                                                                            #5,
                                                                            <omitted>,
                                                                        ),
                                                                        block_results: [],
                                                                    },
                                                                ),
                                                            ],
                                                        },
                                                    ),
                                                },
                                                Instruction {
                                                    location: Some(
                                                        Location {
                                                            file: "file1.vertex",
                                                            line: 3,
                                                            column: 1,
                                                        },
                                                    ),
                                                    data: Simple(
                                                        Add(
                                                            BinaryALUInstruction {
                                                                arguments: [
                                                                    ValueUse {
                                                                        value: IdRef(
                                                                            #7,
                                                                            Value {
                                                                                value_type: Integer {
                                                                                    integer_type: Int32,
                                                                                },
                                                                                name: "loop_counter",
                                                                                const_value: Cell {
                                                                                    value: None,
                                                                                },
                                                                            },
                                                                        ),
                                                                    },
                                                                    ValueUse {
                                                                        value: IdRef(
                                                                            #8,
                                                                            Value {
                                                                                value_type: Integer {
                                                                                    integer_type: Int32,
                                                                                },
                                                                                name: "loop_increment",
                                                                                const_value: Cell {
                                                                                    value: Some(
                                                                                        Integer(
                                                                                            ConstInteger {
                                                                                                value: 1,
                                                                                                integer_type: Int32,
                                                                                            },
                                                                                        ),
                                                                                    ),
                                                                                },
                                                                            },
                                                                        ),
                                                                    },
                                                                ],
                                                                result: ValueDefinition {
                                                                    value: IdRef(
                                                                        #9,
                                                                        Value {
                                                                            value_type: Integer {
                                                                                integer_type: Int32,
                                                                            },
                                                                            name: "loop_counter_next",
                                                                            const_value: Cell {
                                                                                value: None,
                                                                            },
                                                                        },
                                                                    ),
                                                                },
                                                            },
                                                        ),
                                                    ),
                                                },
                                                Instruction {
                                                    location: Some(
                                                        Location {
                                                            file: "file1.vertex",
                                                            line: 4,
                                                            column: 1,
                                                        },
                                                    ),
                                                    data: ContinueLoop(
                                                        ContinueLoop {
                                                            target_loop: IdRef(
                                                                #2,
                                                                <omitted>,
                                                            ),
                                                            block_arguments: [
                                                                ValueUse {
                                                                    value: IdRef(
                                                                        #10,
                                                                        Value {
                                                                            value_type: Integer {
                                                                                integer_type: Int32,
                                                                            },
                                                                            name: "loop_counter_next",
                                                                            const_value: Cell {
                                                                                value: None,
                                                                            },
                                                                        },
                                                                    ),
                                                                },
                                                            ],
                                                        },
                                                    ),
                                                },
                                            ],
                                        ),
                                        result_definitions: Inhabited(
                                            [],
                                        ),
                                    },
                                ),
                            },
                        ),
                    ),
                },
                Instruction {
                    location: Some(
                        Location {
                            file: "file1.vertex",
                            line: 2,
                            column: 1,
                        },
                    ),
                    data: BreakBlock(
                        BreakBlock {
                            block: IdRef(
                                #1,
                                <omitted>,
                            ),
                            block_results: [],
                        },
                    ),
                },
            ],
        ),
        result_definitions: Inhabited(
            [],
        ),
    },
)"#;
    let code = format!("{:#?}", entry_block);
    println!("{}", code);
    assert!(code == expected_code);
}
