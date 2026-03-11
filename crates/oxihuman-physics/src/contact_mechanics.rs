// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hertz contact theory stub.

use std::f32::consts::PI;

/// Hertz contact configuration.
#[derive(Debug, Clone)]
pub struct HertzContact {
    pub radius_a: f32,
    pub radius_b: f32,
    pub modulus_a: f32,
    pub modulus_b: f32,
    pub poisson_a: f32,
    pub poisson_b: f32,
}

impl HertzContact {
    pub fn new(
        radius_a: f32,
        radius_b: f32,
        modulus_a: f32,
        modulus_b: f32,
        poisson_a: f32,
        poisson_b: f32,
    ) -> Self {
        HertzContact {
            radius_a,
            radius_b,
            modulus_a,
            modulus_b,
            poisson_a,
            poisson_b,
        }
    }

    /// Effective modulus E*.
    pub fn effective_modulus(&self) -> f32 {
        let e1 = (1.0 - self.poisson_a * self.poisson_a) / self.modulus_a;
        let e2 = (1.0 - self.poisson_b * self.poisson_b) / self.modulus_b;
        1.0 / (e1 + e2)
    }

    /// Effective radius R*.
    pub fn effective_radius(&self) -> f32 {
        1.0 / (1.0 / self.radius_a + 1.0 / self.radius_b)
    }

    /// Contact radius a for a given normal force F.
    pub fn contact_radius(&self, force: f32) -> f32 {
        let e_star = self.effective_modulus();
        let r_star = self.effective_radius();
        (3.0 * force * r_star / (4.0 * e_star)).powf(1.0 / 3.0)
    }

    /// Maximum Hertz pressure p0 for a given normal force F.
    pub fn max_pressure(&self, force: f32) -> f32 {
        let a = self.contact_radius(force);
        let r_star = self.effective_radius();
        if (a * r_star).abs() < 1e-12 {
            return 0.0;
        }
        3.0 * force / (2.0 * PI * a * a)
    }

    /// Approach (indentation) delta for a given force.
    pub fn approach(&self, force: f32) -> f32 {
        let a = self.contact_radius(force);
        let r_star = self.effective_radius();
        a * a / r_star
    }
}

impl Default for HertzContact {
    fn default() -> Self {
        Self::new(0.01, 0.01, 200e9, 200e9, 0.3, 0.3)
    }
}

/// Compute normal force from approach (inverse Hertz, stub).
pub fn force_from_approach(config: &HertzContact, delta: f32) -> f32 {
    let e_star = config.effective_modulus();
    let r_star = config.effective_radius();
    (4.0 / 3.0) * e_star * r_star.sqrt() * delta.max(0.0).powf(1.5)
}

/// Contact area for a given force.
pub fn contact_area(config: &HertzContact, force: f32) -> f32 {
    let a = config.contact_radius(force);
    PI * a * a
}

/// Estimate stiffness at a given approach delta.
pub fn contact_stiffness(config: &HertzContact, delta: f32) -> f32 {
    let e_star = config.effective_modulus();
    let r_star = config.effective_radius();
    2.0 * e_star * (r_star * delta.max(0.0)).sqrt()
}

/// Create a sphere-on-flat Hertz contact (flat has infinite radius).
pub fn sphere_on_flat(
    radius: f32,
    modulus_sphere: f32,
    modulus_flat: f32,
    poisson: f32,
) -> HertzContact {
    HertzContact::new(radius, 1e12, modulus_sphere, modulus_flat, poisson, poisson)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_radius_equal_spheres() {
        let c = HertzContact::new(0.01, 0.01, 200e9, 200e9, 0.3, 0.3);
        let r = c.effective_radius();
        assert!((r - 0.005).abs() < 1e-6);
    }

    #[test]
    fn test_contact_radius_positive_force() {
        let c = HertzContact::default();
        let a = c.contact_radius(1.0);
        assert!(a > 0.0);
    }

    #[test]
    fn test_max_pressure_positive() {
        let c = HertzContact::default();
        let p = c.max_pressure(1.0);
        assert!(p > 0.0);
    }

    #[test]
    fn test_approach_positive() {
        let c = HertzContact::default();
        let d = c.approach(1.0);
        assert!(d > 0.0);
    }

    #[test]
    fn test_force_from_approach_zero() {
        let c = HertzContact::default();
        let f = force_from_approach(&c, 0.0);
        assert!(f.abs() < 1e-10);
    }

    #[test]
    fn test_force_from_approach_positive() {
        let c = HertzContact::default();
        let f = force_from_approach(&c, 1e-6);
        assert!(f > 0.0);
    }

    #[test]
    fn test_contact_area_positive() {
        let c = HertzContact::default();
        let area = contact_area(&c, 1.0);
        assert!(area > 0.0);
    }

    #[test]
    fn test_contact_stiffness_zero_delta() {
        let c = HertzContact::default();
        let k = contact_stiffness(&c, 0.0);
        assert!(k >= 0.0);
    }

    #[test]
    fn test_sphere_on_flat() {
        let c = sphere_on_flat(0.01, 200e9, 200e9, 0.3);
        /* flat has huge radius so effective radius ~= sphere radius */
        let r_star = c.effective_radius();
        assert!(r_star > 0.009 && r_star < 0.011);
    }

    #[test]
    fn test_effective_modulus_positive() {
        let c = HertzContact::default();
        assert!(c.effective_modulus() > 0.0);
    }
}
