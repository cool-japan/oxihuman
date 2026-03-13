// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A body affected by gravity with configurable gravity scale.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct GravityBody {
    pub mass: f32,
    pub gravity_scale: f32,
    pub gravity_direction: [f32; 3],
    pub gravity_magnitude: f32,
    pub use_custom_gravity: bool,
}

#[allow(dead_code)]
impl GravityBody {
    pub fn new(mass: f32) -> Self {
        Self {
            mass,
            gravity_scale: 1.0,
            gravity_direction: [0.0, -1.0, 0.0],
            gravity_magnitude: 9.81,
            use_custom_gravity: false,
        }
    }

    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    pub fn with_custom_gravity(mut self, direction: [f32; 3], magnitude: f32) -> Self {
        self.gravity_direction = direction;
        self.gravity_magnitude = magnitude;
        self.use_custom_gravity = true;
        self
    }

    pub fn zero_gravity(mass: f32) -> Self {
        Self::new(mass).with_gravity_scale(0.0)
    }

    pub fn gravity_force(&self) -> [f32; 3] {
        let mag = self.mass * self.gravity_magnitude * self.gravity_scale;
        [
            self.gravity_direction[0] * mag,
            self.gravity_direction[1] * mag,
            self.gravity_direction[2] * mag,
        ]
    }

    pub fn weight(&self) -> f32 {
        self.mass * self.gravity_magnitude * self.gravity_scale
    }

    pub fn acceleration(&self) -> [f32; 3] {
        let mag = self.gravity_magnitude * self.gravity_scale;
        [
            self.gravity_direction[0] * mag,
            self.gravity_direction[1] * mag,
            self.gravity_direction[2] * mag,
        ]
    }

    pub fn potential_energy(&self, height: f32) -> f32 {
        self.mass * self.gravity_magnitude * self.gravity_scale * height
    }

    pub fn free_fall_velocity(&self, time: f32) -> f32 {
        self.gravity_magnitude * self.gravity_scale * time
    }

    pub fn free_fall_distance(&self, time: f32) -> f32 {
        0.5 * self.gravity_magnitude * self.gravity_scale * time * time
    }

    pub fn set_mass(&mut self, mass: f32) {
        self.mass = mass;
    }

    pub fn inv_mass(&self) -> f32 {
        if self.mass > 1e-9 {
            1.0 / self.mass
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let gb = GravityBody::new(10.0);
        assert!((gb.mass - 10.0).abs() < 1e-9);
        assert!((gb.gravity_scale - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_gravity_force() {
        let gb = GravityBody::new(2.0);
        let force = gb.gravity_force();
        assert!(force[1] < 0.0);
        assert!((force[1] - (-2.0 * 9.81)).abs() < 1e-4);
    }

    #[test]
    fn test_weight() {
        let gb = GravityBody::new(1.0);
        assert!((gb.weight() - 9.81).abs() < 1e-4);
    }

    #[test]
    fn test_zero_gravity() {
        let gb = GravityBody::zero_gravity(5.0);
        let force = gb.gravity_force();
        assert!(force[0].abs() < 1e-9);
        assert!(force[1].abs() < 1e-9);
    }

    #[test]
    fn test_gravity_scale() {
        let gb = GravityBody::new(1.0).with_gravity_scale(2.0);
        assert!((gb.weight() - 9.81 * 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_custom_gravity() {
        let gb = GravityBody::new(1.0).with_custom_gravity([1.0, 0.0, 0.0], 5.0);
        let force = gb.gravity_force();
        assert!((force[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_potential_energy() {
        let gb = GravityBody::new(2.0);
        let pe = gb.potential_energy(10.0);
        assert!((pe - 2.0 * 9.81 * 10.0).abs() < 1e-3);
    }

    #[test]
    fn test_free_fall_velocity() {
        let gb = GravityBody::new(1.0);
        let v = gb.free_fall_velocity(2.0);
        assert!((v - 9.81 * 2.0).abs() < 1e-3);
    }

    #[test]
    fn test_free_fall_distance() {
        let gb = GravityBody::new(1.0);
        let d = gb.free_fall_distance(2.0);
        assert!((d - 0.5 * 9.81 * 4.0).abs() < 1e-3);
    }

    #[test]
    fn test_inv_mass() {
        let gb = GravityBody::new(4.0);
        assert!((gb.inv_mass() - 0.25).abs() < 1e-9);
    }
}
