// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Individual texture channel viewer (R/G/B/A).

#![allow(dead_code)]

/// Which texture channel to view.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureChannel {
    Red,
    Green,
    Blue,
    Alpha,
    All,
}

/// Config for texture channel viewer.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureChannelViewConfig {
    pub channel: TextureChannel,
    /// Render as grayscale.
    pub grayscale: bool,
    /// Invert the channel value.
    pub invert: bool,
    /// Exposure multiplier.
    pub exposure: f32,
    /// Mip level to display.
    pub mip_level: u32,
}

#[allow(dead_code)]
impl Default for TextureChannelViewConfig {
    fn default() -> Self {
        Self {
            channel: TextureChannel::All,
            grayscale: false,
            invert: false,
            exposure: 1.0,
            mip_level: 0,
        }
    }
}

/// Create default config.
#[allow(dead_code)]
pub fn new_texture_channel_view_config() -> TextureChannelViewConfig {
    TextureChannelViewConfig::default()
}

/// Extract the selected channel value from an RGBA pixel.
#[allow(dead_code)]
pub fn extract_channel(pixel: [f32; 4], channel: TextureChannel) -> f32 {
    match channel {
        TextureChannel::Red => pixel[0],
        TextureChannel::Green => pixel[1],
        TextureChannel::Blue => pixel[2],
        TextureChannel::Alpha => pixel[3],
        TextureChannel::All => (pixel[0] + pixel[1] + pixel[2]) / 3.0,
    }
}

/// Apply the channel view to a pixel.
#[allow(dead_code)]
pub fn apply_channel_view(pixel: [f32; 4], cfg: &TextureChannelViewConfig) -> [f32; 4] {
    let v = extract_channel(pixel, cfg.channel) * cfg.exposure;
    let v = if cfg.invert { 1.0 - v } else { v };
    let v = v.clamp(0.0, 1.0);
    if cfg.grayscale || cfg.channel != TextureChannel::All {
        [v, v, v, 1.0]
    } else {
        [
            pixel[0] * cfg.exposure,
            pixel[1] * cfg.exposure,
            pixel[2] * cfg.exposure,
            pixel[3],
        ]
    }
}

/// Set channel.
#[allow(dead_code)]
pub fn tcv_set_channel(cfg: &mut TextureChannelViewConfig, ch: TextureChannel) {
    cfg.channel = ch;
}

/// Toggle invert.
#[allow(dead_code)]
pub fn tcv_toggle_invert(cfg: &mut TextureChannelViewConfig) {
    cfg.invert = !cfg.invert;
}

/// Toggle grayscale.
#[allow(dead_code)]
pub fn tcv_toggle_grayscale(cfg: &mut TextureChannelViewConfig) {
    cfg.grayscale = !cfg.grayscale;
}

/// Set exposure.
#[allow(dead_code)]
pub fn tcv_set_exposure(cfg: &mut TextureChannelViewConfig, value: f32) {
    cfg.exposure = value.max(0.0);
}

/// Channel label string.
#[allow(dead_code)]
pub fn channel_label(ch: TextureChannel) -> &'static str {
    match ch {
        TextureChannel::Red => "R",
        TextureChannel::Green => "G",
        TextureChannel::Blue => "B",
        TextureChannel::Alpha => "A",
        TextureChannel::All => "RGB",
    }
}

/// Serialize to JSON.
#[allow(dead_code)]
pub fn texture_channel_view_to_json(cfg: &TextureChannelViewConfig) -> String {
    format!(
        r#"{{"channel":"{}","grayscale":{},"invert":{},"exposure":{:.4},"mip_level":{}}}"#,
        channel_label(cfg.channel),
        cfg.grayscale,
        cfg.invert,
        cfg.exposure,
        cfg.mip_level
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let c = TextureChannelViewConfig::default();
        assert_eq!(c.channel, TextureChannel::All);
        assert!(!c.grayscale);
    }

    #[test]
    fn test_extract_red() {
        let pixel = [0.8f32, 0.4, 0.2, 1.0];
        assert!((extract_channel(pixel, TextureChannel::Red) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_extract_alpha() {
        let pixel = [0.0f32, 0.0, 0.0, 0.5];
        assert!((extract_channel(pixel, TextureChannel::Alpha) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_channel_view_grayscale() {
        let cfg = TextureChannelViewConfig {
            channel: TextureChannel::Red,
            grayscale: true,
            invert: false,
            exposure: 1.0,
            mip_level: 0,
        };
        let pixel = [0.6f32, 0.0, 0.0, 1.0];
        let out = apply_channel_view(pixel, &cfg);
        assert!((out[0] - out[1]).abs() < 1e-6);
    }

    #[test]
    fn test_toggle_invert() {
        let mut c = TextureChannelViewConfig::default();
        tcv_toggle_invert(&mut c);
        assert!(c.invert);
    }

    #[test]
    fn test_channel_label() {
        assert_eq!(channel_label(TextureChannel::Alpha), "A");
        assert_eq!(channel_label(TextureChannel::All), "RGB");
    }

    #[test]
    fn test_set_exposure_clamped() {
        let mut c = TextureChannelViewConfig::default();
        tcv_set_exposure(&mut c, -1.0);
        assert!(c.exposure < 1e-6);
    }

    #[test]
    fn test_invert_value() {
        let cfg = TextureChannelViewConfig {
            channel: TextureChannel::Red,
            grayscale: false,
            invert: true,
            exposure: 1.0,
            mip_level: 0,
        };
        let pixel = [1.0f32, 0.0, 0.0, 1.0];
        let out = apply_channel_view(pixel, &cfg);
        assert!(out[0] < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let j = texture_channel_view_to_json(&TextureChannelViewConfig::default());
        assert!(j.contains("channel"));
        assert!(j.contains("exposure"));
    }
}
