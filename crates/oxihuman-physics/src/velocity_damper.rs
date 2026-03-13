// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Velocity damper that applies velocity-dependent resistance forces.

#[allow(dead_code)]
fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[allow(dead_code)]
fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VelocityDamper {
    pub linear_coeff: f32,
    pub angular_coeff: f32,
    pub max_linear_speed: f32,
    pub max_angular_speed: f32,
}

#[allow(dead_code)]
impl VelocityDamper {
    pub fn new(linear: f32, angular: f32) -> Self {
        Self {
            linear_coeff: linear.max(0.0),
            angular_coeff: angular.max(0.0),
            max_linear_speed: f32::MAX,
            max_angular_speed: f32::MAX,
        }
    }

    pub fn with_limits(mut self, max_linear: f32, max_angular: f32) -> Self {
        self.max_linear_speed = max_linear;
        self.max_angular_speed = max_angular;
        self
    }

    pub fn damp_linear(&self, velocity: [f32; 3], dt: f32) -> [f32; 3] {
        let factor = (1.0 - self.linear_coeff * dt).max(0.0);
        let mut v = vec3_scale(velocity, factor);
        let speed = vec3_len(v);
        if speed > self.max_linear_speed && speed > 1e-12 {
            v = vec3_scale(v, self.max_linear_speed / speed);
        }
        v
    }

    pub fn damp_angular(&self, angular_vel: [f32; 3], dt: f32) -> [f32; 3] {
        let factor = (1.0 - self.angular_coeff * dt).max(0.0);
        let mut v = vec3_scale(angular_vel, factor);
        let speed = vec3_len(v);
        if speed > self.max_angular_speed && speed > 1e-12 {
            v = vec3_scale(v, self.max_angular_speed / speed);
        }
        v
    }

    pub fn linear_force(&self, velocity: [f32; 3]) -> [f32; 3] {
        vec3_scale(velocity, -self.linear_coeff)
    }

    pub fn energy_dissipated(&self, velocity: [f32; 3], mass: f32, dt: f32) -> f32 {
        let v0 = vec3_len(velocity);
        let v1 = vec3_len(self.damp_linear(velocity, dt));
        0.5 * mass * (v0 * v0 - v1 * v1)
    }

    pub fn time_to_rest(&self, speed: f32, threshold: f32) -> f32 {
        if self.linear_coeff < 1e-12 || speed <= threshold {
            return 0.0;
        }
        (speed / threshold).ln() / self.linear_coeff
    }

    pub fn none() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn heavy() -> Self {
        Self::new(5.0, 5.0)
    }
}

impl Default for VelocityDamper {
    fn default() -> Self { Self::new(0.1, 0.1) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damp_linear() {
        let d = VelocityDamper::new(0.5, 0.0);
        let v = d.damp_linear([10.0, 0.0, 0.0], 1.0);
        assert!((v[0] - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_no_damping() {
        let d = VelocityDamper::none();
        let v = d.damp_linear([10.0, 0.0, 0.0], 1.0);
        assert!((v[0] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_heavy_damping() {
        let d = VelocityDamper::heavy();
        let v = d.damp_linear([10.0, 0.0, 0.0], 1.0);
        assert!((v[0]).abs() < 1e-5);
    }

    #[test]
    fn test_angular_damping() {
        let d = VelocityDamper::new(0.0, 0.5);
        let w = d.damp_angular([4.0, 0.0, 0.0], 1.0);
        assert!((w[0] - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_speed_limit() {
        let d = VelocityDamper::new(0.0, 0.0).with_limits(5.0, 3.0);
        let v = d.damp_linear([100.0, 0.0, 0.0], 0.01);
        assert!((vec3_len(v) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_linear_force() {
        let d = VelocityDamper::new(2.0, 0.0);
        let f = d.linear_force([3.0, 0.0, 0.0]);
        assert!((f[0] + 6.0).abs() < 1e-5);
    }

    #[test]
    fn test_energy_dissipated() {
        let d = VelocityDamper::new(0.5, 0.0);
        let e = d.energy_dissipated([10.0, 0.0, 0.0], 1.0, 1.0);
        assert!(e > 0.0);
    }

    #[test]
    fn test_time_to_rest() {
        let d = VelocityDamper::new(1.0, 0.0);
        let t = d.time_to_rest(10.0, 0.1);
        assert!(t > 0.0);
    }

    #[test]
    fn test_time_to_rest_already_stopped() {
        let d = VelocityDamper::new(1.0, 0.0);
        let t = d.time_to_rest(0.01, 0.1);
        assert!((t).abs() < 1e-5);
    }

    #[test]
    fn test_default() {
        let d = VelocityDamper::default();
        assert!((d.linear_coeff - 0.1).abs() < 1e-5);
    }
}
