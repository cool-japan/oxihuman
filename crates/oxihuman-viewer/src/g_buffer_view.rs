// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! G-buffer channel inspector stub.

/// G-buffer channel type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GbvChannel {
    Albedo,
    Normal,
    Roughness,
    Metallic,
    Depth,
    Emissive,
}

/// G-buffer view configuration.
#[derive(Debug, Clone)]
pub struct GBufferViewConfig2 {
    pub channel: GbvChannel,
    pub exposure: f32,
    pub enabled: bool,
}

impl Default for GBufferViewConfig2 {
    fn default() -> Self {
        GBufferViewConfig2 {
            channel: GbvChannel::Albedo,
            exposure: 1.0,
            enabled: true,
        }
    }
}

/// Create a new G-buffer view config.
pub fn new_gbuffer_view2() -> GBufferViewConfig2 {
    GBufferViewConfig2::default()
}

/// Set the channel to view.
pub fn gbv2_set_channel(cfg: &mut GBufferViewConfig2, channel: GbvChannel) {
    cfg.channel = channel;
}

/// Set the exposure.
pub fn gbv2_set_exposure(cfg: &mut GBufferViewConfig2, exposure: f32) {
    cfg.exposure = exposure.max(0.0);
}

/// Enable or disable the view.
pub fn gbv2_set_enabled(cfg: &mut GBufferViewConfig2, enabled: bool) {
    cfg.enabled = enabled;
}

/// Return the channel name.
pub fn gbv2_channel_name(cfg: &GBufferViewConfig2) -> &'static str {
    match cfg.channel {
        GbvChannel::Albedo => "albedo",
        GbvChannel::Normal => "normal",
        GbvChannel::Roughness => "roughness",
        GbvChannel::Metallic => "metallic",
        GbvChannel::Depth => "depth",
        GbvChannel::Emissive => "emissive",
    }
}

/// Apply exposure to a pixel value.
pub fn gbv2_apply_exposure(value: f32, exposure: f32) -> f32 {
    value * exposure
}

/// Return a JSON-like string.
pub fn gbv2_to_json(cfg: &GBufferViewConfig2) -> String {
    format!(
        r#"{{"channel":"{}","exposure":{:.4},"enabled":{}}}"#,
        gbv2_channel_name(cfg),
        cfg.exposure,
        cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_channel_albedo() {
        let c = new_gbuffer_view2();
        assert_eq!(
            c.channel,
            GbvChannel::Albedo, /* default channel is Albedo */
        );
    }

    #[test]
    fn test_set_channel_normal() {
        let mut c = new_gbuffer_view2();
        gbv2_set_channel(&mut c, GbvChannel::Normal);
        assert_eq!(
            c.channel,
            GbvChannel::Normal, /* channel should be Normal */
        );
    }

    #[test]
    fn test_set_exposure() {
        let mut c = new_gbuffer_view2();
        gbv2_set_exposure(&mut c, 2.0);
        assert!((c.exposure - 2.0).abs() < 1e-5, /* exposure must match */);
    }

    #[test]
    fn test_set_exposure_negative_clamps() {
        let mut c = new_gbuffer_view2();
        gbv2_set_exposure(&mut c, -1.0);
        assert!((c.exposure).abs() < 1e-6, /* negative exposure clamped to 0 */);
    }

    #[test]
    fn test_set_enabled_false() {
        let mut c = new_gbuffer_view2();
        gbv2_set_enabled(&mut c, false);
        assert!(!c.enabled /* should be disabled */,);
    }

    #[test]
    fn test_channel_name_albedo() {
        let c = new_gbuffer_view2();
        assert_eq!(
            gbv2_channel_name(&c),
            "albedo", /* albedo channel name */
        );
    }

    #[test]
    fn test_channel_name_depth() {
        let mut c = new_gbuffer_view2();
        gbv2_set_channel(&mut c, GbvChannel::Depth);
        assert_eq!(gbv2_channel_name(&c), "depth" /* depth channel name */,);
    }

    #[test]
    fn test_apply_exposure() {
        let v = gbv2_apply_exposure(0.5, 2.0);
        assert!((v - 1.0).abs() < 1e-5 /* 0.5 * 2.0 = 1.0 */,);
    }

    #[test]
    fn test_to_json_contains_channel() {
        let c = new_gbuffer_view2();
        let j = gbv2_to_json(&c);
        assert!(j.contains("channel") /* JSON must contain channel */,);
    }

    #[test]
    fn test_default_enabled_true() {
        let c = new_gbuffer_view2();
        assert!(c.enabled /* enabled by default */,);
    }
}
