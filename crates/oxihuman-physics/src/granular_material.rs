// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Granular flow (angle of repose) stub.
//!
//! Models granular materials (sand, grains) using the Mohr-Coulomb failure
//! criterion and angle-of-repose dynamics.

/// Granular material properties.
#[derive(Debug, Clone)]
pub struct GranularParams {
    /// Angle of internal friction (angle of repose) `[degrees]`.
    pub friction_angle_deg: f64,
    /// Cohesion `[Pa]` (zero for dry sand).
    pub cohesion: f64,
    /// Bulk density [kg/m³].
    pub bulk_density: f64,
    /// Grain diameter `[m]`.
    pub grain_diameter: f64,
    /// Coefficient of restitution for grain collisions.
    pub restitution: f64,
}

impl Default for GranularParams {
    fn default() -> Self {
        Self {
            friction_angle_deg: 34.0,
            cohesion: 0.0,
            bulk_density: 1700.0,
            grain_diameter: 2e-4,
            restitution: 0.7,
        }
    }
}

/// A granular pile state.
#[derive(Debug, Clone)]
pub struct GranularPile {
    pub height: f64,
    pub base_radius: f64,
    pub mass: f64,
    pub settled: bool,
}

impl GranularPile {
    pub fn new(mass: f64, params: &GranularParams) -> Self {
        let phi = params.friction_angle_deg.to_radians();
        let base_radius = mass / (params.bulk_density * std::f64::consts::PI * phi.tan());
        let base_radius = base_radius.cbrt();
        let height = base_radius * phi.tan();
        Self {
            height,
            base_radius,
            mass,
            settled: false,
        }
    }

    pub fn slope_angle_deg(&self) -> f64 {
        if self.base_radius <= 0.0 {
            return 0.0;
        }
        (self.height / self.base_radius).atan().to_degrees()
    }

    pub fn volume(&self) -> f64 {
        std::f64::consts::PI * self.base_radius * self.base_radius * self.height / 3.0
    }
}

/// Mohr-Coulomb failure criterion: τ = c + σ * tan(φ).
pub fn mohr_coulomb_shear_strength(normal_stress: f64, params: &GranularParams) -> f64 {
    let phi = params.friction_angle_deg.to_radians();
    params.cohesion + normal_stress * phi.tan()
}

/// Check if a slope will fail (slip) under the given shear stress.
pub fn will_slope_fail(shear_stress: f64, normal_stress: f64, params: &GranularParams) -> bool {
    shear_stress >= mohr_coulomb_shear_strength(normal_stress, params)
}

/// Compute the critical slope angle (angle of repose) for cohesionless material.
pub fn angle_of_repose_deg(params: &GranularParams) -> f64 {
    params.friction_angle_deg
}

/// Compute the Drucker-Prager equivalent yield stress.
pub fn drucker_prager_yield(
    i1: f64, /* First stress invariant (trace) */
    j2: f64, /* Second deviatoric invariant */
    params: &GranularParams,
) -> f64 {
    let phi = params.friction_angle_deg.to_radians();
    let alpha = 2.0 * phi.sin() / (3.0_f64.sqrt() * (3.0 - phi.sin()));
    let k = 6.0 * params.cohesion * phi.cos() / (3.0_f64.sqrt() * (3.0 - phi.sin()));
    j2.sqrt() - (k - alpha * i1)
}

/// Estimate the avalanche threshold for a granular pile.
pub fn avalanche_threshold_height(base_radius: f64, params: &GranularParams) -> f64 {
    let phi = params.friction_angle_deg.to_radians();
    base_radius * phi.tan()
}

/// Simulate settling: reduce pile height toward equilibrium.
pub fn settle_pile(pile: &mut GranularPile, params: &GranularParams, dt: f64) {
    let target = avalanche_threshold_height(pile.base_radius, params);
    if pile.height <= target {
        pile.settled = true;
        return;
    }
    let rate = (pile.height - target) * 0.5;
    pile.height -= rate * dt;
    if pile.height <= target {
        pile.height = target;
        pile.settled = true;
    }
}

/// Compute bulk pressure at depth h in a granular column.
pub fn overburden_pressure(depth: f64, params: &GranularParams) -> f64 {
    params.bulk_density * 9.81 * depth
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> GranularParams {
        GranularParams::default()
    }

    #[test]
    fn test_mohr_coulomb_increases_with_normal() {
        let p = default_params();
        let s1 = mohr_coulomb_shear_strength(100.0, &p);
        let s2 = mohr_coulomb_shear_strength(200.0, &p);
        assert!(s2 > s1);
    }

    #[test]
    fn test_will_slope_fail_false_below_strength() {
        let p = default_params();
        assert!(!will_slope_fail(1.0, 1000.0, &p));
    }

    #[test]
    fn test_angle_of_repose() {
        let p = default_params();
        assert!((angle_of_repose_deg(&p) - 34.0).abs() < 1e-9);
    }

    #[test]
    fn test_pile_slope_angle_near_friction_angle() {
        let p = default_params();
        let pile = GranularPile::new(10.0, &p);
        let angle = pile.slope_angle_deg();
        assert!(angle > 0.0 && angle < 90.0);
    }

    #[test]
    fn test_pile_volume_positive() {
        let p = default_params();
        let pile = GranularPile::new(10.0, &p);
        assert!(pile.volume() > 0.0);
    }

    #[test]
    fn test_settle_pile_reduces_height() {
        let p = default_params();
        let mut pile = GranularPile::new(10.0, &p);
        pile.height *= 2.0; /* Make it too tall */
        let old_h = pile.height;
        settle_pile(&mut pile, &p, 1.0);
        assert!(pile.height <= old_h);
    }

    #[test]
    fn test_overburden_pressure_positive() {
        assert!(overburden_pressure(1.0, &default_params()) > 0.0);
    }

    #[test]
    fn test_overburden_proportional_to_depth() {
        let p = default_params();
        let p1 = overburden_pressure(1.0, &p);
        let p2 = overburden_pressure(2.0, &p);
        assert!((p2 / p1 - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_avalanche_threshold_positive() {
        let h = avalanche_threshold_height(1.0, &default_params());
        assert!(h > 0.0);
    }
}
