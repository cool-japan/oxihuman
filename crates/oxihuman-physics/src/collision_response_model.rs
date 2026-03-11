// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Models collision response between two bodies.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CollisionResponseModel {
    restitution: f32,
    friction: f32,
    bias_factor: f32,
    slop: f32,
}

#[allow(dead_code)]
impl CollisionResponseModel {
    pub fn new(restitution: f32, friction: f32) -> Self {
        Self {
            restitution: restitution.clamp(0.0, 1.0),
            friction: friction.clamp(0.0, 1.0),
            bias_factor: 0.2,
            slop: 0.005,
        }
    }

    pub fn default_model() -> Self {
        Self::new(0.3, 0.5)
    }

    pub fn restitution(&self) -> f32 {
        self.restitution
    }

    pub fn friction(&self) -> f32 {
        self.friction
    }

    pub fn compute_impulse(&self, relative_vel: f32, inv_mass_sum: f32, penetration: f32) -> f32 {
        if inv_mass_sum <= 0.0 {
            return 0.0;
        }
        let bias = (self.bias_factor / 0.016) * (penetration - self.slop).max(0.0);
        let j = (-(1.0 + self.restitution) * relative_vel + bias) / inv_mass_sum;
        j.max(0.0)
    }

    pub fn compute_friction_impulse(
        &self,
        normal_impulse: f32,
        tangent_vel: f32,
        inv_mass_sum: f32,
    ) -> f32 {
        if inv_mass_sum <= 0.0 {
            return 0.0;
        }
        let max_friction = self.friction * normal_impulse;
        let jt = -tangent_vel / inv_mass_sum;
        jt.clamp(-max_friction, max_friction)
    }

    pub fn combine(
        a: &CollisionResponseModel,
        b: &CollisionResponseModel,
    ) -> CollisionResponseModel {
        CollisionResponseModel {
            restitution: (a.restitution * b.restitution).sqrt(),
            friction: (a.friction + b.friction) * 0.5,
            bias_factor: (a.bias_factor + b.bias_factor) * 0.5,
            slop: a.slop.max(b.slop),
        }
    }

    pub fn is_elastic(&self) -> bool {
        self.restitution > 0.9
    }

    pub fn is_inelastic(&self) -> bool {
        self.restitution < 0.1
    }

    pub fn set_bias(&mut self, factor: f32, slop: f32) {
        self.bias_factor = factor;
        self.slop = slop;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = CollisionResponseModel::new(0.5, 0.3);
        assert!((m.restitution() - 0.5).abs() < 1e-6);
        assert!((m.friction() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        let m = CollisionResponseModel::new(2.0, -1.0);
        assert!((m.restitution() - 1.0).abs() < 1e-6);
        assert!((m.friction()).abs() < 1e-6);
    }

    #[test]
    fn test_default_model() {
        let m = CollisionResponseModel::default_model();
        assert!((m.restitution() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_compute_impulse_separating() {
        let m = CollisionResponseModel::new(0.5, 0.5);
        let j = m.compute_impulse(1.0, 2.0, 0.0);
        assert!((j).abs() < 1e-6);
    }

    #[test]
    fn test_compute_impulse_colliding() {
        let m = CollisionResponseModel::new(0.0, 0.5);
        let j = m.compute_impulse(-2.0, 1.0, 0.0);
        assert!(j > 0.0);
    }

    #[test]
    fn test_compute_friction_impulse() {
        let m = CollisionResponseModel::new(0.5, 0.5);
        let jf = m.compute_friction_impulse(10.0, 1.0, 1.0);
        assert!(jf.abs() <= 5.0 + 1e-6);
    }

    #[test]
    fn test_combine() {
        let a = CollisionResponseModel::new(1.0, 0.4);
        let b = CollisionResponseModel::new(0.5, 0.6);
        let c = CollisionResponseModel::combine(&a, &b);
        assert!((c.friction() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_is_elastic() {
        let m = CollisionResponseModel::new(0.95, 0.5);
        assert!(m.is_elastic());
    }

    #[test]
    fn test_is_inelastic() {
        let m = CollisionResponseModel::new(0.05, 0.5);
        assert!(m.is_inelastic());
    }

    #[test]
    fn test_zero_inv_mass() {
        let m = CollisionResponseModel::new(0.5, 0.5);
        assert!((m.compute_impulse(-1.0, 0.0, 0.1)).abs() < 1e-6);
    }
}
