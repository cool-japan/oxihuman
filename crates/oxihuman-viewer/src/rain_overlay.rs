// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rain overlay — rain streak overlay parameters and generation.

use std::f32::consts::PI;

/// Config for the rain overlay effect.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct RainOverlayConfig {
    /// Number of rain streaks.
    pub streak_count: usize,
    /// Streak length in normalised screen coordinates (0..=1).
    pub streak_length: f32,
    /// Wind angle in radians (0 = straight down, positive = rightward slant).
    pub wind_angle_rad: f32,
    /// Streak opacity 0..=1.
    pub opacity: f32,
    /// Rain intensity 0..=1.
    pub intensity: f32,
}

impl Default for RainOverlayConfig {
    fn default() -> Self {
        Self {
            streak_count: 200,
            streak_length: 0.05,
            wind_angle_rad: PI / 16.0,
            opacity: 0.35,
            intensity: 0.6,
        }
    }
}

/// A single rain streak.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RainStreak {
    /// Screen-space start position (x, y) in 0..=1.
    pub start: [f32; 2],
    /// Screen-space end position.
    pub end: [f32; 2],
    /// Alpha 0..=1.
    pub alpha: f32,
}

/// Lightweight PCG-style PRNG for deterministic generation.
fn pcg(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let x = ((*state >> 18) ^ *state) >> 27;
    let rot = (*state >> 59) as u32;
    let v = (x as u32).rotate_right(rot);
    (v >> 8) as f32 / 16_777_216.0
}

/// Generate rain streaks from a seed and config.
#[allow(dead_code)]
pub fn generate_rain_streaks(seed: u64, cfg: &RainOverlayConfig) -> Vec<RainStreak> {
    let mut rng = seed ^ 0xA1A2_B3B4_C5C6_D7D8;
    let sin_a = cfg.wind_angle_rad.sin();
    let cos_a = cfg.wind_angle_rad.cos();
    (0..cfg.streak_count)
        .map(|_| {
            let x = pcg(&mut rng);
            let y = pcg(&mut rng);
            let alpha = (pcg(&mut rng) * 0.5 + 0.5) * cfg.opacity;
            let dx = sin_a * cfg.streak_length;
            let dy = cos_a * cfg.streak_length;
            RainStreak {
                start: [x, y],
                end: [x + dx, y + dy],
                alpha,
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn new_rain_overlay_config() -> RainOverlayConfig {
    RainOverlayConfig::default()
}

#[allow(dead_code)]
pub fn ro_set_intensity(cfg: &mut RainOverlayConfig, v: f32) {
    cfg.intensity = v.clamp(0.0, 1.0);
    cfg.streak_count = (v.clamp(0.0, 1.0) * 400.0) as usize;
}

#[allow(dead_code)]
pub fn ro_set_wind_angle_deg(cfg: &mut RainOverlayConfig, deg: f32) {
    cfg.wind_angle_rad = deg.to_radians();
}

#[allow(dead_code)]
pub fn ro_set_opacity(cfg: &mut RainOverlayConfig, v: f32) {
    cfg.opacity = v.clamp(0.0, 1.0);
}

/// Average streak length (screen diag).
#[allow(dead_code)]
pub fn ro_avg_streak_diag(cfg: &RainOverlayConfig) -> f32 {
    cfg.streak_length * (cfg.wind_angle_rad.sin().powi(2) + cfg.wind_angle_rad.cos().powi(2)).sqrt()
}

#[allow(dead_code)]
pub fn ro_to_json(cfg: &RainOverlayConfig) -> String {
    format!(
        "{{\"streak_count\":{},\"intensity\":{:.3},\"opacity\":{:.3}}}",
        cfg.streak_count, cfg.intensity, cfg.opacity
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_correct_count() {
        let cfg = new_rain_overlay_config();
        let streaks = generate_rain_streaks(42, &cfg);
        assert_eq!(streaks.len(), cfg.streak_count);
    }

    #[test]
    fn deterministic_generation() {
        let cfg = new_rain_overlay_config();
        let a = generate_rain_streaks(7, &cfg);
        let b = generate_rain_streaks(7, &cfg);
        assert_eq!(a[0].start[0].to_bits(), b[0].start[0].to_bits());
    }

    #[test]
    fn different_seeds_differ() {
        let cfg = new_rain_overlay_config();
        let a = generate_rain_streaks(1, &cfg);
        let b = generate_rain_streaks(2, &cfg);
        assert_ne!(a[0].start[0].to_bits(), b[0].start[0].to_bits());
    }

    #[test]
    fn start_positions_in_range() {
        let cfg = new_rain_overlay_config();
        let streaks = generate_rain_streaks(3, &cfg);
        for s in &streaks {
            assert!((0.0..=1.0).contains(&s.start[0]));
            assert!((0.0..=1.0).contains(&s.start[1]));
        }
    }

    #[test]
    fn alpha_nonzero() {
        let cfg = new_rain_overlay_config();
        let streaks = generate_rain_streaks(5, &cfg);
        assert!(streaks.iter().any(|s| s.alpha > 0.0));
    }

    #[test]
    fn set_opacity_clamps() {
        let mut cfg = new_rain_overlay_config();
        ro_set_opacity(&mut cfg, 5.0);
        assert!((cfg.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn set_intensity_clamps() {
        let mut cfg = new_rain_overlay_config();
        ro_set_intensity(&mut cfg, -1.0);
        assert!(cfg.intensity < 1e-6);
    }

    #[test]
    fn wind_angle_deg_converts() {
        let mut cfg = new_rain_overlay_config();
        ro_set_wind_angle_deg(&mut cfg, 45.0);
        assert!((cfg.wind_angle_rad - 45.0_f32.to_radians()).abs() < 1e-5);
    }

    #[test]
    fn avg_streak_diag_positive() {
        let cfg = new_rain_overlay_config();
        assert!(ro_avg_streak_diag(&cfg) > 0.0);
    }

    #[test]
    fn json_has_keys() {
        let j = ro_to_json(&new_rain_overlay_config());
        assert!(j.contains("streak_count") && j.contains("intensity"));
    }
}
