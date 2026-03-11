// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! G-buffer visualization for deferred rendering debug views.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum GBufferChannel {
    Albedo,
    Normal,
    Depth,
    Metallic,
    Roughness,
    Emission,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GBufferViewConfig {
    pub channel: GBufferChannel,
    pub depth_near: f32,
    pub depth_far: f32,
    pub normal_remap: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GBufferPixel {
    pub albedo: [f32; 3],
    pub normal: [f32; 3],
    pub depth: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub emission: [f32; 3],
}

#[allow(dead_code)]
pub fn default_gbuffer_view_config() -> GBufferViewConfig {
    GBufferViewConfig {
        channel: GBufferChannel::Albedo,
        depth_near: 0.1,
        depth_far: 100.0,
        normal_remap: true,
    }
}

#[allow(dead_code)]
pub fn new_gbuffer_pixel() -> GBufferPixel {
    GBufferPixel {
        albedo: [0.5, 0.5, 0.5],
        normal: [0.0, 1.0, 0.0],
        depth: 1.0,
        metallic: 0.0,
        roughness: 0.5,
        emission: [0.0; 3],
    }
}

#[allow(dead_code)]
pub fn visualize_channel(pixel: &GBufferPixel, cfg: &GBufferViewConfig) -> [f32; 4] {
    match cfg.channel {
        GBufferChannel::Albedo => [pixel.albedo[0], pixel.albedo[1], pixel.albedo[2], 1.0],
        GBufferChannel::Normal => {
            if cfg.normal_remap {
                [
                    pixel.normal[0] * 0.5 + 0.5,
                    pixel.normal[1] * 0.5 + 0.5,
                    pixel.normal[2] * 0.5 + 0.5,
                    1.0,
                ]
            } else {
                [pixel.normal[0], pixel.normal[1], pixel.normal[2], 1.0]
            }
        }
        GBufferChannel::Depth => {
            let range = cfg.depth_far - cfg.depth_near;
            let d = if range > 0.0 {
                ((pixel.depth - cfg.depth_near) / range).clamp(0.0, 1.0)
            } else {
                0.0
            };
            [d, d, d, 1.0]
        }
        GBufferChannel::Metallic => [pixel.metallic, pixel.metallic, pixel.metallic, 1.0],
        GBufferChannel::Roughness => [pixel.roughness, pixel.roughness, pixel.roughness, 1.0],
        GBufferChannel::Emission => [pixel.emission[0], pixel.emission[1], pixel.emission[2], 1.0],
    }
}

#[allow(dead_code)]
pub fn linearize_depth(raw_depth: f32, near: f32, far: f32) -> f32 {
    if far <= near {
        return 0.0;
    }
    let ndc = raw_depth * 2.0 - 1.0;
    let denom = far + near - ndc * (far - near);
    if denom.abs() < 1e-8 {
        0.0
    } else {
        (2.0 * near * far) / denom
    }
}

#[allow(dead_code)]
pub fn gbuffer_view_to_json(cfg: &GBufferViewConfig) -> String {
    let ch = match &cfg.channel {
        GBufferChannel::Albedo => "albedo",
        GBufferChannel::Normal => "normal",
        GBufferChannel::Depth => "depth",
        GBufferChannel::Metallic => "metallic",
        GBufferChannel::Roughness => "roughness",
        GBufferChannel::Emission => "emission",
    };
    format!(
        r#"{{"channel":"{}","near":{},"far":{},"remap":{}}}"#,
        ch, cfg.depth_near, cfg.depth_far, cfg.normal_remap
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_gbuffer_view_config();
        assert_eq!(c.channel, GBufferChannel::Albedo);
    }

    #[test]
    fn test_new_pixel() {
        let p = new_gbuffer_pixel();
        assert!((p.depth - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vis_albedo() {
        let p = new_gbuffer_pixel();
        let cfg = default_gbuffer_view_config();
        let c = visualize_channel(&p, &cfg);
        assert!((c[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_vis_normal_remap() {
        let p = new_gbuffer_pixel();
        let mut cfg = default_gbuffer_view_config();
        cfg.channel = GBufferChannel::Normal;
        let c = visualize_channel(&p, &cfg);
        assert!((c[1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_vis_depth() {
        let mut p = new_gbuffer_pixel();
        p.depth = 50.0;
        let mut cfg = default_gbuffer_view_config();
        cfg.channel = GBufferChannel::Depth;
        let c = visualize_channel(&p, &cfg);
        assert!((0.0..=1.0).contains(&c[0]));
    }

    #[test]
    fn test_vis_metallic() {
        let mut p = new_gbuffer_pixel();
        p.metallic = 0.8;
        let mut cfg = default_gbuffer_view_config();
        cfg.channel = GBufferChannel::Metallic;
        let c = visualize_channel(&p, &cfg);
        assert!((c[0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_linearize_depth() {
        let d = linearize_depth(0.5, 0.1, 100.0);
        assert!(d > 0.0);
    }

    #[test]
    fn test_linearize_depth_invalid() {
        let d = linearize_depth(0.5, 10.0, 5.0);
        assert!(d.abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_gbuffer_view_config();
        let j = gbuffer_view_to_json(&cfg);
        assert!(j.contains("albedo"));
    }

    #[test]
    fn test_vis_roughness() {
        let mut p = new_gbuffer_pixel();
        p.roughness = 0.3;
        let mut cfg = default_gbuffer_view_config();
        cfg.channel = GBufferChannel::Roughness;
        let c = visualize_channel(&p, &cfg);
        assert!((c[0] - 0.3).abs() < 1e-6);
    }
}
