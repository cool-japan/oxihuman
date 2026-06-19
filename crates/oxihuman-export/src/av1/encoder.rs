// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Top-level AV1 intra-image encoder.
//!
//! Encodes RGBA pixels into a raw OBU byte stream:
//! `[TD OBU][SeqHeader OBU][FrameHeader OBU][TileGroup OBU]`
//!
//! The three colour planes (G, B, R — AV1 MC_IDENTITY ordering) are encoded
//! in that order.  Within each plane, 4×4 blocks are processed in raster
//! (left-to-right, top-to-bottom) order.

use super::block::encode_block;
use super::coeffs::CoeffCdfContext;
use super::color::rgba_to_gbr_planes;
use super::ec::BoolEncoder;
use super::frame_header::{frame_header_obu, FrameHeader};
use super::obu::{wrap_obu, ObuType};
use super::partition::block_grid;
use super::seq_header::{sequence_header_obu, SeqHeader};
use super::tile::tile_group_obu;

/// Encode RGBA pixels as a raw AV1-like OBU byte stream.
///
/// Returns the concatenated bytes of:
/// 1. Temporal Delimiter (TD) OBU (empty payload)
/// 2. Sequence Header OBU
/// 3. Frame Header OBU
/// 4. Tile Group OBU (contains the full entropy-coded coefficient stream)
///
/// # Parameters
/// - `pixels`:     slice of `w * h` RGBA pixels.
/// - `w`, `h`:     image dimensions in pixels.
/// - `base_q_idx`: quantiser index (0 = lossless, >0 = lossy).
pub fn encode_av1_intra(pixels: &[[u8; 4]], w: u32, h: u32, base_q_idx: u8) -> Vec<u8> {
    let width = w as usize;
    let height = h as usize;

    debug_assert_eq!(
        pixels.len(),
        width * height,
        "pixel count must equal w*h"
    );

    // Separate RGBA into GBR planes (AV1 MC_IDENTITY ordering).
    let (plane_g_orig, plane_b_orig, plane_r_orig) =
        rgba_to_gbr_planes(pixels, width, height);

    // Encode each plane into the single arithmetic coder stream.
    // One CoeffCdfContext shared across all planes/blocks — adaptation accumulates
    // frame-wide, maximising compression; the decoder mirrors identically.
    let mut enc = BoolEncoder::new();
    let mut coeff_ctx = CoeffCdfContext::new();

    // Plane G
    let mut plane_g = plane_g_orig;
    for (bx, by) in block_grid(width, height) {
        encode_block(&mut enc, &mut coeff_ctx, &mut plane_g, width, bx, by, width, height, base_q_idx);
    }

    // Plane B
    let mut plane_b = plane_b_orig;
    for (bx, by) in block_grid(width, height) {
        encode_block(&mut enc, &mut coeff_ctx, &mut plane_b, width, bx, by, width, height, base_q_idx);
    }

    // Plane R
    let mut plane_r = plane_r_orig;
    for (bx, by) in block_grid(width, height) {
        encode_block(&mut enc, &mut coeff_ctx, &mut plane_r, width, bx, by, width, height, base_q_idx);
    }

    let ec_bytes = enc.finish();

    // Build OBU stream.
    let td_obu = wrap_obu(ObuType::TemporalDelimiter, &[]);
    let seq_obu = sequence_header_obu(&SeqHeader { width: w, height: h });
    let frm_obu = frame_header_obu(&FrameHeader { base_q_idx });
    let tile_obu = tile_group_obu(&ec_bytes);

    let mut stream = Vec::with_capacity(
        td_obu.len() + seq_obu.len() + frm_obu.len() + tile_obu.len(),
    );
    stream.extend_from_slice(&td_obu);
    stream.extend_from_slice(&seq_obu);
    stream.extend_from_slice(&frm_obu);
    stream.extend_from_slice(&tile_obu);
    stream
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av1::obu::parse_obu_stream;

    fn make_test_pixels(w: u32, h: u32) -> Vec<[u8; 4]> {
        (0..(w * h) as usize)
            .map(|i| {
                let v = (i * 4) as u8;
                [v, v.wrapping_add(50), v.wrapping_add(100), 255]
            })
            .collect()
    }

    #[test]
    fn test_encode_produces_nonempty_bytes() {
        let pixels = make_test_pixels(4, 4);
        let bytes = encode_av1_intra(&pixels, 4, 4, 0);
        assert!(!bytes.is_empty(), "encoded bytes must not be empty");
    }

    #[test]
    fn test_encode_produces_four_obus() {
        let pixels = make_test_pixels(8, 8);
        let bytes = encode_av1_intra(&pixels, 8, 8, 0);
        let obus = parse_obu_stream(&bytes);
        assert_eq!(
            obus.len(),
            4,
            "encoder must produce exactly 4 OBUs (TD, SeqHeader, FrameHeader, TileGroup)"
        );
        assert_eq!(obus[0].0, ObuType::TemporalDelimiter as u8);
        assert_eq!(obus[1].0, ObuType::SequenceHeader as u8);
        assert_eq!(obus[2].0, ObuType::FrameHeader as u8);
        assert_eq!(obus[3].0, ObuType::TileGroup as u8);
    }

    #[test]
    fn test_encode_different_sizes_do_not_panic() {
        for (w, h) in [(1u32, 1), (4, 4), (8, 8), (5, 7), (16, 12)] {
            let pixels: Vec<[u8; 4]> = vec![[128u8, 64, 32, 255]; (w * h) as usize];
            let _ = encode_av1_intra(&pixels, w, h, 0);
        }
    }

    #[test]
    fn test_lossless_and_lossy_encode_both_work() {
        let pixels = make_test_pixels(8, 8);
        let _lossless = encode_av1_intra(&pixels, 8, 8, 0);
        let _lossy = encode_av1_intra(&pixels, 8, 8, 24);
    }
}
