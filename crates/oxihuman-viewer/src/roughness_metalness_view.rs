// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Roughness + metalness channel debug visualization.

/// Which channel to display.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RmChannel {
    Roughness,
    Metalness,
    Combined,
}

/// Configuration for roughness/metalness view.
#[derive(Debug, Clone)]
pub struct RoughnessMetalnessViewConfig {
    pub channel: RmChannel,
    pub roughness_tint: [f32; 3],
    pub metalness_tint: [f32; 3],
}

impl Default for RoughnessMetalnessViewConfig {
    fn default() -> Self {
        Self {
            channel: RmChannel::Combined,
            roughness_tint: [1.0, 0.5, 0.0],
            metalness_tint: [0.0, 0.5, 1.0],
        }
    }
}

/// State for roughness/metalness visualization.
#[derive(Debug, Clone)]
pub struct RoughnessMetalnessView {
    pub config: RoughnessMetalnessViewConfig,
    pub enabled: bool,
}

impl Default for RoughnessMetalnessView {
    fn default() -> Self {
        Self { config: RoughnessMetalnessViewConfig::default(), enabled: false }
    }
}

/// Enable the roughness/metalness view.
pub fn rmv_enable(view: &mut RoughnessMetalnessView) {
    view.enabled = true;
}

/// Disable the roughness/metalness view.
pub fn rmv_disable(view: &mut RoughnessMetalnessView) {
    view.enabled = false;
}

/// Set the active display channel.
pub fn rmv_set_channel(view: &mut RoughnessMetalnessView, channel: RmChannel) {
    view.config.channel = channel;
}

/// Map roughness and metalness to a display color.
pub fn rmv_to_color(roughness: f32, metalness: f32, config: &RoughnessMetalnessViewConfig) -> [f32; 4] {
    let r = roughness.clamp(0.0, 1.0);
    let m = metalness.clamp(0.0, 1.0);
    match config.channel {
        RmChannel::Roughness => [r, r, r, 1.0],
        RmChannel::Metalness => [m, m, m, 1.0],
        RmChannel::Combined => {
            let t = config.roughness_tint;
            let u = config.metalness_tint;
            [
                t[0] * r + u[0] * m,
                t[1] * r + u[1] * m,
                t[2] * r + u[2] * m,
                1.0,
            ]
        }
    }
}

/// Return whether the given metalness value indicates a metallic material.
pub fn rmv_is_metallic(metalness: f32) -> bool {
    metalness > 0.5
}

/// Export config to JSON string (stub).
pub fn rmv_to_json(view: &RoughnessMetalnessView) -> String {
    let ch = match view.config.channel {
        RmChannel::Roughness => "roughness",
        RmChannel::Metalness => "metalness",
        RmChannel::Combined => "combined",
    };
    format!(r#"{{"channel":"{}","enabled":{}}}"#, ch, view.enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_disabled() {
        /* default should be disabled */
        let v = RoughnessMetalnessView::default();
        assert!(!v.enabled);
    }

    #[test]
    fn test_enable_disable() {
        /* enable/disable should toggle */
        let mut v = RoughnessMetalnessView::default();
        rmv_enable(&mut v);
        assert!(v.enabled);
        rmv_disable(&mut v);
        assert!(!v.enabled);
    }

    #[test]
    fn test_set_channel() {
        /* channel should be updated */
        let mut v = RoughnessMetalnessView::default();
        rmv_set_channel(&mut v, RmChannel::Roughness);
        assert_eq!(v.config.channel, RmChannel::Roughness);
    }

    #[test]
    fn test_roughness_channel_greyscale() {
        /* roughness channel should produce greyscale */
        let cfg = RoughnessMetalnessViewConfig { channel: RmChannel::Roughness, ..Default::default() };
        let c = rmv_to_color(0.7, 0.0, &cfg);
        assert!((c[0] - 0.7).abs() < 1e-6);
        assert!((c[0] - c[1]).abs() < 1e-6);
    }

    #[test]
    fn test_metalness_channel_greyscale() {
        /* metalness channel should produce greyscale */
        let cfg = RoughnessMetalnessViewConfig { channel: RmChannel::Metalness, ..Default::default() };
        let c = rmv_to_color(0.0, 0.8, &cfg);
        assert!((c[0] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_alpha_always_one() {
        /* alpha should be 1.0 */
        let cfg = RoughnessMetalnessViewConfig::default();
        let c = rmv_to_color(0.5, 0.5, &cfg);
        assert!((c[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_metallic_above_half() {
        /* metalness > 0.5 should be classified as metallic */
        assert!(rmv_is_metallic(0.8));
    }

    #[test]
    fn test_is_not_metallic_below_half() {
        /* metalness <= 0.5 should not be metallic */
        assert!(!rmv_is_metallic(0.4));
    }

    #[test]
    fn test_to_json_channel() {
        /* JSON should contain channel name */
        let v = RoughnessMetalnessView::default();
        let json = rmv_to_json(&v);
        assert!(json.contains("combined"));
    }
}
