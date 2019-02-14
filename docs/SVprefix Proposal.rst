SimpleV Prefix (SVprefix) Proposal v0.2
=======================================

This proposal is designed to be able to operate without SVcsr, but not to
require the absence of SVcsr.

Conventions
===========

Conventions used in this document:
- Bits are numbered starting from 0 at the LSB, so bit 3 is 1 in the integer 8.
- Bit ranges are inclusive on both ends, so 5:3 means bits 5, 4, and 3.

Operations work on variable-length vectors of sub-vectors, where each sub-vector
has a length *svlen*, and an element type *etype*. When the vectors are stored
in registers, all elements are packed so that there is no padding in-between
elements of the same vector. The number of bytes in a sub-vector, *svsz*, is the
product of *svlen* and the element size in bytes.

Half-Precision Floating Point (FP16)
====================================
If the F extension is supported, SVprefix adds support for FP16 in the
base FP instructions by using 10 (H) in the floating-point format field *fmt*
and using 001 (H) in the floating-point load/store *width* field.

Compressed Instructions
=======================
This proposal doesn't include any prefixed RVC instructions, instead, it will
include 32-bit instructions that are compressed forms of SVprefix 48-bit
instructions, in the same manner that RVC instructions are compressed forms of
RVI instructions. The compressed instructions will be defined later by
considering which 48-bit instructions are the most common.

48-bit Prefixed Instructions
============================
All 48-bit prefixed instructions contain a 32-bit "base" instruction as the
last 4 bytes. Since all 32-bit instructions have bits 1:0 set to 11, those bits
are reused for additional encoding space in the 48-bit instructions.

48-bit Instruction Encodings
============================

In the following table, *Reserved* entries must be zero.

+---------------+-------------+-------+----------+----------+--------+----------+--------+--------+------------+------------+-----+------------+------------+------+------------+--------+
| Encoding      | 47:43       | 42:41 | 40:36    | 35:31    | 30:28  | 27:23    | 22:18  | 17     | 16         | 15         | 14  | 13         | 12         | 11:7 | 6          | 5:0    |
+---------------+-------------+-------+----------+----------+--------+----------+--------+--------+------------+------------+-----+------------+------------+------+------------+--------+
|               | 31:27       | 26:25 | 24:20    | 19:15    | 14:12  | 11:7     | 6:2    | 1      | 0          |                                                                         |
+---------------+-------------+-------+----------+----------+--------+----------+--------+--------+------------+--------------------------------------------+------+------------+--------+
| P48LD-type    | imm[11:0]                      | rs1[4:0] | width  | rd[4:0]  | opcode | rd[5]  | rs1[5]     | lsk                                        | vtp5 | *Reserved* | 011111 |
+---------------+---------------------+----------+----------+--------+----------+--------+--------+------------+------------+-------------------------------+------+------------+--------+
| P48ST-type    | imm[11:5]           | rs2[4:0] | rs1[4:0] | width  | imm[4:0] | opcode | lsk[3] | rs1[5]     | rs2[5]     | lsk[2:0]                      | vtp5 | *Reserved* | 011111 |
+---------------+---------------------+----------+----------+--------+----------+--------+--------+------------+------------+-----+------------+------------+------+------------+--------+
| P48R-type     | imm[11:5]           | rs2[4:0] | rs1[4:0] | funct3 | rd[4:0]  | opcode | rd[5]  | rs1[5]     | rs2[5]     | vs2 | vs1        | vitp6             | *Reserved* | 011111 |
+---------------+---------------------+----------+----------+--------+----------+--------+--------+------------+------------+-----+------------+-------------------+------------+--------+
| P48I-type     | imm[11:0]                      | rs1[4:0] | funct3 | rd[4:0]  | opcode | rd[5]  | rs1[5]     | *Reserved* | vd  | vs1        | vitp6             | *Reserved* | 011111 |
+---------------+--------------------------------+----------+--------+----------+--------+--------+------------+------------+-----+------------+-------------------+------------+--------+
| P48U-type     | imm[31:12]                                         | rd[4:0]  | opcode | rd[5]  | *Reserved* | *Reserved* | vd  | *Reserved* | vitp6             | *Reserved* | 011111 |
+---------------+-------------+-------+----------+----------+--------+----------+--------+--------+------------+------------+-----+------------+------------+------+------------+--------+
| P48FR-type    | funct7[6:2] | fmt   | rs2[4:0] | rs1[4:0] | rm     | rd[4:0]  | opcode | rd[5]  | rs1[5]     | rs2[5]     | vs2 | vs1        | *Reserved* | vtp5 | *Reserved* | 011111 |
+---------------+-------------+-------+----------+----------+--------+----------+--------+--------+------------+------------+-----+------------+------------+------+------------+--------+
| P48FI-type    | funct7[6:2] | fmt   | funct5   | rs1[4:0] | rm     | rd[4:0]  | opcode | rd[5]  | rs1[5]     | *Reserved* | vd  | vs1        | *Reserved* | vtp5 | *Reserved* | 011111 |
+---------------+-------------+-------+----------+----------+--------+----------+--------+--------+------------+------------+-----+------------+------------+------+------------+--------+
| P48FR4-type   | rs3[4:0]    | fmt   | rs2[4:0] | rs1[4:0] | rm     | rd[4:0]  | opcode | rd[5]  | rs1[5]     | rs2[5]     | vs2 | rs3[5]     | vs3[#fr4]_ | vtp5 | *Reserved* | 011111 |
+---------------+-------------+-------+----------+----------+--------+----------+--------+--------+------------+------------+-----+------------+------------+------+------------+--------+

.. [#fr4] Only vs2 and vs3 are included in the P48FR4-type encoding because
          there is not enough space for vs1 as well, and because it is more
          useful to have a scalar argument for each of the multiplication and
          addition portions of fmadd than to have two scalars on the
          multiplication portion.

vs#/vd Fields' Encoding
=======================

+--------+----------+----------------------------------------------------------+
| vs#/vd | Mnemonic | Meaning                                                  |
+========+==========+==========================================================+
| 0      | S        | the rs#/rd field specifies a scalar (single sub-vector); |
|        |          | the rs#/rd field is zero-extended to get the actual      |
|        |          | 7-bit register number                                    |
+--------+----------+----------------------------------------------------------+
| 1      | V        | the rs#/rd field specifies a vector; the rs#/rd field is |
|        |          | decoded using the `Vector Register Number Encoding`_ to  |
|        |          | get the actual 7-bit register number                     |
+--------+----------+----------------------------------------------------------+

If a vs#/vd field is not present, it is as if it was present with a value that
is the bitwise-or of all present vs#/vd fields.

Vector Register Number Encoding
===============================

When vs#/vd is 1, the actual 7-bit register number is derived from the
corresponding 6-bit rs#/rd field:

+---------------------------------+
| Actual 7-bit register number    |
+===========+=============+=======+
| Bit 6     | Bits 5:1    | Bit 0 |
+-----------+-------------+-------+
| rs#/rd[0] | rs#/rd[5:1] | 0     |
+-----------+-------------+-------+

Load/Store Kind (lsk) Field Encoding
====================================

+-------+----------+------------+-----------------------------------------------------------------+
| lsk   | Mnemonic | Stride     | Meaning                                                         |
+=======+==========+============+=================================================================+
| 0000  | C        |            | Contiguous                                                      |
+-------+----------+------------+-----------------------------------------------------------------+
| 0001  | X        |            | Gather/Scatter                                                  |
+-------+----------+------------+-----------------------------------------------------------------+
| 0010  | SI       | *svsz* * 2 | Strided with an immediate stride of 2 times the sub-vector size |
+-------+----------+------------+-----------------------------------------------------------------+
| 0011  | SI       | *svsz* * 3 | Strided with an immediate stride of 3 times the sub-vector size |
+-------+----------+------------+-----------------------------------------------------------------+
| 0100  | SI       | *svsz* * 4 | Strided with an immediate stride of 4 times the sub-vector size |
+-------+----------+------------+-----------------------------------------------------------------+
| 0101  | SI       | *svsz* * 5 | Strided with an immediate stride of 5 times the sub-vector size |
+-------+----------+------------+-----------------------------------------------------------------+
| 0110  | SI       | *svsz* * 6 | Strided with an immediate stride of 6 times the sub-vector size |
+-------+----------+------------+-----------------------------------------------------------------+
| 0111  | SI       | *svsz* * 7 | Strided with an immediate stride of 7 times the sub-vector size |
+-------+----------+------------+-----------------------------------------------------------------+
| 1000  | S        | x8 (s0)    | Strided with a stride in bytes specified by a register          |
+-------+----------+------------+                                                                 |
| 1001  | S        | x9 (s1)    |                                                                 |
+-------+----------+------------+                                                                 |
| 1010  | S        | x10 (a0)   |                                                                 |
+-------+----------+------------+                                                                 |
| 1011  | S        | x11 (a1)   |                                                                 |
+-------+----------+------------+                                                                 |
| 1100  | S        | x12 (a2)   |                                                                 |
+-------+----------+------------+                                                                 |
| 1101  | S        | x13 (a3)   |                                                                 |
+-------+----------+------------+                                                                 |
| 1110  | S        | x14 (a4)   |                                                                 |
+-------+----------+------------+                                                                 |
| 1111  | S        | x15 (a5)   |                                                                 |
+-------+----------+------------+-----------------------------------------------------------------+

Sub-Vector Length (svlen) Field Encoding
=======================================================

+----------------+-------+
| svlen Encoding | Value |
+================+=======+
| 00             | 4     |
+----------------+-------+
| 01             | 1     |
+----------------+-------+
| 10             | 2     |
+----------------+-------+
| 11             | 3     |
+----------------+-------+

Predication (pred) Field Encoding
=================================

+---------------+------------+--------------------+------------------------------------------------------------------------------+
| pred Encoding | Mnemonic   | Predicate Register | Meaning                                                                      |
+===============+============+====================+==============================================================================+
| 000           | *None*     | *None*             | The instruction is unpredicated                                              |
+---------------+------------+--------------------+------------------------------------------------------------------------------+
| 001           | *Reserved* | *Reserved*         |                                                                              |
+---------------+------------+--------------------+------------------------------------------------------------------------------+
| 010           | p0         | x9 (s1)            | Each sub-vector operation is executed when the corresponding bit in x9 is 0  |
+---------------+------------+                    +------------------------------------------------------------------------------+
| 011           | p1         |                    | Each sub-vector operation is executed when the corresponding bit in x9 is 1  |
+---------------+------------+--------------------+------------------------------------------------------------------------------+
| 100           | p0         | x10 (a0)           | Each sub-vector operation is executed when the corresponding bit in x10 is 0 |
+---------------+------------+                    +------------------------------------------------------------------------------+
| 101           | p1         |                    | Each sub-vector operation is executed when the corresponding bit in x10 is 1 |
+---------------+------------+--------------------+------------------------------------------------------------------------------+
| 110           | p0         | x11 (a1)           | Each sub-vector operation is executed when the corresponding bit in x11 is 0 |
+---------------+------------+                    +------------------------------------------------------------------------------+
| 111           | p1         |                    | Each sub-vector operation is executed when the corresponding bit in x11 is 1 |
+---------------+------------+--------------------+------------------------------------------------------------------------------+

Integer Element Type (itype) Field Encoding
===========================================

+------------+-------+--------------+----------------------+------------------------------+---------------------------+
| Signedness | itype | Element Type | Mnemonic in          | Mnemonic in Floating-Point   | Meaning                   |
| [#sgn_def]_|       |              | Integer Instructions | Instructions (such as fmv.x) |                           |
+============+=======+==============+======================+==============================+===========================+
| Unsigned   | 00    | u8           | BU                   | BU                           | Unsigned 8-bit Integer    |
|            +-------+--------------+----------------------+------------------------------+---------------------------+
|            | 01    | u16          | HU                   | HU                           | Unsigned 16-bit Integer   |
|            +-------+--------------+----------------------+------------------------------+---------------------------+
|            | 10    | u32          | WU                   | WU                           | Unsigned 32-bit Integer   |
|            +-------+--------------+----------------------+------------------------------+---------------------------+
|            | 11    | uXLEN        | WU/DU/QU             | WU/LU/TU                     | Unsigned XLEN-bit Integer |
+------------+-------+--------------+----------------------+------------------------------+---------------------------+
| Signed     | 00    | i8           | BS                   | BS                           | Signed 8-bit Integer      |
|            +-------+--------------+----------------------+------------------------------+---------------------------+
|            | 01    | i16          | HS                   | HS                           | Signed 16-bit Integer     |
|            +-------+--------------+----------------------+------------------------------+---------------------------+
|            | 10    | i32          | W                    | W                            | Signed 32-bit Integer     |
|            +-------+--------------+----------------------+------------------------------+---------------------------+
|            | 11    | iXLEN        | W/D/Q                | W/L/T                        | Signed XLEN-bit Integer   |
+------------+-------+--------------+----------------------+------------------------------+---------------------------+

.. [#sgn_def] Signedness is defined in `Signedness Decision Procedure`_

Signedness Decision Procedure
=============================

1. If the opcode field is either OP or OP-IMM, then
    1. Signedness is Unsigned.
2. If the opcode field is either OP-32 or OP-IMM-32, then
    1. Signedness is Signed.
3. If Signedness is encoded in a field of the base instruction[#sign_enc]_, then
    1. Signedness uses the encoded value.
4. Otherwise,
    1. Signedness is Unsigned.

.. [#sign_enc] Like in fcvt.d.l[u], but unlike in fmv.x.w, since there is no
               fmv.x.wu

Vector Type and Predication 5-bit (vtp5) Field Encoding
=======================================================

In the following table, X denotes a wildcard that is 0 or 1 and can be a
different value for every occurrence.

+-------+-----------+-----------+
| vtp5  | pred      | svlen     |
+=======+===========+===========+
| 1XXXX | vtp5[4:2] | vtp5[1:0] |
+-------+           |           |
| 01XXX |           |           |
+-------+           |           |
| 000XX |           |           |
+-------+-----------+-----------+
| 001XX | *Reserved*            |
+-------+-----------------------+

Vector Integer Type and Predication 6-bit (vitp6) Field Encoding
================================================================

In the following table, X denotes a wildcard that is 0 or 1 and can be a
different value for every occurrence.

+--------+------------+---------+------------+------------+
| vitp6  | itype      | pred[2] | pred[0:1]  | svlen      |
+========+============+=========+============+============+
| XX1XXX | vitp6[5:4] | 0       | vitp6[3:2] | vitp6[1:0] |
+--------+            |         |            |            |
| XX00XX |            |         |            |            |
+--------+------------+---------+------------+------------+
| XX01XX | *Reserved*                                     |
+--------+------------------------------------------------+

48-bit Instruction Encoding Decision Procedure
==============================================

In the following decision procedure, *Reserved* means that there is not yet a
defined 48-bit instruction encoding for the base instruction.

1. If the base instruction is a load instruction, then
    a. If the base instruction is an I-type instruction, then
        1. The encoding is P48LD-type.
    b. Otherwise
        1. The encoding is *Reserved*.
2. If the base instruction is a store instruction, then
    a. If the base instruction is an S-type instruction, then
        1. The encoding is P48ST-type.
    b. Otherwise
        1. The encoding is *Reserved*.
3. If the base instruction is a SYSTEM instruction, then
    a. The encoding is *Reserved*.
4. If the base instruction is an integer instruction, then
    a. If the base instruction is an R-type instruction, then
        1. The encoding is P48R-type.
    b. If the base instruction is an I-type instruction, then
        1. The encoding is P48I-type.
    c. If the base instruction is an S-type instruction, then
        1. The encoding is *Reserved*.
    d. If the base instruction is an B-type instruction, then
        1. The encoding is *Reserved*.
    e. If the base instruction is an U-type instruction, then
        1. The encoding is P48U-type.
    f. If the base instruction is an J-type instruction, then
        1. The encoding is *Reserved*.
    g. Otherwise
        1. The encoding is *Reserved*.
5. If the base instruction is a floating-point instruction, then
    a. If the base instruction is an R-type instruction, then
        1. The encoding is P48FR-type.
    b. If the base instruction is an I-type instruction, then
        1. The encoding is P48FI-type.
    c. If the base instruction is an S-type instruction, then
        1. The encoding is *Reserved*.
    d. If the base instruction is an B-type instruction, then
        1. The encoding is *Reserved*.
    e. If the base instruction is an U-type instruction, then
        1. The encoding is *Reserved*.
    f. If the base instruction is an J-type instruction, then
        1. The encoding is *Reserved*.
    g. If the base instruction is an R4-type instruction, then
        1. The encoding is P48FR4-type.
    h. Otherwise
        1. The encoding is *Reserved*.
6. Otherwise
    a. The encoding is *Reserved*.

CSR Registers
=============

+--------+-----------------+---------------------------------------------------+
| Name   | Legal Values    | Meaning                                           |
+========+=================+===================================================+
| VL     | 0 <= VL <= XLEN | Vector Length. The number of sub-vectors operated |
|        |                 | on by vector instructions.                        |
+--------+-----------------+---------------------------------------------------+
| Vstart | 0 <= VL < XLEN  | The sub-vector index to start execution at.       |
|        |                 | Vector instructions can be extremely slow when    |
|        |                 | Vstart != 0. Successful execution of a vector     |
|        |                 | instruction sets Vstart to 0. Set to the index of |
|        |                 | the failing sub-vector when a vector instruction  |
|        |                 | traps. Used to resume execution of vector         |
|        |                 | instructions after a trap.                        |
+--------+-----------------+---------------------------------------------------+

SetVL
=====

setvl rd, rs1, imm

imm is the amount of space allocated from the register file by the compiler.

Pseudocode:
1. Trap if imm > XLEN.
2. If rs1 is x0, then
    1. Set VL to imm.
3. Else If regs[rs1] > 2 * imm, then
    1. Set VL to XLEN.
4. Else If regs[rs1] > imm, then
    1. Set VL to regs[rs1] / 2 rounded down.
5. Otherwise,
    1. Set VL to regs[rs1].
6. Set regs[rd] to VL.

Additional Instructions
=======================

Add instructions to convert between integer types.

Add instructions to `swizzle`_ elements in sub-vectors. Note that the sub-vector
lengths of the source and destination won't necessarily match.

.. _swizzle: https://www.khronos.org/opengl/wiki/Data_Type_(GLSL)#Swizzling

Add instructions to transpose (2-4)x(2-4) element matrices.

Add instructions to insert or extract a sub-vector from a vector, with the index
allowed to be both immediate and from a register.

Add a register gather instruction.
