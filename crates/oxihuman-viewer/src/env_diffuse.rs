// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Environment diffuse — spherical harmonics irradiance from environment maps.

use std::f32::consts::{FRAC_1_PI, PI};

/// SH band 0-1 coefficients (9 coefficients for L0+L1+L2).
pub const SH_COEFF_COUNT: usize = 9;

/// SH irradiance probe.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct EnvDiffuseProbe {
    /// 9 SH coefficients per RGB channel.
    pub sh_r: [f32; SH_COEFF_COUNT],
    pub sh_g: [f32; SH_COEFF_COUNT],
    pub sh_b: [f32; SH_COEFF_COUNT],
    pub intensity: f32,
    pub enabled: bool,
}

impl Default for EnvDiffuseProbe {
    fn default() -> Self {
        Self {
            sh_r: [0.0; SH_COEFF_COUNT],
            sh_g: [0.0; SH_COEFF_COUNT],
            sh_b: [0.0; SH_COEFF_COUNT],
            intensity: 1.0,
            enabled: true,
        }
    }
}

/// Environment diffuse configuration.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct EnvDiffuseConfig {
    pub max_probes: usize,
    pub blend_radius: f32,
}

impl Default for EnvDiffuseConfig {
    fn default() -> Self {
        Self {
            max_probes: 8,
            blend_radius: 10.0,
        }
    }
}

/// Environment diffuse system.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct EnvDiffuse {
    pub config: EnvDiffuseConfig,
    pub probes: Vec<EnvDiffuseProbe>,
}

/// Create new env diffuse system.
#[allow(dead_code)]
pub fn new_env_diffuse(cfg: EnvDiffuseConfig) -> EnvDiffuse {
    EnvDiffuse {
        config: cfg,
        probes: Vec::new(),
    }
}

/// Add a probe.
#[allow(dead_code)]
pub fn add_probe(e: &mut EnvDiffuse, probe: EnvDiffuseProbe) -> Option<usize> {
    if e.probes.len() >= e.config.max_probes {
        return None;
    }
    let idx = e.probes.len();
    e.probes.push(probe);
    Some(idx)
}

/// Probe count.
#[allow(dead_code)]
pub fn probe_count_env(e: &EnvDiffuse) -> usize {
    e.probes.len()
}

/// Sample SH irradiance for a given normal direction.
#[allow(dead_code)]
pub fn sample_sh_irradiance(probe: &EnvDiffuseProbe, normal: [f32; 3]) -> [f32; 3] {
    let (nx, ny, nz) = (normal[0], normal[1], normal[2]);
    // SH basis evaluation for L0 and L1
    let y0 = 0.282_094_8; // 1/(2*sqrt(PI))
    let y1 = 0.488_602_5 * ny; // sqrt(3/(4*PI)) * y
    let y2 = 0.488_602_5 * nz; // sqrt(3/(4*PI)) * z
    let y3 = 0.488_602_5 * nx; // sqrt(3/(4*PI)) * x
    let sh = [y0, y1, y2, y3, 0.0, 0.0, 0.0, 0.0, 0.0];
    let mut r = 0.0f32;
    let mut g = 0.0f32;
    let mut b = 0.0f32;
    #[allow(clippy::needless_range_loop)]
    for i in 0..SH_COEFF_COUNT {
        r += probe.sh_r[i] * sh[i];
        g += probe.sh_g[i] * sh[i];
        b += probe.sh_b[i] * sh[i];
    }
    [
        r * probe.intensity,
        g * probe.intensity,
        b * probe.intensity,
    ]
}

/// Constant ambient color using FRAC_1_PI.
#[allow(dead_code)]
pub fn ambient_color(probe: &EnvDiffuseProbe) -> [f32; 3] {
    let scale = FRAC_1_PI * probe.intensity;
    [
        probe.sh_r[0] * scale,
        probe.sh_g[0] * scale,
        probe.sh_b[0] * scale,
    ]
}

/// Compute solid angle weight for a hemisphere sample using PI.
#[allow(dead_code)]
pub fn hemisphere_pdf() -> f32 {
    1.0 / (2.0 * PI)
}

/// Blend two probes by weight.
#[allow(dead_code)]
pub fn blend_probes_env(a: &EnvDiffuseProbe, b: &EnvDiffuseProbe, t: f32) -> EnvDiffuseProbe {
    let t = t.clamp(0.0, 1.0);
    let mut result = EnvDiffuseProbe::default();
    #[allow(clippy::needless_range_loop)]
    for i in 0..SH_COEFF_COUNT {
        result.sh_r[i] = a.sh_r[i] + (b.sh_r[i] - a.sh_r[i]) * t;
        result.sh_g[i] = a.sh_g[i] + (b.sh_g[i] - a.sh_g[i]) * t;
        result.sh_b[i] = a.sh_b[i] + (b.sh_b[i] - a.sh_b[i]) * t;
    }
    result.intensity = a.intensity + (b.intensity - a.intensity) * t;
    result
}

/// Export to JSON-like string.
#[allow(dead_code)]
pub fn env_diffuse_to_json(e: &EnvDiffuse) -> String {
    format!(
        r#"{{"probe_count":{},"max_probes":{}}}"#,
        e.probes.len(),
        e.config.max_probes
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_system_empty() {
        let e = new_env_diffuse(EnvDiffuseConfig::default());
        assert_eq!(probe_count_env(&e), 0);
    }

    #[test]
    fn add_probe_ok() {
        let mut e = new_env_diffuse(EnvDiffuseConfig::default());
        let r = add_probe(&mut e, EnvDiffuseProbe::default());
        assert!(r.is_some());
    }

    #[test]
    fn add_probe_capacity_limit() {
        let mut e = new_env_diffuse(EnvDiffuseConfig {
            max_probes: 1,
            ..Default::default()
        });
        add_probe(&mut e, EnvDiffuseProbe::default());
        let r = add_probe(&mut e, EnvDiffuseProbe::default());
        assert!(r.is_none());
    }

    #[test]
    fn sample_sh_neutral_probe() {
        let probe = EnvDiffuseProbe::default();
        let color = sample_sh_irradiance(&probe, [0.0, 1.0, 0.0]);
        assert_eq!(color, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn ambient_color_uses_frac1pi() {
        let mut probe = EnvDiffuseProbe::default();
        probe.sh_r[0] = PI;
        let c = ambient_color(&probe);
        assert!((c[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn hemisphere_pdf_correct() {
        let pdf = hemisphere_pdf();
        assert!((pdf - 1.0 / (2.0 * PI)).abs() < 1e-6);
    }

    #[test]
    fn blend_probes_at_zero() {
        let a = EnvDiffuseProbe {
            intensity: 0.5,
            ..Default::default()
        };
        let b = EnvDiffuseProbe {
            intensity: 1.0,
            ..Default::default()
        };
        let m = blend_probes_env(&a, &b, 0.0);
        assert!((m.intensity - 0.5).abs() < 1e-6);
    }

    #[test]
    fn blend_probes_at_one() {
        let a = EnvDiffuseProbe {
            intensity: 0.0,
            ..Default::default()
        };
        let b = EnvDiffuseProbe {
            intensity: 1.0,
            ..Default::default()
        };
        let m = blend_probes_env(&a, &b, 1.0);
        assert!((m.intensity - 1.0).abs() < 1e-6);
    }

    #[test]
    fn json_contains_probe_count() {
        let e = new_env_diffuse(EnvDiffuseConfig::default());
        assert!(env_diffuse_to_json(&e).contains("probe_count"));
    }

    #[test]
    fn probes_slice_not_empty() {
        let mut e = new_env_diffuse(EnvDiffuseConfig::default());
        add_probe(&mut e, EnvDiffuseProbe::default());
        assert!(!e.probes.is_empty());
    }
}
