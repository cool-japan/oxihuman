// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge detection filters — Sobel, Laplacian, Roberts cross,
//! and depth/normal-based edge detection for post-processing.

use std::f32::consts::FRAC_1_SQRT_2;

/// Edge detection method.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeMethod {
    Sobel,
    Laplacian,
    RobertsCross,
    DepthBased,
    NormalBased,
}

/// Edge detection config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeConfig {
    pub method: EdgeMethod,
    /// Threshold below which edges are suppressed.
    pub threshold: f32,
    /// Edge color [r, g, b].
    pub edge_color: [f32; 3],
    /// Edge width in pixels.
    pub edge_width: f32,
    /// Depth sensitivity for depth-based detection.
    pub depth_sensitivity: f32,
    /// Normal sensitivity for normal-based detection.
    pub normal_sensitivity: f32,
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            method: EdgeMethod::Sobel,
            threshold: 0.1,
            edge_color: [0.0, 0.0, 0.0],
            edge_width: 1.0,
            depth_sensitivity: 1.0,
            normal_sensitivity: 1.0,
        }
    }
}

/// Apply Sobel operator to a 3x3 neighbourhood of luminance values.
///
/// Layout: `[tl, tc, tr, ml, mc, mr, bl, bc, br]` (top-left to bottom-right).
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn sobel_3x3(neighbourhood: &[f32; 9]) -> f32 {
    let n = neighbourhood;
    let gx = -n[0] + n[2] - 2.0 * n[3] + 2.0 * n[5] - n[6] + n[8];
    let gy = -n[0] - 2.0 * n[1] - n[2] + n[6] + 2.0 * n[7] + n[8];
    (gx * gx + gy * gy).sqrt()
}

/// Apply Laplacian operator to a 3x3 neighbourhood.
#[allow(dead_code)]
pub fn laplacian_3x3(neighbourhood: &[f32; 9]) -> f32 {
    let n = neighbourhood;
    let lap = -n[1] - n[3] + 4.0 * n[4] - n[5] - n[7];
    lap.abs()
}

/// Roberts cross operator on a 2x2 block.
///
/// `block`: `[tl, tr, bl, br]`.
#[allow(dead_code)]
pub fn roberts_cross(block: &[f32; 4]) -> f32 {
    let gx = block[0] - block[3];
    let gy = block[1] - block[2];
    (gx * gx + gy * gy).sqrt()
}

/// Depth-based edge detection: compare centre depth to neighbours.
#[allow(dead_code)]
pub fn depth_edge(depths: &[f32; 9], sensitivity: f32) -> f32 {
    let centre = depths[4];
    if centre.abs() < 1e-9 {
        return 0.0;
    }
    let mut max_diff = 0.0_f32;
    for (i, &d) in depths.iter().enumerate() {
        if i == 4 { continue; }
        let diff = ((d - centre) / centre).abs();
        max_diff = max_diff.max(diff);
    }
    (max_diff * sensitivity).clamp(0.0, 1.0)
}

/// Normal-based edge detection: compare centre normal to neighbours.
#[allow(dead_code)]
pub fn normal_edge(normals: &[[f32; 3]; 9], sensitivity: f32) -> f32 {
    let cn = normals[4];
    let mut max_diff = 0.0_f32;
    for (i, n) in normals.iter().enumerate() {
        if i == 4 { continue; }
        let dot = cn[0] * n[0] + cn[1] * n[1] + cn[2] * n[2];
        let diff = 1.0 - dot.clamp(-1.0, 1.0);
        max_diff = max_diff.max(diff);
    }
    (max_diff * sensitivity).clamp(0.0, 1.0)
}

/// Convert RGB to luminance.
#[allow(dead_code)]
pub fn luminance(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Threshold an edge value.
#[allow(dead_code)]
pub fn threshold_edge(edge: f32, threshold: f32) -> f32 {
    if edge > threshold { edge } else { 0.0 }
}

/// Blend edge colour into fragment colour.
#[allow(dead_code)]
pub fn apply_edge_color(
    frag_color: [f32; 3],
    edge_strength: f32,
    edge_color: [f32; 3],
) -> [f32; 3] {
    let s = edge_strength.clamp(0.0, 1.0);
    [
        frag_color[0] * (1.0 - s) + edge_color[0] * s,
        frag_color[1] * (1.0 - s) + edge_color[1] * s,
        frag_color[2] * (1.0 - s) + edge_color[2] * s,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = EdgeConfig::default();
        assert_eq!(c.method, EdgeMethod::Sobel);
    }

    #[test]
    fn test_sobel_flat() {
        let flat = [0.5; 9];
        let e = sobel_3x3(&flat);
        assert!(e.abs() < 1e-5);
    }

    #[test]
    fn test_sobel_edge() {
        let edge = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        let e = sobel_3x3(&edge);
        assert!(e > 1.0);
    }

    #[test]
    fn test_laplacian_flat() {
        let flat = [0.5; 9];
        let e = laplacian_3x3(&flat);
        assert!(e.abs() < 1e-5);
    }

    #[test]
    fn test_laplacian_peak() {
        let peak = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let e = laplacian_3x3(&peak);
        assert!(e > 3.0);
    }

    #[test]
    fn test_roberts_cross_flat() {
        let flat = [0.5; 4];
        let e = roberts_cross(&flat);
        assert!(e.abs() < 1e-5);
    }

    #[test]
    fn test_depth_edge_uniform() {
        let depths = [1.0; 9];
        let e = depth_edge(&depths, 1.0);
        assert!(e.abs() < 1e-5);
    }

    #[test]
    fn test_luminance_white() {
        let l = luminance(1.0, 1.0, 1.0);
        assert!((l - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_threshold_below() {
        assert_eq!(threshold_edge(0.05, 0.1), 0.0);
    }

    #[test]
    fn test_apply_edge_color_full() {
        let r = apply_edge_color([1.0, 1.0, 1.0], 1.0, [0.0, 0.0, 0.0]);
        assert!(r[0].abs() < 1e-5);
    }
}
