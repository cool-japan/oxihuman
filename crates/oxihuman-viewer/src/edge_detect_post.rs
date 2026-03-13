// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

//! Edge detection post-process: Sobel, Laplacian, and Canny-style edge detection.

use std::f32::consts::FRAC_1_SQRT_2;

/// Edge detection method.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EdgeMethod {
    Sobel,
    Laplacian,
    Roberts,
}

/// Configuration for edge detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeDetectConfig {
    pub method: EdgeMethod,
    pub threshold: f32,
    pub edge_color: [f32; 3],
    pub background_color: [f32; 3],
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_edge_detect_config() -> EdgeDetectConfig {
    EdgeDetectConfig {
        method: EdgeMethod::Sobel,
        threshold: 0.1,
        edge_color: [1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0],
        enabled: true,
    }
}

/// Sobel gradient magnitude for a 3x3 neighbourhood of luminances.
#[allow(dead_code)]
pub fn sobel_magnitude(neighborhood: &[f32; 9]) -> f32 {
    let gx = -neighborhood[0] + neighborhood[2] - 2.0 * neighborhood[3] + 2.0 * neighborhood[5]
        - neighborhood[6]
        + neighborhood[8];
    let gy = -neighborhood[0] - 2.0 * neighborhood[1] - neighborhood[2]
        + neighborhood[6]
        + 2.0 * neighborhood[7]
        + neighborhood[8];
    (gx * gx + gy * gy).sqrt()
}

/// Laplacian for a 3x3 neighbourhood.
#[allow(dead_code)]
pub fn laplacian_magnitude(neighborhood: &[f32; 9]) -> f32 {
    let lap = -4.0 * neighborhood[4]
        + neighborhood[1]
        + neighborhood[3]
        + neighborhood[5]
        + neighborhood[7];
    lap.abs()
}

/// Roberts cross gradient magnitude.
#[allow(dead_code)]
pub fn roberts_magnitude(tl: f32, tr: f32, bl: f32, br: f32) -> f32 {
    let gx = br - tl;
    let gy = bl - tr;
    (gx * gx + gy * gy).sqrt()
}

#[allow(dead_code)]
pub fn is_edge(magnitude: f32, threshold: f32) -> bool {
    magnitude > threshold
}

#[allow(dead_code)]
pub fn edge_method_name(method: EdgeMethod) -> &'static str {
    match method {
        EdgeMethod::Sobel => "sobel",
        EdgeMethod::Laplacian => "laplacian",
        EdgeMethod::Roberts => "roberts",
    }
}

#[allow(dead_code)]
pub fn edge_detect_to_json(cfg: &EdgeDetectConfig) -> String {
    format!(
        r#"{{"method":"{}","threshold":{:.4},"enabled":{}}}"#,
        edge_method_name(cfg.method),
        cfg.threshold,
        cfg.enabled
    )
}

/// Diagonal edge weight factor.
#[allow(dead_code)]
pub fn diagonal_weight() -> f32 {
    FRAC_1_SQRT_2
}

#[allow(dead_code)]
pub fn set_edge_threshold(cfg: &mut EdgeDetectConfig, v: f32) {
    cfg.threshold = v.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_edge_detect_config();
        assert_eq!(cfg.method, EdgeMethod::Sobel);
        assert!((cfg.threshold - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_sobel_flat() {
        let n = [0.5; 9];
        assert!(sobel_magnitude(&n).abs() < 1e-6);
    }

    #[test]
    fn test_sobel_edge() {
        let n = [0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0];
        assert!(sobel_magnitude(&n) > 0.0);
    }

    #[test]
    fn test_laplacian_flat() {
        let n = [0.5; 9];
        assert!(laplacian_magnitude(&n).abs() < 1e-6);
    }

    #[test]
    fn test_laplacian_edge() {
        let mut n = [0.0; 9];
        n[4] = 1.0;
        assert!(laplacian_magnitude(&n) > 0.0);
    }

    #[test]
    fn test_roberts_flat() {
        assert!(roberts_magnitude(0.5, 0.5, 0.5, 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_is_edge() {
        assert!(is_edge(0.5, 0.1));
        assert!(!is_edge(0.05, 0.1));
    }

    #[test]
    fn test_edge_method_name() {
        assert_eq!(edge_method_name(EdgeMethod::Roberts), "roberts");
    }

    #[test]
    fn test_diagonal_weight() {
        assert!((diagonal_weight() - FRAC_1_SQRT_2).abs() < 1e-6);
    }

    #[test]
    fn test_set_threshold() {
        let mut cfg = default_edge_detect_config();
        set_edge_threshold(&mut cfg, 0.5);
        assert!((cfg.threshold - 0.5).abs() < 1e-6);
    }
}
