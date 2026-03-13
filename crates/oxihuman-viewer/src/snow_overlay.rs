// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Snow overlay — snowflake particle overlay data and generation.

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SnowOverlayConfig {
    /// Number of snowflakes.
    pub flake_count: usize,
    /// Min/max flake radius in normalised screen units.
    pub radius_min: f32,
    pub radius_max: f32,
    /// Drift speed (horizontal wander rate), normalised per frame.
    pub drift: f32,
    /// Fall speed, normalised per frame.
    pub fall_speed: f32,
    /// Opacity 0..=1.
    pub opacity: f32,
    pub enabled: bool,
}

impl Default for SnowOverlayConfig {
    fn default() -> Self {
        Self {
            flake_count: 150,
            radius_min: 0.002,
            radius_max: 0.008,
            drift: 0.001,
            fall_speed: 0.003,
            opacity: 0.8,
            enabled: true,
        }
    }
}

/// A single snowflake particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnowFlake {
    /// Screen-space position (x, y) 0..=1.
    pub pos: [f32; 2],
    /// Radius in normalised screen units.
    pub radius: f32,
    /// Phase offset for drift oscillation 0..=1.
    pub phase: f32,
    /// Alpha 0..=1.
    pub alpha: f32,
}

fn pcg(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let x = ((*state >> 18) ^ *state) >> 27;
    let rot = (*state >> 59) as u32;
    let v = (x as u32).rotate_right(rot);
    (v >> 8) as f32 / 16_777_216.0
}

/// Generate snowflake particles from a seed and config.
#[allow(dead_code)]
pub fn generate_snowflakes(seed: u64, cfg: &SnowOverlayConfig) -> Vec<SnowFlake> {
    let mut rng = seed ^ 0xD1CE_BEEF_5A1A_0001;
    let r_range = cfg.radius_max - cfg.radius_min;
    (0..cfg.flake_count)
        .map(|_| {
            let x = pcg(&mut rng);
            let y = pcg(&mut rng);
            let radius = cfg.radius_min + pcg(&mut rng) * r_range;
            let phase = pcg(&mut rng);
            let alpha = (pcg(&mut rng) * 0.4 + 0.6) * cfg.opacity;
            SnowFlake {
                pos: [x, y],
                radius,
                phase,
                alpha,
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn new_snow_overlay_config() -> SnowOverlayConfig {
    SnowOverlayConfig::default()
}

#[allow(dead_code)]
pub fn so_set_count(cfg: &mut SnowOverlayConfig, count: usize) {
    cfg.flake_count = count;
}

#[allow(dead_code)]
pub fn so_set_opacity(cfg: &mut SnowOverlayConfig, v: f32) {
    cfg.opacity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn so_set_fall_speed(cfg: &mut SnowOverlayConfig, v: f32) {
    cfg.fall_speed = v.clamp(0.0, 1.0);
}

/// Simulate one step: moves flake positions by fall_speed downward.
#[allow(dead_code)]
pub fn so_step(flakes: &mut [SnowFlake], cfg: &SnowOverlayConfig) {
    for f in flakes.iter_mut() {
        f.pos[1] += cfg.fall_speed;
        if f.pos[1] > 1.0 {
            f.pos[1] -= 1.0;
        }
    }
}

#[allow(dead_code)]
pub fn so_average_radius(flakes: &[SnowFlake]) -> f32 {
    if flakes.is_empty() {
        return 0.0;
    }
    flakes.iter().map(|f| f.radius).sum::<f32>() / flakes.len() as f32
}

#[allow(dead_code)]
pub fn so_to_json(cfg: &SnowOverlayConfig) -> String {
    format!(
        "{{\"flake_count\":{},\"opacity\":{:.3},\"enabled\":{}}}",
        cfg.flake_count, cfg.opacity, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_correct_count() {
        let cfg = new_snow_overlay_config();
        let flakes = generate_snowflakes(42, &cfg);
        assert_eq!(flakes.len(), cfg.flake_count);
    }

    #[test]
    fn deterministic() {
        let cfg = new_snow_overlay_config();
        let a = generate_snowflakes(99, &cfg);
        let b = generate_snowflakes(99, &cfg);
        assert_eq!(a[0].pos[0].to_bits(), b[0].pos[0].to_bits());
    }

    #[test]
    fn positions_in_range() {
        let cfg = new_snow_overlay_config();
        let flakes = generate_snowflakes(7, &cfg);
        for f in &flakes {
            assert!((0.0..=1.0).contains(&f.pos[0]));
            assert!((0.0..=1.0).contains(&f.pos[1]));
        }
    }

    #[test]
    fn radius_in_config_range() {
        let cfg = new_snow_overlay_config();
        let flakes = generate_snowflakes(1, &cfg);
        for f in &flakes {
            assert!(f.radius >= cfg.radius_min && f.radius <= cfg.radius_max + 1e-5);
        }
    }

    #[test]
    fn step_advances_y() {
        let cfg = new_snow_overlay_config();
        let mut flakes = generate_snowflakes(3, &cfg);
        let y0 = flakes[0].pos[1];
        so_step(&mut flakes, &cfg);
        let y1 = flakes[0].pos[1];
        // wrap-around check: either increased or wrapped
        assert!(y1 != y0 || cfg.fall_speed < 1e-8);
    }

    #[test]
    fn step_wraps_around() {
        let cfg = new_snow_overlay_config();
        let mut flakes = vec![SnowFlake {
            pos: [0.5, 0.99],
            radius: 0.004,
            phase: 0.0,
            alpha: 1.0,
        }];
        so_step(&mut flakes, &cfg);
        assert!(flakes[0].pos[1] <= 1.0);
    }

    #[test]
    fn opacity_clamps() {
        let mut cfg = new_snow_overlay_config();
        so_set_opacity(&mut cfg, 5.0);
        assert!((cfg.opacity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn average_radius_in_range() {
        let cfg = new_snow_overlay_config();
        let flakes = generate_snowflakes(5, &cfg);
        let avg = so_average_radius(&flakes);
        assert!(avg >= cfg.radius_min && avg <= cfg.radius_max + 1e-5);
    }

    #[test]
    fn empty_average_radius_zero() {
        assert!(so_average_radius(&[]) < 1e-8);
    }

    #[test]
    fn json_has_keys() {
        let j = so_to_json(&new_snow_overlay_config());
        assert!(j.contains("flake_count") && j.contains("enabled"));
    }
}
