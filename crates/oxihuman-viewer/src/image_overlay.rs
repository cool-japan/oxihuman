// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Image overlay compositing for reference images, watermarks, and backgrounds.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum OverlayBlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Add,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImageOverlayConfig {
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
    pub opacity: f32,
    pub blend_mode: OverlayBlendMode,
    pub visible: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImageOverlayLayer {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub config: ImageOverlayConfig,
}

#[allow(dead_code)]
pub fn default_image_overlay_config() -> ImageOverlayConfig {
    ImageOverlayConfig {
        offset_x: 0.0,
        offset_y: 0.0,
        scale: 1.0,
        opacity: 1.0,
        blend_mode: OverlayBlendMode::Normal,
        visible: true,
    }
}

#[allow(dead_code)]
pub fn new_image_overlay_layer(name: &str, w: u32, h: u32) -> ImageOverlayLayer {
    ImageOverlayLayer {
        name: name.to_string(),
        width: w,
        height: h,
        config: default_image_overlay_config(),
    }
}

#[allow(dead_code)]
pub fn blend_pixel(base: [f32; 4], over: [f32; 4], mode: &OverlayBlendMode, opacity: f32) -> [f32; 4] {
    let a = opacity * over[3];
    let blended = match mode {
        OverlayBlendMode::Normal => [over[0], over[1], over[2]],
        OverlayBlendMode::Multiply => [base[0] * over[0], base[1] * over[1], base[2] * over[2]],
        OverlayBlendMode::Screen => [
            1.0 - (1.0 - base[0]) * (1.0 - over[0]),
            1.0 - (1.0 - base[1]) * (1.0 - over[1]),
            1.0 - (1.0 - base[2]) * (1.0 - over[2]),
        ],
        OverlayBlendMode::Overlay => {
            let ov = |b: f32, o: f32| -> f32 {
                if b < 0.5 { 2.0 * b * o } else { 1.0 - 2.0 * (1.0 - b) * (1.0 - o) }
            };
            [ov(base[0], over[0]), ov(base[1], over[1]), ov(base[2], over[2])]
        }
        OverlayBlendMode::Add => [
            (base[0] + over[0]).min(1.0),
            (base[1] + over[1]).min(1.0),
            (base[2] + over[2]).min(1.0),
        ],
    };
    let inv = 1.0 - a;
    [
        base[0] * inv + blended[0] * a,
        base[1] * inv + blended[1] * a,
        base[2] * inv + blended[2] * a,
        base[3],
    ]
}

#[allow(dead_code)]
pub fn set_overlay_opacity(layer: &mut ImageOverlayLayer, opacity: f32) {
    layer.config.opacity = opacity.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn set_overlay_offset(layer: &mut ImageOverlayLayer, x: f32, y: f32) {
    layer.config.offset_x = x;
    layer.config.offset_y = y;
}

#[allow(dead_code)]
pub fn set_overlay_scale(layer: &mut ImageOverlayLayer, scale: f32) {
    layer.config.scale = scale.clamp(0.01, 10.0);
}

#[allow(dead_code)]
pub fn overlay_to_json(layer: &ImageOverlayLayer) -> String {
    let mode_str = match &layer.config.blend_mode {
        OverlayBlendMode::Normal => "normal",
        OverlayBlendMode::Multiply => "multiply",
        OverlayBlendMode::Screen => "screen",
        OverlayBlendMode::Overlay => "overlay",
        OverlayBlendMode::Add => "add",
    };
    format!(
        r#"{{"name":"{}","w":{},"h":{},"opacity":{},"mode":"{}"}}"#,
        layer.name, layer.width, layer.height, layer.config.opacity, mode_str
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_image_overlay_config();
        assert!(c.visible);
        assert!((c.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_layer() {
        let l = new_image_overlay_layer("ref", 1920, 1080);
        assert_eq!(l.width, 1920);
    }

    #[test]
    fn test_blend_normal() {
        let r = blend_pixel([0.5; 4], [1.0; 4], &OverlayBlendMode::Normal, 1.0);
        assert!((r[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_blend_multiply() {
        let r = blend_pixel([0.5, 0.5, 0.5, 1.0], [0.5, 0.5, 0.5, 1.0], &OverlayBlendMode::Multiply, 1.0);
        assert!((r[0] - 0.25).abs() < 1e-4);
    }

    #[test]
    fn test_blend_screen() {
        let r = blend_pixel([0.5, 0.5, 0.5, 1.0], [0.5, 0.5, 0.5, 1.0], &OverlayBlendMode::Screen, 1.0);
        assert!((r[0] - 0.75).abs() < 1e-4);
    }

    #[test]
    fn test_blend_add() {
        let r = blend_pixel([0.7, 0.7, 0.7, 1.0], [0.5, 0.5, 0.5, 1.0], &OverlayBlendMode::Add, 1.0);
        assert!((r[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_set_opacity() {
        let mut l = new_image_overlay_layer("t", 10, 10);
        set_overlay_opacity(&mut l, 0.5);
        assert!((l.config.opacity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_offset() {
        let mut l = new_image_overlay_layer("t", 10, 10);
        set_overlay_offset(&mut l, 5.0, 3.0);
        assert!((l.config.offset_x - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let l = new_image_overlay_layer("test", 100, 100);
        let j = overlay_to_json(&l);
        assert!(j.contains("test"));
    }

    #[test]
    fn test_set_scale_clamp() {
        let mut l = new_image_overlay_layer("t", 10, 10);
        set_overlay_scale(&mut l, 0.001);
        assert!((l.config.scale - 0.01).abs() < 1e-6);
    }
}
