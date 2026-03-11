// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Knee morph control — patella shape, flexion correctives,
//! popliteal fossa, and fat pad modelling.

use std::f32::consts::PI;

/// Knee morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct KneeParams {
    /// Patella (kneecap) prominence, 0..=1.
    pub patella_prominence: f32,
    /// Patella width scale.
    pub patella_width: f32,
    /// Flexion angle in radians.
    pub flexion: f32,
    /// Fat pad thickness (infrapatellar), 0..=1.
    pub fat_pad: f32,
    /// Popliteal fossa depth (back of knee), 0..=1.
    pub popliteal_depth: f32,
    /// Valgus/varus angle (knock-knee/bow-leg) in radians.
    pub valgus: f32,
}

impl Default for KneeParams {
    fn default() -> Self {
        Self {
            patella_prominence: 0.5,
            patella_width: 1.0,
            flexion: 0.0,
            fat_pad: 0.2,
            popliteal_depth: 0.3,
            valgus: 0.0,
        }
    }
}

/// Knee evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KneeResult {
    pub displacements: Vec<(usize, [f32; 3])>,
    pub flexion_corrective: f32,
    pub skin_fold_weight: f32,
}

/// Flexion corrective (smoothstep).
#[allow(dead_code)]
pub fn flexion_corrective(angle: f32) -> f32 {
    let t = (angle / PI).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Patella bump profile (front of knee).
#[allow(dead_code)]
pub fn patella_profile(angle: f32, height_t: f32, prominence: f32, width: f32) -> f32 {
    // Patella is on the anterior side (angle ~= PI)
    let angular = 0.5 * (1.0 + (angle - PI).cos());
    if angular < 0.3 {
        return 0.0;
    }
    // Height profile: patella is at the joint line
    let h_factor = (-((height_t - 0.5) / (0.15 * width)).powi(2)).exp();
    prominence * 0.01 * angular * h_factor
}

/// Popliteal fossa (back of knee depression).
#[allow(dead_code)]
pub fn popliteal_profile(angle: f32, height_t: f32, depth: f32) -> f32 {
    // Posterior at angle ~= 0
    let angular = 0.5 * (1.0 + angle.cos());
    if angular < 0.3 {
        return 0.0;
    }
    let h_factor = (-((height_t - 0.5) / 0.12).powi(2)).exp();
    -depth * 0.005 * angular * h_factor
}

/// Fat pad (infrapatellar).
#[allow(dead_code)]
pub fn fat_pad_profile(angle: f32, height_t: f32, fat: f32) -> f32 {
    let angular = 0.5 * (1.0 + (angle - PI).cos());
    if angular < 0.3 {
        return 0.0;
    }
    // Below patella
    let h_factor = if (0.6..=0.8).contains(&height_t) {
        1.0 - ((height_t - 0.7) / 0.1).abs()
    } else {
        0.0
    };
    fat * 0.004 * angular * h_factor
}

/// Skin fold weight behind the knee during flexion.
#[allow(dead_code)]
pub fn skin_fold_weight(flexion: f32) -> f32 {
    let t = (flexion / (PI * 0.75)).clamp(0.0, 1.0);
    t * t
}

/// Evaluate knee morph.
///
/// `knee_coords`: `(height_t, angle_around, radial_dist)` per vertex.
#[allow(dead_code)]
pub fn evaluate_knee(knee_coords: &[(f32, f32, f32)], params: &KneeParams) -> KneeResult {
    let flex_w = flexion_corrective(params.flexion);
    let fold_w = skin_fold_weight(params.flexion);
    let mut disps = Vec::with_capacity(knee_coords.len());

    for (i, &(ht, angle, _r)) in knee_coords.iter().enumerate() {
        let pat = patella_profile(angle, ht, params.patella_prominence, params.patella_width);
        let pop = popliteal_profile(angle, ht, params.popliteal_depth);
        let fp = fat_pad_profile(angle, ht, params.fat_pad);

        let radial = pat + pop + fp;
        if radial.abs() > 1e-7 {
            let nx = angle.cos();
            let nz = angle.sin();
            disps.push((i, [nx * radial, 0.0, nz * radial]));
        }
    }

    KneeResult {
        displacements: disps,
        flexion_corrective: flex_w,
        skin_fold_weight: fold_w,
    }
}

/// Blend knee params.
#[allow(dead_code)]
pub fn blend_knee_params(a: &KneeParams, b: &KneeParams, t: f32) -> KneeParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    KneeParams {
        patella_prominence: a.patella_prominence * inv + b.patella_prominence * t,
        patella_width: a.patella_width * inv + b.patella_width * t,
        flexion: a.flexion * inv + b.flexion * t,
        fat_pad: a.fat_pad * inv + b.fat_pad * t,
        popliteal_depth: a.popliteal_depth * inv + b.popliteal_depth * t,
        valgus: a.valgus * inv + b.valgus * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = KneeParams::default();
        assert!((0.0..=1.0).contains(&p.patella_prominence));
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
    fn test_patella_anterior() {
        let p = patella_profile(PI, 0.5, 1.0, 1.0);
        assert!(p > 0.0);
    }

    #[test]
    fn test_patella_posterior() {
        let p = patella_profile(0.0, 0.5, 1.0, 1.0);
        assert!(p.abs() < 1e-6);
    }

    #[test]
    fn test_popliteal_posterior() {
        let p = popliteal_profile(0.0, 0.5, 1.0);
        assert!(p < 0.0);
    }

    #[test]
    fn test_fat_pad_below_patella() {
        let f = fat_pad_profile(PI, 0.7, 1.0);
        assert!(f > 0.0);
    }

    #[test]
    fn test_skin_fold_weight_zero() {
        assert!(skin_fold_weight(0.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_knee(&[], &KneeParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_blend_knee_midpoint() {
        let a = KneeParams {
            patella_prominence: 0.0,
            ..Default::default()
        };
        let b = KneeParams {
            patella_prominence: 1.0,
            ..Default::default()
        };
        let r = blend_knee_params(&a, &b, 0.5);
        assert!((r.patella_prominence - 0.5).abs() < 1e-6);
    }
}
