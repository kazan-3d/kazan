# SimpleV Prefix Proposal

Note that if we have a transpose instruction, we don't need the Remap functionality, eliminating a major use of extra instruction bits.

All operations are packed by default.

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

### Load/Store Kind (lsk)

| lsk | Kind               | Address formula                 |
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

### Element Size W (elmszw)

| elmszw | Element Size |
|--------|--------------|
| 0      | 8-bit (b)    |
| 1      | 32-bit (w)   |

### Element Size D (elmszd)

| elmszd | Element Size |
|--------|--------------|
| 0      | 16-bit (h)   |
| 1      | 64-bit (d)   |

### Floating-point Element Size (felmsz)

| felmsz | Element Size                                   |
|--------|------------------------------------------------|
| 00     | 32-bit (s)                                     |
| 01     | 64-bit (d)                                     |
| 10     | 16-bit (h)                                     |
| 11     | 128-bit (q) (only if Q extension is supported) |

## 16-bit prefixes

### 32-bit base instructions

All 48-bit instructions have bits 7:0 set to 1011111 (second half of 48-bit instruction set).

#### Load/Store operations
<table>
<tr>
    <th>Instruction(s)</th>
    <th colspan="30">Base Instruction Encoding</th>
    <th colspan="2">Reused</th>
    <th colspan="8">Prefix</th>
    <th>Original<br/> Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr>
    <td></td>
    <td colspan="7">47:41</td>
    <td colspan="5">40:36</td>
    <td colspan="5">35:31</td>
    <td>30</td>
    <td colspan="2">29:28</td>
    <td colspan="5">27:23</td>
    <td colspan="5">22:18</td>
    <td colspan="2">17:16</td>
    <td colspan="2">15:14</td>
    <td colspan="2">13:12</td>
    <td colspan="4">11:8</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="7">31:25</td>
    <td colspan="5">24:20</td>
    <td colspan="5">19:15</td>
    <td>14</td>
    <td colspan="2">13:12</td>
    <td colspan="5">11:7</td>
    <td colspan="5">6:2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td>lb,lh,lw,ld</td>
    <td colspan="12" rowspan="2">imm[11:0]</td>
    <td colspan="5" rowspan="4">rs1[4:0]</td>
    <td rowspan="4">lsk[2]</td>
    <td colspan="2" rowspan="4">elmsz</td>
    <td colspan="5" rowspan="2">rd[4:0]</td>
    <td colspan="5">00000</td>
    <td colspan="2" rowspan="4">rs1[6:5]</td>
    <td colspan="2" rowspan="2">rd[6:5]</td>
    <td colspan="2" rowspan="2">lsk[1:0]</td>
    <td colspan="4" rowspan="4">vlp4</td>
    <td>lb,lh,lw,ld,lbu,lhu,lwu</td>
</tr>
<tr>
    <td>flh,flw,fld</td>
    <td colspan="5">00001</td>
    <td>flh,flw,fld</td>
</tr>
<tr>
    <td>sb,sh,sw,sd</td>
    <td colspan="7" rowspan="2">imm[11:5]</td>
    <td colspan="5" rowspan="2">rs2[4:0]</td>
    <td colspan="5" rowspan="2">imm[4:0]</td>
    <td colspan="5">01000</td>
    <td colspan="2" rowspan="2">lsk[1:0]</td>
    <td colspan="2" rowspan="2">rs2[6:5]</td>
    <td>sb,sh,sw,sd</td>
</tr>
<tr>
    <td>fsh,fsw,fsd</td>
    <td colspan="5">01001</td>
    <td>fsh,fsw,fsd</td>
</tr>
</table>

#### OP-IMM operations
<table>
<tr>
    <th>Instruction(s)</th>
    <th colspan="30">Base Instruction Encoding</th>
    <th colspan="2">Reused</th>
    <th colspan="8">Prefix</th>
    <th>Original<br/>Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr>
    <td></td>
    <td colspan="6">47:42</td>
    <td colspan="6">41:36</td>
    <td colspan="5">35:31</td>
    <td colspan="3">30:28</td>
    <td colspan="5">27:23</td>
    <td colspan="5">22:18</td>
    <td colspan="2">17:16</td>
    <td colspan="2">15:14</td>
    <td>13</td>
    <td>12</td>
    <td colspan="4">11:8</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="6">31:26</td>
    <td colspan="6">25:20</td>
    <td colspan="5">19:15</td>
    <td colspan="3">14:12</td>
    <td colspan="5">11:7</td>
    <td colspan="5">6:2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td>addi.h,addi.d</td>
    <td colspan="12" rowspan="6">imm[11:0]</td>
    <td colspan="5" rowspan="18">rs1[4:0]</td>
    <td colspan="3">000</td>
    <td colspan="5" rowspan="18">rd[4:0]</td>
    <td colspan="5" rowspan="9">00101</td>
    <td colspan="2" rowspan="18">rs1[6:5]</td>
    <td colspan="2" rowspan="18">rd[6:5]</td>
    <td rowspan="9">elmszd</td>
    <td rowspan="18">0</td>
    <td colspan="4" rowspan="18">vlp4</td>
    <td>addi</td>
</tr>
<tr>
    <td>slti.h,slti.d</td>
    <td colspan="3">010</td>
    <td>slti</td>
</tr>
<tr>
    <td>sltiu.h,sltiu.d</td>
    <td colspan="3">011</td>
    <td>sltiu</td>
</tr>
<tr>
    <td>xori.h,xori.d</td>
    <td colspan="3">100</td>
    <td>xori</td>
</tr>
<tr>
    <td>ori.h,ori.d</td>
    <td colspan="3">110</td>
    <td>ori</td>
</tr>
<tr>
    <td>andi.h,andi.d</td>
    <td colspan="3">111</td>
    <td>andi</td>
</tr>
<tr>
    <td>slli.h,slli.d</td>
    <td colspan="6">000000</td>
    <td colspan="6" rowspan="3">shamt</td>
    <td colspan="3">001</td>
    <td>slli</td>
</tr>
<tr>
    <td>srli.h,srli.d</td>
    <td colspan="6">000000</td>
    <td colspan="3">101</td>
    <td>srli</td>
</tr>
<tr>
    <td>srai.h,srai.d</td>
    <td colspan="6">010000</td>
    <td colspan="3">101</td>
    <td>srai</td>
</tr>
<tr>
    <td>addi.b,addi.w</td>
    <td colspan="12" rowspan="6">imm[11:0]</td>
    <td colspan="3">000</td>
    <td colspan="5" rowspan="9">00111</td>
    <td rowspan="9">elmszw</td>
    <td>addiw</td>
</tr>
<tr>
    <td>slti.b,slti.w</td>
    <td colspan="3">010</td>
    <td><s>sltiw</s></td>
</tr>
<tr>
    <td>sltiu.b,sltiu.w</td>
    <td colspan="3">011</td>
    <td><s>sltiuw</s></td>
</tr>
<tr>
    <td>xori.b,xori.w</td>
    <td colspan="3">100</td>
    <td><s>xoriw</s></td>
</tr>
<tr>
    <td>ori.b,ori.w</td>
    <td colspan="3">110</td>
    <td><s>oriw</s></td>
</tr>
<tr>
    <td>andi.b,andi.w</td>
    <td colspan="3">111</td>
    <td><s>andiw</s></td>
</tr>
<tr>
    <td>slli.b,slli.w</td>
    <td colspan="6">000000</td>
    <td colspan="6" rowspan="3">shamt</td>
    <td colspan="3">001</td>
    <td>slliw</td>
</tr>
<tr>
    <td>srli.b,srli.w</td>
    <td colspan="6">000000</td>
    <td colspan="3">101</td>
    <td>srliw</td>
</tr>
<tr>
    <td>srai.b,srai.w</td>
    <td colspan="6">010000</td>
    <td colspan="3">101</td>
    <td>sraiw</td>
</tr>
</table>

#### Integer Type Conversion

<table>
<tr>
    <th>Instruction(s)</th>
    <th colspan="30">Base Instruction Encoding</th>
    <th colspan="2">Reused</th>
    <th colspan="8">Prefix</th>
    <th>Original<br/>Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr>
    <td></td>
    <td colspan="12">47:36</td>
    <td colspan="5">35:31</td>
    <td>30</td>
    <td colspan="2">29:28</td>
    <td colspan="5">27:23</td>
    <td colspan="5">22:18</td>
    <td colspan="2">17:16</td>
    <td>15</td>
    <td>14</td>
    <td>13</td>
    <td>12</td>
    <td>11</td>
    <td>10</td>
    <td>9</td>
    <td>8</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="12">31:20</td>
    <td colspan="5">19:15</td>
    <td>14</td>
    <td colspan="2">13:12</td>
    <td colspan="5">11:7</td>
    <td colspan="5">6:2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td>convu.&lt;src>.&lt;dest></td>
    <td colspan="12" rowspan="2">0</td>
    <td colspan="5" rowspan="2">rs1[4:0]</td>
    <td colspan="1" rowspan="2">src elmsz[1]</td>
    <td colspan="2" rowspan="2">dest elmsz</td>
    <td colspan="5" rowspan="2">rd[4:0]</td>
    <td colspan="5">00101</td>
    <td colspan="2" rowspan="2">rs1[6:5]</td>
    <td colspan="2" rowspan="2">rd[6:5]</td>
    <td rowspan="2">src elmsz[0]</td>
    <td rowspan="2">1</td>
    <td colspan="4" rowspan="2">vlp4</td>
    <td>addi,slti,sltiu,xori,ori,andi,slli,srli,srai</td>
</tr>
<tr>
    <td>conv.&lt;src>.&lt;dest></td>
    <td colspan="5">00111</td>
    <td>addiw,slliw,srliw,sraiw</td>
</tr>
</table>

#### Vector Data-Movement Operations
<table>
<tr>
    <th>Instruction(s)</th>
    <th colspan="30">Base Instruction Encoding</th>
    <th colspan="2">Reused</th>
    <th colspan="8">Prefix</th>
    <th>Original<br/>Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr>
    <td></td>
    <td colspan="5">47:43</td>
    <td>42</td>
    <td>41</td>
    <td colspan="5">40:36</td>
    <td colspan="5">35:31</td>
    <td>30</td>
    <td colspan="2">29:28</td>
    <td colspan="5">27:23</td>
    <td colspan="3">22:20</td>
    <td>19</td>
    <td>18</td>
    <td colspan="2">17:16</td>
    <td>15</td>
    <td>14</td>
    <td>13</td>
    <td>12</td>
    <td>11</td>
    <td>10</td>
    <td>9</td>
    <td>8</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="5">31:27</td>
    <td>26</td>
    <td>25</td>
    <td colspan="5">24:20</td>
    <td colspan="5">19:15</td>
    <td>14</td>
    <td colspan="2">13:12</td>
    <td colspan="5">11:7</td>
    <td colspan="3">6:4</td>
    <td>3</td>
    <td>2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td><i>Reserved<br/>for swizzle,<br/>register indirect,<br/>and other<br/>data-movement<br/>instructions</i></td>
    <td colspan="12">!= 0</td>
    <td colspan="5">rs1[4:0]</td>
    <td colspan="3">&mdash;</td>
    <td colspan="5">rd[4:0]</td>
    <td colspan="3">001</td>
    <td>elmsz[0] (rs1)</td>
    <td>1</td>
    <td colspan="2">rs1[6:5]</td>
    <td colspan="2">rd[6:5]</td>
    <td>elmsz[1] (rs1)</td>
    <td>1</td>
    <td colspan="4">vlp4</td>
    <td>addi,addiw,slti,sltiu,xori,ori,andi,slli,slliw,srli,srliw,srai,sraiw</td>
</tr>
</table>

#### Reserved operations
<table>
<tr>
    <th>Instruction(s)</th>
    <th colspan="30">Base Instruction Encoding</th>
    <th colspan="2">Reused</th>
    <th colspan="8">Prefix</th>
    <th>Original<br/> Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr>
    <td></td>
    <td colspan="5">47:43</td>
    <td>42</td>
    <td>41</td>
    <td colspan="5">40:36</td>
    <td colspan="5">35:31</td>
    <td>30</td>
    <td colspan="2">29:28</td>
    <td colspan="5">27:23</td>
    <td colspan="3">22:20</td>
    <td>19</td>
    <td>18</td>
    <td colspan="2">17:16</td>
    <td>15</td>
    <td>14</td>
    <td>13</td>
    <td>12</td>
    <td>11</td>
    <td>10</td>
    <td>9</td>
    <td>8</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="5">31:27</td>
    <td>26</td>
    <td>25</td>
    <td colspan="5">24:20</td>
    <td colspan="5">19:15</td>
    <td>14</td>
    <td colspan="2">13:12</td>
    <td colspan="5">11:7</td>
    <td colspan="3">6:4</td>
    <td>3</td>
    <td>2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td><i>Reserved for<br/>auipc</i></td>
    <td colspan="25">&mdash;</td>
    <td colspan="5">00101</td>
    <td colspan="10">&mdash;</td>
    <td>auipc</td>
</tr>
<tr>
    <td><i>Reserved for<br/>lui</i></td>
    <td colspan="25">&mdash;</td>
    <td colspan="5">01101</td>
    <td colspan="10">&mdash;</td>
    <td>lui</td>
</tr>
<tr>
    <td><i>Reserved for<br/>atomics</i></td>
    <td colspan="25">&mdash;</td>
    <td colspan="5">01011</td>
    <td colspan="10">&mdash;</td>
    <td>A extension</td>
</tr>
</table>

#### OP operations
<table>
<tr>
    <th>Instruction(s)</th>
    <th colspan="30">Base Instruction Encoding</th>
    <th colspan="2">Reused</th>
    <th colspan="8">Prefix</th>
    <th>Original<br/> Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr>
    <td></td>
    <td colspan="7">47:41</td>
    <td colspan="5">40:36</td>
    <td colspan="5">35:31</td>
    <td colspan="3">30:28</td>
    <td colspan="5">27:23</td>
    <td colspan="5">22:18</td>
    <td colspan="2">17:16</td>
    <td colspan="2">15:14</td>
    <td colspan="2">13:12</td>
    <td>11</td>
    <td colspan="3">10:8</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="7">31:25</td>
    <td colspan="5">24:20</td>
    <td colspan="5">19:15</td>
    <td colspan="3">14:12</td>
    <td colspan="5">11:7</td>
    <td colspan="5">6:2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td>add.h,add.d</td>
    <td colspan="7">0000000</td>
    <td colspan="5" rowspan="20">rs2[4:0]</td>
    <td colspan="5" rowspan="20">rs1[4:0]</td>
    <td colspan="3">000</td>
    <td colspan="5" rowspan="20">rd[4:0]</td>
    <td colspan="5" rowspan="10">01101</td>
    <td colspan="2" rowspan="20">rs1[6:5]</td>
    <td colspan="2" rowspan="20">rd[6:5]</td>
    <td colspan="2" rowspan="20">rs2[6:5]</td>
    <td rowspan="10">elmszd</td>
    <td colspan="3" rowspan="20">vlp3</td>
    <td>add</td>
</tr>
<tr>
    <td>sub.h,sub.d</td>
    <td colspan="7">0100000</td>
    <td colspan="3">000</td>
    <td>sub</td>
</tr>
<tr>
    <td>sll.h,sll.d</td>
    <td colspan="7">0000000</td>
    <td colspan="3">001</td>
    <td>sll</td>
</tr>
<tr>
    <td>slt.h,slt.d</td>
    <td colspan="7">0000000</td>
    <td colspan="3">010</td>
    <td>slt</td>
</tr>
<tr>
    <td>sltu.h,sltu.d</td>
    <td colspan="7">0000000</td>
    <td colspan="3">011</td>
    <td>sltu</td>
</tr>
<tr>
    <td>xor.h,xor.d</td>
    <td colspan="7">0000000</td>
    <td colspan="3">100</td>
    <td>xor</td>
</tr>
<tr>
    <td>srl.h,srl.d</td>
    <td colspan="7">0000000</td>
    <td colspan="3">101</td>
    <td>srl</td>
</tr>
<tr>
    <td>sra.h,sra.d</td>
    <td colspan="7">0100000</td>
    <td colspan="3">101</td>
    <td>sra</td>
</tr>
<tr>
    <td>or.h,or.d</td>
    <td colspan="7">0000000</td>
    <td colspan="3">110</td>
    <td>or</td>
</tr>
<tr>
    <td>and.h,and.d</td>
    <td colspan="7">0000000</td>
    <td colspan="3">111</td>
    <td>and</td>
</tr>
<tr>
    <td>add.b,add.w</td>
    <td colspan="7">0000000</td>
    <td colspan="3">000</td>
    <td colspan="5" rowspan="10">01111</td>
    <td rowspan="10">elmszw</td>
    <td>addw</td>
</tr>
<tr>
    <td>sub.b,sub.w</td>
    <td colspan="7">0100000</td>
    <td colspan="3">000</td>
    <td>subw</td>
</tr>
<tr>
    <td>sll.b,sll.w</td>
    <td colspan="7">0000000</td>
    <td colspan="3">001</td>
    <td>sllw</td>
</tr>
<tr>
    <td>slt.b,slt.w</td>
    <td colspan="7">0000000</td>
    <td colspan="3">010</td>
    <td><s>sltw</s></td>
</tr>
<tr>
    <td>sltu.b,sltu.w</td>
    <td colspan="7">0000000</td>
    <td colspan="3">011</td>
    <td><s>sltuw</s></td>
</tr>
<tr>
    <td>xor.b,xor.w</td>
    <td colspan="7">0000000</td>
    <td colspan="3">100</td>
    <td><s>xorw</s></td>
</tr>
<tr>
    <td>srl.b,srl.w</td>
    <td colspan="7">0000000</td>
    <td colspan="3">101</td>
    <td>srlw</td>
</tr>
<tr>
    <td>sra.b,sra.w</td>
    <td colspan="7">0100000</td>
    <td colspan="3">101</td>
    <td>sraw</td>
</tr>
<tr>
    <td>or.b,or.w</td>
    <td colspan="7">0000000</td>
    <td colspan="3">110</td>
    <td><s>orw</s></td>
</tr>
<tr>
    <td>and.b,and.w</td>
    <td colspan="7">0000000</td>
    <td colspan="3">111</td>
    <td><s>andw</s></td>
</tr>
</table>

#### M extension operations
<table>
<tr>
    <th>Instruction(s)</th>
    <th colspan="30">Base Instruction Encoding</th>
    <th colspan="2">Reused</th>
    <th colspan="8">Prefix</th>
    <th>Original<br/> Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr>
    <td></td>
    <td colspan="7">47:41</td>
    <td colspan="5">40:36</td>
    <td colspan="5">35:31</td>
    <td colspan="3">30:28</td>
    <td colspan="5">27:23</td>
    <td colspan="5">22:18</td>
    <td colspan="2">17:16</td>
    <td colspan="2">15:14</td>
    <td colspan="2">13:12</td>
    <td>11</td>
    <td colspan="3">10:8</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="7">31:25</td>
    <td colspan="5">24:20</td>
    <td colspan="5">19:15</td>
    <td colspan="3">14:12</td>
    <td colspan="5">11:7</td>
    <td colspan="5">6:2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td>mul.h,mul.d</td>
    <td colspan="7">0000001</td>
    <td colspan="5" rowspan="16">rs2[4:0]</td>
    <td colspan="5" rowspan="16">rs1[4:0]</td>
    <td colspan="3">000</td>
    <td colspan="5" rowspan="16">rd[4:0]</td>
    <td colspan="5" rowspan="8">01101</td>
    <td colspan="2" rowspan="16">rs1[6:5]</td>
    <td colspan="2" rowspan="16">rd[6:5]</td>
    <td colspan="2" rowspan="16">rs2[6:5]</td>
    <td rowspan="8">elmszd</td>
    <td colspan="3" rowspan="16">vlp3</td>
    <td>mul</td>
</tr>
<tr>
    <td>mulh.h,mulh.d</td>
    <td colspan="7">0000001</td>
    <td colspan="3">001</td>
    <td>mulh</td>
</tr>
<tr>
    <td>mulhsu.h,mulhsu.d</td>
    <td colspan="7">0000001</td>
    <td colspan="3">010</td>
    <td>mulhsu</td>
</tr>
<tr>
    <td>mulhu.h,mulhu.d</td>
    <td colspan="7">0000001</td>
    <td colspan="3">011</td>
    <td>mulhu</td>
</tr>
<tr>
    <td>div.h,div.d</td>
    <td colspan="7">0000001</td>
    <td colspan="3">100</td>
    <td>div</td>
</tr>
<tr>
    <td>divu.h,divu.d</td>
    <td colspan="7">0000001</td>
    <td colspan="3">101</td>
    <td>divu</td>
</tr>
<tr>
    <td>rem.h,rem.d</td>
    <td colspan="7">0000001</td>
    <td colspan="3">110</td>
    <td>rem</td>
</tr>
<tr>
    <td>remu.h,remu.d</td>
    <td colspan="7">0000001</td>
    <td colspan="3">111</td>
    <td>remu</td>
</tr>
<tr>
    <td>mul.b,mul.w</td>
    <td colspan="7">0000001</td>
    <td colspan="3">000</td>
    <td colspan="5" rowspan="8">01111</td>
    <td rowspan="8">elmszw</td>
    <td>mulw</td>
</tr>
<tr>
    <td>mulh.b,mulh.w</td>
    <td colspan="7">0000001</td>
    <td colspan="3">001</td>
    <td><s>mulhw</s></td>
</tr>
<tr>
    <td>mulhsu.b,mulhsu.w</td>
    <td colspan="7">0000001</td>
    <td colspan="3">010</td>
    <td><s>mulhsuw</s></td>
</tr>
<tr>
    <td>mulhu.b,mulhu.w</td>
    <td colspan="7">0000001</td>
    <td colspan="3">011</td>
    <td><s>mulhuw</s></td>
</tr>
<tr>
    <td>div.b,div.w</td>
    <td colspan="7">0000001</td>
    <td colspan="3">100</td>
    <td>divw</td>
</tr>
<tr>
    <td>divu.b,divu.w</td>
    <td colspan="7">0000001</td>
    <td colspan="3">101</td>
    <td>divuw</td>
</tr>
<tr>
    <td>rem.b,rem.w</td>
    <td colspan="7">0000001</td>
    <td colspan="3">110</td>
    <td>remw</td>
</tr>
<tr>
    <td>remu.b,remu.w</td>
    <td colspan="7">0000001</td>
    <td colspan="3">111</td>
    <td>remuw</td>
</tr>
</table>

#### mul-add operations
<table>
<tr>
    <th>Instruction(s)</th>
    <th colspan="30">Base Instruction Encoding</th>
    <th colspan="2">Reused</th>
    <th colspan="8">Prefix</th>
    <th>Original<br/> Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr>
    <td></td>
    <td colspan="5">47:43</td>
    <td>42</td>
    <td>41</td>
    <td colspan="5">40:36</td>
    <td colspan="5">35:31</td>
    <td>30</td>
    <td colspan="2">29:28</td>
    <td colspan="5">27:23</td>
    <td colspan="3">22:20</td>
    <td>19</td>
    <td>18</td>
    <td colspan="2">17:16</td>
    <td>15</td>
    <td>14</td>
    <td>13</td>
    <td>12</td>
    <td>11</td>
    <td>10</td>
    <td>9</td>
    <td>8</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="5">31:27</td>
    <td>26</td>
    <td>25</td>
    <td colspan="5">24:20</td>
    <td colspan="5">19:15</td>
    <td>14</td>
    <td colspan="2">13:12</td>
    <td colspan="5">11:7</td>
    <td colspan="3">6:4</td>
    <td>3</td>
    <td>2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td>fmadd.s,fmadd.d,<br/>fmadd.h,fmadd.q</td>
    <td colspan="5">rs3[4:0]</td>
    <td colspan="2">felmsz</td>
    <td colspan="5">rs2[4:0]</td>
    <td colspan="5">rs1[4:0]</td>
    <td colspan="3">rm</td>
    <td colspan="5">rd[4:0]</td>
    <td colspan="5">10000</td>
    <td colspan="2">rs1[6:5]</td>
    <td colspan="2">rd[6:5]</td>
    <td colspan="2">rs2[6:5]</td>
    <td colspan="2">rs3[6:5]</td>
    <td colspan="2">vlp2</td>
    <td>fmadd.s,fmadd.d,fmadd.h,fmadd.q</td>
</tr>
<tr>
    <td>fmsub.s,fmsub.d,<br/>fmsub.h,fmsub.q</td>
    <td colspan="5">rs3[4:0]</td>
    <td colspan="2">felmsz</td>
    <td colspan="5">rs2[4:0]</td>
    <td colspan="5">rs1[4:0]</td>
    <td colspan="3">rm</td>
    <td colspan="5">rd[4:0]</td>
    <td colspan="5">10001</td>
    <td colspan="2">rs1[6:5]</td>
    <td colspan="2">rd[6:5]</td>
    <td colspan="2">rs2[6:5]</td>
    <td colspan="2">rs3[6:5]</td>
    <td colspan="2">vlp2</td>
    <td>fmsub.s,fmsub.d,fmsub.h,fmsub.q</td>
</tr>
<tr>
    <td>fnmsub.s,<br/>fnmsub.d,<br/>fnmsub.h,<br/>fnmsub.q</td>
    <td colspan="5">rs3[4:0]</td>
    <td colspan="2">felmsz</td>
    <td colspan="5">rs2[4:0]</td>
    <td colspan="5">rs1[4:0]</td>
    <td colspan="3">rm</td>
    <td colspan="5">rd[4:0]</td>
    <td colspan="5">10010</td>
    <td colspan="2">rs1[6:5]</td>
    <td colspan="2">rd[6:5]</td>
    <td colspan="2">rs2[6:5]</td>
    <td colspan="2">rs3[6:5]</td>
    <td colspan="2">vlp2</td>
    <td>fnmsub.s,fnmsub.d,fnmsub.h,fnmsub.q</td>
</tr>
<tr>
    <td>fnmadd.s,<br/>fnmadd.d,<br/>fnmadd.h,<br/>fnmadd.q</td>
    <td colspan="5">rs3[4:0]</td>
    <td colspan="2">felmsz</td>
    <td colspan="5">rs2[4:0]</td>
    <td colspan="5">rs1[4:0]</td>
    <td colspan="3">rm</td>
    <td colspan="5">rd[4:0]</td>
    <td colspan="5">10011</td>
    <td colspan="2">rs1[6:5]</td>
    <td colspan="2">rd[6:5]</td>
    <td colspan="2">rs2[6:5]</td>
    <td colspan="2">rs3[6:5]</td>
    <td colspan="2">vlp2</td>
    <td>fnmadd.s,fnmadd.d,fnmadd.h,fnmadd.q</td>
</tr>
</table>

#### Misc operations
<table>
<tr>
    <th>Instruction(s)</th>
    <th colspan="30">Base Instruction Encoding</th>
    <th colspan="2">Reused</th>
    <th colspan="8">Prefix</th>
    <th>Original<br/> Instruction(s)<br/>(from RV64G)</th>
</tr>
<tr>
    <td></td>
    <td colspan="5">47:43</td>
    <td>42</td>
    <td>41</td>
    <td colspan="5">40:36</td>
    <td colspan="5">35:31</td>
    <td>30</td>
    <td colspan="2">29:28</td>
    <td colspan="5">27:23</td>
    <td colspan="3">22:20</td>
    <td>19</td>
    <td>18</td>
    <td colspan="2">17:16</td>
    <td>15</td>
    <td>14</td>
    <td>13</td>
    <td>12</td>
    <td>11</td>
    <td>10</td>
    <td>9</td>
    <td>8</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="5">31:27</td>
    <td>26</td>
    <td>25</td>
    <td colspan="5">24:20</td>
    <td colspan="5">19:15</td>
    <td>14</td>
    <td colspan="2">13:12</td>
    <td colspan="5">11:7</td>
    <td colspan="3">6:4</td>
    <td>3</td>
    <td>2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td>FIXME: finish</td>
</tr>
</table>

### On 16-bit base instructions:

Bits 6:0 are 0101011 (custom-1).

<table>
<tr><th>Instruction</th>
<th colspan="16">Instruction Encoding</th>
<th colspan="9">Prefix</th>
<th>Original<br/> Instruction<br/>(from C extension)</th>
</tr>
<tr>
    <td></td>
    <td colspan="3">31:29</td>
    <td>28</td>
    <td colspan="2">27:26</td>
    <td colspan="3">25:23</td>
    <td colspan="2">22:21</td>
    <td colspan="3">20:18</td>
    <td colspan="2">17:16</td>
    <td>15</td>
    <td>14</td>
    <td>13</td>
    <td>12</td>
    <td>11</td>
    <td>10</td>
    <td>9</td>
    <td>8</td>
    <td>7</td>
    <td></td>
</tr>
<tr>
    <td></td>
    <td colspan="3">15:13</td>
    <td>12</td>
    <td colspan="2">11:10</td>
    <td colspan="3">9:7</td>
    <td colspan="2">6:5</td>
    <td colspan="3">4:2</td>
    <td colspan="2">1:0</td>
    <td colspan="17"></td>
</tr>
<tr>
    <td><i>Reserved</i></td>
    <td colspan="3">000</td>
    <td colspan="11">&mdash;</td>
    <td colspan="2">00</td>
    <td colspan="9">&mdash;</td>
    <td>c.addi4spn</td>
</tr>
<tr>
    <td>c.fld</td>
    <td colspan="3">001</td>
    <td colspan="3">uimm[5:3]</td>
    <td colspan="3">rs1&prime;</td>
    <td colspan="2">uimm[7:6]</td>
    <td colspan="3">rd&prime;</td>
    <td colspan="2">00</td>
    <td colspan="9">FIXME: finish</td>
    <td>c.fld</td>
</tr>
<tr>
    <td>c.mv,c.conv</td>
    <td colspan="3">100</td>
    <td>0</td>
    <td colspan="5">rd[4:0]</td>
    <td colspan="5">rs1[4:0]</td>
    <td colspan="2">10</td>
    <td colspan="2">rs1[6:5]</td>
    <td colspan="2">rd[6:5]</td>
    <td colspan="5">Vector Length /<br/>Element Types</td>
    <td>c.mv,c.jr</td>
</tr>
</table>

## Vector Length Encodings
### c&#46;mv/c.conv Vector Length / Element Types

TODO: Reorder to make decoding faster and select which length-types combinations are most used

<table>
<tr><th>Encoding</th>
<th>Length</th>
<th>rs1 Type</th>
<th>rd Type</th>
<th>Sign Extension</th></tr>
<tr>
    <td>00000</td>
    <td>VL * 1</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td rowspan="2">00001</td>
    <td>VL * 2</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 1</td>
    <td>u16</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td>00010</td>
    <td>VL * 3</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td rowspan="3">00011</td>
    <td>VL * 4</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 2</td>
    <td>u16</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 1</td>
    <td>u32</td>
    <td>u32</td>
    <td>N/A</td>
</tr>
<tr>
    <td rowspan="2">00100</td>
    <td>VL * 6</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 3</td>
    <td>u16</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td rowspan="4">00101</td>
    <td>VL * 8</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 4</td>
    <td>u16</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 2</td>
    <td>u32</td>
    <td>u32</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 1</td>
    <td>u64</td>
    <td>u64</td>
    <td>N/A</td>
</tr>
<tr>
    <td rowspan="3">00110</td>
    <td>VL * 3</td>
    <td>u32</td>
    <td>u32</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 6</td>
    <td>u16</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 12</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td rowspan="4">00111</td>
    <td>VL * 16</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 8</td>
    <td>u16</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 4</td>
    <td>u32</td>
    <td>u32</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 2</td>
    <td>u64</td>
    <td>u64</td>
    <td>N/A</td>
</tr>
<tr>
    <td rowspan="4">01000</td>
    <td>VL * 24</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 12</td>
    <td>u16</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 6</td>
    <td>u32</td>
    <td>u32</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 3</td>
    <td>u64</td>
    <td>u64</td>
    <td>N/A</td>
</tr>
<tr>
    <td rowspan="4">01001</td>
    <td>VL * 32</td>
    <td>u8</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 16</td>
    <td>u16</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 8</td>
    <td>u32</td>
    <td>u32</td>
    <td>N/A</td>
</tr>
<tr>
    <td>VL * 4</td>
    <td>u64</td>
    <td>u64</td>
    <td>N/A</td>
</tr>
<tr>
    <td>01010</td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
</tr>
<tr>
    <td>01011</td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
</tr>
<tr>
    <td>01100</td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
</tr>
<tr>
    <td>01101</td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
    <td><i>Reserved</i></td>
</tr>
<tr>
    <td>01110</td>
    <td>VL</td>
    <td>u16</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>01111</td>
    <td>VL</td>
    <td>u32</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>10000</td>
    <td>VL</td>
    <td>u32</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td>10001</td>
    <td>VL</td>
    <td>u64</td>
    <td>u8</td>
    <td>N/A</td>
</tr>
<tr>
    <td>10010</td>
    <td>VL</td>
    <td>u64</td>
    <td>u16</td>
    <td>N/A</td>
</tr>
<tr>
    <td>10011</td>
    <td>VL</td>
    <td>u64</td>
    <td>u32</td>
    <td>N/A</td>
</tr>
<tr>
    <td>10100</td>
    <td>VL</td>
    <td>u8</td>
    <td>u16</td>
    <td>Zero Extension</td>
</tr>
<tr>
    <td>10101</td>
    <td>VL</td>
    <td>i8</td>
    <td>i16</td>
    <td>Sign Extension</td>
</tr>
<tr>
    <td>10110</td>
    <td>VL</td>
    <td>u8</td>
    <td>u32</td>
    <td>Zero Extension</td>
</tr>
<tr>
    <td>10111</td>
    <td>VL</td>
    <td>i8</td>
    <td>i32</td>
    <td>Sign Extension</td>
</tr>
<tr>
    <td>11000</td>
    <td>VL</td>
    <td>u16</td>
    <td>u32</td>
    <td>Zero Extension</td>
</tr>
<tr>
    <td>11001</td>
    <td>VL</td>
    <td>i16</td>
    <td>i32</td>
    <td>Sign Extension</td>
</tr>
<tr>
    <td>11010</td>
    <td>VL</td>
    <td>u8</td>
    <td>u64</td>
    <td>Zero Extension</td>
</tr>
<tr>
    <td>11011</td>
    <td>VL</td>
    <td>i8</td>
    <td>i64</td>
    <td>Sign Extension</td>
</tr>
<tr>
    <td>11100</td>
    <td>VL</td>
    <td>u16</td>
    <td>u64</td>
    <td>Zero Extension</td>
</tr>
<tr>
    <td>11101</td>
    <td>VL</td>
    <td>i16</td>
    <td>i64</td>
    <td>Sign Extension</td>
</tr>
<tr>
    <td>11110</td>
    <td>VL</td>
    <td>u32</td>
    <td>u64</td>
    <td>Zero Extension</td>
</tr>
<tr>
    <td>11111</td>
    <td>VL</td>
    <td>i32</td>
    <td>i64</td>
    <td>Sign Extension</td>
</tr>
</table>
