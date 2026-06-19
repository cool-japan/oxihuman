// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-4×4-block encode and decode operations.
//!
//! The encode/decode cycle for a single block is:
//!
//! **Encode**:
//! 1. Extract the 4×4 pixel block from the plane.
//! 2. Compute DC-prediction from the already-encoded (reconstructed) top and
//!    left neighbours.
//! 3. Subtract prediction from block → residual.
//! 4. Apply forward transform (WHT for lossless, DCT for lossy).
//! 5. Quantise coefficients.
//! 6. Entropy-encode quantised coefficients.
//!
//! **Decode**:
//! 1. Entropy-decode quantised coefficients.
//! 2. Dequantise.
//! 3. Apply inverse transform.
//! 4. Add DC prediction (from already-decoded left/top data in the plane).
//! 5. Clamp and write the reconstructed block back into the plane.
//!
//! Critically, the decoder reads context (top row / left column) from the
//! *decoded* plane, exactly mirroring what the encoder wrote during
//! reconstruction. This guarantees identical prediction in both paths.

use super::coeffs::{decode_coeffs, encode_coeffs, CoeffCdfContext};
use super::ec::{BoolDecoder, BoolEncoder};
use super::partition::{extract_block, place_block};
use super::predict::{predict_4x4, IntraMode};
use super::quant::{ac_q, dc_q, dequantize_coeff, quantize_coeff};
use super::transform::{fdct4x4, fwht4x4, idct4x4, iwht4x4};

// ─────────────────────────────────────────────────────────────────────────────
// Encoder
// ─────────────────────────────────────────────────────────────────────────────

/// Encode a single 4×4 block from a single image plane.
///
/// # Parameters
/// - `enc`:        the running arithmetic encoder.
/// - `plane`:      flat row-major single-channel pixel buffer (`stride × h`).
/// - `stride`:     pixels per row (usually equals `w`).
/// - `bx, by`:     top-left corner of the block in pixels.
/// - `w, h`:       full image dimensions.
/// - `base_q_idx`: quantiser index (0 = lossless).
///
/// After encoding, the reconstructed pixels are written back into `plane` so
/// that subsequent blocks can use them for prediction context.
pub fn encode_block(
    enc: &mut BoolEncoder,
    ctx: &mut CoeffCdfContext,
    plane: &mut [u8],
    stride: usize,
    bx: usize,
    by: usize,
    w: usize,
    h: usize,
    base_q_idx: u8,
) {
    let block = extract_block(plane, stride, bx, by, w, h);

    let (top, left) = context_refs(plane, stride, bx, by, w, h);

    // Compute prediction.
    let pred = predict_4x4(
        IntraMode::DcPred,
        top.as_deref(),
        left.as_deref(),
    );

    // Residual = original − prediction.
    let mut residual = [0i32; 16];
    for i in 0..16 {
        residual[i] = block[i] as i32 - pred[i] as i32;
    }

    // Forward transform.
    let coeffs = if base_q_idx == 0 {
        fwht4x4(&residual)
    } else {
        fdct4x4(&residual)
    };

    // Quantise.
    let q_dc = dc_q(base_q_idx);
    let q_ac = ac_q(base_q_idx);
    let mut qcoeffs = [0i32; 16];
    qcoeffs[0] = quantize_coeff(coeffs[0], q_dc);
    for i in 1..16 {
        qcoeffs[i] = quantize_coeff(coeffs[i], q_ac);
    }

    // Entropy encode.
    encode_coeffs(enc, ctx, &qcoeffs);

    // Reconstruct into plane for subsequent prediction contexts.
    let recon = reconstruct_block(&qcoeffs, &pred, base_q_idx);
    place_block(plane, stride, bx, by, w, h, &recon);
}

// ─────────────────────────────────────────────────────────────────────────────
// Decoder
// ─────────────────────────────────────────────────────────────────────────────

/// Decode a single 4×4 block and write it into the image plane.
///
/// The plane must already contain the decoded pixels for all blocks above and
/// to the left (in raster order) so that the prediction context is correct.
pub fn decode_block(
    dec: &mut BoolDecoder<'_>,
    ctx: &mut CoeffCdfContext,
    plane: &mut [u8],
    stride: usize,
    bx: usize,
    by: usize,
    w: usize,
    h: usize,
    base_q_idx: u8,
) {
    let (top, left) = context_refs(plane, stride, bx, by, w, h);

    // Entropy decode quantised coefficients.
    let qcoeffs = decode_coeffs(dec, ctx);

    // Prediction + dequant + inverse transform.
    let pred = predict_4x4(
        IntraMode::DcPred,
        top.as_deref(),
        left.as_deref(),
    );

    let recon = reconstruct_block(&qcoeffs, &pred, base_q_idx);
    place_block(plane, stride, bx, by, w, h, &recon);
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Reconstruct a 4×4 pixel block from quantised coefficients and prediction.
///
/// Steps: dequantise → inverse transform → add prediction → clamp to [0, 255].
fn reconstruct_block(qcoeffs: &[i32; 16], pred: &[u8; 16], base_q_idx: u8) -> [u8; 16] {
    let q_dc = dc_q(base_q_idx);
    let q_ac = ac_q(base_q_idx);

    let mut coeffs = [0i32; 16];
    coeffs[0] = dequantize_coeff(qcoeffs[0], q_dc);
    for i in 1..16 {
        coeffs[i] = dequantize_coeff(qcoeffs[i], q_ac);
    }

    let residual = if base_q_idx == 0 {
        iwht4x4(&coeffs)
    } else {
        idct4x4(&coeffs)
    };

    let mut block = [0u8; 16];
    for i in 0..16 {
        block[i] = (pred[i] as i32 + residual[i]).clamp(0, 255) as u8;
    }
    block
}

/// Collect top-row and left-column context pixels from the plane.
///
/// Returns `(Option<[u8;4]>, Option<[u8;4]>)` for the top and left neighbours.
/// `None` when the block is at the image boundary.
fn context_refs(
    plane: &[u8],
    stride: usize,
    bx: usize,
    by: usize,
    w: usize,
    h: usize,
) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
    let top: Option<Vec<u8>> = if by > 0 {
        Some(
            (0..4)
                .map(|c| {
                    let px = bx + c;
                    if px < w {
                        plane[(by - 1) * stride + px]
                    } else {
                        0
                    }
                })
                .collect(),
        )
    } else {
        None
    };

    let left: Option<Vec<u8>> = if bx > 0 {
        Some(
            (0..4)
                .map(|r| {
                    let py = by + r;
                    if py < h {
                        plane[py * stride + (bx - 1)]
                    } else {
                        0
                    }
                })
                .collect(),
        )
    } else {
        None
    };

    (top, left)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av1::ec::{BoolDecoder, BoolEncoder};

    fn encode_decode_plane_block(plane: &[u8], w: usize, h: usize, base_q_idx: u8) -> Vec<u8> {
        let mut plane_enc = plane.to_vec();
        let mut enc = BoolEncoder::new();
        let mut ectx = CoeffCdfContext::new();
        encode_block(&mut enc, &mut ectx, &mut plane_enc, w, 0, 0, w, h, base_q_idx);
        let bytes = enc.finish();

        let mut plane_dec = vec![0u8; w * h];
        let mut dec = BoolDecoder::new(&bytes);
        let mut dctx = CoeffCdfContext::new();
        decode_block(&mut dec, &mut dctx, &mut plane_dec, w, 0, 0, w, h, base_q_idx);
        plane_dec
    }

    #[test]
    fn test_constant_block_lossless() {
        let plane = vec![100u8; 16];
        let decoded = encode_decode_plane_block(&plane, 4, 4, 0);
        assert_eq!(decoded, plane, "constant block must round-trip exactly");
    }

    #[test]
    fn test_gradient_block_lossless() {
        let plane: Vec<u8> = (0u8..16).collect();
        let decoded = encode_decode_plane_block(&plane, 4, 4, 0);
        assert_eq!(decoded, plane, "gradient block must round-trip exactly");
    }

    #[test]
    fn test_zero_block_lossless() {
        let plane = vec![0u8; 16];
        let decoded = encode_decode_plane_block(&plane, 4, 4, 0);
        assert_eq!(decoded, plane);
    }

    #[test]
    fn test_max_value_block_lossless() {
        let plane = vec![255u8; 16];
        let decoded = encode_decode_plane_block(&plane, 4, 4, 0);
        assert_eq!(decoded, plane);
    }

    #[test]
    fn test_mixed_values_block_lossless() {
        let plane: Vec<u8> = [10, 200, 30, 150, 5, 250, 80, 175, 0, 255, 128, 64, 100, 50, 200, 0]
            .to_vec();
        let decoded = encode_decode_plane_block(&plane, 4, 4, 0);
        assert_eq!(decoded, plane, "mixed values must round-trip exactly in lossless mode");
    }

    #[test]
    fn test_8x8_lossless_round_trip() {
        // 8×8 image = 4 blocks. Encode all blocks in sequence.
        let w = 8usize;
        let h = 8usize;
        let original: Vec<u8> = (0u8..64).map(|i| i.wrapping_mul(3)).collect();

        let mut plane_enc = original.clone();
        let mut enc = BoolEncoder::new();
        let mut ectx = CoeffCdfContext::new();
        for (bx, by) in super::super::partition::block_grid(w, h) {
            encode_block(&mut enc, &mut ectx, &mut plane_enc, w, bx, by, w, h, 0);
        }
        let bytes = enc.finish();

        let mut plane_dec = vec![0u8; w * h];
        let mut dec = BoolDecoder::new(&bytes);
        let mut dctx = CoeffCdfContext::new();
        for (bx, by) in super::super::partition::block_grid(w, h) {
            decode_block(&mut dec, &mut dctx, &mut plane_dec, w, bx, by, w, h, 0);
        }

        for (i, (&orig, &got)) in original.iter().zip(plane_dec.iter()).enumerate() {
            assert_eq!(
                orig, got,
                "8×8 lossless mismatch at pixel {i}: orig={orig} got={got}"
            );
        }
    }
}
