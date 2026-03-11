// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Foot arch morph control — models the longitudinal and transverse arches
//! of the foot for flat-foot / high-arch variation.

use std::f32::consts::PI;

/// Foot arch parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ArchParams {
    /// Longitudinal arch height, 0 = flat, 1 = high arch.
    pub longitudinal_height: f32,
    /// Transverse arch curvature, 0 = flat, 1 = pronounced.
    pub transverse_curvature: f32,
    /// Pronation/supination angle in radians (positive = pronation).
    pub pronation: f32,
    /// Overall foot width multiplier.
    pub width_scale: f32,
}

impl Default for ArchParams {
    fn default() -> Self {
        Self {
            longitudinal_height: 0.5,
            transverse_curvature: 0.5,
            pronation: 0.0,
            width_scale: 1.0,
        }
    }
}

/// Arch evaluation result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArchResult {
    /// Per-vertex Y displacements for arch shape.
    pub displacements: Vec<(usize, f32)>,
    /// The computed arch index (ratio of midfoot to full contact).
    pub arch_index: f32,
}

/// Longitudinal arch profile along the foot length axis.
///
/// `t` goes from 0 (heel) to 1 (toe), returns height offset.
#[allow(dead_code)]
pub fn longitudinal_profile(t: f32, height: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    // Arch peaks around 40% of foot length
    let peak = 0.4;
    let width = 0.3;
    let normalized = ((t - peak) / width).clamp(-1.0, 1.0);
    let envelope = 0.5 * (1.0 + (PI * normalized).cos());
    height * 0.02 * envelope
}

/// Transverse arch profile across the foot width.
///
/// `s` goes from -1 (lateral) to 1 (medial).
#[allow(dead_code)]
pub fn transverse_profile(s: f32, curvature: f32) -> f32 {
    let s = s.clamp(-1.0, 1.0);
    curvature * 0.005 * (1.0 - s * s)
}

/// Pronation rotation matrix (around Z axis in foot-local space).
///
/// Returns a 2D rotation for the XY plane of the foot cross-section.
#[allow(dead_code)]
pub fn pronation_rotation(angle: f32) -> [[f32; 2]; 2] {
    let c = angle.cos();
    let s = angle.sin();
    [[c, -s], [s, c]]
}

/// Apply pronation to a cross-section point.
#[allow(dead_code)]
pub fn apply_pronation(x: f32, y: f32, angle: f32) -> (f32, f32) {
    let rot = pronation_rotation(angle);
    let nx = rot[0][0] * x + rot[0][1] * y;
    let ny = rot[1][0] * x + rot[1][1] * y;
    (nx, ny)
}

/// Classify arch type from the longitudinal height parameter.
#[allow(dead_code)]
pub fn classify_arch(longitudinal_height: f32) -> &'static str {
    let h = longitudinal_height.clamp(0.0, 1.0);
    if h < 0.25 {
        "flat"
    } else if h < 0.6 {
        "normal"
    } else {
        "high"
    }
}

/// Compute the arch index: ratio of midfoot contact to total contact.
/// Lower values → higher arch.
#[allow(dead_code)]
pub fn compute_arch_index(longitudinal_height: f32) -> f32 {
    let h = longitudinal_height.clamp(0.0, 1.0);
    1.0 - h * 0.6
}

/// Evaluate arch morph for foot vertices.
///
/// `foot_uvs` are `(length_t, width_s)` parametric coords per vertex.
#[allow(dead_code)]
pub fn evaluate_arch(foot_uvs: &[(f32, f32)], params: &ArchParams) -> ArchResult {
    let mut displacements = Vec::with_capacity(foot_uvs.len());

    for (i, &(length_t, width_s)) in foot_uvs.iter().enumerate() {
        let long = longitudinal_profile(length_t, params.longitudinal_height);
        let trans = transverse_profile(width_s, params.transverse_curvature);
        let total = long + trans;
        if total.abs() > 1e-7 {
            displacements.push((i, total));
        }
    }

    let arch_index = compute_arch_index(params.longitudinal_height);

    ArchResult {
        displacements,
        arch_index,
    }
}

/// Blend two ArchParams.
#[allow(dead_code)]
pub fn blend_arch_params(a: &ArchParams, b: &ArchParams, t: f32) -> ArchParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    ArchParams {
        longitudinal_height: a.longitudinal_height * inv + b.longitudinal_height * t,
        transverse_curvature: a.transverse_curvature * inv + b.transverse_curvature * t,
        pronation: a.pronation * inv + b.pronation * t,
        width_scale: a.width_scale * inv + b.width_scale * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let p = ArchParams::default();
        assert!((p.longitudinal_height - 0.5).abs() < 1e-6);
        assert!((p.width_scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_longitudinal_profile_peak() {
        let v = longitudinal_profile(0.4, 1.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_longitudinal_profile_endpoints() {
        let heel = longitudinal_profile(0.0, 1.0);
        let toe = longitudinal_profile(1.0, 1.0);
        let mid = longitudinal_profile(0.4, 1.0);
        assert!(mid > heel);
        assert!(mid > toe);
    }

    #[test]
    fn test_transverse_profile_centre() {
        let c = transverse_profile(0.0, 1.0);
        assert!(c > 0.0);
    }

    #[test]
    fn test_transverse_profile_edges() {
        let edge = transverse_profile(1.0, 1.0);
        assert!(edge.abs() < 1e-6);
    }

    #[test]
    fn test_classify_arch() {
        assert_eq!(classify_arch(0.1), "flat");
        assert_eq!(classify_arch(0.4), "normal");
        assert_eq!(classify_arch(0.9), "high");
    }

    #[test]
    fn test_arch_index() {
        let high = compute_arch_index(1.0);
        let flat = compute_arch_index(0.0);
        assert!(flat > high);
    }

    #[test]
    fn test_evaluate_arch_empty() {
        let result = evaluate_arch(&[], &ArchParams::default());
        assert!(result.displacements.is_empty());
    }

    #[test]
    fn test_pronation_identity() {
        let (x, y) = apply_pronation(1.0, 0.0, 0.0);
        assert!((x - 1.0).abs() < 1e-5);
        assert!(y.abs() < 1e-5);
    }

    #[test]
    fn test_blend_arch_params() {
        let a = ArchParams::default();
        let b = ArchParams { longitudinal_height: 1.0, ..Default::default() };
        let r = blend_arch_params(&a, &b, 0.5);
        assert!((r.longitudinal_height - 0.75).abs() < 1e-5);
    }
}
