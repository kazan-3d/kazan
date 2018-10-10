# Proposal for Element-Grouping in SimpleV on RV64GC

#### Notes

i*N* is used to denote a sign-agnostic *N*-bit integer,
similarly f*N* is a *N*-bit floating-point number. *VL* is used to denote the current vector length.

For the unused elements in an integer register, the used element closest to the MSB is sign-extended on write and the unused elements are ignored on read.
The unused elements in a floating-point register are treated as-if they are set to all ones on write and are ignored on read, matching the existing standard for storing smaller FP values in larger registers.

For grouped modes, *VL* denotes the number of groups, not the number of elements.

### Group Size 1

| Mode       | # Regs | Regs/Group | Elms/Reg | Packed/Non-packed SIMD   |
|------------|--------|------------|----------|--------------------------|
| i8x1x*VL*  | 1x*VL* | 1          | 1        | i64x1x*VL* (Non-packed)  |
| i16x1x*VL* | 1x*VL* | 1          | 1        | i64x1x*VL* (Non-packed)  |
| i32x1x*VL* | 1x*VL* | 1          | 1        | i64x1x*VL* (Non-packed)  |
| i64x1x*VL* | 1x*VL* | 1          | 1        | i64x1x*VL* (Non-packed)  |
| f16x1x*VL* | 1x*VL* | 1          | 1        | f16x1x*VL* (Non-packed)  |
| f32x1x*VL* | 1x*VL* | 1          | 1        | f32x1x*VL* (Non-packed)  |
| f64x1x*VL* | 1x*VL* | 1          | 1        | f64x1x*VL* (Non-packed)  |

### Group Size 2

| Mode       | # Regs | Regs/Group | Elms/Reg | Packed/Non-packed SIMD   |
|------------|--------|------------|----------|--------------------------|
| i8x2x*VL*  | 1x*VL* | 1          | 2        | Not supported            |
| i16x2x*VL* | 1x*VL* | 1          | 2        | Not supported            |
| i32x2x*VL* | 1x*VL* | 1          | 2        | i32x2x*VL* (Packed)      |
| i64x2x*VL* | 2x*VL* | 2          | 1        | i64x2x*VL* (Non-packed)* |
| f16x2x*VL* | 1x*VL* | 1          | 2        | Not supported            |
| f32x2x*VL* | 1x*VL* | 1          | 2        | f32x2x*VL* (Packed)      |
| f64x2x*VL* | 2x*VL* | 2          | 1        | f64x2x*VL* (Non-packed)* |

\* Not supported unless *VL* is changed

### Group Size 3

| Mode       | # Regs | Regs/Group | Elms/Reg | Packed/Non-packed SIMD   |
|------------|--------|------------|----------|--------------------------|
| i8x3x*VL*  | 1x*VL* | 1          | 3        | Not supported            |
| i16x3x*VL* | 1x*VL* | 1          | 3        | Not supported            |
| i32x3x*VL* | 2x*VL* | 2          | 2        | Not supported            |
| i64x3x*VL* | 3x*VL* | 3          | 1        | i64x3x*VL* (Non-packed)* |
| f16x3x*VL* | 1x*VL* | 1          | 3        | Not supported            |
| f32x3x*VL* | 2x*VL* | 2          | 2        | Not supported            |
| f64x3x*VL* | 3x*VL* | 3          | 1        | f64x3x*VL* (Non-packed)* |

\* Not supported unless *VL* is changed

### Group Size 4

| Mode       | # Regs | Regs/Group | Elms/Reg | Packed/Non-packed SIMD   |
|------------|--------|------------|----------|--------------------------|
| i8x4x*VL*  | 1x*VL* | 1          | 4        | Not supported            |
| i16x4x*VL* | 1x*VL* | 1          | 4        | i16x4x*VL* (Packed)      |
| i32x4x*VL* | 2x*VL* | 2          | 2        | i32x4x*VL* (Packed)*     |
| i64x4x*VL* | 4x*VL* | 4          | 1        | i64x4x*VL* (Non-packed)* |
| f16x4x*VL* | 1x*VL* | 1          | 4        | f16x4x*VL* (Packed)      |
| f32x4x*VL* | 2x*VL* | 2          | 2        | f32x4x*VL* (Packed)*     |
| f64x4x*VL* | 4x*VL* | 4          | 1        | f64x4x*VL* (Non-packed)* |

\* Not supported unless *VL* is changed

...

### Group Size 7

| Mode       | # Regs | Regs/Group | Elms/Reg | Packed/Non-packed SIMD   |
|------------|--------|------------|----------|--------------------------|
| i8x7x*VL*  | 1x*VL* | 1          | 7        | Not supported            |
| i16x7x*VL* | 2x*VL* | 2          | 4        | Not supported            |
| i32x7x*VL* | 4x*VL* | 4          | 2        | Not supported            |
| i64x7x*VL* | 7x*VL* | 7          | 1        | i64x7x*VL* (Non-packed)* |
| f16x7x*VL* | 2x*VL* | 2          | 4        | Not supported            |
| f32x7x*VL* | 4x*VL* | 4          | 2        | Not supported            |
| f64x7x*VL* | 7x*VL* | 7          | 1        | f64x7x*VL* (Non-packed)* |

\* Not supported unless *VL* is changed

### Group Size 8

| Mode       | # Regs | Regs/Group | Elms/Reg | Packed/Non-packed SIMD   |
|------------|--------|------------|----------|--------------------------|
| i8x8x*VL*  | 1x*VL* | 1          | 8        | i8x8x*VL*  (Packed)      |
| i16x8x*VL* | 2x*VL* | 2          | 4        | i16x8x*VL* (Packed)*     |
| i32x8x*VL* | 4x*VL* | 4          | 2        | i32x8x*VL* (Packed)*     |
| i64x8x*VL* | 8x*VL* | 8          | 1        | i64x8x*VL* (Non-packed)* |
| f16x8x*VL* | 2x*VL* | 2          | 4        | f16x8x*VL* (Packed)*     |
| f32x8x*VL* | 4x*VL* | 4          | 2        | f32x8x*VL* (Packed)*     |
| f64x8x*VL* | 8x*VL* | 8          | 1        | f64x8x*VL* (Non-packed)* |

\* Not supported unless *VL* is changed
