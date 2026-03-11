// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Magnetoelectric coupling (multiferroic) stub.

/// Multiferroic material parameters.
#[derive(Debug, Clone)]
pub struct MultiferroicConfig {
    pub me_coefficient: f32,            /* magnetoelectric coupling alpha [s/m] */
    pub electric_permittivity: f32,     /* eps [F/m] */
    pub magnetic_permeability: f32,     /* mu [H/m] */
    pub spontaneous_polarization: f32,  /* Ps [C/m^2] */
    pub spontaneous_magnetization: f32, /* Ms [A/m] */
}

impl MultiferroicConfig {
    pub fn new(
        me_coefficient: f32,
        electric_permittivity: f32,
        magnetic_permeability: f32,
        spontaneous_polarization: f32,
        spontaneous_magnetization: f32,
    ) -> Self {
        MultiferroicConfig {
            me_coefficient,
            electric_permittivity,
            magnetic_permeability,
            spontaneous_polarization,
            spontaneous_magnetization,
        }
    }

    pub fn bfo() -> Self {
        /* BiFeO3 approximate values */
        MultiferroicConfig::new(
            6e-12,
            40.0 * 8.854e-12,
            1.0 * 4.0e-7 * std::f32::consts::PI,
            0.9,
            2.0,
        )
    }
}

impl Default for MultiferroicConfig {
    fn default() -> Self {
        Self::bfo()
    }
}

/// Polarization induced by magnetic field (direct ME effect).
pub fn electric_polarization_from_h(config: &MultiferroicConfig, h_field: f32) -> f32 {
    config.me_coefficient * config.magnetic_permeability * h_field
}

/// Magnetization induced by electric field (converse ME effect).
pub fn magnetization_from_e(config: &MultiferroicConfig, e_field: f32) -> f32 {
    config.me_coefficient * config.electric_permittivity * e_field
}

/// ME voltage coefficient (alpha_v = alpha / eps).
pub fn me_voltage_coefficient(config: &MultiferroicConfig) -> f32 {
    if config.electric_permittivity <= 0.0 {
        return 0.0;
    }
    config.me_coefficient / config.electric_permittivity
}

/// ME current coefficient (alpha_i = alpha / mu).
pub fn me_current_coefficient(config: &MultiferroicConfig) -> f32 {
    if config.magnetic_permeability <= 0.0 {
        return 0.0;
    }
    config.me_coefficient / config.magnetic_permeability
}

/// ME coupling figure of merit (alpha^2 / (eps * mu)).
pub fn me_figure_of_merit(config: &MultiferroicConfig) -> f32 {
    let eps = config.electric_permittivity;
    let mu = config.magnetic_permeability;
    if eps <= 0.0 || mu <= 0.0 {
        return 0.0;
    }
    config.me_coefficient * config.me_coefficient / (eps * mu)
}

/// Phase diagram indicator: is the material magnetically ordered?
pub fn is_magnetically_ordered(config: &MultiferroicConfig, temp: f32, neel_temp: f32) -> bool {
    temp < neel_temp && config.spontaneous_magnetization > 0.0
}

/// Phase diagram indicator: is the material ferroelectrically ordered?
pub fn is_ferroelectrically_ordered(
    config: &MultiferroicConfig,
    temp: f32,
    curie_temp: f32,
) -> bool {
    temp < curie_temp && config.spontaneous_polarization > 0.0
}

/// Combined order parameter (product of normalized M and P).
pub fn combined_order_parameter(
    config: &MultiferroicConfig,
    temp: f32,
    neel_temp: f32,
    curie_temp: f32,
) -> f32 {
    let m = if temp < neel_temp {
        ((neel_temp - temp) / neel_temp).sqrt().min(1.0)
    } else {
        0.0
    };
    let p = if temp < curie_temp {
        ((curie_temp - temp) / curie_temp).sqrt().min(1.0)
    } else {
        0.0
    };
    let _ = config;
    m * p
}

/// Effective ME coupling under applied stress (piezomagnetic modulation, stub).
pub fn stress_modulated_coupling(config: &MultiferroicConfig, stress: f32) -> f32 {
    /* Simplified: stress modulates ME by ~10% per GPa */
    config.me_coefficient * (1.0 + stress * 1e-10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electric_polarization_from_h_positive() {
        let c = MultiferroicConfig::default();
        assert!(electric_polarization_from_h(&c, 1e5) > 0.0);
    }

    #[test]
    fn test_magnetization_from_e_positive() {
        let c = MultiferroicConfig::default();
        assert!(magnetization_from_e(&c, 1e6) > 0.0);
    }

    #[test]
    fn test_me_voltage_coefficient_positive() {
        let c = MultiferroicConfig::default();
        assert!(me_voltage_coefficient(&c) > 0.0);
    }

    #[test]
    fn test_me_current_coefficient_positive() {
        let c = MultiferroicConfig::default();
        assert!(me_current_coefficient(&c) > 0.0);
    }

    #[test]
    fn test_me_figure_of_merit_positive() {
        let c = MultiferroicConfig::default();
        assert!(me_figure_of_merit(&c) > 0.0);
    }

    #[test]
    fn test_is_magnetically_ordered_below_neel() {
        let c = MultiferroicConfig::default();
        assert!(is_magnetically_ordered(&c, 300.0, 643.0));
    }

    #[test]
    fn test_is_magnetically_ordered_above_neel() {
        let c = MultiferroicConfig::default();
        assert!(!is_magnetically_ordered(&c, 700.0, 643.0));
    }

    #[test]
    fn test_is_ferroelectrically_ordered() {
        let c = MultiferroicConfig::default();
        assert!(is_ferroelectrically_ordered(&c, 300.0, 1100.0));
    }

    #[test]
    fn test_combined_order_parameter_range() {
        let c = MultiferroicConfig::default();
        let op = combined_order_parameter(&c, 300.0, 643.0, 1100.0);
        assert!((0.0..=1.0).contains(&op));
    }

    #[test]
    fn test_stress_modulated_coupling() {
        let c = MultiferroicConfig::default();
        let alpha0 = c.me_coefficient;
        let alpha_stressed = stress_modulated_coupling(&c, 0.0);
        assert!((alpha_stressed - alpha0).abs() < 1e-30);
    }
}
