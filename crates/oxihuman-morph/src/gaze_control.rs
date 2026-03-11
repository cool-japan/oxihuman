#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Gaze direction control (eye tracking).

use std::f32::consts::FRAC_PI_2;

/// Gaze direction control parameters.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct GazeControl {
    /// Horizontal rotation in radians (positive = right).
    pub yaw: f32,
    /// Vertical rotation in radians (positive = up).
    pub pitch: f32,
    /// Convergence blend [0, 1] (0 = parallel, 1 = fully converged).
    pub convergence: f32,
    /// Blink synchronisation [0, 1].
    pub blink_sync: f32,
}

/// Create a default `GazeControl` facing straight ahead.
#[allow(dead_code)]
pub fn default_gaze_control() -> GazeControl {
    GazeControl {
        yaw: 0.0,
        pitch: 0.0,
        convergence: 0.0,
        blink_sync: 0.0,
    }
}

/// Apply gaze control to a morph-weight slice.
///
/// Expects at least 4 elements: `[yaw_weight, pitch_weight, convergence, blink_sync]`.
/// Yaw and pitch are normalised to [−1, 1] relative to ±π/2.
#[allow(dead_code)]
pub fn apply_gaze(weights: &mut [f32], gc: &GazeControl) {
    let yaw_w = (gc.yaw / FRAC_PI_2).clamp(-1.0, 1.0);
    let pitch_w = (gc.pitch / FRAC_PI_2).clamp(-1.0, 1.0);
    if !weights.is_empty() {
        weights[0] = yaw_w;
    }
    if weights.len() >= 2 {
        weights[1] = pitch_w;
    }
    if weights.len() >= 3 {
        weights[2] = gc.convergence.clamp(0.0, 1.0);
    }
    if weights.len() >= 4 {
        weights[3] = gc.blink_sync.clamp(0.0, 1.0);
    }
}

/// Linearly blend two `GazeControl` structs.
#[allow(dead_code)]
pub fn gaze_blend(a: &GazeControl, b: &GazeControl, t: f32) -> GazeControl {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: f32, y: f32| x + (y - x) * t;
    GazeControl {
        yaw: lerp(a.yaw, b.yaw),
        pitch: lerp(a.pitch, b.pitch),
        convergence: lerp(a.convergence, b.convergence),
        blink_sync: lerp(a.blink_sync, b.blink_sync),
    }
}

/// Compute the angular deviation of the gaze from straight ahead (in radians).
#[allow(dead_code)]
pub fn gaze_deviation(gc: &GazeControl) -> f32 {
    (gc.yaw * gc.yaw + gc.pitch * gc.pitch).sqrt()
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_gaze_is_straight() {
        let gc = default_gaze_control();
        assert!((gc.yaw).abs() < 1e-9);
        assert!((gc.pitch).abs() < 1e-9);
        assert!((gaze_deviation(&gc)).abs() < 1e-9);
    }

    #[test]
    fn gaze_deviation_diagonal() {
        let gc = GazeControl { yaw: 0.3, pitch: 0.4, convergence: 0.0, blink_sync: 0.0 };
        assert!((gaze_deviation(&gc) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn apply_gaze_sets_weights() {
        let gc = GazeControl { yaw: FRAC_PI_2, pitch: 0.0, convergence: 0.5, blink_sync: 0.3 };
        let mut w = vec![0.0_f32; 4];
        apply_gaze(&mut w, &gc);
        assert!((w[0] - 1.0).abs() < 1e-5);
        assert!((w[2] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn apply_gaze_short_slice() {
        let gc = default_gaze_control();
        let mut w: Vec<f32> = Vec::new();
        apply_gaze(&mut w, &gc); // must not panic
    }

    #[test]
    fn gaze_blend_at_zero_returns_a() {
        let a = default_gaze_control();
        let b = GazeControl { yaw: 1.0, pitch: 0.5, convergence: 1.0, blink_sync: 1.0 };
        let r = gaze_blend(&a, &b, 0.0);
        assert!((r.yaw - a.yaw).abs() < 1e-6);
    }

    #[test]
    fn gaze_blend_at_one_returns_b() {
        let a = default_gaze_control();
        let b = GazeControl { yaw: 1.0, pitch: 0.5, convergence: 1.0, blink_sync: 1.0 };
        let r = gaze_blend(&a, &b, 1.0);
        assert!((r.yaw - b.yaw).abs() < 1e-6);
    }

    #[test]
    fn gaze_blend_midpoint() {
        let a = GazeControl { yaw: 0.0, pitch: 0.0, convergence: 0.0, blink_sync: 0.0 };
        let b = GazeControl { yaw: 1.0, pitch: 1.0, convergence: 1.0, blink_sync: 1.0 };
        let r = gaze_blend(&a, &b, 0.5);
        assert!((r.yaw - 0.5).abs() < 1e-5);
    }

    #[test]
    fn apply_gaze_clamps_yaw() {
        let gc = GazeControl { yaw: 100.0, pitch: 0.0, convergence: 0.0, blink_sync: 0.0 };
        let mut w = vec![0.0_f32; 4];
        apply_gaze(&mut w, &gc);
        assert!((0.0..=1.0).contains(&w[0]));
    }

    #[test]
    fn gaze_deviation_zero() {
        let gc = default_gaze_control();
        assert!((gaze_deviation(&gc)).abs() < 1e-9);
    }

    #[test]
    fn gaze_deviation_positive() {
        let gc = GazeControl { yaw: 0.5, pitch: 0.0, convergence: 0.0, blink_sync: 0.0 };
        assert!(gaze_deviation(&gc) > 0.0);
    }
}
