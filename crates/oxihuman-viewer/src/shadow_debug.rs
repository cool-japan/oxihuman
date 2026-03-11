// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Shadow map cascade debug visualization settings.

/// Shadow cascade debug configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShadowDebugConfig {
    /// Number of shadow cascades.
    pub cascade_count: u32,
    /// Currently visualized cascade index.
    pub active_cascade: u32,
    /// Show cascade split boundaries.
    pub show_splits: bool,
    /// Cascade colors for visualization.
    pub cascade_colors: [[f32; 4]; 4],
    /// Depth bias visualization scale.
    pub bias_scale: f32,
    /// Show shadow map texels overlay.
    pub show_texels: bool,
    /// Opacity of debug overlay 0..=1.
    pub overlay_opacity: f32,
}

impl Default for ShadowDebugConfig {
    fn default() -> Self {
        Self {
            cascade_count: 4,
            active_cascade: 0,
            show_splits: true,
            cascade_colors: [
                [1.0, 0.3, 0.3, 0.4],
                [0.3, 1.0, 0.3, 0.4],
                [0.3, 0.3, 1.0, 0.4],
                [1.0, 1.0, 0.3, 0.4],
            ],
            bias_scale: 1.0,
            show_texels: false,
            overlay_opacity: 0.4,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_shadow_debug_config() -> ShadowDebugConfig {
    ShadowDebugConfig::default()
}

/// Set active cascade.
#[allow(dead_code)]
pub fn sd_set_active_cascade(cfg: &mut ShadowDebugConfig, idx: u32) {
    cfg.active_cascade = idx.min(cfg.cascade_count.saturating_sub(1));
}

/// Set overlay opacity.
#[allow(dead_code)]
pub fn sd_set_opacity(cfg: &mut ShadowDebugConfig, opacity: f32) {
    cfg.overlay_opacity = opacity.clamp(0.0, 1.0);
}

/// Toggle split boundary display.
#[allow(dead_code)]
pub fn sd_toggle_splits(cfg: &mut ShadowDebugConfig) {
    cfg.show_splits = !cfg.show_splits;
}

/// Get cascade color for the active cascade.
#[allow(dead_code)]
pub fn sd_active_cascade_color(cfg: &ShadowDebugConfig) -> [f32; 4] {
    let idx = (cfg.active_cascade as usize).min(3);
    cfg.cascade_colors[idx]
}

/// Compute cascade split distance (simple logarithmic split).
#[allow(dead_code)]
pub fn sd_cascade_split(near: f32, far: f32, cascade: u32, total: u32) -> f32 {
    if total == 0 {
        return far;
    }
    let ratio = far / near.max(0.001);
    let lambda = 0.7f32;
    let t = cascade as f32 / total as f32;
    let log_split = near * ratio.powf(t);
    let lin_split = near + (far - near) * t;
    lambda * log_split + (1.0 - lambda) * lin_split
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn shadow_debug_to_json(cfg: &ShadowDebugConfig) -> String {
    format!(
        r#"{{"cascade_count":{},"active_cascade":{},"show_splits":{},"overlay_opacity":{:.4}}}"#,
        cfg.cascade_count, cfg.active_cascade, cfg.show_splits, cfg.overlay_opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = ShadowDebugConfig::default();
        assert_eq!(c.cascade_count, 4);
    }

    #[test]
    fn test_set_active_cascade_clamp() {
        let mut c = ShadowDebugConfig::default();
        sd_set_active_cascade(&mut c, 99);
        assert!(c.active_cascade < c.cascade_count);
    }

    #[test]
    fn test_set_opacity_clamp() {
        let mut c = ShadowDebugConfig::default();
        sd_set_opacity(&mut c, 5.0);
        assert!((c.overlay_opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_toggle_splits() {
        let mut c = ShadowDebugConfig::default();
        sd_toggle_splits(&mut c);
        assert!(!c.show_splits);
        sd_toggle_splits(&mut c);
        assert!(c.show_splits);
    }

    #[test]
    fn test_active_cascade_color() {
        let c = ShadowDebugConfig::default();
        let col = sd_active_cascade_color(&c);
        assert!((col[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cascade_split_ordered() {
        let s0 = sd_cascade_split(0.1, 100.0, 0, 4);
        let s1 = sd_cascade_split(0.1, 100.0, 1, 4);
        let s2 = sd_cascade_split(0.1, 100.0, 2, 4);
        assert!(s0 < s1);
        assert!(s1 < s2);
    }

    #[test]
    fn test_cascade_split_zero_total() {
        let s = sd_cascade_split(0.1, 100.0, 0, 0);
        assert!((s - 100.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let j = shadow_debug_to_json(&ShadowDebugConfig::default());
        assert!(j.contains("cascade_count"));
        assert!(j.contains("show_splits"));
    }

    #[test]
    fn test_set_active_cascade_valid() {
        let mut c = ShadowDebugConfig::default();
        sd_set_active_cascade(&mut c, 2);
        assert_eq!(c.active_cascade, 2);
    }

    #[test]
    fn test_overlay_opacity_range() {
        let c = ShadowDebugConfig::default();
        assert!((0.0..=1.0).contains(&c.overlay_opacity));
    }
}
