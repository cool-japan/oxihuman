// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Wrist morph control — ulnar/radial styloid prominence,
//! flexion/extension correctives, and tendon visibility.

use std::f32::consts::PI;

/// Wrist morph parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct WristParams {
    /// Overall thickness, 0.5..=1.5.
    pub thickness: f32,
    /// Ulnar styloid prominence (pinky side), 0..=1.
    pub ulnar_prominence: f32,
    /// Radial styloid prominence (thumb side), 0..=1.
    pub radial_prominence: f32,
    /// Tendon visibility (extensor tendons), 0..=1.
    pub tendon_visibility: f32,
    /// Flexion angle in radians (palm down).
    pub flexion: f32,
    /// Ulnar/radial deviation in radians.
    pub deviation: f32,
}

impl Default for WristParams {
    fn default() -> Self {
        Self {
            thickness: 1.0,
            ulnar_prominence: 0.4,
            radial_prominence: 0.3,
            tendon_visibility: 0.2,
            flexion: 0.0,
            deviation: 0.0,
        }
    }
}

/// Wrist evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WristResult {
    pub displacements: Vec<(usize, f32)>,
    pub flexion_corrective: f32,
    pub deviation_corrective: f32,
}

/// Flexion corrective weight.
#[allow(dead_code)]
pub fn flexion_weight(angle: f32) -> f32 {
    let t = (angle.abs() / (PI / 3.0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Deviation corrective weight.
#[allow(dead_code)]
pub fn deviation_weight(angle: f32) -> f32 {
    let t = (angle.abs() / (PI / 6.0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Styloid prominence bump.
#[allow(dead_code)]
pub fn styloid_bump(angle: f32, centre: f32, prominence: f32) -> f32 {
    let diff = (angle - centre).abs();
    let diff = if diff > PI { 2.0 * PI - diff } else { diff };
    if diff < 0.4 {
        prominence * 0.006 * (1.0 - diff / 0.4)
    } else {
        0.0
    }
}

/// Tendon ridge pattern on dorsal side.
#[allow(dead_code)]
pub fn tendon_ridge(angle: f32, along_t: f32, visibility: f32) -> f32 {
    if visibility < 0.05 {
        return 0.0;
    }
    // Dorsal side at angle ~= PI
    let dorsal = 0.5 * (1.0 + (angle - PI).cos());
    if dorsal < 0.3 {
        return 0.0;
    }
    // Multiple tendon ridges
    let ridges = ((along_t * 6.0) * PI).cos().abs();
    visibility * 0.002 * dorsal * ridges
}

/// Evaluate wrist morph.
///
/// `wrist_coords`: `(along_t, angle_around, radius)` per vertex.
#[allow(dead_code)]
pub fn evaluate_wrist(wrist_coords: &[(f32, f32, f32)], params: &WristParams) -> WristResult {
    let flex_w = flexion_weight(params.flexion);
    let dev_w = deviation_weight(params.deviation);
    let mut displacements = Vec::with_capacity(wrist_coords.len());

    for (i, &(along, angle, _r)) in wrist_coords.iter().enumerate() {
        let thick = (params.thickness - 1.0) * 0.005;
        let ulnar = styloid_bump(angle, PI / 2.0, params.ulnar_prominence);
        let radial = styloid_bump(angle, -PI / 2.0, params.radial_prominence);
        let tendon = tendon_ridge(angle, along, params.tendon_visibility);

        let height_falloff = (-((along - 0.5) / 0.3).powi(2)).exp();
        let disp = (thick + ulnar + radial + tendon) * height_falloff;

        if disp.abs() > 1e-7 {
            displacements.push((i, disp));
        }
    }

    WristResult {
        displacements,
        flexion_corrective: flex_w,
        deviation_corrective: dev_w,
    }
}

/// Blend wrist params.
#[allow(dead_code)]
pub fn blend_wrist_params(a: &WristParams, b: &WristParams, t: f32) -> WristParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    WristParams {
        thickness: a.thickness * inv + b.thickness * t,
        ulnar_prominence: a.ulnar_prominence * inv + b.ulnar_prominence * t,
        radial_prominence: a.radial_prominence * inv + b.radial_prominence * t,
        tendon_visibility: a.tendon_visibility * inv + b.tendon_visibility * t,
        flexion: a.flexion * inv + b.flexion * t,
        deviation: a.deviation * inv + b.deviation * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_params() {
        let p = WristParams::default();
        assert!((p.thickness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_flexion_weight_zero() {
        assert!(flexion_weight(0.0).abs() < 1e-5);
    }

    #[test]
    fn test_flexion_weight_max() {
        let w = flexion_weight(PI / 3.0);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_deviation_weight_zero() {
        assert!(deviation_weight(0.0).abs() < 1e-5);
    }

    #[test]
    fn test_styloid_bump_at_centre() {
        let b = styloid_bump(PI / 2.0, PI / 2.0, 1.0);
        assert!(b > 0.0);
    }

    #[test]
    fn test_styloid_bump_far_away() {
        let b = styloid_bump(0.0, PI, 1.0);
        assert!(b.abs() < 1e-6);
    }

    #[test]
    fn test_tendon_ridge_zero_vis() {
        let t = tendon_ridge(PI, 0.5, 0.0);
        assert!(t.abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_wrist(&[], &WristParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_blend_wrist_midpoint() {
        let a = WristParams {
            thickness: 0.8,
            ..Default::default()
        };
        let b = WristParams {
            thickness: 1.2,
            ..Default::default()
        };
        let r = blend_wrist_params(&a, &b, 0.5);
        assert!((r.thickness - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_produces_output() {
        let coords = vec![(0.5, PI, 0.02), (0.5, PI / 2.0, 0.02)];
        let params = WristParams {
            ulnar_prominence: 1.0,
            ..Default::default()
        };
        let r = evaluate_wrist(&coords, &params);
        assert!(!r.displacements.is_empty());
    }
}
