// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SheenView {
    pub show_color: bool,
    pub show_roughness: bool,
}

pub fn new_sheen_view() -> SheenView {
    SheenView {
        show_color: true,
        show_roughness: false,
    }
}

pub fn sheen_color_debug(color: [f32; 3]) -> [f32; 3] {
    color
}

pub fn sheen_roughness_color(roughness: f32) -> [f32; 3] {
    let r = roughness.clamp(0.0, 1.0);
    [r, r * 0.5, 1.0 - r]
}

pub fn sheen_directional_albedo(roughness: f32, cos_theta: f32) -> f32 {
    /* stub: 1 - roughness * cos_theta */
    (1.0 - roughness.clamp(0.0, 1.0) * cos_theta.clamp(0.0, 1.0)).clamp(0.0, 1.0)
}

pub fn sheen_is_visible(color: [f32; 3]) -> bool {
    color[0] > 0.01 || color[1] > 0.01 || color[2] > 0.01
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_sheen_view() {
        /* show_color defaults to true */
        let v = new_sheen_view();
        assert!(v.show_color);
    }

    #[test]
    fn test_sheen_color_debug_passthrough() {
        /* color passthrough */
        let c = sheen_color_debug([0.5, 0.3, 0.8]);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_sheen_roughness_color() {
        /* roughness=0 -> [0,0,1] */
        let c = sheen_roughness_color(0.0);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sheen_directional_albedo() {
        /* zero roughness gives albedo=1 */
        let a = sheen_directional_albedo(0.0, 0.5);
        assert!((a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sheen_is_visible() {
        /* non-zero color is visible */
        assert!(sheen_is_visible([0.5, 0.0, 0.0]));
        assert!(!sheen_is_visible([0.0, 0.0, 0.0]));
    }
}
