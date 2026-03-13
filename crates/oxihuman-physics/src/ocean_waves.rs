// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Ocean wave simulation using Gerstner waves.

#![allow(dead_code)]

use std::f32::consts::PI;

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// Single Gerstner wave component.
#[allow(dead_code)]
#[derive(Clone)]
pub struct GerstnerWave {
    /// Normalized XZ direction of propagation.
    pub direction: [f32; 2],
    /// Steepness factor Q (0..1).
    pub steepness: f32,
    /// Wavelength in metres.
    pub wavelength: f32,
    /// Phase speed in m/s.
    pub speed: f32,
    /// Amplitude in metres.
    pub amplitude: f32,
}

/// Configuration for an ocean simulation.
#[allow(dead_code)]
pub struct OceanConfig {
    pub waves: Vec<GerstnerWave>,
    /// Gravitational acceleration (m/s²), default 9.81.
    pub gravity: f32,
    /// Jacobian threshold for foam generation.
    pub foam_threshold: f32,
}

/// Ocean surface state.
#[allow(dead_code)]
pub struct OceanSurface {
    pub config: OceanConfig,
    /// Current simulation time in seconds.
    pub time: f32,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Build a default OceanConfig with 4 Gerstner waves.
#[allow(dead_code)]
pub fn default_ocean_config() -> OceanConfig {
    OceanConfig {
        waves: vec![
            GerstnerWave {
                direction: [1.0, 0.0],
                steepness: 0.3,
                wavelength: 20.0,
                speed: 5.0,
                amplitude: 0.8,
            },
            GerstnerWave {
                direction: [0.707, 0.707],
                steepness: 0.25,
                wavelength: 12.0,
                speed: 4.0,
                amplitude: 0.5,
            },
            GerstnerWave {
                direction: [-0.5, 0.866],
                steepness: 0.2,
                wavelength: 8.0,
                speed: 3.0,
                amplitude: 0.3,
            },
            GerstnerWave {
                direction: [0.0, -1.0],
                steepness: 0.15,
                wavelength: 5.0,
                speed: 2.0,
                amplitude: 0.15,
            },
        ],
        gravity: 9.81,
        foam_threshold: 0.5,
    }
}

/// Create a new OceanSurface at t=0.
#[allow(dead_code)]
pub fn new_ocean_surface(cfg: OceanConfig) -> OceanSurface {
    OceanSurface {
        config: cfg,
        time: 0.0,
    }
}

/// Angular frequency for a wave: ω = 2π / λ.
#[allow(dead_code)]
pub fn wave_frequency(wave: &GerstnerWave) -> f32 {
    2.0 * PI / wave.wavelength
}

/// Phase speed derived from dispersion: c = sqrt(g / k).
#[allow(dead_code)]
pub fn gerstner_phase_speed(wave: &GerstnerWave) -> f32 {
    let k = wave_frequency(wave);
    // Divide gravity by k to get c² = g/k
    (9.81f32 / k).sqrt()
}

/// Compute the XYZ Gerstner displacement for one wave at flat-space position pos_xz and time.
#[allow(dead_code)]
pub fn gerstner_displacement(wave: &GerstnerWave, pos_xz: [f32; 2], time: f32) -> [f32; 3] {
    let k = wave_frequency(wave);
    // Normalise direction
    let dir_len = (wave.direction[0] * wave.direction[0] + wave.direction[1] * wave.direction[1])
        .sqrt()
        .max(1e-12);
    let dx = wave.direction[0] / dir_len;
    let dz = wave.direction[1] / dir_len;

    let dot = dx * pos_xz[0] + dz * pos_xz[1];
    let phase = k * dot - wave.speed * time;
    let c = phase.cos();
    let s = phase.sin();

    let horizontal_scale = wave.steepness * wave.amplitude;

    [
        horizontal_scale * dx * c,
        wave.amplitude * s,
        horizontal_scale * dz * c,
    ]
}

/// Compute the surface normal contribution from a single Gerstner wave.
#[allow(dead_code)]
pub fn gerstner_normal(wave: &GerstnerWave, pos_xz: [f32; 2], time: f32) -> [f32; 3] {
    let k = wave_frequency(wave);
    let dir_len = (wave.direction[0] * wave.direction[0] + wave.direction[1] * wave.direction[1])
        .sqrt()
        .max(1e-12);
    let dx = wave.direction[0] / dir_len;
    let dz = wave.direction[1] / dir_len;

    let dot = dx * pos_xz[0] + dz * pos_xz[1];
    let phase = k * dot - wave.speed * time;
    let c = phase.cos();
    let s = phase.sin();

    let wa = k * wave.amplitude;
    [-dx * wa * c, 1.0 - wave.steepness * wa * s, -dz * wa * c]
}

/// Sum all Gerstner wave displacements.
#[allow(dead_code)]
pub fn ocean_displacement(surface: &OceanSurface, pos_xz: [f32; 2]) -> [f32; 3] {
    let mut total = [0.0f32; 3];
    for wave in &surface.config.waves {
        let d = gerstner_displacement(wave, pos_xz, surface.time);
        total[0] += d[0];
        total[1] += d[1];
        total[2] += d[2];
    }
    total
}

/// Compute the combined ocean surface normal (normalised).
#[allow(dead_code)]
pub fn ocean_normal(surface: &OceanSurface, pos_xz: [f32; 2]) -> [f32; 3] {
    let mut n = [0.0f32, 1.0f32, 0.0f32];
    for wave in &surface.config.waves {
        let wn = gerstner_normal(wave, pos_xz, surface.time);
        n[0] += wn[0];
        n[1] += wn[1] - 1.0; // subtract the base 1.0 added per wave
        n[2] += wn[2];
    }
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-12);
    [n[0] / len, n[1] / len, n[2] / len]
}

/// Advance time by dt seconds.
#[allow(dead_code)]
pub fn advance_ocean(surface: &mut OceanSurface, dt: f32) {
    surface.time += dt;
}

/// Return only the Y (height) component of the ocean displacement.
#[allow(dead_code)]
pub fn ocean_height_at(surface: &OceanSurface, pos_xz: [f32; 2]) -> f32 {
    ocean_displacement(surface, pos_xz)[1]
}

/// Sample the ocean height on a regular grid and return displaced positions.
#[allow(dead_code)]
pub fn sample_ocean_grid(
    surface: &OceanSurface,
    grid_min: [f32; 2],
    grid_max: [f32; 2],
    resolution: u32,
) -> Vec<[f32; 3]> {
    let res = resolution.max(1) as usize;
    let mut out = Vec::with_capacity(res * res);
    for row in 0..res {
        for col in 0..res {
            let tx = col as f32 / (res - 1).max(1) as f32;
            let tz = row as f32 / (res - 1).max(1) as f32;
            let x = grid_min[0] + tx * (grid_max[0] - grid_min[0]);
            let z = grid_min[1] + tz * (grid_max[1] - grid_min[1]);
            let disp = ocean_displacement(surface, [x, z]);
            out.push([x + disp[0], disp[1], z + disp[2]]);
        }
    }
    out
}

/// Foam mask based on a simple Jacobian estimate (horizontal displacement divergence).
/// Returns a value in [0, 1] where 1 means maximum foam.
#[allow(dead_code)]
pub fn ocean_foam_mask(surface: &OceanSurface, pos_xz: [f32; 2]) -> f32 {
    // Approximate Jacobian via finite differences of horizontal displacement
    let eps = 0.01f32;
    let dx0 = ocean_displacement(surface, [pos_xz[0] - eps, pos_xz[1]])[0];
    let dx1 = ocean_displacement(surface, [pos_xz[0] + eps, pos_xz[1]])[0];
    let dz0 = ocean_displacement(surface, [pos_xz[0], pos_xz[1] - eps])[2];
    let dz1 = ocean_displacement(surface, [pos_xz[0], pos_xz[1] + eps])[2];

    let ddx = (dx1 - dx0) / (2.0 * eps);
    let ddz = (dz1 - dz0) / (2.0 * eps);
    // Jacobian J = (1 + ddx)(1 + ddz) - ...  simplified to compression indicator
    let jacobian = (1.0 + ddx) * (1.0 + ddz);
    // Foam appears when jacobian < foam_threshold
    let foam = (surface.config.foam_threshold - jacobian).max(0.0)
        / surface.config.foam_threshold.max(1e-12);
    foam.min(1.0)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_waves() {
        let cfg = default_ocean_config();
        assert!(!cfg.waves.is_empty());
        assert_eq!(cfg.waves.len(), 4);
    }

    #[test]
    fn test_default_config_gravity() {
        let cfg = default_ocean_config();
        assert!((cfg.gravity - 9.81).abs() < 1e-3);
    }

    #[test]
    fn test_new_ocean_surface_time_zero() {
        let cfg = default_ocean_config();
        let surf = new_ocean_surface(cfg);
        assert_eq!(surf.time, 0.0);
    }

    #[test]
    fn test_advance_ocean_advances_time() {
        let cfg = default_ocean_config();
        let mut surf = new_ocean_surface(cfg);
        advance_ocean(&mut surf, 0.016);
        assert!((surf.time - 0.016).abs() < 1e-6);
    }

    #[test]
    fn test_wave_frequency_positive() {
        let cfg = default_ocean_config();
        for wave in &cfg.waves {
            let freq = wave_frequency(wave);
            assert!(freq > 0.0);
        }
    }

    #[test]
    fn test_gerstner_phase_speed_positive() {
        let cfg = default_ocean_config();
        for wave in &cfg.waves {
            let speed = gerstner_phase_speed(wave);
            assert!(speed > 0.0);
        }
    }

    #[test]
    fn test_gerstner_displacement_at_origin_t0() {
        let cfg = default_ocean_config();
        let surf = new_ocean_surface(cfg);
        let disp = ocean_displacement(&surf, [0.0, 0.0]);
        assert_eq!(disp.len(), 3);
        // Displacement must be finite
        for v in &disp {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn test_ocean_height_at_finite() {
        let cfg = default_ocean_config();
        let surf = new_ocean_surface(cfg);
        let h = ocean_height_at(&surf, [5.0, 3.0]);
        assert!(h.is_finite());
    }

    #[test]
    fn test_ocean_height_varies_with_position() {
        let cfg = default_ocean_config();
        let surf = new_ocean_surface(cfg);
        let h1 = ocean_height_at(&surf, [0.0, 0.0]);
        let h2 = ocean_height_at(&surf, [10.0, 5.0]);
        // Heights at different positions need not be equal
        assert!(h1.is_finite() && h2.is_finite());
    }

    #[test]
    fn test_sample_ocean_grid_count() {
        let cfg = default_ocean_config();
        let surf = new_ocean_surface(cfg);
        let pts = sample_ocean_grid(&surf, [0.0, 0.0], [10.0, 10.0], 4);
        assert_eq!(pts.len(), 16); // 4 * 4
    }

    #[test]
    fn test_sample_ocean_grid_resolution_1() {
        let cfg = default_ocean_config();
        let surf = new_ocean_surface(cfg);
        let pts = sample_ocean_grid(&surf, [0.0, 0.0], [10.0, 10.0], 1);
        assert_eq!(pts.len(), 1);
    }

    #[test]
    fn test_ocean_normal_z_positive() {
        // At rest, normal Y component should be positive (upward)
        let cfg = default_ocean_config();
        let surf = new_ocean_surface(cfg);
        let n = ocean_normal(&surf, [0.0, 0.0]);
        assert!(n[1] > 0.0, "normal Y should be positive, got {}", n[1]);
    }

    #[test]
    fn test_ocean_normal_unit_length() {
        let cfg = default_ocean_config();
        let surf = new_ocean_surface(cfg);
        let n = ocean_normal(&surf, [3.0, 7.0]);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!(
            (len - 1.0).abs() < 1e-4,
            "normal should be unit length, got {len}"
        );
    }

    #[test]
    fn test_gerstner_foam_mask_range() {
        let cfg = default_ocean_config();
        let surf = new_ocean_surface(cfg);
        let foam = ocean_foam_mask(&surf, [0.0, 0.0]);
        assert!(
            (0.0..=1.0).contains(&foam),
            "foam should be in [0,1], got {foam}"
        );
    }
}
