// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Decal lifetime / fade lifecycle debug visualization.

/// Configuration for decal lifetime view.
#[derive(Debug, Clone)]
pub struct DecalLifetimeViewConfig {
    pub fade_duration: f32,
    pub show_age: bool,
    pub opacity: f32,
}

impl Default for DecalLifetimeViewConfig {
    fn default() -> Self {
        Self { fade_duration: 2.0, show_age: true, opacity: 0.7 }
    }
}

/// State for decal lifetime visualization.
#[derive(Debug, Clone)]
pub struct DecalLifetimeView {
    pub config: DecalLifetimeViewConfig,
    pub enabled: bool,
}

impl Default for DecalLifetimeView {
    fn default() -> Self {
        Self { config: DecalLifetimeViewConfig::default(), enabled: false }
    }
}

/// Enable the decal lifetime view.
pub fn dlv_enable(view: &mut DecalLifetimeView) {
    view.enabled = true;
}

/// Disable the decal lifetime view.
pub fn dlv_disable(view: &mut DecalLifetimeView) {
    view.enabled = false;
}

/// Compute the fade alpha for a decal given its age and total lifetime.
pub fn dlv_fade_alpha(age: f32, lifetime: f32, config: &DecalLifetimeViewConfig) -> f32 {
    let fade_start = (lifetime - config.fade_duration).max(0.0);
    if age <= fade_start {
        1.0
    } else {
        let fade_t = (age - fade_start) / config.fade_duration.max(0.0001);
        (1.0 - fade_t).clamp(0.0, 1.0)
    }
}

/// Map decal age to a debug colour.
pub fn dlv_age_to_color(age: f32, lifetime: f32, config: &DecalLifetimeViewConfig) -> [f32; 4] {
    let t = (age / lifetime.max(0.0001)).clamp(0.0, 1.0);
    let alpha = dlv_fade_alpha(age, lifetime, config);
    [1.0 - t, t * 0.5, 0.0, alpha * config.opacity]
}

/// Return whether the decal has fully faded.
pub fn dlv_is_expired(age: f32, lifetime: f32) -> bool {
    age >= lifetime
}

/// Set the fade duration.
pub fn dlv_set_fade_duration(view: &mut DecalLifetimeView, duration: f32) {
    view.config.fade_duration = duration.max(0.0);
}

/// Export config to JSON string (stub).
pub fn dlv_to_json(view: &DecalLifetimeView) -> String {
    format!(
        r#"{{"fade_duration":{:.2},"show_age":{},"enabled":{}}}"#,
        view.config.fade_duration, view.config.show_age, view.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = DecalLifetimeView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = DecalLifetimeView::default();
        dlv_enable(&mut v);
        assert!(v.enabled);
        dlv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_fade_alpha_young() {
        /* fresh decal should have full alpha */
        let cfg = DecalLifetimeViewConfig::default();
        let alpha = dlv_fade_alpha(0.0, 5.0, &cfg);
        assert!((alpha - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fade_alpha_expired() {
        /* expired decal should have zero alpha */
        let cfg = DecalLifetimeViewConfig::default();
        let alpha = dlv_fade_alpha(5.0, 5.0, &cfg);
        assert!(alpha < 1e-6);
    }

    #[test]
    fn test_is_expired() {
        /* age >= lifetime should be expired */
        assert!(dlv_is_expired(5.0, 5.0));
    }

    #[test]
    fn test_not_expired() {
        /* age < lifetime should not be expired */
        assert!(!dlv_is_expired(3.0, 5.0));
    }

    #[test]
    fn test_age_to_color_alpha_one() {
        /* fresh decal color alpha component should be nonzero */
        let cfg = DecalLifetimeViewConfig::default();
        let c = dlv_age_to_color(0.0, 10.0, &cfg);
        assert!(c[3] > 0.0);
    }

    #[test]
    fn test_set_fade_duration() {
        /* fade duration should be stored */
        let mut v = DecalLifetimeView::default();
        dlv_set_fade_duration(&mut v, 3.5);
        assert!((v.config.fade_duration - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_to_json_enabled() {
        /* JSON should contain enabled field */
        let mut v = DecalLifetimeView::default();
        dlv_enable(&mut v);
        let json = dlv_to_json(&v);
        assert!(json.contains("true"));
    }
}
