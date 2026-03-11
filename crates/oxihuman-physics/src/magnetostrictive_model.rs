// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Magnetostrictive (Villari effect) stub.

/// Magnetostrictive material parameters.
#[derive(Debug, Clone)]
pub struct MagnetostrictiveConfig {
    pub saturation_magnetostriction: f32, /* lambda_s [dimensionless] */
    pub saturation_magnetization: f32,    /* Ms [A/m] */
    pub young_modulus: f32,
    pub coupling_coefficient: f32, /* k_m */
}

impl MagnetostrictiveConfig {
    pub fn new(
        saturation_magnetostriction: f32,
        saturation_magnetization: f32,
        young_modulus: f32,
        coupling_coefficient: f32,
    ) -> Self {
        MagnetostrictiveConfig {
            saturation_magnetostriction,
            saturation_magnetization,
            young_modulus,
            coupling_coefficient,
        }
    }

    pub fn terfenol_d() -> Self {
        /* Terfenol-D approximate values */
        MagnetostrictiveConfig::new(1000e-6, 0.8e6, 30e9, 0.7)
    }
}

impl Default for MagnetostrictiveConfig {
    fn default() -> Self {
        Self::terfenol_d()
    }
}

/// Magnetostrictive strain at normalized magnetization m = M/Ms.
pub fn magnetostrictive_strain(config: &MagnetostrictiveConfig, m_norm: f32) -> f32 {
    let m = m_norm.clamp(-1.0, 1.0);
    (3.0 / 2.0) * config.saturation_magnetostriction * (m * m - 1.0 / 3.0)
}

/// Villari effect: change in permeability due to stress.
pub fn villari_delta_perm(config: &MagnetostrictiveConfig, stress: f32) -> f32 {
    config.coupling_coefficient * config.saturation_magnetostriction * stress / config.young_modulus
}

/// Blocking stress (maximum stress at zero strain).
pub fn blocking_stress(config: &MagnetostrictiveConfig) -> f32 {
    config.young_modulus * config.saturation_magnetostriction
}

/// Magnetostrictive force on cross section `area` at normalized magnetization.
pub fn magnetostrictive_force(config: &MagnetostrictiveConfig, m_norm: f32, area: f32) -> f32 {
    let strain = magnetostrictive_strain(config, m_norm);
    config.young_modulus * strain * area.max(0.0)
}

/// Normalized magnetization from applied field (simple Langevin-like saturation stub).
pub fn magnetization_from_field(config: &MagnetostrictiveConfig, h_field: f32) -> f32 {
    let h_sat = config.saturation_magnetization;
    if h_sat <= 0.0 {
        return 0.0;
    }
    let x = h_field / h_sat;
    /* simplified tanh saturation */
    x.tanh()
}

/// Strain directly from applied H field.
pub fn strain_from_field(config: &MagnetostrictiveConfig, h_field: f32) -> f32 {
    let m_norm = magnetization_from_field(config, h_field);
    magnetostrictive_strain(config, m_norm)
}

/// Energy density stored in magnetostrictive material.
pub fn magnetostrictive_energy_density(config: &MagnetostrictiveConfig, m_norm: f32) -> f32 {
    let strain = magnetostrictive_strain(config, m_norm);
    0.5 * config.young_modulus * strain * strain
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturation_strain() {
        let c = MagnetostrictiveConfig::default();
        let s = magnetostrictive_strain(&c, 1.0);
        /* At m=1: (3/2)*lambda_s*(1 - 1/3) = lambda_s; use relative tolerance for f32 */
        let expected = c.saturation_magnetostriction;
        assert!((s - expected).abs() / expected.abs() < 1e-5);
    }

    #[test]
    fn test_strain_zero_at_sqrt_third() {
        let c = MagnetostrictiveConfig::default();
        let m = (1.0f32 / 3.0).sqrt();
        let s = magnetostrictive_strain(&c, m);
        assert!(s.abs() < 1e-10);
    }

    #[test]
    fn test_strain_symmetry() {
        let c = MagnetostrictiveConfig::default();
        let s_pos = magnetostrictive_strain(&c, 0.5);
        let s_neg = magnetostrictive_strain(&c, -0.5);
        assert!((s_pos - s_neg).abs() < 1e-10);
    }

    #[test]
    fn test_blocking_stress_positive() {
        let c = MagnetostrictiveConfig::default();
        assert!(blocking_stress(&c) > 0.0);
    }

    #[test]
    fn test_magnetization_from_field_bounded() {
        let c = MagnetostrictiveConfig::default();
        let m = magnetization_from_field(&c, 1e7);
        assert!((0.0..=1.0).contains(&m));
    }

    #[test]
    fn test_magnetization_zero_field() {
        let c = MagnetostrictiveConfig::default();
        assert_eq!(magnetization_from_field(&c, 0.0), 0.0);
    }

    #[test]
    fn test_strain_from_field_positive_h() {
        let c = MagnetostrictiveConfig::default();
        /* Use a field strong enough to drive m well above sqrt(1/3) ≈ 0.577 */
        let s = strain_from_field(&c, 1e6);
        assert!(s > 0.0);
    }

    #[test]
    fn test_magnetostrictive_force_positive() {
        let c = MagnetostrictiveConfig::default();
        let f = magnetostrictive_force(&c, 1.0, 1e-4);
        /* at m=1, strain = lambda_s > 0, so force > 0 */
        assert!(f >= 0.0);
    }

    #[test]
    fn test_energy_density_positive() {
        let c = MagnetostrictiveConfig::default();
        assert!(magnetostrictive_energy_density(&c, 1.0) > 0.0);
    }

    #[test]
    fn test_villari_delta_perm_sign() {
        let c = MagnetostrictiveConfig::default();
        /* compressive stress (negative) with positive lambda_s => negative permeability change */
        let d = villari_delta_perm(&c, -1e6);
        assert!(d < 0.0);
    }
}
