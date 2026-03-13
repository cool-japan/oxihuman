// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Draw call frequency heatmap.

/// Draw call heatmap configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DrawCallHeatmapConfig {
    /// Enable heatmap overlay.
    pub enabled: bool,
    /// Maximum draw calls per region for full saturation.
    pub max_calls: u32,
    /// Grid resolution (number of cells per axis).
    pub grid_resolution: u32,
    /// Color for zero draw calls.
    pub color_cold: [f32; 4],
    /// Color for max draw calls.
    pub color_hot: [f32; 4],
    /// Overlay opacity 0..=1.
    pub opacity: f32,
}

impl Default for DrawCallHeatmapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_calls: 50,
            grid_resolution: 16,
            color_cold: [0.0, 0.0, 1.0, 0.5],
            color_hot: [1.0, 0.0, 0.0, 0.5],
            opacity: 0.5,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_draw_call_heatmap_config() -> DrawCallHeatmapConfig {
    DrawCallHeatmapConfig::default()
}

/// Map a draw call count to a heatmap color.
#[allow(dead_code)]
pub fn draw_call_to_color(count: u32, cfg: &DrawCallHeatmapConfig) -> [f32; 4] {
    let t = (count as f32 / cfg.max_calls as f32).clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let r = cfg.color_cold[0] * inv + cfg.color_hot[0] * t;
    let g = cfg.color_cold[1] * inv + cfg.color_hot[1] * t;
    let b = cfg.color_cold[2] * inv + cfg.color_hot[2] * t;
    [r, g, b, cfg.opacity]
}

/// Enable.
#[allow(dead_code)]
pub fn hm_enable(cfg: &mut DrawCallHeatmapConfig) {
    cfg.enabled = true;
}

/// Disable.
#[allow(dead_code)]
pub fn hm_disable(cfg: &mut DrawCallHeatmapConfig) {
    cfg.enabled = false;
}

/// Set max calls.
#[allow(dead_code)]
pub fn hm_set_max_calls(cfg: &mut DrawCallHeatmapConfig, max: u32) {
    cfg.max_calls = max.max(1);
}

/// Set grid resolution.
#[allow(dead_code)]
pub fn hm_set_grid_resolution(cfg: &mut DrawCallHeatmapConfig, res: u32) {
    cfg.grid_resolution = res.clamp(4, 256);
}

/// Set opacity.
#[allow(dead_code)]
pub fn hm_set_opacity(cfg: &mut DrawCallHeatmapConfig, opacity: f32) {
    cfg.opacity = opacity.clamp(0.0, 1.0);
}

/// Total cell count in the grid.
#[allow(dead_code)]
pub fn hm_cell_count(cfg: &DrawCallHeatmapConfig) -> u32 {
    cfg.grid_resolution * cfg.grid_resolution
}

/// Find the hottest cell in a grid buffer.
#[allow(dead_code)]
pub fn hm_find_hottest(grid: &[u32]) -> u32 {
    grid.iter().copied().max().unwrap_or(0)
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn draw_call_heatmap_to_json(cfg: &DrawCallHeatmapConfig) -> String {
    format!(
        r#"{{"enabled":{},"max_calls":{},"grid_resolution":{},"opacity":{:.4}}}"#,
        cfg.enabled, cfg.max_calls, cfg.grid_resolution, cfg.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = DrawCallHeatmapConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.grid_resolution, 16);
    }

    #[test]
    fn test_color_cold() {
        let c = DrawCallHeatmapConfig::default();
        let col = draw_call_to_color(0, &c);
        assert!((col[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_color_hot() {
        let c = DrawCallHeatmapConfig::default();
        let col = draw_call_to_color(c.max_calls, &c);
        assert!((col[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = DrawCallHeatmapConfig::default();
        hm_enable(&mut c);
        assert!(c.enabled);
        hm_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_max_calls_min() {
        let mut c = DrawCallHeatmapConfig::default();
        hm_set_max_calls(&mut c, 0);
        assert_eq!(c.max_calls, 1);
    }

    #[test]
    fn test_set_grid_resolution_clamp() {
        let mut c = DrawCallHeatmapConfig::default();
        hm_set_grid_resolution(&mut c, 1000);
        assert!(c.grid_resolution <= 256);
    }

    #[test]
    fn test_set_opacity() {
        let mut c = DrawCallHeatmapConfig::default();
        hm_set_opacity(&mut c, 0.3);
        assert!((c.opacity - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_cell_count() {
        let c = DrawCallHeatmapConfig::default();
        assert_eq!(hm_cell_count(&c), 16 * 16);
    }

    #[test]
    fn test_find_hottest() {
        let grid = [1u32, 5, 3, 2];
        assert_eq!(hm_find_hottest(&grid), 5);
    }

    #[test]
    fn test_to_json() {
        let j = draw_call_heatmap_to_json(&DrawCallHeatmapConfig::default());
        assert!(j.contains("max_calls"));
        assert!(j.contains("grid_resolution"));
    }
}
