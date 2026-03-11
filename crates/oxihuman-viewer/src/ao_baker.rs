// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Ambient occlusion baking — ray-based AO computation for mesh vertices
//! using a hemisphere sampling strategy.

use std::f32::consts::{FRAC_1_SQRT_2, PI};

/// AO baker configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AoBakerConfig {
    /// Number of sample directions per vertex.
    pub sample_count: u32,
    /// Maximum ray distance.
    pub max_distance: f32,
    /// Falloff exponent.
    pub falloff: f32,
    /// Bias to avoid self-intersection.
    pub bias: f32,
    /// Whether to use cosine-weighted hemisphere.
    pub cosine_weighted: bool,
}

impl Default for AoBakerConfig {
    fn default() -> Self {
        Self {
            sample_count: 64,
            max_distance: 1.0,
            falloff: 1.0,
            bias: 0.001,
            cosine_weighted: true,
        }
    }
}

/// Result of AO baking for one vertex.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct AoSample {
    pub vertex_index: usize,
    pub occlusion: f32,
}

/// Generate a deterministic hemisphere direction using Hammersley sequence.
///
/// `i` is the sample index, `n` is total samples.
/// Returns a direction `[x, y, z]` in tangent space (y = up).
#[allow(dead_code)]
pub fn hammersley_hemisphere(i: u32, n: u32, cosine_weighted: bool) -> [f32; 3] {
    if n == 0 {
        return [0.0, 1.0, 0.0];
    }
    let xi1 = i as f32 / n as f32;
    let xi2 = radical_inverse_base2(i);

    let phi = 2.0 * PI * xi1;

    let (cos_theta, sin_theta) = if cosine_weighted {
        let cos_theta = xi2.sqrt();
        let sin_theta = (1.0 - xi2).sqrt();
        (cos_theta, sin_theta)
    } else {
        let cos_theta = xi2;
        let sin_theta = (1.0 - xi2 * xi2).sqrt();
        (cos_theta, sin_theta)
    };

    [sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin()]
}

/// Radical inverse in base 2 (van der Corput sequence).
#[allow(dead_code)]
pub fn radical_inverse_base2(mut bits: u32) -> f32 {
    bits = (bits << 16) | (bits >> 16);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
    bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
    bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);
    bits as f32 / 0x1_0000_0000_u64 as f32
}

/// Build a tangent-space basis from a normal vector.
///
/// Returns `(tangent, bitangent)`.
#[allow(dead_code)]
pub fn tangent_basis(normal: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let up = if normal[1].abs() < 0.999 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let tangent = cross(up, normal);
    let tangent = normalize(tangent);
    let bitangent = cross(normal, tangent);
    (tangent, bitangent)
}

/// Transform a tangent-space direction to world space.
#[allow(dead_code)]
pub fn tangent_to_world(
    dir: [f32; 3],
    tangent: [f32; 3],
    normal: [f32; 3],
    bitangent: [f32; 3],
) -> [f32; 3] {
    [
        dir[0] * tangent[0] + dir[1] * normal[0] + dir[2] * bitangent[0],
        dir[0] * tangent[1] + dir[1] * normal[1] + dir[2] * bitangent[1],
        dir[0] * tangent[2] + dir[1] * normal[2] + dir[2] * bitangent[2],
    ]
}

/// Simple ray-triangle intersection (Moller-Trumbore).
///
/// Returns distance if hit, `None` otherwise.
#[allow(dead_code)]
pub fn ray_triangle_intersect(
    origin: [f32; 3],
    dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> Option<f32> {
    let edge1 = sub(v1, v0);
    let edge2 = sub(v2, v0);
    let h = cross(dir, edge2);
    let a = dot(edge1, h);
    if a.abs() < 1e-7 {
        return None;
    }
    let f = 1.0 / a;
    let s = sub(origin, v0);
    let u = f * dot(s, h);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(s, edge1);
    let v = f * dot(dir, q);
    if v < 0.0 || u + v > 1.0 {
        return None;
    }
    let t = f * dot(edge2, q);
    if t > 1e-7 { Some(t) } else { None }
}

/// Compute AO for a single vertex against a set of triangles.
#[allow(dead_code)]
pub fn compute_vertex_ao(
    position: [f32; 3],
    normal: [f32; 3],
    triangles: &[[usize; 3]],
    positions: &[[f32; 3]],
    config: &AoBakerConfig,
) -> f32 {
    let (tangent, bitangent) = tangent_basis(normal);
    let biased_pos = [
        position[0] + normal[0] * config.bias,
        position[1] + normal[1] * config.bias,
        position[2] + normal[2] * config.bias,
    ];

    let mut occluded = 0u32;
    for i in 0..config.sample_count {
        let ts_dir = hammersley_hemisphere(i, config.sample_count, config.cosine_weighted);
        let ws_dir = tangent_to_world(ts_dir, tangent, normal, bitangent);

        let mut hit = false;
        for tri in triangles {
            let v0 = positions[tri[0]];
            let v1 = positions[tri[1]];
            let v2 = positions[tri[2]];
            if let Some(dist) = ray_triangle_intersect(biased_pos, ws_dir, v0, v1, v2) {
                if dist < config.max_distance {
                    hit = true;
                    break;
                }
            }
        }
        if hit {
            occluded += 1;
        }
    }

    if config.sample_count == 0 {
        return 0.0;
    }
    let ao = 1.0 - (occluded as f32 / config.sample_count as f32);
    ao.powf(config.falloff)
}

/// Bake AO for all vertices.
#[allow(dead_code)]
pub fn bake_ao(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    triangles: &[[usize; 3]],
    config: &AoBakerConfig,
) -> Vec<AoSample> {
    let mut results = Vec::with_capacity(positions.len());
    for (i, (pos, nrm)) in positions.iter().zip(normals.iter()).enumerate() {
        let ao = compute_vertex_ao(*pos, *nrm, triangles, positions, config);
        results.push(AoSample {
            vertex_index: i,
            occlusion: ao,
        });
    }
    results
}

// --- helpers ---

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 { return [0.0, 1.0, 0.0]; }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_default_config() {
        let c = AoBakerConfig::default();
        assert_eq!(c.sample_count, 64);
        assert!(c.max_distance > 0.0);
    }

    #[test]
    fn test_radical_inverse_base2_zero() {
        let v = radical_inverse_base2(0);
        assert!(v.abs() < 1e-6);
    }

    #[test]
    fn test_radical_inverse_base2_range() {
        for i in 0..16 {
            let v = radical_inverse_base2(i);
            assert!((0.0..=1.0).contains(&v), "Out of range: {v}");
        }
    }

    #[test]
    fn test_hammersley_hemisphere_y_positive() {
        for i in 0..8 {
            let d = hammersley_hemisphere(i, 8, true);
            assert!(d[1] >= 0.0, "y should be non-negative, got {}", d[1]);
        }
    }

    #[test]
    fn test_hammersley_zero_n() {
        let d = hammersley_hemisphere(0, 0, true);
        assert!((d[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_tangent_basis_up() {
        let (t, b) = tangent_basis([0.0, 1.0, 0.0]);
        let d = dot(t, b);
        assert!(d.abs() < 1e-5, "Tangent and bitangent should be orthogonal");
    }

    #[test]
    fn test_ray_triangle_hit() {
        let origin = [0.0, 0.0, -1.0];
        let dir = [0.0, 0.0, 1.0];
        let v0 = [-1.0, -1.0, 0.0];
        let v1 = [1.0, -1.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let hit = ray_triangle_intersect(origin, dir, v0, v1, v2);
        assert!(hit.is_some());
        assert!((hit.unwrap() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_ray_triangle_miss() {
        let origin = [10.0, 10.0, -1.0];
        let dir = [0.0, 0.0, 1.0];
        let v0 = [-1.0, -1.0, 0.0];
        let v1 = [1.0, -1.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        assert!(ray_triangle_intersect(origin, dir, v0, v1, v2).is_none());
    }

    #[test]
    fn test_bake_ao_empty() {
        let r = bake_ao(&[], &[], &[], &AoBakerConfig::default());
        assert!(r.is_empty());
    }

    #[test]
    fn test_compute_vertex_ao_no_triangles() {
        let ao = compute_vertex_ao(
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            &[],
            &[],
            &AoBakerConfig { sample_count: 8, ..Default::default() },
        );
        assert!((ao - 1.0).abs() < 1e-5, "No geometry means full AO = 1.0");
    }
}
