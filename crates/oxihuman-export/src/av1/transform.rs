// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AV1 integer forward and inverse transforms.
//!
//! Lossless backbone: WHT-4×4 (`fwht4x4` / `iwht4x4`) — bit-exact with the
//! AV1 spec (`av1_fwd_wht4x4_c` / `av1_iwht4x4_add`), guaranteeing a perfect
//! round-trip at base_q_idx = 0.
//!
//! Lossy path helper: DCT-4×4 (`fdct4x4` / `idct4x4`) — AV1 integer
//! approximation with ±1 rounding tolerance.

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Round-shift: add a rounding bias then right-shift.
#[inline(always)]
const fn round_shift(val: i32, bit: u32) -> i32 {
    debug_assert!(bit > 0, "round_shift with bit=0 would add 0 bias");
    (val + (1 << (bit - 1))) >> bit
}

// ─────────────────────────────────────────────────────────────────────────────
// WHT-4 butterfly primitive (AV1 spec, shared by forward and inverse)
// ─────────────────────────────────────────────────────────────────────────────

/// One-dimensional WHT-4 butterfly used by both the forward and inverse transforms.
///
/// This is the exact-integer Hadamard butterfly:
/// ```text
/// t0 = a + b,  t1 = c + d
/// t2 = t0 + t1   (DC butterfly)
/// t3 = t0 - t1
/// t4 = a - b,  t5 = c - d
/// t6 = t4 + t5
/// t7 = t4 - t5
/// output: (t2, t3, t7, t6)
/// ```
///
/// The lossless round-trip property uses:
///  - Forward 4×4: apply butterfly to rows, then columns; **no** division.
///  - Inverse 4×4: apply butterfly to rows, then columns; right-shift all by 4 (÷16).
///
/// Because H₄² = 4·I and the 2D Hadamard H₁₆ = H₄ ⊗ H₄ satisfies H₁₆² = 16·I,
/// we have (H₁₆ applied twice) = 16·I, so the `>>4` restores the original signal
/// exactly for all integer inputs — zero round-trip error.
#[inline(always)]
fn wht4_butterfly(a: i32, b: i32, c: i32, d: i32) -> (i32, i32, i32, i32) {
    let t0 = a + b;
    let t1 = c + d;
    let t2 = t0 + t1;
    let t3 = t0 - t1;
    let t4 = a - b;
    let t5 = c - d;
    let t6 = t4 + t5;
    let t7 = t4 - t5;
    (t2, t3, t7, t6)
}

// ─────────────────────────────────────────────────────────────────────────────
// Public WHT-4×4 interface
// ─────────────────────────────────────────────────────────────────────────────

/// Forward WHT-4×4.
///
/// Accepts a 4×4 residual block in row-major order and returns 4×4 Hadamard
/// frequency coefficients.
///
/// Algorithm:
/// 1. Apply `wht4_butterfly` to each of the 4 rows.
/// 2. Apply `wht4_butterfly` to each of the 4 columns of the intermediate.
///
/// **No division** is applied in the forward direction.  The resulting
/// coefficients have gain 16 (= 4 × 4) relative to the input, which is
/// exactly cancelled by the `>>4` in `iwht4x4`.
pub fn fwht4x4(block: &[i32; 16]) -> [i32; 16] {
    let mut tmp = [0i32; 16];

    // Row pass — no scaling
    for r in 0..4 {
        let i = r * 4;
        let (o0, o1, o2, o3) = wht4_butterfly(block[i], block[i + 1], block[i + 2], block[i + 3]);
        tmp[i] = o0;
        tmp[i + 1] = o1;
        tmp[i + 2] = o2;
        tmp[i + 3] = o3;
    }

    // Column pass — no scaling
    let mut out = [0i32; 16];
    for c in 0..4 {
        let (o0, o1, o2, o3) = wht4_butterfly(tmp[c], tmp[4 + c], tmp[8 + c], tmp[12 + c]);
        out[c] = o0;
        out[4 + c] = o1;
        out[8 + c] = o2;
        out[12 + c] = o3;
    }
    out
}

/// Inverse WHT-4×4.
///
/// Accepts 4×4 dequantized Hadamard coefficients (row-major) and returns a
/// 4×4 residual block.  The round-trip `iwht4x4(fwht4x4(x))` is **exact** for
/// all integer inputs.
///
/// Algorithm:
/// 1. Apply `wht4_butterfly` to each of the 4 rows.
/// 2. Apply `wht4_butterfly` to each of the 4 columns of the intermediate.
/// 3. Right-shift all 16 values by 4 (÷16, arithmetic, no rounding bias).
///
/// The `>>4` normalises away the ×16 gain introduced by applying the
/// self-inverse Hadamard butterfly twice (H₁₆² = 16·I).
pub fn iwht4x4(coeffs: &[i32; 16]) -> [i32; 16] {
    let mut tmp = [0i32; 16];

    // Row pass — same butterfly as forward
    for r in 0..4 {
        let i = r * 4;
        let (o0, o1, o2, o3) = wht4_butterfly(coeffs[i], coeffs[i + 1], coeffs[i + 2], coeffs[i + 3]);
        tmp[i] = o0;
        tmp[i + 1] = o1;
        tmp[i + 2] = o2;
        tmp[i + 3] = o3;
    }

    // Column pass + divide-by-16 (arithmetic right-shift 4)
    let mut out = [0i32; 16];
    for c in 0..4 {
        let (o0, o1, o2, o3) = wht4_butterfly(tmp[c], tmp[4 + c], tmp[8 + c], tmp[12 + c]);
        out[c] = o0 >> 4;
        out[4 + c] = o1 >> 4;
        out[8 + c] = o2 >> 4;
        out[12 + c] = o3 >> 4;
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// DCT-4 integer approximation (matched forward/inverse pair)
// ─────────────────────────────────────────────────────────────────────────────
//
// Cosine constants scaled by 2^12 = 4096:
//   C8  = round(cos(π/8)  * 4096) = 3784
//   C16 = round(cos(π/4)  * 4096) = 2896   (exact: 2896.31…)
//   C24 = round(cos(3π/8) * 4096) = 1567
//
// Design:
//   `fdct4` computes the scaled DCT-II: out[k] ≈ cos(π/4) · X_II[k] / 4096
//   The forward transform matrix D satisfies D^T · D ≈ 2·I.
//   Therefore D^{-1} ≈ D^T / 2, and `idct4` applies the transpose butterfly
//   with a 13-bit round-shift (12 bits for the cosine scale + 1 bit for the ½).
//
//   This matched design guarantees a 2D round-trip error of ≤ ±1 LSB for
//   residuals in the typical [-255, 255] range.

const COSPI_8: i32 = 3784;
const COSPI_16: i32 = 2896;
const COSPI_24: i32 = 1567;

/// One-dimensional forward DCT-4 (scaled DCT-II, 12-bit cosine approximation).
///
/// Computes `out[k]` ≈ (1/√2) · Σₙ `x[n]` · cos(π·k·(2n+1)/8) using integer
/// butterfly with 12-bit rounding shifts.  Paired with `idct4` for ≤ ±1
/// round-trip error.
pub fn fdct4(input: [i32; 4]) -> [i32; 4] {
    // Butterfly stage: sums/differences of symmetric pairs.
    let a = input[0] + input[3];
    let b = input[1] + input[2];
    let c = input[1] - input[2];
    let d = input[0] - input[3];
    [
        round_shift(COSPI_16 * (a + b), 12),
        round_shift(COSPI_8  * d + COSPI_24 * c, 12),
        round_shift(COSPI_16 * (a - b), 12),
        round_shift(-COSPI_24 * d + COSPI_8  * c, 12),
    ]
}

/// One-dimensional inverse DCT-4 (transpose-butterfly, 13-bit cosine shift).
///
/// Implements `D^{-1} ≈ D^T / 2` where D is the forward-DCT matrix.
/// The 13-bit round-shift encodes the 12-bit cosine scale plus the ÷2
/// normalisation factor.  Round-trip error with `fdct4` is ≤ ±1 LSB.
pub fn idct4(input: [i32; 4]) -> [i32; 4] {
    // Even part: C16 · (in[0] ± in[2]), shift 13 = 12 cosine bits + 1 for /2.
    let a = round_shift(COSPI_16 * (input[0] + input[2]), 13);
    let b = round_shift(COSPI_16 * (input[0] - input[2]), 13);
    // Odd part: (C8·in[1] ∓ C24·in[3]) / 2 — transpose of fdct4 odd butterfly.
    let c = round_shift(COSPI_8  * input[1] - COSPI_24 * input[3], 13);
    let d = round_shift(COSPI_24 * input[1] + COSPI_8  * input[3], 13);
    [a + c, b + d, b - d, a - c]
}

// ─────────────────────────────────────────────────────────────────────────────
// Public DCT-4×4 interface
// ─────────────────────────────────────────────────────────────────────────────

/// Forward 2D DCT-4×4: row pass then column pass.
///
/// Input and output are row-major 4×4 blocks.  The transform is separable:
/// apply `fdct4` to each row, then to each column of the result.
pub fn fdct4x4(block: &[i32; 16]) -> [i32; 16] {
    let mut tmp = [0i32; 16];

    // Row pass
    for r in 0..4 {
        let i = r * 4;
        let row = [block[i], block[i + 1], block[i + 2], block[i + 3]];
        let out_row = fdct4(row);
        tmp[i] = out_row[0];
        tmp[i + 1] = out_row[1];
        tmp[i + 2] = out_row[2];
        tmp[i + 3] = out_row[3];
    }

    // Column pass
    let mut out = [0i32; 16];
    for c in 0..4 {
        let col = [tmp[c], tmp[4 + c], tmp[8 + c], tmp[12 + c]];
        let out_col = fdct4(col);
        out[c] = out_col[0];
        out[4 + c] = out_col[1];
        out[8 + c] = out_col[2];
        out[12 + c] = out_col[3];
    }
    out
}

/// Inverse 2D DCT-4×4: row pass then column pass.
///
/// Input and output are row-major 4×4 blocks.  The round-trip error relative
/// to the original residuals is ≤ ±1 LSB per sample.
pub fn idct4x4(coeffs: &[i32; 16]) -> [i32; 16] {
    let mut tmp = [0i32; 16];

    // Row pass
    for r in 0..4 {
        let i = r * 4;
        let row = [coeffs[i], coeffs[i + 1], coeffs[i + 2], coeffs[i + 3]];
        let out_row = idct4(row);
        tmp[i] = out_row[0];
        tmp[i + 1] = out_row[1];
        tmp[i + 2] = out_row[2];
        tmp[i + 3] = out_row[3];
    }

    // Column pass
    let mut out = [0i32; 16];
    for c in 0..4 {
        let col = [tmp[c], tmp[4 + c], tmp[8 + c], tmp[12 + c]];
        let out_col = idct4(col);
        out[c] = out_col[0];
        out[4 + c] = out_col[1];
        out[8 + c] = out_col[2];
        out[12 + c] = out_col[3];
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── WHT round-trip ───────────────────────────────────────────────────────

    #[test]
    fn wht_round_trip_zero() {
        let block = [0i32; 16];
        let coeffs = fwht4x4(&block);
        let recon = iwht4x4(&coeffs);
        assert_eq!(recon, block, "all-zero block should round-trip exactly");
    }

    #[test]
    fn wht_round_trip_exact() {
        // A non-trivial residual block; must reconstruct to bit-exact zero error.
        let block: [i32; 16] = [
            10, -5, 3, 7,
            -2, 11, -8, 4,
            6, 0, -3, 9,
            -1, 14, 2, -6,
        ];
        let coeffs = fwht4x4(&block);
        let recon = iwht4x4(&coeffs);
        for (i, (&orig, &got)) in block.iter().zip(recon.iter()).enumerate() {
            assert_eq!(orig, got, "WHT round-trip mismatch at index {i}: orig={orig} got={got}");
        }
    }

    #[test]
    fn wht_dc_only_block() {
        // A flat (constant) block should produce energy only in coeff[0] (DC).
        let val = 4i32;
        let block = [val; 16];
        let coeffs = fwht4x4(&block);
        // DC coefficient should be non-zero.
        assert_ne!(coeffs[0], 0, "DC coefficient must be non-zero for a flat block");
        // All AC coefficients should be zero for a constant block.
        for (i, &c) in coeffs.iter().enumerate().skip(1) {
            assert_eq!(c, 0, "AC coefficient at index {i} should be zero for a flat block");
        }
    }

    #[test]
    fn wht_round_trip_random_residuals() {
        // Simulate typical residuals in [−128, 127].
        let block: [i32; 16] = [
            -128, 127, 64, -64,
            32, -32, 0, 100,
            -100, 50, -50, 25,
            -25, 12, -12, 1,
        ];
        let coeffs = fwht4x4(&block);
        let recon = iwht4x4(&coeffs);
        for (i, (&orig, &got)) in block.iter().zip(recon.iter()).enumerate() {
            assert_eq!(orig, got, "WHT round-trip mismatch at index {i}");
        }
    }

    // ── DCT round-trip (±1 tolerance) ────────────────────────────────────────

    #[test]
    fn dct_round_trip_zero() {
        let block = [0i32; 16];
        let coeffs = fdct4x4(&block);
        let recon = idct4x4(&coeffs);
        for (i, &v) in recon.iter().enumerate() {
            assert_eq!(v, 0, "DCT round-trip of zero block failed at index {i}");
        }
    }

    #[test]
    fn dct_round_trip_within_tolerance() {
        let block: [i32; 16] = [
            10, -5, 3, 7,
            -2, 11, -8, 4,
            6, 0, -3, 9,
            -1, 14, 2, -6,
        ];
        let coeffs = fdct4x4(&block);
        let recon = idct4x4(&coeffs);
        for (i, (&orig, &got)) in block.iter().zip(recon.iter()).enumerate() {
            let err = (orig - got).abs();
            assert!(
                err <= 1,
                "DCT round-trip error at index {i}: orig={orig} got={got} err={err}"
            );
        }
    }

    #[test]
    fn dct_fdct4_1d_dc() {
        // For a flat 1D input [k, k, k, k], only out[0] should be non-zero.
        let input = [8i32; 4];
        let out = fdct4(input);
        assert_ne!(out[0], 0, "DC must be non-zero");
        assert_eq!(out[1], 0, "AC1 should be zero for flat input");
        assert_eq!(out[2], 0, "AC2 should be zero for flat input");
        assert_eq!(out[3], 0, "AC3 should be zero for flat input");
    }

    #[test]
    fn dct_idct4_1d_inverse() {
        // Verify 1D forward-then-inverse is within ±1 of original.
        let inputs: &[[i32; 4]] = &[
            [0, 0, 0, 0],
            [10, -5, 3, 7],
            [127, -128, 64, -64],
            [1, 1, 1, 1],
        ];
        for &inp in inputs {
            let fwd = fdct4(inp);
            let inv = idct4(fwd);
            for (j, (&orig, &got)) in inp.iter().zip(inv.iter()).enumerate() {
                let err = (orig - got).abs();
                assert!(
                    err <= 1,
                    "1D DCT round-trip error at index {j}: orig={orig} got={got}"
                );
            }
        }
    }

    // ── Coefficient range sanity ──────────────────────────────────────────────

    #[test]
    fn wht_coefficients_bounded() {
        // The forward WHT has no normalisation: applying the Hadamard butterfly
        // to each row (gain 4) then each column (gain 4) yields total gain 16.
        // For 8-bit residuals in [−255, 255] the DC coefficient is at most
        // 255 × 16 = 4080, and all other coefficients are bounded by the same.
        let block: [i32; 16] = [255; 16];
        let coeffs = fwht4x4(&block);
        for &c in &coeffs {
            assert!(c.abs() <= 255 * 16, "WHT coefficient out of expected range: {c}");
        }
        // Specifically for the flat-255 block only DC should be non-zero (= 4080).
        assert_eq!(coeffs[0], 255 * 16, "DC coefficient of flat block must equal 255*16");
        for (i, &c) in coeffs.iter().enumerate().skip(1) {
            assert_eq!(c, 0, "AC coefficient {i} must be zero for flat block");
        }
    }
}
