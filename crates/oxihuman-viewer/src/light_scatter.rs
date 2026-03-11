// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Light scatter — volumetric light scattering (god ray) post-process effect.

use std::f32::consts::PI;

/// Configuration for light scatter.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightScatterConfig {
    pub num_samples: u32,
    pub density: f32,
    pub weight: f32,
    pub decay: f32,
    pub exposure: f32,
    pub enabled: bool,
}

#[allow(dead_code)]
pub fn default_light_scatter_config() -> LightScatterConfig {
    LightScatterConfig {
        num_samples: 100,
        density: 0.96,
        weight: 0.4,
        decay: 0.98,
        exposure: 0.2,
        enabled: true,
    }
}

#[allow(dead_code)]
pub fn lsc_set_samples(cfg: &mut LightScatterConfig, n: u32) {
    cfg.num_samples = n.clamp(8, 512);
}

#[allow(dead_code)]
pub fn lsc_set_density(cfg: &mut LightScatterConfig, v: f32) {
    cfg.density = v.clamp(0.0, 1.0);
}

#[allow(dead_code)]
pub fn lsc_set_enabled(cfg: &mut LightScatterConfig, enabled: bool) {
    cfg.enabled = enabled;
}

#[allow(dead_code)]
pub fn lsc_set_exposure(cfg: &mut LightScatterConfig, v: f32) {
    cfg.exposure = v.max(0.0);
}

/// Approximate radial scatter weight for a sample at normalized distance `r`.
#[allow(dead_code)]
pub fn lsc_radial_weight(cfg: &LightScatterConfig, r: f32) -> f32 {
    cfg.weight * cfg.decay.powf(r * cfg.num_samples as f32)
}

#[allow(dead_code)]
pub fn lsc_mie_phase(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = 1.0 + g2 - 2.0 * g * cos_theta;
    (1.0 - g2) / (4.0 * PI * denom.powf(1.5))
}

#[allow(dead_code)]
pub fn lsc_blend(a: &LightScatterConfig, b: &LightScatterConfig, t: f32) -> LightScatterConfig {
    let t = t.clamp(0.0, 1.0);
    LightScatterConfig {
        num_samples: a.num_samples,
        density: a.density + (b.density - a.density) * t,
        weight: a.weight + (b.weight - a.weight) * t,
        decay: a.decay + (b.decay - a.decay) * t,
        exposure: a.exposure + (b.exposure - a.exposure) * t,
        enabled: a.enabled,
    }
}

#[allow(dead_code)]
pub fn lsc_to_json(cfg: &LightScatterConfig) -> String {
    format!(
        r#"{{"samples":{},"density":{:.4},"weight":{:.4},"decay":{:.4},"enabled":{}}}"#,
        cfg.num_samples, cfg.density, cfg.weight, cfg.decay, cfg.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = default_light_scatter_config();
        assert_eq!(cfg.num_samples, 100);
    }

    #[test]
    fn set_samples_clamps() {
        let mut cfg = default_light_scatter_config();
        lsc_set_samples(&mut cfg, 1);
        assert_eq!(cfg.num_samples, 8);
    }

    #[test]
    fn set_density_clamps() {
        let mut cfg = default_light_scatter_config();
        lsc_set_density(&mut cfg, 2.0);
        assert!((cfg.density - 1.0).abs() < 1e-6);
    }

    #[test]
    fn disabled_no_effect() {
        let mut cfg = default_light_scatter_config();
        lsc_set_enabled(&mut cfg, false);
        assert!(!cfg.enabled);
    }

    #[test]
    fn radial_weight_at_zero() {
        let cfg = default_light_scatter_config();
        let w = lsc_radial_weight(&cfg, 0.0);
        assert!((w - cfg.weight).abs() < 1e-6);
    }

    #[test]
    fn mie_phase_positive() {
        let p = lsc_mie_phase(1.0, 0.8);
        assert!(p > 0.0);
    }

    #[test]
    fn mie_phase_forward_scatter_greater() {
        let fwd = lsc_mie_phase(1.0, 0.8);
        let back = lsc_mie_phase(-1.0, 0.8);
        assert!(fwd > back);
    }

    #[test]
    fn blend_midpoint() {
        let a = default_light_scatter_config();
        let mut b = default_light_scatter_config();
        lsc_set_density(&mut b, 0.5);
        let m = lsc_blend(&a, &b, 0.5);
        assert!((m.density - (a.density + 0.5) * 0.5).abs() < 1e-5);
    }

    #[test]
    fn to_json_fields() {
        let cfg = default_light_scatter_config();
        let j = lsc_to_json(&cfg);
        assert!(j.contains("density"));
    }
}
