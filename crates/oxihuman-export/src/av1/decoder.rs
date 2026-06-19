// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Top-level AV1-like intra-image decoder.
//!
//! Decodes a raw OBU byte stream (as produced by `encoder::encode_av1_intra`)
//! back to RGBA pixels.  The decoder is the exact inverse of the encoder:
//!
//! 1. Parse OBU stream → Sequence Header, Frame Header, Tile Group.
//! 2. Extract `width`, `height`, `base_q_idx` from the headers.
//! 3. Decode entropy-coded coefficients from the tile-group payload.
//! 4. For each plane (G, B, R), decode 4×4 blocks in raster order using
//!    `block::decode_block`, which reads prediction context from the
//!    already-decoded portion of the plane.
//! 5. Recombine GBR planes to RGBA with alpha=255.

use super::block::decode_block;
use super::coeffs::CoeffCdfContext;
use super::color::gbr_planes_to_rgba;
use super::ec::BoolDecoder;
use super::frame_header::decode_frame_header;
use super::obu::parse_obu_stream;
use super::partition::block_grid;
use super::seq_header::decode_sequence_header;
use super::tile::parse_tile_group;

/// Decode an AV1-like intra OBU stream.
///
/// Returns `(pixels, width, height)` on success, where `pixels` is a
/// `Vec<[u8; 4]>` of RGBA values with alpha = 255.
///
/// # Errors
/// Returns a descriptive `String` if the stream is missing required OBUs,
/// the headers are malformed, or the tile data is missing.
pub fn decode_av1_intra(data: &[u8]) -> Result<(Vec<[u8; 4]>, u32, u32), String> {
    let obus = parse_obu_stream(data);

    let mut seq_header_payload: Option<Vec<u8>> = None;
    let mut frame_header_payload: Option<Vec<u8>> = None;
    let mut tile_group_payload: Option<Vec<u8>> = None;

    for (obu_type, payload) in obus {
        match obu_type {
            1 => seq_header_payload = Some(payload),   // SequenceHeader
            3 => frame_header_payload = Some(payload), // FrameHeader
            4 => tile_group_payload = Some(payload),   // TileGroup
            _ => {} // Ignore TD and other OBU types
        }
    }

    let seq_payload = seq_header_payload.ok_or_else(|| "Missing SequenceHeader OBU".to_string())?;
    let frm_payload =
        frame_header_payload.ok_or_else(|| "Missing FrameHeader OBU".to_string())?;
    let tg_payload =
        tile_group_payload.ok_or_else(|| "Missing TileGroup OBU".to_string())?;

    let seq = decode_sequence_header(&seq_payload)
        .ok_or_else(|| "Malformed SequenceHeader payload".to_string())?;

    let frm = decode_frame_header(&frm_payload)
        .ok_or_else(|| "Malformed FrameHeader payload".to_string())?;

    let ec_bytes = parse_tile_group(&tg_payload)
        .ok_or_else(|| "Malformed TileGroup payload".to_string())?;

    let width = seq.width as usize;
    let height = seq.height as usize;
    let base_q_idx = frm.base_q_idx;

    if width == 0 || height == 0 {
        return Err(format!(
            "Invalid image dimensions: {}×{}",
            width, height
        ));
    }

    let mut plane_g = vec![0u8; width * height];
    let mut plane_b = vec![0u8; width * height];
    let mut plane_r = vec![0u8; width * height];

    let mut dec = BoolDecoder::new(ec_bytes);
    // One CoeffCdfContext shared across all planes/blocks, identical initial
    // state to the encoder's context — guaranteed lockstep by symmetric updates.
    let mut coeff_ctx = CoeffCdfContext::new();

    // Decode plane G.
    for (bx, by) in block_grid(width, height) {
        decode_block(&mut dec, &mut coeff_ctx, &mut plane_g, width, bx, by, width, height, base_q_idx);
    }

    // Decode plane B.
    for (bx, by) in block_grid(width, height) {
        decode_block(&mut dec, &mut coeff_ctx, &mut plane_b, width, bx, by, width, height, base_q_idx);
    }

    // Decode plane R.
    for (bx, by) in block_grid(width, height) {
        decode_block(&mut dec, &mut coeff_ctx, &mut plane_r, width, bx, by, width, height, base_q_idx);
    }

    let pixels = gbr_planes_to_rgba(&plane_g, &plane_b, &plane_r, width, height);
    Ok((pixels, seq.width, seq.height))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av1::encoder::encode_av1_intra;

    fn make_pixels(w: u32, h: u32) -> Vec<[u8; 4]> {
        (0..(w * h) as usize)
            .map(|i| {
                let v = (i * 4) as u8;
                [v, v.wrapping_add(50), v.wrapping_add(100), 255]
            })
            .collect()
    }

    #[test]
    fn test_lossless_round_trip_4x4() {
        let w = 4u32;
        let h = 4u32;
        let pixels = make_pixels(w, h);
        let encoded = encode_av1_intra(&pixels, w, h, 0);
        let (decoded, dw, dh) = decode_av1_intra(&encoded).expect("decode must succeed");

        assert_eq!(dw, w);
        assert_eq!(dh, h);
        assert_eq!(decoded.len(), pixels.len());
        for (i, (&orig, &dec)) in pixels.iter().zip(decoded.iter()).enumerate() {
            assert_eq!(orig[0], dec[0], "R mismatch at pixel {i}");
            assert_eq!(orig[1], dec[1], "G mismatch at pixel {i}");
            assert_eq!(orig[2], dec[2], "B mismatch at pixel {i}");
        }
    }

    #[test]
    fn test_lossless_round_trip_8x8() {
        let w = 8u32;
        let h = 8u32;
        let pixels = make_pixels(w, h);
        let encoded = encode_av1_intra(&pixels, w, h, 0);
        let (decoded, dw, dh) = decode_av1_intra(&encoded).expect("decode must succeed");

        assert_eq!(dw, w);
        assert_eq!(dh, h);
        for (i, (&orig, &dec)) in pixels.iter().zip(decoded.iter()).enumerate() {
            assert_eq!(orig[0], dec[0], "R mismatch at pixel {i}");
            assert_eq!(orig[1], dec[1], "G mismatch at pixel {i}");
            assert_eq!(orig[2], dec[2], "B mismatch at pixel {i}");
        }
    }

    #[test]
    fn test_lossless_round_trip_non_multiple_dims() {
        // 5×7 image — block grid will have partial boundary blocks.
        let w = 5u32;
        let h = 7u32;
        let pixels: Vec<[u8; 4]> = (0..(w * h) as usize)
            .map(|i| [(i % 256) as u8, ((i + 77) % 256) as u8, ((i + 155) % 256) as u8, 255])
            .collect();
        let encoded = encode_av1_intra(&pixels, w, h, 0);
        let (decoded, dw, dh) = decode_av1_intra(&encoded).expect("decode must succeed");

        assert_eq!(dw, w);
        assert_eq!(dh, h);
        for (i, (&orig, &dec)) in pixels.iter().zip(decoded.iter()).enumerate() {
            assert_eq!(orig[0], dec[0], "R mismatch at pixel {i}");
            assert_eq!(orig[1], dec[1], "G mismatch at pixel {i}");
            assert_eq!(orig[2], dec[2], "B mismatch at pixel {i}");
        }
    }

    #[test]
    fn test_lossless_solid_color() {
        // All pixels identical.
        let w = 8u32;
        let h = 8u32;
        let pixels = vec![[42u8, 100, 200, 255]; (w * h) as usize];
        let encoded = encode_av1_intra(&pixels, w, h, 0);
        let (decoded, _, _) = decode_av1_intra(&encoded).expect("decode must succeed");
        for (i, (&orig, &dec)) in pixels.iter().zip(decoded.iter()).enumerate() {
            assert_eq!(orig[0], dec[0], "R mismatch at pixel {i}");
            assert_eq!(orig[1], dec[1], "G mismatch at pixel {i}");
            assert_eq!(orig[2], dec[2], "B mismatch at pixel {i}");
        }
    }

    #[test]
    fn test_decode_empty_data_returns_error() {
        let result = decode_av1_intra(&[]);
        assert!(result.is_err(), "empty data must return an error");
    }

    #[test]
    fn test_decoded_alpha_is_255() {
        let pixels = make_pixels(4, 4);
        let encoded = encode_av1_intra(&pixels, 4, 4, 0);
        let (decoded, _, _) = decode_av1_intra(&encoded).expect("decode must succeed");
        for (i, px) in decoded.iter().enumerate() {
            assert_eq!(px[3], 255, "alpha must be 255 at pixel {i}");
        }
    }
}
