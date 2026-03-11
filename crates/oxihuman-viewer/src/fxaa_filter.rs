// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Fast Approximate Anti-Aliasing (FXAA) filter implementation.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum FxaaQuality {
    Low,
    Medium,
    High,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FxaaConfig {
    pub quality: FxaaQuality,
    pub edge_threshold: f32,
    pub edge_threshold_min: f32,
    pub subpixel_quality: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FxaaBuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<[f32; 4]>,
}

#[allow(dead_code)]
pub fn default_fxaa_config() -> FxaaConfig {
    FxaaConfig {
        quality: FxaaQuality::Medium,
        edge_threshold: 0.166,
        edge_threshold_min: 0.0833,
        subpixel_quality: 0.75,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn new_fxaa_buffer(w: u32, h: u32) -> FxaaBuffer {
    FxaaBuffer {
        width: w,
        height: h,
        pixels: vec![[0.0, 0.0, 0.0, 1.0]; (w as usize) * (h as usize)],
    }
}

#[allow(dead_code)]
pub fn pixel_luminance_fxaa(px: [f32; 4]) -> f32 {
    0.299 * px[0] + 0.587 * px[1] + 0.114 * px[2]
}

#[allow(dead_code)]
pub fn is_edge_pixel(lum_center: f32, lum_n: f32, lum_s: f32, lum_e: f32, lum_w: f32, threshold: f32) -> bool {
    let range = lum_n.max(lum_s).max(lum_e).max(lum_w).max(lum_center)
        - lum_n.min(lum_s).min(lum_e).min(lum_w).min(lum_center);
    range > threshold
}

#[allow(dead_code)]
pub fn sample_pixel(buf: &FxaaBuffer, x: i32, y: i32) -> [f32; 4] {
    let cx = x.clamp(0, buf.width as i32 - 1) as usize;
    let cy = y.clamp(0, buf.height as i32 - 1) as usize;
    buf.pixels[cy * buf.width as usize + cx]
}

#[allow(dead_code)]
pub fn apply_fxaa_pixel(buf: &FxaaBuffer, x: u32, y: u32, cfg: &FxaaConfig) -> [f32; 4] {
    if !cfg.enabled {
        return sample_pixel(buf, x as i32, y as i32);
    }
    let c = sample_pixel(buf, x as i32, y as i32);
    let n = sample_pixel(buf, x as i32, y as i32 - 1);
    let s = sample_pixel(buf, x as i32, y as i32 + 1);
    let e = sample_pixel(buf, x as i32 + 1, y as i32);
    let w = sample_pixel(buf, x as i32 - 1, y as i32);

    let lc = pixel_luminance_fxaa(c);
    let ln = pixel_luminance_fxaa(n);
    let ls = pixel_luminance_fxaa(s);
    let le = pixel_luminance_fxaa(e);
    let lw = pixel_luminance_fxaa(w);

    if !is_edge_pixel(lc, ln, ls, le, lw, cfg.edge_threshold) {
        return c;
    }

    let blend = cfg.subpixel_quality;
    let inv = 1.0 - blend;
    [
        c[0] * inv + (n[0] + s[0] + e[0] + w[0]) * 0.25 * blend,
        c[1] * inv + (n[1] + s[1] + e[1] + w[1]) * 0.25 * blend,
        c[2] * inv + (n[2] + s[2] + e[2] + w[2]) * 0.25 * blend,
        c[3],
    ]
}

#[allow(dead_code)]
pub fn fxaa_config_to_json(cfg: &FxaaConfig) -> String {
    let q = match &cfg.quality {
        FxaaQuality::Low => "low",
        FxaaQuality::Medium => "medium",
        FxaaQuality::High => "high",
    };
    format!(
        r#"{{"quality":"{}","threshold":{},"enabled":{}}}"#,
        q, cfg.edge_threshold, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = default_fxaa_config();
        assert_eq!(c.quality, FxaaQuality::Medium);
        assert!(c.enabled);
    }

    #[test]
    fn test_new_buffer() {
        let b = new_fxaa_buffer(4, 4);
        assert_eq!(b.pixels.len(), 16);
    }

    #[test]
    fn test_luminance() {
        let lum = pixel_luminance_fxaa([1.0, 1.0, 1.0, 1.0]);
        assert!((lum - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_luminance_black() {
        let lum = pixel_luminance_fxaa([0.0, 0.0, 0.0, 1.0]);
        assert!(lum.abs() < 1e-6);
    }

    #[test]
    fn test_is_edge() {
        assert!(is_edge_pixel(1.0, 0.0, 0.0, 0.0, 0.0, 0.5));
    }

    #[test]
    fn test_not_edge() {
        assert!(!is_edge_pixel(0.5, 0.5, 0.5, 0.5, 0.5, 0.1));
    }

    #[test]
    fn test_sample_clamp() {
        let b = new_fxaa_buffer(2, 2);
        let px = sample_pixel(&b, -1, -1);
        assert!((px[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fxaa_disabled() {
        let b = new_fxaa_buffer(4, 4);
        let mut cfg = default_fxaa_config();
        cfg.enabled = false;
        let px = apply_fxaa_pixel(&b, 1, 1, &cfg);
        assert!((px[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let c = default_fxaa_config();
        let j = fxaa_config_to_json(&c);
        assert!(j.contains("medium"));
    }

    #[test]
    fn test_fxaa_uniform_no_change() {
        let b = new_fxaa_buffer(4, 4);
        let cfg = default_fxaa_config();
        let px = apply_fxaa_pixel(&b, 1, 1, &cfg);
        assert!(px[0].abs() < 1e-6);
    }
}
