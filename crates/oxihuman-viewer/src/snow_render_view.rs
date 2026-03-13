// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub struct SnowRenderView {
    pub show_ao: bool,
    pub show_depth: bool,
    pub grain_size_mm: f32,
    pub density: f32,
}

pub fn new_snow_render_view() -> SnowRenderView {
    SnowRenderView {
        show_ao: true,
        show_depth: false,
        grain_size_mm: 0.5,
        density: 100.0,
    }
}

/// Blue-tinted subsurface scattering color for snow.
pub fn snow_sss_color(depth_cm: f32) -> [f32; 3] {
    let t = (depth_cm / 10.0).clamp(0.0, 1.0);
    [1.0 - t * 0.5, 1.0 - t * 0.2, 1.0]
}

pub fn snow_sparkle_intensity(cos_theta: f32, grain_size: f32) -> f32 {
    let base = cos_theta.max(0.0);
    (base * grain_size * 10.0).clamp(0.0, 1.0)
}

pub fn snow_ao_color(ao: f32) -> [f32; 3] {
    let v = ao.clamp(0.0, 1.0);
    [v, v, v]
}

/// Fresh snow has density < 150.0 kg/m³.
pub fn snow_is_fresh(density: f32) -> bool {
    density < 150.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_snow_render_view() {
        /* grain size defaults to 0.5mm */
        let v = new_snow_render_view();
        assert!((v.grain_size_mm - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_snow_sss_color_surface() {
        /* at depth 0, color is nearly white */
        let c = snow_sss_color(0.0);
        assert!((c[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_snow_is_fresh_true() {
        /* density 100 is fresh */
        assert!(snow_is_fresh(100.0));
    }

    #[test]
    fn test_snow_is_fresh_false() {
        /* density 300 is not fresh */
        assert!(!snow_is_fresh(300.0));
    }

    #[test]
    fn test_snow_ao_color_gray() {
        /* AO color is gray-scale */
        let c = snow_ao_color(0.5);
        assert!((c[0] - c[1]).abs() < 1e-6);
        assert!((c[1] - c[2]).abs() < 1e-6);
    }
}
