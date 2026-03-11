//! Motor / servo joint constraint.
//!
//! Models a driven rotational joint that tries to reach a target angular
//! velocity subject to a maximum torque limit.  Suitable for servo motors,
//! electric drives, and powered hinges in rigid-body simulations.

// ── public structs ────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Static configuration for a motor joint.
pub struct MotorJointConfig {
    /// Desired angular velocity (rad/s).
    pub target_velocity: f32,
    /// Maximum torque the motor can apply (N·m).
    pub max_torque: f32,
    /// Damping applied to relative velocity (friction-like).
    pub damping: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
/// Runtime state of a motor joint between two bodies.
pub struct MotorJoint {
    /// Index of body A.
    pub body_a: usize,
    /// Index of body B.
    pub body_b: usize,
    /// Target angular velocity (rad/s).
    pub target_velocity: f32,
    /// Maximum torque (N·m).
    pub max_torque: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Torque applied during the last constraint solve (N·m).
    pub last_torque: f32,
    /// Whether the motor is active.
    pub enabled: bool,
}

// ── public functions ──────────────────────────────────────────────────────────

#[allow(dead_code)]
/// Returns a [`MotorJointConfig`] with sensible defaults.
pub fn default_motor_joint_config() -> MotorJointConfig {
    MotorJointConfig {
        target_velocity: 0.0,
        max_torque: 100.0,
        damping: 5.0,
    }
}

#[allow(dead_code)]
/// Creates a new [`MotorJoint`] connecting `body_a` and `body_b`.
pub fn new_motor_joint(body_a: usize, body_b: usize, cfg: &MotorJointConfig) -> MotorJoint {
    MotorJoint {
        body_a,
        body_b,
        target_velocity: cfg.target_velocity,
        max_torque: cfg.max_torque,
        damping: cfg.damping,
        last_torque: 0.0,
        enabled: true,
    }
}

#[allow(dead_code)]
/// Sets the target angular velocity on the joint.
pub fn set_motor_target_velocity(joint: &mut MotorJoint, vel: f32) {
    joint.target_velocity = vel;
}

#[allow(dead_code)]
/// Sets the maximum torque on the joint.
pub fn set_motor_max_torque(joint: &mut MotorJoint, torque: f32) {
    joint.max_torque = torque.max(0.0);
}

#[allow(dead_code)]
/// Applies the motor torque impulse to the angular velocities of both bodies.
///
/// `vel_a` and `vel_b` are the angular velocities (rad/s) of body A and B
/// about the joint axis.  The motor drives the relative velocity
/// (vel_b − vel_a) towards `target_velocity`.
pub fn apply_motor_torque(joint: &MotorJoint, vel_a: &mut f32, vel_b: &mut f32, dt: f32) {
    if !joint.enabled || dt <= 0.0 {
        return;
    }

    let rel_vel = *vel_b - *vel_a;
    let vel_error = joint.target_velocity - rel_vel;

    // Proportional torque request
    let torque_request = vel_error * joint.damping;

    // Clamp to max torque
    let torque = torque_request.clamp(-joint.max_torque, joint.max_torque);

    // Apply as impulse (assuming unit inertia for simplicity;
    // real usage would divide by moment of inertia)
    let impulse = torque * dt;
    *vel_a -= impulse * 0.5;
    *vel_b += impulse * 0.5;
}

#[allow(dead_code)]
/// Returns `true` if the current velocity is within `tol` of the target.
pub fn motor_is_at_target(joint: &MotorJoint, current_vel: f32, tol: f32) -> bool {
    (current_vel - joint.target_velocity).abs() <= tol.abs()
}

#[allow(dead_code)]
/// Returns the indices of the two bodies connected by this joint.
pub fn motor_joint_bodies(joint: &MotorJoint) -> (usize, usize) {
    (joint.body_a, joint.body_b)
}

#[allow(dead_code)]
/// Returns the torque applied during the last constraint solve.
pub fn motor_joint_torque(joint: &MotorJoint) -> f32 {
    joint.last_torque
}

#[allow(dead_code)]
/// Enables or disables the motor joint.
pub fn set_motor_enabled(joint: &mut MotorJoint, enabled: bool) {
    joint.enabled = enabled;
}

#[allow(dead_code)]
/// Returns `true` if the motor joint is currently enabled.
pub fn motor_joint_is_enabled(joint: &MotorJoint) -> bool {
    joint.enabled
}

// ── unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_joint() -> MotorJoint {
        new_motor_joint(0, 1, &default_motor_joint_config())
    }

    #[test]
    fn test_default_config_max_torque() {
        let cfg = default_motor_joint_config();
        assert!(cfg.max_torque > 0.0);
    }

    #[test]
    fn test_new_motor_joint_bodies() {
        let j = make_joint();
        assert_eq!(motor_joint_bodies(&j), (0, 1));
    }

    #[test]
    fn test_motor_joint_enabled_by_default() {
        let j = make_joint();
        assert!(motor_joint_is_enabled(&j));
    }

    #[test]
    fn test_set_motor_enabled_false() {
        let mut j = make_joint();
        set_motor_enabled(&mut j, false);
        assert!(!motor_joint_is_enabled(&j));
    }

    #[test]
    fn test_apply_motor_torque_drives_velocity() {
        let mut j = make_joint();
        set_motor_target_velocity(&mut j, 10.0);
        let mut va = 0.0_f32;
        let mut vb = 0.0_f32;
        apply_motor_torque(&j, &mut va, &mut vb, 0.1);
        // vb should have increased (motor drives body_b faster)
        assert!(vb > 0.0, "vb should increase: {vb}");
    }

    #[test]
    fn test_apply_motor_torque_disabled_no_effect() {
        let mut j = make_joint();
        set_motor_target_velocity(&mut j, 10.0);
        set_motor_enabled(&mut j, false);
        let mut va = 0.0_f32;
        let mut vb = 0.0_f32;
        apply_motor_torque(&j, &mut va, &mut vb, 0.1);
        assert_eq!(va, 0.0);
        assert_eq!(vb, 0.0);
    }

    #[test]
    fn test_motor_is_at_target_true() {
        let j = make_joint(); // target = 0.0
        assert!(motor_is_at_target(&j, 0.05, 0.1));
    }

    #[test]
    fn test_motor_is_at_target_false() {
        let mut j = make_joint();
        set_motor_target_velocity(&mut j, 5.0);
        assert!(!motor_is_at_target(&j, 0.0, 0.1));
    }

    #[test]
    fn test_set_motor_max_torque_clamps_negative() {
        let mut j = make_joint();
        set_motor_max_torque(&mut j, -50.0);
        assert_eq!(j.max_torque, 0.0, "negative torque should clamp to 0");
    }
}
