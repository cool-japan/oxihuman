// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Electrowetting-on-dielectric (EWOD) stub.

use std::f32::consts::PI;

/// EWOD configuration.
#[derive(Debug, Clone)]
pub struct EwodConfig {
    pub surface_tension: f32,         /* gamma [N/m] */
    pub contact_angle_eq: f32,        /* Young angle [rad] */
    pub dielectric_thickness: f32,    /* d [m] */
    pub dielectric_permittivity: f32, /* eps_d [F/m] */
}

impl EwodConfig {
    pub fn new(
        surface_tension: f32,
        contact_angle_eq: f32,
        dielectric_thickness: f32,
        dielectric_permittivity: f32,
    ) -> Self {
        EwodConfig {
            surface_tension,
            contact_angle_eq,
            dielectric_thickness,
            dielectric_permittivity,
        }
    }
}

impl Default for EwodConfig {
    fn default() -> Self {
        EwodConfig::new(0.072, 110.0 * PI / 180.0, 1e-6, 2.1 * 8.854e-12)
    }
}

/// Lippmann-Young equation: cos(theta) = cos(theta0) + C*V^2/(2*gamma).
pub fn contact_angle_ewod(config: &EwodConfig, voltage: f32) -> f32 {
    let c = config.dielectric_permittivity / config.dielectric_thickness;
    let cos_theta =
        config.contact_angle_eq.cos() + c * voltage * voltage / (2.0 * config.surface_tension);
    cos_theta.clamp(-1.0, 1.0).acos()
}

/// Electrowetting number (dimensionless).
pub fn ewod_number(config: &EwodConfig, voltage: f32) -> f32 {
    let c = config.dielectric_permittivity / config.dielectric_thickness;
    c * voltage * voltage / (2.0 * config.surface_tension)
}

/// Capillary pressure for a droplet of radius `r`.
pub fn capillary_pressure(config: &EwodConfig, radius: f32, voltage: f32) -> f32 {
    if radius <= 0.0 {
        return 0.0;
    }
    let theta = contact_angle_ewod(config, voltage);
    2.0 * config.surface_tension * theta.cos() / radius
}

/// Contact line force (electrowetting force per unit length).
pub fn contact_line_force(config: &EwodConfig, voltage: f32) -> f32 {
    let c = config.dielectric_permittivity / config.dielectric_thickness;
    0.5 * c * voltage * voltage
}

/// Droplet spreading velocity (stub, dimensional analysis).
pub fn spreading_velocity_stub(config: &EwodConfig, voltage: f32, viscosity: f32) -> f32 {
    if viscosity <= 0.0 {
        return 0.0;
    }
    contact_line_force(config, voltage) / viscosity
}

/// Check if contact angle can be reduced further (saturation limit ~0 deg).
pub fn is_saturated(config: &EwodConfig, voltage: f32) -> bool {
    let theta = contact_angle_ewod(config, voltage);
    theta < 5.0 * PI / 180.0 /* less than 5 degrees */
}

/// Restore contact angle (voltage = 0).
pub fn restore_contact_angle(config: &EwodConfig) -> f32 {
    config.contact_angle_eq
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_angle_zero_voltage() {
        let c = EwodConfig::default();
        let theta = contact_angle_ewod(&c, 0.0);
        assert!((theta - c.contact_angle_eq).abs() < 1e-5);
    }

    #[test]
    fn test_contact_angle_decreases_with_voltage() {
        let c = EwodConfig::default();
        let t0 = contact_angle_ewod(&c, 0.0);
        let t1 = contact_angle_ewod(&c, 10.0);
        /* electrowetting reduces contact angle */
        assert!(t1 <= t0);
    }

    #[test]
    fn test_ewod_number_zero() {
        let c = EwodConfig::default();
        assert_eq!(ewod_number(&c, 0.0), 0.0);
    }

    #[test]
    fn test_ewod_number_positive() {
        let c = EwodConfig::default();
        assert!(ewod_number(&c, 10.0) > 0.0);
    }

    #[test]
    fn test_capillary_pressure_zero_radius() {
        let c = EwodConfig::default();
        assert_eq!(capillary_pressure(&c, 0.0, 0.0), 0.0);
    }

    #[test]
    fn test_contact_line_force_positive() {
        let c = EwodConfig::default();
        assert!(contact_line_force(&c, 10.0) > 0.0);
    }

    #[test]
    fn test_contact_line_force_zero_voltage() {
        let c = EwodConfig::default();
        assert_eq!(contact_line_force(&c, 0.0), 0.0);
    }

    #[test]
    fn test_spreading_velocity_zero_viscosity() {
        let c = EwodConfig::default();
        assert_eq!(spreading_velocity_stub(&c, 10.0, 0.0), 0.0);
    }

    #[test]
    fn test_restore_contact_angle() {
        let c = EwodConfig::default();
        assert!((restore_contact_angle(&c) - c.contact_angle_eq).abs() < 1e-10);
    }

    #[test]
    fn test_contact_angle_clamped() {
        let c = EwodConfig::default();
        let theta = contact_angle_ewod(&c, 1e6); /* extreme voltage */
        assert!((0.0..=PI).contains(&theta));
    }
}
