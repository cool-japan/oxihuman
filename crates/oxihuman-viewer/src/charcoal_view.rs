// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Charcoal smudge effect view stub.

/// Charcoal smudge direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmudgeDir {
    Horizontal,
    Vertical,
    Radial,
    Random,
}

/// Charcoal view configuration.
#[derive(Debug, Clone)]
pub struct CharcoalView {
    pub smudge_dir: SmudgeDir,
    pub smudge_amount: f32,
    pub darkness: f32,
    pub grain_scale: f32,
    pub enabled: bool,
}

impl CharcoalView {
    pub fn new() -> Self {
        CharcoalView {
            smudge_dir: SmudgeDir::Random,
            smudge_amount: 0.4,
            darkness: 0.7,
            grain_scale: 1.0,
            enabled: true,
        }
    }
}

impl Default for CharcoalView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new charcoal view.
pub fn new_charcoal_view() -> CharcoalView {
    CharcoalView::new()
}

/// Set smudge direction.
pub fn chv_set_smudge_dir(view: &mut CharcoalView, dir: SmudgeDir) {
    view.smudge_dir = dir;
}

/// Set smudge amount.
pub fn chv_set_smudge_amount(view: &mut CharcoalView, amount: f32) {
    view.smudge_amount = amount.clamp(0.0, 1.0);
}

/// Set darkness level.
pub fn chv_set_darkness(view: &mut CharcoalView, darkness: f32) {
    view.darkness = darkness.clamp(0.0, 1.0);
}

/// Set grain scale.
pub fn chv_set_grain_scale(view: &mut CharcoalView, scale: f32) {
    view.grain_scale = scale.clamp(0.1, 5.0);
}

/// Enable or disable.
pub fn chv_set_enabled(view: &mut CharcoalView, enabled: bool) {
    view.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn chv_to_json(view: &CharcoalView) -> String {
    let dir = match view.smudge_dir {
        SmudgeDir::Horizontal => "horizontal",
        SmudgeDir::Vertical => "vertical",
        SmudgeDir::Radial => "radial",
        SmudgeDir::Random => "random",
    };
    format!(
        r#"{{"smudge_dir":"{}","smudge_amount":{},"darkness":{},"grain_scale":{},"enabled":{}}}"#,
        dir, view.smudge_amount, view.darkness, view.grain_scale, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dir() {
        let v = new_charcoal_view();
        assert_eq!(
            v.smudge_dir,
            SmudgeDir::Random /* default smudge_dir must be Random */
        );
    }

    #[test]
    fn test_set_dir() {
        let mut v = new_charcoal_view();
        chv_set_smudge_dir(&mut v, SmudgeDir::Radial);
        assert_eq!(
            v.smudge_dir,
            SmudgeDir::Radial /* smudge_dir must be set */
        );
    }

    #[test]
    fn test_smudge_amount_clamped() {
        let mut v = new_charcoal_view();
        chv_set_smudge_amount(&mut v, 2.0);
        assert!((v.smudge_amount - 1.0).abs() < 1e-6 /* smudge_amount clamped to 1.0 */);
    }

    #[test]
    fn test_darkness_clamped() {
        let mut v = new_charcoal_view();
        chv_set_darkness(&mut v, -0.5);
        assert!((v.darkness).abs() < 1e-6 /* darkness clamped to 0.0 */);
    }

    #[test]
    fn test_grain_scale_clamped_min() {
        let mut v = new_charcoal_view();
        chv_set_grain_scale(&mut v, 0.0);
        assert!((v.grain_scale - 0.1).abs() < 1e-6 /* grain_scale clamped to 0.1 */);
    }

    #[test]
    fn test_grain_scale_clamped_max() {
        let mut v = new_charcoal_view();
        chv_set_grain_scale(&mut v, 10.0);
        assert!((v.grain_scale - 5.0).abs() < 1e-6 /* grain_scale clamped to 5.0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut v = new_charcoal_view();
        chv_set_enabled(&mut v, false);
        assert!(!v.enabled /* must be disabled */);
    }

    #[test]
    fn test_to_json_has_dir() {
        let v = new_charcoal_view();
        let j = chv_to_json(&v);
        assert!(j.contains("\"smudge_dir\"") /* JSON must have smudge_dir */);
    }

    #[test]
    fn test_enabled_default() {
        let v = new_charcoal_view();
        assert!(v.enabled /* must be enabled by default */);
    }

    #[test]
    fn test_default_trait() {
        let v = CharcoalView::default();
        assert!((v.darkness - 0.7).abs() < 1e-6 /* Default trait must give 0.7 darkness */);
    }
}
