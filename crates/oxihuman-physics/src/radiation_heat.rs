// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// Stefan-Boltzmann constant (W·m⁻²·K⁻⁴).
pub const STEFAN_BOLTZMANN: f32 = 5.67e-8;

/// Radiation heat model based on Stefan-Boltzmann law.
#[derive(Debug, Clone)]
pub struct RadiationHeatModel {
    pub emissivity: f32,
    pub area: f32,
    pub temperature: f32,
}

/// Create a new RadiationHeatModel.
pub fn new_radiation_heat(eps: f32, area: f32, temp: f32) -> RadiationHeatModel {
    RadiationHeatModel {
        emissivity: eps,
        area,
        temperature: temp,
    }
}

/// Total radiated power: P = eps * sigma * A * T^4.
pub fn radiated_power(m: &RadiationHeatModel) -> f32 {
    m.emissivity * STEFAN_BOLTZMANN * m.area * m.temperature.powi(4)
}

/// Net radiation exchange with an ambient body at t_ambient (K).
pub fn net_radiation(m: &RadiationHeatModel, t_ambient: f32) -> f32 {
    m.emissivity * STEFAN_BOLTZMANN * m.area * (m.temperature.powi(4) - t_ambient.powi(4))
}

/// Blackbody emission intensity: sigma * T^4.
pub fn blackbody_emission(temp: f32) -> f32 {
    STEFAN_BOLTZMANN * temp.powi(4)
}

/// Radiative heat flux (W/m²): eps * sigma * T^4.
pub fn radiative_flux(m: &RadiationHeatModel) -> f32 {
    m.emissivity * STEFAN_BOLTZMANN * m.temperature.powi(4)
}

/// Effective radiation temperature given power input (inverse SB law).
pub fn effective_temperature(m: &RadiationHeatModel, power: f32) -> f32 {
    let denom = m.emissivity * STEFAN_BOLTZMANN * m.area;
    if denom < 1e-30 {
        return 0.0;
    }
    (power / denom).powf(0.25)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_radiation_heat() {
        /* constructor */
        let m = new_radiation_heat(0.9, 1.0, 300.0);
        assert!((m.emissivity - 0.9).abs() < 1e-9);
        assert!((m.area - 1.0).abs() < 1e-9);
        assert!((m.temperature - 300.0).abs() < 1e-9);
    }

    #[test]
    fn test_radiated_power_positive() {
        /* power at 300 K */
        let m = new_radiation_heat(1.0, 1.0, 300.0);
        let p = radiated_power(&m);
        let expected = STEFAN_BOLTZMANN * 300.0f32.powi(4);
        assert!((p - expected).abs() / expected < 1e-4);
    }

    #[test]
    fn test_blackbody_emission_zero() {
        /* zero temperature -> zero emission */
        let e = blackbody_emission(0.0);
        assert!(e.abs() < 1e-9);
    }

    #[test]
    fn test_net_radiation_self() {
        /* same temp -> zero net */
        let m = new_radiation_heat(0.8, 1.0, 300.0);
        let nr = net_radiation(&m, 300.0);
        assert!(nr.abs() < 1e-3);
    }

    #[test]
    fn test_net_radiation_sign() {
        /* hotter surface -> positive net radiation */
        let m = new_radiation_heat(0.8, 1.0, 400.0);
        let nr = net_radiation(&m, 300.0);
        assert!(nr > 0.0);
    }

    #[test]
    fn test_stefan_boltzmann_constant() {
        /* constant value check */
        assert!((STEFAN_BOLTZMANN - 5.67e-8_f32).abs() < 1e-12);
    }

    #[test]
    fn test_radiative_flux() {
        let m = new_radiation_heat(1.0, 2.0, 300.0);
        let flux = radiative_flux(&m);
        let expected = STEFAN_BOLTZMANN * 300.0f32.powi(4);
        assert!((flux - expected).abs() / expected < 1e-4);
    }

    #[test]
    fn test_effective_temperature_roundtrip() {
        /* compute power then invert */
        let m = new_radiation_heat(1.0, 1.0, 400.0);
        let p = radiated_power(&m);
        let t = effective_temperature(&m, p);
        assert!((t - 400.0).abs() < 0.1);
    }

    #[test]
    fn test_blackbody_emission_doubles() {
        /* doubling T increases by 2^4 = 16 */
        let e1 = blackbody_emission(200.0);
        let e2 = blackbody_emission(400.0);
        assert!((e2 / e1 - 16.0).abs() < 0.01);
    }
}
