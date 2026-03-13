// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Shadow catcher setup view.

/// Shadow catcher configuration.
#[derive(Debug, Clone)]
pub struct ShadowCatcherView {
    pub enabled: bool,
    pub only_shadow: bool,
    pub shadow_intensity: f32,
    pub object_ids: Vec<u32>,
}

impl Default for ShadowCatcherView {
    fn default() -> Self {
        Self {
            enabled: false,
            only_shadow: true,
            shadow_intensity: 1.0,
            object_ids: Vec::new(),
        }
    }
}

/// Create a new ShadowCatcherView.
pub fn new_shadow_catcher_view() -> ShadowCatcherView {
    ShadowCatcherView::default()
}

/// Enable or disable the shadow catcher pass.
pub fn shadow_catcher_set_enabled(view: &mut ShadowCatcherView, enabled: bool) {
    view.enabled = enabled;
}

/// Set shadow-only mode (render only the shadow, not the catcher geometry).
pub fn shadow_catcher_set_only_shadow(view: &mut ShadowCatcherView, only: bool) {
    view.only_shadow = only;
}

/// Set shadow intensity factor (0.0–1.0).
pub fn shadow_catcher_set_intensity(view: &mut ShadowCatcherView, v: f32) {
    view.shadow_intensity = v.clamp(0.0, 1.0);
}

/// Register an object as a shadow catcher.
pub fn shadow_catcher_add_object(view: &mut ShadowCatcherView, id: u32) {
    if !view.object_ids.contains(&id) {
        view.object_ids.push(id);
    }
}

/// Serialize to JSON.
pub fn shadow_catcher_to_json(view: &ShadowCatcherView) -> String {
    format!(
        r#"{{"enabled":{},"only_shadow":{},"intensity":{},"objects":{}}}"#,
        view.enabled,
        view.only_shadow,
        view.shadow_intensity,
        view.object_ids.len(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_shadow_catcher_view();
        assert!(!v.enabled /* disabled by default */);
    }

    #[test]
    fn test_enable() {
        let mut v = new_shadow_catcher_view();
        shadow_catcher_set_enabled(&mut v, true);
        assert!(v.enabled /* enabled */);
    }

    #[test]
    fn test_only_shadow_default() {
        let v = new_shadow_catcher_view();
        assert!(v.only_shadow /* only shadow on by default */);
    }

    #[test]
    fn test_intensity_clamp() {
        let mut v = new_shadow_catcher_view();
        shadow_catcher_set_intensity(&mut v, 5.0);
        assert!((v.shadow_intensity - 1.0).abs() < 1e-6 /* clamped */);
    }

    #[test]
    fn test_add_object() {
        let mut v = new_shadow_catcher_view();
        shadow_catcher_add_object(&mut v, 42);
        assert_eq!(v.object_ids.len(), 1 /* one object */);
    }

    #[test]
    fn test_add_duplicate() {
        let mut v = new_shadow_catcher_view();
        shadow_catcher_add_object(&mut v, 42);
        shadow_catcher_add_object(&mut v, 42);
        assert_eq!(v.object_ids.len(), 1 /* no duplicate */);
    }

    #[test]
    fn test_json_key() {
        let v = new_shadow_catcher_view();
        let j = shadow_catcher_to_json(&v);
        assert!(j.contains("only_shadow") /* key */);
    }

    #[test]
    fn test_default_intensity() {
        let v = ShadowCatcherView::default();
        assert!((v.shadow_intensity - 1.0).abs() < 1e-6 /* full intensity */);
    }

    #[test]
    fn test_clone() {
        let v = new_shadow_catcher_view();
        let c = v.clone();
        assert!((c.shadow_intensity - v.shadow_intensity).abs() < 1e-6 /* equal */);
    }
}
