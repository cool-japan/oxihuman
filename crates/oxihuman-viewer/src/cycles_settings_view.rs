// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Cycles render settings panel view.

/// Cycles render settings.
#[derive(Debug, Clone)]
pub struct CyclesSettingsView {
    pub max_bounces: u32,
    pub diffuse_bounces: u32,
    pub glossy_bounces: u32,
    pub samples: u32,
    pub use_denoiser: bool,
}

impl Default for CyclesSettingsView {
    fn default() -> Self {
        Self {
            max_bounces: 12,
            diffuse_bounces: 4,
            glossy_bounces: 4,
            samples: 128,
            use_denoiser: true,
        }
    }
}

/// Create a new CyclesSettingsView.
pub fn new_cycles_settings_view() -> CyclesSettingsView {
    CyclesSettingsView::default()
}

/// Set maximum path trace bounces.
pub fn cycles_set_max_bounces(view: &mut CyclesSettingsView, n: u32) {
    view.max_bounces = n.clamp(0, 1024);
}

/// Set diffuse bounce limit.
pub fn cycles_set_diffuse(view: &mut CyclesSettingsView, n: u32) {
    view.diffuse_bounces = n.min(view.max_bounces);
}

/// Set glossy bounce limit.
pub fn cycles_set_glossy(view: &mut CyclesSettingsView, n: u32) {
    view.glossy_bounces = n.min(view.max_bounces);
}

/// Set render sample count.
pub fn cycles_set_samples(view: &mut CyclesSettingsView, n: u32) {
    view.samples = n.clamp(1, 65536);
}

/// Serialize to JSON.
pub fn cycles_settings_to_json(view: &CyclesSettingsView) -> String {
    format!(
        r#"{{"max_bounces":{},"diffuse":{},"glossy":{},"samples":{},"denoiser":{}}}"#,
        view.max_bounces,
        view.diffuse_bounces,
        view.glossy_bounces,
        view.samples,
        view.use_denoiser,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let v = new_cycles_settings_view();
        assert_eq!(v.max_bounces, 12 /* default */);
    }

    #[test]
    fn test_max_bounces_clamp() {
        let mut v = new_cycles_settings_view();
        cycles_set_max_bounces(&mut v, 2000);
        assert_eq!(v.max_bounces, 1024 /* clamped */);
    }

    #[test]
    fn test_diffuse_capped() {
        let mut v = new_cycles_settings_view();
        cycles_set_max_bounces(&mut v, 5);
        cycles_set_diffuse(&mut v, 10);
        assert_eq!(v.diffuse_bounces, 5 /* capped to max */);
    }

    #[test]
    fn test_glossy_capped() {
        let mut v = new_cycles_settings_view();
        cycles_set_max_bounces(&mut v, 3);
        cycles_set_glossy(&mut v, 10);
        assert_eq!(v.glossy_bounces, 3 /* capped */);
    }

    #[test]
    fn test_samples_clamp_low() {
        let mut v = new_cycles_settings_view();
        cycles_set_samples(&mut v, 0);
        assert_eq!(v.samples, 1 /* min */);
    }

    #[test]
    fn test_samples_store() {
        let mut v = new_cycles_settings_view();
        cycles_set_samples(&mut v, 512);
        assert_eq!(v.samples, 512 /* stored */);
    }

    #[test]
    fn test_denoiser_default() {
        let v = new_cycles_settings_view();
        assert!(v.use_denoiser /* on by default */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_cycles_settings_view();
        let j = cycles_settings_to_json(&v);
        assert!(j.contains("max_bounces") /* key */);
    }

    #[test]
    fn test_clone() {
        let v = new_cycles_settings_view();
        let c = v.clone();
        assert_eq!(c.samples, v.samples /* equal */);
    }

    #[test]
    fn test_default_trait() {
        let v = CyclesSettingsView::default();
        assert_eq!(v.samples, 128 /* default samples */);
    }
}
