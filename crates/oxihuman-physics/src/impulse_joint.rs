// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Impulse-based joint constraints for connecting rigid bodies.
//!
//! Supports three joint types: ball-socket (full 3-DOF rotation), hinge (1-DOF
//! rotation around a fixed axis), and fixed (zero DOF — rigid attachment).

// ──────────────────────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────────────────────

/// The variety of joint constraint.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpulseJointType {
    /// 3-DOF rotational freedom (ball in socket).
    BallSocket,
    /// 1-DOF rotational freedom around a fixed axis.
    Hinge,
    /// 0 DOF — rigid attachment between two bodies.
    Fixed,
}

/// Configuration parameters shared by all impulse joints.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImpulseJointConfig {
    /// Position-error correction stiffness (Baumgarte stabilisation).
    pub stiffness: f32,
    /// Velocity-level damping coefficient.
    pub damping: f32,
    /// Maximum impulse magnitude that can break the joint.
    pub break_force: f32,
    /// Local-space anchor on body A.
    pub anchor_a: [f32; 3],
    /// Local-space anchor on body B.
    pub anchor_b: [f32; 3],
}

/// A single impulse-based joint constraint.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImpulseJoint {
    /// The type of joint.
    pub joint_type: ImpulseJointType,
    /// Index of the first body.
    pub body_a: usize,
    /// Index of the second body.
    pub body_b: usize,
    /// Configuration (stiffness, damping, anchors, break force).
    pub cfg: ImpulseJointConfig,
    /// Whether the joint has been broken.
    pub broken: bool,
    /// Accumulated impulse magnitude from the last solve (for break detection).
    pub accumulated_impulse: f32,
}

// ──────────────────────────────────────────────────────────────────────────────
// Public functions
// ──────────────────────────────────────────────────────────────────────────────

/// Returns a sensible default [`ImpulseJointConfig`].
#[allow(dead_code)]
pub fn default_joint_config() -> ImpulseJointConfig {
    ImpulseJointConfig {
        stiffness: 100.0,
        damping: 5.0,
        break_force: f32::INFINITY,
        anchor_a: [0.0; 3],
        anchor_b: [0.0; 3],
    }
}

/// Creates a new [`ImpulseJoint`] between bodies `body_a` and `body_b`.
#[allow(dead_code)]
pub fn new_impulse_joint(
    type_: ImpulseJointType,
    body_a: usize,
    body_b: usize,
    cfg: &ImpulseJointConfig,
) -> ImpulseJoint {
    ImpulseJoint {
        joint_type: type_,
        body_a,
        body_b,
        cfg: cfg.clone(),
        broken: false,
        accumulated_impulse: 0.0,
    }
}

/// Returns a human-readable name for the given [`ImpulseJointType`].
#[allow(dead_code)]
pub fn joint_type_name(jt: ImpulseJointType) -> &'static str {
    match jt {
        ImpulseJointType::BallSocket => "ball_socket",
        ImpulseJointType::Hinge => "hinge",
        ImpulseJointType::Fixed => "fixed",
    }
}

/// Applies a velocity-level impulse to satisfy the joint constraint.
///
/// Modifies `vel_a` and `vel_b` in-place.  The impulse is proportional to
/// the relative velocity scaled by stiffness and damping.
#[allow(dead_code)]
pub fn apply_joint_impulse(
    joint: &ImpulseJoint,
    vel_a: &mut [f32; 3],
    vel_b: &mut [f32; 3],
    dt: f32,
) {
    if joint.broken {
        return;
    }
    // Relative velocity at the constraint point
    let rv = [
        vel_b[0] - vel_a[0],
        vel_b[1] - vel_a[1],
        vel_b[2] - vel_a[2],
    ];

    // Scale by stiffness and dt (simplified impulse)
    let scale = joint.cfg.stiffness * dt * 0.5;
    let damp = joint.cfg.damping * dt * 0.5;

    let impulse = [
        rv[0] * scale - vel_a[0] * damp,
        rv[1] * scale - vel_a[1] * damp,
        rv[2] * scale - vel_a[2] * damp,
    ];

    match joint.joint_type {
        ImpulseJointType::BallSocket | ImpulseJointType::Fixed => {
            // Apply full 3-D impulse
            vel_a[0] += impulse[0];
            vel_a[1] += impulse[1];
            vel_a[2] += impulse[2];
            vel_b[0] -= impulse[0];
            vel_b[1] -= impulse[1];
            vel_b[2] -= impulse[2];
        }
        ImpulseJointType::Hinge => {
            // Hinge constrains X and Z relative motion; Y is the free axis
            vel_a[0] += impulse[0];
            vel_a[2] += impulse[2];
            vel_b[0] -= impulse[0];
            vel_b[2] -= impulse[2];
        }
    }
}

/// Returns a scalar position-error for the joint (distance between anchors).
#[allow(dead_code)]
pub fn joint_error(joint: &ImpulseJoint, pos_a: [f32; 3], pos_b: [f32; 3]) -> f32 {
    let wa = add3(pos_a, joint.cfg.anchor_a);
    let wb = add3(pos_b, joint.cfg.anchor_b);
    dist3(wa, wb)
}

/// Returns the indices of the two bodies connected by the joint.
#[allow(dead_code)]
pub fn joint_bodies(joint: &ImpulseJoint) -> (usize, usize) {
    (joint.body_a, joint.body_b)
}

/// Updates the stiffness of an existing joint.
#[allow(dead_code)]
pub fn set_joint_stiffness(joint: &mut ImpulseJoint, stiffness: f32) {
    joint.cfg.stiffness = stiffness;
}

/// Updates the damping of an existing joint.
#[allow(dead_code)]
pub fn set_joint_damping(joint: &mut ImpulseJoint, damping: f32) {
    joint.cfg.damping = damping;
}

/// Returns `true` if the joint has been broken.
#[allow(dead_code)]
pub fn joint_is_broken(joint: &ImpulseJoint) -> bool {
    joint.broken
}

/// Forcibly breaks the joint.
#[allow(dead_code)]
pub fn break_joint_impulse(joint: &mut ImpulseJoint) {
    joint.broken = true;
}

// ──────────────────────────────────────────────────────────────────────────────
// Private helpers
// ──────────────────────────────────────────────────────────────────────────────

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_joint(t: ImpulseJointType) -> ImpulseJoint {
        let cfg = default_joint_config();
        new_impulse_joint(t, 0, 1, &cfg)
    }

    #[test]
    fn default_config_stiffness_positive() {
        let cfg = default_joint_config();
        assert!(cfg.stiffness > 0.0);
        assert!(cfg.damping >= 0.0);
    }

    #[test]
    fn joint_type_names() {
        assert_eq!(joint_type_name(ImpulseJointType::BallSocket), "ball_socket");
        assert_eq!(joint_type_name(ImpulseJointType::Hinge), "hinge");
        assert_eq!(joint_type_name(ImpulseJointType::Fixed), "fixed");
    }

    #[test]
    fn joint_bodies_correct() {
        let joint = make_joint(ImpulseJointType::BallSocket);
        assert_eq!(joint_bodies(&joint), (0, 1));
    }

    #[test]
    fn joint_not_broken_initially() {
        let joint = make_joint(ImpulseJointType::Fixed);
        assert!(!joint_is_broken(&joint));
    }

    #[test]
    fn break_joint_marks_broken() {
        let mut joint = make_joint(ImpulseJointType::Hinge);
        break_joint_impulse(&mut joint);
        assert!(joint_is_broken(&joint));
    }

    #[test]
    fn apply_impulse_modifies_velocities() {
        let joint = make_joint(ImpulseJointType::BallSocket);
        let mut va = [0.0_f32; 3];
        let mut vb = [1.0_f32, 0.0, 0.0];
        apply_joint_impulse(&joint, &mut va, &mut vb, 0.016);
        // va and vb should have changed
        let any_change = (va[0].abs() + va[1].abs() + va[2].abs()) > 0.0
            || (vb[0] - 1.0).abs() > 0.0;
        assert!(any_change);
    }

    #[test]
    fn broken_joint_skips_impulse() {
        let mut joint = make_joint(ImpulseJointType::Fixed);
        break_joint_impulse(&mut joint);
        let mut va = [0.0_f32; 3];
        let mut vb = [10.0_f32, 0.0, 0.0];
        apply_joint_impulse(&joint, &mut va, &mut vb, 0.016);
        // va should remain zero if joint is broken
        assert_eq!(va, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn joint_error_zero_at_same_pos() {
        let joint = make_joint(ImpulseJointType::BallSocket);
        let pos = [1.0, 2.0, 3.0];
        let err = joint_error(&joint, pos, pos);
        assert!(err.abs() < 1e-6);
    }

    #[test]
    fn set_stiffness_updates_value() {
        let mut joint = make_joint(ImpulseJointType::Hinge);
        set_joint_stiffness(&mut joint, 500.0);
        assert!((joint.cfg.stiffness - 500.0).abs() < 1e-9);
    }

    #[test]
    fn set_damping_updates_value() {
        let mut joint = make_joint(ImpulseJointType::Fixed);
        set_joint_damping(&mut joint, 20.0);
        assert!((joint.cfg.damping - 20.0).abs() < 1e-9);
    }
}
