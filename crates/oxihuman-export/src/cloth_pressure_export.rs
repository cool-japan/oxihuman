// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export cloth internal pressure simulation parameters.

/// Cloth pressure export configuration.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothPressureConfig {
    pub pressure: f32,
    pub volume_stiffness: f32,
    pub shrink_factor: f32,
    pub vertex_count: usize,
}

impl Default for ClothPressureConfig {
    fn default() -> Self {
        Self {
            pressure: 1.0,
            volume_stiffness: 1.0,
            shrink_factor: 0.0,
            vertex_count: 0,
        }
    }
}

/// Per-vertex pressure force cache.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PressureForceCache {
    pub forces: Vec<[f32; 3]>,
}

/// Compute approximate mesh volume using the divergence theorem (triangle soup).
#[allow(dead_code)]
pub fn compute_signed_volume(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let mut vol = 0.0_f32;
    for tri in indices.chunks_exact(3) {
        let a = positions[tri[0] as usize];
        let b = positions[tri[1] as usize];
        let c = positions[tri[2] as usize];
        vol += a[0] * (b[1] * c[2] - c[1] * b[2]);
        vol += a[1] * (b[2] * c[0] - c[2] * b[0]);
        vol += a[2] * (b[0] * c[1] - c[0] * b[1]);
    }
    vol.abs() / 6.0
}

/// Compute pressure forces acting on each vertex.
#[allow(dead_code)]
pub fn compute_pressure_forces(
    positions: &[[f32; 3]],
    indices: &[u32],
    cfg: &ClothPressureConfig,
) -> PressureForceCache {
    let vol = compute_signed_volume(positions, indices);
    let scale = cfg.pressure * cfg.volume_stiffness / (vol.max(1e-8));
    let mut forces = vec![[0.0_f32; 3]; positions.len()];
    for tri in indices.chunks_exact(3) {
        let (ai, bi, ci) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let a = positions[ai];
        let b = positions[bi];
        let c = positions[ci];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        let f = [n[0] * scale / 3.0, n[1] * scale / 3.0, n[2] * scale / 3.0];
        for &vi in &[ai, bi, ci] {
            forces[vi][0] += f[0];
            forces[vi][1] += f[1];
            forces[vi][2] += f[2];
        }
    }
    PressureForceCache { forces }
}

/// Serialise pressure config to flat buffer.
#[allow(dead_code)]
pub fn serialise_pressure_config(cfg: &ClothPressureConfig) -> Vec<f32> {
    vec![cfg.pressure, cfg.volume_stiffness, cfg.shrink_factor]
}

/// Maximum force magnitude in the cache.
#[allow(dead_code)]
pub fn max_force_magnitude(cache: &PressureForceCache) -> f32 {
    cache
        .forces
        .iter()
        .map(|f| (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt())
        .fold(0.0_f32, f32::max)
}

/// Scale all forces by a factor.
#[allow(dead_code)]
pub fn scale_forces(cache: &mut PressureForceCache, factor: f32) {
    for f in &mut cache.forces {
        f[0] *= factor;
        f[1] *= factor;
        f[2] *= factor;
    }
}

/// Check whether the pressure is outward (positive).
#[allow(dead_code)]
pub fn is_outward_pressure(cfg: &ClothPressureConfig) -> bool {
    cfg.pressure > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_tetra() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0_f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let idx = vec![0, 1, 2, 0, 3, 1, 0, 2, 3, 1, 3, 2];
        (pos, idx)
    }

    #[test]
    fn test_default_config() {
        let cfg = ClothPressureConfig::default();
        assert!((cfg.pressure - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_signed_volume_positive() {
        let (pos, idx) = unit_tetra();
        let v = compute_signed_volume(&pos, &idx);
        assert!(v > 0.0);
    }

    #[test]
    fn test_pressure_forces_count() {
        let (pos, idx) = unit_tetra();
        let cfg = ClothPressureConfig::default();
        let cache = compute_pressure_forces(&pos, &idx, &cfg);
        assert_eq!(cache.forces.len(), pos.len());
    }

    #[test]
    fn test_serialise_config_length() {
        let cfg = ClothPressureConfig::default();
        assert_eq!(serialise_pressure_config(&cfg).len(), 3);
    }

    #[test]
    fn test_max_force_zero_for_empty() {
        let cache = PressureForceCache { forces: vec![] };
        assert_eq!(max_force_magnitude(&cache), 0.0);
    }

    #[test]
    fn test_scale_forces() {
        let mut cache = PressureForceCache {
            forces: vec![[1.0, 0.0, 0.0]],
        };
        scale_forces(&mut cache, 2.0);
        assert!((cache.forces[0][0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_is_outward_pressure_true() {
        let cfg = ClothPressureConfig {
            pressure: 0.5,
            ..Default::default()
        };
        assert!(is_outward_pressure(&cfg));
    }

    #[test]
    fn test_is_outward_pressure_false() {
        let cfg = ClothPressureConfig {
            pressure: -1.0,
            ..Default::default()
        };
        assert!(!is_outward_pressure(&cfg));
    }

    #[test]
    fn test_max_force_magnitude_known() {
        let cache = PressureForceCache {
            forces: vec![[3.0, 4.0, 0.0]],
        };
        assert!((max_force_magnitude(&cache) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_shrink_factor_default() {
        let cfg = ClothPressureConfig::default();
        assert!((cfg.shrink_factor - 0.0).abs() < 1e-6);
    }
}
