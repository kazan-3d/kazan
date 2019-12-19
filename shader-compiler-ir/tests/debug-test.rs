// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information

use shader_compiler_ir::{
    Block, BreakBlock, Const, ConstData, ConstInteger, GlobalState, Inhabited, Instruction,
    InstructionData, IntegerType, InternedString, Location, LocationData, Loop, LoopHeader, Type,
    TypeData, Value, ValueDefinition, ValueUse,
};
use std::rc::Rc;
use std::rc::Weak;

#[test]
fn test_debug() {
    let global_state = GlobalState::default();
    let int32_type = Type::get(
        &TypeData::Integer {
            integer_type: IntegerType::Int32,
        },
        &global_state,
    );
    let code = Rc::new(Block {
        body: vec![
            Instruction {
                location: Location::new(
                    &LocationData {
                        file: InternedString::new("file1.vertex", &global_state),
                        line: 1,
                        column: 1,
                    },
                    &global_state,
                ),
                data: InstructionData::Loop({
                    let loop_arg0 = ValueDefinition::new(
                        InternedString::new("loop_arg0", &global_state),
                        int32_type.clone(),
                    );
                    Rc::new(Loop {
                        arguments: vec![ValueUse::new(Value::from_const(
                            Const::get(
                                &ConstData::Integer(ConstInteger {
                                    integer_type: IntegerType::Int32,
                                    value: 0,
                                }),
                                &global_state,
                            ),
                            InternedString::empty(),
                        ))],
                        header: LoopHeader {
                            argument_definitions: vec![loop_arg0],
                        },
                        body: Rc::new(Block {
                            body: vec![Instruction {
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
                            }],
                            result_definitions: Inhabited(Vec::new()),
                        }),
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
