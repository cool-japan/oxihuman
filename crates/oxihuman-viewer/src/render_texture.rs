// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Render-to-texture target configuration.

#![allow(dead_code)]

/// Pixel format for a render texture.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum RenderTextureFormat {
    Rgba8,
    Rgba16F,
    Depth32F,
}

/// Configuration for a render texture.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderTextureConfig {
    pub width: u32,
    pub height: u32,
    pub format: RenderTextureFormat,
    pub mip_levels: u32,
}

/// Runtime state for a render texture.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderTextureState {
    pub config: RenderTextureConfig,
    pub id: u32,
    pub active: bool,
}

#[allow(dead_code)]
pub fn default_render_texture_config() -> RenderTextureConfig {
    RenderTextureConfig {
        width: 512,
        height: 512,
        format: RenderTextureFormat::Rgba8,
        mip_levels: 1,
    }
}

#[allow(dead_code)]
pub fn new_render_texture_state(id: u32) -> RenderTextureState {
    RenderTextureState {
        config: default_render_texture_config(),
        id,
        active: false,
    }
}

#[allow(dead_code)]
pub fn rt_set_active(state: &mut RenderTextureState, active: bool) {
    state.active = active;
}

#[allow(dead_code)]
pub fn rt_is_active(state: &RenderTextureState) -> bool {
    state.active
}

#[allow(dead_code)]
pub fn rt_pixel_count(state: &RenderTextureState) -> u64 {
    state.config.width as u64 * state.config.height as u64
}

#[allow(dead_code)]
pub fn rt_format_name(state: &RenderTextureState) -> &'static str {
    match state.config.format {
        RenderTextureFormat::Rgba8 => "RGBA8",
        RenderTextureFormat::Rgba16F => "RGBA16F",
        RenderTextureFormat::Depth32F => "Depth32F",
    }
}

#[allow(dead_code)]
pub fn rt_to_json(state: &RenderTextureState) -> String {
    format!(
        r#"{{"id":{},"width":{},"height":{},"format":"{}","active":{}}}"#,
        state.id,
        state.config.width,
        state.config.height,
        rt_format_name(state),
        state.active
    )
}

#[allow(dead_code)]
pub fn rt_byte_size(state: &RenderTextureState) -> u64 {
    let bytes_per_pixel: u64 = match state.config.format {
        RenderTextureFormat::Rgba8 => 4,
        RenderTextureFormat::Rgba16F => 8,
        RenderTextureFormat::Depth32F => 4,
    };
    rt_pixel_count(state) * bytes_per_pixel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_render_texture_config();
        assert_eq!(cfg.width, 512);
        assert_eq!(cfg.height, 512);
        assert_eq!(cfg.format, RenderTextureFormat::Rgba8);
        assert_eq!(cfg.mip_levels, 1);
    }

    #[test]
    fn test_new_state_inactive() {
        let s = new_render_texture_state(42);
        assert_eq!(s.id, 42);
        assert!(!rt_is_active(&s));
    }

    #[test]
    fn test_set_active() {
        let mut s = new_render_texture_state(1);
        rt_set_active(&mut s, true);
        assert!(rt_is_active(&s));
    }

    #[test]
    fn test_pixel_count() {
        let s = new_render_texture_state(1);
        assert_eq!(rt_pixel_count(&s), 512 * 512);
    }

    #[test]
    fn test_format_name_rgba8() {
        let s = new_render_texture_state(1);
        assert_eq!(rt_format_name(&s), "RGBA8");
    }

    #[test]
    fn test_byte_size_rgba8() {
        let s = new_render_texture_state(1);
        assert_eq!(rt_byte_size(&s), 512 * 512 * 4);
    }

    #[test]
    fn test_format_name_rgba16f() {
        let mut s = new_render_texture_state(1);
        s.config.format = RenderTextureFormat::Rgba16F;
        assert_eq!(rt_format_name(&s), "RGBA16F");
        assert_eq!(rt_byte_size(&s), 512 * 512 * 8);
    }

    #[test]
    fn test_to_json_contains_fields() {
        let s = new_render_texture_state(7);
        let j = rt_to_json(&s);
        assert!(j.contains("id"));
        assert!(j.contains("width"));
        assert!(j.contains("format"));
    }
}
