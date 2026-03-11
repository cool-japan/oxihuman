// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct TransmissionView {
    pub show_factor: bool,
    pub show_ior: bool,
    pub show_thickness: bool,
}

pub fn new_transmission_view() -> TransmissionView {
    TransmissionView {
        show_factor: true,
        show_ior: false,
        show_thickness: false,
    }
}

pub fn transmission_factor_color(factor: f32) -> [f32; 3] {
    let f = factor.clamp(0.0, 1.0);
    [0.0, f, 1.0 - f * 0.5]
}

pub fn transmission_ior_color(ior: f32) -> [f32; 3] {
    /* map 1.0-3.0 to hue via simple ramp */
    let t = ((ior - 1.0) / 2.0).clamp(0.0, 1.0);
    [t, 1.0 - t, (1.0 - t) * 0.5]
}

pub fn transmission_fresnel_at_normal(ior: f32) -> f32 {
    /* Schlick at cos_theta=1 (normal incidence) */
    ((1.0 - ior) / (1.0 + ior)).powi(2)
}

pub fn transmission_is_refractive(factor: f32) -> bool {
    factor > 0.01
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_transmission_view() {
        /* show_factor defaults to true */
        let v = new_transmission_view();
        assert!(v.show_factor);
    }

    #[test]
    fn test_transmission_factor_color_zero() {
        /* factor=0 -> [0,0,1.0] */
        let c = transmission_factor_color(0.0);
        assert!((c[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_transmission_ior_color_air() {
        /* ior=1.0 maps to t=0 */
        let c = transmission_ior_color(1.0);
        assert!((c[0] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_transmission_fresnel_at_normal() {
        /* glass ior=1.5 r0 ~ 0.04 */
        let f = transmission_fresnel_at_normal(1.5);
        assert!(f > 0.0 && f < 0.1);
    }

    #[test]
    fn test_transmission_is_refractive() {
        assert!(transmission_is_refractive(0.5));
        assert!(!transmission_is_refractive(0.0));
    }
}
