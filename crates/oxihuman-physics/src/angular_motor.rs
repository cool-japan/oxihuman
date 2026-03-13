// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Angular motor: drives a joint to a target angle with torque limits.

use std::f32::consts::PI;

/// Angular motor state.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AngularMotor {
    pub target_angle: f32,
    pub current_angle: f32,
    pub angular_velocity: f32,
    pub max_torque: f32,
    pub stiffness: f32,
    pub damping: f32,
}

/// Create a new AngularMotor.
#[allow(dead_code)]
pub fn new_angular_motor(
    target_angle: f32,
    max_torque: f32,
    stiffness: f32,
    damping: f32,
) -> AngularMotor {
    AngularMotor {
        target_angle,
        current_angle: 0.0,
        angular_velocity: 0.0,
        max_torque,
        stiffness,
        damping,
    }
}

/// Compute the drive torque for this motor.
#[allow(dead_code)]
pub fn motor_torque(m: &AngularMotor) -> f32 {
    let error = angle_diff(m.target_angle, m.current_angle);
    let torque = m.stiffness * error - m.damping * m.angular_velocity;
    torque.clamp(-m.max_torque, m.max_torque)
}

/// Integrate motor state one timestep.
#[allow(dead_code)]
pub fn motor_step(m: &mut AngularMotor, inertia: f32, dt: f32) {
    let torque = motor_torque(m);
    let alpha = torque / inertia.max(1e-6);
    m.angular_velocity += alpha * dt;
    m.current_angle += m.angular_velocity * dt;
}

/// Shortest signed angular difference (target - current) in [-PI, PI].
#[allow(dead_code)]
pub fn angle_diff(target: f32, current: f32) -> f32 {
    let raw = target - current;
    wrap_angle(raw)
}

/// Wrap angle to [-PI, PI].
#[allow(dead_code)]
pub fn wrap_angle(a: f32) -> f32 {
    let mut r = a;
    while r > PI {
        r -= 2.0 * PI;
    }
    while r < -PI {
        r += 2.0 * PI;
    }
    r
}

/// Whether motor is within tolerance of target.
#[allow(dead_code)]
pub fn motor_at_target(m: &AngularMotor, tol: f32) -> bool {
    angle_diff(m.target_angle, m.current_angle).abs() < tol
}

/// Set a new target angle.
#[allow(dead_code)]
pub fn motor_set_target(m: &mut AngularMotor, angle: f32) {
    m.target_angle = wrap_angle(angle);
}

/// Kinetic energy of the rotating mass.
#[allow(dead_code)]
pub fn motor_kinetic_energy(m: &AngularMotor, inertia: f32) -> f32 {
    0.5 * inertia * m.angular_velocity * m.angular_velocity
}

/// Critical damping coefficient for given stiffness and inertia.
#[allow(dead_code)]
pub fn critical_damping(stiffness: f32, inertia: f32) -> f32 {
    2.0 * (stiffness * inertia).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torque_direction() {
        let m = new_angular_motor(PI * 0.5, 100.0, 50.0, 5.0);
        // target > current → positive torque
        assert!(motor_torque(&m) > 0.0);
    }

    #[test]
    fn test_torque_clamped() {
        let m = new_angular_motor(PI, 1.0, 1000.0, 0.0);
        assert!((motor_torque(&m).abs() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_angle_diff_wrap() {
        let d = angle_diff(0.1, 2.0 * PI - 0.1);
        assert!(d.abs() < 0.3);
    }

    #[test]
    fn test_wrap_angle() {
        // PI + 0.5 wraps to -(PI - 0.5) ≈ -2.64, which is in (-PI, 0)
        let a = wrap_angle(PI + 0.5);
        assert!((-PI..=PI).contains(&a));
        assert!(a < 0.0);
    }

    #[test]
    fn test_motor_step_moves_toward_target() {
        let mut m = new_angular_motor(1.0, 100.0, 50.0, 2.0);
        let initial = m.current_angle;
        motor_step(&mut m, 1.0, 0.1);
        assert!(m.current_angle > initial);
    }

    #[test]
    fn test_at_target() {
        let mut m = new_angular_motor(0.0, 100.0, 50.0, 5.0);
        m.current_angle = 0.0;
        assert!(motor_at_target(&m, 0.01));
    }

    #[test]
    fn test_kinetic_energy() {
        let mut m = new_angular_motor(0.0, 10.0, 1.0, 0.0);
        m.angular_velocity = 2.0;
        let ke = motor_kinetic_energy(&m, 1.0);
        assert!((ke - 2.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_critical_damping() {
        let cd = critical_damping(100.0, 1.0);
        assert!((cd - 20.0_f32).abs() < 1e-4);
    }

    #[test]
    fn test_set_target() {
        let mut m = new_angular_motor(0.0, 10.0, 1.0, 0.1);
        motor_set_target(&mut m, 3.0 * PI);
        assert!(m.target_angle.abs() <= PI + 1e-4);
    }
}
