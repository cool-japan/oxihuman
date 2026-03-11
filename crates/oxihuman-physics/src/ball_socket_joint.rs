// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Ball-and-socket joint constraint.

#![allow(dead_code)]

/// Configuration for a ball-socket joint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BallSocketConfig {
    pub compliance: f32,
    pub damping: f32,
    pub max_force: f32,
}

/// A ball-and-socket joint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BallSocketJoint {
    pub anchor_a: [f32; 3],
    pub anchor_b: [f32; 3],
    pub lambda: f32,
    pub config: BallSocketConfig,
}

/// Result of solving a ball-socket joint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BallSocketResult {
    pub delta_a: [f32; 3],
    pub delta_b: [f32; 3],
    pub error: f32,
}

/// Return the default ball-socket config.
#[allow(dead_code)]
pub fn default_ball_socket_config() -> BallSocketConfig {
    BallSocketConfig {
        compliance: 0.0,
        damping: 0.01,
        max_force: f32::INFINITY,
    }
}

/// Create a new ball-socket joint.
#[allow(dead_code)]
pub fn new_ball_socket_joint(anchor_a: [f32; 3], anchor_b: [f32; 3], config: BallSocketConfig) -> BallSocketJoint {
    BallSocketJoint { anchor_a, anchor_b, lambda: 0.0, config }
}

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Solve the ball-socket joint given inverse masses.
#[allow(dead_code)]
pub fn ball_socket_solve(
    joint: &mut BallSocketJoint,
    inv_mass_a: f32,
    inv_mass_b: f32,
    dt: f32,
) -> BallSocketResult {
    let diff = vec3_sub(joint.anchor_b, joint.anchor_a);
    let err = vec3_len(diff);
    let w = inv_mass_a + inv_mass_b;
    let alpha = joint.config.compliance / (dt * dt);
    let delta_lambda = if (w + alpha).abs() > 1e-10 {
        -err / (w + alpha)
    } else {
        0.0
    };
    joint.lambda += delta_lambda;

    let dir = if err > 1e-10 {
        vec3_scale(diff, 1.0 / err)
    } else {
        [0.0, 0.0, 0.0]
    };

    let delta_a = vec3_scale(dir, -inv_mass_a * delta_lambda);
    let delta_b = vec3_scale(dir, inv_mass_b * delta_lambda);

    BallSocketResult { delta_a, delta_b, error: err }
}

/// Return the positional error (distance between anchors).
#[allow(dead_code)]
pub fn ball_socket_error(joint: &BallSocketJoint) -> f32 {
    vec3_len(vec3_sub(joint.anchor_b, joint.anchor_a))
}

/// Reset the accumulated lambda.
#[allow(dead_code)]
pub fn ball_socket_reset(joint: &mut BallSocketJoint) {
    joint.lambda = 0.0;
}

/// Set the compliance of the joint.
#[allow(dead_code)]
pub fn ball_socket_set_compliance(joint: &mut BallSocketJoint, compliance: f32) {
    joint.config.compliance = compliance;
}

/// Return the stiffness (reciprocal of compliance, clamped).
#[allow(dead_code)]
pub fn ball_socket_stiffness(joint: &BallSocketJoint) -> f32 {
    if joint.config.compliance.abs() < 1e-10 {
        f32::INFINITY
    } else {
        1.0 / joint.config.compliance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_ball_socket_config();
        assert_eq!(cfg.compliance, 0.0);
        assert!(cfg.damping >= 0.0);
    }

    #[test]
    fn test_new_joint() {
        let cfg = default_ball_socket_config();
        let joint = new_ball_socket_joint([0.0; 3], [1.0, 0.0, 0.0], cfg);
        assert_eq!(joint.lambda, 0.0);
    }

    #[test]
    fn test_error_at_distance() {
        let cfg = default_ball_socket_config();
        let joint = new_ball_socket_joint([0.0; 3], [3.0, 4.0, 0.0], cfg);
        assert!((ball_socket_error(&joint) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_solve_reduces_error() {
        let cfg = default_ball_socket_config();
        let mut joint = new_ball_socket_joint([0.0; 3], [1.0, 0.0, 0.0], cfg);
        let result = ball_socket_solve(&mut joint, 1.0, 1.0, 0.016);
        assert!(result.error > 0.0);
        // The deltas should point the bodies toward each other
        assert!(result.delta_a[0] > 0.0 || result.delta_b[0] < 0.0 || result.error < 1e-10);
    }

    #[test]
    fn test_reset() {
        let mut joint = new_ball_socket_joint([0.0; 3], [1.0; 3], default_ball_socket_config());
        joint.lambda = 5.0;
        ball_socket_reset(&mut joint);
        assert_eq!(joint.lambda, 0.0);
    }

    #[test]
    fn test_set_compliance() {
        let mut joint = new_ball_socket_joint([0.0; 3], [0.0; 3], default_ball_socket_config());
        ball_socket_set_compliance(&mut joint, 0.1);
        assert!((joint.config.compliance - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_stiffness_zero_compliance() {
        let joint = new_ball_socket_joint([0.0; 3], [0.0; 3], default_ball_socket_config());
        assert_eq!(ball_socket_stiffness(&joint), f32::INFINITY);
    }

    #[test]
    fn test_stiffness_nonzero_compliance() {
        let mut joint = new_ball_socket_joint([0.0; 3], [0.0; 3], default_ball_socket_config());
        ball_socket_set_compliance(&mut joint, 0.5);
        assert!((ball_socket_stiffness(&joint) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_zero_error_solve() {
        let cfg = default_ball_socket_config();
        let mut joint = new_ball_socket_joint([1.0; 3], [1.0; 3], cfg);
        let result = ball_socket_solve(&mut joint, 1.0, 1.0, 0.016);
        assert!(result.error < 1e-10);
    }
}
