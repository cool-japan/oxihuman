// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::TAU;

pub struct WaterSurfaceView {
    pub show_normals: bool,
    pub show_foam: bool,
    pub wave_amplitude: f32,
    pub wave_frequency: f32,
}

pub fn new_water_surface_view() -> WaterSurfaceView {
    WaterSurfaceView {
        show_normals: false,
        show_foam: false,
        wave_amplitude: 0.3,
        wave_frequency: 1.0,
    }
}

/// Gerstner wave height at (x, z, t).
pub fn water_gerstner_height(x: f32, z: f32, t: f32, amplitude: f32, frequency: f32) -> f32 {
    let phase = TAU * frequency * (x + z) - t;
    amplitude * phase.sin()
}

pub fn water_foam_factor(wave_height: f32, threshold: f32) -> f32 {
    ((wave_height - threshold) / threshold.abs().max(1e-6)).clamp(0.0, 1.0)
}

pub fn water_normal_from_height(h: impl Fn(f32, f32) -> f32, x: f32, z: f32, eps: f32) -> [f32; 3] {
    let dx = (h(x + eps, z) - h(x - eps, z)) / (2.0 * eps);
    let dz = (h(x, z + eps) - h(x, z - eps)) / (2.0 * eps);
    let len = (dx * dx + 1.0 + dz * dz).sqrt();
    [-dx / len, 1.0 / len, -dz / len]
}

pub fn water_fresnel(cos_theta: f32, ior: f32) -> f32 {
    let f0 = ((ior - 1.0) / (ior + 1.0)).powi(2);
    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_water_surface_view() {
        /* wave amplitude defaults to 0.3 */
        let v = new_water_surface_view();
        assert!((v.wave_amplitude - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_water_gerstner_height_range() {
        /* height should be bounded by amplitude */
        let h = water_gerstner_height(0.0, 0.0, 0.0, 0.3, 1.0);
        assert!(h.abs() <= 0.3 + 1e-5);
    }

    #[test]
    fn test_water_foam_factor_zero() {
        /* below threshold, foam is 0 */
        let f = water_foam_factor(0.0, 0.5);
        assert_eq!(f, 0.0);
    }

    #[test]
    fn test_water_normal_from_height_up() {
        /* flat surface has normal pointing up */
        let n = water_normal_from_height(|_, _| 0.0, 0.0, 0.0, 0.01);
        assert!(n[1] > 0.9);
    }

    #[test]
    fn test_water_fresnel_grazing() {
        /* at grazing angle (cos=0), fresnel approaches 1 */
        let f = water_fresnel(0.0, 1.33);
        assert!((f - 1.0).abs() < 1e-5);
    }
}
