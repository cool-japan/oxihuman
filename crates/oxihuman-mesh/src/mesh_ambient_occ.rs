// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Per-vertex ambient occlusion estimate using bent-normal method.

use std::f32::consts::PI;

/// Configuration for AO estimation.
#[allow(dead_code)]
pub struct AmbientOccV2Config {
    pub sample_count: usize,
    pub max_distance: f32,
    pub bias: f32,
}

impl Default for AmbientOccV2Config {
    fn default() -> Self {
        Self {
            sample_count: 16,
            max_distance: 1.0,
            bias: 0.01,
        }
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Generate hemisphere samples using a deterministic Hammersley-like pattern.
#[allow(dead_code)]
pub fn hemisphere_samples_v2(count: usize) -> Vec<[f32; 3]> {
    let mut samples = Vec::with_capacity(count);
    for i in 0..count {
        let phi = (2.0 * PI * i as f32) / count as f32;
        let cos_theta = 1.0 - (i as f32 + 0.5) / count as f32;
        let sin_theta = (1.0 - cos_theta * cos_theta).max(0.0).sqrt();
        samples.push([sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta]);
    }
    samples
}

/// Transform sample from Z-up to align with a given normal.
fn transform_to_normal(sample: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let up = if normal[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let tangent = normalize3(cross3(up, normal));
    let bitangent = cross3(normal, tangent);
    [
        sample[0] * tangent[0] + sample[1] * bitangent[0] + sample[2] * normal[0],
        sample[0] * tangent[1] + sample[1] * bitangent[1] + sample[2] * normal[1],
        sample[0] * tangent[2] + sample[1] * bitangent[2] + sample[2] * normal[2],
    ]
}

/// Check if a ray from `origin` in `direction` hits any triangle.
fn ray_hits_any(
    origin: [f32; 3],
    direction: [f32; 3],
    positions: &[[f32; 3]],
    indices: &[u32],
    max_dist: f32,
) -> bool {
    let n_tri = indices.len() / 3;
    for t in 0..n_tri {
        let p0 = positions[indices[t * 3] as usize];
        let p1 = positions[indices[t * 3 + 1] as usize];
        let p2 = positions[indices[t * 3 + 2] as usize];
        let e1 = sub3(p1, p0);
        let e2 = sub3(p2, p0);
        let h = cross3(direction, e2);
        let a = dot3(e1, h);
        if a.abs() < 1e-9 {
            continue;
        }
        let f = 1.0 / a;
        let s = sub3(origin, p0);
        let u = f * dot3(s, h);
        if !(0.0..=1.0).contains(&u) {
            continue;
        }
        let q = cross3(s, e1);
        let v = f * dot3(direction, q);
        if v < 0.0 || u + v > 1.0 {
            continue;
        }
        let t_dist = f * dot3(e2, q);
        if t_dist > 1e-4 && t_dist < max_dist {
            return true;
        }
    }
    false
}

/// Compute per-vertex AO using bent-normal hemisphere sampling.
#[allow(dead_code)]
pub fn compute_vertex_ao(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    indices: &[u32],
    config: &AmbientOccV2Config,
) -> Vec<f32> {
    let n = positions.len();
    let mut ao = vec![0.0_f32; n];
    let samples = hemisphere_samples_v2(config.sample_count);
    for i in 0..n {
        let pos = positions[i];
        let normal = if i < normals.len() {
            normals[i]
        } else {
            [0.0, 1.0, 0.0]
        };
        let norm_n = normalize3(normal);
        let mut occluded = 0.0_f32;
        for s in &samples {
            let dir = transform_to_normal(*s, norm_n);
            let origin = [
                pos[0] + norm_n[0] * config.bias,
                pos[1] + norm_n[1] * config.bias,
                pos[2] + norm_n[2] * config.bias,
            ];
            if ray_hits_any(origin, dir, positions, indices, config.max_distance) {
                occluded += 1.0;
            }
        }
        ao[i] = 1.0 - occluded / config.sample_count as f32;
    }
    ao
}

/// Apply AO to vertex colors (multiply RGB by AO factor).
#[allow(dead_code)]
pub fn apply_ao_to_colors(colors: &mut [[f32; 4]], ao: &[f32]) {
    for (i, c) in colors.iter_mut().enumerate() {
        if i < ao.len() {
            c[0] *= ao[i];
            c[1] *= ao[i];
            c[2] *= ao[i];
        }
    }
}

/// Average AO value.
#[allow(dead_code)]
pub fn average_ao(ao: &[f32]) -> f32 {
    if ao.is_empty() {
        return 1.0;
    }
    ao.iter().sum::<f32>() / ao.len() as f32
}

/// Bent normal: direction of least occlusion (approximated as average unoccluded direction).
#[allow(dead_code)]
pub fn bent_normal_at(
    pos: [f32; 3],
    normal: [f32; 3],
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &AmbientOccV2Config,
) -> [f32; 3] {
    let samples = hemisphere_samples_v2(config.sample_count);
    let norm_n = normalize3(normal);
    let mut bent = [0.0_f32; 3];
    for s in &samples {
        let dir = transform_to_normal(*s, norm_n);
        let origin = [
            pos[0] + norm_n[0] * config.bias,
            pos[1] + norm_n[1] * config.bias,
            pos[2] + norm_n[2] * config.bias,
        ];
        if !ray_hits_any(origin, dir, positions, indices, config.max_distance) {
            bent[0] += dir[0];
            bent[1] += dir[1];
            bent[2] += dir[2];
        }
    }
    normalize3(bent)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_plane_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normals = vec![[0.0, 0.0, 1.0]; 4];
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3];
        (positions, normals, indices)
    }

    #[test]
    fn ao_vector_size_correct() {
        let (pos, norm, idx) = flat_plane_mesh();
        let config = AmbientOccV2Config::default();
        let ao = compute_vertex_ao(&pos, &norm, &idx, &config);
        assert_eq!(ao.len(), pos.len());
    }

    #[test]
    fn ao_values_in_range() {
        let (pos, norm, idx) = flat_plane_mesh();
        let config = AmbientOccV2Config::default();
        let ao = compute_vertex_ao(&pos, &norm, &idx, &config);
        for v in &ao {
            assert!((0.0..=1.0).contains(v), "ao value out of range: {v}");
        }
    }

    #[test]
    fn hemisphere_samples_v2_count() {
        let s = hemisphere_samples_v2(16);
        assert_eq!(s.len(), 16);
    }

    #[test]
    fn hemisphere_samples_v2_unit() {
        for s in hemisphere_samples_v2(8) {
            let l = (s[0] * s[0] + s[1] * s[1] + s[2] * s[2]).sqrt();
            assert!((l - 1.0).abs() < 1e-5, "not unit: {l}");
        }
    }

    #[test]
    fn average_ao_in_range() {
        let (pos, norm, idx) = flat_plane_mesh();
        let config = AmbientOccV2Config::default();
        let ao = compute_vertex_ao(&pos, &norm, &idx, &config);
        let avg = average_ao(&ao);
        assert!((0.0..=1.0).contains(&avg));
    }

    #[test]
    fn apply_ao_scales_colors() {
        let mut colors = vec![[1.0_f32; 4]; 4];
        let ao = vec![0.5; 4];
        apply_ao_to_colors(&mut colors, &ao);
        for c in &colors {
            assert!((c[0] - 0.5).abs() < 1e-5);
        }
    }

    #[test]
    fn bent_normal_unit_length() {
        let (pos, norm, idx) = flat_plane_mesh();
        let config = AmbientOccV2Config::default();
        let bn = bent_normal_at(pos[0], norm[0], &pos, &idx, &config);
        let l = (bn[0] * bn[0] + bn[1] * bn[1] + bn[2] * bn[2]).sqrt();
        assert!(
            (l - 1.0).abs() < 1e-5 || l < 1e-9,
            "bent normal not unit: {l}"
        );
    }

    #[test]
    fn default_config_positive() {
        let c = AmbientOccV2Config::default();
        assert!(c.sample_count > 0);
        assert!(c.max_distance > 0.0);
    }

    #[test]
    fn hemisphere_samples_upper_hemisphere() {
        for s in hemisphere_samples_v2(8) {
            assert!(s[2] >= 0.0, "sample below equator: z={}", s[2]);
        }
    }

    #[test]
    fn average_ao_empty() {
        let avg = average_ao(&[]);
        assert!((avg - 1.0).abs() < 1e-5);
    }
}
