// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Models contact friction using Coulomb friction law.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ContactFrictionModel {
    static_coeff: f32,
    dynamic_coeff: f32,
    velocity_threshold: f32,
}

#[allow(dead_code)]
impl ContactFrictionModel {
    pub fn new(static_coeff: f32, dynamic_coeff: f32) -> Self {
        Self {
            static_coeff: static_coeff.max(0.0),
            dynamic_coeff: dynamic_coeff.max(0.0).min(static_coeff),
            velocity_threshold: 0.01,
        }
    }

    pub fn default_model() -> Self {
        Self::new(0.6, 0.4)
    }

    pub fn ice() -> Self {
        Self::new(0.05, 0.03)
    }

    pub fn rubber() -> Self {
        Self::new(1.0, 0.8)
    }

    pub fn static_coeff(&self) -> f32 {
        self.static_coeff
    }

    pub fn dynamic_coeff(&self) -> f32 {
        self.dynamic_coeff
    }

    pub fn with_velocity_threshold(mut self, threshold: f32) -> Self {
        self.velocity_threshold = threshold;
        self
    }

    pub fn effective_coeff(&self, tangent_speed: f32) -> f32 {
        if tangent_speed < self.velocity_threshold {
            self.static_coeff
        } else {
            self.dynamic_coeff
        }
    }

    pub fn max_friction_force(&self, normal_force: f32, tangent_speed: f32) -> f32 {
        let coeff = self.effective_coeff(tangent_speed);
        coeff * normal_force.max(0.0)
    }

    pub fn friction_impulse(
        &self,
        normal_impulse: f32,
        tangent_vel: f32,
        inv_mass_sum: f32,
    ) -> f32 {
        if inv_mass_sum <= 0.0 {
            return 0.0;
        }
        let coeff = self.effective_coeff(tangent_vel.abs());
        let max_j = coeff * normal_impulse.abs();
        let jt = -tangent_vel / inv_mass_sum;
        jt.clamp(-max_j, max_j)
    }

    pub fn combine(a: &ContactFrictionModel, b: &ContactFrictionModel) -> ContactFrictionModel {
        ContactFrictionModel {
            static_coeff: (a.static_coeff * b.static_coeff).sqrt(),
            dynamic_coeff: (a.dynamic_coeff * b.dynamic_coeff).sqrt(),
            velocity_threshold: a.velocity_threshold.max(b.velocity_threshold),
        }
    }

    pub fn is_frictionless(&self) -> bool {
        self.static_coeff < 1e-6 && self.dynamic_coeff < 1e-6
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = ContactFrictionModel::new(0.5, 0.3);
        assert!((m.static_coeff() - 0.5).abs() < 1e-6);
        assert!((m.dynamic_coeff() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_default() {
        let m = ContactFrictionModel::default_model();
        assert!((m.static_coeff() - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_ice() {
        let m = ContactFrictionModel::ice();
        assert!(m.static_coeff() < 0.1);
    }

    #[test]
    fn test_rubber() {
        let m = ContactFrictionModel::rubber();
        assert!(m.static_coeff() >= 0.9);
    }

    #[test]
    fn test_effective_coeff_static() {
        let m = ContactFrictionModel::new(0.6, 0.4).with_velocity_threshold(0.1);
        assert!((m.effective_coeff(0.01) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_effective_coeff_dynamic() {
        let m = ContactFrictionModel::new(0.6, 0.4).with_velocity_threshold(0.1);
        assert!((m.effective_coeff(1.0) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_max_friction_force() {
        let m = ContactFrictionModel::new(0.5, 0.3);
        let f = m.max_friction_force(100.0, 0.0);
        assert!((f - 50.0).abs() < 1e-4);
    }

    #[test]
    fn test_friction_impulse_clamped() {
        let m = ContactFrictionModel::new(0.5, 0.3);
        let j = m.friction_impulse(10.0, 100.0, 1.0);
        assert!(j.abs() <= 5.0 + 1e-6);
    }

    #[test]
    fn test_combine() {
        let a = ContactFrictionModel::new(1.0, 0.5);
        let b = ContactFrictionModel::new(0.5, 0.2);
        let c = ContactFrictionModel::combine(&a, &b);
        assert!(c.static_coeff() > 0.0);
    }

    #[test]
    fn test_is_frictionless() {
        let m = ContactFrictionModel::new(0.0, 0.0);
        assert!(m.is_frictionless());
    }
}
