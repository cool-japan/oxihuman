// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Laser body: directed energy beam with power and attenuation model.

use std::f32::consts::PI;

/// Configuration for a laser beam.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LaserConfig {
    pub power_watts: f32,
    pub wavelength_nm: f32,
    pub divergence_rad: f32,
    pub attenuation_per_m: f32,
}

impl Default for LaserConfig {
    fn default() -> Self {
        Self {
            power_watts: 1.0,
            wavelength_nm: 532.0,
            divergence_rad: 1e-3,
            attenuation_per_m: 0.01,
        }
    }
}

/// A laser body in the simulation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LaserBody {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    pub config: LaserConfig,
    pub active: bool,
    pub energy_deposited_j: f32,
}

/// Create a new `LaserBody`.
#[allow(dead_code)]
pub fn new_laser_body(origin: [f32; 3], direction: [f32; 3]) -> LaserBody {
    LaserBody {
        origin,
        direction: normalize3(direction),
        config: LaserConfig::default(),
        active: true,
        energy_deposited_j: 0.0,
    }
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Power at distance `d` meters (Beer-Lambert attenuation).
#[allow(dead_code)]
pub fn lb_power_at(body: &LaserBody, d: f32) -> f32 {
    if !body.active || d < 0.0 {
        return 0.0;
    }
    body.config.power_watts * (-body.config.attenuation_per_m * d).exp()
}

/// Beam radius at distance `d` (Gaussian beam approximation).
#[allow(dead_code)]
pub fn lb_beam_radius(body: &LaserBody, d: f32) -> f32 {
    body.config.divergence_rad * d.max(0.0)
}

/// Irradiance (W/m²) at distance `d` assuming uniform disk profile.
#[allow(dead_code)]
pub fn lb_irradiance(body: &LaserBody, d: f32) -> f32 {
    let r = lb_beam_radius(body, d).max(1e-9);
    lb_power_at(body, d) / (PI * r * r)
}

/// Step the laser by `dt` seconds — deposit energy into `energy_deposited_j`.
#[allow(dead_code)]
pub fn lb_step(body: &mut LaserBody, dt: f32) {
    if body.active && dt > 0.0 {
        body.energy_deposited_j += body.config.power_watts * dt;
    }
}

/// Set laser active state.
#[allow(dead_code)]
pub fn lb_set_active(body: &mut LaserBody, active: bool) {
    body.active = active;
}

/// Set power.
#[allow(dead_code)]
pub fn lb_set_power(body: &mut LaserBody, watts: f32) {
    body.config.power_watts = watts.max(0.0);
}

/// Reset deposited energy.
#[allow(dead_code)]
pub fn lb_reset_energy(body: &mut LaserBody) {
    body.energy_deposited_j = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_laser_body() {
        let lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        assert!(lb.active);
        assert!((lb.energy_deposited_j).abs() < 1e-9);
    }

    #[test]
    fn test_power_at_zero_distance() {
        let lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        assert!((lb_power_at(&lb, 0.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_power_decreases_with_distance() {
        let lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        let p0 = lb_power_at(&lb, 0.0);
        let p1 = lb_power_at(&lb, 10.0);
        assert!(p0 > p1);
    }

    #[test]
    fn test_inactive_power_zero() {
        let mut lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        lb_set_active(&mut lb, false);
        assert!((lb_power_at(&lb, 0.0)).abs() < 1e-9);
    }

    #[test]
    fn test_beam_radius_grows() {
        let lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        assert!(lb_beam_radius(&lb, 100.0) > lb_beam_radius(&lb, 10.0));
    }

    #[test]
    fn test_irradiance_positive() {
        let lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        assert!(lb_irradiance(&lb, 1.0) > 0.0);
    }

    #[test]
    fn test_step_deposits_energy() {
        let mut lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        lb_step(&mut lb, 1.0);
        assert!((lb.energy_deposited_j - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_inactive_no_energy() {
        let mut lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        lb_set_active(&mut lb, false);
        lb_step(&mut lb, 1.0);
        assert!((lb.energy_deposited_j).abs() < 1e-9);
    }

    #[test]
    fn test_reset_energy() {
        let mut lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        lb_step(&mut lb, 2.0);
        lb_reset_energy(&mut lb);
        assert!((lb.energy_deposited_j).abs() < 1e-9);
    }

    #[test]
    fn test_set_power() {
        let mut lb = new_laser_body([0.0; 3], [0.0, 0.0, 1.0]);
        lb_set_power(&mut lb, 50.0);
        assert!((lb.config.power_watts - 50.0).abs() < 1e-5);
    }
}
