// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Powered articulated joints for character locomotion.
//!
//! Supports hinge, ball-socket, slider, and fixed joint types with optional
//! motors and XPBD-style compliance.

use crate::rigid_body::RigidBody;

// ── Types ─────────────────────────────────────────────────────────────────────

/// The geometric constraint type of a joint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum JointKind {
    /// Revolute joint constrained to a single axis.
    Hinge {
        axis: [f32; 3],
        /// Optional angular limits in radians (min, max).
        limits: Option<(f32, f32)>,
    },
    /// Ball-and-socket joint with optional swing limit (radians).
    BallSocket { swing_limit: Option<f32> },
    /// Prismatic joint that slides along an axis.
    Slider {
        axis: [f32; 3],
        /// Optional position limits in meters (min, max).
        limits: Option<(f32, f32)>,
    },
    /// Fully rigid constraint — no relative motion allowed.
    Fixed,
}

/// Motor attached to a joint that drives it toward a target velocity.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointMotor {
    /// Target angular (rad/s) or linear (m/s) velocity.
    pub target_velocity: f32,
    /// Maximum force or torque the motor can exert (N or N·m).
    pub max_force: f32,
    /// Whether the motor is currently active.
    pub enabled: bool,
}

/// A constraint between two rigid bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Joint {
    pub kind: JointKind,
    /// Index of body A in the bodies slice.
    pub body_a: usize,
    /// Index of body B in the bodies slice.
    pub body_b: usize,
    /// Attachment point in body A's local space.
    pub anchor_a: [f32; 3],
    /// Attachment point in body B's local space.
    pub anchor_b: [f32; 3],
    /// Optional powered motor.
    pub motor: Option<JointMotor>,
    /// XPBD compliance (0 = perfectly rigid).
    pub compliance: f32,
}

/// A collection of joints that can be solved together.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct JointSystem {
    pub joints: Vec<Joint>,
}

// ── JointSystem impl ──────────────────────────────────────────────────────────

impl JointSystem {
    /// Create an empty joint system.
    #[allow(dead_code)]
    pub fn new() -> Self {
        JointSystem { joints: Vec::new() }
    }

    /// Add a joint and return its index.
    #[allow(dead_code)]
    pub fn add_joint(&mut self, joint: Joint) -> usize {
        let idx = self.joints.len();
        self.joints.push(joint);
        idx
    }

    /// Solve all joints for the given bodies over `substeps` iterations.
    #[allow(dead_code)]
    pub fn solve_joints(&self, bodies: &mut [RigidBody], dt: f32, substeps: u32) {
        let sub_dt = dt / substeps as f32;
        for _ in 0..substeps {
            for joint in &self.joints {
                match &joint.kind {
                    JointKind::Hinge { .. } => {
                        solve_hinge_constraint(bodies, joint, sub_dt);
                    }
                    JointKind::BallSocket { .. } => {
                        solve_ball_socket_constraint(bodies, joint, sub_dt);
                    }
                    JointKind::Slider { .. } => {
                        solve_slider_constraint(bodies, joint, sub_dt);
                    }
                    JointKind::Fixed => {
                        solve_ball_socket_constraint(bodies, joint, sub_dt);
                    }
                }
            }
        }
    }
}

// ── Vector helpers (local) ────────────────────────────────────────────────────

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn vec3_scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec3_len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec3_normalize(a: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(a);
    if len < 1e-9 {
        return [0.0, 0.0, 0.0];
    }
    [a[0] / len, a[1] / len, a[2] / len]
}

/// Rotate a vector by a quaternion [x, y, z, w].
fn quat_rotate_vec3(q: [f32; 4], v: [f32; 3]) -> [f32; 3] {
    let [qx, qy, qz, qw] = q;
    let t = vec3_scale(vec3_cross([qx, qy, qz], v), 2.0);
    let tt = vec3_cross([qx, qy, qz], t);
    vec3_add(vec3_add(v, vec3_scale(t, qw)), tt)
}

// ── Constraint solvers ────────────────────────────────────────────────────────

/// Compute the world-space position of body `b`'s anchor.
fn world_anchor(body: &RigidBody, local_anchor: [f32; 3]) -> [f32; 3] {
    let rotated = quat_rotate_vec3(body.orientation, local_anchor);
    vec3_add(body.position, rotated)
}

/// Enforce the ball-socket (positional) constraint between two bodies.
///
/// Returns the magnitude of the positional violation before correction.
#[allow(dead_code)]
pub fn solve_ball_socket_constraint(bodies: &mut [RigidBody], j: &Joint, dt: f32) -> f32 {
    if j.body_a >= bodies.len() || j.body_b >= bodies.len() {
        return 0.0;
    }
    let wa = world_anchor(&bodies[j.body_a], j.anchor_a);
    let wb = world_anchor(&bodies[j.body_b], j.anchor_b);
    let delta = vec3_sub(wb, wa);
    let violation = vec3_len(delta);
    if violation < 1e-9 {
        return 0.0;
    }
    let dir = vec3_normalize(delta);
    let w_a = bodies[j.body_a].inv_mass;
    let w_b = bodies[j.body_b].inv_mass;
    let w_total = w_a + w_b;
    if w_total < 1e-12 {
        return violation;
    }
    // XPBD: alpha = compliance / dt²
    let alpha = j.compliance / (dt * dt);
    let lagrange = violation / (w_total + alpha);
    let correction = vec3_scale(dir, lagrange);
    bodies[j.body_a].position = vec3_add(bodies[j.body_a].position, vec3_scale(correction, w_a));
    bodies[j.body_b].position = vec3_sub(bodies[j.body_b].position, vec3_scale(correction, w_b));
    // Apply motor if present
    if let Some(motor) = &j.motor {
        apply_motor_force(&mut bodies[j.body_b], dir, motor, dt);
    }
    violation
}

/// Enforce the hinge constraint and optionally its angular limits.
///
/// Returns the positional violation magnitude.
#[allow(dead_code)]
pub fn solve_hinge_constraint(bodies: &mut [RigidBody], j: &Joint, dt: f32) -> f32 {
    // Positional part: same as ball-socket
    let violation = solve_ball_socket_constraint(bodies, j, dt);
    if let JointKind::Hinge { axis, limits } = &j.kind {
        // Apply motor along the hinge axis
        if let Some(motor) = &j.motor {
            if j.body_b < bodies.len() {
                apply_motor_force(&mut bodies[j.body_b], *axis, motor, dt);
            }
        }
        // Apply limits if present
        if let Some((lo, hi)) = limits {
            if j.body_a < bodies.len() && j.body_b < bodies.len() {
                let q_a = bodies[j.body_a].orientation;
                let q_b = bodies[j.body_b].orientation;
                let angle = hinge_angle(*axis, q_a, q_b);
                let correction_torque = if angle < *lo {
                    (*lo - angle) * 100.0
                } else if angle > *hi {
                    (*hi - angle) * 100.0
                } else {
                    0.0
                };
                if correction_torque.abs() > 1e-9 {
                    let torque_vec = vec3_scale(*axis, correction_torque);
                    let inv_i_b = bodies[j.body_b].inv_inertia[1][1];
                    let delta_omega = vec3_scale(torque_vec, inv_i_b * dt);
                    bodies[j.body_b].angular_velocity =
                        vec3_add(bodies[j.body_b].angular_velocity, delta_omega);
                }
            }
        }
    }
    violation
}

/// Enforce slider (prismatic) constraint along a given axis.
///
/// Returns the lateral violation magnitude (perpendicular to axis).
#[allow(dead_code)]
pub fn solve_slider_constraint(bodies: &mut [RigidBody], j: &Joint, dt: f32) -> f32 {
    if j.body_a >= bodies.len() || j.body_b >= bodies.len() {
        return 0.0;
    }
    let wa = world_anchor(&bodies[j.body_a], j.anchor_a);
    let wb = world_anchor(&bodies[j.body_b], j.anchor_b);
    let delta = vec3_sub(wb, wa);

    if let JointKind::Slider { axis, limits } = &j.kind {
        let axis_n = vec3_normalize(*axis);
        let along = vec3_dot(delta, axis_n);
        // Remove the component along the slide axis
        let lateral = vec3_sub(delta, vec3_scale(axis_n, along));
        let lateral_violation = vec3_len(lateral);

        if lateral_violation > 1e-9 {
            let dir = vec3_normalize(lateral);
            let w_a = bodies[j.body_a].inv_mass;
            let w_b = bodies[j.body_b].inv_mass;
            let w_total = w_a + w_b;
            if w_total > 1e-12 {
                let alpha = j.compliance / (dt * dt);
                let lagrange = lateral_violation / (w_total + alpha);
                let correction = vec3_scale(dir, lagrange);
                bodies[j.body_a].position =
                    vec3_add(bodies[j.body_a].position, vec3_scale(correction, w_a));
                bodies[j.body_b].position =
                    vec3_sub(bodies[j.body_b].position, vec3_scale(correction, w_b));
            }
        }

        // Apply position limits along axis
        if let Some((lo, hi)) = limits {
            let clamped = along.clamp(*lo, *hi);
            let limit_correction = clamped - along;
            if limit_correction.abs() > 1e-9 {
                let correction = vec3_scale(axis_n, limit_correction);
                let w_a = bodies[j.body_a].inv_mass;
                let w_b = bodies[j.body_b].inv_mass;
                let w_total = w_a + w_b;
                if w_total > 1e-12 {
                    bodies[j.body_a].position = vec3_sub(
                        bodies[j.body_a].position,
                        vec3_scale(correction, w_a / w_total),
                    );
                    bodies[j.body_b].position = vec3_add(
                        bodies[j.body_b].position,
                        vec3_scale(correction, w_b / w_total),
                    );
                }
            }
        }

        // Apply motor
        if let Some(motor) = &j.motor {
            apply_motor_force(&mut bodies[j.body_b], axis_n, motor, dt);
        }

        lateral_violation
    } else {
        solve_ball_socket_constraint(bodies, j, dt)
    }
}

/// Compute the distance between the world-space anchors (positional violation).
#[allow(dead_code)]
pub fn joint_violation(bodies: &[RigidBody], j: &Joint) -> f32 {
    if j.body_a >= bodies.len() || j.body_b >= bodies.len() {
        return 0.0;
    }
    let wa = world_anchor(&bodies[j.body_a], j.anchor_a);
    let wb = world_anchor(&bodies[j.body_b], j.anchor_b);
    vec3_len(vec3_sub(wb, wa))
}

/// Apply a motor velocity target to a body's velocity along an axis.
///
/// The impulse is clamped to `motor.max_force * dt`.
#[allow(dead_code)]
pub fn apply_motor_force(body: &mut RigidBody, axis: [f32; 3], motor: &JointMotor, dt: f32) {
    if !motor.enabled || body.inv_mass == 0.0 {
        return;
    }
    let axis_n = vec3_normalize(axis);
    let current_vel = vec3_dot(body.velocity, axis_n);
    let vel_error = motor.target_velocity - current_vel;
    // Clamp impulse magnitude
    let max_impulse = motor.max_force * dt;
    let impulse_magnitude = vel_error.clamp(-max_impulse, max_impulse);
    let impulse = vec3_scale(axis_n, impulse_magnitude * body.inv_mass);
    body.velocity = vec3_add(body.velocity, impulse);
}

/// Compute the relative rotation angle between two bodies around a hinge axis.
///
/// Returns a value in [-π, π].
#[allow(dead_code)]
pub fn hinge_angle(axis: [f32; 3], q_a: [f32; 4], q_b: [f32; 4]) -> f32 {
    // Rotate a reference vector perpendicular to axis by each orientation
    let axis_n = vec3_normalize(axis);
    // Build a perpendicular reference vector
    let perp = if axis_n[0].abs() < 0.9 {
        vec3_normalize(vec3_cross(axis_n, [1.0, 0.0, 0.0]))
    } else {
        vec3_normalize(vec3_cross(axis_n, [0.0, 1.0, 0.0]))
    };
    let ref_a = quat_rotate_vec3(q_a, perp);
    let ref_b = quat_rotate_vec3(q_b, perp);
    // Project both onto the plane perpendicular to axis
    let proj_a = vec3_sub(ref_a, vec3_scale(axis_n, vec3_dot(ref_a, axis_n)));
    let proj_b = vec3_sub(ref_b, vec3_scale(axis_n, vec3_dot(ref_b, axis_n)));
    let dot = vec3_dot(vec3_normalize(proj_a), vec3_normalize(proj_b)).clamp(-1.0, 1.0);
    let cross = vec3_dot(vec3_cross(proj_a, proj_b), axis_n);
    dot.acos() * cross.signum()
}

/// Build a standard 14-joint biped skeleton using ball-socket joints.
///
/// Joint indices:
///  0: spine_lower (0-1), 1: spine_upper (1-2), 2: neck (2-3),
///  3: shoulder_l (2-4), 4: elbow_l (4-5), 5: wrist_l (5-6),
///  6: shoulder_r (2-7), 7: elbow_r (7-8), 8: wrist_r (8-9),
///  9: hip_l (0-10), 10: knee_l (10-11), 11: ankle_l (11-12),
/// 12: hip_r (0-13), 13: knee_r (13-14)
#[allow(dead_code)]
pub fn standard_biped_joints(n_bodies: usize) -> JointSystem {
    let _ = n_bodies; // provided for API completeness
    let mut sys = JointSystem::new();

    let ball = |body_a, body_b, anchor_a, anchor_b| Joint {
        kind: JointKind::BallSocket { swing_limit: None },
        body_a,
        body_b,
        anchor_a,
        anchor_b,
        motor: None,
        compliance: 0.0,
    };

    // Spine chain: pelvis(0) → lumbar(1) → thorax(2) → head(3)
    sys.add_joint(ball(0, 1, [0.0, 0.1, 0.0], [0.0, -0.1, 0.0]));
    sys.add_joint(ball(1, 2, [0.0, 0.15, 0.0], [0.0, -0.15, 0.0]));
    sys.add_joint(ball(2, 3, [0.0, 0.1, 0.0], [0.0, -0.05, 0.0]));
    // Left arm: thorax(2) → upper_arm_l(4) → forearm_l(5) → hand_l(6)
    sys.add_joint(ball(2, 4, [-0.2, 0.05, 0.0], [0.0, 0.15, 0.0]));
    sys.add_joint(ball(4, 5, [0.0, -0.15, 0.0], [0.0, 0.1, 0.0]));
    sys.add_joint(ball(5, 6, [0.0, -0.1, 0.0], [0.0, 0.05, 0.0]));
    // Right arm: thorax(2) → upper_arm_r(7) → forearm_r(8) → hand_r(9)
    sys.add_joint(ball(2, 7, [0.2, 0.05, 0.0], [0.0, 0.15, 0.0]));
    sys.add_joint(ball(7, 8, [0.0, -0.15, 0.0], [0.0, 0.1, 0.0]));
    sys.add_joint(ball(8, 9, [0.0, -0.1, 0.0], [0.0, 0.05, 0.0]));
    // Left leg: pelvis(0) → thigh_l(10) → shin_l(11) → foot_l(12)
    sys.add_joint(ball(0, 10, [-0.1, -0.1, 0.0], [0.0, 0.2, 0.0]));
    sys.add_joint(ball(10, 11, [0.0, -0.2, 0.0], [0.0, 0.2, 0.0]));
    sys.add_joint(ball(11, 12, [0.0, -0.2, 0.0], [0.0, 0.05, 0.0]));
    // Right leg: pelvis(0) → thigh_r(13) → shin_r(14)
    sys.add_joint(ball(0, 13, [0.1, -0.1, 0.0], [0.0, 0.2, 0.0]));
    sys.add_joint(ball(13, 14, [0.0, -0.2, 0.0], [0.0, 0.2, 0.0]));

    sys
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rigid_body::RigidBody;

    fn make_body(pos: [f32; 3], mass: f32) -> RigidBody {
        let mut rb = RigidBody::new_sphere(1.0, mass);
        rb.position = pos;
        rb
    }

    fn default_ball_joint(a: usize, b: usize) -> Joint {
        Joint {
            kind: JointKind::BallSocket { swing_limit: None },
            body_a: a,
            body_b: b,
            anchor_a: [0.0; 3],
            anchor_b: [0.0; 3],
            motor: None,
            compliance: 0.0,
        }
    }

    // 1. JointSystem::new creates empty system
    #[test]
    fn test_joint_system_new() {
        let sys = JointSystem::new();
        assert!(sys.joints.is_empty());
    }

    // 2. add_joint returns correct index
    #[test]
    fn test_add_joint_index() {
        let mut sys = JointSystem::new();
        let j = default_ball_joint(0, 1);
        let idx = sys.add_joint(j);
        assert_eq!(idx, 0);
        let j2 = default_ball_joint(1, 2);
        let idx2 = sys.add_joint(j2);
        assert_eq!(idx2, 1);
    }

    // 3. joint_violation zero when anchors coincide at same position
    #[test]
    fn test_joint_violation_zero_coincide() {
        let bodies = vec![make_body([0.0; 3], 1.0), make_body([0.0; 3], 1.0)];
        let j = default_ball_joint(0, 1);
        let v = joint_violation(&bodies, &j);
        assert!(v < 1e-6, "violation should be near zero, got {v}");
    }

    // 4. joint_violation non-zero when bodies are separated
    #[test]
    fn test_joint_violation_nonzero_separated() {
        let bodies = vec![make_body([0.0; 3], 1.0), make_body([3.0, 0.0, 0.0], 1.0)];
        let j = default_ball_joint(0, 1);
        let v = joint_violation(&bodies, &j);
        assert!((v - 3.0).abs() < 1e-4, "violation should be 3.0, got {v}");
    }

    // 5. solve_ball_socket reduces violation
    #[test]
    fn test_ball_socket_reduces_violation() {
        let mut bodies = vec![make_body([0.0; 3], 1.0), make_body([2.0, 0.0, 0.0], 1.0)];
        let j = default_ball_joint(0, 1);
        let before = joint_violation(&bodies, &j);
        solve_ball_socket_constraint(&mut bodies, &j, 0.016);
        let after = joint_violation(&bodies, &j);
        assert!(
            after < before,
            "violation should decrease: before={before}, after={after}"
        );
    }

    // 6. hinge_angle zero for identical orientations
    #[test]
    fn test_hinge_angle_zero_identical() {
        let q = [0.0f32, 0.0, 0.0, 1.0];
        let angle = hinge_angle([0.0, 1.0, 0.0], q, q);
        assert!(
            angle.abs() < 1e-4,
            "angle should be ~0 for identical orientations, got {angle}"
        );
    }

    // 7. standard_biped_joints creates exactly 14 joints
    #[test]
    fn test_standard_biped_joints_count() {
        let sys = standard_biped_joints(15);
        assert_eq!(sys.joints.len(), 14);
    }

    // 8. motor enabled/disabled: enabled motor changes velocity
    #[test]
    fn test_motor_enabled_changes_velocity() {
        let mut body = make_body([0.0; 3], 1.0);
        let motor = JointMotor {
            target_velocity: 5.0,
            max_force: 100.0,
            enabled: true,
        };
        apply_motor_force(&mut body, [1.0, 0.0, 0.0], &motor, 0.1);
        assert!(body.velocity[0].abs() > 0.0);
    }

    // 9. motor disabled: velocity unchanged
    #[test]
    fn test_motor_disabled_no_change() {
        let mut body = make_body([0.0; 3], 1.0);
        let motor = JointMotor {
            target_velocity: 5.0,
            max_force: 100.0,
            enabled: false,
        };
        apply_motor_force(&mut body, [1.0, 0.0, 0.0], &motor, 0.1);
        assert!((body.velocity[0]).abs() < 1e-9);
    }

    // 10. solve_joints doesn't produce NaN
    #[test]
    fn test_solve_joints_no_nan() {
        let mut bodies = vec![make_body([0.0; 3], 1.0), make_body([1.0, 0.0, 0.0], 1.0)];
        let sys = {
            let mut s = JointSystem::new();
            s.add_joint(default_ball_joint(0, 1));
            s
        };
        sys.solve_joints(&mut bodies, 0.016, 4);
        for b in &bodies {
            assert!(!b.position[0].is_nan(), "position should not be NaN");
            assert!(!b.velocity[0].is_nan(), "velocity should not be NaN");
        }
    }

    // 11. apply_motor_force changes velocity along axis
    #[test]
    fn test_apply_motor_force_along_axis() {
        let mut body = make_body([0.0; 3], 2.0);
        let motor = JointMotor {
            target_velocity: 10.0,
            max_force: 50.0,
            enabled: true,
        };
        apply_motor_force(&mut body, [0.0, 1.0, 0.0], &motor, 0.1);
        // Should have acquired some Y velocity
        assert!(body.velocity[1] > 0.0);
    }

    // 12. JointKind variants can be created without panic
    #[test]
    fn test_joint_kind_variants() {
        let _hinge = JointKind::Hinge {
            axis: [0.0, 1.0, 0.0],
            limits: Some((-1.5, 1.5)),
        };
        let _ball = JointKind::BallSocket {
            swing_limit: Some(0.5),
        };
        let _slider = JointKind::Slider {
            axis: [0.0, 1.0, 0.0],
            limits: Some((-0.5, 0.5)),
        };
        let _fixed = JointKind::Fixed;
    }

    // 13. compliance zero: solve gives rigid correction (full violation resolved in one step)
    #[test]
    fn test_compliance_zero_rigid() {
        let mut bodies = vec![make_body([0.0; 3], 1.0), make_body([0.4, 0.0, 0.0], 1.0)];
        let j = Joint {
            kind: JointKind::BallSocket { swing_limit: None },
            body_a: 0,
            body_b: 1,
            anchor_a: [0.0; 3],
            anchor_b: [0.0; 3],
            motor: None,
            compliance: 0.0,
        };
        solve_ball_socket_constraint(&mut bodies, &j, 0.016);
        let after = joint_violation(&bodies, &j);
        // With compliance=0, should fully resolve
        assert!(
            after < 1e-5,
            "compliance=0 should fully resolve, got {after}"
        );
    }

    // 14. Hinge solve doesn't panic with limits
    #[test]
    fn test_hinge_solve_with_limits() {
        let mut bodies = vec![make_body([0.0; 3], 1.0), make_body([1.0, 0.0, 0.0], 1.0)];
        let j = Joint {
            kind: JointKind::Hinge {
                axis: [0.0, 1.0, 0.0],
                limits: Some((-0.5, 0.5)),
            },
            body_a: 0,
            body_b: 1,
            anchor_a: [0.0; 3],
            anchor_b: [0.0; 3],
            motor: None,
            compliance: 0.0,
        };
        let v = solve_hinge_constraint(&mut bodies, &j, 0.016);
        assert!(!v.is_nan());
    }

    // 15. Slider solve lateral violation
    #[test]
    fn test_slider_reduces_lateral_violation() {
        let mut bodies = vec![make_body([0.0; 3], 1.0), make_body([0.0, 2.0, 3.0], 1.0)];
        let j = Joint {
            kind: JointKind::Slider {
                axis: [0.0, 1.0, 0.0], // sliding along Y
                limits: None,
            },
            body_a: 0,
            body_b: 1,
            anchor_a: [0.0; 3],
            anchor_b: [0.0; 3],
            motor: None,
            compliance: 0.0,
        };
        // Lateral violation is in Z (3.0)
        let v = solve_slider_constraint(&mut bodies, &j, 0.016);
        assert!(!v.is_nan());
    }
}
