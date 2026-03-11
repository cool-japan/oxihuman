// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Collarbone (clavicle) morph control — bone prominence, length,
//! angle, and supraclavicular fossa depth.

use std::f32::consts::PI;

/// Collarbone morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CollarParams {
    /// Bone prominence, 0 = hidden, 1 = very visible.
    pub prominence: f32,
    /// Clavicle length scale (0.8..=1.2 typical).
    pub length_scale: f32,
    /// Clavicle elevation angle (shoulder droop/raise) in radians.
    pub elevation: f32,
    /// Supraclavicular fossa (hollow above collarbone) depth, 0..=1.
    pub fossa_depth: f32,
    /// Sternal notch depth, 0..=1.
    pub sternal_notch: f32,
    /// Body fat covering (reduces visibility), 0..=1.
    pub fat_cover: f32,
}

impl Default for CollarParams {
    fn default() -> Self {
        Self {
            prominence: 0.5,
            length_scale: 1.0,
            elevation: 0.0,
            fossa_depth: 0.3,
            sternal_notch: 0.4,
            fat_cover: 0.3,
        }
    }
}

/// Evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CollarResult {
    pub displacements: Vec<(usize, [f32; 3])>,
    pub effective_prominence: f32,
}

/// Effective prominence after fat covering.
#[allow(dead_code)]
pub fn effective_prominence(prominence: f32, fat_cover: f32) -> f32 {
    let p = prominence.clamp(0.0, 1.0);
    let f = fat_cover.clamp(0.0, 1.0);
    (p * (1.0 - f * 0.8)).max(0.0)
}

/// Clavicle ridge profile along the bone axis.
///
/// `t` from 0 (sternum) to 1 (shoulder).
#[allow(dead_code)]
pub fn ridge_profile(t: f32, prominence: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    // S-curve shape of clavicle: convex medially, concave laterally
    let medial = 0.5 * (1.0 + (PI * (2.0 * t - 0.3)).cos());
    let lateral = -0.3 * (1.0 + (PI * (2.0 * t - 0.7)).cos());
    let shape = if t < 0.5 { medial } else { medial + lateral };
    prominence * 0.005 * shape.max(0.0)
}

/// Supraclavicular fossa depth profile.
#[allow(dead_code)]
pub fn fossa_profile(t: f32, vertical: f32, depth: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let vertical = vertical.clamp(0.0, 1.0);
    // Fossa is above the clavicle, peaks at ~30% along bone
    let along = (-((t - 0.3) / 0.2).powi(2)).exp();
    let above = if (0.0..=0.3).contains(&vertical) {
        1.0 - vertical / 0.3
    } else {
        0.0
    };
    -depth * 0.004 * along * above
}

/// Sternal notch — the V-shaped hollow at the base of the neck.
#[allow(dead_code)]
pub fn sternal_notch_depth(x: f32, y: f32, depth: f32) -> f32 {
    let dist_sq = x * x + y * y;
    let radius = 0.02;
    if dist_sq < radius * radius {
        -depth * 0.003 * (1.0 - dist_sq / (radius * radius))
    } else {
        0.0
    }
}

/// Elevation rotation of a point around the sternum pivot.
#[allow(dead_code)]
pub fn apply_elevation(x: f32, y: f32, angle: f32) -> (f32, f32) {
    let c = angle.cos();
    let s = angle.sin();
    (x * c - y * s, x * s + y * c)
}

/// Evaluate collarbone morph.
#[allow(dead_code)]
pub fn evaluate_collar(
    positions: &[[f32; 3]],
    collar_origin: [f32; 3],
    collar_direction: [f32; 3],
    params: &CollarParams,
) -> CollarResult {
    let eff_prom = effective_prominence(params.prominence, params.fat_cover);
    let dir_len = (collar_direction[0].powi(2) + collar_direction[1].powi(2) + collar_direction[2].powi(2)).sqrt();
    if dir_len < 1e-6 {
        return CollarResult { displacements: vec![], effective_prominence: eff_prom };
    }
    let dir = [collar_direction[0] / dir_len, collar_direction[1] / dir_len, collar_direction[2] / dir_len];

    let mut disps = Vec::new();
    for (i, pos) in positions.iter().enumerate() {
        let rel = [pos[0] - collar_origin[0], pos[1] - collar_origin[1], pos[2] - collar_origin[2]];
        let along = rel[0] * dir[0] + rel[1] * dir[1] + rel[2] * dir[2];
        let t = (along / (dir_len * params.length_scale)).clamp(0.0, 1.0);

        let ridge = ridge_profile(t, eff_prom);
        let dist_to_line = ((rel[1] - along * dir[1]).powi(2) + (rel[2] - along * dir[2]).powi(2)).sqrt();
        let falloff = (-dist_to_line * dist_to_line * 1000.0).exp();

        let dy = ridge * falloff;
        if dy.abs() > 1e-7 {
            disps.push((i, [0.0, dy, ridge * falloff * 0.5]));
        }
    }

    CollarResult {
        displacements: disps,
        effective_prominence: eff_prom,
    }
}

/// Blend collar params.
#[allow(dead_code)]
pub fn blend_collar_params(a: &CollarParams, b: &CollarParams, t: f32) -> CollarParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    CollarParams {
        prominence: a.prominence * inv + b.prominence * t,
        length_scale: a.length_scale * inv + b.length_scale * t,
        elevation: a.elevation * inv + b.elevation * t,
        fossa_depth: a.fossa_depth * inv + b.fossa_depth * t,
        sternal_notch: a.sternal_notch * inv + b.sternal_notch * t,
        fat_cover: a.fat_cover * inv + b.fat_cover * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let p = CollarParams::default();
        assert!((0.0..=1.0).contains(&p.prominence));
    }

    #[test]
    fn test_effective_prominence_no_fat() {
        let ep = effective_prominence(0.8, 0.0);
        assert!((ep - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_effective_prominence_full_fat() {
        let ep = effective_prominence(1.0, 1.0);
        assert!(ep < 0.25);
    }

    #[test]
    fn test_ridge_profile_start() {
        let r = ridge_profile(0.0, 1.0);
        assert!(r >= 0.0);
    }

    #[test]
    fn test_fossa_profile_below() {
        let f = fossa_profile(0.3, 0.5, 1.0);
        assert!(f.abs() < 1e-6);
    }

    #[test]
    fn test_sternal_notch_outside() {
        let d = sternal_notch_depth(0.1, 0.1, 1.0);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_elevation_identity() {
        let (x, y) = apply_elevation(1.0, 0.0, 0.0);
        assert!((x - 1.0).abs() < 1e-5);
        assert!(y.abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_collar(&[], [0.0; 3], [1.0, 0.0, 0.0], &CollarParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_blend_collar_midpoint() {
        let a = CollarParams { prominence: 0.0, ..Default::default() };
        let b = CollarParams { prominence: 1.0, ..Default::default() };
        let r = blend_collar_params(&a, &b, 0.5);
        assert!((r.prominence - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_zero_direction() {
        let r = evaluate_collar(&[[0.0; 3]], [0.0; 3], [0.0; 3], &CollarParams::default());
        assert!(r.displacements.is_empty());
    }
}
