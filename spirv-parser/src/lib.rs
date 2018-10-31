// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

#![cfg_attr(feature = "cargo-clippy", allow(clippy::unreadable_literal))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::cyclomatic_complexity))]

include!(concat!(env!("OUT_DIR"), "/generated_parser.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::mem;
    use std::slice;

    fn parse_and_dump(bytes: &[u8]) -> Result<String> {
        assert_eq!(bytes.len() % mem::size_of::<u32>(), 0);
        let mut words: Vec<u32> = Vec::new();
        words.resize(bytes.len() / mem::size_of::<u32>(), 0);
        unsafe {
            slice::from_raw_parts_mut(
                words.as_mut_ptr() as *mut u8,
                words.len() * mem::size_of::<u32>(),
            )
            .copy_from_slice(bytes);
        }
        assert!(!words.is_empty());
        if words[0].swap_bytes() == MAGIC_NUMBER {
            for word in words.iter_mut() {
                *word = word.swap_bytes();
            }
        }
        let parser = Parser::start(&words)?;
        let mut out = Vec::<u8>::new();
        println!("Dumped output:");
        print!("{}", parser.header());
        write!(&mut out, "{}", parser.header()).unwrap();
        for instruction in parser {
            let instruction = instruction?;
            print!("{}", instruction);
            write!(&mut out, "{}", instruction).unwrap();
        }
        println!();
        Ok(String::from_utf8(out).unwrap())
    }

    #[test]
    fn parse_test() {
        let output = parse_and_dump(include_bytes!("../test_inputs/test.spv")).unwrap();
        let expected = r#"; SPIR-V
; Version: 1.0
; Generator: 0x80001
; Bound: 44
; Schema: 0
               OpCapability Shader
               OpCapability Int64
          %1 = OpExtInstImport "GLSL.std.450"
               OpMemoryModel Logical GLSL450
               OpEntryPoint Vertex %4 "main" %10 %15
               OpMemberDecorate %8 0 BuiltIn Position
               OpDecorate %8 Block
               OpDecorate %15 Location 0
          %2 = OpTypeVoid
          %3 = OpTypeFunction %2
          %6 = OpTypeFloat 32
          %7 = OpTypeVector %6 4
          %8 = OpTypeStruct %7
          %9 = OpTypePointer Output %8
         %10 = OpVariable %9 Output
         %11 = OpTypeInt 32 1
         %12 = OpConstant %11 0x00000000
         %13 = OpTypeVector %6 3
         %14 = OpTypePointer Input %13
         %15 = OpVariable %14 Input
         %17 = OpConstant %6 0x3F800000
         %22 = OpTypePointer Output %7
         %24 = OpTypePointer Function %11
         %26 = OpTypeInt 32 0
         %27 = OpConstant %26 0x00000002
         %28 = OpTypePointer Input %6
         %31 = OpTypeInt 64 0
         %40 = OpConstant %6 0x00000000
         %41 = OpConstant %26 0x00000000
         %42 = OpTypePointer Output %6
          %4 = OpFunction %2 None %3
          %5 = OpLabel
         %25 = OpVariable %24 Function
         %16 = OpLoad %13 %15
         %18 = OpCompositeExtract %6 %16 0
         %19 = OpCompositeExtract %6 %16 1
         %20 = OpCompositeExtract %6 %16 2
         %21 = OpCompositeConstruct %7 %18 %19 %20 %17
         %23 = OpAccessChain %22 %10 %12
               OpStore %23 %21
         %29 = OpAccessChain %28 %15 %27
         %30 = OpLoad %6 %29
         %32 = OpConvertFToU %31 %30
         %33 = OpUConvert %26 %32
         %34 = OpBitcast %11 %33
               OpStore %25 %34
         %35 = OpLoad %11 %25
               OpSelectionMerge %38 None
               OpSwitch %32 %37 1 %36 2 %36 8 %36
         %37 = OpLabel
         %43 = OpAccessChain %42 %10 %12 %41
               OpStore %43 %40
               OpBranch %38
         %36 = OpLabel
               OpBranch %38
         %38 = OpLabel
               OpReturn
               OpFunctionEnd
"#;
        println!("Line-by-line:");
        for (a, b) in output.lines().zip(expected.lines()) {
            println!("{}\n{}", a, b);
        }
        assert!(output == expected);
    }
}
