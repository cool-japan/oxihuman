// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Principal curvature directions and magnitudes.

/// Principal curvature data for a single vertex.
#[derive(Clone, Debug, Default)]
pub struct PrincipalCurvature {
    pub k1: f32,
    pub k2: f32,
    pub dir1: [f32; 3],
    pub dir2: [f32; 3],
}

impl PrincipalCurvature {
    /// Mean curvature H = (k1 + k2) / 2.
    pub fn mean_curvature(&self) -> f32 {
        (self.k1 + self.k2) * 0.5
    }

    /// Gaussian curvature K = k1 * k2.
    pub fn gaussian_curvature(&self) -> f32 {
        self.k1 * self.k2
    }
}

fn dot3_pc(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3_pc(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize3_pc(v: [f32; 3]) -> [f32; 3] {
    let len = dot3_pc(v, v).sqrt().max(1e-12);
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Estimate per-vertex principal curvatures using a simple quadric fitting approach.
///
/// For each vertex, we fit a height field in the local tangent frame and
/// extract κ₁, κ₂ via the Weingarten map approximation.
pub fn compute_principal_curvatures(
    positions: &[[f32; 3]],
    indices: &[u32],
) -> Vec<PrincipalCurvature> {
    let n = positions.len();
    // Compute per-face normals and accumulate per-vertex smooth normal
    let tri_count = indices.len() / 3;
    let mut normals = vec![[0.0_f32; 3]; n];

    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let fn_ = cross3_pc(ab, ac);
        for &vi in &[ia, ib, ic] {
            normals[vi][0] += fn_[0];
            normals[vi][1] += fn_[1];
            normals[vi][2] += fn_[2];
        }
    }
    let normals: Vec<[f32; 3]> = normals.iter().map(|&n| normalize3_pc(n)).collect();

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

    let mut result = Vec::with_capacity(n);
    for vi in 0..n {
        let p = positions[vi];
        let n_i = normals[vi];

        // Build local tangent frame
        let t0 = if n_i[0].abs() < 0.9 {
            normalize3_pc(cross3_pc(n_i, [1.0, 0.0, 0.0]))
        } else {
            normalize3_pc(cross3_pc(n_i, [0.0, 1.0, 0.0]))
        };
        let t1 = normalize3_pc(cross3_pc(n_i, t0));

        // Fit quadric height field over neighbours: h = a*u^2 + b*u*v + c*v^2
        // Using simplified least-squares (diagonal normal equations)
        let mut sum_u2u2 = 0.0_f32;
        let mut sum_u2v2 = 0.0_f32;
        let mut sum_v2v2 = 0.0_f32;
        let mut sum_hu2 = 0.0_f32;
        let mut sum_huv = 0.0_f32;
        let mut sum_hv2 = 0.0_f32;

        for &nj in &adj[vi] {
            let q = positions[nj];
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let u = dot3_pc(d, t0);
            let v = dot3_pc(d, t1);
            let h = dot3_pc(d, n_i);
            sum_u2u2 += u * u * u * u;
            sum_u2v2 += u * u * v * v;
            sum_v2v2 += v * v * v * v;
            sum_hu2 += h * u * u;
            sum_huv += h * u * v;
            sum_hv2 += h * v * v;
        }

        // 3×3 normal equations [A, B, C]^T where h ≈ A*u² + B*u*v + C*v²
        // Simplified: treat as diagonal to get rough estimates
        let denom_a = sum_u2u2.max(1e-12);
        let denom_c = sum_v2v2.max(1e-12);
        let a_coeff = sum_hu2 / denom_a;
        let c_coeff = sum_hv2 / denom_c;
        let _b_coeff = if (sum_u2v2).abs() > 1e-12 {
            sum_huv / sum_u2v2
        } else {
            0.0
        };

        // Principal curvatures are eigenvalues of [[2a, b],[b, 2c]]
        // k1 = 2a, k2 = 2c (simplified, ignoring b)
        let k1 = 2.0 * a_coeff;
        let k2 = 2.0 * c_coeff;
        let (k1, k2, dir1, dir2) = if k1 >= k2 {
            (k1, k2, t0, t1)
        } else {
            (k2, k1, t1, t0)
        };

        result.push(PrincipalCurvature { k1, k2, dir1, dir2 });
    }
    result
}

/// Return the maximum k1 across all vertices.
pub fn max_k1(curves: &[PrincipalCurvature]) -> f32 {
    curves
        .iter()
        .map(|c| c.k1)
        .fold(f32::NEG_INFINITY, f32::max)
}

/// Return the average mean curvature.
pub fn avg_mean_curvature(curves: &[PrincipalCurvature]) -> f32 {
    if curves.is_empty() {
        return 0.0;
    }
    curves.iter().map(|c| c.mean_curvature()).sum::<f32>() / curves.len() as f32
}

/// Return the average Gaussian curvature.
pub fn avg_gaussian_curvature(curves: &[PrincipalCurvature]) -> f32 {
    if curves.is_empty() {
        return 0.0;
    }
    curves.iter().map(|c| c.gaussian_curvature()).sum::<f32>() / curves.len() as f32
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
    fn principal_curvature_count_matches() {
        let (pos, idx) = flat_quad();
        let curves = compute_principal_curvatures(&pos, &idx);
        assert_eq!(curves.len(), pos.len());
    }

    #[test]
    fn principal_curvature_k1_ge_k2() {
        let (pos, idx) = flat_quad();
        let curves = compute_principal_curvatures(&pos, &idx);
        for c in &curves {
            assert!(c.k1 >= c.k2 || (c.k1 - c.k2).abs() < 1e-4);
        }
    }

    #[test]
    fn flat_mesh_near_zero_curvature() {
        let (pos, idx) = flat_quad();
        let curves = compute_principal_curvatures(&pos, &idx);
        let mean = avg_mean_curvature(&curves);
        /* For a flat plane, curvature should be 0 */
        assert!(mean.abs() < 1e-3, "mean curvature on flat mesh={mean}");
    }

    #[test]
    fn gaussian_curvature_flat_is_zero() {
        let (pos, idx) = flat_quad();
        let curves = compute_principal_curvatures(&pos, &idx);
        let gauss = avg_gaussian_curvature(&curves);
        assert!(gauss.abs() < 1e-3);
    }

    #[test]
    fn directions_finite() {
        let (pos, idx) = flat_quad();
        let curves = compute_principal_curvatures(&pos, &idx);
        for c in &curves {
            assert!(c.dir1.iter().all(|v| v.is_finite()));
            assert!(c.dir2.iter().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn max_k1_is_finite() {
        let (pos, idx) = flat_quad();
        let curves = compute_principal_curvatures(&pos, &idx);
        assert!(max_k1(&curves).is_finite());
    }

    #[test]
    fn empty_mesh_empty_result() {
        let curves = compute_principal_curvatures(&[], &[]);
        assert!(curves.is_empty());
    }

    #[test]
    fn mean_curvature_fn() {
        let c = PrincipalCurvature {
            k1: 2.0,
            k2: 0.0,
            ..Default::default()
        };
        assert!((c.mean_curvature() - 1.0).abs() < 1e-6);
    }
}
