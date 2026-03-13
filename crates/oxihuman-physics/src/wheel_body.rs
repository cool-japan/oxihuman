// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A simple wheel/axle body for vehicle physics (torque, rolling).

use std::f32::consts::PI;

/// A single wheel body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WheelBody {
    pub radius: f32,
    pub mass: f32,
    /// Moment of inertia I = 0.5 * m * r^2 (solid cylinder).
    pub inertia: f32,
    /// Angular velocity (rad/s).
    pub angular_vel: f32,
    /// Angular position (rad).
    pub angle: f32,
    /// Rolling friction coefficient.
    pub rolling_friction: f32,
    /// Drive torque applied externally.
    pub drive_torque: f32,
    /// Brake torque (positive, opposes rotation).
    pub brake_torque: f32,
    pub time: f32,
    pub steps: u64,
}

#[allow(dead_code)]
impl WheelBody {
    pub fn new(radius: f32, mass: f32) -> Self {
        let r = radius.max(1e-6);
        let m = mass.max(1e-6);
        Self {
            radius: r,
            mass: m,
            inertia: 0.5 * m * r * r,
            angular_vel: 0.0,
            angle: 0.0,
            rolling_friction: 0.02,
            drive_torque: 0.0,
            brake_torque: 0.0,
            time: 0.0,
            steps: 0,
        }
    }

    /// Linear velocity at the rim (m/s).
    pub fn rim_speed(&self) -> f32 {
        self.angular_vel * self.radius
    }

    /// RPM.
    pub fn rpm(&self) -> f32 {
        self.angular_vel * 60.0 / (2.0 * PI)
    }

    /// Apply drive torque for next step.
    pub fn set_drive_torque(&mut self, torque: f32) {
        self.drive_torque = torque;
    }

    /// Apply brake torque (magnitude).
    pub fn set_brake_torque(&mut self, torque: f32) {
        self.brake_torque = torque.abs();
    }

    pub fn step(&mut self, dt: f32) {
        // Rolling friction torque (opposes rotation).
        let fric_torque = if self.angular_vel.abs() > 1e-6 {
            -self.rolling_friction * self.mass * 9.81 * self.radius * self.angular_vel.signum()
        } else {
            0.0
        };
        // Brake torque (opposes rotation, clamped).
        let brake = if self.angular_vel.abs() > 1e-6 {
            -self.brake_torque * self.angular_vel.signum()
        } else {
            0.0
        };
        let net_torque = self.drive_torque + fric_torque + brake;
        let alpha = net_torque / self.inertia;
        self.angular_vel += alpha * dt;
        // Clamp to zero if braking would reverse.
        if self.brake_torque > 0.0 && self.angular_vel.abs() < 0.01 {
            self.angular_vel = 0.0;
        }
        self.angle += self.angular_vel * dt;
        self.time += dt;
        self.steps += 1;
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.inertia * self.angular_vel * self.angular_vel
    }

    pub fn reset(&mut self) {
        self.angular_vel = 0.0;
        self.angle = 0.0;
        self.drive_torque = 0.0;
        self.brake_torque = 0.0;
        self.time = 0.0;
        self.steps = 0;
    }
}

impl Default for WheelBody {
    fn default() -> Self {
        Self::new(0.3, 10.0)
    }
}

pub fn new_wheel_body(radius: f32, mass: f32) -> WheelBody {
    WheelBody::new(radius, mass)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_torque_accelerates() {
        let mut w = new_wheel_body(0.3, 10.0);
        w.set_drive_torque(50.0);
        w.step(1.0);
        assert!(w.angular_vel > 0.0);
    }

    #[test]
    fn rim_speed_proportional() {
        let mut w = new_wheel_body(0.5, 5.0);
        w.set_drive_torque(20.0);
        w.step(1.0);
        assert!((w.rim_speed() - w.angular_vel * 0.5).abs() < 1e-5);
    }

    #[test]
    fn brake_reduces_speed() {
        let mut w = new_wheel_body(0.3, 10.0);
        w.angular_vel = 100.0;
        w.set_brake_torque(500.0);
        w.step(0.1);
        assert!(w.angular_vel < 100.0);
    }

    #[test]
    fn angle_advances() {
        let mut w = new_wheel_body(0.3, 5.0);
        w.angular_vel = 10.0;
        w.step(0.1);
        assert!(w.angle > 0.0);
    }

    #[test]
    fn rpm_consistent() {
        let mut w = new_wheel_body(0.3, 5.0);
        w.set_drive_torque(100.0);
        w.step(1.0);
        assert!((w.rpm() - w.angular_vel * 60.0 / (2.0 * PI)).abs() < 1e-4);
    }

    #[test]
    fn kinetic_energy_non_negative() {
        let mut w = new_wheel_body(0.3, 5.0);
        w.set_drive_torque(20.0);
        w.step(0.5);
        assert!(w.kinetic_energy() >= 0.0);
    }

    #[test]
    fn step_count() {
        let mut w = new_wheel_body(0.3, 5.0);
        w.step(0.01);
        w.step(0.01);
        assert_eq!(w.steps, 2);
    }

    #[test]
    fn rolling_friction_decelerates() {
        let mut w = new_wheel_body(0.3, 10.0);
        w.angular_vel = 50.0;
        w.step(1.0);
        assert!(w.angular_vel < 50.0);
    }

    #[test]
    fn reset_zeroes() {
        let mut w = new_wheel_body(0.3, 5.0);
        w.set_drive_torque(100.0);
        w.step(1.0);
        w.reset();
        assert!(w.angular_vel.abs() < 1e-6);
        assert_eq!(w.steps, 0);
    }

    #[test]
    fn inertia_computed_correctly() {
        let w = new_wheel_body(2.0, 4.0);
        assert!((w.inertia - 0.5 * 4.0 * 2.0 * 2.0).abs() < 1e-5);
    }
}
