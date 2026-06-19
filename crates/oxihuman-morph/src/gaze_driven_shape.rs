// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Gaze direction driven morph stub.

/// Gaze direction input in spherical coordinates.
#[derive(Debug, Clone, Copy)]
pub struct GazeDirection {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for GazeDirection {
    fn default() -> Self {
        GazeDirection {
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

/// Gaze-driven shape controller.
#[derive(Debug, Clone)]
pub struct GazeDrivenShape {
    pub direction: GazeDirection,
    pub morph_count: usize,
    pub yaw_gain: f32,
    pub pitch_gain: f32,
    pub enabled: bool,
}

impl GazeDrivenShape {
    pub fn new(morph_count: usize) -> Self {
        GazeDrivenShape {
            direction: GazeDirection::default(),
            morph_count,
            yaw_gain: 1.0,
            pitch_gain: 1.0,
            enabled: true,
        }
    }
}

/// Create a new gaze-driven shape controller.
pub fn new_gaze_driven_shape(morph_count: usize) -> GazeDrivenShape {
    GazeDrivenShape::new(morph_count)
}

/// Update gaze direction.
pub fn gds_set_direction(gds: &mut GazeDrivenShape, direction: GazeDirection) {
    gds.direction = direction;
}

/// Evaluate morph weights from the current gaze direction.
///
/// The output vector has length `morph_count`.  When there are at least four
/// morph targets they are assigned canonical gaze directions:
///
/// | Index | Meaning     | Driven by           |
/// |-------|-------------|---------------------|
/// | 0     | Look left   | yaw < 0  → magnitude |
/// | 1     | Look right  | yaw > 0  → magnitude |
/// | 2     | Look up     | pitch > 0 → magnitude |
/// | 3     | Look down   | pitch < 0 → magnitude |
///
/// When fewer than four morph targets are available the entire vector is
/// scaled uniformly by `‖(yaw_weight, pitch_weight)‖ / morph_count`.
///
/// The function is disabled-aware: when `gds.enabled` is `false` it returns
/// an all-zero vector, matching the behaviour of other disabled controllers.
pub fn gds_evaluate(gds: &GazeDrivenShape) -> Vec<f32> {
    if !gds.enabled || gds.morph_count == 0 {
        return vec![0.0_f32; gds.morph_count];
    }

    let yaw = gds.direction.yaw.clamp(-1.0_f32, 1.0_f32);
    let pitch = gds.direction.pitch.clamp(-1.0_f32, 1.0_f32);

    let yaw_weight = yaw * gds.yaw_gain;
    let pitch_weight = pitch * gds.pitch_gain;

    let mut weights = vec![0.0_f32; gds.morph_count];

    if gds.morph_count >= 4 {
        // Directional four-channel decomposition.
        // Index 0: look-left  (yaw is negative → viewer's left)
        weights[0] = (-yaw_weight).max(0.0_f32).clamp(0.0_f32, 1.0_f32);
        // Index 1: look-right (yaw is positive → viewer's right)
        weights[1] = yaw_weight.max(0.0_f32).clamp(0.0_f32, 1.0_f32);
        // Index 2: look-up    (pitch is positive → upward)
        weights[2] = pitch_weight.max(0.0_f32).clamp(0.0_f32, 1.0_f32);
        // Index 3: look-down  (pitch is negative → downward)
        weights[3] = (-pitch_weight).max(0.0_f32).clamp(0.0_f32, 1.0_f32);
    } else {
        // Scalar fallback: distribute the combined magnitude uniformly.
        let magnitude = (yaw_weight * yaw_weight + pitch_weight * pitch_weight)
            .sqrt()
            .clamp(0.0_f32, 1.0_f32);
        let per_morph = magnitude / gds.morph_count as f32;
        for w in weights.iter_mut() {
            *w = per_morph;
        }
    }

    weights
}

/// Set yaw and pitch gains.
pub fn gds_set_gains(gds: &mut GazeDrivenShape, yaw_gain: f32, pitch_gain: f32) {
    gds.yaw_gain = yaw_gain;
    gds.pitch_gain = pitch_gain;
}

/// Enable or disable.
pub fn gds_set_enabled(gds: &mut GazeDrivenShape, enabled: bool) {
    gds.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn gds_to_json(gds: &GazeDrivenShape) -> String {
    format!(
        r#"{{"morph_count":{},"yaw":{:.4},"pitch":{:.4},"enabled":{}}}"#,
        gds.morph_count, gds.direction.yaw, gds.direction.pitch, gds.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_count() {
        let g = new_gaze_driven_shape(4);
        assert_eq!(g.morph_count, 4 /* morph count must match */,);
    }

    #[test]
    fn test_default_direction_zero() {
        let g = new_gaze_driven_shape(4);
        assert!((g.direction.yaw).abs() < 1e-6, /* default yaw must be zero */);
        assert!((g.direction.pitch).abs() < 1e-6, /* default pitch must be zero */);
    }

    #[test]
    fn test_set_direction() {
        let mut g = new_gaze_driven_shape(4);
        gds_set_direction(
            &mut g,
            GazeDirection {
                yaw: 0.3,
                pitch: -0.1,
            },
        );
        assert!((g.direction.yaw - 0.3).abs() < 1e-5, /* yaw must be set */);
    }

    #[test]
    fn test_evaluate_length() {
        let g = new_gaze_driven_shape(6);
        let out = gds_evaluate(&g);
        assert_eq!(out.len(), 6 /* output length must match morph_count */,);
    }

    #[test]
    fn test_evaluate_zeroed() {
        // With default direction (yaw=0, pitch=0) all weights must be zero
        // regardless of how many morph targets are configured.
        let g = new_gaze_driven_shape(3);
        let out = gds_evaluate(&g);
        assert!(
            out.iter().all(|&v| v.abs() < 1e-6),
            "zero gaze must produce all-zero weights"
        );
    }

    #[test]
    fn test_set_gains() {
        let mut g = new_gaze_driven_shape(2);
        gds_set_gains(&mut g, 0.5, 2.0);
        assert!((g.yaw_gain - 0.5).abs() < 1e-5, /* yaw gain must be set */);
        assert!((g.pitch_gain - 2.0).abs() < 1e-5, /* pitch gain must be set */);
    }

    #[test]
    fn test_set_enabled() {
        let mut g = new_gaze_driven_shape(2);
        gds_set_enabled(&mut g, false);
        assert!(!g.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_morph_count() {
        let g = new_gaze_driven_shape(5);
        let j = gds_to_json(&g);
        assert!(j.contains("\"morph_count\""), /* json must contain morph_count */);
    }

    #[test]
    fn test_enabled_default() {
        let g = new_gaze_driven_shape(1);
        assert!(g.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_default_gains() {
        let g = new_gaze_driven_shape(1);
        assert!((g.yaw_gain - 1.0).abs() < 1e-5, /* default yaw gain must be 1.0 */);
        assert!((g.pitch_gain - 1.0).abs() < 1e-5, /* default pitch gain must be 1.0 */);
    }

    #[test]
    fn gds_nonzero_yaw_gives_nonzero_output() {
        // A non-zero yaw must produce at least one non-zero morph weight.
        let mut g = new_gaze_driven_shape(4);
        gds_set_direction(
            &mut g,
            GazeDirection {
                yaw: 0.5,
                pitch: 0.0,
            },
        );
        let out = gds_evaluate(&g);
        assert!(
            out.iter().any(|&v| v > 1e-6),
            "non-zero yaw must produce at least one non-zero weight"
        );
    }

    #[test]
    fn gds_zero_gaze_gives_zero_output() {
        // Default-constructed direction (yaw=0, pitch=0) must always yield zeros.
        let mut g = new_gaze_driven_shape(6);
        gds_set_direction(
            &mut g,
            GazeDirection {
                yaw: 0.0,
                pitch: 0.0,
            },
        );
        let out = gds_evaluate(&g);
        assert!(
            out.iter().all(|&v| v.abs() < 1e-6),
            "zero gaze direction must produce all-zero weights"
        );
    }
}
