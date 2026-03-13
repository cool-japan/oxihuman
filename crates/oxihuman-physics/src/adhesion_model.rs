// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! JKR/DMT adhesion model stub.

use std::f32::consts::PI;

/// Adhesion model type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdhesionModel {
    /// Johnson-Kendall-Roberts model.
    Jkr,
    /// Derjaguin-Muller-Toporov model.
    Dmt,
}

/// Adhesion parameters.
#[derive(Debug, Clone)]
pub struct AdhesionConfig {
    pub model: AdhesionModel,
    pub effective_modulus: f32,
    pub effective_radius: f32,
    pub work_of_adhesion: f32,
}

impl AdhesionConfig {
    pub fn new(
        model: AdhesionModel,
        effective_modulus: f32,
        effective_radius: f32,
        work_of_adhesion: f32,
    ) -> Self {
        AdhesionConfig {
            model,
            effective_modulus,
            effective_radius,
            work_of_adhesion,
        }
    }
}

impl Default for AdhesionConfig {
    fn default() -> Self {
        Self::new(AdhesionModel::Jkr, 1e9, 1e-6, 0.05)
    }
}

/// JKR pull-off force.
pub fn jkr_pulloff_force(config: &AdhesionConfig) -> f32 {
    1.5 * PI * config.work_of_adhesion * config.effective_radius
}

/// DMT pull-off force.
pub fn dmt_pulloff_force(config: &AdhesionConfig) -> f32 {
    2.0 * PI * config.work_of_adhesion * config.effective_radius
}

/// Pull-off force according to the selected model.
pub fn pulloff_force(config: &AdhesionConfig) -> f32 {
    match config.model {
        AdhesionModel::Jkr => jkr_pulloff_force(config),
        AdhesionModel::Dmt => dmt_pulloff_force(config),
    }
}

/// JKR contact radius at zero applied force.
pub fn jkr_zero_force_radius(config: &AdhesionConfig) -> f32 {
    let w = config.work_of_adhesion;
    let r = config.effective_radius;
    let e_star = config.effective_modulus;
    (6.0 * PI * w * r * r / e_star).cbrt()
}

/// Maugis parameter (Tabor number) — classifies JKR vs DMT regime.
pub fn maugis_parameter(config: &AdhesionConfig, adhesion_range: f32) -> f32 {
    let w = config.work_of_adhesion;
    let r = config.effective_radius;
    let e_star = config.effective_modulus;
    let cube = w * w * r / (e_star * e_star * adhesion_range * adhesion_range * adhesion_range);
    1.16 * cube.cbrt()
}

/// Is this contact in the JKR regime (Tabor number >> 1)?
pub fn is_jkr_regime(config: &AdhesionConfig, adhesion_range: f32) -> bool {
    maugis_parameter(config, adhesion_range) > 1.0
}

/// Adhesion energy stored in contact at radius `a`.
pub fn adhesion_energy(config: &AdhesionConfig, contact_radius: f32) -> f32 {
    PI * contact_radius * contact_radius * config.work_of_adhesion
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jkr_pulloff_positive() {
        let c = AdhesionConfig::default();
        assert!(jkr_pulloff_force(&c) > 0.0);
    }

    #[test]
    fn test_dmt_pulloff_positive() {
        let c = AdhesionConfig::default();
        assert!(dmt_pulloff_force(&c) > 0.0);
    }

    #[test]
    fn test_dmt_greater_than_jkr() {
        let c = AdhesionConfig::default();
        assert!(dmt_pulloff_force(&c) > jkr_pulloff_force(&c));
    }

    #[test]
    fn test_pulloff_jkr_model() {
        let c = AdhesionConfig::new(AdhesionModel::Jkr, 1e9, 1e-6, 0.05);
        let f = pulloff_force(&c);
        assert!((f - jkr_pulloff_force(&c)).abs() < 1e-30);
    }

    #[test]
    fn test_pulloff_dmt_model() {
        let c = AdhesionConfig::new(AdhesionModel::Dmt, 1e9, 1e-6, 0.05);
        let f = pulloff_force(&c);
        assert!((f - dmt_pulloff_force(&c)).abs() < 1e-30);
    }

    #[test]
    fn test_jkr_zero_force_radius_positive() {
        let c = AdhesionConfig::default();
        assert!(jkr_zero_force_radius(&c) > 0.0);
    }

    #[test]
    fn test_maugis_parameter_positive() {
        let c = AdhesionConfig::default();
        let mu = maugis_parameter(&c, 1e-10);
        assert!(mu > 0.0);
    }

    #[test]
    fn test_adhesion_energy_positive() {
        let c = AdhesionConfig::default();
        let e = adhesion_energy(&c, 1e-7);
        assert!(e > 0.0);
    }

    #[test]
    fn test_adhesion_energy_zero_radius() {
        let c = AdhesionConfig::default();
        let e = adhesion_energy(&c, 0.0);
        assert_eq!(e, 0.0);
    }

    #[test]
    fn test_is_jkr_regime_large_maugis() {
        let c = AdhesionConfig::new(AdhesionModel::Jkr, 1e6, 1e-3, 1.0);
        /* large radius, large adhesion => high Maugis number => JKR regime */
        let result = is_jkr_regime(&c, 1e-10);
        assert!(result);
    }
}
