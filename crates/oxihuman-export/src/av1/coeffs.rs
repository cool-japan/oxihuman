// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Entropy coding of 4×4 block coefficients with adaptive arithmetic coding.
//!
//! Each block is coded as:
//! 1. An adaptive **skip** symbol (2-ary, symbol 1 = all zeros).
//! 2. If not skipped, 16 **per-coefficient** values, each encoded as:
//!    - An adaptive **non-zero** symbol (per position class: DC vs AC).
//!    - If non-zero: an adaptive **sign** symbol (per position class) followed
//!      by an adaptive **magnitude class** k (12-ary) plus a k-bit bypass
//!      suffix (or a full exp-Golomb escape for k ≥ K_MAX = 11).
//!
//! All adaptive symbols flow through `encode_symbol_adapt` / `decode_symbol_adapt`
//! in [`BoolEncoder`] / [`BoolDecoder`], with per-frame [`CoeffCdfContext`] shared
//! across all blocks and planes.  The bypass bits remain 50/50.
//! The encoding is lossless: every coefficient round-trips exactly.

use super::cdf_tables::{
    DEFAULT_MAG_CLASS_CDF, DEFAULT_NONZERO_CDF, DEFAULT_SIGN_CDF, DEFAULT_SKIP_CDF,
    K_MAX, MAG_CLASS_CDF_LEN, MAG_CLASS_SYMS, NONZERO_CDF_LEN, SIGN_CDF_LEN, SKIP_CDF_LEN,
};
use super::ec::{BoolDecoder, BoolEncoder};

// ─────────────────────────────────────────────────────────────────────────────
// Per-frame adaptive CDF context
// ─────────────────────────────────────────────────────────────────────────────

const N_POS_CLASSES: usize = 2; // 0 = DC (scan index 0), 1 = AC (indices 1–15)

/// Per-frame adaptive CDF state shared across all blocks and planes.
///
/// Both encoder and decoder construct an identical instance from defaults and
/// apply identical `update_cdf` calls in identical order (code-then-update),
/// guaranteeing lockstep evolution and bit-exact round-trip.
pub struct CoeffCdfContext {
    skip: [u16; SKIP_CDF_LEN],
    nonzero: [[u16; NONZERO_CDF_LEN]; N_POS_CLASSES],
    sign: [[u16; SIGN_CDF_LEN]; N_POS_CLASSES],
    mag_class: [[u16; MAG_CLASS_CDF_LEN]; N_POS_CLASSES],
}

impl Default for CoeffCdfContext {
    fn default() -> Self {
        Self {
            skip: DEFAULT_SKIP_CDF,
            nonzero: [DEFAULT_NONZERO_CDF; N_POS_CLASSES],
            sign: [DEFAULT_SIGN_CDF; N_POS_CLASSES],
            mag_class: [DEFAULT_MAG_CLASS_CDF; N_POS_CLASSES],
        }
    }
}

impl CoeffCdfContext {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline(always)]
    fn pos_class(scan_idx: usize) -> usize {
        (scan_idx != 0) as usize
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public encoder
// ─────────────────────────────────────────────────────────────────────────────

/// Entropy-encode 16 WHT/DCT coefficients into `enc`, adapting `ctx`.
///
/// The coding is lossless: every value is recovered exactly by `decode_coeffs`
/// given the same initial `ctx` state.  Quantisation is applied in `block.rs`
/// before calling this function.
pub fn encode_coeffs(enc: &mut BoolEncoder, ctx: &mut CoeffCdfContext, coeffs: &[i32; 16]) {
    let all_zero = coeffs.iter().all(|&c| c == 0);
    // skip: symbol 1 = skip (all-zero), symbol 0 = data follows.
    enc.encode_symbol_adapt(all_zero as usize, &mut ctx.skip, SKIP_CDF_LEN - 1);
    if all_zero {
        return;
    }
    for (i, &coeff) in coeffs.iter().enumerate() {
        encode_single_coeff(enc, ctx, i, coeff);
    }
}

/// Entropy-decode 16 coefficients from `dec`, adapting `ctx`.
///
/// `ctx` must be initialized identically to the encoder's context at the start
/// of this block (same frame-level accumulated state).
pub fn decode_coeffs(dec: &mut BoolDecoder<'_>, ctx: &mut CoeffCdfContext) -> [i32; 16] {
    let skip = dec.decode_symbol_adapt(&mut ctx.skip, SKIP_CDF_LEN - 1);
    if skip != 0 {
        return [0i32; 16];
    }
    let mut out = [0i32; 16];
    for (i, v) in out.iter_mut().enumerate() {
        *v = decode_single_coeff(dec, ctx, i);
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-coefficient coding (adaptive)
// ─────────────────────────────────────────────────────────────────────────────

fn encode_single_coeff(enc: &mut BoolEncoder, ctx: &mut CoeffCdfContext, scan_idx: usize, coeff: i32) {
    let pc = CoeffCdfContext::pos_class(scan_idx);
    let nz = (coeff != 0) as usize;
    enc.encode_symbol_adapt(nz, &mut ctx.nonzero[pc], NONZERO_CDF_LEN - 1);
    if coeff == 0 {
        return;
    }
    // sign: symbol 0 = positive, symbol 1 = negative.
    let neg = (coeff < 0) as usize;
    enc.encode_symbol_adapt(neg, &mut ctx.sign[pc], SIGN_CDF_LEN - 1);

    // magnitude class: k = floor(log2(mag_minus_1 + 1)), clamped to K_MAX.
    let mag_minus_1 = coeff.unsigned_abs() - 1; // ≥ 0, ≤ 4079
    let k = magnitude_class(mag_minus_1);
    enc.encode_symbol_adapt(k, &mut ctx.mag_class[pc], MAG_CLASS_SYMS);
    if k < K_MAX {
        // Exact: suffix = (mag_minus_1 + 1) − 2^k, written in k bypass bits.
        if k > 0 {
            let suffix = (mag_minus_1 as u64 + 1) - (1u64 << k);
            enc.write_bypass_uint(suffix, k as u32);
        }
    } else {
        // Escape: fall back to the full exp-Golomb coder (handles k ≥ 11).
        encode_exp_golomb(enc, mag_minus_1 as u64);
    }
}

fn decode_single_coeff(dec: &mut BoolDecoder<'_>, ctx: &mut CoeffCdfContext, scan_idx: usize) -> i32 {
    let pc = CoeffCdfContext::pos_class(scan_idx);
    let nz = dec.decode_symbol_adapt(&mut ctx.nonzero[pc], NONZERO_CDF_LEN - 1);
    if nz == 0 {
        return 0;
    }
    let neg = dec.decode_symbol_adapt(&mut ctx.sign[pc], SIGN_CDF_LEN - 1);
    let k = dec.decode_symbol_adapt(&mut ctx.mag_class[pc], MAG_CLASS_SYMS);
    let mag_minus_1: u64 = if k < K_MAX {
        if k == 0 {
            0
        } else {
            let suffix = dec.read_bypass_uint(k as u32);
            (1u64 << k) + suffix - 1
        }
    } else {
        decode_exp_golomb(dec)
    };
    let mag = (mag_minus_1 + 1) as i32;
    if neg != 0 { -mag } else { mag }
}

/// Compute the exp-Golomb magnitude class k = floor(log2(mag_minus_1+1)),
/// clamped to K_MAX (= 11).  For mag_minus_1 = 0, k = 0.
#[inline(always)]
fn magnitude_class(mag_minus_1: u32) -> usize {
    if mag_minus_1 == 0 {
        return 0;
    }
    let k = (u32::BITS - (mag_minus_1 + 1).leading_zeros() - 1) as usize;
    k.min(K_MAX)
}

// ─────────────────────────────────────────────────────────────────────────────
// Exp-Golomb order-0 variable-length integer coder
// ─────────────────────────────────────────────────────────────────────────────
//
// For value `v`:
//   k = floor(log2(v + 1))        (0 for v=0)
//   Write k zero bits (unary prefix).
//   Write one '1' bit (separator).
//   Write k bits of suffix = (v + 1) - 2^k (in binary, MSB first).
//
// Examples:
//   v=0  → k=0  → "1"
//   v=1  → k=1  → "010"
//   v=2  → k=1  → "011"
//   v=3  → k=2  → "00100"
//   v=4  → k=2  → "00101"
//   v=7  → k=3  → "0001000"

fn encode_exp_golomb(enc: &mut BoolEncoder, val: u64) {
    // k = floor(log2(val+1)).  For val=0 this is 0.
    let k = if val == 0 {
        0u32
    } else {
        // val+1 >= 2, so leading_zeros is well-defined and ≤ 63.
        63 - (val + 1).leading_zeros()
    };

    // Write k zero bits.
    for _ in 0..k {
        enc.encode_bit(false);
    }
    // Write the separator '1'.
    enc.encode_bit(true);
    // Write k bits of suffix.
    if k > 0 {
        let suffix = (val + 1) - (1u64 << k);
        enc.write_bypass_uint(suffix, k);
    }
}

fn decode_exp_golomb(dec: &mut BoolDecoder<'_>) -> u64 {
    // Count leading zeros (k).
    let mut k = 0u32;
    loop {
        let bit = dec.decode_bit();
        if bit {
            break; // found the '1' separator
        }
        k += 1;
        // Safety guard: no valid coefficient magnitude should require > 40 bits.
        if k > 40 {
            break;
        }
    }
    if k == 0 {
        return 0;
    }
    // Read k-bit suffix.
    let suffix = dec.read_bypass_uint(k);
    // Reconstruct: v = 2^k - 1 + suffix  (= (1<<k) + suffix - 1)
    (1u64 << k) + suffix - 1
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av1::ec::{BoolDecoder, BoolEncoder};

    fn roundtrip_coeffs(coeffs: &[i32; 16]) -> [i32; 16] {
        let mut enc = BoolEncoder::new();
        let mut ectx = CoeffCdfContext::new();
        encode_coeffs(&mut enc, &mut ectx, coeffs);
        let bytes = enc.finish();
        let mut dec = BoolDecoder::new(&bytes);
        let mut dctx = CoeffCdfContext::new();
        decode_coeffs(&mut dec, &mut dctx)
    }

    #[test]
    fn test_all_zero_round_trip() {
        let coeffs = [0i32; 16];
        let decoded = roundtrip_coeffs(&coeffs);
        assert_eq!(decoded, coeffs, "all-zero block must round-trip exactly");
    }

    #[test]
    fn test_all_ones_round_trip() {
        let coeffs = [1i32; 16];
        let decoded = roundtrip_coeffs(&coeffs);
        assert_eq!(decoded, coeffs);
    }

    #[test]
    fn test_all_negative_ones_round_trip() {
        let coeffs = [-1i32; 16];
        let decoded = roundtrip_coeffs(&coeffs);
        assert_eq!(decoded, coeffs);
    }

    #[test]
    fn test_mixed_round_trip() {
        let coeffs: [i32; 16] = [
            100, -200, 0, 50, -50, 1, -1, 255, -255, 128, -128, 0, 0, 0, 4095, -4095,
        ];
        let decoded = roundtrip_coeffs(&coeffs);
        assert_eq!(decoded, coeffs);
    }

    #[test]
    fn test_large_magnitudes_round_trip() {
        // WHT of an 8-bit image can produce coefficients up to 255*16 = 4080.
        let coeffs: [i32; 16] = [
            4080, -4080, 2048, -2048, 1024, -1024, 512, -512, 256, -256, 127, -127, 63, -63, 31,
            -31,
        ];
        let decoded = roundtrip_coeffs(&coeffs);
        assert_eq!(decoded, coeffs);
    }

    #[test]
    fn test_dc_only_round_trip() {
        let mut coeffs = [0i32; 16];
        coeffs[0] = 1000;
        let decoded = roundtrip_coeffs(&coeffs);
        assert_eq!(decoded, coeffs);
    }

    #[test]
    fn test_skip_flag_skips_all_coeffs() {
        // When all coefficients are zero the skip symbol fires and no coeff
        // data is written.  The decoded block must be all zeros.
        let coeffs = [0i32; 16];
        let mut enc = BoolEncoder::new();
        let mut ectx = CoeffCdfContext::new();
        encode_coeffs(&mut enc, &mut ectx, &coeffs);
        let bytes = enc.finish();

        let mut dec = BoolDecoder::new(&bytes);
        let mut dctx = CoeffCdfContext::new();
        let decoded = decode_coeffs(&mut dec, &mut dctx);
        assert_eq!(decoded, [0i32; 16]);
    }

    #[test]
    fn test_exp_golomb_known_values() {
        // Test the exp-Golomb coder for specific values (escape path).
        for val in [0u64, 1, 2, 3, 4, 7, 8, 15, 100, 1000, 4095] {
            let mut enc = BoolEncoder::new();
            encode_exp_golomb(&mut enc, val);
            let bytes = enc.finish();
            let mut dec = BoolDecoder::new(&bytes);
            let got = decode_exp_golomb(&mut dec);
            assert_eq!(got, val, "exp-Golomb mismatch for val={val}: got={got}");
        }
    }

    #[test]
    fn test_multiple_blocks_in_sequence() {
        // Simulate encoding several blocks in one stream with ONE shared context.
        // This proves cross-block CDF lockstep: the encoder ctx and decoder ctx
        // must evolve identically.
        let blocks: [[i32; 16]; 4] = [
            [0; 16],
            [1, -1, 2, -2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [-50, 50, -50, 50, -50, 50, -50, 50, -50, 50, -50, 50, -50, 50, -50, 50],
        ];

        let mut enc = BoolEncoder::new();
        let mut ectx = CoeffCdfContext::new();
        for block in &blocks {
            encode_coeffs(&mut enc, &mut ectx, block);
        }
        let bytes = enc.finish();

        let mut dec = BoolDecoder::new(&bytes);
        let mut dctx = CoeffCdfContext::new();
        for (i, expected) in blocks.iter().enumerate() {
            let got = decode_coeffs(&mut dec, &mut dctx);
            assert_eq!(got, *expected, "block {i} mismatch");
        }
    }

    #[test]
    fn test_low_entropy_compresses() {
        // Encode 64 identical DC-only blocks.  After adaptation the skip=false
        // + nonzero[DC]=1 + sign=0 + mag_class=0 (k=0, val=0→mag_minus_1=0)
        // path becomes highly predictable and compresses significantly.
        let mut block = [0i32; 16];
        block[0] = 7; // small, repeated DC coefficient

        // Adaptive encoding (with adaptation accumulating over 64 blocks).
        let mut enc_adapt = BoolEncoder::new();
        let mut ctx_adapt = CoeffCdfContext::new();
        for _ in 0..64 {
            encode_coeffs(&mut enc_adapt, &mut ctx_adapt, &block);
        }
        let adaptive_len = enc_adapt.finish().len();

        // Baseline: fresh (unadapted) context per block — no learning.
        let mut enc_flat = BoolEncoder::new();
        for _ in 0..64 {
            let mut ctx_fresh = CoeffCdfContext::new();
            encode_coeffs(&mut enc_flat, &mut ctx_fresh, &block);
        }
        let flat_len = enc_flat.finish().len();

        assert!(
            adaptive_len < flat_len,
            "adaptive ({adaptive_len} bytes) should be smaller than per-block-fresh \
             ({flat_len} bytes) after 64 identical DC blocks"
        );

        // Verify exact round-trip for the adaptive encoding.
        let mut enc3 = BoolEncoder::new();
        let mut ectx = CoeffCdfContext::new();
        for _ in 0..64 {
            encode_coeffs(&mut enc3, &mut ectx, &block);
        }
        let bytes = enc3.finish();
        let mut dec = BoolDecoder::new(&bytes);
        let mut dctx = CoeffCdfContext::new();
        for i in 0..64 {
            let got = decode_coeffs(&mut dec, &mut dctx);
            assert_eq!(got, block, "round-trip mismatch at block {i}");
        }
    }
}
