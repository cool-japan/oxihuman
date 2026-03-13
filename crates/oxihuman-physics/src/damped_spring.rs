// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// A damped spring connecting two points.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct DampedSpring {
    rest_length: f32,
    stiffness: f32,
    damping: f32,
    max_force: f32,
}

#[allow(dead_code)]
impl DampedSpring {
    pub fn new(rest_length: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            rest_length: rest_length.max(0.0),
            stiffness,
            damping,
            max_force: f32::MAX,
        }
    }

    pub fn critically_damped(rest_length: f32, stiffness: f32, mass: f32) -> Self {
        let damping = 2.0 * (stiffness * mass).sqrt();
        Self::new(rest_length, stiffness, damping)
    }

    pub fn with_max_force(mut self, max_force: f32) -> Self {
        self.max_force = max_force;
        self
    }

    pub fn rest_length(&self) -> f32 {
        self.rest_length
    }

    pub fn stiffness(&self) -> f32 {
        self.stiffness
    }

    pub fn damping(&self) -> f32 {
        self.damping
    }

    pub fn force_1d(&self, displacement: f32, velocity: f32) -> f32 {
        let f = -self.stiffness * displacement - self.damping * velocity;
        f.clamp(-self.max_force, self.max_force)
    }

    pub fn force_3d(
        &self,
        pos_a: [f32; 3],
        pos_b: [f32; 3],
        vel_a: [f32; 3],
        vel_b: [f32; 3],
    ) -> [f32; 3] {
        let dx = pos_b[0] - pos_a[0];
        let dy = pos_b[1] - pos_a[1];
        let dz = pos_b[2] - pos_a[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist < 1e-9 {
            return [0.0; 3];
        }
        let nx = dx / dist;
        let ny = dy / dist;
        let nz = dz / dist;
        let stretch = dist - self.rest_length;
        let rel_vel =
            (vel_b[0] - vel_a[0]) * nx + (vel_b[1] - vel_a[1]) * ny + (vel_b[2] - vel_a[2]) * nz;
        let f = (self.stiffness * stretch + self.damping * rel_vel)
            .clamp(-self.max_force, self.max_force);
        [f * nx, f * ny, f * nz]
    }

    pub fn natural_frequency(&self, mass: f32) -> f32 {
        if mass <= 0.0 {
            return 0.0;
        }
        (self.stiffness / mass).sqrt() / (2.0 * PI)
    }

    pub fn damping_ratio(&self, mass: f32) -> f32 {
        let critical = 2.0 * (self.stiffness * mass).sqrt();
        if critical <= 0.0 {
            return 0.0;
        }
        self.damping / critical
    }

    pub fn energy(&self, displacement: f32) -> f32 {
        0.5 * self.stiffness * displacement * displacement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = DampedSpring::new(1.0, 100.0, 10.0);
        assert!((s.rest_length() - 1.0).abs() < 1e-6);
        assert!((s.stiffness() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_force_1d_compressed() {
        let s = DampedSpring::new(0.0, 100.0, 0.0);
        let f = s.force_1d(0.5, 0.0);
        assert!(f < 0.0);
    }

    #[test]
    fn test_force_1d_rest() {
        let s = DampedSpring::new(0.0, 100.0, 0.0);
        let f = s.force_1d(0.0, 0.0);
        assert!(f.abs() < 1e-6);
    }

    #[test]
    fn test_force_3d_rest() {
        let s = DampedSpring::new(1.0, 100.0, 10.0);
        let f = s.force_3d([0.0; 3], [1.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        for v in &f {
            assert!(v.abs() < 1e-4);
        }
    }

    #[test]
    fn test_force_3d_stretched() {
        let s = DampedSpring::new(1.0, 100.0, 0.0);
        let f = s.force_3d([0.0; 3], [2.0, 0.0, 0.0], [0.0; 3], [0.0; 3]);
        assert!(f[0] > 0.0);
    }

    #[test]
    fn test_force_3d_zero_dist() {
        let s = DampedSpring::new(1.0, 100.0, 0.0);
        let f = s.force_3d([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn test_critically_damped() {
        let s = DampedSpring::critically_damped(1.0, 100.0, 1.0);
        let ratio = s.damping_ratio(1.0);
        assert!((ratio - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_natural_frequency() {
        let s = DampedSpring::new(0.0, 100.0, 0.0);
        let f = s.natural_frequency(1.0);
        assert!(f > 0.0);
    }

    #[test]
    fn test_energy() {
        let s = DampedSpring::new(0.0, 200.0, 0.0);
        let e = s.energy(1.0);
        assert!((e - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_force() {
        let s = DampedSpring::new(0.0, 100.0, 0.0).with_max_force(10.0);
        let f = s.force_1d(1.0, 0.0);
        assert!(f.abs() <= 10.0 + 1e-6);
    }
}
