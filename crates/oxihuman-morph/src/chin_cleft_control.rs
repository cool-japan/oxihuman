// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]
#![allow(dead_code)]

//! Chin cleft control for facial morphing.

/// Parameters for chin cleft.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinCleftParams {
    pub depth: f32,
    pub width: f32,
    pub vertical_position: f32,
}

/// Result of chin cleft evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChinCleftResult {
    pub cleft_weight: f32,
    pub depth_weight: f32,
    pub width_weight: f32,
}

/// Default chin cleft parameters.
#[allow(dead_code)]
pub fn default_chin_cleft() -> ChinCleftParams {
    ChinCleftParams {
        depth: 0.0,
        width: 0.5,
        vertical_position: 0.5,
    }
}

/// Evaluate chin cleft morph weights.
#[allow(dead_code)]
pub fn evaluate_chin_cleft(params: &ChinCleftParams) -> ChinCleftResult {
    let d = params.depth.clamp(0.0, 1.0);
    let w = params.width.clamp(0.0, 1.0);
    ChinCleftResult {
        cleft_weight: d * w,
        depth_weight: d,
        width_weight: w,
    }
}

/// Blend two chin cleft params.
#[allow(dead_code)]
pub fn blend_chin_cleft(a: &ChinCleftParams, b: &ChinCleftParams, t: f32) -> ChinCleftParams {
    let t = t.clamp(0.0, 1.0);
    ChinCleftParams {
        depth: a.depth + (b.depth - a.depth) * t,
        width: a.width + (b.width - a.width) * t,
        vertical_position: a.vertical_position + (b.vertical_position - a.vertical_position) * t,
    }
}

/// Set cleft depth.
#[allow(dead_code)]
pub fn set_chin_cleft_depth(params: &mut ChinCleftParams, value: f32) {
    params.depth = value.clamp(0.0, 1.0);
}

/// Compute combined cleft intensity.
#[allow(dead_code)]
pub fn chin_cleft_intensity(params: &ChinCleftParams) -> f32 {
    (params.depth * 0.7 + params.width * 0.3).clamp(0.0, 1.0)
}

/// Validate params.
#[allow(dead_code)]
pub fn is_valid_chin_cleft(params: &ChinCleftParams) -> bool {
    (0.0..=1.0).contains(&params.depth)
        && (0.0..=1.0).contains(&params.width)
        && (0.0..=1.0).contains(&params.vertical_position)
}

/// Reset to defaults.
#[allow(dead_code)]
pub fn reset_chin_cleft(params: &mut ChinCleftParams) {
    *params = default_chin_cleft();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let p = default_chin_cleft();
        assert!(p.depth.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate() {
        let p = ChinCleftParams { depth: 0.5, width: 0.5, vertical_position: 0.5 };
        let r = evaluate_chin_cleft(&p);
        assert!((r.cleft_weight - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_blend() {
        let a = default_chin_cleft();
        let mut b = default_chin_cleft();
        b.depth = 1.0;
        let c = blend_chin_cleft(&a, &b, 0.5);
        assert!((c.depth - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_depth() {
        let mut p = default_chin_cleft();
        set_chin_cleft_depth(&mut p, 0.9);
        assert!((p.depth - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_intensity() {
        let p = ChinCleftParams { depth: 1.0, width: 1.0, vertical_position: 0.5 };
        let v = chin_cleft_intensity(&p);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_valid() {
        assert!(is_valid_chin_cleft(&default_chin_cleft()));
    }

    #[test]
    fn test_invalid() {
        let p = ChinCleftParams { depth: -1.0, width: 0.5, vertical_position: 0.5 };
        assert!(!is_valid_chin_cleft(&p));
    }

    #[test]
    fn test_reset() {
        let mut p = ChinCleftParams { depth: 0.8, width: 0.2, vertical_position: 0.3 };
        reset_chin_cleft(&mut p);
        assert!(p.depth.abs() < 1e-6);
    }

    #[test]
    fn test_zero_depth() {
        let p = ChinCleftParams { depth: 0.0, width: 1.0, vertical_position: 0.5 };
        let r = evaluate_chin_cleft(&p);
        assert!(r.cleft_weight.abs() < 1e-6);
    }

    #[test]
    fn test_blend_identity() {
        let a = default_chin_cleft();
        let c = blend_chin_cleft(&a, &a, 0.5);
        assert!((c.depth - a.depth).abs() < 1e-6);
    }
}
