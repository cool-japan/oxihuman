// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Hinge joint (rotation around a single axis).

#![allow(dead_code)]

/// Configuration for a hinge joint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HingeConfig {
    pub axis: [f32; 3],
    pub min_angle: f32,
    pub max_angle: f32,
    pub compliance: f32,
}

/// A hinge joint state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HingeJoint {
    pub angle: f32,
    pub lambda: f32,
    pub config: HingeConfig,
}

/// Result of solving a hinge joint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HingeResult {
    pub correction: f32,
    pub clamped: bool,
    pub error: f32,
}

/// Return the default hinge config.
#[allow(dead_code)]
pub fn default_hinge_config() -> HingeConfig {
    HingeConfig {
        axis: [0.0, 1.0, 0.0],
        min_angle: -std::f32::consts::PI,
        max_angle: std::f32::consts::PI,
        compliance: 0.0,
    }
}

/// Create a new hinge joint.
#[allow(dead_code)]
pub fn new_hinge_joint(config: HingeConfig) -> HingeJoint {
    HingeJoint { angle: 0.0, lambda: 0.0, config }
}

/// Solve: clamp angle to limits and compute the correction.
#[allow(dead_code)]
pub fn hinge_solve(joint: &mut HingeJoint, dt: f32) -> HingeResult {
    let clamped_angle = joint.angle.clamp(joint.config.min_angle, joint.config.max_angle);
    let clamped = (clamped_angle - joint.angle).abs() > 1e-7;
    let error = joint.angle - clamped_angle;
    let alpha = joint.config.compliance / (dt * dt);
    let correction = if (1.0 + alpha).abs() > 1e-10 {
        -error / (1.0 + alpha)
    } else {
        0.0
    };
    joint.lambda += correction;
    joint.angle += correction;
    HingeResult { correction, clamped, error }
}

/// Return the current angle.
#[allow(dead_code)]
pub fn hinge_current_angle(joint: &HingeJoint) -> f32 {
    joint.angle
}

/// Set the current angle directly.
#[allow(dead_code)]
pub fn hinge_set_angle(joint: &mut HingeJoint, angle: f32) {
    joint.angle = angle;
}

/// Return true if the angle is at either limit.
#[allow(dead_code)]
pub fn hinge_is_at_limit(joint: &HingeJoint) -> bool {
    joint.angle <= joint.config.min_angle || joint.angle >= joint.config.max_angle
}

/// Reset accumulated lambda.
#[allow(dead_code)]
pub fn hinge_reset(joint: &mut HingeJoint) {
    joint.lambda = 0.0;
}

/// Return the (min_angle, max_angle) range.
#[allow(dead_code)]
pub fn hinge_range(joint: &HingeJoint) -> (f32, f32) {
    (joint.config.min_angle, joint.config.max_angle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_hinge_config();
        assert!(cfg.min_angle < 0.0);
        assert!(cfg.max_angle > 0.0);
    }

    #[test]
    fn test_new_joint_zero_angle() {
        let joint = new_hinge_joint(default_hinge_config());
        assert!((hinge_current_angle(&joint)).abs() < 1e-7);
    }

    #[test]
    fn test_set_angle() {
        let mut joint = new_hinge_joint(default_hinge_config());
        hinge_set_angle(&mut joint, 1.0);
        assert!((hinge_current_angle(&joint) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_solve_clamps_at_max() {
        let mut cfg = default_hinge_config();
        cfg.max_angle = 0.5;
        cfg.min_angle = -0.5;
        let mut joint = new_hinge_joint(cfg);
        hinge_set_angle(&mut joint, 1.0);
        let result = hinge_solve(&mut joint, 0.016);
        assert!(result.clamped);
    }

    #[test]
    fn test_solve_no_clamp_within_range() {
        let mut joint = new_hinge_joint(default_hinge_config());
        hinge_set_angle(&mut joint, 0.1);
        let result = hinge_solve(&mut joint, 0.016);
        assert!(!result.clamped);
    }

    #[test]
    fn test_is_at_limit() {
        let cfg = HingeConfig {
            axis: [0.0, 1.0, 0.0],
            min_angle: -1.0,
            max_angle: 1.0,
            compliance: 0.0,
        };
        let mut joint = new_hinge_joint(cfg);
        joint.angle = -1.0;
        assert!(hinge_is_at_limit(&joint));
    }

    #[test]
    fn test_reset() {
        let mut joint = new_hinge_joint(default_hinge_config());
        joint.lambda = 42.0;
        hinge_reset(&mut joint);
        assert_eq!(joint.lambda, 0.0);
    }

    #[test]
    fn test_range() {
        let cfg = HingeConfig {
            axis: [1.0, 0.0, 0.0],
            min_angle: -2.0,
            max_angle: 2.0,
            compliance: 0.0,
        };
        let joint = new_hinge_joint(cfg);
        let (min, max) = hinge_range(&joint);
        assert!((min + 2.0).abs() < 1e-6);
        assert!((max - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_hinge_result_clone() {
        let r = HingeResult { correction: 0.1, clamped: false, error: 0.05 };
        let r2 = r.clone();
        assert!((r2.correction - 0.1).abs() < 1e-6);
    }
}
