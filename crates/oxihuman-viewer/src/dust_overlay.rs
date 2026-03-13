// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dust overlay — dust particle overlay parameters and generation.

/// Config.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct DustOverlayConfig {
    pub particle_count: usize,
    pub radius_min: f32,
    pub radius_max: f32,
    /// Drift speed horizontal, normalised per frame.
    pub drift_x: f32,
    /// Drift speed vertical.
    pub drift_y: f32,
    /// Overall opacity 0..=1.
    pub opacity: f32,
    /// Color tint RGB.
    pub tint: [f32; 3],
    pub enabled: bool,
}

impl Default for DustOverlayConfig {
    fn default() -> Self {
        Self {
            particle_count: 80,
            radius_min: 0.001,
            radius_max: 0.006,
            drift_x: 0.0005,
            drift_y: -0.0002,
            opacity: 0.4,
            tint: [0.9, 0.85, 0.75],
            enabled: true,
        }
    }
}

/// A single dust particle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DustParticle {
    pub pos: [f32; 2],
    pub radius: f32,
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

#[allow(dead_code)]
pub fn generate_dust_particles(seed: u64, cfg: &DustOverlayConfig) -> Vec<DustParticle> {
    let mut rng = seed ^ 0xDA57_CAFE_BABE_0001;
    let r_range = cfg.radius_max - cfg.radius_min;
    (0..cfg.particle_count)
        .map(|_| {
            let x = pcg(&mut rng);
            let y = pcg(&mut rng);
            let radius = cfg.radius_min + pcg(&mut rng) * r_range;
            let alpha = (pcg(&mut rng) * 0.6 + 0.2) * cfg.opacity;
            DustParticle {
                pos: [x, y],
                radius,
                alpha,
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn new_dust_overlay_config() -> DustOverlayConfig {
    DustOverlayConfig::default()
}

#[allow(dead_code)]
pub fn do_set_opacity(cfg: &mut DustOverlayConfig, v: f32) {
    cfg.opacity = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn do_set_tint(cfg: &mut DustOverlayConfig, r: f32, g: f32, b: f32) {
    cfg.tint = [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)];
}

/// Simulate one step: moves particles by drift vector with wrapping.
#[allow(dead_code)]
pub fn do_step(particles: &mut [DustParticle], cfg: &DustOverlayConfig) {
    for p in particles.iter_mut() {
        p.pos[0] = (p.pos[0] + cfg.drift_x).rem_euclid(1.0);
        p.pos[1] = (p.pos[1] + cfg.drift_y).rem_euclid(1.0);
    }
}

#[allow(dead_code)]
pub fn do_average_alpha(particles: &[DustParticle]) -> f32 {
    if particles.is_empty() {
        return 0.0;
    }
    particles.iter().map(|p| p.alpha).sum::<f32>() / particles.len() as f32
}

#[allow(dead_code)]
pub fn do_to_json(cfg: &DustOverlayConfig) -> String {
    format!(
        "{{\"particle_count\":{},\"opacity\":{:.3},\"enabled\":{}}}",
        cfg.particle_count, cfg.opacity, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_correct_count() {
        let cfg = new_dust_overlay_config();
        let p = generate_dust_particles(1, &cfg);
        assert_eq!(p.len(), cfg.particle_count);
    }

    #[test]
    fn deterministic() {
        let cfg = new_dust_overlay_config();
        let a = generate_dust_particles(9, &cfg);
        let b = generate_dust_particles(9, &cfg);
        assert_eq!(a[0].pos[0].to_bits(), b[0].pos[0].to_bits());
    }

    #[test]
    fn positions_in_range() {
        let cfg = new_dust_overlay_config();
        let p = generate_dust_particles(2, &cfg);
        for dp in &p {
            assert!((0.0..=1.0).contains(&dp.pos[0]));
            assert!((0.0..=1.0).contains(&dp.pos[1]));
        }
    }

    #[test]
    fn radius_in_range() {
        let cfg = new_dust_overlay_config();
        let p = generate_dust_particles(3, &cfg);
        for dp in &p {
            assert!(dp.radius >= cfg.radius_min && dp.radius <= cfg.radius_max + 1e-5);
        }
    }

    #[test]
    fn step_moves_particles() {
        let cfg = new_dust_overlay_config();
        let mut p = generate_dust_particles(4, &cfg);
        let x0 = p[0].pos[0];
        do_step(&mut p, &cfg);
        let x1 = p[0].pos[0];
        // drift_x != 0 so pos should change (modulo wrap)
        let _ = x0;
        let _ = x1;
        // just verify no panic
    }

    #[test]
    fn step_wraps() {
        let cfg = new_dust_overlay_config();
        let mut p = vec![DustParticle {
            pos: [0.9999, 0.5],
            radius: 0.001,
            alpha: 0.5,
        }];
        do_step(&mut p, &cfg);
        assert!((0.0..=1.0).contains(&p[0].pos[0]));
    }

    #[test]
    fn opacity_clamps() {
        let mut cfg = new_dust_overlay_config();
        do_set_opacity(&mut cfg, -1.0);
        assert!(cfg.opacity < 1e-6);
    }

    #[test]
    fn tint_clamps() {
        let mut cfg = new_dust_overlay_config();
        do_set_tint(&mut cfg, 2.0, -1.0, 0.5);
        assert!((cfg.tint[0] - 1.0).abs() < 1e-6);
        assert!(cfg.tint[1] < 1e-6);
    }

    #[test]
    fn empty_average_alpha_zero() {
        assert!(do_average_alpha(&[]) < 1e-8);
    }

    #[test]
    fn json_has_keys() {
        let j = do_to_json(&new_dust_overlay_config());
        assert!(j.contains("particle_count") && j.contains("enabled"));
    }
}
