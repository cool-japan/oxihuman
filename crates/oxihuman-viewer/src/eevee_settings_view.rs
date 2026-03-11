// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Eevee render settings panel view.

/// Eevee render settings.
#[derive(Debug, Clone)]
pub struct EeveeSettingsView {
    pub samples: u32,
    pub bloom_enabled: bool,
    pub ssr_enabled: bool,
    pub ambient_occlusion: bool,
    pub shadow_cube_size: u32,
}

impl Default for EeveeSettingsView {
    fn default() -> Self {
        Self {
            samples: 64,
            bloom_enabled: false,
            ssr_enabled: false,
            ambient_occlusion: false,
            shadow_cube_size: 512,
        }
    }
}

/// Create a new EeveeSettingsView with defaults.
pub fn new_eevee_settings_view() -> EeveeSettingsView {
    EeveeSettingsView::default()
}

/// Set render sample count.
pub fn eevee_set_samples(view: &mut EeveeSettingsView, samples: u32) {
    view.samples = samples.clamp(1, 4096);
}

/// Toggle bloom post-process effect.
pub fn eevee_set_bloom(view: &mut EeveeSettingsView, enabled: bool) {
    view.bloom_enabled = enabled;
}

/// Toggle screen-space reflections.
pub fn eevee_set_ssr(view: &mut EeveeSettingsView, enabled: bool) {
    view.ssr_enabled = enabled;
}

/// Set shadow cube map resolution (power-of-two).
pub fn eevee_set_shadow_cube(view: &mut EeveeSettingsView, size: u32) {
    let clamped = size.clamp(64, 4096);
    view.shadow_cube_size = clamped.next_power_of_two();
}

/// Serialize to JSON.
pub fn eevee_settings_to_json(view: &EeveeSettingsView) -> String {
    format!(
        r#"{{"samples":{},"bloom":{},"ssr":{},"ao":{},"shadow_cube":{}}}"#,
        view.samples,
        view.bloom_enabled,
        view.ssr_enabled,
        view.ambient_occlusion,
        view.shadow_cube_size,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let v = new_eevee_settings_view();
        assert_eq!(v.samples, 64 /* default samples */);
    }

    #[test]
    fn test_set_samples_clamp() {
        let mut v = new_eevee_settings_view();
        eevee_set_samples(&mut v, 0);
        assert_eq!(v.samples, 1 /* min clamped */);
    }

    #[test]
    fn test_set_samples_high() {
        let mut v = new_eevee_settings_view();
        eevee_set_samples(&mut v, 9999);
        assert_eq!(v.samples, 4096 /* max clamped */);
    }

    #[test]
    fn test_bloom_toggle() {
        let mut v = new_eevee_settings_view();
        eevee_set_bloom(&mut v, true);
        assert!(v.bloom_enabled /* enabled */);
    }

    #[test]
    fn test_ssr_toggle() {
        let mut v = new_eevee_settings_view();
        eevee_set_ssr(&mut v, true);
        assert!(v.ssr_enabled /* enabled */);
    }

    #[test]
    fn test_shadow_cube_power_of_two() {
        let mut v = new_eevee_settings_view();
        eevee_set_shadow_cube(&mut v, 300);
        assert!(v.shadow_cube_size.is_power_of_two() /* power of 2 */);
    }

    #[test]
    fn test_shadow_cube_clamp_low() {
        let mut v = new_eevee_settings_view();
        eevee_set_shadow_cube(&mut v, 10);
        assert!(v.shadow_cube_size >= 64 /* minimum */);
    }

    #[test]
    fn test_json_keys() {
        let v = new_eevee_settings_view();
        let j = eevee_settings_to_json(&v);
        assert!(j.contains("samples") /* key present */);
    }

    #[test]
    fn test_default_ao_off() {
        let v = EeveeSettingsView::default();
        assert!(!v.ambient_occlusion /* off by default */);
    }

    #[test]
    fn test_clone() {
        let v = new_eevee_settings_view();
        let c = v.clone();
        assert_eq!(c.samples, v.samples /* equal */);
    }
}
