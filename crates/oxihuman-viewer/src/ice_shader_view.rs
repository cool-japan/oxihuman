// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct IceShaderView {
    pub show_absorption: bool,
    pub show_refraction: bool,
    pub ior: f32,
    pub absorption_coeff: [f32; 3],
}

pub fn new_ice_shader_view() -> IceShaderView {
    IceShaderView {
        show_absorption: true,
        show_refraction: true,
        ior: 1.31,
        absorption_coeff: [0.01, 0.005, 0.001],
    }
}

/// Beer-Lambert absorption over distance.
pub fn ice_absorption_color(distance_m: f32, coeff: [f32; 3]) -> [f32; 3] {
    [
        (-coeff[0] * distance_m).exp(),
        (-coeff[1] * distance_m).exp(),
        (-coeff[2] * distance_m).exp(),
    ]
}

pub fn ice_refraction_offset(incident_dir: [f32; 3], normal: [f32; 3], ior: f32) -> [f32; 3] {
    let cos_i =
        -(incident_dir[0] * normal[0] + incident_dir[1] * normal[1] + incident_dir[2] * normal[2]);
    let sin2_t = (1.0 / ior).powi(2) * (1.0 - cos_i * cos_i).max(0.0);
    if sin2_t > 1.0 {
        return [0.0, 0.0, 0.0];
    }
    let cos_t = (1.0 - sin2_t).sqrt();
    let scale = 1.0 / ior;
    let bias = scale * cos_i - cos_t;
    [
        scale * incident_dir[0] + bias * normal[0],
        scale * incident_dir[1] + bias * normal[1],
        scale * incident_dir[2] + bias * normal[2],
    ]
}

pub fn ice_fresnel(cos_theta: f32, ior: f32) -> f32 {
    let f0 = ((ior - 1.0) / (ior + 1.0)).powi(2);
    f0 + (1.0 - f0) * (1.0 - cos_theta.clamp(0.0, 1.0)).powi(5)
}

pub fn ice_caustic_strength(thickness_m: f32) -> f32 {
    (1.0 - (-thickness_m * 0.5).exp()).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ice_shader_view() {
        /* ior defaults to 1.31 */
        let v = new_ice_shader_view();
        assert!((v.ior - 1.31).abs() < 1e-4);
    }

    #[test]
    fn test_ice_absorption_at_zero() {
        /* at zero distance, transmission is 1 */
        let c = ice_absorption_color(0.0, [0.01, 0.005, 0.001]);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ice_absorption_decreases() {
        /* more distance means less transmission */
        let c0 = ice_absorption_color(0.1, [1.0, 1.0, 1.0]);
        let c1 = ice_absorption_color(1.0, [1.0, 1.0, 1.0]);
        assert!(c1[0] < c0[0]);
    }

    #[test]
    fn test_ice_fresnel_at_normal() {
        /* at normal incidence (cos=1), fresnel = f0 */
        let f = ice_fresnel(1.0, 1.31);
        let f0 = ((1.31_f32 - 1.0) / (1.31_f32 + 1.0)).powi(2);
        assert!((f - f0).abs() < 1e-4);
    }

    #[test]
    fn test_ice_caustic_strength() {
        /* thicker ice has stronger caustics */
        let s0 = ice_caustic_strength(0.1);
        let s1 = ice_caustic_strength(1.0);
        assert!(s1 > s0);
    }
}
