// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Night vision effect overlay stub.

/// Night vision generation type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NvGeneration {
    Gen1,
    Gen2,
    Gen3,
}

/// Night vision view configuration.
#[derive(Debug, Clone)]
pub struct NightVisionView {
    pub generation: NvGeneration,
    pub gain: f32,
    pub phosphor_green: bool,
    pub vignette_strength: f32,
    pub noise_intensity: f32,
    pub enabled: bool,
}

impl NightVisionView {
    pub fn new() -> Self {
        NightVisionView {
            generation: NvGeneration::Gen2,
            gain: 10.0,
            phosphor_green: true,
            vignette_strength: 0.6,
            noise_intensity: 0.08,
            enabled: true,
        }
    }
}

impl Default for NightVisionView {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new night vision view.
pub fn new_night_vision_view() -> NightVisionView {
    NightVisionView::new()
}

/// Apply night vision effect to an RGB pixel (stub).
pub fn nvv_apply_pixel(nvv: &NightVisionView, rgb: [f32; 3]) -> [f32; 3] {
    /* Stub: brightens and optionally tints green */
    let lum = (rgb[0] * 0.299 + rgb[1] * 0.587 + rgb[2] * 0.114) * nvv.gain;
    let lum = lum.min(1.0);
    if nvv.phosphor_green {
        [0.0, lum, 0.0]
    } else {
        [lum; 3]
    }
}

/// Set generation.
pub fn nvv_set_generation(nvv: &mut NightVisionView, generation: NvGeneration) {
    nvv.generation = generation;
}

/// Set gain.
pub fn nvv_set_gain(nvv: &mut NightVisionView, gain: f32) {
    nvv.gain = gain.max(1.0);
}

/// Enable or disable.
pub fn nvv_set_enabled(nvv: &mut NightVisionView, enabled: bool) {
    nvv.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn nvv_to_json(nvv: &NightVisionView) -> String {
    let gen = match nvv.generation {
        NvGeneration::Gen1 => "gen1",
        NvGeneration::Gen2 => "gen2",
        NvGeneration::Gen3 => "gen3",
    };
    format!(
        r#"{{"generation":"{}","gain":{},"phosphor_green":{},"enabled":{}}}"#,
        gen, nvv.gain, nvv.phosphor_green, nvv.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_generation_gen2() {
        let n = new_night_vision_view();
        assert_eq!(
            n.generation,
            NvGeneration::Gen2, /* default generation must be Gen2 */
        );
    }

    #[test]
    fn test_phosphor_green_by_default() {
        let n = new_night_vision_view();
        assert!(n.phosphor_green, /* phosphor green must be on by default */);
    }

    #[test]
    fn test_apply_pixel_green() {
        let n = new_night_vision_view();
        let out = nvv_apply_pixel(&n, [0.05, 0.05, 0.05]);
        assert!((out[0]).abs() < 1e-6, /* green mode must have zero red */);
        assert!((out[2]).abs() < 1e-6, /* green mode must have zero blue */);
    }

    #[test]
    fn test_apply_pixel_white_not_green() {
        let mut n = new_night_vision_view();
        n.phosphor_green = false;
        let out = nvv_apply_pixel(&n, [0.05; 3]);
        assert!((out[0] - out[1]).abs() < 1e-5, /* non-green mode must have equal channels */);
    }

    #[test]
    fn test_set_generation() {
        let mut n = new_night_vision_view();
        nvv_set_generation(&mut n, NvGeneration::Gen3);
        assert_eq!(
            n.generation,
            NvGeneration::Gen3, /* generation must be set */
        );
    }

    #[test]
    fn test_set_gain() {
        let mut n = new_night_vision_view();
        nvv_set_gain(&mut n, 20.0);
        assert!((n.gain - 20.0).abs() < 1e-5 /* gain must be set */,);
    }

    #[test]
    fn test_gain_clamped_minimum() {
        let mut n = new_night_vision_view();
        nvv_set_gain(&mut n, 0.1);
        assert!((n.gain - 1.0).abs() < 1e-5, /* gain clamped to 1.0 minimum */);
    }

    #[test]
    fn test_set_enabled() {
        let mut n = new_night_vision_view();
        nvv_set_enabled(&mut n, false);
        assert!(!n.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_generation() {
        let n = new_night_vision_view();
        let j = nvv_to_json(&n);
        assert!(j.contains("\"generation\""), /* json must contain generation */);
    }

    #[test]
    fn test_enabled_default() {
        let n = new_night_vision_view();
        assert!(n.enabled /* must be enabled by default */,);
    }
}
