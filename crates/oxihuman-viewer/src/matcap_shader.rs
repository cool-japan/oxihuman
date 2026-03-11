// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! MatCap (Material Capture) shader — view-space normal to UV mapping
//! for fast material preview rendering.

use std::f32::consts::PI;

/// MatCap lookup configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct MatcapConfig {
    /// Texture resolution (square).
    pub resolution: u32,
    /// Rotation offset in radians (rotate the matcap).
    pub rotation: f32,
    /// Intensity multiplier.
    pub intensity: f32,
    /// Whether to use half-lambert softening.
    pub half_lambert: bool,
}

impl Default for MatcapConfig {
    fn default() -> Self {
        Self {
            resolution: 256,
            rotation: 0.0,
            intensity: 1.0,
            half_lambert: false,
        }
    }
}

/// Compute MatCap UV from a view-space normal.
///
/// View-space normal `n` should be normalised. Returns `(u, v)` in 0..=1.
#[allow(dead_code)]
pub fn normal_to_matcap_uv(n: [f32; 3]) -> (f32, f32) {
    let u = n[0] * 0.5 + 0.5;
    let v = n[1] * 0.5 + 0.5;
    (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

/// Compute MatCap UV with rotation.
#[allow(dead_code)]
pub fn normal_to_matcap_uv_rotated(n: [f32; 3], rotation: f32) -> (f32, f32) {
    let c = rotation.cos();
    let s = rotation.sin();
    let rx = n[0] * c - n[1] * s;
    let ry = n[0] * s + n[1] * c;
    let u = rx * 0.5 + 0.5;
    let v = ry * 0.5 + 0.5;
    (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

/// Transform world-space normal to view-space using a 3x3 view matrix.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn world_to_view_normal(normal: [f32; 3], view_mat: &[[f32; 3]; 3]) -> [f32; 3] {
    let mut result = [0.0; 3];
    for i in 0..3 {
        for j in 0..3 {
            result[i] += view_mat[j][i] * normal[j];
        }
    }
    normalize(result)
}

/// Half-Lambert transformation for softer lighting.
#[allow(dead_code)]
pub fn half_lambert(n_dot_l: f32) -> f32 {
    let h = n_dot_l * 0.5 + 0.5;
    h * h
}

/// Sample a procedural matcap (metallic sphere).
///
/// Returns `[r, g, b]` for the given UV.
#[allow(dead_code)]
pub fn procedural_matcap_metallic(u: f32, v: f32) -> [f32; 3] {
    let x = u * 2.0 - 1.0;
    let y = v * 2.0 - 1.0;
    let r_sq = x * x + y * y;
    if r_sq > 1.0 {
        return [0.0, 0.0, 0.0];
    }
    let z = (1.0 - r_sq).sqrt();
    // Metallic: sharp specular highlight
    let spec = z.powf(32.0);
    let diffuse = z * 0.3;
    let val = (diffuse + spec).clamp(0.0, 1.0);
    [val * 0.8, val * 0.82, val * 0.85]
}

/// Sample a procedural matcap (clay).
#[allow(dead_code)]
pub fn procedural_matcap_clay(u: f32, v: f32) -> [f32; 3] {
    let x = u * 2.0 - 1.0;
    let y = v * 2.0 - 1.0;
    let r_sq = x * x + y * y;
    if r_sq > 1.0 {
        return [0.0, 0.0, 0.0];
    }
    let z = (1.0 - r_sq).sqrt();
    let diffuse = z * 0.6 + 0.2;
    let rim = (1.0 - z).powf(2.0) * 0.15;
    let val = (diffuse + rim).clamp(0.0, 1.0);
    [val * 0.9, val * 0.85, val * 0.8]
}

/// Sample a procedural matcap (toon/cel-shaded).
#[allow(dead_code)]
pub fn procedural_matcap_toon(u: f32, v: f32, levels: u32) -> [f32; 3] {
    let x = u * 2.0 - 1.0;
    let y = v * 2.0 - 1.0;
    let r_sq = x * x + y * y;
    if r_sq > 1.0 {
        return [0.0, 0.0, 0.0];
    }
    let z = (1.0 - r_sq).sqrt();
    let levels_f = levels.max(1) as f32;
    let quantised = (z * levels_f).floor() / levels_f;
    let val = quantised.clamp(0.0, 1.0);
    [val, val, val]
}

/// Pixel coordinate to matcap UV.
#[allow(dead_code)]
pub fn pixel_to_uv(x: u32, y: u32, resolution: u32) -> (f32, f32) {
    if resolution == 0 {
        return (0.5, 0.5);
    }
    let u = (x as f32 + 0.5) / resolution as f32;
    let v = (y as f32 + 0.5) / resolution as f32;
    (u, v)
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 { return [0.0, 0.0, 1.0]; }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = MatcapConfig::default();
        assert_eq!(c.resolution, 256);
    }

    #[test]
    fn test_normal_to_uv_forward() {
        let (u, v) = normal_to_matcap_uv([0.0, 0.0, 1.0]);
        assert!((u - 0.5).abs() < 1e-5);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_normal_to_uv_right() {
        let (u, _v) = normal_to_matcap_uv([1.0, 0.0, 0.0]);
        assert!((u - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_rotated_uv_zero_rotation() {
        let (u1, v1) = normal_to_matcap_uv([0.5, 0.5, 0.0]);
        let (u2, v2) = normal_to_matcap_uv_rotated([0.5, 0.5, 0.0], 0.0);
        assert!((u1 - u2).abs() < 1e-5);
        assert!((v1 - v2).abs() < 1e-5);
    }

    #[test]
    fn test_half_lambert_range() {
        let h = half_lambert(-1.0);
        assert!(h >= 0.0);
        let h = half_lambert(1.0);
        assert!(h <= 1.0);
    }

    #[test]
    fn test_procedural_metallic_centre() {
        let c = procedural_matcap_metallic(0.5, 0.5);
        assert!(c[0] > 0.0);
    }

    #[test]
    fn test_procedural_metallic_outside() {
        let c = procedural_matcap_metallic(0.0, 0.0);
        assert!(c[0].abs() < 1e-5);
    }

    #[test]
    fn test_procedural_clay_centre() {
        let c = procedural_matcap_clay(0.5, 0.5);
        assert!(c[0] > 0.5);
    }

    #[test]
    fn test_procedural_toon_quantised() {
        let c = procedural_matcap_toon(0.5, 0.5, 3);
        // Should be one of 3 discrete levels
        let possible = [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0];
        let matches = possible.iter().any(|&p| (c[0] - p).abs() < 1e-3);
        assert!(matches, "Should be quantised, got {}", c[0]);
    }

    #[test]
    fn test_pixel_to_uv_zero_res() {
        let (u, v) = pixel_to_uv(0, 0, 0);
        assert!((u - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_pixel_to_uv_centre() {
        let (u, v) = pixel_to_uv(127, 127, 256);
        assert!((u - 0.498).abs() < 0.01);
    }
}
