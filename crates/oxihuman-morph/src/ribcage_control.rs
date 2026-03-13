// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ribcage morph control — rib visibility, barrel-chest vs flat-chest,
//! breathing expansion, and xiphoid process.

use std::f32::consts::PI;

/// Ribcage morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct RibcageParams {
    /// Rib visibility (low body fat), 0..=1.
    pub rib_visibility: f32,
    /// Barrel/flat chest shape, -1 = flat, 1 = barrel.
    pub barrel_factor: f32,
    /// Anteroposterior depth scale.
    pub depth_scale: f32,
    /// Lateral width scale.
    pub width_scale: f32,
    /// Xiphoid process prominence, 0..=1.
    pub xiphoid: f32,
    /// Breathing expansion factor, 0..=1.
    pub breath_expansion: f32,
}

impl Default for RibcageParams {
    fn default() -> Self {
        Self {
            rib_visibility: 0.2,
            barrel_factor: 0.0,
            depth_scale: 1.0,
            width_scale: 1.0,
            xiphoid: 0.2,
            breath_expansion: 0.0,
        }
    }
}

/// Evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RibcageResult {
    pub displacements: Vec<(usize, [f32; 3])>,
    pub chest_circumference: f32,
    pub volume_estimate: f32,
}

/// Rib groove pattern: sinusoidal ridges along the vertical axis.
///
/// `v` from 0 (top of ribcage) to 1 (bottom).
#[allow(dead_code)]
pub fn rib_grooves(v: f32, angle: f32, visibility: f32) -> f32 {
    if visibility < 0.05 {
        return 0.0;
    }
    let v = v.clamp(0.0, 1.0);
    // 12 pairs of ribs, but only lower ones visible
    let rib_count = 7.0;
    let rib_phase = v * rib_count * PI;
    let groove = rib_phase.sin();
    // More visible on lateral sides
    let lateral = (angle - PI / 2.0).cos().abs();
    visibility * 0.002 * groove * lateral * (0.3 + 0.7 * v)
}

/// Barrel chest: elliptical cross-section adjustment.
///
/// `angle` around the torso (0 = anterior, PI = posterior).
#[allow(dead_code)]
pub fn barrel_adjustment(angle: f32, barrel: f32) -> f32 {
    // Increases AP diameter relative to lateral
    let ap_component = angle.cos().abs();
    barrel * 0.01 * ap_component
}

/// Breathing expansion: uniform radial expansion.
#[allow(dead_code)]
pub fn breath_expansion(v: f32, expansion: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    // Upper ribcage expands more
    let factor = 1.0 - v * 0.5;
    expansion * 0.01 * factor
}

/// Xiphoid process bump.
#[allow(dead_code)]
pub fn xiphoid_bump(u: f32, v: f32, prominence: f32) -> f32 {
    // At the bottom-centre of sternum
    let dist_sq = u * u + (v - 0.95).powi(2);
    let radius_sq = 0.02 * 0.02;
    if dist_sq < radius_sq {
        prominence * 0.004 * (1.0 - dist_sq / radius_sq)
    } else {
        0.0
    }
}

/// Estimate chest circumference from params.
#[allow(dead_code)]
pub fn estimate_circumference(width: f32, depth: f32) -> f32 {
    // Approximate ellipse circumference (Ramanujan)
    let a = width * 0.15;
    let b = depth * 0.12;
    let h = ((a - b) / (a + b)).powi(2);
    PI * (a + b) * (1.0 + 3.0 * h / (10.0 + (4.0 - 3.0 * h).sqrt()))
}

/// Evaluate ribcage morph.
///
/// `torso_coords`: `(angle_around, vertical_v, radial_dist)` per vertex.
#[allow(dead_code)]
pub fn evaluate_ribcage(
    torso_coords: &[(f32, f32, f32)],
    params: &RibcageParams,
) -> RibcageResult {
    let mut disps = Vec::with_capacity(torso_coords.len());
    let mut total_vol = 0.0_f32;

    for (i, &(angle, v, _r)) in torso_coords.iter().enumerate() {
        let ribs = rib_grooves(v, angle, params.rib_visibility);
        let barrel = barrel_adjustment(angle, params.barrel_factor);
        let breath = breath_expansion(v, params.breath_expansion);
        let width_adj = (params.width_scale - 1.0) * 0.01 * angle.sin().abs();
        let depth_adj = (params.depth_scale - 1.0) * 0.01 * angle.cos().abs();
        let xp = xiphoid_bump(angle.sin() * 0.1, v, params.xiphoid);

        let radial = ribs + barrel + breath + width_adj + depth_adj + xp;
        if radial.abs() > 1e-7 {
            let nx = angle.cos();
            let nz = angle.sin();
            disps.push((i, [nx * radial, 0.0, nz * radial]));
            total_vol += radial.abs();
        }
    }

    let circ = estimate_circumference(params.width_scale, params.depth_scale);

    RibcageResult {
        displacements: disps,
        chest_circumference: circ,
        volume_estimate: total_vol * 0.001,
    }
}

/// Blend ribcage params.
#[allow(dead_code)]
pub fn blend_ribcage_params(a: &RibcageParams, b: &RibcageParams, t: f32) -> RibcageParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    RibcageParams {
        rib_visibility: a.rib_visibility * inv + b.rib_visibility * t,
        barrel_factor: a.barrel_factor * inv + b.barrel_factor * t,
        depth_scale: a.depth_scale * inv + b.depth_scale * t,
        width_scale: a.width_scale * inv + b.width_scale * t,
        xiphoid: a.xiphoid * inv + b.xiphoid * t,
        breath_expansion: a.breath_expansion * inv + b.breath_expansion * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = RibcageParams::default();
        assert!((0.0..=1.0).contains(&p.rib_visibility));
    }

    #[test]
    fn test_rib_grooves_zero_vis() {
        let g = rib_grooves(0.5, PI / 2.0, 0.0);
        assert!(g.abs() < 1e-6);
    }

    #[test]
    fn test_rib_grooves_produces_pattern() {
        let g1 = rib_grooves(0.3, PI / 2.0, 1.0);
        let g2 = rib_grooves(0.35, PI / 2.0, 1.0);
        // Different heights should produce different values (sinusoidal)
        assert!((g1 - g2).abs() > 1e-6);
    }

    #[test]
    fn test_barrel_adjustment_zero() {
        let b = barrel_adjustment(PI / 2.0, 0.0);
        assert!(b.abs() < 1e-6);
    }

    #[test]
    fn test_breath_expansion_zero() {
        let b = breath_expansion(0.5, 0.0);
        assert!(b.abs() < 1e-6);
    }

    #[test]
    fn test_xiphoid_bump_outside() {
        let x = xiphoid_bump(0.5, 0.5, 1.0);
        assert!(x.abs() < 1e-6);
    }

    #[test]
    fn test_estimate_circumference_positive() {
        let c = estimate_circumference(1.0, 1.0);
        assert!(c > 0.0);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_ribcage(&[], &RibcageParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_evaluate_produces_output() {
        let coords = vec![(PI / 2.0, 0.5, 0.1), (0.0, 0.3, 0.1)];
        let params = RibcageParams { rib_visibility: 1.0, barrel_factor: 0.5, ..Default::default() };
        let r = evaluate_ribcage(&coords, &params);
        assert!(!r.displacements.is_empty());
    }

    #[test]
    fn test_blend_ribcage_midpoint() {
        let a = RibcageParams { rib_visibility: 0.0, ..Default::default() };
        let b = RibcageParams { rib_visibility: 1.0, ..Default::default() };
        let r = blend_ribcage_params(&a, &b, 0.5);
        assert!((r.rib_visibility - 0.5).abs() < 1e-6);
    }
}
