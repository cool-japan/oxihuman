// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Depth buffer visualization (linear/log remapping).

/// Depth remapping mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthRemapMode {
    Linear,
    Logarithmic,
    Reversed,
}

/// Depth buffer view configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DepthBufferViewConfig {
    /// Near plane distance.
    pub near: f32,
    /// Far plane distance.
    pub far: f32,
    /// Remapping mode.
    pub mode: DepthRemapMode,
    /// Enabled.
    pub enabled: bool,
    /// Contrast boost 0..=4.
    pub contrast: f32,
}

impl Default for DepthBufferViewConfig {
    fn default() -> Self {
        Self {
            near: 0.1,
            far: 100.0,
            mode: DepthRemapMode::Linear,
            enabled: false,
            contrast: 1.0,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_depth_buffer_view_config() -> DepthBufferViewConfig {
    DepthBufferViewConfig::default()
}

/// Remap a raw depth value [0..=1] to a display value [0..=1].
#[allow(dead_code)]
pub fn remap_depth(raw: f32, cfg: &DepthBufferViewConfig) -> f32 {
    let raw = raw.clamp(0.0, 1.0);
    let range = cfg.far - cfg.near;
    let linear = if range > 0.0 {
        (raw * range + cfg.near - cfg.near) / range
    } else {
        raw
    };
    let result = match cfg.mode {
        DepthRemapMode::Linear => linear,
        DepthRemapMode::Logarithmic => {
            if linear > 0.0 {
                (linear.ln() + 1.0).clamp(0.0, 1.0)
            } else {
                0.0
            }
        }
        DepthRemapMode::Reversed => 1.0 - linear,
    };
    let c = cfg.contrast.clamp(0.0, 4.0);
    ((result - 0.5) * c + 0.5).clamp(0.0, 1.0)
}

/// Enable/disable depth view.
#[allow(dead_code)]
pub fn dbv_enable(cfg: &mut DepthBufferViewConfig) {
    cfg.enabled = true;
}

/// Disable depth view.
#[allow(dead_code)]
pub fn dbv_disable(cfg: &mut DepthBufferViewConfig) {
    cfg.enabled = false;
}

/// Set remap mode.
#[allow(dead_code)]
pub fn dbv_set_mode(cfg: &mut DepthBufferViewConfig, mode: DepthRemapMode) {
    cfg.mode = mode;
}

/// Set contrast.
#[allow(dead_code)]
pub fn dbv_set_contrast(cfg: &mut DepthBufferViewConfig, contrast: f32) {
    cfg.contrast = contrast.clamp(0.0, 4.0);
}

/// Depth value to grayscale color [R, G, B, A].
#[allow(dead_code)]
pub fn depth_to_color(raw: f32, cfg: &DepthBufferViewConfig) -> [f32; 4] {
    let d = remap_depth(raw, cfg);
    [d, d, d, 1.0]
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn depth_buffer_view_to_json(cfg: &DepthBufferViewConfig) -> String {
    let mode = match cfg.mode {
        DepthRemapMode::Linear => "linear",
        DepthRemapMode::Logarithmic => "log",
        DepthRemapMode::Reversed => "reversed",
    };
    format!(
        r#"{{"near":{:.4},"far":{:.4},"mode":"{}","enabled":{},"contrast":{:.4}}}"#,
        cfg.near, cfg.far, mode, cfg.enabled, cfg.contrast
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = DepthBufferViewConfig::default();
        assert!(!c.enabled);
    }

    #[test]
    fn test_remap_linear_pass() {
        let c = DepthBufferViewConfig {
            contrast: 1.0,
            ..Default::default()
        };
        let d = remap_depth(0.5, &c);
        assert!((d - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_remap_reversed() {
        let c = DepthBufferViewConfig {
            mode: DepthRemapMode::Reversed,
            contrast: 1.0,
            ..Default::default()
        };
        let d = remap_depth(0.0, &c);
        assert!((d - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_remap_clamp() {
        let c = DepthBufferViewConfig::default();
        let d = remap_depth(5.0, &c);
        assert!((0.0..=1.0).contains(&d));
    }

    #[test]
    fn test_enable_disable() {
        let mut c = DepthBufferViewConfig::default();
        dbv_enable(&mut c);
        assert!(c.enabled);
        dbv_disable(&mut c);
        assert!(!c.enabled);
    }

    #[test]
    fn test_set_mode() {
        let mut c = DepthBufferViewConfig::default();
        dbv_set_mode(&mut c, DepthRemapMode::Logarithmic);
        assert_eq!(c.mode, DepthRemapMode::Logarithmic);
    }

    #[test]
    fn test_set_contrast_clamp() {
        let mut c = DepthBufferViewConfig::default();
        dbv_set_contrast(&mut c, 10.0);
        assert!((c.contrast - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_depth_to_color_channels() {
        let c = DepthBufferViewConfig::default();
        let col = depth_to_color(0.5, &c);
        assert!((col[0] - col[1]).abs() < 1e-6);
        assert!((col[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = depth_buffer_view_to_json(&DepthBufferViewConfig::default());
        assert!(j.contains("near"));
        assert!(j.contains("mode"));
    }

    #[test]
    fn test_log_mode_nonzero() {
        let c = DepthBufferViewConfig {
            mode: DepthRemapMode::Logarithmic,
            contrast: 1.0,
            ..Default::default()
        };
        let d = remap_depth(0.5, &c);
        assert!(d >= 0.0);
    }
}
