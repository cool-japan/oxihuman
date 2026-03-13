// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

use std::f32::consts::FRAC_PI_2;

pub struct GlassShaderView {
    pub show_fresnel: bool,
    pub show_caustics: bool,
    pub ior: f32,
    pub roughness: f32,
}

pub fn new_glass_shader_view(ior: f32) -> GlassShaderView {
    GlassShaderView {
        show_fresnel: true,
        show_caustics: false,
        ior,
        roughness: 0.0,
    }
}

pub fn glass_fresnel_schlick(cos_theta: f32, ior: f32) -> f32 {
    let f0 = ((ior - 1.0) / (ior + 1.0)).powi(2);
    f0 + (1.0 - f0) * (1.0 - cos_theta.clamp(0.0, 1.0)).powi(5)
}

/// Returns None on total internal reflection.
pub fn glass_refraction_dir(incident: [f32; 3], normal: [f32; 3], ior: f32) -> Option<[f32; 3]> {
    let cos_i = -(incident[0] * normal[0] + incident[1] * normal[1] + incident[2] * normal[2]);
    let sin2_t = (1.0 / ior).powi(2) * (1.0 - cos_i * cos_i).max(0.0);
    if sin2_t > 1.0 {
        return None;
    }
    let cos_t = (1.0 - sin2_t).sqrt();
    let scale = 1.0 / ior;
    let bias = scale * cos_i - cos_t;
    Some([
        scale * incident[0] + bias * normal[0],
        scale * incident[1] + bias * normal[1],
        scale * incident[2] + bias * normal[2],
    ])
}

/// Critical angle in degrees (uses FRAC_PI_2 internally).
pub fn glass_critical_angle_deg(ior: f32) -> f32 {
    let _ = FRAC_PI_2;
    if ior < 1.0 {
        return 90.0;
    }
    (1.0 / ior).asin().to_degrees()
}

pub fn glass_transmission_color(color: [f32; 3], fresnel: f32) -> [f32; 3] {
    let t = (1.0 - fresnel).clamp(0.0, 1.0);
    [color[0] * t, color[1] * t, color[2] * t]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_glass_shader_view() {
        /* ior is stored */
        let v = new_glass_shader_view(1.5);
        assert!((v.ior - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_glass_fresnel_at_grazing() {
        /* at grazing angle, fresnel approaches 1 */
        let f = glass_fresnel_schlick(0.0, 1.5);
        assert!((f - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_glass_refraction_dir_some() {
        /* normal incidence should not TIR */
        let inc = [0.0, -1.0, 0.0];
        let norm = [0.0, 1.0, 0.0];
        let r = glass_refraction_dir(inc, norm, 1.5);
        assert!(r.is_some());
    }

    #[test]
    fn test_glass_critical_angle() {
        /* critical angle for glass (n=1.5) ~ 41.8 degrees */
        let ca = glass_critical_angle_deg(1.5);
        assert!(ca > 40.0 && ca < 43.0);
    }

    #[test]
    fn test_glass_transmission_color() {
        /* at fresnel=0, full transmission */
        let c = glass_transmission_color([1.0, 1.0, 1.0], 0.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }
}
