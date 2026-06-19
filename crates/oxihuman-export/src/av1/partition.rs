// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Block grid helpers for the simplified AV1-like codec.
//!
//! All blocks are 4×4 pixels with PARTITION_NONE.  The image is tiled into a
//! regular grid of 4×4 blocks, processed left-to-right, top-to-bottom.
//! Blocks that extend beyond the image boundary are zero-padded on input and
//! clipped on output.

/// Total 4×4 block count for an image of given pixel dimensions.
///
/// # Examples
/// ```
/// # use oxihuman_export::av1::partition::block_count;
/// assert_eq!(block_count(8, 8), 4);
/// assert_eq!(block_count(5, 5), 4); // rounds up
/// ```
pub fn block_count(width: usize, height: usize) -> usize {
    let bw = width.div_ceil(4);
    let bh = height.div_ceil(4);
    bw * bh
}

/// Compute the (bx, by) — top-left pixel coordinates of every 4×4 block in
/// raster (left-to-right, top-to-bottom) order.
pub fn block_grid(width: usize, height: usize) -> Vec<(usize, usize)> {
    let bw = width.div_ceil(4);
    let bh = height.div_ceil(4);
    let mut coords = Vec::with_capacity(bw * bh);
    for by in 0..bh {
        for bx in 0..bw {
            coords.push((bx * 4, by * 4));
        }
    }
    coords
}

/// Extract a 4×4 pixel block from a single-channel plane.
///
/// Pixels outside the image boundary (`px >= w` or `py >= h`) are set to
/// zero (black) rather than wrapping or reflecting.
///
/// # Parameters
/// - `plane`:  flat row-major pixel buffer (length = `stride * h` at minimum).
/// - `stride`: number of pixels per row (may equal `w` or be larger).
/// - `bx, by`: top-left pixel coordinate of the block.
/// - `w, h`:   full image dimensions in pixels.
pub fn extract_block(
    plane: &[u8],
    stride: usize,
    bx: usize,
    by: usize,
    w: usize,
    h: usize,
) -> [u8; 16] {
    let mut block = [0u8; 16];
    for r in 0..4 {
        for c in 0..4 {
            let px = bx + c;
            let py = by + r;
            if px < w && py < h {
                block[r * 4 + c] = plane[py * stride + px];
            }
        }
    }
    block
}

/// Write a decoded 4×4 block back into a single-channel plane.
///
/// Only pixels within `[0, w) × [0, h)` are written; boundary pixels are
/// silently discarded.
///
/// # Parameters
/// - `plane`:  mutable flat row-major pixel buffer.
/// - `stride`: number of pixels per row.
/// - `bx, by`: top-left pixel coordinate of the block.
/// - `w, h`:   full image dimensions in pixels.
/// - `block`:  16 decoded pixel values in row-major order.
pub fn place_block(
    plane: &mut [u8],
    stride: usize,
    bx: usize,
    by: usize,
    w: usize,
    h: usize,
    block: &[u8; 16],
) {
    for r in 0..4 {
        for c in 0..4 {
            let px = bx + c;
            let py = by + r;
            if px < w && py < h {
                plane[py * stride + px] = block[r * 4 + c];
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_count_exact_multiples() {
        assert_eq!(block_count(4, 4), 1);
        assert_eq!(block_count(8, 8), 4);
        assert_eq!(block_count(16, 8), 8);
    }

    #[test]
    fn test_block_count_non_multiples() {
        // 5×5 → 2×2 = 4 blocks (rounded up)
        assert_eq!(block_count(5, 5), 4);
        // 7×9 → 2×3 = 6 blocks
        assert_eq!(block_count(7, 9), 6);
        // 1×1 → 1×1 = 1 block
        assert_eq!(block_count(1, 1), 1);
    }

    #[test]
    fn test_block_grid_order() {
        let grid = block_grid(8, 8);
        assert_eq!(grid.len(), 4);
        // Raster order: (0,0), (4,0), (0,4), (4,4)
        assert_eq!(grid[0], (0, 0));
        assert_eq!(grid[1], (4, 0));
        assert_eq!(grid[2], (0, 4));
        assert_eq!(grid[3], (4, 4));
    }

    #[test]
    fn test_block_grid_single_block() {
        let grid = block_grid(4, 4);
        assert_eq!(grid, vec![(0usize, 0usize)]);
    }

    #[test]
    fn test_extract_block_full() {
        // 8×4 plane, all pixels = their index mod 256.
        let plane: Vec<u8> = (0u8..32).collect();
        // Block at (0,0): rows 0–3, cols 0–3.
        let block = extract_block(&plane, 8, 0, 0, 8, 4);
        assert_eq!(block[0], 0);
        assert_eq!(block[1], 1);
        assert_eq!(block[4], 8); // row 1, col 0
        assert_eq!(block[15], 27); // row 3, col 3
    }

    #[test]
    fn test_extract_block_at_boundary() {
        // 6×6 image; block at (4,4) overlaps boundary on right and bottom.
        let plane = vec![1u8; 36]; // all ones
        let block = extract_block(&plane, 6, 4, 4, 6, 6);
        // Valid pixels: (4,4),(5,4),(4,5),(5,5) → block[0],[1],[4],[5] = 1
        assert_eq!(block[0], 1); // (4,4)
        assert_eq!(block[1], 1); // (5,4)
        assert_eq!(block[2], 0); // (6,4) out-of-bounds → 0
        assert_eq!(block[3], 0); // (7,4) out-of-bounds → 0
        assert_eq!(block[4], 1); // (4,5)
        assert_eq!(block[5], 1); // (5,5)
        assert_eq!(block[6], 0); // (6,5) out-of-bounds → 0
        assert_eq!(block[8], 0); // row 2 is entirely OOB
    }

    #[test]
    fn test_place_block_and_extract_round_trip() {
        let mut plane = vec![0u8; 64]; // 8×8
        let block: [u8; 16] = [
            10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160,
        ];
        place_block(&mut plane, 8, 0, 0, 8, 8, &block);
        let extracted = extract_block(&plane, 8, 0, 0, 8, 8);
        assert_eq!(extracted, block);
    }

    #[test]
    fn test_place_block_boundary_clipping() {
        // Place a block that partially extends beyond a 6×6 image.
        let mut plane = vec![0u8; 36]; // 6×6
        let block = [255u8; 16];
        place_block(&mut plane, 6, 4, 4, 6, 6, &block);
        // Only pixels (4,4), (5,4), (4,5), (5,5) should be written.
        assert_eq!(plane[4 * 6 + 4], 255);
        assert_eq!(plane[4 * 6 + 5], 255);
        assert_eq!(plane[5 * 6 + 4], 255);
        assert_eq!(plane[5 * 6 + 5], 255);
        // Out-of-bounds writes must not corrupt memory — verified implicitly
        // by the fact that plane has exactly 36 bytes and no panic occurred.
    }
}
