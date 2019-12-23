// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use shader_compiler_ir::{
    Allocate, Block, BreakBlock, Const, ConstInteger, GlobalState, Inhabited, Instruction,
    InstructionData, IntegerType, Intern, Interned, Location, Loop, LoopHeader, OnceCell, Type,
    Value, ValueDefinition, ValueUse,
};

#[test]
fn test_debug() {
    let global_state = GlobalState::default();
    let global_state = &global_state;
    let int32_type = global_state.intern(&Type::Integer {
        integer_type: IntegerType::Int32,
    });
    let block0 = global_state.alloc(Block {
        body: OnceCell::new(),
        result_definitions: Inhabited(Vec::new()),
    });
    block0
        .body
        .set(vec![Instruction {
            location: Some(global_state.intern(&Location {
                file: global_state.intern("file1.vertex"),
                line: 2,
                column: 1,
            })),
            data: InstructionData::BreakBlock(BreakBlock {
                block: block0,
                block_results: vec![],
            }),
        }])
        .unwrap();
    let code = global_state.alloc(Block {
        body: vec![
            Instruction {
                location: Some(global_state.intern(&Location {
                    file: global_state.intern("file1.vertex"),
                    line: 1,
                    column: 1,
                })),
                data: InstructionData::Loop({
                    let loop_arg0 = ValueDefinition::new(
                        global_state.intern("loop_arg0"),
                        int32_type.clone(),
                        global_state,
                    );
                    global_state.alloc(Loop {
                        arguments: vec![ValueUse::new(Value::from_const(
                            global_state.intern(&Const::Integer(ConstInteger {
                                integer_type: IntegerType::Int32,
                                value: 0,
                            })),
                            global_state.intern(""),
                            global_state,
                        ))],
                        header: LoopHeader {
                            argument_definitions: vec![loop_arg0],
                        },
                        body: block0,
                    })
                }),
            },
            Instruction {
                location: Location::new(
                    &LocationData {
                        file: InternedString::new("file1.vertex", &global_state),
                        line: 2,
                        column: 1,
                    },
                    &global_state,
                ),
                data: InstructionData::BreakBlock(BreakBlock {
                    block: Weak::new(),
                    block_results: vec![],
                }),
            },
        ],
        result_definitions: Inhabited(Vec::new()),
    });
    // FIXME: finish implementing
    let expected_code = "";
    let code = format!("{:#?}", code);
    println!("{}", code);
    assert!(code == expected_code);
}
