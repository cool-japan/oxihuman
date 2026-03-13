// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ferroelectric hysteresis stub.

/// Ferroelectric material parameters.
#[derive(Debug, Clone)]
pub struct FerroelectricConfig {
    pub saturation_polarization: f32, /* Ps [C/m^2] */
    pub remnant_polarization: f32,    /* Pr [C/m^2] */
    pub coercive_field: f32,          /* Ec [V/m] */
    pub permittivity: f32,
}

impl FerroelectricConfig {
    pub fn new(
        saturation_polarization: f32,
        remnant_polarization: f32,
        coercive_field: f32,
        permittivity: f32,
    ) -> Self {
        FerroelectricConfig {
            saturation_polarization,
            remnant_polarization,
            coercive_field,
            permittivity,
        }
    }

    pub fn bst() -> Self {
        /* BaSrTiO3 approximate values */
        FerroelectricConfig::new(0.3, 0.2, 2e5, 1000.0 * 8.854e-12)
    }
}

impl Default for FerroelectricConfig {
    fn default() -> Self {
        Self::bst()
    }
}

/// Simplified hysteresis polarization (Preisach-like, single-loop).
pub fn hysteresis_polarization(config: &FerroelectricConfig, e_field: f32) -> f32 {
    /* Sigmoid-like: P = Ps * tanh((E - Ec) / Ec) if E > 0, mirrored otherwise */
    let ec = config.coercive_field;
    if ec <= 0.0 {
        return 0.0;
    }
    let x = (e_field - ec) / ec;
    config.saturation_polarization * x.tanh()
}

/// Remnant polarization after field removed (stub: returns configured Pr, sign from last field).
pub fn remnant_polarization(config: &FerroelectricConfig, last_field_sign: f32) -> f32 {
    if last_field_sign >= 0.0 {
        config.remnant_polarization
    } else {
        -config.remnant_polarization
    }
}

/// Check if field exceeds coercive (switching threshold).
pub fn is_above_coercive(config: &FerroelectricConfig, e_field: f32) -> bool {
    e_field.abs() > config.coercive_field
}

/// Energy density in one hysteresis loop (2 * Pr * Ec * area_factor, stub).
pub fn hysteresis_energy_density(config: &FerroelectricConfig) -> f32 {
    4.0 * config.remnant_polarization * config.coercive_field
}

/// Dielectric displacement D = eps*E + P.
pub fn displacement_field(config: &FerroelectricConfig, e_field: f32) -> f32 {
    let p = hysteresis_polarization(config, e_field);
    config.permittivity * e_field + p
}

/// Small-signal permittivity near saturation (stub).
pub fn small_signal_permittivity(config: &FerroelectricConfig, e_field: f32) -> f32 {
    /* dp/de at operating point */
    let ec = config.coercive_field;
    if ec <= 0.0 {
        return config.permittivity;
    }
    let x = (e_field - ec) / ec;
    let tanh_x = x.tanh();
    let sech2 = 1.0 - tanh_x * tanh_x;
    config.permittivity + config.saturation_polarization * sech2 / ec
}

/// Curie temperature dependence of Ps (simplified linear stub).
pub fn polarization_vs_temp(config: &FerroelectricConfig, temp: f32, curie_temp: f32) -> f32 {
    if temp >= curie_temp {
        return 0.0;
    }
    config.saturation_polarization * ((curie_temp - temp) / curie_temp).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hysteresis_saturation() {
        let c = FerroelectricConfig::default();
        let p = hysteresis_polarization(&c, 1e8); /* very large field */
        assert!(p < c.saturation_polarization * 1.01);
    }

    #[test]
    fn test_hysteresis_near_coercive() {
        let c = FerroelectricConfig::default();
        let p_pos = hysteresis_polarization(&c, c.coercive_field);
        assert!(p_pos.abs() < c.saturation_polarization);
    }

    #[test]
    fn test_remnant_positive() {
        let c = FerroelectricConfig::default();
        assert_eq!(remnant_polarization(&c, 1.0), c.remnant_polarization);
    }

    #[test]
    fn test_remnant_negative() {
        let c = FerroelectricConfig::default();
        assert_eq!(remnant_polarization(&c, -1.0), -c.remnant_polarization);
    }

    #[test]
    fn test_is_above_coercive_true() {
        let c = FerroelectricConfig::default();
        assert!(is_above_coercive(&c, c.coercive_field * 2.0));
    }

    #[test]
    fn test_is_above_coercive_false() {
        let c = FerroelectricConfig::default();
        assert!(!is_above_coercive(&c, c.coercive_field * 0.5));
    }

    #[test]
    fn test_hysteresis_energy_positive() {
        let c = FerroelectricConfig::default();
        assert!(hysteresis_energy_density(&c) > 0.0);
    }

    #[test]
    fn test_displacement_field_at_zero() {
        let c = FerroelectricConfig::default();
        /* At E=0: D = P(0) */
        let d = displacement_field(&c, 0.0);
        let p0 = hysteresis_polarization(&c, 0.0);
        assert!((d - p0).abs() < 1e-20);
    }

    #[test]
    fn test_polarization_vs_temp_zero_at_curie() {
        let c = FerroelectricConfig::default();
        assert_eq!(polarization_vs_temp(&c, 400.0, 400.0), 0.0);
    }

    #[test]
    fn test_polarization_vs_temp_positive_below_curie() {
        let c = FerroelectricConfig::default();
        assert!(polarization_vs_temp(&c, 300.0, 400.0) > 0.0);
    }
}
