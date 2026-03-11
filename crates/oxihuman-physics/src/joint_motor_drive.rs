// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A motor that drives a joint to a target position or velocity.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DriveMode {
    Position,
    Velocity,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct JointMotorDrive {
    mode: DriveMode,
    target: f32,
    max_force: f32,
    stiffness: f32,
    damping: f32,
    enabled: bool,
}

#[allow(dead_code)]
impl JointMotorDrive {
    pub fn new(mode: DriveMode, target: f32) -> Self {
        Self {
            mode,
            target,
            max_force: 1000.0,
            stiffness: 500.0,
            damping: 50.0,
            enabled: true,
        }
    }

    pub fn position_drive(target: f32) -> Self {
        Self::new(DriveMode::Position, target)
    }

    pub fn velocity_drive(target: f32) -> Self {
        Self::new(DriveMode::Velocity, target)
    }

    pub fn with_max_force(mut self, max_force: f32) -> Self {
        self.max_force = max_force;
        self
    }

    pub fn with_stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    pub fn with_damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    pub fn mode(&self) -> DriveMode {
        self.mode
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    pub fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn compute_force(&self, current_pos: f32, current_vel: f32) -> f32 {
        if !self.enabled {
            return 0.0;
        }
        let f = match self.mode {
            DriveMode::Position => {
                let error = self.target - current_pos;
                error * self.stiffness - current_vel * self.damping
            }
            DriveMode::Velocity => {
                let vel_error = self.target - current_vel;
                vel_error * self.damping
            }
        };
        f.clamp(-self.max_force, self.max_force)
    }

    pub fn at_target(&self, current: f32, tolerance: f32) -> bool {
        (current - self.target).abs() <= tolerance
    }

    pub fn target_angle_deg(degrees: f32) -> Self {
        Self::position_drive(degrees * PI / 180.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_drive() {
        let d = JointMotorDrive::position_drive(1.0);
        assert_eq!(d.mode(), DriveMode::Position);
        assert!((d.target() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_velocity_drive() {
        let d = JointMotorDrive::velocity_drive(2.0);
        assert_eq!(d.mode(), DriveMode::Velocity);
    }

    #[test]
    fn test_compute_force_position_at_target() {
        let d = JointMotorDrive::position_drive(1.0);
        let f = d.compute_force(1.0, 0.0);
        assert!(f.abs() < 1e-4);
    }

    #[test]
    fn test_compute_force_position_error() {
        let d = JointMotorDrive::position_drive(1.0);
        let f = d.compute_force(0.0, 0.0);
        assert!(f > 0.0);
    }

    #[test]
    fn test_compute_force_velocity() {
        let d = JointMotorDrive::velocity_drive(5.0);
        let f = d.compute_force(0.0, 0.0);
        assert!(f > 0.0);
    }

    #[test]
    fn test_disabled() {
        let mut d = JointMotorDrive::position_drive(1.0);
        d.disable();
        let f = d.compute_force(0.0, 0.0);
        assert!(f.abs() < 1e-6);
    }

    #[test]
    fn test_max_force_clamp() {
        let d = JointMotorDrive::position_drive(1000.0).with_max_force(10.0);
        let f = d.compute_force(0.0, 0.0);
        assert!(f.abs() <= 10.0 + 1e-6);
    }

    #[test]
    fn test_set_target() {
        let mut d = JointMotorDrive::position_drive(0.0);
        d.set_target(5.0);
        assert!((d.target() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_at_target() {
        let d = JointMotorDrive::position_drive(1.0);
        assert!(d.at_target(1.005, 0.01));
        assert!(!d.at_target(2.0, 0.01));
    }

    #[test]
    fn test_target_angle_deg() {
        let d = JointMotorDrive::target_angle_deg(90.0);
        assert!((d.target() - PI / 2.0).abs() < 1e-4);
    }
}
