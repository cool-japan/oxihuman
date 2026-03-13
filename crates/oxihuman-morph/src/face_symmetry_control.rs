// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Face symmetry / asymmetry injection control.

use std::f32::consts::TAU;

/// Axes of asymmetry.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsymmetryAxis {
    Horizontal,
    Vertical,
    Depth,
}

/// A single asymmetry override.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AsymmetryEntry {
    pub axis: AsymmetryAxis,
    /// Deviation amount (-1..1).
    pub deviation: f32,
}

/// Face symmetry state.
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct FaceSymmetryState {
    pub entries: Vec<AsymmetryEntry>,
    /// Overall symmetry enforcement (1.0 = fully symmetric, 0.0 = unmodified).
    pub enforce_weight: f32,
}

/// Config.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct FaceSymmetryConfig {
    pub max_deviation: f32,
}

impl Default for FaceSymmetryConfig {
    fn default() -> Self {
        Self { max_deviation: 1.0 }
    }
}

#[allow(dead_code)]
pub fn new_face_symmetry_state() -> FaceSymmetryState {
    FaceSymmetryState::default()
}

#[allow(dead_code)]
pub fn default_face_symmetry_config() -> FaceSymmetryConfig {
    FaceSymmetryConfig::default()
}

#[allow(dead_code)]
pub fn fs_set_deviation(
    state: &mut FaceSymmetryState,
    cfg: &FaceSymmetryConfig,
    axis: AsymmetryAxis,
    v: f32,
) {
    let v = v.clamp(-cfg.max_deviation, cfg.max_deviation);
    if let Some(e) = state.entries.iter_mut().find(|e| e.axis == axis) {
        e.deviation = v;
    } else {
        state.entries.push(AsymmetryEntry { axis, deviation: v });
    }
}

#[allow(dead_code)]
pub fn fs_set_enforce(state: &mut FaceSymmetryState, w: f32) {
    state.enforce_weight = w.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn fs_get_deviation(state: &FaceSymmetryState, axis: AsymmetryAxis) -> f32 {
    state
        .entries
        .iter()
        .find(|e| e.axis == axis)
        .map(|e| e.deviation)
        .unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn fs_reset(state: &mut FaceSymmetryState) {
    state.entries.clear();
    state.enforce_weight = 0.0;
}

#[allow(dead_code)]
pub fn fs_is_symmetric(state: &FaceSymmetryState) -> bool {
    state.entries.iter().all(|e| e.deviation.abs() < 1e-4)
}

#[allow(dead_code)]
pub fn fs_total_deviation(state: &FaceSymmetryState) -> f32 {
    state.entries.iter().map(|e| e.deviation.abs()).sum()
}

#[allow(dead_code)]
pub fn fs_blend(a: &FaceSymmetryState, b: &FaceSymmetryState, t: f32) -> FaceSymmetryState {
    let t = t.clamp(0.0, 1.0);
    let axes = [
        AsymmetryAxis::Horizontal,
        AsymmetryAxis::Vertical,
        AsymmetryAxis::Depth,
    ];
    let entries = axes
        .iter()
        .map(|&ax| {
            let da = a
                .entries
                .iter()
                .find(|e| e.axis == ax)
                .map(|e| e.deviation)
                .unwrap_or(0.0);
            let db = b
                .entries
                .iter()
                .find(|e| e.axis == ax)
                .map(|e| e.deviation)
                .unwrap_or(0.0);
            AsymmetryEntry {
                axis: ax,
                deviation: da + (db - da) * t,
            }
        })
        .collect();
    FaceSymmetryState {
        entries,
        enforce_weight: a.enforce_weight + (b.enforce_weight - a.enforce_weight) * t,
    }
}

/// Circular noise for organic-feeling asymmetry (uses TAU).
#[allow(dead_code)]
pub fn fs_circular_noise(seed: f32) -> f32 {
    (seed * TAU).sin() * 0.5
}

#[allow(dead_code)]
pub fn fs_to_json(state: &FaceSymmetryState) -> String {
    let e: Vec<String> = state
        .entries
        .iter()
        .map(|en| {
            let ax = match en.axis {
                AsymmetryAxis::Horizontal => "horizontal",
                AsymmetryAxis::Vertical => "vertical",
                AsymmetryAxis::Depth => "depth",
            };
            format!("{{\"axis\":\"{}\",\"dev\":{:.4}}}", ax, en.deviation)
        })
        .collect();
    format!(
        "{{\"entries\":[{}],\"enforce\":{:.4}}}",
        e.join(","),
        state.enforce_weight
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_symmetric() {
        assert!(fs_is_symmetric(&new_face_symmetry_state()));
    }

    #[test]
    fn set_deviation_stores() {
        let mut s = new_face_symmetry_state();
        let cfg = default_face_symmetry_config();
        fs_set_deviation(&mut s, &cfg, AsymmetryAxis::Horizontal, 0.3);
        assert!((fs_get_deviation(&s, AsymmetryAxis::Horizontal) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn deviation_clamp() {
        let mut s = new_face_symmetry_state();
        let cfg = default_face_symmetry_config();
        fs_set_deviation(&mut s, &cfg, AsymmetryAxis::Vertical, 5.0);
        assert!(fs_get_deviation(&s, AsymmetryAxis::Vertical) <= cfg.max_deviation);
    }

    #[test]
    fn unknown_axis_zero() {
        let s = new_face_symmetry_state();
        assert!((fs_get_deviation(&s, AsymmetryAxis::Depth)).abs() < 1e-5);
    }

    #[test]
    fn reset_clears() {
        let mut s = new_face_symmetry_state();
        let cfg = default_face_symmetry_config();
        fs_set_deviation(&mut s, &cfg, AsymmetryAxis::Horizontal, 0.5);
        fs_reset(&mut s);
        assert!(fs_is_symmetric(&s));
    }

    #[test]
    fn total_deviation_sum() {
        let mut s = new_face_symmetry_state();
        let cfg = default_face_symmetry_config();
        fs_set_deviation(&mut s, &cfg, AsymmetryAxis::Horizontal, 0.5);
        fs_set_deviation(&mut s, &cfg, AsymmetryAxis::Vertical, 0.5);
        assert!((fs_total_deviation(&s) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn blend_midpoint() {
        let cfg = default_face_symmetry_config();
        let mut a = new_face_symmetry_state();
        let mut b = new_face_symmetry_state();
        fs_set_deviation(&mut a, &cfg, AsymmetryAxis::Depth, 0.0);
        fs_set_deviation(&mut b, &cfg, AsymmetryAxis::Depth, 1.0);
        let m = fs_blend(&a, &b, 0.5);
        let d = m
            .entries
            .iter()
            .find(|e| e.axis == AsymmetryAxis::Depth)
            .map(|e| e.deviation)
            .unwrap_or(0.0);
        assert!((d - 0.5).abs() < 1e-4);
    }

    #[test]
    fn circular_noise_bounded() {
        let v = fs_circular_noise(0.25);
        assert!((-1.0..=1.0).contains(&v));
    }

    #[test]
    fn json_contains_enforce() {
        assert!(fs_to_json(&new_face_symmetry_state()).contains("enforce"));
    }

    #[test]
    fn enforce_weight_clamped() {
        let mut s = new_face_symmetry_state();
        fs_set_enforce(&mut s, 5.0);
        assert!(s.enforce_weight <= 1.0);
    }
}
