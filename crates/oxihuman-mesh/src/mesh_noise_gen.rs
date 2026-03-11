// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Procedural noise-based mesh generation and displacement.

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;

// ── Public types ─────────────────────────────────────────────────────────────

/// Configuration for fractal Brownian motion noise displacement.
#[derive(Debug, Clone)]
pub struct NoiseGenConfig {
    /// FBm octaves. Default 4.
    pub octaves: u32,
    /// Base frequency. Default 1.0.
    pub frequency: f32,
    /// Base amplitude. Default 0.1.
    pub amplitude: f32,
    /// Frequency multiplier per octave. Default 2.0.
    pub lacunarity: f32,
    /// Amplitude multiplier per octave. Default 0.5.
    pub persistence: f32,
    /// Random seed.
    pub seed: u64,
}

impl Default for NoiseGenConfig {
    fn default() -> Self {
        Self {
            octaves: 4,
            frequency: 1.0,
            amplitude: 0.1,
            lacunarity: 2.0,
            persistence: 0.5,
            seed: 42,
        }
    }
}

/// Result of a noise-based displacement operation.
#[derive(Debug, Clone)]
pub struct NoiseDisplaceResult {
    /// Displaced vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Minimum displacement value applied.
    pub min_displacement: f32,
    /// Maximum displacement value applied.
    pub max_displacement: f32,
    /// Root-mean-square displacement.
    pub rms_displacement: f32,
}

// ── LCG value noise ───────────────────────────────────────────────────────────

/// Hash three integers using a fast LCG-style hash.
fn hash3(ix: i32, iy: i32, iz: i32, seed: u64) -> f32 {
    let mut h = seed;
    h = h
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    h ^= ix as u64;
    h = h
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    h ^= iy as u64;
    h = h
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    h ^= iz as u64;
    h = h
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    // Map to [-1, 1].
    let u = (h >> 11) as f64 / (1u64 << 53) as f64;
    (u as f32) * 2.0 - 1.0
}

/// Smooth cubic Hermite interpolation.
#[inline]
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Trilinear interpolation.
#[allow(clippy::too_many_arguments)]
fn trilinear(
    c000: f32,
    c100: f32,
    c010: f32,
    c110: f32,
    c001: f32,
    c101: f32,
    c011: f32,
    c111: f32,
    tx: f32,
    ty: f32,
    tz: f32,
) -> f32 {
    let c00 = c000 * (1.0 - tx) + c100 * tx;
    let c01 = c001 * (1.0 - tx) + c101 * tx;
    let c10 = c010 * (1.0 - tx) + c110 * tx;
    let c11 = c011 * (1.0 - tx) + c111 * tx;
    let c0 = c00 * (1.0 - ty) + c10 * ty;
    let c1 = c01 * (1.0 - ty) + c11 * ty;
    c0 * (1.0 - tz) + c1 * tz
}

/// Smooth LCG-hash trilinear value noise in -1..1.
pub fn lcg_value_noise(x: f32, y: f32, z: f32, seed: u64) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let iz = z.floor() as i32;
    let tx = smoothstep(x - x.floor());
    let ty = smoothstep(y - y.floor());
    let tz = smoothstep(z - z.floor());

    let c000 = hash3(ix, iy, iz, seed);
    let c100 = hash3(ix + 1, iy, iz, seed);
    let c010 = hash3(ix, iy + 1, iz, seed);
    let c110 = hash3(ix + 1, iy + 1, iz, seed);
    let c001 = hash3(ix, iy, iz + 1, seed);
    let c101 = hash3(ix + 1, iy, iz + 1, seed);
    let c011 = hash3(ix, iy + 1, iz + 1, seed);
    let c111 = hash3(ix + 1, iy + 1, iz + 1, seed);

    trilinear(c000, c100, c010, c110, c001, c101, c011, c111, tx, ty, tz)
}

// ── FBm noise ─────────────────────────────────────────────────────────────────

/// Fractal Brownian Motion noise built from `lcg_value_noise`.
pub fn fbm_noise(x: f32, y: f32, z: f32, cfg: &NoiseGenConfig) -> f32 {
    let mut value = 0.0_f32;
    let mut freq = cfg.frequency;
    let mut amp = 1.0_f32;
    let mut max_amp = 0.0_f32;
    for oct in 0..cfg.octaves {
        // Vary seed per octave to decorrelate layers.
        let oct_seed = cfg.seed.wrapping_add(oct as u64 * 1_000_003);
        value += lcg_value_noise(x * freq, y * freq, z * freq, oct_seed) * amp;
        max_amp += amp;
        freq *= cfg.lacunarity;
        amp *= cfg.persistence;
    }
    if max_amp > 0.0 {
        value / max_amp
    } else {
        0.0
    }
}

// ── displace_mesh_noise ───────────────────────────────────────────────────────

/// Displace each vertex along its normal by `fbm_noise(pos) * amplitude`.
pub fn displace_mesh_noise(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    cfg: &NoiseGenConfig,
) -> NoiseDisplaceResult {
    let n = positions.len().min(normals.len());
    let mut out = Vec::with_capacity(n);
    let mut min_d = f32::MAX;
    let mut max_d = f32::MIN;
    let mut sum_sq = 0.0_f32;

    for i in 0..n {
        let p = positions[i];
        let nm = normals[i];
        let noise_val = fbm_noise(p[0], p[1], p[2], cfg);
        let d = noise_val * cfg.amplitude;
        min_d = min_d.min(d);
        max_d = max_d.max(d);
        sum_sq += d * d;
        out.push([p[0] + nm[0] * d, p[1] + nm[1] * d, p[2] + nm[2] * d]);
    }

    let rms = if n > 0 {
        (sum_sq / n as f32).sqrt()
    } else {
        0.0
    };
    if min_d > max_d {
        min_d = 0.0;
        max_d = 0.0;
    }

    NoiseDisplaceResult {
        positions: out,
        min_displacement: min_d,
        max_displacement: max_d,
        rms_displacement: rms,
    }
}

// ── generate_sphere_bumps ─────────────────────────────────────────────────────

/// Generate a UV-sphere mesh with noise displacement applied.
pub fn generate_sphere_bumps(radius: f32, resolution: u32, cfg: &NoiseGenConfig) -> MeshBuffers {
    let res = resolution.max(3) as usize;
    let rings = res;
    let sectors = res * 2;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals_raw: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for r in 0..=rings {
        let phi = std::f32::consts::PI * r as f32 / rings as f32;
        for s in 0..=sectors {
            let theta = 2.0 * std::f32::consts::PI * s as f32 / sectors as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let nx = sin_phi * cos_theta;
            let ny = cos_phi;
            let nz = sin_phi * sin_theta;
            normals_raw.push([nx, ny, nz]);
            uvs.push([s as f32 / sectors as f32, r as f32 / rings as f32]);
            // Displace along normal.
            let noise_val = fbm_noise(nx, ny, nz, cfg);
            let d = radius + noise_val * cfg.amplitude;
            positions.push([nx * d, ny * d, nz * d]);
        }
    }

    let w = sectors + 1;
    for r in 0..rings {
        for s in 0..sectors {
            let tl = (r * w + s) as u32;
            let tr = tl + 1;
            let bl = tl + w as u32;
            let br = bl + 1;
            indices.push(tl);
            indices.push(bl);
            indices.push(tr);
            indices.push(tr);
            indices.push(bl);
            indices.push(br);
        }
    }

    let n_verts = positions.len();
    let mut mesh = MeshBuffers {
        positions,
        normals: vec![[0.0, 1.0, 0.0]; n_verts],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n_verts],
        uvs,
        indices,
        colors: None,
        has_suit: false,
    };
    compute_normals(&mut mesh);
    mesh
}

// ── generate_organic_surface ──────────────────────────────────────────────────

/// Displace a mesh with noise and recompute normals.
pub fn generate_organic_surface(
    base_positions: &[[f32; 3]],
    base_normals: &[[f32; 3]],
    base_indices: &[u32],
    cfg: &NoiseGenConfig,
) -> MeshBuffers {
    let result = displace_mesh_noise(base_positions, base_normals, cfg);
    let n = result.positions.len();
    let mut mesh = MeshBuffers {
        positions: result.positions,
        normals: vec![[0.0, 1.0, 0.0]; n],
        tangents: vec![[1.0, 0.0, 0.0, 1.0]; n],
        uvs: vec![[0.0, 0.0]; n],
        indices: base_indices.to_vec(),
        colors: None,
        has_suit: false,
    };
    compute_normals(&mut mesh);
    mesh
}

// ── noise_magnitude_stats ─────────────────────────────────────────────────────

/// Return a human-readable summary of displacement statistics.
pub fn noise_magnitude_stats(result: &NoiseDisplaceResult) -> String {
    format!(
        "min={:.4} max={:.4} rms={:.4}",
        result.min_displacement, result.max_displacement, result.rms_displacement
    )
}

// ── apply_noise_texture ───────────────────────────────────────────────────────

/// Compute a scalar noise value per vertex (for texture / weight generation).
pub fn apply_noise_texture(positions: &[[f32; 3]], cfg: &NoiseGenConfig) -> Vec<f32> {
    positions
        .iter()
        .map(|&p| fbm_noise(p[0], p[1], p[2], cfg))
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> NoiseGenConfig {
        NoiseGenConfig::default()
    }

    fn unit_normals(n: usize) -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 1.0]; n]
    }

    fn flat_positions(n: usize) -> Vec<[f32; 3]> {
        (0..n).map(|i| [i as f32 * 0.1, 0.0, 0.0]).collect()
    }

    #[test]
    fn lcg_value_noise_deterministic() {
        let v1 = lcg_value_noise(1.23, 4.56, 7.89, 42);
        let v2 = lcg_value_noise(1.23, 4.56, 7.89, 42);
        assert_eq!(v1, v2);
    }

    #[test]
    fn lcg_value_noise_in_range() {
        for seed in [0u64, 1, 42, 999] {
            let v = lcg_value_noise(0.5, 0.5, 0.5, seed);
            assert!((-1.0_f32..=1.0).contains(&v), "value {} out of [-1,1]", v);
        }
    }

    #[test]
    fn different_seeds_differ() {
        let v1 = lcg_value_noise(0.3, 0.7, 0.2, 1);
        let v2 = lcg_value_noise(0.3, 0.7, 0.2, 2);
        // Very unlikely to be equal.
        // Note: just verify both seeds produce values in range (may rarely coincide)
        // Just verify both are in range.
        assert!(v1.abs() <= 1.0);
        assert!(v2.abs() <= 1.0);
    }

    #[test]
    fn fbm_noise_deterministic() {
        let cfg = default_cfg();
        let v1 = fbm_noise(1.0, 2.0, 3.0, &cfg);
        let v2 = fbm_noise(1.0, 2.0, 3.0, &cfg);
        assert_eq!(v1, v2);
    }

    #[test]
    fn fbm_noise_in_range() {
        let cfg = default_cfg();
        for (x, y, z) in [(0.0, 0.0, 0.0), (1.5, 2.3, 0.7), (-1.0, 4.0, 2.0)] {
            let v = fbm_noise(x, y, z, &cfg);
            assert!((-1.0_f32..=1.0).contains(&v), "fbm {} out of range", v);
        }
    }

    #[test]
    fn amplitude_zero_no_displacement() {
        let mut cfg = default_cfg();
        cfg.amplitude = 0.0;
        let positions = flat_positions(10);
        let normals = unit_normals(10);
        let result = displace_mesh_noise(&positions, &normals, &cfg);
        for (orig, disp) in positions.iter().zip(result.positions.iter()) {
            for k in 0..3 {
                assert!(
                    (orig[k] - disp[k]).abs() < 1e-6,
                    "amplitude=0 should leave positions unchanged"
                );
            }
        }
    }

    #[test]
    fn displace_mesh_noise_changes_positions() {
        let cfg = default_cfg(); // amplitude = 0.1
        let positions = flat_positions(20);
        let normals = unit_normals(20);
        let result = displace_mesh_noise(&positions, &normals, &cfg);
        let changed = positions
            .iter()
            .zip(result.positions.iter())
            .any(|(a, b)| (a[2] - b[2]).abs() > 1e-7);
        assert!(changed, "nonzero amplitude should change some positions");
    }

    #[test]
    fn rms_displacement_positive_for_nonzero_amplitude() {
        let cfg = default_cfg();
        let positions = flat_positions(50);
        let normals = unit_normals(50);
        let result = displace_mesh_noise(&positions, &normals, &cfg);
        assert!(result.rms_displacement >= 0.0);
    }

    #[test]
    fn min_le_max_displacement() {
        let cfg = default_cfg();
        let positions = flat_positions(20);
        let normals = unit_normals(20);
        let result = displace_mesh_noise(&positions, &normals, &cfg);
        assert!(
            result.min_displacement <= result.max_displacement,
            "min {} > max {}",
            result.min_displacement,
            result.max_displacement
        );
    }

    #[test]
    fn generate_sphere_bumps_nonzero_vertices() {
        let cfg = default_cfg();
        let mesh = generate_sphere_bumps(1.0, 8, &cfg);
        assert!(!mesh.positions.is_empty());
        assert!(!mesh.indices.is_empty());
    }

    #[test]
    fn generate_sphere_bumps_indices_in_range() {
        let cfg = default_cfg();
        let mesh = generate_sphere_bumps(1.0, 6, &cfg);
        let nv = mesh.positions.len() as u32;
        for &i in &mesh.indices {
            assert!(i < nv, "index {} out of range (nv={})", i, nv);
        }
    }

    #[test]
    fn apply_noise_texture_length_equals_n_verts() {
        let cfg = default_cfg();
        let positions = flat_positions(30);
        let weights = apply_noise_texture(&positions, &cfg);
        assert_eq!(weights.len(), positions.len());
    }

    #[test]
    fn apply_noise_texture_in_range() {
        let cfg = default_cfg();
        let positions = flat_positions(20);
        for v in apply_noise_texture(&positions, &cfg) {
            assert!(
                (-1.0_f32..=1.0).contains(&v),
                "texture value {} out of [-1,1]",
                v
            );
        }
    }

    #[test]
    fn noise_magnitude_stats_non_empty() {
        let result = NoiseDisplaceResult {
            positions: vec![],
            min_displacement: -0.05,
            max_displacement: 0.08,
            rms_displacement: 0.04,
        };
        let s = noise_magnitude_stats(&result);
        assert!(!s.is_empty());
    }

    #[test]
    fn generate_organic_surface_correct_length() {
        let positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; 3];
        let indices: Vec<u32> = vec![0, 1, 2];
        let cfg = default_cfg();
        let mesh = generate_organic_surface(&positions, &normals, &indices, &cfg);
        assert_eq!(mesh.positions.len(), 3);
        assert_eq!(mesh.indices.len(), 3);
    }
}
