// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Film grain post-processing effect — deterministic grain pattern
//! using hash-based noise with configurable intensity and luminance response.

/// Film grain configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct FilmGrainConfig {
    /// Grain intensity, 0..=1.
    pub intensity: f32,
    /// Grain size in pixels.
    pub grain_size: f32,
    /// Luminance response: higher = more grain in shadows, less in highlights.
    pub luminance_response: f32,
    /// Colour grain (vs monochrome), 0..=1.
    pub color_amount: f32,
    /// Time seed for temporal variation (deterministic per frame).
    pub time_seed: u32,
}

impl Default for FilmGrainConfig {
    fn default() -> Self {
        Self {
            intensity: 0.05,
            grain_size: 1.0,
            luminance_response: 0.5,
            color_amount: 0.0,
            time_seed: 0,
        }
    }
}

/// Integer hash for deterministic noise.
#[allow(dead_code)]
pub fn hash_u32(mut x: u32) -> u32 {
    x = x.wrapping_mul(0x9E3779B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EBCA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2AE35);
    x ^= x >> 16;
    x
}

/// Convert hash to float in 0..1.
#[allow(dead_code)]
pub fn hash_to_float(hash: u32) -> f32 {
    (hash & 0x00FFFFFF) as f32 / 16777215.0
}

/// Generate grain value for a pixel.
///
/// Returns a value centred around 0.0 (-0.5..0.5).
#[allow(dead_code)]
pub fn grain_value(x: u32, y: u32, seed: u32) -> f32 {
    let h = hash_u32(
        x.wrapping_mul(374761393).wrapping_add(
            y.wrapping_mul(668265263)
                .wrapping_add(seed.wrapping_mul(1013904223)),
        ),
    );
    hash_to_float(h) - 0.5
}

/// Luminance-dependent grain scaling.
///
/// More grain in shadows when `response > 0`.
#[allow(dead_code)]
pub fn luminance_scale(luminance: f32, response: f32) -> f32 {
    let l = luminance.clamp(0.0, 1.0);
    let r = response.clamp(0.0, 1.0);
    // Blend between uniform (r=0) and shadow-heavy (r=1)
    1.0 - r * l
}

/// Apply monochrome grain to a pixel.
#[allow(dead_code)]
pub fn apply_mono_grain(color: [f32; 3], x: u32, y: u32, config: &FilmGrainConfig) -> [f32; 3] {
    let lum = 0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2];
    let grain = grain_value(x, y, config.time_seed);
    let scale = luminance_scale(lum, config.luminance_response);
    let offset = grain * config.intensity * scale;
    [
        (color[0] + offset).clamp(0.0, 1.0),
        (color[1] + offset).clamp(0.0, 1.0),
        (color[2] + offset).clamp(0.0, 1.0),
    ]
}

/// Apply colour grain to a pixel.
#[allow(dead_code)]
pub fn apply_color_grain(color: [f32; 3], x: u32, y: u32, config: &FilmGrainConfig) -> [f32; 3] {
    let lum = 0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2];
    let scale = luminance_scale(lum, config.luminance_response);

    let gr = grain_value(x, y, config.time_seed);
    let gg = grain_value(x.wrapping_add(1000), y, config.time_seed);
    let gb = grain_value(x, y.wrapping_add(1000), config.time_seed);

    let mono_w = 1.0 - config.color_amount;
    let col_w = config.color_amount;

    [
        (color[0] + (gr * col_w + gr * mono_w) * config.intensity * scale).clamp(0.0, 1.0),
        (color[1] + (gg * col_w + gr * mono_w) * config.intensity * scale).clamp(0.0, 1.0),
        (color[2] + (gb * col_w + gr * mono_w) * config.intensity * scale).clamp(0.0, 1.0),
    ]
}

/// Apply film grain (auto-selects mono or colour based on config).
#[allow(dead_code)]
pub fn apply_film_grain(color: [f32; 3], x: u32, y: u32, config: &FilmGrainConfig) -> [f32; 3] {
    if config.color_amount < 0.01 {
        apply_mono_grain(color, x, y, config)
    } else {
        apply_color_grain(color, x, y, config)
    }
}

/// Estimate the perceived grain noise level.
#[allow(dead_code)]
pub fn estimate_noise_level(config: &FilmGrainConfig) -> f32 {
    // RMS of uniform distribution * intensity
    let rms = 1.0 / (12.0_f32).sqrt(); // ~0.289
    config.intensity * rms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = FilmGrainConfig::default();
        assert!((0.0..=1.0).contains(&c.intensity));
    }

    #[test]
    fn test_hash_deterministic() {
        let h1 = hash_u32(42);
        let h2 = hash_u32(42);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_to_float_range() {
        for i in 0..100 {
            let v = hash_to_float(hash_u32(i));
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn test_grain_value_range() {
        for i in 0..50 {
            let g = grain_value(i, i * 7, 0);
            assert!((-0.5..=0.5).contains(&g));
        }
    }

    #[test]
    fn test_luminance_scale_bright() {
        let s = luminance_scale(1.0, 1.0);
        assert!(s.abs() < 1e-5, "Bright pixels should have minimal grain");
    }

    #[test]
    fn test_luminance_scale_dark() {
        let s = luminance_scale(0.0, 1.0);
        assert!((s - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_apply_mono_clamped() {
        let c = FilmGrainConfig {
            intensity: 0.5,
            ..Default::default()
        };
        let result = apply_mono_grain([0.0, 0.0, 0.0], 0, 0, &c);
        for channel in &result {
            assert!((0.0..=1.0).contains(channel));
        }
    }

    #[test]
    fn test_apply_film_grain_mono() {
        let c = FilmGrainConfig {
            color_amount: 0.0,
            ..Default::default()
        };
        let r = apply_film_grain([0.5, 0.5, 0.5], 10, 10, &c);
        // All channels should shift equally (mono)
        let diff01 = (r[0] - r[1]).abs();
        let diff12 = (r[1] - r[2]).abs();
        assert!(diff01 < 1e-5);
        assert!(diff12 < 1e-5);
    }

    #[test]
    fn test_estimate_noise_level() {
        let c = FilmGrainConfig {
            intensity: 1.0,
            ..Default::default()
        };
        let n = estimate_noise_level(&c);
        assert!(n > 0.0 && n < 1.0);
    }

    #[test]
    fn test_zero_intensity_no_change() {
        let c = FilmGrainConfig {
            intensity: 0.0,
            ..Default::default()
        };
        let input = [0.5, 0.3, 0.7];
        let result = apply_film_grain(input, 5, 5, &c);
        for i in 0..3 {
            assert!((result[i] - input[i]).abs() < 1e-5);
        }
    }
}
