// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Revolute (hinge) joint with limits and motor.

/// Revolute joint rotating around a fixed axis.
#[derive(Debug, Clone)]
pub struct RevoluteJoint {
    pub angle: f32,
    pub angular_velocity: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub damping: f32,
    pub motor_torque: f32,
    pub motor_target_vel: f32,
    pub motor_enabled: bool,
}

#[allow(dead_code)]
impl RevoluteJoint {
    pub fn new(min_angle: f32, max_angle: f32) -> Self {
        RevoluteJoint {
            angle: 0.0,
            angular_velocity: 0.0,
            min_angle,
            max_angle,
            damping: 0.0,
            motor_torque: 0.0,
            motor_target_vel: 0.0,
            motor_enabled: false,
        }
    }

    pub fn step(&mut self, dt: f32, inertia: f32) {
        let mut torque = -self.angular_velocity * self.damping;
        if self.motor_enabled {
            let vel_err = self.motor_target_vel - self.angular_velocity;
            torque += vel_err.signum() * self.motor_torque.abs().min(vel_err.abs() * inertia / dt);
        }
        self.angular_velocity += torque / inertia * dt;
        self.angle += self.angular_velocity * dt;
        self.clamp_to_limits();
    }

    pub fn clamp_to_limits(&mut self) {
        if self.angle < self.min_angle {
            self.angle = self.min_angle;
            if self.angular_velocity < 0.0 {
                self.angular_velocity = 0.0;
            }
        } else if self.angle > self.max_angle {
            self.angle = self.max_angle;
            if self.angular_velocity > 0.0 {
                self.angular_velocity = 0.0;
            }
        }
    }

    pub fn is_at_lower_limit(&self) -> bool {
        self.angle <= self.min_angle + 1e-6
    }

    pub fn is_at_upper_limit(&self) -> bool {
        self.angle >= self.max_angle - 1e-6
    }

    pub fn range(&self) -> f32 {
        self.max_angle - self.min_angle
    }

    pub fn set_motor(&mut self, target_vel: f32, max_torque: f32) {
        self.motor_enabled = true;
        self.motor_target_vel = target_vel;
        self.motor_torque = max_torque;
    }

    pub fn angle_deg(&self) -> f32 {
        self.angle.to_degrees()
    }

    pub fn normalized_angle(&self) -> f32 {
        if self.range().abs() < 1e-6 {
            return 0.0;
        }
        (self.angle - self.min_angle) / self.range()
    }
}

pub fn new_revolute_joint(min_deg: f32, max_deg: f32) -> RevoluteJoint {
    RevoluteJoint::new(min_deg.to_radians(), max_deg.to_radians())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn initial_zero() {
        let j = new_revolute_joint(-90.0, 90.0);
        assert_eq!(j.angle, 0.0);
    }

    #[test]
    fn range_calculation() {
        let j = new_revolute_joint(-90.0, 90.0);
        assert!((j.range() - PI).abs() < 1e-5);
    }

    #[test]
    fn clamp_lower_limit() {
        let mut j = new_revolute_joint(-45.0, 45.0);
        j.angle = -1.0;
        j.angular_velocity = -1.0;
        j.clamp_to_limits();
        assert!(j.is_at_lower_limit());
        assert!(j.angular_velocity >= 0.0);
    }

    #[test]
    fn clamp_upper_limit() {
        let mut j = new_revolute_joint(-45.0, 45.0);
        j.angle = 2.0;
        j.angular_velocity = 1.0;
        j.clamp_to_limits();
        assert!(j.is_at_upper_limit());
        assert!(j.angular_velocity <= 0.0);
    }

    #[test]
    fn motor_accelerates() {
        let mut j = new_revolute_joint(-90.0, 90.0);
        j.set_motor(1.0, 10.0);
        j.step(0.1, 1.0);
        assert!(j.angular_velocity > 0.0);
    }

    #[test]
    fn damping_reduces_velocity() {
        let mut j = new_revolute_joint(-90.0, 90.0);
        j.angular_velocity = 10.0;
        j.damping = 5.0;
        j.step(0.1, 1.0);
        assert!(j.angular_velocity < 10.0);
    }

    #[test]
    fn angle_deg_conversion() {
        let mut j = new_revolute_joint(-180.0, 180.0);
        j.angle = PI;
        assert!((j.angle_deg() - 180.0).abs() < 1e-4);
    }

    #[test]
    fn normalized_angle_mid() {
        let mut j = new_revolute_joint(-90.0, 90.0);
        j.angle = 0.0;
        assert!((j.normalized_angle() - 0.5).abs() < 1e-4);
    }

    #[test]
    fn step_integrates_angle() {
        let mut j = new_revolute_joint(-360.0, 360.0);
        j.angular_velocity = 1.0;
        j.step(1.0, 1.0);
        assert!(j.angle > 0.0);
    }
}
