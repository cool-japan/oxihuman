// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Computes centripetal force for circular motion.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CentripetalForce {
    pub mass: f32,
    pub radius: f32,
    pub angular_velocity: f32,
}

#[allow(dead_code)]
impl CentripetalForce {
    pub fn new(mass: f32, radius: f32, angular_velocity: f32) -> Self {
        Self {
            mass,
            radius,
            angular_velocity,
        }
    }

    pub fn from_linear_velocity(mass: f32, radius: f32, linear_velocity: f32) -> Self {
        let omega = if radius.abs() > 1e-9 {
            linear_velocity / radius
        } else {
            0.0
        };
        Self::new(mass, radius, omega)
    }

    pub fn magnitude(&self) -> f32 {
        self.mass * self.radius * self.angular_velocity * self.angular_velocity
    }

    pub fn linear_velocity(&self) -> f32 {
        self.radius * self.angular_velocity
    }

    pub fn period(&self) -> f32 {
        if self.angular_velocity.abs() < 1e-9 {
            return f32::INFINITY;
        }
        2.0 * PI / self.angular_velocity.abs()
    }

    pub fn frequency(&self) -> f32 {
        let p = self.period();
        if p.is_infinite() {
            0.0
        } else {
            1.0 / p
        }
    }

    pub fn acceleration(&self) -> f32 {
        self.radius * self.angular_velocity * self.angular_velocity
    }

    pub fn force_direction(&self, angle: f32) -> [f32; 3] {
        [-angle.cos(), 0.0, -angle.sin()]
    }

    pub fn force_vector(&self, angle: f32) -> [f32; 3] {
        let mag = self.magnitude();
        let dir = self.force_direction(angle);
        [dir[0] * mag, dir[1] * mag, dir[2] * mag]
    }

    pub fn kinetic_energy(&self) -> f32 {
        let v = self.linear_velocity();
        0.5 * self.mass * v * v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cf = CentripetalForce::new(1.0, 2.0, 3.0);
        assert!((cf.mass - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_magnitude() {
        let cf = CentripetalForce::new(2.0, 3.0, 4.0);
        // F = m * r * omega^2 = 2 * 3 * 16 = 96
        assert!((cf.magnitude() - 96.0).abs() < 1e-6);
    }

    #[test]
    fn test_linear_velocity() {
        let cf = CentripetalForce::new(1.0, 5.0, 2.0);
        assert!((cf.linear_velocity() - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_period() {
        let cf = CentripetalForce::new(1.0, 1.0, 2.0 * PI);
        assert!((cf.period() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_zero_angular_velocity() {
        let cf = CentripetalForce::new(1.0, 1.0, 0.0);
        assert!(cf.period().is_infinite());
        assert!(cf.frequency().abs() < 1e-9);
        assert!(cf.magnitude().abs() < 1e-9);
    }

    #[test]
    fn test_acceleration() {
        let cf = CentripetalForce::new(1.0, 2.0, 3.0);
        // a = r * omega^2 = 2 * 9 = 18
        assert!((cf.acceleration() - 18.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_linear_velocity() {
        let cf = CentripetalForce::from_linear_velocity(1.0, 5.0, 10.0);
        assert!((cf.angular_velocity - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_kinetic_energy() {
        let cf = CentripetalForce::new(2.0, 1.0, 3.0);
        let v = cf.linear_velocity();
        let expected = 0.5 * 2.0 * v * v;
        assert!((cf.kinetic_energy() - expected).abs() < 1e-6);
    }

    #[test]
    fn test_force_direction_magnitude() {
        let cf = CentripetalForce::new(1.0, 1.0, 1.0);
        let dir = cf.force_direction(0.0);
        let mag = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
        assert!((mag - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_frequency() {
        let cf = CentripetalForce::new(1.0, 1.0, 2.0 * PI);
        assert!((cf.frequency() - 1.0).abs() < 1e-5);
    }
}
