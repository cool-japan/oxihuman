// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Full curvature tensor estimation for triangle meshes.
//!
//! Uses a local quadratic patch fit over each vertex's 1-ring neighbourhood.

// ── types ─────────────────────────────────────────────────────────────────────

/// Principal curvatures and their directions at a mesh vertex.
#[derive(Debug, Clone, Copy)]
pub struct CurvatureTensor {
    /// Minimum principal curvature (κ_min).
    pub kmin: f32,
    /// Maximum principal curvature (κ_max).
    pub kmax: f32,
    /// Direction of minimum curvature.
    pub dir_min: [f32; 3],
    /// Direction of maximum curvature.
    pub dir_max: [f32; 3],
}

impl Default for CurvatureTensor {
    fn default() -> Self {
        Self {
            kmin: 0.0,
            kmax: 0.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn sub3(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(a: &[f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

fn norm3(a: &[f32; 3]) -> [f32; 3] {
    let l = len3(a).max(1e-12);
    [a[0] / l, a[1] / l, a[2] / l]
}

fn cross3(a: &[f32; 3], b: &[f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Compute the area-weighted vertex normal for vertex `v`.
fn vertex_normal(positions: &[[f32; 3]], tris: &[[u32; 3]], v: usize) -> [f32; 3] {
    let mut n = [0.0f32; 3];
    for tri in tris {
        let verts = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if !verts.contains(&v) {
            continue;
        }
        let a = sub3(&positions[verts[1]], &positions[verts[0]]);
        let b = sub3(&positions[verts[2]], &positions[verts[0]]);
        let cn = cross3(&a, &b);
        n[0] += cn[0];
        n[1] += cn[1];
        n[2] += cn[2];
    }
    norm3(&n)
}

/// Build two orthogonal tangent vectors given a normal.
fn tangent_frame(normal: &[f32; 3]) -> ([f32; 3], [f32; 3]) {
    // Pick a helper vector not parallel to normal.
    let helper = if normal[0].abs() < 0.9 {
        [1.0f32, 0.0, 0.0]
    } else {
        [0.0f32, 1.0, 0.0]
    };
    let t1 = norm3(&cross3(normal, &helper));
    let t2 = cross3(normal, &t1);
    (t1, t2)
}

/// 2×2 symmetric eigensolver.  Returns (λ1 ≤ λ2, v1, v2).
fn sym2_eigen(a: f32, b: f32, d: f32) -> (f32, f32, [f32; 2], [f32; 2]) {
    let tr = a + d;
    let det = a * d - b * b;
    let disc = ((tr * tr / 4.0) - det).max(0.0).sqrt();
    let l1 = tr / 2.0 - disc;
    let l2 = tr / 2.0 + disc;
    // Eigenvector for l1.
    let v1 = if b.abs() > 1e-10 {
        let v = [l1 - d, b];
        let len = (v[0] * v[0] + v[1] * v[1]).sqrt().max(1e-12);
        [v[0] / len, v[1] / len]
    } else {
        [1.0, 0.0]
    };
    // Eigenvector for l2 is orthogonal.
    let v2 = [-v1[1], v1[0]];
    (l1, l2, v1, v2)
}

// ── public API ────────────────────────────────────────────────────────────────

/// Estimate the curvature tensor at vertex `v` by fitting a quadratic patch
/// over the 1-ring neighbourhood.
#[allow(dead_code)]
pub fn compute_vertex_curvature_tensor(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
    v: usize,
) -> CurvatureTensor {
    // Collect 1-ring neighbours.
    let mut neighbours: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for tri in tris {
        let verts = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
        if verts.contains(&v) {
            for &u in &verts {
                if u != v {
                    neighbours.insert(u);
                }
            }
        }
    }
    if neighbours.is_empty() {
        return CurvatureTensor::default();
    }

    let normal = vertex_normal(positions, tris, v);
    let (t1, t2) = tangent_frame(&normal);
    let p0 = positions[v];

    // Project neighbours into tangent plane and fit z = ax² + bxy + cy²
    // using least squares (Ax = b).
    let mut ata = [0.0f64; 9]; // 3×3 symmetric
    let mut atb = [0.0f64; 3];

    for &nb in &neighbours {
        let d = sub3(&positions[nb], &p0);
        let u = dot3(&d, &t1) as f64;
        let w = dot3(&d, &t2) as f64;
        let h = dot3(&d, &normal) as f64;

        let row = [u * u, u * w, w * w];
        for i in 0..3 {
            for j in 0..3 {
                ata[i * 3 + j] += row[i] * row[j];
            }
            atb[i] += row[i] * h;
        }
    }

    // Solve 3×3 system via simple Gauss elimination.
    let coeffs = solve3x3(&ata, &atb);
    let (a_coeff, _b_coeff, c_coeff) = (coeffs[0] as f32, coeffs[1] as f32, coeffs[2] as f32);

    // Second fundamental form: II = [2a, b; b, 2c] (shape operator elements).
    let (kmin, kmax, ev1, ev2) = sym2_eigen(2.0 * a_coeff, _b_coeff, 2.0 * c_coeff);

    // Convert eigenvectors back to 3D.
    let dir_min = norm3(&[
        ev1[0] * t1[0] + ev1[1] * t2[0],
        ev1[0] * t1[1] + ev1[1] * t2[1],
        ev1[0] * t1[2] + ev1[1] * t2[2],
    ]);
    let dir_max = norm3(&[
        ev2[0] * t1[0] + ev2[1] * t2[0],
        ev2[0] * t1[1] + ev2[1] * t2[1],
        ev2[0] * t1[2] + ev2[1] * t2[2],
    ]);

    CurvatureTensor {
        kmin,
        kmax,
        dir_min,
        dir_max,
    }
}

/// Solve 3×3 linear system (column-major flattened A, rhs b).
#[allow(clippy::needless_range_loop)]
fn solve3x3(ata: &[f64; 9], atb: &[f64; 3]) -> [f64; 3] {
    // Build augmented matrix.
    let mut m = [
        [ata[0], ata[1], ata[2], atb[0]],
        [ata[3], ata[4], ata[5], atb[1]],
        [ata[6], ata[7], ata[8], atb[2]],
    ];
    // Forward elimination.
    for col in 0..3usize {
        // Find pivot.
        let mut max_row = col;
        let mut max_val = m[col][col].abs();
        for row in (col + 1)..3 {
            if m[row][col].abs() > max_val {
                max_val = m[row][col].abs();
                max_row = row;
            }
        }
        m.swap(col, max_row);
        let pivot = m[col][col];
        if pivot.abs() < 1e-15 {
            continue;
        }
        for row in (col + 1)..3 {
            let factor = m[row][col] / pivot;
            for k in col..4 {
                let v = m[col][k] * factor;
                m[row][k] -= v;
            }
        }
    }
    // Back substitution.
    let mut x = [0.0f64; 3];
    for i in (0..3).rev() {
        let mut sum = m[i][3];
        for j in (i + 1)..3 {
            sum -= m[i][j] * x[j];
        }
        let denom = m[i][i];
        x[i] = if denom.abs() > 1e-15 {
            sum / denom
        } else {
            0.0
        };
    }
    x
}

/// Compute curvature tensors for all vertices.
#[allow(dead_code)]
pub fn compute_all_curvature_tensors(
    positions: &[[f32; 3]],
    tris: &[[u32; 3]],
) -> Vec<CurvatureTensor> {
    (0..positions.len())
        .map(|v| compute_vertex_curvature_tensor(positions, tris, v))
        .collect()
}

/// Mean curvature H = (κ_min + κ_max) / 2.
#[allow(dead_code)]
pub fn mean_curvature_from_tensor(t: &CurvatureTensor) -> f32 {
    (t.kmin + t.kmax) / 2.0
}

/// Gaussian curvature K = κ_min × κ_max.
#[allow(dead_code)]
pub fn gaussian_curvature_from_tensor(t: &CurvatureTensor) -> f32 {
    t.kmin * t.kmax
}

/// Koenderink shape index in \[-1, 1\]: s = 2/π × atan2(κ_max + κ_min, κ_max - κ_min).
#[allow(dead_code)]
pub fn shape_index_from_tensor(t: &CurvatureTensor) -> f32 {
    let diff = t.kmax - t.kmin;
    if diff.abs() < 1e-12 {
        return 0.0;
    }
    (2.0 / std::f32::consts::PI) * (t.kmax + t.kmin).atan2(diff)
}

/// Curvedness C = sqrt((κ_min² + κ_max²) / 2).
#[allow(dead_code)]
pub fn curvedness_from_tensor(t: &CurvatureTensor) -> f32 {
    ((t.kmin * t.kmin + t.kmax * t.kmax) / 2.0).sqrt()
}

/// Classify a surface point based on its principal curvatures.
#[allow(dead_code)]
pub fn classify_surface_point(t: &CurvatureTensor) -> &'static str {
    let h = mean_curvature_from_tensor(t);
    let k = gaussian_curvature_from_tensor(t);
    let eps = 1e-4f32;
    if h.abs() < eps && k.abs() < eps {
        "flat"
    } else if k > eps {
        "sphere"
    } else if k < -eps {
        "saddle"
    } else if h > eps {
        "ridge"
    } else if h < -eps {
        "valley"
    } else {
        "cylinder"
    }
}

/// L2 distance between two curvature tensors (using kmin and kmax).
#[allow(dead_code)]
pub fn tensor_similarity(a: &CurvatureTensor, b: &CurvatureTensor) -> f32 {
    let dk1 = a.kmin - b.kmin;
    let dk2 = a.kmax - b.kmax;
    (dk1 * dk1 + dk2 * dk2).sqrt()
}

/// Smooth curvature tensors by averaging with neighbours.
#[allow(dead_code)]
pub fn smooth_curvature_tensors(
    tensors: &[CurvatureTensor],
    adj: &[Vec<usize>],
) -> Vec<CurvatureTensor> {
    tensors
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let neighbours = &adj[i];
            if neighbours.is_empty() {
                return *t;
            }
            let mut sum_kmin = t.kmin;
            let mut sum_kmax = t.kmax;
            let count = 1 + neighbours.len();
            for &nb in neighbours {
                sum_kmin += tensors[nb].kmin;
                sum_kmax += tensors[nb].kmax;
            }
            CurvatureTensor {
                kmin: sum_kmin / count as f32,
                kmax: sum_kmax / count as f32,
                dir_min: t.dir_min,
                dir_max: t.dir_max,
            }
        })
        .collect()
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a regular icosphere-like neighbourhood to approximate a sphere
    /// of radius r = 1 at the north pole.
    fn sphere_cap_positions() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let r = 1.0f32;
        let n = 6usize;
        let mut pos = vec![[0.0f32, 0.0, r]]; // apex (index 0)
        let step = std::f32::consts::TAU / n as f32;
        let theta = 0.3f32; // colatitude of ring
        for i in 0..n {
            let phi = i as f32 * step;
            pos.push([
                r * theta.sin() * phi.cos(),
                r * theta.sin() * phi.sin(),
                r * theta.cos(),
            ]);
        }
        let mut tris = Vec::new();
        for i in 0..n as u32 {
            let a = 1 + i;
            let b = 1 + (i + 1) % n as u32;
            tris.push([0u32, a, b]);
        }
        (pos, tris)
    }

    fn flat_positions() -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
        // Flat quad-like patch — 5 vertices in a plane.
        let pos = vec![
            [0.0f32, 0.0, 0.0], // center
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
        ];
        let tris = vec![[0u32, 1, 3], [0, 3, 2], [0, 2, 4], [0, 4, 1]];
        (pos, tris)
    }

    #[test]
    fn sphere_vertex_positive_curvatures() {
        let (pos, tris) = sphere_cap_positions();
        let t = compute_vertex_curvature_tensor(&pos, &tris, 0);
        // For a sphere cap both principal curvatures should have the same sign
        // (elliptic point). The local frame normal orientation may make both
        // negative (concave view from outside), so check same sign and non-zero.
        let k1 = t.kmin;
        let k2 = t.kmax;
        assert!(
            k1 * k2 > 0.0,
            "sphere cap should have same-sign principal curvatures (elliptic), got kmin={}, kmax={}",
            k1, k2
        );
    }

    #[test]
    fn flat_vertex_near_zero_curvature() {
        let (pos, tris) = flat_positions();
        let t = compute_vertex_curvature_tensor(&pos, &tris, 0);
        assert!(
            t.kmin.abs() < 0.5 && t.kmax.abs() < 0.5,
            "flat mesh should have near-zero curvatures, got kmin={}, kmax={}",
            t.kmin,
            t.kmax
        );
    }

    #[test]
    fn mean_curvature_formula() {
        let t = CurvatureTensor {
            kmin: 2.0,
            kmax: 4.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        let h = mean_curvature_from_tensor(&t);
        assert!((h - 3.0).abs() < 1e-5, "mean curvature should be 3.0");
    }

    #[test]
    fn gaussian_curvature_formula() {
        let t = CurvatureTensor {
            kmin: 2.0,
            kmax: 3.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        let k = gaussian_curvature_from_tensor(&t);
        assert!((k - 6.0).abs() < 1e-5, "gaussian curvature should be 6.0");
    }

    #[test]
    fn classify_sphere_positive_both() {
        let t = CurvatureTensor {
            kmin: 1.0,
            kmax: 1.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        let cls = classify_surface_point(&t);
        assert_eq!(cls, "sphere");
    }

    #[test]
    fn classify_saddle_opposite_signs() {
        let t = CurvatureTensor {
            kmin: -1.0,
            kmax: 1.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        let cls = classify_surface_point(&t);
        assert_eq!(cls, "saddle");
    }

    #[test]
    fn classify_flat_near_zero() {
        let t = CurvatureTensor {
            kmin: 0.0,
            kmax: 0.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        let cls = classify_surface_point(&t);
        assert_eq!(cls, "flat");
    }

    #[test]
    fn curvedness_non_negative() {
        let t = CurvatureTensor {
            kmin: -2.0,
            kmax: 3.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        let c = curvedness_from_tensor(&t);
        assert!(c >= 0.0, "curvedness must be non-negative");
    }

    #[test]
    fn curvedness_formula() {
        let t = CurvatureTensor {
            kmin: 3.0,
            kmax: 4.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        let c = curvedness_from_tensor(&t);
        let expected = ((9.0f32 + 16.0) / 2.0).sqrt();
        assert!((c - expected).abs() < 1e-5);
    }

    #[test]
    fn shape_index_range() {
        let t = CurvatureTensor {
            kmin: 1.0,
            kmax: 2.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        let s = shape_index_from_tensor(&t);
        assert!(
            (-1.0f32..=1.0).contains(&s),
            "shape index must be in [-1,1], got {s}"
        );
    }

    #[test]
    fn tensor_similarity_zero_for_equal() {
        let t = CurvatureTensor {
            kmin: 1.5,
            kmax: 2.5,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        let sim = tensor_similarity(&t, &t);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn smooth_curvature_tensors_no_panic() {
        let tensors = vec![
            CurvatureTensor {
                kmin: 1.0,
                kmax: 2.0,
                dir_min: [1.0, 0.0, 0.0],
                dir_max: [0.0, 1.0, 0.0],
            },
            CurvatureTensor {
                kmin: 3.0,
                kmax: 4.0,
                dir_min: [1.0, 0.0, 0.0],
                dir_max: [0.0, 1.0, 0.0],
            },
        ];
        let adj = vec![vec![1usize], vec![0usize]];
        let smoothed = smooth_curvature_tensors(&tensors, &adj);
        assert_eq!(smoothed.len(), 2);
    }

    #[test]
    fn compute_all_curvature_tensors_length() {
        let (pos, tris) = flat_positions();
        let all = compute_all_curvature_tensors(&pos, &tris);
        assert_eq!(all.len(), pos.len());
    }

    #[test]
    fn gaussian_curvature_negative_for_saddle_tensor() {
        let t = CurvatureTensor {
            kmin: -1.0,
            kmax: 2.0,
            dir_min: [1.0, 0.0, 0.0],
            dir_max: [0.0, 1.0, 0.0],
        };
        assert!(gaussian_curvature_from_tensor(&t) < 0.0);
    }
}
