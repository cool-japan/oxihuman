// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Elastohydrodynamic lubrication (EHL) model stub.

/// Lubrication regime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LubricationRegime {
    BoundaryLayer,
    Mixed,
    Hydrodynamic,
    Elastohydrodynamic,
}

/// EHL lubricant parameters.
#[derive(Debug, Clone)]
pub struct LubricantConfig {
    pub dynamic_viscosity: f32,
    pub pressure_viscosity_coeff: f32,
    pub density: f32,
    pub thermal_conductivity: f32,
}

impl LubricantConfig {
    pub fn new(
        dynamic_viscosity: f32,
        pressure_viscosity_coeff: f32,
        density: f32,
        thermal_conductivity: f32,
    ) -> Self {
        LubricantConfig {
            dynamic_viscosity,
            pressure_viscosity_coeff,
            density,
            thermal_conductivity,
        }
    }

    pub fn mineral_oil() -> Self {
        LubricantConfig::new(0.1, 2e-8, 880.0, 0.13)
    }
}

impl Default for LubricantConfig {
    fn default() -> Self {
        Self::mineral_oil()
    }
}

/// Pressure-dependent viscosity (Barus equation).
pub fn barus_viscosity(config: &LubricantConfig, pressure: f32) -> f32 {
    config.dynamic_viscosity * (config.pressure_viscosity_coeff * pressure.max(0.0)).exp()
}

/// Minimum film thickness (Grubin approximation, stub).
pub fn grubin_film_thickness(
    config: &LubricantConfig,
    speed: f32,
    load: f32,
    effective_radius: f32,
    effective_modulus: f32,
) -> f32 {
    if load <= 0.0 || effective_modulus <= 0.0 || effective_radius <= 0.0 {
        return 0.0;
    }
    let u = config.dynamic_viscosity * speed.max(0.0) / (effective_modulus * effective_radius);
    let g = config.pressure_viscosity_coeff * effective_modulus;
    let w = load / (effective_modulus * effective_radius * effective_radius);
    /* Grubin: h0/R* = 1.95 * U^0.73 * G^0.73 * W^(-0.09) */
    1.95 * effective_radius * u.powf(0.73) * g.powf(0.73) * w.powf(-0.09_f32)
}

/// Classify the lubrication regime by Sommerfeld number.
pub fn classify_regime(
    config: &LubricantConfig,
    speed: f32,
    load: f32,
    radius: f32,
) -> LubricationRegime {
    if load <= 0.0 || radius <= 0.0 {
        return LubricationRegime::BoundaryLayer;
    }
    let sommerfeld = config.dynamic_viscosity * speed.max(0.0) * radius / load;
    if sommerfeld < 1e-8 {
        LubricationRegime::BoundaryLayer
    } else if sommerfeld < 1e-5 {
        LubricationRegime::Mixed
    } else if sommerfeld < 1e-2 {
        LubricationRegime::Hydrodynamic
    } else {
        LubricationRegime::Elastohydrodynamic
    }
}

/// Viscous shear stress on the film.
pub fn film_shear_stress(config: &LubricantConfig, velocity_gradient: f32) -> f32 {
    config.dynamic_viscosity * velocity_gradient.abs()
}

/// Heat generated per unit area by viscous dissipation.
pub fn viscous_heat(config: &LubricantConfig, velocity_gradient: f32) -> f32 {
    let tau = film_shear_stress(config, velocity_gradient);
    tau * velocity_gradient.abs()
}

/// Entrainment velocity (mean surface velocity).
pub fn entrainment_velocity(v1: f32, v2: f32) -> f32 {
    (v1 + v2) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barus_viscosity_zero_pressure() {
        let c = LubricantConfig::default();
        let v = barus_viscosity(&c, 0.0);
        assert!((v - c.dynamic_viscosity).abs() < 1e-10);
    }

    #[test]
    fn test_barus_viscosity_increases_with_pressure() {
        let c = LubricantConfig::default();
        let v0 = barus_viscosity(&c, 0.0);
        let v1 = barus_viscosity(&c, 1e8);
        assert!(v1 > v0);
    }

    #[test]
    fn test_grubin_film_thickness_positive() {
        let c = LubricantConfig::default();
        let h = grubin_film_thickness(&c, 1.0, 1000.0, 0.01, 200e9);
        assert!(h > 0.0);
    }

    #[test]
    fn test_grubin_zero_load() {
        let c = LubricantConfig::default();
        let h = grubin_film_thickness(&c, 1.0, 0.0, 0.01, 200e9);
        assert_eq!(h, 0.0);
    }

    #[test]
    fn test_classify_regime_boundary() {
        let c = LubricantConfig::default();
        let r = classify_regime(&c, 0.0, 1000.0, 0.01);
        assert_eq!(r, LubricationRegime::BoundaryLayer);
    }

    #[test]
    fn test_classify_regime_ehl() {
        let c = LubricantConfig::default();
        let r = classify_regime(&c, 100.0, 0.001, 0.01);
        assert_eq!(r, LubricationRegime::Elastohydrodynamic);
    }

    #[test]
    fn test_film_shear_stress_positive() {
        let c = LubricantConfig::default();
        assert!(film_shear_stress(&c, 1000.0) > 0.0);
    }

    #[test]
    fn test_viscous_heat_positive() {
        let c = LubricantConfig::default();
        assert!(viscous_heat(&c, 1000.0) > 0.0);
    }

    #[test]
    fn test_entrainment_velocity() {
        assert!((entrainment_velocity(2.0, 4.0) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_mineral_oil_viscosity() {
        let c = LubricantConfig::mineral_oil();
        assert!((c.dynamic_viscosity - 0.1).abs() < 1e-6);
    }
}
