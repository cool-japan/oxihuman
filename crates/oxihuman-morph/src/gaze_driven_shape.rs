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

/// Evaluate morph weights from current gaze (stub: uniform distribution).
pub fn gds_evaluate(gds: &GazeDrivenShape) -> Vec<f32> {
    /* Stub: returns zeroed weights */
    vec![0.0; gds.morph_count]
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
        let g = new_gaze_driven_shape(3);
        let out = gds_evaluate(&g);
        assert!(out.iter().all(|&v| v.abs() < 1e-6), /* stub must return zeros */);
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
}
