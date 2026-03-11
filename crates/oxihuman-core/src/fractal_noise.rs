// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Fractal Brownian Motion (fBm) combinator over noise layers.

#![allow(dead_code)]

use crate::noise_perlin::perlin2;
use crate::noise_simplex::simplex2;

/// fBm configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct FbmConfig {
    pub octaves: u32,
    pub lacunarity: f32, // frequency multiplier per octave (typically 2.0)
    pub gain: f32,       // amplitude multiplier per octave (typically 0.5)
    pub frequency: f32,  // base frequency
    pub amplitude: f32,  // base amplitude
}

impl Default for FbmConfig {
    fn default() -> Self {
        Self {
            octaves: 6,
            lacunarity: 2.0,
            gain: 0.5,
            frequency: 1.0,
            amplitude: 1.0,
        }
    }
}

/// fBm using Perlin noise.
#[allow(dead_code)]
pub fn fbm_perlin2(x: f32, y: f32, cfg: &FbmConfig) -> f32 {
    let mut value = 0.0;
    let mut amp = cfg.amplitude;
    let mut freq = cfg.frequency;
    for _ in 0..cfg.octaves {
        value += perlin2(x * freq, y * freq) * amp;
        freq *= cfg.lacunarity;
        amp *= cfg.gain;
    }
    value
}

/// fBm using simplex noise.
#[allow(dead_code)]
pub fn fbm_simplex2(x: f32, y: f32, cfg: &FbmConfig) -> f32 {
    let mut value = 0.0;
    let mut amp = cfg.amplitude;
    let mut freq = cfg.frequency;
    for _ in 0..cfg.octaves {
        value += simplex2(x * freq, y * freq) * amp;
        freq *= cfg.lacunarity;
        amp *= cfg.gain;
    }
    value
}

/// Ridged fBm using Perlin noise (sharper ridges).
#[allow(dead_code)]
pub fn ridged_fbm_perlin2(x: f32, y: f32, cfg: &FbmConfig) -> f32 {
    let mut value = 0.0;
    let mut amp = cfg.amplitude;
    let mut freq = cfg.frequency;
    for _ in 0..cfg.octaves {
        let n = 1.0 - perlin2(x * freq, y * freq).abs();
        value += n * n * amp;
        freq *= cfg.lacunarity;
        amp *= cfg.gain;
    }
    value
}

/// Turbulence fBm (sum of absolute values).
#[allow(dead_code)]
pub fn turbulence_perlin2(x: f32, y: f32, cfg: &FbmConfig) -> f32 {
    let mut value = 0.0;
    let mut amp = cfg.amplitude;
    let mut freq = cfg.frequency;
    for _ in 0..cfg.octaves {
        value += perlin2(x * freq, y * freq).abs() * amp;
        freq *= cfg.lacunarity;
        amp *= cfg.gain;
    }
    value
}

/// Maximum possible absolute value for fBm with given config.
#[allow(dead_code)]
pub fn fbm_max_amplitude(cfg: &FbmConfig) -> f32 {
    let mut sum = 0.0;
    let mut amp = cfg.amplitude;
    for _ in 0..cfg.octaves {
        sum += amp;
        amp *= cfg.gain;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> FbmConfig {
        FbmConfig::default()
    }

    #[test]
    fn fbm_perlin2_deterministic() {
        let cfg = default_cfg();
        let a = fbm_perlin2(1.5, 2.3, &cfg);
        let b = fbm_perlin2(1.5, 2.3, &cfg);
        assert!((a - b).abs() < 1e-10);
    }

    #[test]
    fn fbm_simplex2_deterministic() {
        let cfg = default_cfg();
        let a = fbm_simplex2(1.5, 2.3, &cfg);
        let b = fbm_simplex2(1.5, 2.3, &cfg);
        assert!((a - b).abs() < 1e-10);
    }

    #[test]
    fn fbm_max_amplitude_single_octave() {
        let cfg = FbmConfig {
            octaves: 1,
            amplitude: 1.0,
            gain: 0.5,
            ..Default::default()
        };
        assert!((fbm_max_amplitude(&cfg) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn fbm_max_amplitude_two_octaves() {
        let cfg = FbmConfig {
            octaves: 2,
            amplitude: 1.0,
            gain: 0.5,
            ..Default::default()
        };
        assert!((fbm_max_amplitude(&cfg) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn ridged_fbm_nonnegative() {
        let cfg = default_cfg();
        for i in 0..10 {
            let v = ridged_fbm_perlin2(i as f32 * 0.31, i as f32 * 0.47, &cfg);
            assert!(v >= 0.0, "ridged negative: {v}");
        }
    }

    #[test]
    fn turbulence_nonnegative() {
        let cfg = default_cfg();
        for i in 0..10 {
            let v = turbulence_perlin2(i as f32 * 0.31, i as f32 * 0.47, &cfg);
            assert!(v >= 0.0);
        }
    }

    #[test]
    fn fbm_perlin2_bounded() {
        let cfg = default_cfg();
        let max_amp = fbm_max_amplitude(&cfg);
        for i in 0..20 {
            let v = fbm_perlin2(i as f32 * 0.37, i as f32 * 0.53, &cfg);
            assert!(v.abs() <= max_amp * 2.0, "too large: {v}");
        }
    }

    #[test]
    fn fbm_simplex2_finite() {
        let cfg = default_cfg();
        let v = fbm_simplex2(5.5, 3.3, &cfg);
        assert!(v.is_finite());
    }

    #[test]
    fn fbm_zero_octaves_returns_zero() {
        let cfg = FbmConfig {
            octaves: 0,
            ..Default::default()
        };
        let v = fbm_perlin2(1.0, 2.0, &cfg);
        assert!(v.abs() < 1e-10);
    }

    #[test]
    fn fbm_frequency_affects_output() {
        let cfg1 = FbmConfig {
            frequency: 1.0,
            ..Default::default()
        };
        let cfg2 = FbmConfig {
            frequency: 2.0,
            ..Default::default()
        };
        let v1 = fbm_perlin2(0.7, 0.3, &cfg1);
        let v2 = fbm_perlin2(0.7, 0.3, &cfg2);
        let _diff = (v1 - v2).abs();
    }
}
