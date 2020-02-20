// SPDX-License-Identifier: LGPL-2.1-or-later
// See Notices.txt for copyright information
#![cfg_attr(not(test), no_std)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::redundant_closure_call)]

#[macro_use]
extern crate alloc;

mod generated_parser;

pub use generated_parser::*;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem;
    use std::io::Write;

    /// produce dump output using:
    ///
    /// ```sh
    /// spirv-dis --offsets --raw-id <input-file.spv>
    /// ```
    ///
    /// manual editing will still be needed to convert constants to
    /// their raw hex bytes and use the raw hex value for
    /// the generator header field.
    fn parse_and_dump(bytes: &[u8]) -> Result<String> {
        let words = convert_bytes_to_words(bytes)?;
        let mut parser = Parser::start(&words)?;
        let mut out = Vec::<u8>::new();
        println!("Dumped output:");
        println!("{}", parser.header());
        writeln!(&mut out, "{}", parser.header()).unwrap();
        loop {
            let next_byte_location = parser.next_location() * mem::size_of::<u32>();
            if let Some(instruction) = parser.next() {
                let instruction = instruction?;
                println!("{} ; 0x{:08x}", instruction, next_byte_location);
                writeln!(&mut out, "{} ; 0x{:08x}", instruction, next_byte_location).unwrap();
            } else {
                break;
            }
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
               OpCapability Shader ; 0x00000014
               OpCapability Int64 ; 0x0000001c
          %1 = OpExtInstImport "GLSL.std.450" ; 0x00000024
               OpMemoryModel Logical GLSL450 ; 0x0000003c
               OpEntryPoint Vertex %4 "main" %10 %15 ; 0x00000048
               OpMemberDecorate %8 0 BuiltIn Position ; 0x00000064
               OpDecorate %8 Block ; 0x00000078
               OpDecorate %15 Location 0 ; 0x00000084
          %2 = OpTypeVoid ; 0x00000094
          %3 = OpTypeFunction %2 ; 0x0000009c
          %6 = OpTypeFloat 32 ; 0x000000a8
          %7 = OpTypeVector %6 4 ; 0x000000b4
          %8 = OpTypeStruct %7 ; 0x000000c4
          %9 = OpTypePointer Output %8 ; 0x000000d0
         %10 = OpVariable %9 Output ; 0x000000e0
         %11 = OpTypeInt 32 1 ; 0x000000f0
         %12 = OpConstant %11 0x00000000 ; 0x00000100
         %13 = OpTypeVector %6 3 ; 0x00000110
         %14 = OpTypePointer Input %13 ; 0x00000120
         %15 = OpVariable %14 Input ; 0x00000130
         %17 = OpConstant %6 0x3F800000 ; 0x00000140
         %22 = OpTypePointer Output %7 ; 0x00000150
         %24 = OpTypePointer Function %11 ; 0x00000160
         %26 = OpTypeInt 32 0 ; 0x00000170
         %27 = OpConstant %26 0x00000002 ; 0x00000180
         %28 = OpTypePointer Input %6 ; 0x00000190
         %31 = OpTypeInt 64 0 ; 0x000001a0
         %40 = OpConstant %6 0x00000000 ; 0x000001b0
         %41 = OpConstant %26 0x00000000 ; 0x000001c0
         %42 = OpTypePointer Output %6 ; 0x000001d0
          %4 = OpFunction %2 None %3 ; 0x000001e0
          %5 = OpLabel ; 0x000001f4
         %25 = OpVariable %24 Function ; 0x000001fc
         %16 = OpLoad %13 %15 ; 0x0000020c
         %18 = OpCompositeExtract %6 %16 0 ; 0x0000021c
         %19 = OpCompositeExtract %6 %16 1 ; 0x00000230
         %20 = OpCompositeExtract %6 %16 2 ; 0x00000244
         %21 = OpCompositeConstruct %7 %18 %19 %20 %17 ; 0x00000258
         %23 = OpAccessChain %22 %10 %12 ; 0x00000274
               OpStore %23 %21 ; 0x00000288
         %29 = OpAccessChain %28 %15 %27 ; 0x00000294
         %30 = OpLoad %6 %29 ; 0x000002a8
         %32 = OpConvertFToU %31 %30 ; 0x000002b8
         %33 = OpUConvert %26 %32 ; 0x000002c8
         %34 = OpBitcast %11 %33 ; 0x000002d8
               OpStore %25 %34 ; 0x000002e8
         %35 = OpLoad %11 %25 ; 0x000002f4
               OpSelectionMerge %38 None ; 0x00000304
               OpSwitch %32 %37 1 %36 2 %36 8 %36 ; 0x00000310
         %37 = OpLabel ; 0x00000340
         %43 = OpAccessChain %42 %10 %12 %41 ; 0x00000348
               OpStore %43 %40 ; 0x00000360
               OpBranch %38 ; 0x0000036c
         %36 = OpLabel ; 0x00000374
               OpBranch %38 ; 0x0000037c
         %38 = OpLabel ; 0x00000384
               OpReturn ; 0x0000038c
               OpFunctionEnd ; 0x00000390
"#;
        println!("Line-by-line:");
        for (a, b) in output.lines().zip(expected.lines()) {
            println!("{}\n{}", a, b);
        }
        assert!(output == expected);
    }

    #[test]
    fn parse_test2() {
        let output = parse_and_dump(include_bytes!("../test_inputs/test2.spv")).unwrap();
        let expected = r#"; SPIR-V
; Version: 1.3
; Generator: 0x70000
; Bound: 12
; Schema: 0
               OpCapability Shader ; 0x00000014
               OpMemoryModel Logical GLSL450 ; 0x0000001c
               OpEntryPoint Vertex %1 "main" ; 0x00000028
          %2 = OpTypeVoid ; 0x0000003c
          %3 = OpTypeFloat 32 ; 0x00000044
          %4 = OpTypeVector %3 4 ; 0x00000050
          %5 = OpTypeFunction %2 ; 0x00000060
          %1 = OpFunction %2 None %5 ; 0x0000006c
          %6 = OpLabel ; 0x00000080
          %7 = OpImageSampleImplicitLod %4 %8 %9 Bias|MinLod %10 %11 ; 0x00000088
               OpReturn ; 0x000000a8
               OpFunctionEnd ; 0x000000ac
"#;
        println!("Line-by-line:");
        for (a, b) in output.lines().zip(expected.lines()) {
            println!("{}\n{}", a, b);
        }
        assert!(output == expected);
    }

    #[test]
    fn parse_test3() {
        let output = parse_and_dump(include_bytes!("../test_inputs/test3.spv")).unwrap();
        let expected = r#"; SPIR-V
; Version: 1.0
; Generator: 0x80007
; Bound: 38
; Schema: 0
               OpCapability Shader ; 0x00000014
          %1 = OpExtInstImport "GLSL.std.450" ; 0x0000001c
               OpMemoryModel Logical GLSL450 ; 0x00000034
               OpEntryPoint GLCompute %4 "main" ; 0x00000040
               OpExecutionMode %4 LocalSize 1 1 1 ; 0x00000054
               OpSource GLSL 450 ; 0x0000006c
               OpName %4 "main" ; 0x00000078
               OpName %8 "f(" ; 0x00000088
               OpName %10 "g(" ; 0x00000094
               OpName %14 "h(" ; 0x000000a0
               OpName %16 "A" ; 0x000000ac
               OpName %17 "B" ; 0x000000b8
               OpName %19 "C" ; 0x000000c4
               OpDecorate %16 SpecId 0 ; 0x000000d0
               OpDecorate %17 SpecId 1 ; 0x000000e0
               OpDecorate %19 SpecId 2 ; 0x000000f0
          %2 = OpTypeVoid ; 0x00000100
          %3 = OpTypeFunction %2 ; 0x00000108
          %6 = OpTypeInt 32 1 ; 0x00000114
          %7 = OpTypeFunction %6 ; 0x00000124
         %12 = OpTypeFloat 32 ; 0x00000130
         %13 = OpTypeFunction %12 ; 0x0000013c
         %16 = OpSpecConstant %6 0x00000000 ; 0x00000148
         %17 = OpSpecConstant %6 0x00000001 ; 0x00000158
         %18 = OpSpecConstantOp %6 IMul %16 %17 ; 0x00000168
         %19 = OpSpecConstant %6 0x00000002 ; 0x00000180
         %20 = OpSpecConstantOp %6 SDiv %18 %19 ; 0x00000190
         %23 = OpSpecConstantOp %6 BitwiseAnd %16 %17 ; 0x000001a8
         %24 = OpSpecConstantOp %6 BitwiseXor %23 %19 ; 0x000001c0
         %29 = OpConstant %12 0x3F490FDB ; 0x000001d8
         %30 = OpTypeVector %12 2 ; 0x000001e8
          %4 = OpFunction %2 None %3 ; 0x000001f8
          %5 = OpLabel ; 0x0000020c
         %35 = OpFunctionCall %6 %8 ; 0x00000214
         %36 = OpFunctionCall %6 %10 ; 0x00000224
         %37 = OpFunctionCall %12 %14 ; 0x00000234
               OpReturn ; 0x00000244
               OpFunctionEnd ; 0x00000248
          %8 = OpFunction %6 None %7 ; 0x0000024c
          %9 = OpLabel ; 0x00000260
               OpReturnValue %20 ; 0x00000268
               OpFunctionEnd ; 0x00000270
         %10 = OpFunction %6 None %7 ; 0x00000274
         %11 = OpLabel ; 0x00000288
               OpReturnValue %24 ; 0x00000290
               OpFunctionEnd ; 0x00000298
         %14 = OpFunction %12 None %13 ; 0x0000029c
         %15 = OpLabel ; 0x000002b0
         %27 = OpConvertSToF %12 %16 ; 0x000002b8
         %28 = OpExtInst %12 %1 Cos %27 ; 0x000002c8
         %31 = OpCompositeConstruct %30 %28 %29 ; 0x000002e0
         %32 = OpExtInst %12 %1 Length %31 ; 0x000002f4
               OpReturnValue %32 ; 0x0000030c
               OpFunctionEnd ; 0x00000314
"#;
        println!("Line-by-line:");
        for (a, b) in output.lines().zip(expected.lines()) {
            println!("{}\n{}", a, b);
        }
        assert!(output == expected);
    }
}
