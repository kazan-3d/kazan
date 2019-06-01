64-bit Uniform Instruction Form Proposal
========================================

This is a proposal to have a 64-bit instruction that all shorter instructions
can be translated to, such that the internal representation used in a CPU can
be this 64-bit format.

The instruction format is designed for ease of decoding, rather than encoding
density.

Register Fields
===============

+---------------------+-----------------+
| 7                   | 6:0             |
+=====================+=================+
| Float (1) / Int (0) | Register Number |
+---------------------+-----------------+
