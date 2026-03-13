#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Motor-driven constraint for joints.

/// A motor that drives a constraint to a target velocity.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstraintMotor {
    pub target_velocity: f32,
    pub max_force: f32,
    pub active: bool,
    pub output: f32,
}

#[allow(dead_code)]
pub fn new_constraint_motor(target_velocity: f32, max_force: f32) -> ConstraintMotor {
    ConstraintMotor {
        target_velocity,
        max_force,
        active: true,
        output: 0.0,
    }
}

#[allow(dead_code)]
pub fn motor_target_velocity(motor: &ConstraintMotor) -> f32 {
    motor.target_velocity
}

#[allow(dead_code)]
pub fn motor_max_force(motor: &ConstraintMotor) -> f32 {
    motor.max_force
}

#[allow(dead_code)]
pub fn motor_solve(motor: &mut ConstraintMotor, current_velocity: f32, dt: f32) -> f32 {
    if !motor.active || dt <= 0.0 {
        motor.output = 0.0;
        return 0.0;
    }
    let error = motor.target_velocity - current_velocity;
    let force = error / dt;
    let clamped = force.clamp(-motor.max_force, motor.max_force);
    motor.output = clamped;
    clamped
}

#[allow(dead_code)]
pub fn motor_is_active(motor: &ConstraintMotor) -> bool {
    motor.active
}

#[allow(dead_code)]
pub fn motor_enable(motor: &mut ConstraintMotor) {
    motor.active = true;
}

#[allow(dead_code)]
pub fn motor_disable(motor: &mut ConstraintMotor) {
    motor.active = false;
    motor.output = 0.0;
}

#[allow(dead_code)]
pub fn motor_output(motor: &ConstraintMotor) -> f32 {
    motor.output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_constraint_motor(5.0, 100.0);
        assert!((m.target_velocity - 5.0).abs() < 1e-6);
        assert!(motor_is_active(&m));
    }

    #[test]
    fn test_target_velocity() {
        let m = new_constraint_motor(10.0, 50.0);
        assert!((motor_target_velocity(&m) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_force() {
        let m = new_constraint_motor(10.0, 50.0);
        assert!((motor_max_force(&m) - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_solve() {
        let mut m = new_constraint_motor(10.0, 100.0);
        let f = motor_solve(&mut m, 0.0, 1.0);
        assert!((f - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_solve_clamped() {
        let mut m = new_constraint_motor(1000.0, 5.0);
        let f = motor_solve(&mut m, 0.0, 1.0);
        assert!((f - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_disable() {
        let mut m = new_constraint_motor(10.0, 100.0);
        motor_disable(&mut m);
        assert!(!motor_is_active(&m));
        let f = motor_solve(&mut m, 0.0, 1.0);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn test_enable() {
        let mut m = new_constraint_motor(10.0, 100.0);
        motor_disable(&mut m);
        motor_enable(&mut m);
        assert!(motor_is_active(&m));
    }

    #[test]
    fn test_output() {
        let mut m = new_constraint_motor(5.0, 100.0);
        motor_solve(&mut m, 0.0, 1.0);
        assert!((motor_output(&m) - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_zero_dt() {
        let mut m = new_constraint_motor(10.0, 100.0);
        let f = motor_solve(&mut m, 0.0, 0.0);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn test_negative_error() {
        let mut m = new_constraint_motor(-5.0, 100.0);
        let f = motor_solve(&mut m, 0.0, 1.0);
        assert!(f < 0.0);
    }
}
