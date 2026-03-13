// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

pub struct HairShaderView {
    pub show_r: bool,
    pub show_tt: bool,
    pub show_trt: bool,
    pub fiber_roughness: f32,
}

pub fn new_hair_shader_view() -> HairShaderView {
    HairShaderView {
        show_r: true,
        show_tt: true,
        show_trt: true,
        fiber_roughness: 0.3,
    }
}

/// R lobe (specular reflection).
pub fn hair_r_lobe_color(theta_deg: f32) -> [f32; 3] {
    let t = (theta_deg / 180.0).clamp(0.0, 1.0);
    [1.0, 1.0 - t, 0.0]
}

/// TT lobe (transmission).
pub fn hair_tt_lobe_color(theta_deg: f32) -> [f32; 3] {
    let t = (theta_deg / 180.0).clamp(0.0, 1.0);
    [0.8 * (1.0 - t), 0.9, 0.0]
}

/// TRT lobe (glint).
pub fn hair_trt_lobe_color(theta_deg: f32) -> [f32; 3] {
    let t = (theta_deg / 180.0).clamp(0.0, 1.0);
    [1.0, 0.8 * t, 0.0]
}

/// Azimuthal distribution (Gaussian approximation).
pub fn hair_azimuthal_distribution(phi: f32, roughness: f32) -> f32 {
    let beta = (roughness * PI).max(1e-4);
    (-phi * phi / (2.0 * beta * beta)).exp() / (beta * (2.0 * PI).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_hair_shader_view() {
        /* fiber roughness defaults to 0.3 */
        let v = new_hair_shader_view();
        assert!((v.fiber_roughness - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_hair_r_lobe_color() {
        /* at theta=0, green channel is 1 */
        let c = hair_r_lobe_color(0.0);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_hair_tt_lobe_color() {
        /* color has non-zero components */
        let c = hair_tt_lobe_color(90.0);
        assert!(c[1] > 0.0);
    }

    #[test]
    fn test_hair_trt_lobe_color() {
        /* red channel is 1 */
        let c = hair_trt_lobe_color(0.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_hair_azimuthal_distribution_peak() {
        /* distribution peaks at phi=0 */
        let peak = hair_azimuthal_distribution(0.0, 0.3);
        let off = hair_azimuthal_distribution(1.0, 0.3);
        assert!(peak > off);
    }
}
