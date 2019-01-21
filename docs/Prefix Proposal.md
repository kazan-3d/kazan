# SimpleV Prefix Proposal

Note that if we have a transpose instruction, we don't need the Remap functionality, eliminating a major use of extra instruction bits.

Conventions used in this document:
- Bits are numbered starting from 0 at the LSB, so bit 3 is 1 in the integer 8.
- Bit ranges are inclusive on both ends, so 5:3 means bits 5, 4, and 3.

### Registers

Predicate Registers:

| pr0     | pr1      |
|---------|----------|
| x9 (s0) | x15 (a5) |

Stride Registers:

| stride0  | stride1  | stride2  | stride3  | stride4  | stride5  |
|----------|----------|----------|----------|----------|----------|
| x10 (a0) | x11 (a1) | x12 (a2) | x13 (a3) | x14 (a4) | x15 (a5) |

### 4-bit Vector Length and Predication (vlp4)

| vlp4 | Predication               | Vector Length |
|------|---------------------------|---------------|
| 0000 | unpredicated              | 1 (scalar)    |
| 0001 | unpredicated              | VL * 1        |
| 0010 | unpredicated              | VL * 2        |
| 0011 | unpredicated              | VL * 3        |
| 0100 | unpredicated              | VL * 4        |
| 0101 | _Reserved_                | _Reserved_    |
| 0110 | (pr1 & (1 << N / 1)) != 0 | VL * 1        |
| 0111 | (pr1 & (1 << N / 1)) == 0 | VL * 1        |
| 1000 | (pr0 & (1 << N / 1)) != 0 | VL * 1        |
| 1001 | (pr0 & (1 << N / 1)) == 0 | VL * 1        |
| 1010 | (pr0 & (1 << N / 2)) != 0 | VL * 2        |
| 1011 | (pr0 & (1 << N / 2)) == 0 | VL * 2        |
| 1100 | (pr0 & (1 << N / 3)) != 0 | VL * 3        |
| 1101 | (pr0 & (1 << N / 3)) == 0 | VL * 3        |
| 1110 | (pr0 & (1 << N / 4)) != 0 | VL * 4        |
| 1111 | (pr0 & (1 << N / 4)) == 0 | VL * 4        |

### 3-bit Vector Length and Predication (vlp3)

| vlp3 | Predication               | Vector Length |
|------|---------------------------|---------------|
| 000  | (pr0 & (1 << N / 4)) != 0 | VL * 4        |
| 001  | unpredicated              | VL * 1        |
| 010  | unpredicated              | VL * 2        |
| 011  | unpredicated              | VL * 3        |
| 100  | unpredicated              | VL * 4        |
| 101  | (pr0 & (1 << N / 4)) == 0 | VL * 4        |
| 110  | (pr0 & (1 << N / 1)) != 0 | VL * 1        |
| 111  | (pr0 & (1 << N / 1)) == 0 | VL * 1        |

### 2-bit Vector Length and Predication (vlp2)

| vlp2 | Predication               | Vector Length |
|------|---------------------------|---------------|
| 00   | unpredicated              | VL * 4        |
| 01   | unpredicated              | VL * 1        |
| 10   | unpredicated              | VL * 2        |
| 11   | unpredicated              | VL * 3        |

### Load/Store Kind (ld_st_kind)

| ld_st_kind | Kind               | Address formula                 |
|------------|--------------------|---------------------------------|
| 000        | Compact Vector     | rs1 (scalar) + N * element_size |
| 001        | Gather/Scatter     | rs1 (vector)                    |
| 010        | Strided (stride0)  | rs1 (scalar) + N * stride0      |
| 011        | Strided (stride1)  | rs1 (scalar) + N * stride1      |
| 100        | Strided (stride2)  | rs1 (scalar) + N * stride2      |
| 101        | Strided (stride3)  | rs1 (scalar) + N * stride3      |
| 110        | Strided (stride4)  | rs1 (scalar) + N * stride4      |
| 111        | Strided (stride5)  | rs1 (scalar) + N * stride5      |

### Element Size (elmsz)

| elmsz | Element Size |
|-------|--------------|
| 00    | 8-bit (b)    |
| 01    | 16-bit (h)   |
| 10    | 32-bit (w)   |
| 11    | 64-bit (d)   |

### Floating-point Element Size (felmsz)

| felmsz | Element Size                                   |
|--------|------------------------------------------------|
| 00     | 32-bit (s)                                     |
| 01     | 64-bit (d)                                     |
| 10     | 16-bit (h)                                     |
| 11     | 128-bit (q) (only if Q extension is supported) |

### Convert Destination Type (conv_dest_type)

| conv_dest_type | Dest Element Size | Dest Signedness |
|----------------|-------------------|-----------------|
| 000            | 8-bit             | Unsigned        |
| 001            | 16-bit            | Unsigned        |
| 010            | 32-bit            | Unsigned        |
| 011            | 64-bit            | Unsigned        |
| 100            | 8-bit             | Signed          |
| 101            | 16-bit            | Signed          |
| 110            | 32-bit            | Signed          |
| 111            | 64-bit            | Signed          |

## 16-bit prefixes

### On 32-bit base instructions:

Bits 7:0 are 1011111 (second half of 48-bit instruction set)

<table>
<tr><th>Instruction(s)</th>
<th colspan="30">Base Instruction Encoding</th><th colspan="2">Reused</th><th colspan="8">Prefix</th><th>Original<br/> Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr><td></td><td colspan="5">47:43</td><td>42</td><td>41</td><td colspan="5">40:36</td><td colspan="5">35:31</td><td>30</td><td colspan="2">29:28</td><td colspan="5">27:23</td><td colspan="3">22:20</td><td>19</td><td>18</td><td colspan="2">17:16</td><td>15</td><td>14</td><td>13</td><td>12</td><td>11</td><td>10</td><td>9</td><td>8</td><td></td></tr>
<tr><td></td><td colspan="5">31:27</td><td>26</td><td>25</td><td colspan="5">24:20</td><td colspan="5">19:15</td><td>14</td><td colspan="2">13:12</td><td colspan="5">11:7</td><td colspan="3">6:4</td><td>3</td><td>2</td><td colspan="2">1:0</td><td colspan="17"></td></tr>
<tr><td>lb,lh,lw,ld</td><td colspan="12">imm[11:0]</td><td colspan="5">rs1[4:0]</td><td>ld_st_kind[2]</td><td colspan="2">elmsz</td><td colspan="5">rd[4:0]</td><td colspan="5">00000</td><td colspan="2">rs1[6:5]</td><td colspan="2">rd[6:5]</td><td colspan="2">ld_st_kind[1:0]</td><td colspan="4">vlp4</td><td>lb,lh,lw,ld,lbu,lhu,lwu</td></tr>
<tr><td>flh,flw,fld</td><td colspan="12">imm[11:0]</td><td colspan="5">rs1[4:0]</td><td>ld_st_kind[2]</td><td colspan="2">elmsz</td><td colspan="5">rd[4:0]</td><td colspan="5">00001</td><td colspan="2">rs1[6:5]</td><td colspan="2">rd[6:5]</td><td colspan="2">ld_st_kind[1:0]</td><td colspan="4">vlp4</td><td>flh,flw,fld</td></tr>
<tr><td>addib,addih,addiw,addi</td><td colspan="12" rowspan="6">imm[11:0]</td><td colspan="5" rowspan="9">rs1[4:0]</td><td colspan="3">000</td><td colspan="5" rowspan="9">rd[4:0]</td><td colspan="3" rowspan="9">001</td><td rowspan="9">elmsz[0]</td><td rowspan="9">1</td><td colspan="2" rowspan="9">rs1[6:5]</td><td colspan="2" rowspan="9">rd[6:5]</td><td rowspan="9">elmsz[1]</td><td rowspan="9">0</td><td colspan="4" rowspan="9">vlp4</td><td>addi,addiw</td></tr>
<tr><td>slti.b,slti.h,slti.w,slti.d</td><td colspan="3">010</td><td>slti</td></tr>
<tr><td>sltiu.b,sltiu.h,sltiu.w,sltiu.d</td><td colspan="3">011</td><td>sltiu</td></tr>
<tr><td>xori.b,xori.h,xori.w,xori.d</td><td colspan="3">100</td><td>xori</td></tr>
<tr><td>ori.b,ori.h,ori.w,ori.d</td><td colspan="3">110</td><td>ori</td></tr>
<tr><td>andi.b,andi.h,andi.w,andi.d</td><td colspan="3">111</td><td>andi</td></tr>
<tr><td>slli.b,slli.h,slli.w,slli.d</td><td colspan="6">000000</td><td colspan="6">shamt</td><td colspan="3">001</td><td>slli,slliw</td></tr>
<tr><td>srli.b,srli.h,srli.w,srli.d</td><td colspan="6">000000</td><td colspan="6">shamt</td><td colspan="3">101</td><td>srli,srliw</td></tr>
<tr><td>srai.b,srai.h,srai.w,srai.d</td><td colspan="6">010000</td><td colspan="6">shamt</td><td colspan="3">101</td><td>srai,sraiw</td></tr>
<tr><td>conv.&lt;src>.&lt;dest>,<br/>convu.&lt;src>.&lt;dest></td><td colspan="12">0</td><td colspan="5">rs1[4:0]</td><td colspan="3">conv_dest_type</td><td colspan="5">rd[4:0]</td><td colspan="3">001</td><td>elmsz[0] (rs1)</td><td>1</td><td colspan="2">rs1[6:5]</td><td colspan="2">rd[6:5]</td><td>elmsz[1] (rs1)</td><td>1</td><td colspan="4">vlp4</td><td>addi,addiw,slti,sltiu,xori,ori,andi,slli,slliw,srli,srliw,srai,sraiw</td></tr>
<tr><td><i>Reserved for swizzle, register indirect, and other data movement instructions</i></td><td colspan="12">!= 0</td><td colspan="5">rs1[4:0]</td><td colspan="3">&mdash;</td><td colspan="5">rd[4:0]</td><td colspan="3">001</td><td>elmsz[0] (rs1)</td><td>1</td><td colspan="2">rs1[6:5]</td><td colspan="2">rd[6:5]</td><td>elmsz[1] (rs1)</td><td>1</td><td colspan="4">vlp4</td><td>addi,addiw,slti,sltiu,xori,ori,andi,slli,slliw,srli,srliw,srai,sraiw</td></tr>
<tr><td><i>Reserved for auipc</i></td><td colspan="25">&mdash;</td><td colspan="5">00101</td><td colspan="10">&mdash;</td><td>auipc</td></tr>
<tr><td>sb,sh,sw,sd</td><td colspan="7">imm[11:5]</td><td colspan="5">rs2[4:0]</td><td colspan="5">rs1[4:0]</td><td>ld_st_kind[2]</td><td colspan="2">elmsz</td><td colspan="5">imm[4:0]</td><td colspan="5">01000</td><td colspan="2">rs1[6:5]</td><td colspan="2">ld_st_kind[1:0]</td><td colspan="2">rs2[6:5]</td><td colspan="4">vlp4</td><td>sb,sh,sw,sd</td></tr>
<tr><td>fsh,fsw,fsd</td><td colspan="7">imm[11:5]</td><td colspan="5">rs2[4:0]</td><td colspan="5">rs1[4:0]</td><td>ld_st_kind[2]</td><td colspan="2">elmsz</td><td colspan="5">imm[4:0]</td><td colspan="5">01001</td><td colspan="2">rs1[6:5]</td><td colspan="2">ld_st_kind[1:0]</td><td colspan="2">rs2[6:5]</td><td colspan="4">vlp4</td><td>fsh,fsw,fsd</td></tr>
<tr><td><i>Reserved for atomics</i></td><td colspan="25">&mdash;</td><td colspan="5">01011</td><td colspan="10">&mdash;</td><td>A extension</td></tr>
<tr><td>add.b,add.h,add.w,add.d</td><td colspan="7">0000000</td><td colspan="5" rowspan="18">rs2[4:0]</td><td colspan="5" rowspan="18">rs1[4:0]</td><td colspan="3">000</td><td colspan="5" rowspan="18">rd[4:0]</td><td colspan="3" rowspan="18">011</td><td rowspan="18">elmsz[0]</td><td rowspan="18">1</td><td colspan="2" rowspan="18">rs1[6:5]</td><td colspan="2" rowspan="18">rd[6:5]</td><td colspan="2" rowspan="18">rs2[6:5]</td><td rowspan="18">elmsz[1]</td><td colspan="3" rowspan="18">vlp3</td><td>add,addw</td></tr>
<tr><td>sub.b,sub.h,sub.w,sub.d</td><td colspan="7">0100000</td><td colspan="3">000</td><td>sub,subw</td></tr>
<tr><td>sll.b,sll.h,sll.w,sll.d</td><td colspan="7">0000000</td><td colspan="3">001</td><td>sll,sllw</td></tr>
<tr><td>slt.b,slt.h,slt.w,slt.d</td><td colspan="7">0000000</td><td colspan="3">010</td><td>slt</td></tr>
<tr><td>sltu.b,sltu.h,sltu.w,sltu.d</td><td colspan="7">0000000</td><td colspan="3">011</td><td>sltu</td></tr>
<tr><td>xor.b,xor.h,xor.w,xor.d</td><td colspan="7">0000000</td><td colspan="3">100</td><td>xor</td></tr>
<tr><td>srl.b,srl.h,srl.w,srl.d</td><td colspan="7">0000000</td><td colspan="3">101</td><td>srl,srlw</td></tr>
<tr><td>sra.b,sra.h,sra.w,sra.d</td><td colspan="7">0100000</td><td colspan="3">101</td><td>sra,sraw</td></tr>
<tr><td>or.b,or.h,or.w,or.d</td><td colspan="7">0000000</td><td colspan="3">110</td><td>or</td></tr>
<tr><td>and.b,and.h,and.w,and.d</td><td colspan="7">0000000</td><td colspan="3">111</td><td>and</td></tr>
<tr><td>mul.b,mul.h,mul.w,mul.d</td><td colspan="7">0000001</td><td colspan="3">000</td><td>mul,mulw</td></tr>
<tr><td>mulh.b,mulh.h,mulh.w,mulh.d</td><td colspan="7">0000001</td><td colspan="3">001</td><td>mulh</td></tr>
<tr><td>mulhsu.b,mulhsu.h,mulhsu.w,mulhsu.d</td><td colspan="7">0000001</td><td colspan="3">010</td><td>mulhsu</td></tr>
<tr><td>mulhu.b,mulhu.h,mulhu.w,mulhu.d</td><td colspan="7">0000001</td><td colspan="3">011</td><td>mulhu</td></tr>
<tr><td>div.b,div.h,div.w,div.d</td><td colspan="7">0000001</td><td colspan="3">100</td><td>div,divw</td></tr>
<tr><td>divu.b,divu.h,divu.w,divu.d</td><td colspan="7">0000001</td><td colspan="3">101</td><td>divu,divuw</td></tr>
<tr><td>rem.b,rem.h,rem.w,rem.d</td><td colspan="7">0000001</td><td colspan="3">110</td><td>rem,remw</td></tr>
<tr><td>remu.b,remu.h,remu.w,remu.d</td><td colspan="7">0000001</td><td colspan="3">111</td><td>remu,remuw</td></tr>
<tr><td><i>Reserved for lui</i></td><td colspan="25">&mdash;</td><td colspan="5">01101</td><td colspan="10">&mdash;</td><td>lui</td></tr>
<tr><td>fmadd.s,fmadd.d,fmadd.h,fmadd.q</td><td colspan="5">rs3[4:0]</td><td colspan="2">felmsz</td><td colspan="5">rs2[4:0]</td><td colspan="5">rs1[4:0]</td><td colspan="3">rm</td><td colspan="5">rd[4:0]</td><td colspan="5">10000</td><td colspan="2">rs1[6:5]</td><td colspan="2">rd[6:5]</td><td colspan="2">rs2[6:5]</td><td colspan="2">rs3[6:5]</td><td colspan="2">vlp2</td><td>fmadd.s,fmadd.d,fmadd.h,fmadd.q</td></tr>
<tr><td>fmsub.s,fmsub.d,fmsub.h,fmsub.q</td><td colspan="5">rs3[4:0]</td><td colspan="2">felmsz</td><td colspan="5">rs2[4:0]</td><td colspan="5">rs1[4:0]</td><td colspan="3">rm</td><td colspan="5">rd[4:0]</td><td colspan="5">10001</td><td colspan="2">rs1[6:5]</td><td colspan="2">rd[6:5]</td><td colspan="2">rs2[6:5]</td><td colspan="2">rs3[6:5]</td><td colspan="2">vlp2</td><td>fmsub.s,fmsub.d,fmsub.h,fmsub.q</td></tr>
<tr><td>fnmsub.s,fnmsub.d,fnmsub.h,fnmsub.q</td><td colspan="5">rs3[4:0]</td><td colspan="2">felmsz</td><td colspan="5">rs2[4:0]</td><td colspan="5">rs1[4:0]</td><td colspan="3">rm</td><td colspan="5">rd[4:0]</td><td colspan="5">10010</td><td colspan="2">rs1[6:5]</td><td colspan="2">rd[6:5]</td><td colspan="2">rs2[6:5]</td><td colspan="2">rs3[6:5]</td><td colspan="2">vlp2</td><td>fnmsub.s,fnmsub.d,fnmsub.h,fnmsub.q</td></tr>
<tr><td>fnmadd.s,fnmadd.d,fnmadd.h,fnmadd.q</td><td colspan="5">rs3[4:0]</td><td colspan="2">felmsz</td><td colspan="5">rs2[4:0]</td><td colspan="5">rs1[4:0]</td><td colspan="3">rm</td><td colspan="5">rd[4:0]</td><td colspan="5">10011</td><td colspan="2">rs1[6:5]</td><td colspan="2">rd[6:5]</td><td colspan="2">rs2[6:5]</td><td colspan="2">rs3[6:5]</td><td colspan="2">vlp2</td><td>fnmadd.s,fnmadd.d,fnmadd.h,fnmadd.q</td></tr>
<tr><td>FIXME: finish</td></tr>
</table>

### On 16-bit base instructions:

Bits 6:0 are 0101011 (custom-1).

<table>
<tr><th>Instruction</th>
<th colspan="16">Instruction Encoding</th><th colspan="9">Prefix</th><th>Original<br/> Instruction<br/>(from C extension)</th>
</tr>
<tr><td></td><td colspan="3">31:29</td><td>28</td><td colspan="2">27:26</td><td colspan="3">25:23</td><td colspan="2">22:21</td><td colspan="3">20:18</td><td colspan="2">17:16</td><td>15</td><td>14</td><td>13</td><td>12</td><td>11</td><td>10</td><td>9</td><td>8</td><td>7</td><td></td></tr>
<tr><td></td><td colspan="3">15:13</td><td>12</td><td colspan="2">11:10</td><td colspan="3">9:7</td><td colspan="2">6:5</td><td colspan="3">4:2</td><td colspan="2">1:0</td><td colspan="17"></td></tr>
<tr><td><i>Reserved</i></td><td colspan="3">000</td><td colspan="11">&mdash;</td><td colspan="2">00</td><td colspan="9">&mdash;</td><td>c.addi4spn</td></tr>
<tr><td>c.fld</td><td colspan="3">001</td><td colspan="3">uimm[5:3]</td><td colspan="3">rs1&prime;</td><td colspan="2">uimm[7:6]</td><td colspan="3">rd&prime;</td><td colspan="2">00</td><td colspan="9">FIXME: finish</td><td>c.fld</td></tr>
<tr><td>c.mv,c.conv</td><td colspan="3">100</td><td>0</td><td colspan="5">rd[4:0]</td><td colspan="5">rs1[4:0]</td><td colspan="2">10</td><td colspan="2">rs1[6:5]</td><td colspan="2">rd[6:5]</td><td colspan="5">Vector Length /<br/>Element Types</td><td>c.mv,c.jr</td></tr>
</table>

## Vector Length Encodings
### c&#46;mv/c.conv Vector Length / Element Types

TODO: Reorder to make decoding faster and select which length-types combinations are most used

<table>
<tr><th>Encoding</th><th>Length</th><th>rs1 Type</th><th>rd Type</th><th>Sign Extension</th></tr>
<tr><td>00000</td><td>VL * 1</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td rowspan="2">00001</td><td>VL * 2</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td>VL * 1</td><td>u16</td><td>u16</td><td>N/A</td></tr>
<tr><td>00010</td><td>VL * 3</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td rowspan="3">00011</td><td>VL * 4</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td>VL * 2</td><td>u16</td><td>u16</td><td>N/A</td></tr>
<tr><td>VL * 1</td><td>u32</td><td>u32</td><td>N/A</td></tr>
<tr><td rowspan="2">00100</td><td>VL * 6</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td>VL * 3</td><td>u16</td><td>u16</td><td>N/A</td></tr>
<tr><td rowspan="4">00101</td><td>VL * 8</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td>VL * 4</td><td>u16</td><td>u16</td><td>N/A</td></tr>
<tr><td>VL * 2</td><td>u32</td><td>u32</td><td>N/A</td></tr>
<tr><td>VL * 1</td><td>u64</td><td>u64</td><td>N/A</td></tr>
<tr><td rowspan="3">00110</td><td>VL * 3</td><td>u32</td><td>u32</td><td>N/A</td></tr>
<tr><td>VL * 6</td><td>u16</td><td>u16</td><td>N/A</td></tr>
<tr><td>VL * 12</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td rowspan="4">00111</td><td>VL * 16</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td>VL * 8</td><td>u16</td><td>u16</td><td>N/A</td></tr>
<tr><td>VL * 4</td><td>u32</td><td>u32</td><td>N/A</td></tr>
<tr><td>VL * 2</td><td>u64</td><td>u64</td><td>N/A</td></tr>
<tr><td rowspan="4">01000</td><td>VL * 24</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td>VL * 12</td><td>u16</td><td>u16</td><td>N/A</td></tr>
<tr><td>VL * 6</td><td>u32</td><td>u32</td><td>N/A</td></tr>
<tr><td>VL * 3</td><td>u64</td><td>u64</td><td>N/A</td></tr>
<tr><td rowspan="4">01001</td><td>VL * 32</td><td>u8</td><td>u8</td><td>N/A</td></tr>
<tr><td>VL * 16</td><td>u16</td><td>u16</td><td>N/A</td></tr>
<tr><td>VL * 8</td><td>u32</td><td>u32</td><td>N/A</td></tr>
<tr><td>VL * 4</td><td>u64</td><td>u64</td><td>N/A</td></tr>
<tr><td>01010</td><td><i>Reserved</i></td><td><i>Reserved</i></td><td><i>Reserved</i></td><td><i>Reserved</i></td></tr>
<tr><td>01011</td><td><i>Reserved</i></td><td><i>Reserved</i></td><td><i>Reserved</i></td><td><i>Reserved</i></td></tr>
<tr><td>01100</td><td><i>Reserved</i></td><td><i>Reserved</i></td><td><i>Reserved</i></td><td><i>Reserved</i></td></tr>
<tr><td>01101</td><td><i>Reserved</i></td><td><i>Reserved</i></td><td><i>Reserved</i></td><td><i>Reserved</i></td></tr>
<tr><td>01110</td><td>VL</td><td>u16</td><td>u8</td><td>N/A</td></tr>
<tr><td>01111</td><td>VL</td><td>u32</td><td>u8</td><td>N/A</td></tr>
<tr><td>10000</td><td>VL</td><td>u32</td><td>u16</td><td>N/A</td></tr>
<tr><td>10001</td><td>VL</td><td>u64</td><td>u8</td><td>N/A</td></tr>
<tr><td>10010</td><td>VL</td><td>u64</td><td>u16</td><td>N/A</td></tr>
<tr><td>10011</td><td>VL</td><td>u64</td><td>u32</td><td>N/A</td></tr>
<tr><td>10100</td><td>VL</td><td>u8</td><td>u16</td><td>Zero Extension</td></tr>
<tr><td>10101</td><td>VL</td><td>i8</td><td>i16</td><td>Sign Extension</td></tr>
<tr><td>10110</td><td>VL</td><td>u8</td><td>u32</td><td>Zero Extension</td></tr>
<tr><td>10111</td><td>VL</td><td>i8</td><td>i32</td><td>Sign Extension</td></tr>
<tr><td>11000</td><td>VL</td><td>u16</td><td>u32</td><td>Zero Extension</td></tr>
<tr><td>11001</td><td>VL</td><td>i16</td><td>i32</td><td>Sign Extension</td></tr>
<tr><td>11010</td><td>VL</td><td>u8</td><td>u64</td><td>Zero Extension</td></tr>
<tr><td>11011</td><td>VL</td><td>i8</td><td>i64</td><td>Sign Extension</td></tr>
<tr><td>11100</td><td>VL</td><td>u16</td><td>u64</td><td>Zero Extension</td></tr>
<tr><td>11101</td><td>VL</td><td>i16</td><td>i64</td><td>Sign Extension</td></tr>
<tr><td>11110</td><td>VL</td><td>u32</td><td>u64</td><td>Zero Extension</td></tr>
<tr><td>11111</td><td>VL</td><td>i32</td><td>i64</td><td>Sign Extension</td></tr>
</table>
