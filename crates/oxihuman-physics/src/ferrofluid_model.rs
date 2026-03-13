// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ferrofluid deformation model.

use std::f32::consts::PI;

/// Ferrofluid parameters.
#[derive(Debug, Clone)]
pub struct FerrofluidConfig {
    pub saturation_magnetization: f32, /* Ms [A/m] */
    pub initial_susceptibility: f32,   /* chi_0 */
    pub viscosity: f32,                /* eta [Pa.s] */
    pub density: f32,                  /* rho [kg/m^3] */
    pub surface_tension: f32,          /* gamma [N/m] */
}

impl FerrofluidConfig {
    pub fn new(
        saturation_magnetization: f32,
        initial_susceptibility: f32,
        viscosity: f32,
        density: f32,
        surface_tension: f32,
    ) -> Self {
        FerrofluidConfig {
            saturation_magnetization,
            initial_susceptibility,
            viscosity,
            density,
            surface_tension,
        }
    }

    pub fn typical() -> Self {
        FerrofluidConfig::new(400e3, 3.0, 0.006, 1200.0, 0.026)
    }
}

impl Default for FerrofluidConfig {
    fn default() -> Self {
        Self::typical()
    }
}

/// Langevin magnetization M(H) (simplified).
pub fn magnetization(config: &FerrofluidConfig, h_field: f32) -> f32 {
    if h_field.abs() < 1e-20 {
        return 0.0;
    }
    let ms = config.saturation_magnetization;
    /* Simplified: M = Ms * tanh(chi * H / Ms) */
    let x = config.initial_susceptibility * h_field / ms;
    ms * x.tanh()
}

/// Magnetic pressure on ferrofluid interface.
pub fn magnetic_pressure(config: &FerrofluidConfig, h_field: f32) -> f32 {
    let mu0 = 4.0e-7 * PI;
    let m = magnetization(config, h_field);
    0.5 * mu0 * m * m
}

/// Rosensweig (normal field) instability threshold field.
pub fn rosensweig_threshold(config: &FerrofluidConfig) -> f32 {
    let mu0 = 4.0e-7 * PI;
    /* H_c = sqrt(2*gamma / (mu0*chi)) roughly */
    if mu0 * config.initial_susceptibility <= 0.0 {
        return 0.0;
    }
    (2.0 * config.surface_tension / (mu0 * config.initial_susceptibility)).sqrt()
}

/// Ferrohydrodynamic body force density (Kelvin force).
pub fn kelvin_force_density(config: &FerrofluidConfig, h_field: f32, grad_h: f32) -> f32 {
    let mu0 = 4.0e-7 * PI;
    let m = magnetization(config, h_field);
    mu0 * m * grad_h
}

/// Spike height above critical field (Rosensweig peaks, stub).
pub fn spike_height(config: &FerrofluidConfig, h_field: f32) -> f32 {
    let h_c = rosensweig_threshold(config);
    if h_field <= h_c {
        return 0.0;
    }
    let excess = h_field - h_c;
    let g = 9.81f32;
    /* Balance magnetic pressure vs gravitational: stub estimate */
    let mu0 = 4.0e-7 * PI;
    mu0 * excess * excess / (2.0 * config.density * g)
}

/// Effective viscosity of ferrofluid in magnetic field (rotational viscosity stub).
pub fn effective_viscosity(config: &FerrofluidConfig, h_field: f32, shear_rate: f32) -> f32 {
    if shear_rate.abs() < 1e-12 {
        return config.viscosity;
    }
    let m = magnetization(config, h_field);
    let delta_eta = 0.5 * m * m * 4.0e-7 * PI / shear_rate.abs();
    config.viscosity + delta_eta.min(config.viscosity * 10.0)
}

/// Wetting pressure (capillary vs magnetic balance).
pub fn wetting_pressure(config: &FerrofluidConfig, radius: f32, h_field: f32) -> f32 {
    if radius <= 0.0 {
        return 0.0;
    }
    let cap = 2.0 * config.surface_tension / radius;
    let mag = magnetic_pressure(config, h_field);
    cap + mag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnetization_zero_field() {
        let c = FerrofluidConfig::default();
        assert_eq!(magnetization(&c, 0.0), 0.0);
    }

    #[test]
    fn test_magnetization_saturates() {
        let c = FerrofluidConfig::default();
        let m = magnetization(&c, 1e9); /* very large field */
        assert!(m < c.saturation_magnetization * 1.01);
    }

    #[test]
    fn test_magnetization_positive_positive_field() {
        let c = FerrofluidConfig::default();
        assert!(magnetization(&c, 1e4) > 0.0);
    }

    #[test]
    fn test_magnetic_pressure_positive() {
        let c = FerrofluidConfig::default();
        assert!(magnetic_pressure(&c, 1e4) > 0.0);
    }

    #[test]
    fn test_rosensweig_threshold_positive() {
        let c = FerrofluidConfig::default();
        assert!(rosensweig_threshold(&c) > 0.0);
    }

    #[test]
    fn test_spike_height_zero_below_threshold() {
        let c = FerrofluidConfig::default();
        let h_c = rosensweig_threshold(&c);
        assert_eq!(spike_height(&c, h_c - 1.0), 0.0);
    }

    #[test]
    fn test_spike_height_positive_above_threshold() {
        let c = FerrofluidConfig::default();
        let h_c = rosensweig_threshold(&c);
        assert!(spike_height(&c, h_c + 1000.0) > 0.0);
    }

    #[test]
    fn test_kelvin_force_density_positive() {
        let c = FerrofluidConfig::default();
        assert!(kelvin_force_density(&c, 1e4, 1e4) > 0.0);
    }

    #[test]
    fn test_effective_viscosity_increases_with_field() {
        let c = FerrofluidConfig::default();
        let v0 = effective_viscosity(&c, 0.0, 100.0);
        let v1 = effective_viscosity(&c, 1e5, 100.0);
        assert!(v1 >= v0);
    }

    #[test]
    fn test_wetting_pressure_zero_radius() {
        let c = FerrofluidConfig::default();
        assert_eq!(wetting_pressure(&c, 0.0, 1e4), 0.0);
    }
}
