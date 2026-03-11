// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Gluteal muscle morph control — gluteus maximus size/shape,
//! hip extension correctives, and adipose distribution.

use std::f32::consts::PI;

/// Gluteal morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GluteParams {
    /// Overall size, 0..=1.
    pub size: f32,
    /// Roundness vs flatness, 0 = flat, 1 = round.
    pub roundness: f32,
    /// Upper vs lower mass distribution, -1 = lower-heavy, 1 = upper.
    pub mass_centre: f32,
    /// Fat deposit layer, 0..=1.
    pub adipose: f32,
    /// Muscle definition / striations, 0..=1.
    pub definition: f32,
    /// Hip extension angle in radians (leg back).
    pub extension_angle: f32,
}

impl Default for GluteParams {
    fn default() -> Self {
        Self {
            size: 0.5,
            roundness: 0.5,
            mass_centre: 0.0,
            adipose: 0.3,
            definition: 0.2,
            extension_angle: 0.0,
        }
    }
}

/// Evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GluteResult {
    pub displacements: Vec<(usize, f32)>,
    pub extension_corrective: f32,
    pub estimated_volume: f32,
}

/// Vertical profile of the gluteal mass.
///
/// `v` from 0 (top) to 1 (bottom / gluteal fold).
#[allow(dead_code)]
pub fn vertical_profile(v: f32, mass_centre: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    let shifted_peak = 0.5 - mass_centre * 0.2;
    let sigma = 0.3;
    (-(v - shifted_peak).powi(2) / (2.0 * sigma * sigma)).exp()
}

/// Horizontal profile (side view curvature).
///
/// `u` from 0 (lateral) to 1 (medial/cleft).
#[allow(dead_code)]
pub fn horizontal_profile(u: f32, roundness: f32) -> f32 {
    let u = u.clamp(0.0, 1.0);
    let exponent = 1.0 + (1.0 - roundness) * 2.0;
    (1.0 - (2.0 * u - 1.0).abs().powf(exponent)).max(0.0)
}

/// Adipose (fat) layer contribution.
#[allow(dead_code)]
pub fn adipose_layer(v: f32, adipose: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    // Fat accumulates more in lower region
    let distribution = 0.5 + 0.5 * v;
    adipose * distribution * 0.008
}

/// Extension corrective weight (smoothstep).
#[allow(dead_code)]
pub fn extension_corrective(angle: f32) -> f32 {
    let t = (angle / (PI / 4.0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Muscle definition groove (striation lines).
#[allow(dead_code)]
pub fn definition_groove(u: f32, v: f32, definition: f32) -> f32 {
    if definition < 0.05 {
        return 0.0;
    }
    // Diagonal striations
    let pattern = ((u * 5.0 + v * 3.0) * PI).sin();
    -definition * 0.001 * pattern.max(0.0)
}

/// Evaluate glute morph.
///
/// `glute_uvs`: per-vertex `(u_horizontal, v_vertical)` in gluteal region.
#[allow(dead_code)]
pub fn evaluate_glute(
    glute_uvs: &[(f32, f32)],
    params: &GluteParams,
) -> GluteResult {
    let ext_w = extension_corrective(params.extension_angle);
    let mut displacements = Vec::with_capacity(glute_uvs.len());
    let mut total_vol = 0.0_f32;

    for (i, &(u, v)) in glute_uvs.iter().enumerate() {
        let vert = vertical_profile(v, params.mass_centre);
        let horiz = horizontal_profile(u, params.roundness);
        let fat = adipose_layer(v, params.adipose);
        let groove = definition_groove(u, v, params.definition);

        let disp = params.size * vert * horiz * 0.03 + fat + groove;
        if disp.abs() > 1e-7 {
            displacements.push((i, disp));
            total_vol += disp.abs();
        }
    }

    GluteResult {
        displacements,
        extension_corrective: ext_w,
        estimated_volume: total_vol * 0.001,
    }
}

/// Blend glute params.
#[allow(dead_code)]
pub fn blend_glute_params(a: &GluteParams, b: &GluteParams, t: f32) -> GluteParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    GluteParams {
        size: a.size * inv + b.size * t,
        roundness: a.roundness * inv + b.roundness * t,
        mass_centre: a.mass_centre * inv + b.mass_centre * t,
        adipose: a.adipose * inv + b.adipose * t,
        definition: a.definition * inv + b.definition * t,
        extension_angle: a.extension_angle * inv + b.extension_angle * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = GluteParams::default();
        assert!((0.0..=1.0).contains(&p.size));
    }

    #[test]
    fn test_vertical_profile_peak() {
        let v = vertical_profile(0.5, 0.0);
        assert!(v > 0.9);
    }

    #[test]
    fn test_horizontal_profile_centre() {
        let h = horizontal_profile(0.5, 1.0);
        assert!(h > 0.9);
    }

    #[test]
    fn test_horizontal_profile_edges() {
        let h0 = horizontal_profile(0.0, 0.5);
        let h1 = horizontal_profile(1.0, 0.5);
        let hm = horizontal_profile(0.5, 0.5);
        assert!(hm > h0);
        assert!(hm > h1);
    }

    #[test]
    fn test_adipose_increases_with_v() {
        let low = adipose_layer(0.0, 1.0);
        let high = adipose_layer(1.0, 1.0);
        assert!(high > low);
    }

    #[test]
    fn test_extension_corrective_bounds() {
        let w0 = extension_corrective(0.0);
        let w1 = extension_corrective(PI / 4.0);
        assert!(w0.abs() < 1e-5);
        assert!((w1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_definition_groove_zero() {
        let g = definition_groove(0.5, 0.5, 0.0);
        assert!(g.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_glute(&[], &GluteParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_evaluate_nonzero() {
        let uvs = vec![(0.5, 0.5), (0.3, 0.4)];
        let params = GluteParams { size: 1.0, ..Default::default() };
        let r = evaluate_glute(&uvs, &params);
        assert!(!r.displacements.is_empty());
    }

    #[test]
    fn test_blend_glute_midpoint() {
        let a = GluteParams { size: 0.0, ..Default::default() };
        let b = GluteParams { size: 1.0, ..Default::default() };
        let r = blend_glute_params(&a, &b, 0.5);
        assert!((r.size - 0.5).abs() < 1e-6);
    }
}
