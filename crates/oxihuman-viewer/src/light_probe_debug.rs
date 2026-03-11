// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Spherical harmonic light probe debug visualization.

#![allow(dead_code)]

use std::f32::consts::PI;

/// 9-coefficient L2 SH probe (RGB).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShProbe {
    /// SH coefficients: `[coeff][rgb]`.
    pub coeffs: [[f32; 3]; 9],
}

#[allow(dead_code)]
impl Default for ShProbe {
    fn default() -> Self {
        Self {
            coeffs: [[0.0; 3]; 9],
        }
    }
}

/// Debug visualization config.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LightProbeDebugConfig {
    /// Sphere resolution for debug rendering.
    pub sphere_subdivisions: u32,
    /// Exposure multiplier.
    pub exposure: f32,
    /// Show axes overlay.
    pub show_axes: bool,
}

#[allow(dead_code)]
impl Default for LightProbeDebugConfig {
    fn default() -> Self {
        Self {
            sphere_subdivisions: 32,
            exposure: 1.0,
            show_axes: true,
        }
    }
}

/// Create a new default config.
#[allow(dead_code)]
pub fn new_light_probe_debug_config() -> LightProbeDebugConfig {
    LightProbeDebugConfig::default()
}

/// Create a new empty SH probe.
#[allow(dead_code)]
pub fn new_sh_probe() -> ShProbe {
    ShProbe::default()
}

/// Evaluate the L0 SH coefficient (ambient).
#[allow(dead_code)]
pub fn sh_l0_scale() -> f32 {
    1.0 / (2.0 * (PI).sqrt())
}

/// Evaluate SH irradiance at a direction given by spherical angles.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn evaluate_sh_irradiance(probe: &ShProbe, theta: f32, phi: f32) -> [f32; 3] {
    let x = theta.sin() * phi.cos();
    let y = theta.cos();
    let z = theta.sin() * phi.sin();
    let basis = [
        0.282_095f32,
        0.488_603 * y,
        0.488_603 * z,
        0.488_603 * x,
        1.092_548 * x * y,
        1.092_548 * y * z,
        0.315_392 * (3.0 * z * z - 1.0),
        1.092_548 * x * z,
        0.546_274 * (x * x - y * y),
    ];
    let mut out = [0.0f32; 3];
    for c in 0..9 {
        out[0] += probe.coeffs[c][0] * basis[c];
        out[1] += probe.coeffs[c][1] * basis[c];
        out[2] += probe.coeffs[c][2] * basis[c];
    }
    out
}

/// Set exposure.
#[allow(dead_code)]
pub fn lpd_set_exposure(cfg: &mut LightProbeDebugConfig, value: f32) {
    cfg.exposure = value.max(0.0);
}

/// Reset probe coefficients to zero.
#[allow(dead_code)]
pub fn sh_probe_reset(probe: &mut ShProbe) {
    *probe = ShProbe::default();
}

/// Compute total energy of the probe.
#[allow(dead_code)]
pub fn sh_probe_energy(probe: &ShProbe) -> f32 {
    probe
        .coeffs
        .iter()
        .flat_map(|c| c.iter())
        .map(|v| v * v)
        .sum::<f32>()
        .sqrt()
}

/// Serialize config to JSON.
#[allow(dead_code)]
pub fn light_probe_debug_to_json(cfg: &LightProbeDebugConfig) -> String {
    format!(
        r#"{{"sphere_subdivisions":{},"exposure":{:.4},"show_axes":{}}}"#,
        cfg.sphere_subdivisions, cfg.exposure, cfg.show_axes
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let c = LightProbeDebugConfig::default();
        assert_eq!(c.sphere_subdivisions, 32);
        assert!((c.exposure - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sh_l0_scale() {
        let s = sh_l0_scale();
        assert!(s > 0.0 && s < 1.0);
    }

    #[test]
    fn test_evaluate_sh_zero_probe() {
        let probe = ShProbe::default();
        let irr = evaluate_sh_irradiance(&probe, 0.0, 0.0);
        assert!(irr[0].abs() < 1e-5);
    }

    #[test]
    fn test_sh_probe_energy_zero() {
        let probe = ShProbe::default();
        assert!(sh_probe_energy(&probe) < 1e-6);
    }

    #[test]
    fn test_sh_probe_energy_nonzero() {
        let mut probe = ShProbe::default();
        probe.coeffs[0][0] = 1.0;
        assert!(sh_probe_energy(&probe) > 0.0);
    }

    #[test]
    fn test_set_exposure_clamped() {
        let mut c = LightProbeDebugConfig::default();
        lpd_set_exposure(&mut c, -2.0);
        assert!(c.exposure < 1e-6);
    }

    #[test]
    fn test_sh_probe_reset() {
        let mut p = ShProbe {
            coeffs: [[1.0; 3]; 9],
        };
        sh_probe_reset(&mut p);
        assert!(sh_probe_energy(&p) < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let j = light_probe_debug_to_json(&LightProbeDebugConfig::default());
        assert!(j.contains("sphere_subdivisions"));
    }

    #[test]
    fn test_pi_consts_used() {
        let s = sh_l0_scale();
        let expected = 1.0 / (2.0 * PI.sqrt());
        assert!((s - expected).abs() < 1e-5);
    }
}
