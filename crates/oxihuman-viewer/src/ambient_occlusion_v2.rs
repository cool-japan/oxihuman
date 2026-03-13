// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Improved ambient-occlusion (SSAO v2) configuration and sampling helpers.

use std::f32::consts::PI;

/// AO technique variant.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AoTechnique {
    Ssao,
    Hbao,
    Gtao,
}

/// Configuration for AO v2.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct AoV2Config {
    pub technique: AoTechnique,
    pub radius: f32,
    pub bias: f32,
    pub intensity: f32,
    pub sample_count: u32,
    pub blur_passes: u32,
}

impl Default for AoV2Config {
    fn default() -> Self {
        Self {
            technique: AoTechnique::Ssao,
            radius: 0.5,
            bias: 0.025,
            intensity: 1.0,
            sample_count: 16,
            blur_passes: 2,
        }
    }
}

#[allow(dead_code)]
pub fn new_ao_v2_config() -> AoV2Config {
    AoV2Config::default()
}

#[allow(dead_code)]
pub fn ao_v2_set_technique(cfg: &mut AoV2Config, t: AoTechnique) {
    cfg.technique = t;
}

#[allow(dead_code)]
pub fn ao_v2_set_radius(cfg: &mut AoV2Config, r: f32) {
    cfg.radius = r.max(0.01);
}

#[allow(dead_code)]
pub fn ao_v2_set_intensity(cfg: &mut AoV2Config, v: f32) {
    cfg.intensity = v.clamp(0.0, 4.0);
}

#[allow(dead_code)]
pub fn ao_v2_set_samples(cfg: &mut AoV2Config, n: u32) {
    cfg.sample_count = n.clamp(4, 64);
}

/// Cosine-weighted hemisphere sample at angle index i out of n (uses PI).
#[allow(dead_code)]
pub fn ao_v2_hemisphere_dir(i: u32, n: u32) -> [f32; 3] {
    if n == 0 {
        return [0.0, 1.0, 0.0];
    }
    let t = i as f32 / n as f32;
    let phi = t * 2.0 * PI;
    let cos_theta = (1.0 - t).sqrt();
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    [sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin()]
}

#[allow(dead_code)]
pub fn ao_v2_technique_name(t: AoTechnique) -> &'static str {
    match t {
        AoTechnique::Ssao => "ssao",
        AoTechnique::Hbao => "hbao",
        AoTechnique::Gtao => "gtao",
    }
}

#[allow(dead_code)]
pub fn ao_v2_estimated_cost(cfg: &AoV2Config) -> f32 {
    cfg.sample_count as f32 * (1.0 + cfg.blur_passes as f32 * 0.2)
}

#[allow(dead_code)]
pub fn ao_v2_to_json(cfg: &AoV2Config) -> String {
    format!(
        "{{\"technique\":\"{}\",\"radius\":{:.4},\"intensity\":{:.4},\"samples\":{}}}",
        ao_v2_technique_name(cfg.technique),
        cfg.radius,
        cfg.intensity,
        cfg.sample_count
    )
}

#[allow(dead_code)]
pub fn ao_v2_blend_configs(a: &AoV2Config, b: &AoV2Config, t: f32) -> AoV2Config {
    let t = t.clamp(0.0, 1.0);
    AoV2Config {
        technique: if t < 0.5 { a.technique } else { b.technique },
        radius: a.radius + (b.radius - a.radius) * t,
        bias: a.bias + (b.bias - a.bias) * t,
        intensity: a.intensity + (b.intensity - a.intensity) * t,
        sample_count: if t < 0.5 {
            a.sample_count
        } else {
            b.sample_count
        },
        blur_passes: if t < 0.5 {
            a.blur_passes
        } else {
            b.blur_passes
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_technique_ssao() {
        assert_eq!(new_ao_v2_config().technique, AoTechnique::Ssao);
    }

    #[test]
    fn radius_min_clamp() {
        let mut cfg = new_ao_v2_config();
        ao_v2_set_radius(&mut cfg, -1.0);
        assert!(cfg.radius >= 0.01);
    }

    #[test]
    fn intensity_clamp() {
        let mut cfg = new_ao_v2_config();
        ao_v2_set_intensity(&mut cfg, 10.0);
        assert!(cfg.intensity <= 4.0);
    }

    #[test]
    fn sample_count_clamp_min() {
        let mut cfg = new_ao_v2_config();
        ao_v2_set_samples(&mut cfg, 1);
        assert!(cfg.sample_count >= 4);
    }

    #[test]
    fn hemisphere_dir_len() {
        let d = ao_v2_hemisphere_dir(0, 16);
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-4);
    }

    #[test]
    fn technique_names() {
        assert_eq!(ao_v2_technique_name(AoTechnique::Gtao), "gtao");
    }

    #[test]
    fn cost_positive() {
        assert!(ao_v2_estimated_cost(&new_ao_v2_config()) > 0.0);
    }

    #[test]
    fn json_has_technique() {
        assert!(ao_v2_to_json(&new_ao_v2_config()).contains("technique"));
    }

    #[test]
    fn blend_t0_is_a() {
        let a = new_ao_v2_config();
        let mut b = new_ao_v2_config();
        ao_v2_set_intensity(&mut b, 2.0);
        let r = ao_v2_blend_configs(&a, &b, 0.0);
        assert!((r.intensity - a.intensity).abs() < 1e-5);
    }

    #[test]
    fn zero_samples_hemisphere_safe() {
        let d = ao_v2_hemisphere_dir(0, 0);
        assert!((d[1] - 1.0).abs() < 1e-5);
    }
}
