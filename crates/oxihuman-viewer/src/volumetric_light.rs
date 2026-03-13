// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Volumetric light (god rays) — ray marching through a volume
//! to compute in-scattered light contribution.

use std::f32::consts::PI;

/// Volumetric light configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct VolumetricConfig {
    /// Number of ray march steps.
    pub steps: u32,
    /// Scattering coefficient.
    pub scattering: f32,
    /// Absorption coefficient.
    pub absorption: f32,
    /// Anisotropy (Henyey-Greenstein g parameter, -1..1).
    pub anisotropy: f32,
    /// Light intensity.
    pub intensity: f32,
    /// Light colour [r, g, b].
    pub light_color: [f32; 3],
    /// Maximum ray distance.
    pub max_distance: f32,
}

impl Default for VolumetricConfig {
    fn default() -> Self {
        Self {
            steps: 32,
            scattering: 0.01,
            absorption: 0.005,
            anisotropy: 0.3,
            intensity: 1.0,
            light_color: [1.0, 0.95, 0.9],
            max_distance: 50.0,
        }
    }
}

/// Volumetric result per pixel.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VolumetricResult {
    /// In-scattered light [r, g, b].
    pub inscatter: [f32; 3],
    /// Transmittance (1 = no absorption).
    pub transmittance: f32,
}

/// Henyey-Greenstein phase function.
///
/// `cos_theta` is the cosine of the angle between view and light directions.
/// `g` is the anisotropy parameter (-1..1).
#[allow(dead_code)]
pub fn henyey_greenstein(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = 1.0 + g2 - 2.0 * g * cos_theta;
    if denom < 1e-8 {
        return 1.0 / (4.0 * PI);
    }
    (1.0 - g2) / (4.0 * PI * denom * denom.sqrt())
}

/// Rayleigh phase function (for sky scattering).
#[allow(dead_code)]
pub fn rayleigh_phase(cos_theta: f32) -> f32 {
    (3.0 / (16.0 * PI)) * (1.0 + cos_theta * cos_theta)
}

/// Beer-Lambert transmittance over distance.
#[allow(dead_code)]
pub fn beer_lambert(extinction: f32, distance: f32) -> f32 {
    (-extinction * distance).exp()
}

/// Combined extinction coefficient.
#[allow(dead_code)]
pub fn extinction(scattering: f32, absorption: f32) -> f32 {
    scattering + absorption
}

/// Simple volumetric ray march (uniform density).
///
/// `view_pos`: camera position.
/// `view_dir`: normalised direction towards the fragment.
/// `light_dir`: normalised direction towards the light.
/// `fragment_depth`: distance to the surface.
#[allow(dead_code)]
pub fn ray_march_volumetric(
    view_dir: [f32; 3],
    light_dir: [f32; 3],
    fragment_depth: f32,
    config: &VolumetricConfig,
) -> VolumetricResult {
    let march_dist = fragment_depth.min(config.max_distance);
    if config.steps == 0 || march_dist <= 0.0 {
        return VolumetricResult {
            inscatter: [0.0; 3],
            transmittance: 1.0,
        };
    }

    let step_size = march_dist / config.steps as f32;
    let ext = extinction(config.scattering, config.absorption);
    let cos_theta = dot(view_dir, light_dir);
    let phase = henyey_greenstein(cos_theta, config.anisotropy);

    let mut transmittance = 1.0_f32;
    let mut inscatter = [0.0_f32; 3];

    for _ in 0..config.steps {
        let step_transmittance = beer_lambert(ext, step_size);
        let in_scat = config.scattering * phase * config.intensity;

        for j in 0..3 {
            inscatter[j] += transmittance * in_scat * config.light_color[j] * step_size;
        }

        transmittance *= step_transmittance;

        if transmittance < 1e-4 {
            break;
        }
    }

    VolumetricResult {
        inscatter,
        transmittance,
    }
}

/// Apply volumetric result to a fragment.
#[allow(dead_code)]
pub fn apply_volumetric(frag_color: [f32; 3], vol: &VolumetricResult) -> [f32; 3] {
    [
        frag_color[0] * vol.transmittance + vol.inscatter[0],
        frag_color[1] * vol.transmittance + vol.inscatter[1],
        frag_color[2] * vol.transmittance + vol.inscatter[2],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_config() {
        let c = VolumetricConfig::default();
        assert!(c.steps > 0);
    }

    #[test]
    fn test_hg_isotropic() {
        let p = henyey_greenstein(0.0, 0.0);
        assert!((p - 1.0 / (4.0 * PI)).abs() < 1e-5);
    }

    #[test]
    fn test_hg_forward_scattering() {
        let fwd = henyey_greenstein(1.0, 0.5);
        let bwd = henyey_greenstein(-1.0, 0.5);
        assert!(fwd > bwd);
    }

    #[test]
    fn test_rayleigh_symmetry() {
        let a = rayleigh_phase(0.5);
        let b = rayleigh_phase(-0.5);
        assert!((a - b).abs() < 1e-5);
    }

    #[test]
    fn test_beer_lambert_zero() {
        let t = beer_lambert(0.1, 0.0);
        assert!((t - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_beer_lambert_decreasing() {
        let t1 = beer_lambert(0.1, 1.0);
        let t2 = beer_lambert(0.1, 10.0);
        assert!(t1 > t2);
    }

    #[test]
    fn test_extinction_sum() {
        let e = extinction(0.01, 0.005);
        assert!((e - 0.015).abs() < 1e-6);
    }

    #[test]
    fn test_ray_march_zero_steps() {
        let c = VolumetricConfig { steps: 0, ..Default::default() };
        let r = ray_march_volumetric([0.0, 0.0, -1.0], [0.0, -1.0, 0.0], 10.0, &c);
        assert!((r.transmittance - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_march_produces_inscatter() {
        let c = VolumetricConfig::default();
        let r = ray_march_volumetric([0.0, 0.0, -1.0], [0.0, -1.0, 0.0], 20.0, &c);
        assert!(r.inscatter[0] > 0.0);
        assert!(r.transmittance < 1.0);
    }

    #[test]
    fn test_apply_volumetric() {
        let vol = VolumetricResult { inscatter: [0.1, 0.1, 0.1], transmittance: 0.8 };
        let r = apply_volumetric([1.0, 1.0, 1.0], &vol);
        assert!((r[0] - 0.9).abs() < 1e-5);
    }
}
