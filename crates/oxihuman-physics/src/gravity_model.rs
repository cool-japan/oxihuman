// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Models gravitational force with configurable direction and magnitude.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct GravityModel {
    direction: [f32; 3],
    magnitude: f32,
    enabled: bool,
}

#[allow(dead_code)]
impl GravityModel {
    pub fn new(direction: [f32; 3], magnitude: f32) -> Self {
        Self {
            direction,
            magnitude,
            enabled: true,
        }
    }

    pub fn earth() -> Self {
        Self::new([0.0, -1.0, 0.0], 9.81)
    }

    pub fn moon() -> Self {
        Self::new([0.0, -1.0, 0.0], 1.62)
    }

    pub fn zero() -> Self {
        Self::new([0.0, 0.0, 0.0], 0.0)
    }

    pub fn custom_down(magnitude: f32) -> Self {
        Self::new([0.0, -1.0, 0.0], magnitude)
    }

    pub fn direction(&self) -> [f32; 3] {
        self.direction
    }

    pub fn magnitude(&self) -> f32 {
        self.magnitude
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn set_magnitude(&mut self, magnitude: f32) {
        self.magnitude = magnitude;
    }

    pub fn set_direction(&mut self, direction: [f32; 3]) {
        self.direction = direction;
    }

    pub fn acceleration(&self) -> [f32; 3] {
        if !self.enabled {
            return [0.0; 3];
        }
        let len = (self.direction[0] * self.direction[0]
            + self.direction[1] * self.direction[1]
            + self.direction[2] * self.direction[2])
            .sqrt();
        if len < 1e-9 {
            return [0.0; 3];
        }
        [
            self.direction[0] / len * self.magnitude,
            self.direction[1] / len * self.magnitude,
            self.direction[2] / len * self.magnitude,
        ]
    }

    pub fn force(&self, mass: f32) -> [f32; 3] {
        let acc = self.acceleration();
        [acc[0] * mass, acc[1] * mass, acc[2] * mass]
    }

    pub fn potential_energy(&self, mass: f32, height: f32) -> f32 {
        mass * self.magnitude * height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_earth() {
        let g = GravityModel::earth();
        assert!((g.magnitude() - 9.81).abs() < 1e-4);
    }

    #[test]
    fn test_moon() {
        let g = GravityModel::moon();
        assert!((g.magnitude() - 1.62).abs() < 1e-4);
    }

    #[test]
    fn test_zero() {
        let g = GravityModel::zero();
        let acc = g.acceleration();
        for v in &acc {
            assert!(v.abs() < 1e-6);
        }
    }

    #[test]
    fn test_acceleration_earth() {
        let g = GravityModel::earth();
        let acc = g.acceleration();
        assert!((acc[1] - (-9.81)).abs() < 1e-3);
    }

    #[test]
    fn test_force() {
        let g = GravityModel::earth();
        let f = g.force(10.0);
        assert!((f[1] - (-98.1)).abs() < 0.1);
    }

    #[test]
    fn test_disabled() {
        let mut g = GravityModel::earth();
        g.disable();
        let acc = g.acceleration();
        assert_eq!(acc, [0.0; 3]);
    }

    #[test]
    fn test_enable() {
        let mut g = GravityModel::earth();
        g.disable();
        g.enable();
        assert!(g.is_enabled());
        assert!(g.acceleration()[1] < 0.0);
    }

    #[test]
    fn test_potential_energy() {
        let g = GravityModel::earth();
        let pe = g.potential_energy(1.0, 10.0);
        assert!((pe - 98.1).abs() < 0.1);
    }

    #[test]
    fn test_custom_down() {
        let g = GravityModel::custom_down(5.0);
        let acc = g.acceleration();
        assert!((acc[1] - (-5.0)).abs() < 1e-4);
    }

    #[test]
    fn test_set_magnitude() {
        let mut g = GravityModel::earth();
        g.set_magnitude(20.0);
        assert!((g.magnitude() - 20.0).abs() < 1e-6);
    }
}
