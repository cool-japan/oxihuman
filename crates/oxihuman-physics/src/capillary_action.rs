// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Model of a capillary tube for Jurin's law calculations.
pub struct CapillaryTube {
    pub radius: f32,
    pub contact_angle_rad: f32,
    pub surface_tension: f32,
    pub fluid_density: f32,
    pub g: f32,
}

impl CapillaryTube {
    pub fn new_water(radius: f32) -> Self {
        CapillaryTube {
            radius,
            contact_angle_rad: 0.0,
            surface_tension: 0.072,
            fluid_density: 1000.0,
            g: 9.81,
        }
    }
}

pub fn new_capillary_tube(radius: f32) -> CapillaryTube {
    CapillaryTube::new_water(radius)
}

/// h = 2γ cos θ / (ρ g r)
pub fn capillary_rise_height(t: &CapillaryTube) -> f32 {
    2.0 * t.surface_tension * t.contact_angle_rad.cos() / (t.fluid_density * t.g * t.radius)
}

/// ΔP = 2γ cos θ / r
pub fn capillary_pressure(t: &CapillaryTube) -> f32 {
    2.0 * t.surface_tension * t.contact_angle_rad.cos() / t.radius
}

/// V = π r² h
pub fn capillary_volume(t: &CapillaryTube) -> f32 {
    PI * t.radius * t.radius * capillary_rise_height(t)
}

pub fn capillary_set_angle(t: &mut CapillaryTube, angle_rad: f32) {
    t.contact_angle_rad = angle_rad;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        /* new tube has water defaults */
        let t = new_capillary_tube(1e-4);
        assert!((t.surface_tension - 0.072).abs() < 1e-6);
        assert!((t.fluid_density - 1000.0).abs() < 1e-3);
    }

    #[test]
    fn test_rise_height_positive() {
        /* rise height is positive for water (zero angle) */
        let t = new_capillary_tube(1e-4);
        assert!(capillary_rise_height(&t) > 0.0);
    }

    #[test]
    fn test_rise_height_decreases_with_radius() {
        /* larger radius => lower rise */
        let t1 = new_capillary_tube(1e-4);
        let t2 = new_capillary_tube(2e-4);
        assert!(capillary_rise_height(&t1) > capillary_rise_height(&t2));
    }

    #[test]
    fn test_pressure_positive() {
        /* capillary pressure is positive */
        let t = new_capillary_tube(1e-4);
        assert!(capillary_pressure(&t) > 0.0);
    }

    #[test]
    fn test_volume_positive() {
        /* capillary volume is positive */
        let t = new_capillary_tube(1e-4);
        assert!(capillary_volume(&t) > 0.0);
    }

    #[test]
    fn test_set_angle_90_degrees() {
        /* contact angle of 90° gives zero rise */
        let mut t = new_capillary_tube(1e-4);
        capillary_set_angle(&mut t, std::f32::consts::FRAC_PI_2);
        assert!(capillary_rise_height(&t).abs() < 1e-4);
    }

    #[test]
    fn test_jurin_law_numeric() {
        /* verify Jurin's law: h ≈ 0.147 m for r=1e-4 m water */
        let t = new_capillary_tube(1e-4);
        let h = capillary_rise_height(&t);
        assert!((h - 0.1468).abs() < 0.01);
    }
}
