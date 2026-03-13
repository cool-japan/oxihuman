// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! TV safe area guides overlay for broadcast production.

/// Safe area standard preset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SafeAreaPreset {
    Action,
    Title,
    Custom,
}

/// Safe area view configuration.
#[derive(Debug, Clone)]
pub struct SafeAreaView {
    pub preset: SafeAreaPreset,
    pub action_margin: f32,
    pub title_margin: f32,
    pub show_action: bool,
    pub show_title: bool,
    pub color: [f32; 4],
}

impl SafeAreaView {
    pub fn new() -> Self {
        Self {
            preset: SafeAreaPreset::Action,
            action_margin: 0.05,
            title_margin: 0.1,
            show_action: true,
            show_title: true,
            color: [0.0, 1.0, 0.0, 0.8],
        }
    }
}

impl Default for SafeAreaView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new safe area view.
pub fn new_safe_area_view() -> SafeAreaView {
    SafeAreaView::new()
}

/// Set action-safe margin as fraction of frame.
pub fn sav_set_action_margin(view: &mut SafeAreaView, margin: f32) {
    view.action_margin = margin.clamp(0.0, 0.5);
}

/// Set title-safe margin as fraction of frame.
pub fn sav_set_title_margin(view: &mut SafeAreaView, margin: f32) {
    view.title_margin = margin.clamp(0.0, 0.5);
}

/// Toggle visibility of action safe zone.
pub fn sav_show_action(view: &mut SafeAreaView, show: bool) {
    view.show_action = show;
}

/// Toggle visibility of title safe zone.
pub fn sav_show_title(view: &mut SafeAreaView, show: bool) {
    view.show_title = show;
}

/// Compute action-safe inner area fraction.
pub fn sav_action_safe_area(view: &SafeAreaView) -> f32 {
    let inner = (1.0 - 2.0 * view.action_margin).max(0.0);
    inner * inner
}

/// Serialize to JSON-like string.
pub fn safe_area_view_to_json(view: &SafeAreaView) -> String {
    let preset_str = match view.preset {
        SafeAreaPreset::Action => "action",
        SafeAreaPreset::Title => "title",
        SafeAreaPreset::Custom => "custom",
    };
    format!(
        r#"{{"preset":"{preset_str}","action_margin":{:.4},"title_margin":{:.4}}}"#,
        view.action_margin, view.title_margin
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let v = new_safe_area_view();
        assert_eq!(v.preset, SafeAreaPreset::Action);
        assert!(v.show_action);
    }

    #[test]
    fn test_action_margin_clamp() {
        let mut v = new_safe_area_view();
        sav_set_action_margin(&mut v, 1.0);
        assert_eq!(v.action_margin, 0.5);
    }

    #[test]
    fn test_title_margin() {
        let mut v = new_safe_area_view();
        sav_set_title_margin(&mut v, 0.15);
        assert!((v.title_margin - 0.15).abs() < 1e-6);
    }

    #[test]
    fn test_show_toggle() {
        let mut v = new_safe_area_view();
        sav_show_title(&mut v, false);
        assert!(!v.show_title);
    }

    #[test]
    fn test_action_area_positive() {
        let v = new_safe_area_view();
        assert!(sav_action_safe_area(&v) > 0.0);
    }

    #[test]
    fn test_action_area_max_margin() {
        let mut v = new_safe_area_view();
        sav_set_action_margin(&mut v, 0.5);
        assert_eq!(sav_action_safe_area(&v), 0.0);
    }

    #[test]
    fn test_json() {
        let v = new_safe_area_view();
        let s = safe_area_view_to_json(&v);
        assert!(s.contains("action_margin"));
    }

    #[test]
    fn test_clone() {
        let v = new_safe_area_view();
        let v2 = v.clone();
        assert_eq!(v2.show_title, v.show_title);
    }

    #[test]
    fn test_default_trait() {
        let v: SafeAreaView = Default::default();
        assert!((v.action_margin - 0.05).abs() < 1e-6);
    }
}
