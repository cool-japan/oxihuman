// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Per-vertex curvature tensor computation using shape operator.

// ── Types ─────────────────────────────────────────────────────────────────────

/// Configuration for curvature tensor computation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvatureTensorConfig {
    /// Number of Laplacian smoothing passes applied to curvature values.
    pub smooth_iterations: u32,
    /// Weight neighbor contributions by incident angles when true.
    pub use_angle_weights: bool,
}

/// Per-vertex curvature tensor storing principal curvatures and directions.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct CurvatureTensorShape {
    /// First (larger-magnitude) principal curvature κ₁.
    pub principal_k1: f32,
    /// Second (smaller-magnitude) principal curvature κ₂.
    pub principal_k2: f32,
    /// Direction of κ₁ (unit vector in tangent plane).
    pub direction_k1: [f32; 3],
    /// Direction of κ₂ (unit vector in tangent plane).
    pub direction_k2: [f32; 3],
}

/// Result holding per-vertex curvature tensors plus derived scalar fields.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurvatureTensorResult {
    /// One tensor per input vertex.
    pub tensors: Vec<CurvatureTensorShape>,
    /// Mean curvature H = (κ₁ + κ₂) / 2 per vertex.
    pub mean_curvature: Vec<f32>,
    /// Gaussian curvature K = κ₁ · κ₂ per vertex.
    pub gaussian_curvature: Vec<f32>,
}

// ── Internal helpers ──────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [1.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Build a simple vertex → neighbor list from triangles.
fn build_neighbors(n_verts: usize, triangles: &[[u32; 3]]) -> Vec<Vec<usize>> {
    let mut nbrs: Vec<Vec<usize>> = vec![Vec::new(); n_verts];
    for tri in triangles {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        for &(u, v) in &[(a, b), (a, c), (b, a), (b, c), (c, a), (c, b)] {
            if !nbrs[u].contains(&v) {
                nbrs[u].push(v);
            }
        }
    }
    nbrs
}

/// Angle at vertex `vi` in the triangle formed by `vi`, `vj`, `vk`.
fn corner_angle(p_i: [f32; 3], p_j: [f32; 3], p_k: [f32; 3]) -> f32 {
    let ij = normalize3(sub3(p_j, p_i));
    let ik = normalize3(sub3(p_k, p_i));
    dot3(ij, ik).clamp(-1.0, 1.0).acos()
}

/// Estimate principal curvatures from the normal variation within the 1-ring.
/// This is a simplified shape-operator approach using finite differences.
fn estimate_shape_tensor(
    vi: usize,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    nbrs: &[usize],
    use_angle_weights: bool,
    triangles: &[[u32; 3]],
) -> CurvatureTensorShape {
    let pi = positions[vi];
    let ni = normals[vi];

    // Build a tangent frame from the normal.
    let t1 = {
        let arb = if ni[0].abs() < 0.9 {
            [1.0_f32, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        normalize3(cross3(ni, arb))
    };
    let t2 = normalize3(cross3(ni, t1));

    // Accumulate shape-operator entries A·x = b in 2×2 tangent space.
    // We use the Weingarten map: for each neighbor j, project edge and
    // normal difference into tangent plane.
    let mut sum_w = 0.0_f32;
    let mut a11 = 0.0_f32; // Σ w · eu² · κn
    let mut a12 = 0.0_f32; // Σ w · eu·ev · κn
    let mut a22 = 0.0_f32; // Σ w · ev² · κn

    for &vj in nbrs {
        let pj = positions[vj];
        let edge = sub3(pj, pi);
        let edge_len = (dot3(edge, edge)).sqrt();
        if edge_len < 1e-10 {
            continue;
        }

        // Weight: inverse edge length or angle-based
        let w = if use_angle_weights {
            // Use angle at vi in any triangle containing the edge vi–vj
            let mut angle_sum = 0.0_f32;
            let mut angle_count = 0;
            for tri in triangles {
                let tidx = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
                if tidx.contains(&vi) && tidx.contains(&vj) {
                    let vk = tidx.iter().copied().find(|&x| x != vi && x != vj);
                    if let Some(vk) = vk {
                        angle_sum += corner_angle(pi, pj, positions[vk]);
                        angle_count += 1;
                    }
                }
            }
            if angle_count > 0 {
                angle_sum / angle_count as f32
            } else {
                1.0 / edge_len
            }
        } else {
            1.0 / edge_len
        };

        // Normal curvature along the edge direction
        let nj = normals[vj];
        let dn = sub3(nj, ni);
        let kn = dot3(dn, edge) / (edge_len * edge_len);

        // Project edge into tangent plane
        let eu = dot3(edge, t1) / edge_len;
        let ev = dot3(edge, t2) / edge_len;

        a11 += w * eu * eu * kn;
        a12 += w * eu * ev * kn;
        a22 += w * ev * ev * kn;
        sum_w += w;
    }

    if sum_w < 1e-10 {
        return CurvatureTensorShape {
            principal_k1: 0.0,
            principal_k2: 0.0,
            direction_k1: t1,
            direction_k2: t2,
        };
    }

    a11 /= sum_w;
    a12 /= sum_w;
    a22 /= sum_w;

    // Eigenvalues of 2×2 symmetric matrix [[a11, a12],[a12, a22]]
    let tr = a11 + a22;
    let det = a11 * a22 - a12 * a12;
    let disc = ((tr * tr * 0.25 - det).max(0.0)).sqrt();
    let k1 = tr * 0.5 + disc;
    let k2 = tr * 0.5 - disc;

    // Eigenvector for k1 in tangent space, then lift to 3D
    let dir_k1_3d = if a12.abs() > 1e-10 {
        let ex = k1 - a22;
        let ey = a12;
        let len = (ex * ex + ey * ey).sqrt().max(1e-12);
        let (ex, ey) = (ex / len, ey / len);
        normalize3([
            ex * t1[0] + ey * t2[0],
            ex * t1[1] + ey * t2[1],
            ex * t1[2] + ey * t2[2],
        ])
    } else {
        t1
    };
    let dir_k2_3d = normalize3(cross3(ni, dir_k1_3d));

    CurvatureTensorShape {
        principal_k1: k1,
        principal_k2: k2,
        direction_k1: dir_k1_3d,
        direction_k2: dir_k2_3d,
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return a default [`CurvatureTensorConfig`].
#[allow(dead_code)]
pub fn default_curvature_tensor_config() -> CurvatureTensorConfig {
    CurvatureTensorConfig {
        smooth_iterations: 1,
        use_angle_weights: true,
    }
}

/// Compute per-vertex curvature tensors for the mesh.
///
/// * `positions`  – vertex positions (one per vertex)
/// * `normals`    – vertex normals   (one per vertex, pre-normalised)
/// * `triangles`  – index triples
/// * `cfg`        – configuration
#[allow(dead_code)]
pub fn compute_curvature_tensors(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    triangles: &[[u32; 3]],
    cfg: &CurvatureTensorConfig,
) -> CurvatureTensorResult {
    let n = positions.len().min(normals.len());
    if n == 0 {
        return CurvatureTensorResult {
            tensors: Vec::new(),
            mean_curvature: Vec::new(),
            gaussian_curvature: Vec::new(),
        };
    }

    let nbrs = build_neighbors(n, triangles);

    let mut tensors: Vec<CurvatureTensorShape> = (0..n)
        .map(|vi| {
            estimate_shape_tensor(
                vi,
                &positions[..n],
                &normals[..n],
                &nbrs[vi],
                cfg.use_angle_weights,
                triangles,
            )
        })
        .collect();

    // Laplacian smoothing of principal curvature values.
    for _ in 0..cfg.smooth_iterations {
        let prev = tensors.clone();
        for vi in 0..n {
            let neighbors = &nbrs[vi];
            if neighbors.is_empty() {
                continue;
            }
            let k1_avg: f32 =
                neighbors.iter().map(|&j| prev[j].principal_k1).sum::<f32>() / neighbors.len() as f32;
            let k2_avg: f32 =
                neighbors.iter().map(|&j| prev[j].principal_k2).sum::<f32>() / neighbors.len() as f32;
            tensors[vi].principal_k1 = (prev[vi].principal_k1 + k1_avg) * 0.5;
            tensors[vi].principal_k2 = (prev[vi].principal_k2 + k2_avg) * 0.5;
        }
    }

    let mean_curvature: Vec<f32> = tensors.iter().map(mean_curvature_at).collect();
    let gaussian_curvature: Vec<f32> = tensors.iter().map(gaussian_curvature_at).collect();

    CurvatureTensorResult {
        tensors,
        mean_curvature,
        gaussian_curvature,
    }
}

/// Mean curvature H = (κ₁ + κ₂) / 2.
#[allow(dead_code)]
#[inline]
pub fn mean_curvature_at(t: &CurvatureTensorShape) -> f32 {
    (t.principal_k1 + t.principal_k2) * 0.5
}

/// Gaussian curvature K = κ₁ · κ₂.
#[allow(dead_code)]
#[inline]
pub fn gaussian_curvature_at(t: &CurvatureTensorShape) -> f32 {
    t.principal_k1 * t.principal_k2
}

/// Returns `true` when the surface is elliptic (K > 0).
#[allow(dead_code)]
#[inline]
pub fn is_elliptic(t: &CurvatureTensorShape) -> bool {
    gaussian_curvature_at(t) > 0.0
}

/// Returns `true` when the surface is hyperbolic (K < 0).
#[allow(dead_code)]
#[inline]
pub fn is_hyperbolic(t: &CurvatureTensorShape) -> bool {
    gaussian_curvature_at(t) < 0.0
}

/// Serialize a single tensor to a compact JSON string.
#[allow(dead_code)]
pub fn curvature_tensor_to_json(t: &CurvatureTensorShape) -> String {
    format!(
        "{{\"k1\":{},\"k2\":{},\"dir_k1\":[{},{},{}],\"dir_k2\":[{},{},{}]}}",
        t.principal_k1,
        t.principal_k2,
        t.direction_k1[0],
        t.direction_k1[1],
        t.direction_k1[2],
        t.direction_k2[0],
        t.direction_k2[1],
        t.direction_k2[2],
    )
}

/// Average mean curvature over all vertices in the result.
#[allow(dead_code)]
pub fn result_avg_mean_curvature(r: &CurvatureTensorResult) -> f32 {
    if r.mean_curvature.is_empty() {
        return 0.0;
    }
    r.mean_curvature.iter().sum::<f32>() / r.mean_curvature.len() as f32
}

/// Maximum absolute principal curvature over all vertices.
#[allow(dead_code)]
pub fn result_max_curvature(r: &CurvatureTensorResult) -> f32 {
    r.tensors
        .iter()
        .map(|t| t.principal_k1.abs().max(t.principal_k2.abs()))
        .fold(0.0_f32, f32::max)
}

/// Number of tensors (== number of vertices).
#[allow(dead_code)]
pub fn tensor_count(r: &CurvatureTensorResult) -> usize {
    r.tensors.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::type_complexity)]
    fn flat_quad() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[u32; 3]>) {
        let positions = vec![
            [0.0_f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normals = vec![[0.0_f32, 0.0, 1.0]; 4];
        let triangles = vec![[0u32, 1, 2], [0, 2, 3]];
        (positions, normals, triangles)
    }

    #[test]
    fn default_config_smoke() {
        let cfg = default_curvature_tensor_config();
        assert_eq!(cfg.smooth_iterations, 1);
        assert!(cfg.use_angle_weights);
    }

    #[test]
    fn flat_mesh_gaussian_near_zero() {
        let (pos, nrm, tris) = flat_quad();
        let cfg = CurvatureTensorConfig {
            smooth_iterations: 0,
            use_angle_weights: false,
        };
        let result = compute_curvature_tensors(&pos, &nrm, &tris, &cfg);
        assert_eq!(tensor_count(&result), 4);
        for k in &result.gaussian_curvature {
            assert!(k.abs() < 0.2, "gaussian should be near zero on flat mesh");
        }
    }

    #[test]
    fn tensor_count_matches_vertex_count() {
        let (pos, nrm, tris) = flat_quad();
        let cfg = default_curvature_tensor_config();
        let result = compute_curvature_tensors(&pos, &nrm, &tris, &cfg);
        assert_eq!(result.tensors.len(), pos.len());
        assert_eq!(result.mean_curvature.len(), pos.len());
        assert_eq!(result.gaussian_curvature.len(), pos.len());
    }

    #[test]
    fn mean_and_gaussian_helpers() {
        let t = CurvatureTensorShape {
            principal_k1: 3.0,
            principal_k2: 1.0,
            direction_k1: [1.0, 0.0, 0.0],
            direction_k2: [0.0, 1.0, 0.0],
        };
        assert!((mean_curvature_at(&t) - 2.0).abs() < 1e-6);
        assert!((gaussian_curvature_at(&t) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn elliptic_hyperbolic_classification() {
        let elliptic = CurvatureTensorShape {
            principal_k1: 2.0,
            principal_k2: 1.0,
            direction_k1: [1.0, 0.0, 0.0],
            direction_k2: [0.0, 1.0, 0.0],
        };
        let hyperbolic = CurvatureTensorShape {
            principal_k1: 2.0,
            principal_k2: -1.0,
            direction_k1: [1.0, 0.0, 0.0],
            direction_k2: [0.0, 1.0, 0.0],
        };
        assert!(is_elliptic(&elliptic));
        assert!(!is_hyperbolic(&elliptic));
        assert!(is_hyperbolic(&hyperbolic));
        assert!(!is_elliptic(&hyperbolic));
    }

    #[test]
    fn json_output_contains_keys() {
        let t = CurvatureTensorShape {
            principal_k1: 1.5,
            principal_k2: -0.5,
            direction_k1: [1.0, 0.0, 0.0],
            direction_k2: [0.0, 1.0, 0.0],
        };
        let json = curvature_tensor_to_json(&t);
        assert!(json.contains("\"k1\""));
        assert!(json.contains("\"k2\""));
        assert!(json.contains("\"dir_k1\""));
        assert!(json.contains("\"dir_k2\""));
    }

    #[test]
    fn result_avg_and_max() {
        let (pos, nrm, tris) = flat_quad();
        let cfg = CurvatureTensorConfig {
            smooth_iterations: 0,
            use_angle_weights: false,
        };
        let result = compute_curvature_tensors(&pos, &nrm, &tris, &cfg);
        let _avg = result_avg_mean_curvature(&result);
        let max_c = result_max_curvature(&result);
        assert!(max_c >= 0.0);
    }

    #[test]
    fn empty_mesh_returns_empty_result() {
        let cfg = default_curvature_tensor_config();
        let result = compute_curvature_tensors(&[], &[], &[], &cfg);
        assert_eq!(result.tensors.len(), 0);
        assert_eq!(result.mean_curvature.len(), 0);
        assert_eq!(result.gaussian_curvature.len(), 0);
    }
}
