// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Infrared spectrum simulation stub.

/// Infrared color mapping style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IrColorMap {
    Grayscale,
    FalseColor,
    IronBow,
}

/// Infrared view configuration.
#[derive(Debug, Clone)]
pub struct InfraredView {
    pub color_map: IrColorMap,
    pub glow_radius: f32,
    pub foliage_boost: f32,
    pub sky_darken: f32,
    pub channel_mix: [f32; 3],
    pub enabled: bool,
}

impl InfraredView {
    pub fn new() -> Self {
        InfraredView {
            color_map: IrColorMap::Grayscale,
            glow_radius: 2.0,
            foliage_boost: 1.5,
            sky_darken: 0.8,
            channel_mix: [0.299, 0.587, 0.114],
            enabled: true,
        }
    }
}

impl Default for InfraredView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new infrared view.
pub fn new_infrared_view() -> InfraredView {
    InfraredView::new()
}

/// Apply the IR color map to an RGB pixel (stub: returns grayscale).
pub fn irv_apply_pixel(irv: &InfraredView, rgb: [f32; 3]) -> [f32; 3] {
    /* Stub: converts to luminance based on channel_mix and returns grayscale or maps */
    let lum =
        rgb[0] * irv.channel_mix[0] + rgb[1] * irv.channel_mix[1] + rgb[2] * irv.channel_mix[2];
    match irv.color_map {
        IrColorMap::Grayscale => [lum; 3],
        IrColorMap::FalseColor => [1.0 - lum, lum, 0.0],
        IrColorMap::IronBow => [lum, lum * 0.5, 0.0],
    }
}

/// Set the color mapping style.
pub fn irv_set_color_map(irv: &mut InfraredView, color_map: IrColorMap) {
    irv.color_map = color_map;
}

/// Set foliage boost.
pub fn irv_set_foliage_boost(irv: &mut InfraredView, boost: f32) {
    irv.foliage_boost = boost.max(0.0);
}

/// Enable or disable.
pub fn irv_set_enabled(irv: &mut InfraredView, enabled: bool) {
    irv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn irv_to_json(irv: &InfraredView) -> String {
    let cm = match irv.color_map {
        IrColorMap::Grayscale => "grayscale",
        IrColorMap::FalseColor => "false_color",
        IrColorMap::IronBow => "iron_bow",
    };
    format!(
        r#"{{"color_map":"{}","glow_radius":{},"foliage_boost":{},"enabled":{}}}"#,
        cm, irv.glow_radius, irv.foliage_boost, irv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_color_map_grayscale() {
        let i = new_infrared_view();
        assert_eq!(
            i.color_map,
            IrColorMap::Grayscale, /* default color map must be grayscale */
        );
    }

    #[test]
    fn test_apply_pixel_grayscale() {
        let i = new_infrared_view();
        let out = irv_apply_pixel(&i, [1.0, 1.0, 1.0]);
        assert!((out[0] - out[1]).abs() < 1e-5, /* grayscale output must have equal channels */);
    }

    #[test]
    fn test_apply_pixel_false_color() {
        let mut i = new_infrared_view();
        irv_set_color_map(&mut i, IrColorMap::FalseColor);
        let out = irv_apply_pixel(&i, [0.0, 0.0, 0.0]);
        assert!((out[0] - 1.0).abs() < 1e-5, /* false color of black must have red=1 */);
    }

    #[test]
    fn test_set_color_map() {
        let mut i = new_infrared_view();
        irv_set_color_map(&mut i, IrColorMap::IronBow);
        assert_eq!(
            i.color_map,
            IrColorMap::IronBow, /* color map must be set */
        );
    }

    #[test]
    fn test_set_foliage_boost() {
        let mut i = new_infrared_view();
        irv_set_foliage_boost(&mut i, 2.0);
        assert!((i.foliage_boost - 2.0).abs() < 1e-5, /* foliage boost must be set */);
    }

    #[test]
    fn test_foliage_boost_clamped() {
        let mut i = new_infrared_view();
        irv_set_foliage_boost(&mut i, -1.0);
        assert!((i.foliage_boost).abs() < 1e-6, /* negative boost clamped to 0 */);
    }

    #[test]
    fn test_set_enabled() {
        let mut i = new_infrared_view();
        irv_set_enabled(&mut i, false);
        assert!(!i.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_color_map() {
        let i = new_infrared_view();
        let j = irv_to_json(&i);
        assert!(j.contains("\"color_map\""), /* json must contain color_map */);
    }

    #[test]
    fn test_enabled_default() {
        let i = new_infrared_view();
        assert!(i.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_channel_mix_sums_to_one() {
        let i = new_infrared_view();
        let sum: f32 = i.channel_mix.iter().sum();
        assert!((sum - 1.0).abs() < 1e-4, /* channel mix must sum to ~1.0 */);
    }
}
