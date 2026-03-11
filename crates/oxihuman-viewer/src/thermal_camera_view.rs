// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct ThermalCameraConfig {
    pub width: u32,
    pub height: u32,
    pub min_temp_c: f32,
    pub max_temp_c: f32,
    pub emissivity: f32,
}

pub fn new_thermal_camera_config(w: u32, h: u32) -> ThermalCameraConfig {
    ThermalCameraConfig {
        width: w,
        height: h,
        min_temp_c: 0.0,
        max_temp_c: 100.0,
        emissivity: 0.95,
    }
}

pub fn thermal_pixel_to_temp(cfg: &ThermalCameraConfig, raw: u16) -> f32 {
    let t = raw as f32 / u16::MAX as f32;
    cfg.min_temp_c + t * (cfg.max_temp_c - cfg.min_temp_c)
}

pub fn thermal_temp_to_color(temp_c: f32, min: f32, max: f32) -> [u8; 3] {
    let t = ((temp_c - min) / (max - min)).clamp(0.0, 1.0);
    let r = (t * 255.0) as u8;
    let b = ((1.0 - t) * 255.0) as u8;
    [r, 0, b]
}

pub fn thermal_mean_temp(cfg: &ThermalCameraConfig, pixels: &[u16]) -> f32 {
    if pixels.is_empty() {
        return cfg.min_temp_c;
    }
    let sum: f32 = pixels.iter().map(|&p| thermal_pixel_to_temp(cfg, p)).sum();
    sum / pixels.len() as f32
}

pub fn thermal_hot_spot(cfg: &ThermalCameraConfig, pixels: &[u16]) -> u32 {
    pixels
        .iter()
        .enumerate()
        .max_by_key(|(_, &v)| v)
        .map(|(i, _)| i as u32)
        .unwrap_or(0)
        .min(cfg.width * cfg.height - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_config() {
        /* width/height set */
        let cfg = new_thermal_camera_config(320, 240);
        assert_eq!(cfg.width, 320);
        assert_eq!(cfg.height, 240);
    }

    #[test]
    fn test_pixel_to_temp_zero() {
        /* raw 0 => min_temp */
        let cfg = new_thermal_camera_config(10, 10);
        let t = thermal_pixel_to_temp(&cfg, 0);
        assert!((t - cfg.min_temp_c).abs() < 1e-3);
    }

    #[test]
    fn test_pixel_to_temp_max() {
        /* raw max => max_temp */
        let cfg = new_thermal_camera_config(10, 10);
        let t = thermal_pixel_to_temp(&cfg, u16::MAX);
        assert!((t - cfg.max_temp_c).abs() < 0.1);
    }

    #[test]
    fn test_temp_to_color_cold() {
        /* cold => blue */
        let c = thermal_temp_to_color(0.0, 0.0, 100.0);
        assert_eq!(c[0], 0);
        assert_eq!(c[2], 255);
    }

    #[test]
    fn test_temp_to_color_hot() {
        /* hot => red */
        let c = thermal_temp_to_color(100.0, 0.0, 100.0);
        assert_eq!(c[0], 255);
        assert_eq!(c[2], 0);
    }

    #[test]
    fn test_hot_spot() {
        /* hot spot returns index of max */
        let cfg = new_thermal_camera_config(3, 1);
        let pixels = vec![100u16, 500, 200];
        assert_eq!(thermal_hot_spot(&cfg, &pixels), 1);
    }
}
