// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A motor constraint that drives a joint to a target angle or velocity.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct MotorConstraint {
    target_angle: f32,
    target_velocity: f32,
    max_torque: f32,
    stiffness: f32,
    damping: f32,
    mode: MotorMode,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotorMode {
    Position,
    Velocity,
}

#[allow(dead_code)]
impl MotorConstraint {
    pub fn position_motor(target_angle: f32, max_torque: f32) -> Self {
        Self {
            target_angle,
            target_velocity: 0.0,
            max_torque,
            stiffness: 100.0,
            damping: 10.0,
            mode: MotorMode::Position,
        }
    }

    pub fn velocity_motor(target_velocity: f32, max_torque: f32) -> Self {
        Self {
            target_angle: 0.0,
            target_velocity,
            max_torque,
            stiffness: 0.0,
            damping: 10.0,
            mode: MotorMode::Velocity,
        }
    }

    pub fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    pub fn mode(&self) -> MotorMode {
        self.mode
    }

    pub fn max_torque(&self) -> f32 {
        self.max_torque
    }

    pub fn compute_torque(&self, current_angle: f32, current_velocity: f32) -> f32 {
        let torque = match self.mode {
            MotorMode::Position => {
                let error = self.target_angle - current_angle;
                let spring = error * self.stiffness;
                let damp = -current_velocity * self.damping;
                spring + damp
            }
            MotorMode::Velocity => {
                let error = self.target_velocity - current_velocity;
                error * self.damping
            }
        };
        torque.clamp(-self.max_torque, self.max_torque)
    }

    pub fn set_target_angle(&mut self, angle: f32) {
        self.target_angle = angle;
    }

    pub fn set_target_velocity(&mut self, velocity: f32) {
        self.target_velocity = velocity;
    }

    pub fn target_angle(&self) -> f32 {
        self.target_angle
    }

    pub fn target_velocity(&self) -> f32 {
        self.target_velocity
    }

    pub fn power(&self, current_angle: f32, current_velocity: f32) -> f32 {
        let torque = self.compute_torque(current_angle, current_velocity);
        torque * current_velocity
    }

    pub fn is_at_target(&self, current_angle: f32, tolerance: f32) -> bool {
        match self.mode {
            MotorMode::Position => (self.target_angle - current_angle).abs() <= tolerance,
            MotorMode::Velocity => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_position_motor() {
        let m = MotorConstraint::position_motor(1.0, 50.0);
        assert_eq!(m.mode(), MotorMode::Position);
        assert!((m.max_torque() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_velocity_motor() {
        let m = MotorConstraint::velocity_motor(5.0, 100.0);
        assert_eq!(m.mode(), MotorMode::Velocity);
    }

    #[test]
    fn test_position_torque_toward_target() {
        let m = MotorConstraint::position_motor(PI / 2.0, 1000.0);
        let torque = m.compute_torque(0.0, 0.0);
        assert!(torque > 0.0);
    }

    #[test]
    fn test_position_torque_at_target() {
        let m = MotorConstraint::position_motor(1.0, 100.0);
        let torque = m.compute_torque(1.0, 0.0);
        assert!(torque.abs() < 1e-5);
    }

    #[test]
    fn test_velocity_torque() {
        let m = MotorConstraint::velocity_motor(10.0, 100.0);
        let torque = m.compute_torque(0.0, 0.0);
        assert!(torque > 0.0);
    }

    #[test]
    fn test_torque_clamping() {
        let m = MotorConstraint::position_motor(100.0, 5.0).with_stiffness(1000.0);
        let torque = m.compute_torque(0.0, 0.0);
        assert!((torque - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_damping() {
        let m = MotorConstraint::position_motor(0.0, 100.0).with_damping(20.0);
        let torque = m.compute_torque(0.0, 5.0);
        assert!(torque < 0.0); // damping opposes velocity
    }

    #[test]
    fn test_set_target_angle() {
        let mut m = MotorConstraint::position_motor(0.0, 100.0);
        m.set_target_angle(PI);
        assert!((m.target_angle() - PI).abs() < 1e-9);
    }

    #[test]
    fn test_is_at_target() {
        let m = MotorConstraint::position_motor(1.0, 100.0);
        assert!(m.is_at_target(1.0, 0.01));
        assert!(!m.is_at_target(2.0, 0.01));
    }

    #[test]
    fn test_power() {
        let m = MotorConstraint::velocity_motor(10.0, 100.0);
        let power = m.power(0.0, 5.0);
        // torque = (10 - 5) * 10 = 50, power = 50 * 5 = 250
        assert!((power - 250.0).abs() < 1e-3);
    }
}
