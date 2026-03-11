// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Denoiser before/after split view stub.

/// Denoiser view split mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DenoiseSplitMode {
    Horizontal,
    Vertical,
    FullBefore,
    FullAfter,
}

/// Denoiser view config.
#[derive(Debug, Clone)]
pub struct DenoiserViewConfig {
    pub split_mode: DenoiseSplitMode,
    pub split_position: f32,
    pub enabled: bool,
}

impl Default for DenoiserViewConfig {
    fn default() -> Self {
        DenoiserViewConfig {
            split_mode: DenoiseSplitMode::Horizontal,
            split_position: 0.5,
            enabled: true,
        }
    }
}

/// Create a new denoiser view config.
pub fn new_denoiser_view() -> DenoiserViewConfig {
    DenoiserViewConfig::default()
}

/// Set the split position.
pub fn dnv_set_split(cfg: &mut DenoiserViewConfig, pos: f32) {
    cfg.split_position = pos.clamp(0.0, 1.0);
}

/// Set the split mode.
pub fn dnv_set_mode(cfg: &mut DenoiserViewConfig, mode: DenoiseSplitMode) {
    cfg.split_mode = mode;
}

/// Enable or disable.
pub fn dnv_set_enabled(cfg: &mut DenoiserViewConfig, enabled: bool) {
    cfg.enabled = enabled;
}

/// Determine which side of the split a pixel falls on.
pub fn dnv_is_before_side(cfg: &DenoiserViewConfig, uv: [f32; 2]) -> bool {
    match cfg.split_mode {
        DenoiseSplitMode::Horizontal => uv[0] < cfg.split_position,
        DenoiseSplitMode::Vertical => uv[1] < cfg.split_position,
        DenoiseSplitMode::FullBefore => true,
        DenoiseSplitMode::FullAfter => false,
    }
}

/// Return a JSON-like string.
pub fn dnv_to_json(cfg: &DenoiserViewConfig) -> String {
    format!(
        r#"{{"split":{:.4},"enabled":{}}}"#,
        cfg.split_position, cfg.enabled
    )
}

/// Return the mode name.
pub fn dnv_mode_name(cfg: &DenoiserViewConfig) -> &'static str {
    match cfg.split_mode {
        DenoiseSplitMode::Horizontal => "horizontal",
        DenoiseSplitMode::Vertical => "vertical",
        DenoiseSplitMode::FullBefore => "full_before",
        DenoiseSplitMode::FullAfter => "full_after",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_split_mode() {
        let c = new_denoiser_view();
        assert_eq!(
            c.split_mode,
            DenoiseSplitMode::Horizontal, /* default is Horizontal */
        );
    }

    #[test]
    fn test_set_split_position() {
        let mut c = new_denoiser_view();
        dnv_set_split(&mut c, 0.7);
        assert!((c.split_position - 0.7).abs() < 1e-5, /* split position must match */);
    }

    #[test]
    fn test_set_split_clamps() {
        let mut c = new_denoiser_view();
        dnv_set_split(&mut c, 2.0);
        assert!((c.split_position - 1.0).abs() < 1e-5, /* split clamped to 1 */);
    }

    #[test]
    fn test_set_mode() {
        let mut c = new_denoiser_view();
        dnv_set_mode(&mut c, DenoiseSplitMode::Vertical);
        assert_eq!(
            c.split_mode,
            DenoiseSplitMode::Vertical, /* mode must match */
        );
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_denoiser_view();
        dnv_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_is_before_horizontal_left() {
        let c = new_denoiser_view();
        assert!(dnv_is_before_side(&c, [0.3, 0.5]), /* left of center is before */);
    }

    #[test]
    fn test_is_before_horizontal_right() {
        let c = new_denoiser_view();
        assert!(!dnv_is_before_side(&c, [0.8, 0.5]), /* right of center is after */);
    }

    #[test]
    fn test_full_before_mode() {
        let mut c = new_denoiser_view();
        dnv_set_mode(&mut c, DenoiseSplitMode::FullBefore);
        assert!(dnv_is_before_side(&c, [0.9, 0.9]), /* full before is always before */);
    }

    #[test]
    fn test_to_json_contains_split() {
        let c = new_denoiser_view();
        let j = dnv_to_json(&c);
        assert!(j.contains("split") /* JSON must contain split */,);
    }

    #[test]
    fn test_mode_name_horizontal() {
        let c = new_denoiser_view();
        assert_eq!(
            dnv_mode_name(&c),
            "horizontal", /* mode name must match */
        );
    }
}
