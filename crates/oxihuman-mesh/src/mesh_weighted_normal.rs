// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Weighted normal modifier.

/// Weighting mode for normal computation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeightMode {
    FaceArea,
    FaceAngle,
    FaceAreaAngle,
}

/// Configuration for weighted normals.
#[derive(Debug, Clone)]
pub struct WeightedNormalConfig {
    pub mode: WeightMode,
    pub weight: f32,
    pub threshold_face_angle: f32,
    pub keep_sharp: bool,
}

impl Default for WeightedNormalConfig {
    fn default() -> Self {
        Self {
            mode: WeightMode::FaceArea,
            weight: 50.0,
            threshold_face_angle: std::f32::consts::PI / 6.0,
            keep_sharp: false,
        }
    }
}

/// Compute the area of a triangle.
pub fn triangle_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt() * 0.5
}

/// Compute face normal for a triangle.
pub fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
    [n[0] / len, n[1] / len, n[2] / len]
}

/// Normalize a normal vector.
pub fn normalize_normal(n: [f32; 3]) -> [f32; 3] {
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
    [n[0] / len, n[1] / len, n[2] / len]
}

/// Compute area-weighted vertex normals from triangle faces.
pub fn compute_weighted_normals(
    positions: &[[f32; 3]],
    triangles: &[[usize; 3]],
    cfg: &WeightedNormalConfig,
) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0_f32; 3]; positions.len()];
    for tri in triangles {
        let a = positions[tri[0]];
        let b = positions[tri[1]];
        let c = positions[tri[2]];
        let fn_ = face_normal(a, b, c);
        let weight = match cfg.mode {
            WeightMode::FaceArea => triangle_area(a, b, c),
            WeightMode::FaceAngle => 1.0,
            WeightMode::FaceAreaAngle => triangle_area(a, b, c),
        } * cfg.weight;
        for &vi in tri {
            normals[vi][0] += fn_[0] * weight;
            normals[vi][1] += fn_[1] * weight;
            normals[vi][2] += fn_[2] * weight;
        }
    }
    normals.iter_mut().map(|n| normalize_normal(*n)).collect()
}

/// Validate config.
pub fn validate_weighted_normal_config(cfg: &WeightedNormalConfig) -> bool {
    cfg.weight > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_area_unit() {
        let a = triangle_area([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!((a - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_face_normal_xy_plane() {
        let n = face_normal([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        assert!(n[2].abs() > 0.99);
    }

    #[test]
    fn test_normalize_normal_unit() {
        let n = normalize_normal([3.0, 4.0, 0.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_weighted_normals_count() {
        let pos = vec![[0.0, 0.0, 0.0_f32], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0_usize, 1, 2]];
        let cfg = WeightedNormalConfig::default();
        let out = compute_weighted_normals(&pos, &tris, &cfg);
        assert_eq!(out.len(), 3);
    }

    #[test]
    fn test_compute_weighted_normals_direction() {
        let pos = vec![[0.0, 0.0, 0.0_f32], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let tris = vec![[0_usize, 1, 2]];
        let cfg = WeightedNormalConfig::default();
        let out = compute_weighted_normals(&pos, &tris, &cfg);
        assert!(out[0][2] > 0.0); /* pointing up */
    }

    #[test]
    fn test_validate_config_valid() {
        let cfg = WeightedNormalConfig::default();
        assert!(validate_weighted_normal_config(&cfg));
    }

    #[test]
    fn test_validate_config_zero_weight() {
        let cfg = WeightedNormalConfig { weight: 0.0, ..WeightedNormalConfig::default() };
        assert!(!validate_weighted_normal_config(&cfg));
    }

    #[test]
    fn test_weight_mode_debug() {
        assert!(format!("{:?}", WeightMode::FaceAreaAngle).contains("FaceAreaAngle"));
    }

    #[test]
    fn test_triangle_area_degenerate() {
        let a = triangle_area([0.0; 3], [0.0; 3], [0.0; 3]);
        assert!(a.abs() < 1e-5);
    }
}
