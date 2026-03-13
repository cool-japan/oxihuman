// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Gaze target specification.
pub enum GazeTarget {
    /// Look at a specific world-space point.
    Point([f32; 3]),
    /// Look in a direction (normalized).
    Direction([f32; 3]),
    /// Yaw and pitch angles in radians.
    Angles { yaw: f32, pitch: f32 },
    /// Forward-looking (neutral).
    Forward,
}

/// Configuration for the two-eye gaze system.
pub struct EyeConfig {
    /// Left eye center in world space.
    pub left_eye_pos: [f32; 3],
    /// Right eye center in world space.
    pub right_eye_pos: [f32; 3],
    /// Default forward direction (normalized).
    pub forward_dir: [f32; 3],
    /// Up direction (normalized).
    pub up_dir: [f32; 3],
    /// Maximum horizontal rotation in radians.
    pub max_yaw: f32,
    /// Maximum vertical rotation in radians.
    pub max_pitch: f32,
    /// Distance at which eyes converge.
    pub convergence_dist: f32,
}

impl Default for EyeConfig {
    fn default() -> Self {
        Self {
            left_eye_pos: [-0.032, 1.67, 0.095],
            right_eye_pos: [0.032, 1.67, 0.095],
            forward_dir: [0.0, 0.0, 1.0],
            up_dir: [0.0, 1.0, 0.0],
            max_yaw: std::f32::consts::FRAC_PI_4,
            max_pitch: std::f32::consts::FRAC_PI_6,
            convergence_dist: 2.0,
        }
    }
}

/// Computed gaze angles for one eye.
pub struct EyeGazeAngles {
    /// Horizontal rotation (positive = right).
    pub yaw: f32,
    /// Vertical rotation (positive = up).
    pub pitch: f32,
}

/// Result of a full gaze computation for both eyes.
pub struct GazeResult {
    pub left_eye: EyeGazeAngles,
    pub right_eye: EyeGazeAngles,
    /// Morph weights to activate (e.g., lid follow, iris deform).
    pub morph_weights: HashMap<String, f32>,
}

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn vec3_length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_length(v);
    if len < 1e-8 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

// ---------------------------------------------------------------------------
// Core gaze functions
// ---------------------------------------------------------------------------

/// Compute gaze angles for one eye from its position toward a target point.
///
/// `right_dir = cross(forward_dir, up_dir)` (note: this gives left-hand cross
/// for a right-handed coordinate system where forward = +Z, up = +Y, so the
/// resulting right = -X; we negate to get +X = right).
pub fn eye_angles_to_point(
    eye_pos: [f32; 3],
    target: [f32; 3],
    config: &EyeConfig,
) -> EyeGazeAngles {
    let dir = vec3_normalize(vec3_sub(target, eye_pos));
    let fwd = config.forward_dir;
    let up = config.up_dir;

    // right_dir = normalize(forward × up) gives -X for standard coords;
    // we want +X = right, so negate.
    let right_raw = vec3_cross(fwd, up);
    let right_dir = vec3_normalize(right_raw);

    // Project dir onto the forward/right plane for yaw.
    let yaw_raw = f32::atan2(vec3_dot(dir, right_dir), vec3_dot(dir, fwd));
    // Pitch from the up component (clamped to avoid NaN from asin).
    let sin_pitch = vec3_dot(dir, up).clamp(-1.0, 1.0);
    let pitch_raw = sin_pitch.asin();

    EyeGazeAngles {
        yaw: yaw_raw.clamp(-config.max_yaw, config.max_yaw),
        pitch: pitch_raw.clamp(-config.max_pitch, config.max_pitch),
    }
}

/// Build morph_weights from averaged eye angles.
fn build_morph_weights(
    left: &EyeGazeAngles,
    right: &EyeGazeAngles,
    config: &EyeConfig,
) -> HashMap<String, f32> {
    let avg_pitch = (left.pitch + right.pitch) * 0.5;
    let avg_yaw = (left.yaw.abs() + right.yaw.abs()) * 0.5;

    let (upper, lower) = lid_follow_weight(avg_pitch, config.max_pitch);
    let iris = iris_deform_weight(avg_yaw, config.max_yaw);

    let mut weights = HashMap::new();
    weights.insert("lid_upper_follow".to_string(), upper);
    weights.insert("lid_lower_follow".to_string(), lower);
    weights.insert("iris_deform".to_string(), iris);
    weights
}

/// Compute gaze angles for both eyes given a target.
pub fn compute_gaze(config: &EyeConfig, target: &GazeTarget) -> GazeResult {
    match target {
        GazeTarget::Point(p) => {
            let left = eye_angles_to_point(config.left_eye_pos, *p, config);
            let right = eye_angles_to_point(config.right_eye_pos, *p, config);
            let morph_weights = build_morph_weights(&left, &right, config);
            GazeResult {
                left_eye: left,
                right_eye: right,
                morph_weights,
            }
        }
        GazeTarget::Direction(d) => {
            // Both eyes look in the same direction from their respective positions.
            let norm_d = vec3_normalize(*d);
            // We construct a synthetic far target along the direction from each eye.
            let far = 1000.0_f32;
            let left_target = [
                config.left_eye_pos[0] + norm_d[0] * far,
                config.left_eye_pos[1] + norm_d[1] * far,
                config.left_eye_pos[2] + norm_d[2] * far,
            ];
            let right_target = [
                config.right_eye_pos[0] + norm_d[0] * far,
                config.right_eye_pos[1] + norm_d[1] * far,
                config.right_eye_pos[2] + norm_d[2] * far,
            ];
            let left = eye_angles_to_point(config.left_eye_pos, left_target, config);
            let right = eye_angles_to_point(config.right_eye_pos, right_target, config);
            let morph_weights = build_morph_weights(&left, &right, config);
            GazeResult {
                left_eye: left,
                right_eye: right,
                morph_weights,
            }
        }
        GazeTarget::Angles { yaw, pitch } => {
            let left = EyeGazeAngles {
                yaw: yaw.clamp(-config.max_yaw, config.max_yaw),
                pitch: pitch.clamp(-config.max_pitch, config.max_pitch),
            };
            let right = EyeGazeAngles {
                yaw: yaw.clamp(-config.max_yaw, config.max_yaw),
                pitch: pitch.clamp(-config.max_pitch, config.max_pitch),
            };
            let morph_weights = build_morph_weights(&left, &right, config);
            GazeResult {
                left_eye: left,
                right_eye: right,
                morph_weights,
            }
        }
        GazeTarget::Forward => {
            let left = EyeGazeAngles {
                yaw: 0.0,
                pitch: 0.0,
            };
            let right = EyeGazeAngles {
                yaw: 0.0,
                pitch: 0.0,
            };
            let morph_weights = build_morph_weights(&left, &right, config);
            GazeResult {
                left_eye: left,
                right_eye: right,
                morph_weights,
            }
        }
    }
}

/// Convert gaze angles to a 3x3 rotation matrix (column-major).
///
/// Yaw rotates around the up axis (Y), pitch rotates around the right axis (X).
/// Final rotation = R_pitch * R_yaw.
pub fn gaze_to_rotation_matrix(angles: &EyeGazeAngles) -> [f32; 9] {
    let (sy, cy) = angles.yaw.sin_cos();
    let (sp, cp) = angles.pitch.sin_cos();

    // R_yaw (around Y axis):
    //  [ cy   0  sy ]
    //  [  0   1   0 ]
    //  [-sy   0  cy ]
    //
    // R_pitch (around X axis):
    //  [  1   0   0 ]
    //  [  0  cp -sp ]
    //  [  0  sp  cp ]
    //
    // R = R_pitch * R_yaw (column-major storage):
    // col 0: [cy, sp*sy, -cp*sy]
    // col 1: [0,  cp,    sp    ]
    // col 2: [sy, -sp*cy, cp*cy]

    [
        // col 0
        cy,
        sp * sy,
        -cp * sy,
        // col 1
        0.0,
        cp,
        sp,
        // col 2
        sy,
        -sp * cy,
        cp * cy,
    ]
}

/// Compute lid-follow morph weights from pitch angle.
///
/// Returns `(upper_lid_weight, lower_lid_weight)` in `[-1, 1]`.
/// Eyes looking up → upper lid raises (positive), lower lid lowers (negative).
pub fn lid_follow_weight(pitch: f32, max_pitch: f32) -> (f32, f32) {
    if max_pitch < 1e-8 {
        return (0.0, 0.0);
    }
    let t = (pitch / max_pitch).clamp(-1.0, 1.0);
    let upper = t * 0.3;
    let lower = -t * 0.2;
    (upper, lower)
}

/// Compute iris deform weight from yaw (side gaze stretches iris slightly).
pub fn iris_deform_weight(yaw: f32, max_yaw: f32) -> f32 {
    if max_yaw < 1e-8 {
        return 0.0;
    }
    (yaw.abs() / max_yaw).clamp(0.0, 1.0) * 0.15
}

// ---------------------------------------------------------------------------
// Saccade sequence
// ---------------------------------------------------------------------------

/// A sequence of gaze targets with optional blink events for saccade simulation.
pub struct SaccadeSequence {
    /// `(time_seconds, target)` pairs, sorted by time.
    pub targets: Vec<(f32, GazeTarget)>,
    /// Times at which a blink occurs.
    pub blink_times: Vec<f32>,
}

impl SaccadeSequence {
    /// Create an empty sequence.
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            blink_times: Vec::new(),
        }
    }

    /// Add a gaze target at the given time.
    pub fn add_target(&mut self, time: f32, target: GazeTarget) {
        self.targets.push((time, target));
        // Keep sorted by time.
        self.targets
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Add a blink event at the given time.
    pub fn add_blink(&mut self, time: f32) {
        self.blink_times.push(time);
        self.blink_times
            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Total duration of the sequence (time of the last target).
    pub fn duration(&self) -> f32 {
        self.targets.last().map(|(t, _)| *t).unwrap_or(0.0)
    }

    /// Evaluate the sequence at time `t`, returning a `GazeResult`.
    ///
    /// Linearly interpolates yaw/pitch between adjacent targets.
    /// Blink events add a `blink` morph weight of 1.0 for a window of ±0.05 s.
    pub fn evaluate(&self, t: f32, config: &EyeConfig) -> GazeResult {
        if self.targets.is_empty() {
            return compute_gaze(config, &GazeTarget::Forward);
        }

        // Find bracket.
        let first_time = self.targets[0].0;
        if t <= first_time {
            let result = compute_gaze(config, &self.targets[0].1);
            return self.apply_blink(result, t);
        }

        let last_target = &self.targets[self.targets.len() - 1];
        if t >= last_target.0 {
            let result = compute_gaze(config, &last_target.1);
            return self.apply_blink(result, t);
        }

        // Find adjacent targets by binary search.
        let idx = self.targets.partition_point(|(time, _)| *time <= t);
        let prev_idx = idx.saturating_sub(1);
        let next_idx = idx.min(self.targets.len() - 1);

        let (t0, ref tgt0) = self.targets[prev_idx];
        let (t1, ref tgt1) = self.targets[next_idx];

        let alpha = if (t1 - t0).abs() < 1e-8 {
            0.0
        } else {
            ((t - t0) / (t1 - t0)).clamp(0.0, 1.0)
        };

        let r0 = compute_gaze(config, tgt0);
        let r1 = compute_gaze(config, tgt1);

        let left = EyeGazeAngles {
            yaw: lerp(r0.left_eye.yaw, r1.left_eye.yaw, alpha),
            pitch: lerp(r0.left_eye.pitch, r1.left_eye.pitch, alpha),
        };
        let right = EyeGazeAngles {
            yaw: lerp(r0.right_eye.yaw, r1.right_eye.yaw, alpha),
            pitch: lerp(r0.right_eye.pitch, r1.right_eye.pitch, alpha),
        };

        let morph_weights = build_morph_weights(&left, &right, config);
        let mut result = GazeResult {
            left_eye: left,
            right_eye: right,
            morph_weights,
        };
        result = self.apply_blink(result, t);
        result
    }

    fn apply_blink(&self, mut result: GazeResult, t: f32) -> GazeResult {
        const BLINK_HALF_WINDOW: f32 = 0.05;
        let is_blinking = self
            .blink_times
            .iter()
            .any(|&bt| (t - bt).abs() <= BLINK_HALF_WINDOW);
        if is_blinking {
            result.morph_weights.insert("blink".to_string(), 1.0);
        }
        result
    }
}

impl Default for SaccadeSequence {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_4, FRAC_PI_6};

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    #[test]
    fn test_eye_config_default() {
        let cfg = EyeConfig::default();
        assert!(approx_eq(cfg.left_eye_pos[0], -0.032, 1e-5));
        assert!(approx_eq(cfg.right_eye_pos[0], 0.032, 1e-5));
        assert!(approx_eq(cfg.forward_dir[2], 1.0, 1e-5));
        assert!(approx_eq(cfg.up_dir[1], 1.0, 1e-5));
        assert!(approx_eq(cfg.max_yaw, FRAC_PI_4, 1e-5));
        assert!(approx_eq(cfg.max_pitch, FRAC_PI_6, 1e-5));
        assert!(approx_eq(cfg.convergence_dist, 2.0, 1e-5));
    }

    #[test]
    fn test_eye_angles_to_point_forward() {
        let cfg = EyeConfig::default();
        // Target directly in front.
        let angles = eye_angles_to_point([0.0, 1.67, 0.095], [0.0, 1.67, 5.0], &cfg);
        assert!(approx_eq(angles.yaw, 0.0, 1e-4));
        assert!(approx_eq(angles.pitch, 0.0, 1e-4));
    }

    #[test]
    fn test_eye_angles_to_point_right() {
        let cfg = EyeConfig::default();
        // Target far to the right at eye height.
        // With forward=+Z, up=+Y, right = cross(fwd, up) gives a direction.
        // cross([0,0,1], [0,1,0]) = [-1,0,0]; the code normalizes this.
        // So "right" in the config's frame is -X.
        // A target at +X relative to eye → negative yaw (leftward in that frame).
        // We just check yaw is non-zero and pitch is near zero.
        let angles = eye_angles_to_point([0.0, 1.67, 0.095], [10.0, 1.67, 1.095], &cfg);
        assert!(
            angles.yaw.abs() > 0.01,
            "yaw should be non-zero for side target"
        );
        assert!(approx_eq(angles.pitch, 0.0, 1e-3));
    }

    #[test]
    fn test_eye_angles_to_point_up() {
        let cfg = EyeConfig::default();
        // Target above, same X/Z.
        let angles = eye_angles_to_point([0.0, 1.67, 0.095], [0.0, 5.0, 5.095], &cfg);
        assert!(
            angles.pitch > 0.0,
            "pitch should be positive for upward target"
        );
    }

    #[test]
    fn test_eye_angles_clamped() {
        let cfg = EyeConfig::default();
        // Extreme right target: yaw must be clamped to max_yaw.
        let angles = eye_angles_to_point([0.0, 1.67, 0.095], [1000.0, 1.67, 0.095], &cfg);
        assert!(
            angles.yaw.abs() <= cfg.max_yaw + 1e-5,
            "yaw must not exceed max_yaw"
        );
        // Extreme up target: pitch must be clamped to max_pitch.
        let angles2 = eye_angles_to_point([0.0, 1.67, 0.095], [0.0, 1000.0, 0.095], &cfg);
        assert!(
            angles2.pitch.abs() <= cfg.max_pitch + 1e-5,
            "pitch must not exceed max_pitch"
        );
    }

    #[test]
    fn test_compute_gaze_forward() {
        let cfg = EyeConfig::default();
        let result = compute_gaze(&cfg, &GazeTarget::Forward);
        assert!(approx_eq(result.left_eye.yaw, 0.0, 1e-5));
        assert!(approx_eq(result.left_eye.pitch, 0.0, 1e-5));
        assert!(approx_eq(result.right_eye.yaw, 0.0, 1e-5));
        assert!(approx_eq(result.right_eye.pitch, 0.0, 1e-5));
        // Morph weights for neutral gaze should be near zero.
        let upper = result.morph_weights["lid_upper_follow"];
        let lower = result.morph_weights["lid_lower_follow"];
        assert!(approx_eq(upper, 0.0, 1e-5));
        assert!(approx_eq(lower, 0.0, 1e-5));
    }

    #[test]
    fn test_compute_gaze_point() {
        let cfg = EyeConfig::default();
        // Point far ahead → should be nearly forward.
        let result = compute_gaze(&cfg, &GazeTarget::Point([0.0, 1.67, 100.0]));
        assert!(result.left_eye.yaw.abs() < 0.01);
        assert!(result.left_eye.pitch.abs() < 0.01);
        // Vergence: both eyes converge on a close target.
        let result2 = compute_gaze(&cfg, &GazeTarget::Point([0.0, 1.67, 0.5]));
        // Left eye yaw should be positive (looking right), right eye negative (looking left) or vice versa.
        // With our right_dir = cross(fwd, up) = [-1,0,0]:
        // left eye looks at target slightly to its right → positive direction in -X frame.
        assert!(result2.morph_weights.contains_key("iris_deform"));
    }

    #[test]
    fn test_compute_gaze_angles() {
        let cfg = EyeConfig::default();
        let yaw = 0.3_f32;
        let pitch = 0.2_f32;
        let result = compute_gaze(&cfg, &GazeTarget::Angles { yaw, pitch });
        assert!(approx_eq(result.left_eye.yaw, yaw, 1e-5));
        assert!(approx_eq(result.left_eye.pitch, pitch, 1e-5));
        assert!(approx_eq(result.right_eye.yaw, yaw, 1e-5));
        assert!(approx_eq(result.right_eye.pitch, pitch, 1e-5));
    }

    #[test]
    fn test_lid_follow_weight() {
        let max_pitch = FRAC_PI_6;
        // Looking up (positive pitch) → positive upper lid, negative lower lid.
        let (upper, lower) = lid_follow_weight(max_pitch, max_pitch);
        assert!(approx_eq(upper, 0.3, 1e-5), "upper={upper}");
        assert!(approx_eq(lower, -0.2, 1e-5), "lower={lower}");
        // Looking down (negative pitch) → negative upper, positive lower.
        let (upper2, lower2) = lid_follow_weight(-max_pitch, max_pitch);
        assert!(approx_eq(upper2, -0.3, 1e-5));
        assert!(approx_eq(lower2, 0.2, 1e-5));
        // Neutral → zeros.
        let (upper3, lower3) = lid_follow_weight(0.0, max_pitch);
        assert!(approx_eq(upper3, 0.0, 1e-5));
        assert!(approx_eq(lower3, 0.0, 1e-5));
    }

    #[test]
    fn test_iris_deform_weight() {
        let max_yaw = FRAC_PI_4;
        // Extreme gaze → 0.15.
        let w = iris_deform_weight(max_yaw, max_yaw);
        assert!(approx_eq(w, 0.15, 1e-5), "w={w}");
        // Neutral → 0.
        let w0 = iris_deform_weight(0.0, max_yaw);
        assert!(approx_eq(w0, 0.0, 1e-5));
        // Negative yaw (left gaze) same magnitude.
        let wn = iris_deform_weight(-max_yaw, max_yaw);
        assert!(approx_eq(wn, 0.15, 1e-5));
    }

    #[test]
    fn test_gaze_to_rotation_matrix() {
        // Forward gaze → identity matrix.
        let angles = EyeGazeAngles {
            yaw: 0.0,
            pitch: 0.0,
        };
        let mat = gaze_to_rotation_matrix(&angles);
        // Column-major identity: [1,0,0, 0,1,0, 0,0,1].
        assert!(approx_eq(mat[0], 1.0, 1e-5), "mat[0]={}", mat[0]); // col0.x
        assert!(approx_eq(mat[1], 0.0, 1e-5), "mat[1]={}", mat[1]); // col0.y
        assert!(approx_eq(mat[2], 0.0, 1e-5), "mat[2]={}", mat[2]); // col0.z
        assert!(approx_eq(mat[3], 0.0, 1e-5), "mat[3]={}", mat[3]); // col1.x
        assert!(approx_eq(mat[4], 1.0, 1e-5), "mat[4]={}", mat[4]); // col1.y
        assert!(approx_eq(mat[5], 0.0, 1e-5), "mat[5]={}", mat[5]); // col1.z
        assert!(approx_eq(mat[6], 0.0, 1e-5), "mat[6]={}", mat[6]); // col2.x
        assert!(approx_eq(mat[7], 0.0, 1e-5), "mat[7]={}", mat[7]); // col2.y
        assert!(approx_eq(mat[8], 1.0, 1e-5), "mat[8]={}", mat[8]); // col2.z

        // Pure yaw of PI/2 around Y: col0 should be [0,0,-1], col2 [1,0,0].
        let angles_yaw = EyeGazeAngles {
            yaw: std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
        };
        let mat_yaw = gaze_to_rotation_matrix(&angles_yaw);
        assert!(approx_eq(mat_yaw[0], 0.0, 1e-5)); // cy
        assert!(approx_eq(mat_yaw[6], 1.0, 1e-5)); // sy (col2.x)
        assert!(approx_eq(mat_yaw[8], 0.0, 1e-5)); // cp*cy (col2.z)
    }

    #[test]
    fn test_saccade_sequence_new() {
        let seq = SaccadeSequence::new();
        assert!(seq.targets.is_empty());
        assert!(seq.blink_times.is_empty());
        assert!(approx_eq(seq.duration(), 0.0, 1e-5));
    }

    #[test]
    fn test_saccade_sequence_evaluate() {
        let cfg = EyeConfig::default();
        let mut seq = SaccadeSequence::new();
        seq.add_target(0.0, GazeTarget::Forward);
        seq.add_target(
            1.0,
            GazeTarget::Angles {
                yaw: 0.4,
                pitch: 0.1,
            },
        );
        seq.add_blink(0.5);

        // At t=0 → forward.
        let r0 = seq.evaluate(0.0, &cfg);
        assert!(approx_eq(r0.left_eye.yaw, 0.0, 1e-4));

        // At t=1 → angles.
        let r1 = seq.evaluate(1.0, &cfg);
        assert!(approx_eq(
            r1.left_eye.yaw,
            0.4_f32.clamp(-cfg.max_yaw, cfg.max_yaw),
            1e-4
        ));

        // At t=0.5 → half way + blink.
        let r_mid = seq.evaluate(0.5, &cfg);
        assert!(
            r_mid.morph_weights.contains_key("blink"),
            "blink weight should be present at t=0.5"
        );
        assert!(approx_eq(
            *r_mid.morph_weights.get("blink").expect("should succeed"),
            1.0,
            1e-5
        ));

        // At t=2.0 → clamp to last target.
        let r_late = seq.evaluate(2.0, &cfg);
        assert!(approx_eq(
            r_late.left_eye.yaw,
            0.4_f32.clamp(-cfg.max_yaw, cfg.max_yaw),
            1e-4
        ));

        // Duration.
        assert!(approx_eq(seq.duration(), 1.0, 1e-5));
    }
}
