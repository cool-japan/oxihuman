// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Occlusion map — ambient occlusion texture / buffer for GI approximation.

/// Ambient occlusion algorithm.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AoAlgorithm {
    Ssao,
    Hbao,
    Gtao,
    BakedTexture,
}

/// Occlusion map configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcclusionMapConfig {
    pub algorithm: AoAlgorithm,
    pub radius: f32,
    pub bias: f32,
    pub intensity: f32,
    pub sample_count: u32,
    pub enabled: bool,
}

/// Flat occlusion buffer (per-pixel values in [0, 1]).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OcclusionBuffer {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
}

#[allow(dead_code)]
pub fn default_occlusion_map_config() -> OcclusionMapConfig {
    OcclusionMapConfig {
        algorithm: AoAlgorithm::Ssao,
        radius: 0.5,
        bias: 0.025,
        intensity: 1.5,
        sample_count: 16,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn new_occlusion_buffer(width: usize, height: usize) -> OcclusionBuffer {
    OcclusionBuffer {
        width,
        height,
        data: vec![1.0; width * height],
    }
}

#[allow(dead_code)]
pub fn om_set_pixel(buf: &mut OcclusionBuffer, x: usize, y: usize, value: f32) {
    if x < buf.width && y < buf.height {
        let idx = y * buf.width + x;
        buf.data[idx] = value.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn om_get_pixel(buf: &OcclusionBuffer, x: usize, y: usize) -> f32 {
    if x < buf.width && y < buf.height {
        buf.data[y * buf.width + x]
    } else {
        1.0
    }
}

#[allow(dead_code)]
pub fn om_clear(buf: &mut OcclusionBuffer) {
    for v in buf.data.iter_mut() {
        *v = 1.0;
    }
}

#[allow(dead_code)]
pub fn om_average(buf: &OcclusionBuffer) -> f32 {
    if buf.data.is_empty() {
        return 1.0;
    }
    buf.data.iter().sum::<f32>() / buf.data.len() as f32
}

#[allow(dead_code)]
pub fn om_apply_intensity(cfg: &OcclusionMapConfig, raw_ao: f32) -> f32 {
    if !cfg.enabled {
        return 1.0;
    }
    raw_ao.powf(cfg.intensity).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn om_set_intensity(cfg: &mut OcclusionMapConfig, v: f32) {
    cfg.intensity = v.clamp(0.0, 5.0);
}

#[allow(dead_code)]
pub fn om_set_radius(cfg: &mut OcclusionMapConfig, v: f32) {
    cfg.radius = v.clamp(0.001, 10.0);
}

#[allow(dead_code)]
pub fn om_to_json(cfg: &OcclusionMapConfig) -> String {
    let algo = match cfg.algorithm {
        AoAlgorithm::Ssao => "ssao",
        AoAlgorithm::Hbao => "hbao",
        AoAlgorithm::Gtao => "gtao",
        AoAlgorithm::BakedTexture => "baked",
    };
    format!(
        r#"{{"algorithm":"{}","radius":{:.4},"intensity":{:.4},"enabled":{}}}"#,
        algo, cfg.radius, cfg.intensity, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_enabled() {
        let cfg = default_occlusion_map_config();
        assert!(cfg.enabled);
    }

    #[test]
    fn new_buffer_all_ones() {
        let buf = new_occlusion_buffer(4, 4);
        assert!(buf.data.iter().all(|&v| (v - 1.0).abs() < 1e-6));
    }

    #[test]
    fn set_and_get_pixel() {
        let mut buf = new_occlusion_buffer(4, 4);
        om_set_pixel(&mut buf, 1, 2, 0.5);
        assert!((om_get_pixel(&buf, 1, 2) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn get_out_of_bounds_returns_one() {
        let buf = new_occlusion_buffer(2, 2);
        assert!((om_get_pixel(&buf, 10, 10) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn clear_resets() {
        let mut buf = new_occlusion_buffer(2, 2);
        om_set_pixel(&mut buf, 0, 0, 0.0);
        om_clear(&mut buf);
        assert!((om_average(&buf) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn average_computed() {
        let mut buf = new_occlusion_buffer(2, 1);
        om_set_pixel(&mut buf, 0, 0, 0.0);
        om_set_pixel(&mut buf, 1, 0, 1.0);
        assert!((om_average(&buf) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn apply_intensity_disabled() {
        let mut cfg = default_occlusion_map_config();
        cfg.enabled = false;
        assert!((om_apply_intensity(&cfg, 0.3) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_intensity_clamps() {
        let mut cfg = default_occlusion_map_config();
        om_set_intensity(&mut cfg, 100.0);
        assert!((cfg.intensity - 5.0).abs() < 1e-6);
    }

    #[test]
    fn set_radius_clamps() {
        let mut cfg = default_occlusion_map_config();
        om_set_radius(&mut cfg, 0.0);
        assert!(cfg.radius > 0.0);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_occlusion_map_config();
        let j = om_to_json(&cfg);
        assert!(j.contains("algorithm"));
        assert!(j.contains("intensity"));
    }
}
