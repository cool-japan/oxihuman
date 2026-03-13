// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Anisotropic spring (different stiffness per axis).

#![allow(dead_code)]

/// Anisotropic spring with per-axis stiffness and damping.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnisotropicSpring {
    pub stiffness: [f64; 3],
    pub damping: [f64; 3],
    pub rest_length: [f64; 3],
    pub displacement: [f64; 3],
    pub velocity: [f64; 3],
}

impl AnisotropicSpring {
    #[allow(dead_code)]
    pub fn new(stiffness: [f64; 3], damping: [f64; 3]) -> Self {
        Self {
            stiffness,
            damping,
            rest_length: [0.0; 3],
            displacement: [0.0; 3],
            velocity: [0.0; 3],
        }
    }

    #[allow(dead_code)]
    pub fn with_rest_length(mut self, rest: [f64; 3]) -> Self {
        self.rest_length = rest;
        self
    }

    /// Compute restoring force per axis.
    #[allow(dead_code)]
    #[allow(clippy::needless_range_loop)]
    pub fn force(&self) -> [f64; 3] {
        let mut f = [0.0f64; 3];
        for i in 0..3 {
            let d = self.displacement[i] - self.rest_length[i];
            f[i] = -self.stiffness[i] * d - self.damping[i] * self.velocity[i];
        }
        f
    }

    #[allow(dead_code)]
    #[allow(clippy::needless_range_loop)]
    pub fn step(&mut self, mass: f64, dt: f64) {
        let f = self.force();
        for i in 0..3 {
            let acc = f[i] / mass;
            self.velocity[i] += acc * dt;
            self.displacement[i] += self.velocity[i] * dt;
        }
    }

    /// Elastic potential energy: sum_i 0.5 * k_i * (d_i - r_i)^2
    #[allow(dead_code)]
    pub fn elastic_energy(&self) -> f64 {
        let mut e = 0.0;
        for i in 0..3 {
            let d = self.displacement[i] - self.rest_length[i];
            e += 0.5 * self.stiffness[i] * d * d;
        }
        e
    }

    /// Kinetic energy.
    #[allow(dead_code)]
    pub fn kinetic_energy(&self, mass: f64) -> f64 {
        let v2: f64 = self.velocity.iter().map(|&v| v * v).sum();
        0.5 * mass * v2
    }

    /// Set displacement (external constraint).
    #[allow(dead_code)]
    pub fn set_displacement(&mut self, d: [f64; 3]) {
        self.displacement = d;
    }

    /// Reset to rest.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.displacement = [0.0; 3];
        self.velocity = [0.0; 3];
    }

    /// Force magnitude.
    #[allow(dead_code)]
    pub fn force_magnitude(&self) -> f64 {
        let f = self.force();
        (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt()
    }
}

/// Coupled anisotropic spring pair.
#[allow(dead_code)]
pub struct SpringPair {
    pub spring: AnisotropicSpring,
    pub pos_a: [f64; 3],
    pub pos_b: [f64; 3],
}

impl SpringPair {
    #[allow(dead_code)]
    pub fn new(stiffness: [f64; 3], damping: [f64; 3], a: [f64; 3], b: [f64; 3]) -> Self {
        let rest: [f64; 3] = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let spring = AnisotropicSpring::new(stiffness, damping).with_rest_length(rest);
        Self { spring, pos_a: a, pos_b: b }
    }

    #[allow(dead_code)]
    pub fn current_extension(&self) -> [f64; 3] {
        [
            self.spring.displacement[0] - self.spring.rest_length[0],
            self.spring.displacement[1] - self.spring.rest_length[1],
            self.spring.displacement[2] - self.spring.rest_length[2],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_at_rest() {
        let spring = AnisotropicSpring::new([100.0, 50.0, 200.0], [0.0; 3]);
        let f = spring.force();
        assert!(f.iter().all(|&x| x.abs() < 1e-12));
    }

    #[test]
    fn test_force_x_only() {
        let mut spring = AnisotropicSpring::new([100.0, 50.0, 200.0], [0.0; 3]);
        spring.displacement = [1.0, 0.0, 0.0];
        let f = spring.force();
        assert!((f[0] - (-100.0)).abs() < 1e-9);
        assert!(f[1].abs() < 1e-9);
    }

    #[test]
    fn test_elastic_energy_displaced() {
        let mut spring = AnisotropicSpring::new([100.0, 50.0, 200.0], [0.0; 3]);
        spring.displacement = [1.0, 0.0, 0.0];
        assert!((spring.elastic_energy() - 50.0).abs() < 1e-9);
    }

    #[test]
    fn test_elastic_energy_zero_at_rest() {
        let spring = AnisotropicSpring::new([100.0, 50.0, 200.0], [0.0; 3]);
        assert!((spring.elastic_energy()).abs() < 1e-12);
    }

    #[test]
    fn test_step_oscillates() {
        let mut spring = AnisotropicSpring::new([100.0, 100.0, 100.0], [1.0; 3]);
        spring.displacement = [1.0, 0.0, 0.0];
        spring.step(1.0, 0.01);
        assert_ne!(spring.displacement[0], 1.0);
    }

    #[test]
    fn test_reset() {
        let mut spring = AnisotropicSpring::new([100.0, 50.0, 200.0], [0.0; 3]);
        spring.displacement = [2.0, 1.0, 3.0];
        spring.velocity = [0.1, 0.2, 0.3];
        spring.reset();
        assert!(spring.displacement.iter().all(|&x| x.abs() < 1e-12));
        assert!(spring.velocity.iter().all(|&x| x.abs() < 1e-12));
    }

    #[test]
    fn test_force_magnitude_positive() {
        let mut spring = AnisotropicSpring::new([100.0, 50.0, 200.0], [0.0; 3]);
        spring.displacement = [1.0, 1.0, 1.0];
        assert!(spring.force_magnitude() > 0.0);
    }

    #[test]
    fn test_spring_pair_rest_length() {
        let pair = SpringPair::new([100.0; 3], [0.0; 3], [0.0; 3], [1.0, 0.0, 0.0]);
        assert!((pair.spring.rest_length[0] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let spring = AnisotropicSpring::new([100.0, 50.0, 200.0], [0.0; 3]);
        assert!((spring.kinetic_energy(1.0)).abs() < 1e-12);
    }

    #[test]
    fn test_anisotropy_different_axis_forces() {
        let mut spring = AnisotropicSpring::new([100.0, 50.0, 200.0], [0.0; 3]);
        spring.displacement = [1.0, 1.0, 1.0];
        let f = spring.force();
        assert!((f[0] + 100.0).abs() < 1e-9);
        assert!((f[1] + 50.0).abs() < 1e-9);
        assert!((f[2] + 200.0).abs() < 1e-9);
    }
}
