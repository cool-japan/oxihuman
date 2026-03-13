// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Anisotropic mesh smoothing (bilateral-style).

/// Config for anisotropic smoothing.
#[derive(Clone, Debug)]
pub struct AnisoSmoothConfig {
    pub iterations: usize,
    /// Spatial influence range (σ_s).
    pub sigma_spatial: f32,
    /// Feature influence range (σ_n): normals within this angle are blended.
    pub sigma_normal: f32,
}

impl Default for AnisoSmoothConfig {
    fn default() -> Self {
        Self {
            iterations: 5,
            sigma_spatial: 0.1,
            sigma_normal: 0.5,
        }
    }
}

/// Result of anisotropic smoothing.
#[derive(Clone, Debug, Default)]
pub struct AnisoSmoothResult {
    pub positions: Vec<[f32; 3]>,
    pub iterations_done: usize,
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3_as(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize3_as(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v).max(1e-12);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn face_normal_as(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    normalize3_as([
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ])
}

/// Perform one bilateral smoothing step.
pub fn aniso_smooth_step(
    positions: &[[f32; 3]],
    indices: &[u32],
    sigma_spatial: f32,
    sigma_normal: f32,
) -> Vec<[f32; 3]> {
    let n = positions.len();
    let tri_count = indices.len() / 3;

    // Compute per-vertex normals
    let mut normals = vec![[0.0_f32; 3]; n];
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        let fn_ = face_normal_as(positions[ia], positions[ib], positions[ic]);
        for &vi in &[ia, ib, ic] {
            normals[vi][0] += fn_[0];
            normals[vi][1] += fn_[1];
            normals[vi][2] += fn_[2];
        }
    }
    let normals: Vec<[f32; 3]> = normals.iter().map(|&nv| normalize3_as(nv)).collect();

    // Build adjacency
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for t in 0..tri_count {
        let verts = [
            indices[t * 3] as usize,
            indices[t * 3 + 1] as usize,
            indices[t * 3 + 2] as usize,
        ];
        for k in 0..3 {
            let a = verts[k];
            let b = verts[(k + 1) % 3];
            if !adj[a].contains(&b) {
                adj[a].push(b);
            }
            if !adj[b].contains(&a) {
                adj[b].push(a);
            }
        }
    }

    let ss2 = (sigma_spatial * sigma_spatial).max(1e-12);
    let sn2 = (sigma_normal * sigma_normal).max(1e-12);

    let mut result = positions.to_vec();
    for vi in 0..n {
        let p = positions[vi];
        let ni = normals[vi];
        let mut sum_w = 0.0_f32;
        let mut acc = [0.0_f32; 3];

        for &nj in &adj[vi] {
            let q = positions[nj];
            let d = sub3(q, p);
            let dist2 = dot3_as(d, d);
            let nj_n = normals[nj];
            let cos_angle = dot3_as(ni, nj_n).clamp(-1.0, 1.0);
            let angle = cos_angle.acos();
            let w_s = (-dist2 / (2.0 * ss2)).exp();
            let w_n = (-(angle * angle) / (2.0 * sn2)).exp();
            let w = w_s * w_n;
            acc[0] += w * q[0];
            acc[1] += w * q[1];
            acc[2] += w * q[2];
            sum_w += w;
        }

        if sum_w > 1e-10 {
            result[vi] = [acc[0] / sum_w, acc[1] / sum_w, acc[2] / sum_w];
        }
    }
    result
}

/// Run anisotropic smoothing for multiple iterations.
pub fn anisotropic_smooth(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: &AnisoSmoothConfig,
) -> AnisoSmoothResult {
    let mut pos = positions.to_vec();
    for _ in 0..config.iterations {
        pos = aniso_smooth_step(&pos, indices, config.sigma_spatial, config.sigma_normal);
    }
    AnisoSmoothResult {
        positions: pos,
        iterations_done: config.iterations,
    }
}

/// Return vertex count.
pub fn aniso_vertex_count(r: &AnisoSmoothResult) -> usize {
    r.positions.len()
}

/// Compute average displacement from original positions.
pub fn aniso_avg_displacement(original: &[[f32; 3]], result: &AnisoSmoothResult) -> f32 {
    if original.is_empty() {
        return 0.0;
    }
    let sum: f32 = original
        .iter()
        .zip(result.positions.iter())
        .map(|(a, b)| len3(sub3(*b, *a)))
        .sum();
    sum / original.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_quad() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let idx = vec![0, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn config_default_sane() {
        let c = AnisoSmoothConfig::default();
        assert!(c.iterations > 0);
        assert!(c.sigma_spatial > 0.0);
        assert!(c.sigma_normal > 0.0);
    }

    #[test]
    fn smooth_preserves_count() {
        let (pos, idx) = flat_quad();
        let cfg = AnisoSmoothConfig {
            iterations: 1,
            ..Default::default()
        };
        let r = anisotropic_smooth(&pos, &idx, &cfg);
        assert_eq!(aniso_vertex_count(&r), pos.len());
    }

    #[test]
    fn smooth_positions_finite() {
        let (pos, idx) = flat_quad();
        let cfg = AnisoSmoothConfig::default();
        let r = anisotropic_smooth(&pos, &idx, &cfg);
        for p in &r.positions {
            assert!(p.iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn smooth_zero_iterations_unchanged() {
        let (pos, idx) = flat_quad();
        let cfg = AnisoSmoothConfig {
            iterations: 0,
            ..Default::default()
        };
        let r = anisotropic_smooth(&pos, &idx, &cfg);
        assert_eq!(r.positions, pos);
    }

    #[test]
    fn aniso_avg_displacement_nonneg() {
        let (pos, idx) = flat_quad();
        let cfg = AnisoSmoothConfig {
            iterations: 3,
            ..Default::default()
        };
        let r = anisotropic_smooth(&pos, &idx, &cfg);
        assert!(aniso_avg_displacement(&pos, &r) >= 0.0);
    }

    #[test]
    fn smooth_step_count_preserved() {
        let (pos, idx) = flat_quad();
        let next = aniso_smooth_step(&pos, &idx, 0.5, 1.0);
        assert_eq!(next.len(), pos.len());
    }

    #[test]
    fn flat_mesh_stays_flat() {
        let (pos, idx) = flat_quad();
        let cfg = AnisoSmoothConfig {
            iterations: 5,
            ..Default::default()
        };
        let r = anisotropic_smooth(&pos, &idx, &cfg);
        /* All z values should remain ~0 for a flat mesh */
        for p in &r.positions {
            assert!(p[2].abs() < 1e-4, "z={}", p[2]);
        }
    }

    #[test]
    fn iterations_done_stored() {
        let (pos, idx) = flat_quad();
        let cfg = AnisoSmoothConfig {
            iterations: 3,
            ..Default::default()
        };
        let r = anisotropic_smooth(&pos, &idx, &cfg);
        assert_eq!(r.iterations_done, 3);
    }
}
