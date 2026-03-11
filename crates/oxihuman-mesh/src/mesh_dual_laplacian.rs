// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Dual Laplacian (cotangent-weighted) smoothing for triangle meshes.

/// Config for dual Laplacian smoothing.
#[allow(dead_code)]
pub struct DualLaplacianConfig {
    pub iterations: usize,
    pub lambda: f32,
}

impl Default for DualLaplacianConfig {
    fn default() -> Self {
        Self {
            iterations: 5,
            lambda: 0.5,
        }
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Cotangent of angle at vertex opposite to edge (v_a, v_b) in triangle (v_a, v_shared, v_b).
#[allow(dead_code)]
pub fn cotangent_weight(v_shared: [f32; 3], v_a: [f32; 3], v_b: [f32; 3]) -> f32 {
    let ea = sub3(v_a, v_shared);
    let eb = sub3(v_b, v_shared);
    let cos_a = dot3(ea, eb);
    let cross = [
        ea[1] * eb[2] - ea[2] * eb[1],
        ea[2] * eb[0] - ea[0] * eb[2],
        ea[0] * eb[1] - ea[1] * eb[0],
    ];
    let sin_a = length3(cross);
    if sin_a.abs() < 1e-9 {
        0.0
    } else {
        cos_a / sin_a
    }
}

/// Perform dual Laplacian smoothing using cotangent weights.
#[allow(dead_code)]
pub fn dual_laplacian_smooth(
    positions: &mut [[f32; 3]],
    indices: &[u32],
    config: &DualLaplacianConfig,
) {
    let n = positions.len();
    for _ in 0..config.iterations {
        let mut weight_sum = vec![0.0_f32; n];
        let mut delta = vec![[0.0_f32; 3]; n];
        let n_tri = indices.len() / 3;
        for t in 0..n_tri {
            let i0 = indices[t * 3] as usize;
            let i1 = indices[t * 3 + 1] as usize;
            let i2 = indices[t * 3 + 2] as usize;
            let p0 = positions[i0];
            let p1 = positions[i1];
            let p2 = positions[i2];
            let w0 = cotangent_weight(p0, p1, p2).max(0.0);
            let w1 = cotangent_weight(p1, p0, p2).max(0.0);
            let w2 = cotangent_weight(p2, p0, p1).max(0.0);
            for k in 0..3 {
                delta[i0][k] += (w1 + w2) * (p1[k] + p2[k] - 2.0 * p0[k]);
                delta[i1][k] += (w0 + w2) * (p0[k] + p2[k] - 2.0 * p1[k]);
                delta[i2][k] += (w0 + w1) * (p0[k] + p1[k] - 2.0 * p2[k]);
            }
            weight_sum[i0] += w1 + w2;
            weight_sum[i1] += w0 + w2;
            weight_sum[i2] += w0 + w1;
        }
        for i in 0..n {
            let w = weight_sum[i];
            if w > 1e-9 {
                for k in 0..3 {
                    positions[i][k] += config.lambda * delta[i][k] / w;
                }
            }
        }
    }
}

/// Compute the cotangent Laplacian operator value at a vertex.
#[allow(dead_code)]
pub fn laplacian_at_vertex(vertex_idx: usize, positions: &[[f32; 3]], indices: &[u32]) -> [f32; 3] {
    let p = positions[vertex_idx];
    let mut delta = [0.0_f32; 3];
    let mut w_total = 0.0_f32;
    let n_tri = indices.len() / 3;
    for t in 0..n_tri {
        let i0 = indices[t * 3] as usize;
        let i1 = indices[t * 3 + 1] as usize;
        let i2 = indices[t * 3 + 2] as usize;
        let verts = [i0, i1, i2];
        let pos = [positions[i0], positions[i1], positions[i2]];
        for (local, &vi) in verts.iter().enumerate() {
            if vi != vertex_idx {
                continue;
            }
            let j = (local + 1) % 3;
            let k = (local + 2) % 3;
            let w = cotangent_weight(p, pos[j], pos[k]).max(0.0);
            w_total += w;
            for d in 0..3 {
                delta[d] += w * (pos[j][d] + pos[k][d] - 2.0 * p[d]);
            }
        }
    }
    if w_total > 1e-9 {
        [delta[0] / w_total, delta[1] / w_total, delta[2] / w_total]
    } else {
        [0.0; 3]
    }
}

/// Laplacian energy (sum of squared Laplacian vectors).
#[allow(dead_code)]
pub fn laplacian_energy_dual(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let mut energy = 0.0_f32;
    for i in 0..positions.len() {
        let lap = laplacian_at_vertex(i, positions, indices);
        energy += lap[0] * lap[0] + lap[1] * lap[1] + lap[2] * lap[2];
    }
    energy
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_grid_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [2.0, 1.0, 0.0],
        ];
        let indices: Vec<u32> = vec![0, 1, 4, 0, 4, 3, 1, 2, 5, 1, 5, 4];
        (positions, indices)
    }

    #[test]
    fn smooth_does_not_panic() {
        let (mut pos, idx) = flat_grid_mesh();
        let config = DualLaplacianConfig::default();
        dual_laplacian_smooth(&mut pos, &idx, &config);
        assert_eq!(pos.len(), 6);
    }

    #[test]
    fn smooth_zero_iterations_unchanged() {
        let (mut pos, idx) = flat_grid_mesh();
        let orig = pos.clone();
        let config = DualLaplacianConfig {
            iterations: 0,
            lambda: 0.5,
        };
        dual_laplacian_smooth(&mut pos, &idx, &config);
        for (a, b) in pos.iter().zip(orig.iter()) {
            for k in 0..3 {
                assert!((a[k] - b[k]).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn cotangent_weight_right_angle() {
        let v_shared = [0.0, 0.0, 0.0];
        let v_a = [1.0, 0.0, 0.0];
        let v_b = [0.0, 1.0, 0.0];
        let w = cotangent_weight(v_shared, v_a, v_b);
        assert!(w.abs() < 1e-5, "cot(90) should be ~0, got {w}");
    }

    #[test]
    fn laplacian_at_interior_vertex() {
        let (pos, idx) = flat_grid_mesh();
        let lap = laplacian_at_vertex(4, &pos, &idx);
        assert!(lap[0].abs() < 5.0 && lap[1].abs() < 5.0);
    }

    #[test]
    fn laplacian_energy_nonneg() {
        let (pos, idx) = flat_grid_mesh();
        let e = laplacian_energy_dual(&pos, &idx);
        assert!(e >= 0.0);
    }

    #[test]
    fn default_config_reasonable() {
        let c = DualLaplacianConfig::default();
        assert!(c.iterations > 0);
        assert!((0.0..=1.0).contains(&c.lambda));
    }

    #[test]
    fn smooth_preserves_vertex_count() {
        let (mut pos, idx) = flat_grid_mesh();
        let n = pos.len();
        let config = DualLaplacianConfig::default();
        dual_laplacian_smooth(&mut pos, &idx, &config);
        assert_eq!(pos.len(), n);
    }

    #[test]
    fn laplacian_flat_mesh_near_zero() {
        let (pos, idx) = flat_grid_mesh();
        let lap = laplacian_at_vertex(4, &pos, &idx);
        assert!(lap[2].abs() < 1e-5, "Z laplacian on flat mesh should be ~0");
    }

    #[test]
    fn cotangent_weight_acute_positive() {
        let v_shared = [0.0, 0.0, 0.0];
        let v_a = [1.0, 0.1, 0.0];
        let v_b = [0.1, 1.0, 0.0];
        let w = cotangent_weight(v_shared, v_a, v_b);
        assert!(w >= 0.0);
    }
}
