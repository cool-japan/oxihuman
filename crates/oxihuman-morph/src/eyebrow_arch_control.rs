// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Eyebrow arch morph control — arch height, peak position,
//! thickness, and tilt for detailed brow shaping.

use std::f32::consts::PI;

/// Eyebrow arch parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct EyebrowArchParams {
    /// Arch height, 0 = flat, 1 = very arched.
    pub arch_height: f32,
    /// Horizontal position of the arch peak, 0..=1 (medial to lateral).
    pub peak_position: f32,
    /// Brow thickness, 0..=1.
    pub thickness: f32,
    /// Tilt angle in radians (positive = lateral end raised).
    pub tilt: f32,
    /// Inner (medial) end height offset, -1..=1.
    pub inner_height: f32,
    /// Outer (lateral) end height offset, -1..=1.
    pub outer_height: f32,
}

impl Default for EyebrowArchParams {
    fn default() -> Self {
        Self {
            arch_height: 0.5,
            peak_position: 0.65,
            thickness: 0.5,
            tilt: 0.0,
            inner_height: 0.0,
            outer_height: 0.0,
        }
    }
}

/// Result of eyebrow arch evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EyebrowArchResult {
    /// Vertical displacements per vertex (index, dy).
    pub displacements: Vec<(usize, f32)>,
    /// Computed arch curvature (peak height - average endpoint height).
    pub curvature: f32,
}

/// Arch height curve along the brow.
///
/// `t` from 0 (inner/medial end) to 1 (outer/lateral end).
#[allow(dead_code)]
pub fn arch_curve(t: f32, peak_pos: f32, arch_height: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let peak_pos = peak_pos.clamp(0.1, 0.9);
    // Quadratic arch with adjustable peak position
    let a = if t <= peak_pos {
        t / peak_pos
    } else {
        (1.0 - t) / (1.0 - peak_pos)
    };
    arch_height * a * 0.01
}

/// Tilt offset — linear height change across the brow.
#[allow(dead_code)]
pub fn tilt_offset(t: f32, tilt: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    // Centre at 0.5
    (t - 0.5) * tilt.sin() * 0.005
}

/// Endpoint height adjustments.
#[allow(dead_code)]
pub fn endpoint_offset(t: f32, inner: f32, outer: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let inner_w = 1.0 - t;
    let outer_w = t;
    // Fade influence: only strong at endpoints
    let inner_fade = if t < 0.3 { 1.0 - t / 0.3 } else { 0.0 };
    let outer_fade = if t > 0.7 { (t - 0.7) / 0.3 } else { 0.0 };
    let _ = inner_w;
    let _ = outer_w;
    inner * inner_fade * 0.005 + outer * outer_fade * 0.005
}

/// Thickness displacement (vertical spread above/below brow line).
#[allow(dead_code)]
pub fn thickness_displacement(vertical_offset: f32, thickness: f32) -> f32 {
    let v = vertical_offset.abs();
    let half_thick = thickness * 0.005;
    if v < half_thick {
        1.0 - v / half_thick
    } else {
        0.0
    }
}

/// Evaluate the eyebrow arch morph.
///
/// `brow_coords`: per-vertex `(along_t, vertical_offset_from_brow_line)`.
#[allow(dead_code)]
pub fn evaluate_eyebrow_arch(
    brow_coords: &[(f32, f32)],
    params: &EyebrowArchParams,
) -> EyebrowArchResult {
    let mut displacements = Vec::with_capacity(brow_coords.len());

    for (i, &(t, v_off)) in brow_coords.iter().enumerate() {
        let arch = arch_curve(t, params.peak_position, params.arch_height);
        let tilt = tilt_offset(t, params.tilt);
        let endpoint = endpoint_offset(t, params.inner_height, params.outer_height);
        let thick_w = thickness_displacement(v_off, params.thickness);

        let dy = (arch + tilt + endpoint) * thick_w;
        if dy.abs() > 1e-7 {
            displacements.push((i, dy));
        }
    }

    let inner_h = arch_curve(0.0, params.peak_position, params.arch_height);
    let outer_h = arch_curve(1.0, params.peak_position, params.arch_height);
    let peak_h = arch_curve(params.peak_position, params.peak_position, params.arch_height);
    let curvature = peak_h - (inner_h + outer_h) * 0.5;

    EyebrowArchResult {
        displacements,
        curvature,
    }
}

/// Blend eyebrow arch params.
#[allow(dead_code)]
pub fn blend_eyebrow_arch(a: &EyebrowArchParams, b: &EyebrowArchParams, t: f32) -> EyebrowArchParams {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    EyebrowArchParams {
        arch_height: a.arch_height * inv + b.arch_height * t,
        peak_position: a.peak_position * inv + b.peak_position * t,
        thickness: a.thickness * inv + b.thickness * t,
        tilt: a.tilt * inv + b.tilt * t,
        inner_height: a.inner_height * inv + b.inner_height * t,
        outer_height: a.outer_height * inv + b.outer_height * t,
    }
}

/// Mirror eyebrow arch (swap inner/outer).
#[allow(dead_code)]
pub fn mirror_eyebrow_arch(params: &EyebrowArchParams) -> EyebrowArchParams {
    EyebrowArchParams {
        peak_position: 1.0 - params.peak_position,
        tilt: -params.tilt,
        inner_height: params.outer_height,
        outer_height: params.inner_height,
        ..params.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let p = EyebrowArchParams::default();
        assert!((0.0..=1.0).contains(&p.arch_height));
        assert!((0.0..=1.0).contains(&p.peak_position));
    }

    #[test]
    fn test_arch_curve_peak() {
        let v = arch_curve(0.65, 0.65, 1.0);
        assert!(v > 0.0);
    }

    #[test]
    fn test_arch_curve_endpoints() {
        let inner = arch_curve(0.0, 0.65, 1.0);
        let outer = arch_curve(1.0, 0.65, 1.0);
        assert!(inner.abs() < 1e-6);
        assert!(outer.abs() < 1e-6);
    }

    #[test]
    fn test_tilt_offset_zero() {
        let o = tilt_offset(0.5, 0.0);
        assert!(o.abs() < 1e-6);
    }

    #[test]
    fn test_tilt_offset_symmetry() {
        let left = tilt_offset(0.0, 0.5);
        let right = tilt_offset(1.0, 0.5);
        assert!((left + right).abs() < 1e-6);
    }

    #[test]
    fn test_thickness_inside() {
        let w = thickness_displacement(0.0, 0.5);
        assert!((w - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_thickness_outside() {
        let w = thickness_displacement(1.0, 0.5);
        assert!(w.abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_empty() {
        let r = evaluate_eyebrow_arch(&[], &EyebrowArchParams::default());
        assert!(r.displacements.is_empty());
    }

    #[test]
    fn test_mirror_peak_position() {
        let p = EyebrowArchParams { peak_position: 0.7, ..Default::default() };
        let m = mirror_eyebrow_arch(&p);
        assert!((m.peak_position - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_blend_eyebrow_arch() {
        let a = EyebrowArchParams { arch_height: 0.0, ..Default::default() };
        let b = EyebrowArchParams { arch_height: 1.0, ..Default::default() };
        let r = blend_eyebrow_arch(&a, &b, 0.5);
        assert!((r.arch_height - 0.5).abs() < 1e-6);
    }
}
