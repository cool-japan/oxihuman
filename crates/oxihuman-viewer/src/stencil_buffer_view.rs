// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Stencil buffer debug view (value to color mapping).

/// Stencil buffer view configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StencilBufferViewConfig {
    /// Enable the debug view.
    pub enabled: bool,
    /// Number of distinct stencil values to color map (up to 256).
    pub value_count: u32,
    /// Color palette: one [R, G, B, A] per stencil value.
    pub palette: Vec<[f32; 4]>,
    /// Background color for stencil value 0.
    pub background_color: [f32; 4],
    /// Opacity of overlay 0..=1.
    pub opacity: f32,
}

impl Default for StencilBufferViewConfig {
    fn default() -> Self {
        let mut palette = Vec::with_capacity(8);
        // Deterministic color palette based on value index
        for i in 0u32..8 {
            let h = i as f32 / 8.0;
            let r = (h * 6.0).fract();
            let s = (h * 6.0 + 2.0).fract();
            let t = (h * 6.0 + 4.0).fract();
            palette.push([r, s, t, 0.6]);
        }
        Self {
            enabled: false,
            value_count: 8,
            palette,
            background_color: [0.0, 0.0, 0.0, 0.0],
            opacity: 0.7,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_stencil_buffer_view_config() -> StencilBufferViewConfig {
    StencilBufferViewConfig::default()
}

/// Map a stencil value to a color.
#[allow(dead_code)]
pub fn stencil_value_to_color(value: u8, cfg: &StencilBufferViewConfig) -> [f32; 4] {
    if value == 0 {
        return cfg.background_color;
    }
    if cfg.palette.is_empty() {
        return [1.0, 1.0, 1.0, 1.0];
    }
    let idx = (value as usize - 1) % cfg.palette.len();
    let mut c = cfg.palette[idx];
    c[3] = cfg.opacity;
    c
}

/// Enable.
#[allow(dead_code)]
pub fn sbv_enable(cfg: &mut StencilBufferViewConfig) {
    cfg.enabled = true;
}

/// Disable.
#[allow(dead_code)]
pub fn sbv_disable(cfg: &mut StencilBufferViewConfig) {
    cfg.enabled = false;
}

/// Set opacity.
#[allow(dead_code)]
pub fn sbv_set_opacity(cfg: &mut StencilBufferViewConfig, opacity: f32) {
    cfg.opacity = opacity.clamp(0.0, 1.0);
}

/// Set palette entry.
#[allow(dead_code)]
pub fn sbv_set_palette_entry(cfg: &mut StencilBufferViewConfig, index: usize, color: [f32; 4]) {
    if index < cfg.palette.len() {
        cfg.palette[index] = color;
    }
}

/// Palette size.
#[allow(dead_code)]
pub fn sbv_palette_size(cfg: &StencilBufferViewConfig) -> usize {
    cfg.palette.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn stencil_buffer_view_to_json(cfg: &StencilBufferViewConfig) -> String {
    format!(
        r#"{{"enabled":{},"value_count":{},"opacity":{:.4},"palette_size":{}}}"#,
        cfg.enabled,
        cfg.value_count,
        cfg.opacity,
        cfg.palette.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = StencilBufferViewConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.palette.len(), 8);
    }

    #[test]
    fn test_value_zero_is_background() {
        let c = StencilBufferViewConfig::default();
        let col = stencil_value_to_color(0, &c);
        assert_eq!(col, c.background_color);
    }

    #[test]
    fn test_value_nonzero_different_from_bg() {
        let c = StencilBufferViewConfig::default();
        let col = stencil_value_to_color(1, &c);
        assert!(col != c.background_color);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = StencilBufferViewConfig::default();
        sbv_enable(&mut c);
        assert!(c.enabled);
        sbv_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_opacity_clamp() {
        let mut c = StencilBufferViewConfig::default();
        sbv_set_opacity(&mut c, 5.0);
        assert!((c.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_palette_entry() {
        let mut c = StencilBufferViewConfig::default();
        sbv_set_palette_entry(&mut c, 0, [1.0, 0.0, 0.0, 1.0]);
        assert!((c.palette[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_palette_size() {
        let c = StencilBufferViewConfig::default();
        assert_eq!(sbv_palette_size(&c), 8);
    }

    #[test]
    fn test_wrapping() {
        let c = StencilBufferViewConfig::default();
        let c8 = stencil_value_to_color(1, &c);
        let c16 = stencil_value_to_color(9, &c);
        assert!((c8[0] - c16[0]).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let j = stencil_buffer_view_to_json(&StencilBufferViewConfig::default());
        assert!(j.contains("enabled"));
        assert!(j.contains("palette_size"));
    }

    #[test]
    fn test_opacity_applied_to_color() {
        let mut c = StencilBufferViewConfig::default();
        sbv_set_opacity(&mut c, 0.3);
        let col = stencil_value_to_color(1, &c);
        assert!((col[3] - 0.3).abs() < 1e-5);
    }
}
