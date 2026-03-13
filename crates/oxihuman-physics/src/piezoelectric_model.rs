// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Piezoelectric coupling model stub.

/// Piezoelectric material parameters (simplified, isotropic stub).
#[derive(Debug, Clone)]
pub struct PiezoConfig {
    pub d33: f32, /* strain coefficient along polarization axis [m/V] */
    pub d31: f32, /* transverse strain coefficient [m/V] */
    pub e33: f32, /* stress coefficient [C/m^2] */
    pub young_modulus: f32,
    pub permittivity: f32, /* dielectric permittivity [F/m] */
}

impl PiezoConfig {
    pub fn new(d33: f32, d31: f32, e33: f32, young_modulus: f32, permittivity: f32) -> Self {
        PiezoConfig {
            d33,
            d31,
            e33,
            young_modulus,
            permittivity,
        }
    }

    pub fn pzt5h() -> Self {
        /* PZT-5H typical values */
        PiezoConfig::new(593e-12, -274e-12, 23.3, 61e9, 3400.0 * 8.854e-12)
    }
}

impl Default for PiezoConfig {
    fn default() -> Self {
        Self::pzt5h()
    }
}

/// Converse piezoelectric effect: strain from electric field.
pub fn strain_from_field(config: &PiezoConfig, e_field: f32) -> f32 {
    config.d33 * e_field
}

/// Transverse strain from electric field.
pub fn transverse_strain_from_field(config: &PiezoConfig, e_field: f32) -> f32 {
    config.d31 * e_field
}

/// Direct piezoelectric effect: polarization from stress.
pub fn polarization_from_stress(config: &PiezoConfig, stress: f32) -> f32 {
    config.d33 * stress
}

/// Mechanical displacement along thickness `t` for voltage `V`.
pub fn displacement_from_voltage(config: &PiezoConfig, thickness: f32, voltage: f32) -> f32 {
    if thickness <= 0.0 {
        return 0.0;
    }
    let e_field = voltage / thickness;
    strain_from_field(config, e_field) * thickness
}

/// Blocking force: maximum force at zero displacement.
pub fn blocking_force(config: &PiezoConfig, voltage: f32, area: f32) -> f32 {
    let strain_free = config.d33 * voltage;
    config.young_modulus * strain_free * area
}

/// Electromechanical coupling coefficient k33.
pub fn coupling_coefficient(config: &PiezoConfig) -> f32 {
    /* k33^2 = d33^2 * Y33^E / eps33^T */
    let k33_sq = config.d33 * config.d33 * config.young_modulus / config.permittivity;
    k33_sq.sqrt().min(1.0)
}

/// Generated charge from stress over area.
pub fn charge_from_stress(config: &PiezoConfig, stress: f32, area: f32) -> f32 {
    polarization_from_stress(config, stress) * area
}

/// Resonant frequency (simplified, length-extensional mode).
pub fn resonant_frequency_stub(config: &PiezoConfig, length: f32, density: f32) -> f32 {
    if length <= 0.0 || density <= 0.0 {
        return 0.0;
    }
    let v = (config.young_modulus / density).sqrt();
    v / (2.0 * length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strain_from_field_positive() {
        let c = PiezoConfig::default();
        /* d33 > 0, E > 0 => positive strain */
        assert!(strain_from_field(&c, 1e6) > 0.0);
    }

    #[test]
    fn test_transverse_strain_opposite_sign() {
        let c = PiezoConfig::default();
        /* d31 < 0, E > 0 => negative transverse strain */
        assert!(transverse_strain_from_field(&c, 1e6) < 0.0);
    }

    #[test]
    fn test_polarization_from_stress() {
        let c = PiezoConfig::default();
        let p = polarization_from_stress(&c, 1e6);
        assert!(p > 0.0);
    }

    #[test]
    fn test_displacement_from_voltage_positive() {
        let c = PiezoConfig::default();
        let d = displacement_from_voltage(&c, 1e-3, 100.0);
        assert!(d > 0.0);
    }

    #[test]
    fn test_displacement_zero_thickness() {
        let c = PiezoConfig::default();
        assert_eq!(displacement_from_voltage(&c, 0.0, 100.0), 0.0);
    }

    #[test]
    fn test_blocking_force_positive() {
        let c = PiezoConfig::default();
        assert!(blocking_force(&c, 100.0, 1e-4) > 0.0);
    }

    #[test]
    fn test_coupling_coefficient_range() {
        let c = PiezoConfig::default();
        let k = coupling_coefficient(&c);
        assert!((0.0..=1.0).contains(&k));
    }

    #[test]
    fn test_charge_from_stress_positive() {
        let c = PiezoConfig::default();
        let q = charge_from_stress(&c, 1e6, 1e-4);
        assert!(q > 0.0);
    }

    #[test]
    fn test_resonant_frequency_positive() {
        let c = PiezoConfig::default();
        let f = resonant_frequency_stub(&c, 0.01, 7500.0);
        assert!(f > 0.0);
    }

    #[test]
    fn test_resonant_frequency_zero_length() {
        let c = PiezoConfig::default();
        assert_eq!(resonant_frequency_stub(&c, 0.0, 7500.0), 0.0);
    }
}
