// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-tile light count visualization.

/// Light count overlay configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightCountOverlayConfig {
    /// Enable the overlay.
    pub enabled: bool,
    /// Tile size in pixels.
    pub tile_size_px: u32,
    /// Maximum light count for full-red.
    pub max_lights: u32,
    /// Color for zero lights.
    pub color_zero: [f32; 4],
    /// Color for max lights.
    pub color_max: [f32; 4],
    /// Show tile count numbers.
    pub show_numbers: bool,
    /// Overlay opacity 0..=1.
    pub opacity: f32,
}

impl Default for LightCountOverlayConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            tile_size_px: 16,
            max_lights: 32,
            color_zero: [0.0, 0.0, 0.5, 0.5],
            color_max: [1.0, 0.0, 0.0, 0.8],
            show_numbers: true,
            opacity: 0.6,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_light_count_overlay_config() -> LightCountOverlayConfig {
    LightCountOverlayConfig::default()
}

/// Map light count to overlay color.
#[allow(dead_code)]
pub fn light_count_to_color(count: u32, cfg: &LightCountOverlayConfig) -> [f32; 4] {
    let t = (count as f32 / cfg.max_lights as f32).clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let r = cfg.color_zero[0] * inv + cfg.color_max[0] * t;
    let g = cfg.color_zero[1] * inv + cfg.color_max[1] * t;
    let b = cfg.color_zero[2] * inv + cfg.color_max[2] * t;
    let a = cfg.color_zero[3] * inv + cfg.color_max[3] * t;
    [r, g, b, a * cfg.opacity]
}

/// Enable.
#[allow(dead_code)]
pub fn lco_enable(cfg: &mut LightCountOverlayConfig) {
    cfg.enabled = true;
}

/// Disable.
#[allow(dead_code)]
pub fn lco_disable(cfg: &mut LightCountOverlayConfig) {
    cfg.enabled = false;
}

/// Set tile size.
#[allow(dead_code)]
pub fn lco_set_tile_size(cfg: &mut LightCountOverlayConfig, size: u32) {
    cfg.tile_size_px = size.clamp(4, 128);
}

/// Set max lights.
#[allow(dead_code)]
pub fn lco_set_max_lights(cfg: &mut LightCountOverlayConfig, max: u32) {
    cfg.max_lights = max.max(1);
}

/// Set opacity.
#[allow(dead_code)]
pub fn lco_set_opacity(cfg: &mut LightCountOverlayConfig, opacity: f32) {
    cfg.opacity = opacity.clamp(0.0, 1.0);
}

/// Compute tile grid dimensions.
#[allow(dead_code)]
pub fn lco_tile_grid(width: u32, height: u32, cfg: &LightCountOverlayConfig) -> (u32, u32) {
    let ts = cfg.tile_size_px.max(1);
    (width.div_ceil(ts), height.div_ceil(ts))
}

/// Average light count from tile buffer.
#[allow(dead_code)]
pub fn lco_average_lights(tile_counts: &[u32]) -> f32 {
    if tile_counts.is_empty() {
        return 0.0;
    }
    let sum: u64 = tile_counts.iter().map(|&c| c as u64).sum();
    sum as f32 / tile_counts.len() as f32
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn light_count_overlay_to_json(cfg: &LightCountOverlayConfig) -> String {
    format!(
        r#"{{"enabled":{},"tile_size_px":{},"max_lights":{},"opacity":{:.4}}}"#,
        cfg.enabled, cfg.tile_size_px, cfg.max_lights, cfg.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = LightCountOverlayConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.tile_size_px, 16);
    }

    #[test]
    fn test_color_zero() {
        let c = LightCountOverlayConfig::default();
        let col = light_count_to_color(0, &c);
        /* blue channel equals color_zero[2] when t=0 */
        assert!((col[2] - c.color_zero[2]).abs() < 1e-4);
    }

    #[test]
    fn test_color_max_red() {
        let c = LightCountOverlayConfig::default();
        let col = light_count_to_color(c.max_lights, &c);
        assert!(col[0] > col[2]);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = LightCountOverlayConfig::default();
        lco_enable(&mut c);
        assert!(c.enabled);
        lco_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_tile_size_clamp() {
        let mut c = LightCountOverlayConfig::default();
        lco_set_tile_size(&mut c, 0);
        assert!(c.tile_size_px >= 4);
    }

    #[test]
    fn test_set_max_lights_min() {
        let mut c = LightCountOverlayConfig::default();
        lco_set_max_lights(&mut c, 0);
        assert_eq!(c.max_lights, 1);
    }

    #[test]
    fn test_tile_grid() {
        let c = LightCountOverlayConfig::default();
        let (tx, ty) = lco_tile_grid(1920, 1080, &c);
        assert_eq!(tx, 120);
        assert_eq!(ty, 68);
    }

    #[test]
    fn test_average_lights() {
        let tiles = [2u32, 4, 6];
        let avg = lco_average_lights(&tiles);
        assert!((avg - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_average_lights_empty() {
        assert!(lco_average_lights(&[]).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = light_count_overlay_to_json(&LightCountOverlayConfig::default());
        assert!(j.contains("tile_size_px"));
        assert!(j.contains("max_lights"));
    }
}
