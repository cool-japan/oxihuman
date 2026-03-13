// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Elbow morph control — olecranon prominence, flexion correctives,
//! carrying angle, and skin fold modelling.

use std::f32::consts::PI;

/// Elbow morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ElbowParams {
    /// Olecranon (elbow bone) prominence, 0..=1.
    pub olecranon_prominence: f32,
    /// Flexion angle in radians.
    pub flexion: f32,
    /// Carrying angle in radians (valgus angle).
    pub carrying_angle: f32,
    /// Skin fold intensity when flexed, 0..=1.
    pub fold_intensity: f32,
    /// Body fat coverage, 0..=1.
    pub fat_cover: f32,
    /// Epicondyle width scale.
    pub width_scale: f32,
}

impl Default for ElbowParams {
    fn default() -> Self {
        Self {
            olecranon_prominence: 0.5,
            flexion: 0.0,
            carrying_angle: 0.1,
            fold_intensity: 0.5,
            fat_cover: 0.3,
            width_scale: 1.0,
        }
    }
}

/// Elbow evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ElbowResult {
    pub displacements: Vec<(usize, [f32; 3])>,
    pub flexion_corrective: f32,
    pub fold_depth: f32,
}

/// Flexion corrective (smoothstep).
#[allow(dead_code)]
pub fn flexion_corrective(angle: f32) -> f32 {
    let t = (angle / PI).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Olecranon bump profile.
///
/// `angle_around` is the angle around the elbow axis (0 = posterior).
#[allow(dead_code)]
pub fn olecranon_profile(angle_around: f32, prominence: f32, fat: f32) -> f32 {
    let eff = (prominence * (1.0 - fat * 0.7)).max(0.0);
    let angular = 0.5 * (1.0 + angle_around.cos());
    eff * 0.008 * angular
}

/// Skin fold in the antecubital fossa (inner elbow crease).
#[allow(dead_code)]
pub fn skin_fold(angle_around: f32, flexion: f32, intensity: f32) -> f32 {
    let flex_factor = (flexion / (PI * 0.75)).clamp(0.0, 1.0);
    // Fold is at ~PI (anterior side)
    let diff = (angle_around - PI).abs();
    let diff = if diff > PI { 2.0 * PI - diff } else { diff };
    let angular = if diff < 0.5 { 1.0 - diff / 0.5 } else { 0.0 };
    -intensity * 0.003 * flex_factor * angular
}

/// Epicondyle width modifier.
#[allow(dead_code)]
pub fn epicondyle_width(height_t: f32, width_scale: f32) -> f32 {
    let t = height_t.clamp(0.0, 1.0);
    // Width peaks at the joint line (t = 0.5)
    let profile = (-((t - 0.5) / 0.1).powi(2)).exp();
    (width_scale - 1.0) * 0.01 * profile
}

/// Carrying angle offset — lateral displacement of forearm.
#[allow(dead_code)]
pub fn carrying_offset(t_below: f32, carrying_angle: f32) -> f32 {
    let t = t_below.clamp(0.0, 1.0);
    carrying_angle.sin() * t * 0.02
}

/// Evaluate elbow morph.
///
/// `elbow_coords`: `(height_t, angle_around, radial_dist)` per vertex.
#[allow(dead_code)]
pub fn evaluate_elbow(
    elbow_coords: &[(f32, f32, f32)],
    params: &ElbowParams,
) -> ElbowResult {
    let flex_w = flexion_corrective(params.flexion);
    let mut disps = Vec::with_capacity(elbow_coords.len());
    let mut max_fold = 0.0_f32;

    for (i, &(ht, angle, _r)) in elbow_coords.iter().enumerate() {
        let olec = olecranon_profile(angle, params.olecranon_prominence, params.fat_cover);
        let fold = skin_fold(angle, params.flexion, params.fold_intensity);
        let epi = epicondyle_width(ht, params.width_scale);

        let radial = olec + fold + epi;
        max_fold = max_fold.max(fold.abs());

        if radial.abs() > 1e-7 {
            let nx = angle.cos();
            let nz = angle.sin();
            disps.push((i, [nx * radial, 0.0, nz * radial]));
        }
    }

    ElbowResult {
        displacements: disps,
        flexion_corrective: flex_w,
        fold_depth: max_fold,
    }
}

/// Blend elbow params.
#[allow(dead_code)]
pub fn blend_elbow_params(a: &ElbowParams, b: &ElbowParams, t: f32) -> ElbowParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    ElbowParams {
        olecranon_prominence: a.olecranon_prominence * inv + b.olecranon_prominence * t,
        flexion: a.flexion * inv + b.flexion * t,
        carrying_angle: a.carrying_angle * inv + b.carrying_angle * t,
        fold_intensity: a.fold_intensity * inv + b.fold_intensity * t,
        fat_cover: a.fat_cover * inv + b.fat_cover * t,
        width_scale: a.width_scale * inv + b.width_scale * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = ElbowParams::default();
        assert!((0.0..=1.0).contains(&p.olecranon_prominence));
    }

    #[test]
    fn test_flexion_corrective_zero() {
        assert!(flexion_corrective(0.0).abs() < 1e-5);
    }

    #[test]
    fn test_flexion_corrective_full() {
        let w = flexion_corrective(PI);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_olecranon_max_posterior() {
        let v = olecranon_profile(0.0, 1.0, 0.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_skin_fold_no_flexion() {
        let f = skin_fold(PI, 0.0, 1.0);
        assert!(f.abs() < 1e-6);
    }

    #[test]
    fn test_epicondyle_at_joint() {
        let w = epicondyle_width(0.5, 1.2);
        assert!(w > 0.0);
    }

    #[test]
    fn test_carrying_offset_zero_angle() {
        let o = carrying_offset(0.5, 0.0);
        assert!(o.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_elbow(&[], &ElbowParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_blend_elbow_endpoints() {
        let a = ElbowParams::default();
        let b = ElbowParams { flexion: PI, ..Default::default() };
        let r0 = blend_elbow_params(&a, &b, 0.0);
        let r1 = blend_elbow_params(&a, &b, 1.0);
        assert!((r0.flexion - a.flexion).abs() < 1e-6);
        assert!((r1.flexion - b.flexion).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_produces_output() {
        let coords = vec![(0.5, 0.0, 0.03), (0.5, PI, 0.03)];
        let params = ElbowParams {
            olecranon_prominence: 1.0,
            flexion: PI / 2.0,
            fold_intensity: 1.0,
            ..Default::default()
        };
        let r = evaluate_elbow(&coords, &params);
        assert!(!r.displacements.is_empty());
    }
}
