// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! RGB↔GBR plane packing for AV1 MC_IDENTITY profile (matrix_coefficients=0).
//!
//! In AV1 with MC_IDENTITY the three luma/chroma planes carry:
//!   - plane 0 = G (green)
//!   - plane 1 = B (blue)
//!   - plane 2 = R (red)

/// Separate RGBA pixels into three AV1 GBR planes.
///
/// Returns `(plane_g, plane_b, plane_r)`, each of length `w * h`.
/// The alpha channel is discarded.
///
/// # Panics
/// Panics (debug-only) if `pixels.len() != w * h`.
pub fn rgba_to_gbr_planes(
    pixels: &[[u8; 4]],
    w: usize,
    h: usize,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    debug_assert_eq!(pixels.len(), w * h, "pixel slice length must equal w*h");
    let n = w * h;
    let mut plane_g = Vec::with_capacity(n);
    let mut plane_b = Vec::with_capacity(n);
    let mut plane_r = Vec::with_capacity(n);

    for px in pixels {
        let [r, g, b, _a] = *px;
        plane_g.push(g);
        plane_b.push(b);
        plane_r.push(r);
    }

    (plane_g, plane_b, plane_r)
}

/// Recombine three AV1 GBR planes back to RGBA pixels with alpha=255.
///
/// `plane_g`, `plane_b`, `plane_r` must all have length `w * h`.
pub fn gbr_planes_to_rgba(
    plane_g: &[u8],
    plane_b: &[u8],
    plane_r: &[u8],
    w: usize,
    h: usize,
) -> Vec<[u8; 4]> {
    let n = w * h;
    debug_assert_eq!(plane_g.len(), n);
    debug_assert_eq!(plane_b.len(), n);
    debug_assert_eq!(plane_r.len(), n);

    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        out.push([plane_r[i], plane_g[i], plane_b[i], 255u8]);
    }
    out
}

/// Produce an interleaved [G, B, R] triple for each pixel (alpha discarded).
///
/// Returns a `Vec<[u8; 3]>` of length `w * h`, useful for the lossless WHT path
/// where all three components are processed together per spatial position.
pub fn pack_gbr_interleaved(pixels: &[[u8; 4]], w: usize, h: usize) -> Vec<[u8; 3]> {
    debug_assert_eq!(pixels.len(), w * h, "pixel slice length must equal w*h");
    let n = w * h;
    let mut out = Vec::with_capacity(n);
    for px in pixels {
        let [r, g, b, _a] = *px;
        out.push([g, b, r]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip_rgba_gbr_rgba() {
        let pixels: Vec<[u8; 4]> = vec![
            [255, 0, 0, 255],   // pure red
            [0, 255, 0, 128],   // pure green (alpha 128 – should become 255 after round-trip)
            [0, 0, 255, 64],    // pure blue
            [123, 45, 67, 200], // mixed
            [0, 0, 0, 0],       // black transparent
            [255, 255, 255, 255], // white
        ];
        let w = 2;
        let h = 3;

        let (pg, pb, pr) = rgba_to_gbr_planes(&pixels, w, h);
        let restored = gbr_planes_to_rgba(&pg, &pb, &pr, w, h);

        assert_eq!(restored.len(), pixels.len());
        for (i, (orig, got)) in pixels.iter().zip(restored.iter()).enumerate() {
            // R, G, B must match; alpha is always 255 after round-trip.
            assert_eq!(got[0], orig[0], "R mismatch at {i}");
            assert_eq!(got[1], orig[1], "G mismatch at {i}");
            assert_eq!(got[2], orig[2], "B mismatch at {i}");
            assert_eq!(got[3], 255u8, "alpha should be 255 at {i}");
        }
    }

    #[test]
    fn test_known_pixel_correctness() {
        // pixel = [R=0x10, G=0x20, B=0x30, A=0xFF]
        let pixels = vec![[0x10u8, 0x20, 0x30, 0xFF]];
        let (pg, pb, pr) = rgba_to_gbr_planes(&pixels, 1, 1);
        assert_eq!(pg[0], 0x20, "plane_g should be G");
        assert_eq!(pb[0], 0x30, "plane_b should be B");
        assert_eq!(pr[0], 0x10, "plane_r should be R");
    }

    #[test]
    fn test_plane_lengths() {
        let w = 7;
        let h = 5;
        let pixels: Vec<[u8; 4]> = (0..w * h)
            .map(|i| [(i & 0xFF) as u8, ((i + 1) & 0xFF) as u8, ((i + 2) & 0xFF) as u8, 255])
            .collect();
        let (pg, pb, pr) = rgba_to_gbr_planes(&pixels, w, h);
        assert_eq!(pg.len(), w * h);
        assert_eq!(pb.len(), w * h);
        assert_eq!(pr.len(), w * h);
    }

    #[test]
    fn test_pack_gbr_interleaved_ordering() {
        let pixels = vec![
            [0xAAu8, 0xBB, 0xCC, 0xFF],
            [0x11u8, 0x22, 0x33, 0x00],
        ];
        let packed = pack_gbr_interleaved(&pixels, 2, 1);
        assert_eq!(packed.len(), 2);
        // First pixel: G=0xBB, B=0xCC, R=0xAA
        assert_eq!(packed[0], [0xBB, 0xCC, 0xAA]);
        // Second pixel: G=0x22, B=0x33, R=0x11
        assert_eq!(packed[1], [0x22, 0x33, 0x11]);
    }

    #[test]
    fn test_empty_image() {
        let pixels: Vec<[u8; 4]> = Vec::new();
        let (pg, pb, pr) = rgba_to_gbr_planes(&pixels, 0, 0);
        assert!(pg.is_empty());
        assert!(pb.is_empty());
        assert!(pr.is_empty());
        let restored = gbr_planes_to_rgba(&pg, &pb, &pr, 0, 0);
        assert!(restored.is_empty());
    }
}
