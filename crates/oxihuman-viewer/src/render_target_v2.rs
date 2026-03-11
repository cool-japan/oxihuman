// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Render target v2 — multi-attachment render target descriptor.

use std::f32::consts::FRAC_PI_4;

pub const MAX_COLOR_ATTACHMENTS: usize = 8;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtFormatV2 {
    Rgba8,
    Rgba16F,
    R32F,
    Depth32F,
}

impl RtFormatV2 {
    #[allow(dead_code)]
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            RtFormatV2::Rgba8 => 4,
            RtFormatV2::Rgba16F => 8,
            RtFormatV2::R32F => 4,
            RtFormatV2::Depth32F => 4,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderTargetV2Config {
    pub width: u32,
    pub height: u32,
    pub color_formats: [Option<RtFormatV2>; MAX_COLOR_ATTACHMENTS],
    pub depth_format: Option<RtFormatV2>,
    pub msaa_samples: u32,
}

impl Default for RenderTargetV2Config {
    fn default() -> Self {
        let mut color_formats = [None; MAX_COLOR_ATTACHMENTS];
        color_formats[0] = Some(RtFormatV2::Rgba8);
        Self {
            width: 1280,
            height: 720,
            color_formats,
            depth_format: Some(RtFormatV2::Depth32F),
            msaa_samples: 1,
        }
    }
}

#[allow(dead_code)]
pub fn default_render_target_v2_config() -> RenderTargetV2Config {
    RenderTargetV2Config::default()
}

#[allow(dead_code)]
pub fn rtv2_attachment_count(cfg: &RenderTargetV2Config) -> usize {
    cfg.color_formats.iter().filter(|f| f.is_some()).count()
}

#[allow(dead_code)]
pub fn rtv2_memory_bytes(cfg: &RenderTargetV2Config) -> usize {
    let px = cfg.width as usize * cfg.height as usize;
    let color: usize = cfg
        .color_formats
        .iter()
        .flatten()
        .map(|f| f.bytes_per_pixel() * px * cfg.msaa_samples as usize)
        .sum();
    let depth: usize = cfg
        .depth_format
        .map_or(0, |f| f.bytes_per_pixel() * px * cfg.msaa_samples as usize);
    color + depth
}

#[allow(dead_code)]
pub fn rtv2_set_size(cfg: &mut RenderTargetV2Config, w: u32, h: u32) {
    cfg.width = w.max(1);
    cfg.height = h.max(1);
}

#[allow(dead_code)]
pub fn rtv2_aspect_ratio(cfg: &RenderTargetV2Config) -> f32 {
    cfg.width as f32 / cfg.height as f32
}

#[allow(dead_code)]
pub fn rtv2_fov_angle_rad(cfg: &RenderTargetV2Config) -> f32 {
    rtv2_aspect_ratio(cfg).atan().min(FRAC_PI_4)
}

#[allow(dead_code)]
pub fn rtv2_is_valid(cfg: &RenderTargetV2Config) -> bool {
    cfg.width > 0 && cfg.height > 0 && cfg.msaa_samples >= 1
}

#[allow(dead_code)]
pub fn rtv2_to_json(cfg: &RenderTargetV2Config) -> String {
    format!(
        "{{\"width\":{},\"height\":{},\"attachments\":{}}}",
        cfg.width,
        cfg.height,
        rtv2_attachment_count(cfg)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_valid() {
        assert!(rtv2_is_valid(&default_render_target_v2_config()));
    }
    #[test]
    fn default_attachment_count_one() {
        assert_eq!(rtv2_attachment_count(&default_render_target_v2_config()), 1);
    }
    #[test]
    fn set_size_applies() {
        let mut c = default_render_target_v2_config();
        rtv2_set_size(&mut c, 800, 600);
        assert_eq!(c.width, 800);
    }
    #[test]
    fn set_size_clamps_zero() {
        let mut c = default_render_target_v2_config();
        rtv2_set_size(&mut c, 0, 0);
        assert!(c.width >= 1 && c.height >= 1);
    }
    #[test]
    fn aspect_ratio_16_9() {
        let c = default_render_target_v2_config();
        assert!((rtv2_aspect_ratio(&c) - 1280.0 / 720.0).abs() < 1e-4);
    }
    #[test]
    fn fov_angle_nonneg() {
        assert!(rtv2_fov_angle_rad(&default_render_target_v2_config()) >= 0.0);
    }
    #[test]
    fn memory_bytes_positive() {
        assert!(rtv2_memory_bytes(&default_render_target_v2_config()) > 0);
    }
    #[test]
    fn bytes_per_pixel_rgba8() {
        assert_eq!(RtFormatV2::Rgba8.bytes_per_pixel(), 4);
    }
    #[test]
    fn bytes_per_pixel_rgba16f() {
        assert_eq!(RtFormatV2::Rgba16F.bytes_per_pixel(), 8);
    }
    #[test]
    fn to_json_has_width() {
        assert!(rtv2_to_json(&default_render_target_v2_config()).contains("\"width\""));
    }
}
