SimpleV Prefix (SVprefix) Proposal v0.2
=======================================

Conventions used in this document:
- Bits are numbered starting from 0 at the LSB, so bit 3 is 1 in the integer 8.
- Bit ranges are inclusive on both ends, so 5:3 means bits 5, 4, and 3.

This proposal is designed to be able to operate without SVcsr, but not to
require the absence of SVcsr.

All operations are packed by default.

Half-precision Floating Point (FP16)
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

48-bit prefixed instructions
============================
All 48-bit prefixed instructions contain a 32-bit "base" instruction as the
last 4 bytes. Since all 32-bit instructions have bits 1:0 set to 11, those bits
are reused for additional encoding space in the 48-bit instructions.

48-bit instruction Encodings
============================

+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| Encoding      | 47:43 | 42 | 41 | 40:36 | 35:31    | 30:28 | 27:23   | 22:18  | 17 | 16 | 15 | 14 | 13 | 12 | 11 | 10 | 9 | 8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
+               +-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
|               | 31:27 | 26 | 25 | 24:20 | 19:15    | 14:12 | 11:7    | 6:2    | 1  | 0  |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| P48LD-type    | imm[11:0]               | rs1[4:0] | width | rd[4:0] | opcode |    |    |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| P48ST-type    |       |    |    |       |          |       |         |        |    |    |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| P48R-type     |       |    |    |       |          |       |         |        |    |    |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| P48I-type     |       |    |    |       |          |       |         |        |    |    |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| P48U-type     |       |    |    |       |          |       |         |        |    |    |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| P48FR-type    |       |    |    |       |          |       |         |        |    |    |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| P48FI-type    |       |    |    |       |          |       |         |        |    |    |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| P48FS-type    |       |    |    |       |          |       |         |        |    |    |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+
| P48FR4-type   |       |    |    |       |          |       |         |        |    |    |    |    |    |    |    |    |   |   |   |   |   |   |   |   |   |   |
+---------------+-------+----+----+-------+----------+-------+---------+--------+----+----+----+----+----+----+----+----+---+---+---+---+---+---+---+---+---+---+

48-bit instruction encoding decision procedure
==============================================

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
