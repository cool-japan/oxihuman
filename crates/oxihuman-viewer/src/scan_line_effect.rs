// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Scan-line effect — CRT scan-line overlay parameters.

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ScanLineEffectConfig {
    /// Spacing between scan lines in pixels.
    pub line_spacing_px: f32,
    /// Darkness of scan lines 0 (invisible) .. 1 (black).
    pub darkness: f32,
    /// Slight curvature offset applied to each line position (normalised).
    pub wave_amplitude: f32,
    /// Frequency of the wave (in lines per scan).
    pub wave_frequency: f32,
    /// Flicker rate (lines per second that shift).
    pub flicker_rate: f32,
    /// Overall opacity of the scan-line overlay 0..=1.
    pub opacity: f32,
    pub enabled: bool,
}

impl Default for ScanLineEffectConfig {
    fn default() -> Self {
        Self {
            line_spacing_px: 3.0,
            darkness: 0.25,
            wave_amplitude: 0.001,
            wave_frequency: 0.1,
            flicker_rate: 0.02,
            opacity: 0.7,
            enabled: true,
        }
    }
}

#[allow(dead_code)]
pub fn new_scan_line_effect_config() -> ScanLineEffectConfig {
    ScanLineEffectConfig::default()
}

#[allow(dead_code)]
pub fn sl_set_darkness(cfg: &mut ScanLineEffectConfig, v: f32) {
    cfg.darkness = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sl_set_opacity(cfg: &mut ScanLineEffectConfig, v: f32) {
    cfg.opacity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn sl_set_line_spacing(cfg: &mut ScanLineEffectConfig, px: f32) {
    cfg.line_spacing_px = px.clamp(1.0, 32.0);
}

/// Returns the darkening factor at a given pixel y-coordinate and time.
#[allow(dead_code)]
pub fn sl_darkening_at(y_px: f32, time_s: f32, cfg: &ScanLineEffectConfig) -> f32 {
    if !cfg.enabled {
        return 0.0;
    }
    let flicker = (time_s * cfg.flicker_rate).fract();
    let effective_y = y_px + flicker * cfg.line_spacing_px;
    let phase = (effective_y / cfg.line_spacing_px).fract();
    // Phase near 0 or 1 → dark scan line
    let on_line = if !(0.3..=0.7).contains(&phase) {
        1.0
    } else {
        0.0
    };
    on_line * cfg.darkness * cfg.opacity
}

/// Number of scan lines for a given screen height.
#[allow(dead_code)]
pub fn sl_line_count(height_px: f32, cfg: &ScanLineEffectConfig) -> u32 {
    (height_px / cfg.line_spacing_px.max(1e-3)) as u32
}

/// Effective darkness at the midpoint of the screen.
#[allow(dead_code)]
pub fn sl_midpoint_darkening(height_px: f32, cfg: &ScanLineEffectConfig) -> f32 {
    sl_darkening_at(height_px * 0.5, 0.0, cfg)
}

#[allow(dead_code)]
pub fn sl_to_json(cfg: &ScanLineEffectConfig) -> String {
    format!(
        "{{\"line_spacing_px\":{:.1},\"darkness\":{:.3},\"opacity\":{:.3},\"enabled\":{}}}",
        cfg.line_spacing_px, cfg.darkness, cfg.opacity, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_zero_darkening() {
        let mut cfg = new_scan_line_effect_config();
        cfg.enabled = false;
        assert!(sl_darkening_at(10.0, 0.0, &cfg) < 1e-8);
    }

    #[test]
    fn darkening_in_range() {
        let cfg = new_scan_line_effect_config();
        let d = sl_darkening_at(0.0, 0.0, &cfg);
        assert!((0.0..=1.0).contains(&d));
    }

    #[test]
    fn line_count_positive() {
        let cfg = new_scan_line_effect_config();
        assert!(sl_line_count(480.0, &cfg) > 0);
    }

    #[test]
    fn line_count_proportional() {
        let cfg = new_scan_line_effect_config();
        let a = sl_line_count(240.0, &cfg);
        let b = sl_line_count(480.0, &cfg);
        assert!(b >= a);
    }

    #[test]
    fn darkness_clamps() {
        let mut cfg = new_scan_line_effect_config();
        sl_set_darkness(&mut cfg, 5.0);
        assert!((cfg.darkness - 1.0).abs() < 1e-6);
    }

    #[test]
    fn opacity_clamps() {
        let mut cfg = new_scan_line_effect_config();
        sl_set_opacity(&mut cfg, -1.0);
        assert!(cfg.opacity < 1e-6);
    }

    #[test]
    fn line_spacing_clamps_low() {
        let mut cfg = new_scan_line_effect_config();
        sl_set_line_spacing(&mut cfg, 0.0);
        assert!((cfg.line_spacing_px - 1.0).abs() < 1e-6);
    }

    #[test]
    fn line_spacing_clamps_high() {
        let mut cfg = new_scan_line_effect_config();
        sl_set_line_spacing(&mut cfg, 100.0);
        assert!((cfg.line_spacing_px - 32.0).abs() < 1e-6);
    }

    #[test]
    fn midpoint_darkening_in_range() {
        let cfg = new_scan_line_effect_config();
        let d = sl_midpoint_darkening(480.0, &cfg);
        assert!((0.0..=1.0).contains(&d));
    }

    #[test]
    fn json_has_keys() {
        let j = sl_to_json(&new_scan_line_effect_config());
        assert!(j.contains("line_spacing_px") && j.contains("enabled"));
    }
}
