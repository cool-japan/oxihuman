// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Clamps forces to prevent instability in physics simulation.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ForceClamp {
    max_linear: f32,
    max_angular: f32,
    max_impulse: f32,
}

#[allow(dead_code)]
impl ForceClamp {
    pub fn new(max_linear: f32, max_angular: f32, max_impulse: f32) -> Self {
        Self {
            max_linear: max_linear.max(0.0),
            max_angular: max_angular.max(0.0),
            max_impulse: max_impulse.max(0.0),
        }
    }

    pub fn default_clamp() -> Self {
        Self::new(10000.0, 5000.0, 1000.0)
    }

    pub fn max_linear(&self) -> f32 {
        self.max_linear
    }

    pub fn max_angular(&self) -> f32 {
        self.max_angular
    }

    pub fn max_impulse(&self) -> f32 {
        self.max_impulse
    }

    pub fn clamp_force(&self, force: [f32; 3]) -> [f32; 3] {
        let mag = (force[0] * force[0] + force[1] * force[1] + force[2] * force[2]).sqrt();
        if mag <= self.max_linear || mag < 1e-9 {
            return force;
        }
        let scale = self.max_linear / mag;
        [force[0] * scale, force[1] * scale, force[2] * scale]
    }

    pub fn clamp_torque(&self, torque: [f32; 3]) -> [f32; 3] {
        let mag = (torque[0] * torque[0] + torque[1] * torque[1] + torque[2] * torque[2]).sqrt();
        if mag <= self.max_angular || mag < 1e-9 {
            return torque;
        }
        let scale = self.max_angular / mag;
        [torque[0] * scale, torque[1] * scale, torque[2] * scale]
    }

    pub fn clamp_impulse(&self, impulse: f32) -> f32 {
        impulse.clamp(-self.max_impulse, self.max_impulse)
    }

    pub fn clamp_scalar(&self, value: f32, max: f32) -> f32 {
        value.clamp(-max, max)
    }

    pub fn is_clamped_force(&self, force: [f32; 3]) -> bool {
        let mag_sq = force[0] * force[0] + force[1] * force[1] + force[2] * force[2];
        mag_sq > self.max_linear * self.max_linear
    }

    pub fn force_magnitude(force: [f32; 3]) -> f32 {
        (force[0] * force[0] + force[1] * force[1] + force[2] * force[2]).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fc = ForceClamp::new(100.0, 50.0, 10.0);
        assert!((fc.max_linear() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_default() {
        let fc = ForceClamp::default_clamp();
        assert!(fc.max_linear() > 0.0);
    }

    #[test]
    fn test_clamp_force_within() {
        let fc = ForceClamp::new(100.0, 50.0, 10.0);
        let f = fc.clamp_force([1.0, 0.0, 0.0]);
        assert!((f[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_force_exceeds() {
        let fc = ForceClamp::new(10.0, 50.0, 10.0);
        let f = fc.clamp_force([100.0, 0.0, 0.0]);
        let mag = ForceClamp::force_magnitude(f);
        assert!((mag - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_clamp_torque() {
        let fc = ForceClamp::new(100.0, 5.0, 10.0);
        let t = fc.clamp_torque([0.0, 100.0, 0.0]);
        let mag = ForceClamp::force_magnitude(t);
        assert!((mag - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_clamp_impulse() {
        let fc = ForceClamp::new(100.0, 50.0, 10.0);
        assert!((fc.clamp_impulse(100.0) - 10.0).abs() < 1e-6);
        assert!((fc.clamp_impulse(-100.0) - (-10.0)).abs() < 1e-6);
    }

    #[test]
    fn test_is_clamped() {
        let fc = ForceClamp::new(10.0, 50.0, 10.0);
        assert!(fc.is_clamped_force([100.0, 0.0, 0.0]));
        assert!(!fc.is_clamped_force([1.0, 0.0, 0.0]));
    }

    #[test]
    fn test_force_magnitude() {
        let mag = ForceClamp::force_magnitude([3.0, 4.0, 0.0]);
        assert!((mag - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_zero_force() {
        let fc = ForceClamp::new(10.0, 5.0, 1.0);
        let f = fc.clamp_force([0.0, 0.0, 0.0]);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn test_negative_max() {
        let fc = ForceClamp::new(-5.0, -5.0, -5.0);
        assert!((fc.max_linear()).abs() < 1e-6);
    }
}
