// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Weld/fixed joint (keeps two bodies together).

#![allow(dead_code)]

/// Configuration for a weld joint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeldConfig {
    pub compliance: f32,
    pub angular_compliance: f32,
}

/// A weld joint state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeldJoint {
    pub offset: [f32; 3],
    pub lambda_pos: f32,
    pub lambda_rot: f32,
    pub config: WeldConfig,
}

/// Result of solving a weld joint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeldResult {
    pub pos_error: f32,
    pub satisfied: bool,
}

/// Return the default weld config.
#[allow(dead_code)]
pub fn default_weld_config() -> WeldConfig {
    WeldConfig { compliance: 0.0, angular_compliance: 0.0 }
}

/// Create a new weld joint.
#[allow(dead_code)]
pub fn new_weld_joint(offset: [f32; 3], config: WeldConfig) -> WeldJoint {
    WeldJoint { offset, lambda_pos: 0.0, lambda_rot: 0.0, config }
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Solve the positional part of the weld joint.
#[allow(dead_code)]
pub fn weld_solve_position(
    joint: &mut WeldJoint,
    pos_a: [f32; 3],
    pos_b: [f32; 3],
    inv_mass_a: f32,
    inv_mass_b: f32,
    dt: f32,
) -> WeldResult {
    let diff = [
        pos_b[0] - pos_a[0] - joint.offset[0],
        pos_b[1] - pos_a[1] - joint.offset[1],
        pos_b[2] - pos_a[2] - joint.offset[2],
    ];
    let err = vec3_len(diff);
    let alpha = joint.config.compliance / (dt * dt);
    let w = inv_mass_a + inv_mass_b + alpha;
    let delta_lambda = if w.abs() > 1e-10 { -err / w } else { 0.0 };
    joint.lambda_pos += delta_lambda;
    let satisfied = err < 1e-4;
    WeldResult { pos_error: err, satisfied }
}

/// Return the positional error.
#[allow(dead_code)]
pub fn weld_pos_error(joint: &WeldJoint, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
    let diff = [
        pos_b[0] - pos_a[0] - joint.offset[0],
        pos_b[1] - pos_a[1] - joint.offset[1],
        pos_b[2] - pos_a[2] - joint.offset[2],
    ];
    vec3_len(diff)
}

/// Return true if the positional error is below tolerance.
#[allow(dead_code)]
pub fn weld_is_satisfied(joint: &WeldJoint, pos_a: [f32; 3], pos_b: [f32; 3]) -> bool {
    weld_pos_error(joint, pos_a, pos_b) < 1e-4
}

/// Reset accumulated lambdas.
#[allow(dead_code)]
pub fn weld_reset(joint: &mut WeldJoint) {
    joint.lambda_pos = 0.0;
    joint.lambda_rot = 0.0;
}

/// Set the positional compliance.
#[allow(dead_code)]
pub fn weld_set_compliance(joint: &mut WeldJoint, compliance: f32) {
    joint.config.compliance = compliance;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_weld_config();
        assert_eq!(cfg.compliance, 0.0);
        assert_eq!(cfg.angular_compliance, 0.0);
    }

    #[test]
    fn test_new_joint() {
        let joint = new_weld_joint([0.0; 3], default_weld_config());
        assert_eq!(joint.lambda_pos, 0.0);
        assert_eq!(joint.lambda_rot, 0.0);
    }

    #[test]
    fn test_pos_error_zero_when_at_offset() {
        let joint = new_weld_joint([1.0, 0.0, 0.0], default_weld_config());
        let err = weld_pos_error(&joint, [0.0; 3], [1.0, 0.0, 0.0]);
        assert!(err < 1e-6);
    }

    #[test]
    fn test_pos_error_nonzero() {
        let joint = new_weld_joint([0.0; 3], default_weld_config());
        let err = weld_pos_error(&joint, [0.0; 3], [3.0, 4.0, 0.0]);
        assert!((err - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_is_satisfied_when_coincident() {
        let joint = new_weld_joint([0.0; 3], default_weld_config());
        assert!(weld_is_satisfied(&joint, [1.0; 3], [1.0; 3]));
    }

    #[test]
    fn test_is_not_satisfied_when_apart() {
        let joint = new_weld_joint([0.0; 3], default_weld_config());
        assert!(!weld_is_satisfied(&joint, [0.0; 3], [10.0, 0.0, 0.0]));
    }

    #[test]
    fn test_solve_position() {
        let mut joint = new_weld_joint([0.0; 3], default_weld_config());
        let result = weld_solve_position(&mut joint, [0.0; 3], [1.0, 0.0, 0.0], 1.0, 1.0, 0.016);
        assert!(result.pos_error > 0.0);
        assert!(!result.satisfied);
    }

    #[test]
    fn test_reset() {
        let mut joint = new_weld_joint([0.0; 3], default_weld_config());
        joint.lambda_pos = 5.0;
        joint.lambda_rot = 3.0;
        weld_reset(&mut joint);
        assert_eq!(joint.lambda_pos, 0.0);
        assert_eq!(joint.lambda_rot, 0.0);
    }

    #[test]
    fn test_set_compliance() {
        let mut joint = new_weld_joint([0.0; 3], default_weld_config());
        weld_set_compliance(&mut joint, 0.01);
        assert!((joint.config.compliance - 0.01).abs() < 1e-6);
    }

    #[test]
    fn test_weld_result_clone() {
        let r = WeldResult { pos_error: 0.5, satisfied: false };
        let r2 = r.clone();
        assert!((r2.pos_error - 0.5).abs() < 1e-6);
    }
}
