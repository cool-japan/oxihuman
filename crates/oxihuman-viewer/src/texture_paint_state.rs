#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// Texture paint mode state.
#[derive(Debug, Clone)]
pub struct TexturePaintState {
    pub active_texture: String,
    pub radius: f32,
    pub strength: f32,
    pub color: [f32; 4],
    pub blend_mode: u8,
    pub uv_layer: String,
}

#[allow(dead_code)]
pub fn new_texture_paint_state() -> TexturePaintState {
    TexturePaintState {
        active_texture: "Texture".to_string(),
        radius: 0.1,
        strength: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        blend_mode: 0,
        uv_layer: "UVMap".to_string(),
    }
}

#[allow(dead_code)]
pub fn tp_set_active_texture(state: &mut TexturePaintState, name: &str) {
    state.active_texture = name.to_string();
}

#[allow(dead_code)]
pub fn tp_set_color(state: &mut TexturePaintState, c: [f32; 4]) {
    state.color = c;
}

#[allow(dead_code)]
pub fn tp_set_radius(state: &mut TexturePaintState, r: f32) {
    state.radius = r.max(0.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let s = new_texture_paint_state();
        assert_eq!(s.active_texture, "Texture");
        assert_eq!(s.uv_layer, "UVMap");
    }

    #[test]
    fn test_set_active_texture() {
        let mut s = new_texture_paint_state();
        tp_set_active_texture(&mut s, "MyTex");
        assert_eq!(s.active_texture, "MyTex");
    }

    #[test]
    fn test_set_color() {
        let mut s = new_texture_paint_state();
        tp_set_color(&mut s, [0.2, 0.4, 0.6, 1.0]);
        assert!((s.color[0] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_set_radius() {
        let mut s = new_texture_paint_state();
        tp_set_radius(&mut s, 0.5);
        assert!((s.radius - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_radius_clamps_negative() {
        let mut s = new_texture_paint_state();
        tp_set_radius(&mut s, -2.0);
        assert!((s.radius).abs() < 1e-6);
    }

    #[test]
    fn test_strength_default() {
        let s = new_texture_paint_state();
        assert!((s.strength - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_blend_mode_default() {
        let s = new_texture_paint_state();
        assert_eq!(s.blend_mode, 0);
    }

    #[test]
    fn test_color_default_white() {
        let s = new_texture_paint_state();
        assert_eq!(s.color, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_set_color_alpha() {
        let mut s = new_texture_paint_state();
        tp_set_color(&mut s, [1.0, 0.0, 0.0, 0.5]);
        assert!((s.color[3] - 0.5).abs() < 1e-6);
    }
}
