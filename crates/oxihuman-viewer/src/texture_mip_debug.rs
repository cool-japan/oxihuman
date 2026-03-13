// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Mipmap level debug visualization (color-coded levels).

/// Mip level debug configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureMipDebugConfig {
    /// Enable mip debug view.
    pub enabled: bool,
    /// Maximum mip levels to distinguish (up to 16).
    pub max_levels: u32,
    /// Colors per mip level [R, G, B, A].
    pub level_colors: Vec<[f32; 4]>,
    /// Overlay opacity 0..=1.
    pub opacity: f32,
    /// Show mip level number as text.
    pub show_numbers: bool,
}

impl Default for TextureMipDebugConfig {
    fn default() -> Self {
        // Deterministic palette for mip levels
        let colors: Vec<[f32; 4]> = vec![
            [0.2, 0.8, 0.2, 1.0], // Level 0 - green (full res)
            [0.8, 0.8, 0.2, 1.0], // Level 1 - yellow
            [0.8, 0.5, 0.1, 1.0], // Level 2 - orange
            [0.8, 0.2, 0.2, 1.0], // Level 3 - red
            [0.6, 0.2, 0.8, 1.0], // Level 4 - purple
            [0.2, 0.4, 0.9, 1.0], // Level 5 - blue
            [0.2, 0.8, 0.8, 1.0], // Level 6 - cyan
            [0.5, 0.5, 0.5, 1.0], // Level 7+ - gray
        ];
        Self {
            enabled: false,
            max_levels: 8,
            level_colors: colors,
            opacity: 0.7,
            show_numbers: true,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_texture_mip_debug_config() -> TextureMipDebugConfig {
    TextureMipDebugConfig::default()
}

/// Get color for a mip level.
#[allow(dead_code)]
pub fn mip_level_color(level: u32, cfg: &TextureMipDebugConfig) -> [f32; 4] {
    if cfg.level_colors.is_empty() {
        return [1.0, 1.0, 1.0, 1.0];
    }
    let idx = (level as usize).min(cfg.level_colors.len() - 1);
    let mut c = cfg.level_colors[idx];
    c[3] = cfg.opacity;
    c
}

/// Enable.
#[allow(dead_code)]
pub fn tmd_enable(cfg: &mut TextureMipDebugConfig) {
    cfg.enabled = true;
}

/// Disable.
#[allow(dead_code)]
pub fn tmd_disable(cfg: &mut TextureMipDebugConfig) {
    cfg.enabled = false;
}

/// Set opacity.
#[allow(dead_code)]
pub fn tmd_set_opacity(cfg: &mut TextureMipDebugConfig, opacity: f32) {
    cfg.opacity = opacity.clamp(0.0, 1.0);
}

/// Compute mip level from texture size ratio.
#[allow(dead_code)]
pub fn compute_mip_level(base_size: u32, current_size: u32) -> u32 {
    if current_size == 0 || base_size == 0 {
        return 0;
    }
    let ratio = base_size as f32 / current_size as f32;
    ratio.log2().max(0.0) as u32
}

/// Total color entries.
#[allow(dead_code)]
pub fn tmd_color_count(cfg: &TextureMipDebugConfig) -> usize {
    cfg.level_colors.len()
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn texture_mip_debug_to_json(cfg: &TextureMipDebugConfig) -> String {
    format!(
        r#"{{"enabled":{},"max_levels":{},"opacity":{:.4},"show_numbers":{}}}"#,
        cfg.enabled, cfg.max_levels, cfg.opacity, cfg.show_numbers
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = TextureMipDebugConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.level_colors.len(), 8);
    }

    #[test]
    fn test_level_color_0() {
        let c = TextureMipDebugConfig::default();
        let col = mip_level_color(0, &c);
        /* level 0 green = 0.8; only alpha is replaced with opacity */
        assert!((col[1] - 0.8).abs() < 1e-5);
    }

    #[test]
    fn test_level_color_clamped() {
        let c = TextureMipDebugConfig::default();
        let last = mip_level_color(99, &c);
        let eight = mip_level_color(7, &c);
        assert!((last[0] - eight[0]).abs() < 1e-5);
    }

    #[test]
    fn test_enable_disable() {
        let mut c = TextureMipDebugConfig::default();
        tmd_enable(&mut c);
        assert!(c.enabled);
        tmd_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_opacity() {
        let mut c = TextureMipDebugConfig::default();
        tmd_set_opacity(&mut c, 0.4);
        assert!((c.opacity - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_compute_mip_level() {
        assert_eq!(compute_mip_level(1024, 256), 2);
        assert_eq!(compute_mip_level(1024, 1024), 0);
    }

    #[test]
    fn test_compute_mip_zero_size() {
        assert_eq!(compute_mip_level(1024, 0), 0);
    }

    #[test]
    fn test_color_count() {
        let c = TextureMipDebugConfig::default();
        assert_eq!(tmd_color_count(&c), 8);
    }

    #[test]
    fn test_opacity_applied() {
        let c = TextureMipDebugConfig {
            opacity: 0.5,
            ..Default::default()
        };
        let col = mip_level_color(0, &c);
        assert!((col[3] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let j = texture_mip_debug_to_json(&TextureMipDebugConfig::default());
        assert!(j.contains("max_levels"));
        assert!(j.contains("show_numbers"));
    }
}
