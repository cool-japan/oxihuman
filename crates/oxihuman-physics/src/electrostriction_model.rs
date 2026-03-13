// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Electrostriction deformation stub.

/// Electrostriction material parameters.
#[derive(Debug, Clone)]
pub struct ElectrostrictiveConfig {
    pub m11: f32,          /* electrostriction coefficient [m^2/V^2] */
    pub m12: f32,          /* transverse electrostriction coefficient */
    pub permittivity: f32, /* dielectric permittivity [F/m] */
    pub young_modulus: f32,
}

impl ElectrostrictiveConfig {
    pub fn new(m11: f32, m12: f32, permittivity: f32, young_modulus: f32) -> Self {
        ElectrostrictiveConfig {
            m11,
            m12,
            permittivity,
            young_modulus,
        }
    }

    pub fn pmn_pt() -> Self {
        /* PMN-PT typical values (stub) */
        ElectrostrictiveConfig::new(2.6e-18, -1.3e-18, 3000.0 * 8.854e-12, 120e9)
    }
}

impl Default for ElectrostrictiveConfig {
    fn default() -> Self {
        Self::pmn_pt()
    }
}

/// Electrostrictive strain (quadratic in polarization).
pub fn electrostrictive_strain(config: &ElectrostrictiveConfig, polarization: f32) -> f32 {
    config.m11 * polarization * polarization
}

/// Transverse electrostrictive strain.
pub fn transverse_strain(config: &ElectrostrictiveConfig, polarization: f32) -> f32 {
    config.m12 * polarization * polarization
}

/// Polarization from electric field (linear approximation).
pub fn polarization_from_field(config: &ElectrostrictiveConfig, e_field: f32) -> f32 {
    config.permittivity * e_field
}

/// Strain from electric field (combining polarization step).
pub fn strain_from_e_field(config: &ElectrostrictiveConfig, e_field: f32) -> f32 {
    let p = polarization_from_field(config, e_field);
    electrostrictive_strain(config, p)
}

/// Maxwell stress (electrostatic pressure on dielectric surface).
pub fn maxwell_stress(config: &ElectrostrictiveConfig, e_field: f32) -> f32 {
    0.5 * config.permittivity * e_field * e_field
}

/// Electrostriction force on a capacitor of area `area` and gap `gap`.
pub fn electrostriction_force(
    config: &ElectrostrictiveConfig,
    voltage: f32,
    gap: f32,
    area: f32,
) -> f32 {
    if gap <= 0.0 {
        return 0.0;
    }
    let e_field = voltage / gap;
    maxwell_stress(config, e_field) * area
}

/// Energy stored in dielectric under applied field.
pub fn dielectric_energy(config: &ElectrostrictiveConfig, e_field: f32, volume: f32) -> f32 {
    0.5 * config.permittivity * e_field * e_field * volume.max(0.0)
}

/// Actuation strain as a function of DC bias field.
pub fn dc_bias_strain(config: &ElectrostrictiveConfig, dc_field: f32, ac_field: f32) -> f32 {
    let p_dc = polarization_from_field(config, dc_field);
    let p_ac = polarization_from_field(config, ac_field);
    /* Linear approximation: S ≈ 2*M*P_dc*P_ac */
    2.0 * config.m11 * p_dc * p_ac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electrostrictive_strain_positive() {
        let c = ElectrostrictiveConfig::default();
        /* m11 > 0 and P^2 >= 0 => positive strain */
        let s = electrostrictive_strain(&c, 0.1);
        assert!(s >= 0.0);
    }

    #[test]
    fn test_electrostrictive_strain_quadratic() {
        let c = ElectrostrictiveConfig::default();
        let s1 = electrostrictive_strain(&c, 1.0);
        let s2 = electrostrictive_strain(&c, 2.0);
        /* doubling P should quadruple strain */
        assert!((s2 / s1 - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_polarization_from_field_positive() {
        let c = ElectrostrictiveConfig::default();
        assert!(polarization_from_field(&c, 1e6) > 0.0);
    }

    #[test]
    fn test_strain_from_e_field_positive() {
        let c = ElectrostrictiveConfig::default();
        assert!(strain_from_e_field(&c, 1e6) > 0.0);
    }

    #[test]
    fn test_maxwell_stress_positive() {
        let c = ElectrostrictiveConfig::default();
        assert!(maxwell_stress(&c, 1e6) > 0.0);
    }

    #[test]
    fn test_electrostriction_force_zero_gap() {
        let c = ElectrostrictiveConfig::default();
        assert_eq!(electrostriction_force(&c, 100.0, 0.0, 1e-4), 0.0);
    }

    #[test]
    fn test_electrostriction_force_positive() {
        let c = ElectrostrictiveConfig::default();
        assert!(electrostriction_force(&c, 100.0, 1e-3, 1e-4) > 0.0);
    }

    #[test]
    fn test_dielectric_energy_positive() {
        let c = ElectrostrictiveConfig::default();
        assert!(dielectric_energy(&c, 1e6, 1e-6) > 0.0);
    }

    #[test]
    fn test_dielectric_energy_zero_volume() {
        let c = ElectrostrictiveConfig::default();
        assert_eq!(dielectric_energy(&c, 1e6, 0.0), 0.0);
    }

    #[test]
    fn test_transverse_strain_sign() {
        let c = ElectrostrictiveConfig::default();
        /* m12 < 0 => negative transverse strain */
        assert!(transverse_strain(&c, 1.0) < 0.0);
    }
}
