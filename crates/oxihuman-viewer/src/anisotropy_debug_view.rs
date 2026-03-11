// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct AnisotropyDebugView {
    pub show_tangents: bool,
    pub show_bitangents: bool,
    pub scale: f32,
    pub roughness_threshold: f32,
}

pub fn new_anisotropy_debug_view() -> AnisotropyDebugView {
    AnisotropyDebugView {
        show_tangents: true,
        show_bitangents: false,
        scale: 1.0,
        roughness_threshold: 0.5,
    }
}

pub fn aniso_tangent_color(anisotropy_strength: f32) -> [f32; 3] {
    let s = anisotropy_strength.clamp(0.0, 1.0);
    [s, 1.0 - s, 0.0]
}

pub fn aniso_should_show(v: &AnisotropyDebugView, roughness: f32) -> bool {
    roughness <= v.roughness_threshold
}

pub fn aniso_debug_line(pos: [f32; 3], tangent: [f32; 3], scale: f32) -> ([f32; 3], [f32; 3]) {
    let end = [
        pos[0] + tangent[0] * scale,
        pos[1] + tangent[1] * scale,
        pos[2] + tangent[2] * scale,
    ];
    (pos, end)
}

pub fn aniso_rotation_from_map(angle_deg: f32) -> [f32; 2] {
    let r = angle_deg.to_radians();
    [r.cos(), r.sin()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_anisotropy_debug_view() {
        /* scale defaults to 1 */
        let v = new_anisotropy_debug_view();
        assert!((v.scale - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_aniso_tangent_color_zero() {
        /* zero strength gives [0,1,0] */
        let c = aniso_tangent_color(0.0);
        assert!((c[0] - 0.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_aniso_should_show_true() {
        /* low roughness passes threshold */
        let v = new_anisotropy_debug_view();
        assert!(aniso_should_show(&v, 0.2));
    }

    #[test]
    fn test_aniso_debug_line() {
        /* end point offset by tangent * scale */
        let (_, end) = aniso_debug_line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 2.0);
        assert!((end[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_aniso_rotation_from_map() {
        /* 0 deg gives [1,0] */
        let r = aniso_rotation_from_map(0.0);
        assert!((r[0] - 1.0).abs() < 1e-6);
        assert!((r[1] - 0.0).abs() < 1e-6);
    }
}
