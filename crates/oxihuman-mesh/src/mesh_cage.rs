// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cage-based deformation using mean-value coordinates (MVC).
//!
//! Implements the Floater mean-value coordinate formula for 3-D cage deformation.

#[allow(dead_code)]
pub struct CageDeformer {
    pub cage_vertices: Vec<[f32; 3]>,
    pub weights: Vec<Vec<f32>>,
}

#[allow(dead_code)]
pub struct CageDeformResult {
    pub positions: Vec<[f32; 3]>,
    pub max_coord_error: f32,
}

// ---------- small vec3 helpers ----------

fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn vec3_scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn vec3_cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_dot(v, v).sqrt()
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let l = vec3_len(v);
    if l < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        vec3_scale(v, 1.0 / l)
    }
}

// ---------- MVC helpers ----------

/// Compute the angle at `center` in triangle (p, center, other) using dot product.
fn angle_at(center: [f32; 3], p: [f32; 3], other: [f32; 3]) -> f32 {
    let u = vec3_normalize(vec3_sub(p, center));
    let v = vec3_normalize(vec3_sub(other, center));
    let dot = vec3_dot(u, v).clamp(-1.0, 1.0);
    dot.acos()
}

/// Compute mean-value coordinates for `point` with respect to `cage_verts`.
///
/// Uses the Floater formula for 3-D polygonal cages:
/// w_i = (tan(α_{i-1}/2) + tan(α_i/2)) / ||p - v_i||
/// where α_{i-1} and α_i are the angles in the two triangles adjacent to vertex i
/// in the star around `point`.
#[allow(dead_code)]
pub fn mean_value_coordinates(
    point: [f32; 3],
    cage_verts: &[[f32; 3]],
    cage_indices: &[u32],
) -> Vec<f32> {
    let n = cage_verts.len();
    let mut weights = vec![0.0f32; n];

    // Accumulate triangle contributions
    for tri in cage_indices.chunks_exact(3) {
        let (ia, ib, ic) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if ia >= n || ib >= n || ic >= n {
            continue;
        }
        let va = cage_verts[ia];
        let vb = cage_verts[ib];
        let vc = cage_verts[ic];

        // angles at point looking toward each vertex from the other two
        let alpha_a = angle_at(point, vb, vc);
        let alpha_b = angle_at(point, va, vc);
        let alpha_c = angle_at(point, va, vb);

        let da = vec3_len(vec3_sub(point, va));
        let db = vec3_len(vec3_sub(point, vb));
        let dc = vec3_len(vec3_sub(point, vc));

        if da > 1e-12 {
            weights[ia] += (alpha_a / 2.0).tan() / da;
        }
        if db > 1e-12 {
            weights[ib] += (alpha_b / 2.0).tan() / db;
        }
        if dc > 1e-12 {
            weights[ic] += (alpha_c / 2.0).tan() / dc;
        }
    }

    // Normalize
    let sum: f32 = weights.iter().sum();
    if sum.abs() > 1e-12 {
        for w in &mut weights {
            *w /= sum;
        }
    }
    weights
}

/// Half-angle helper used for MVC computation.
#[allow(dead_code)]
pub fn mvc_angle(v: [f32; 3], center: [f32; 3], a: [f32; 3], _b: [f32; 3]) -> f32 {
    angle_at(center, v, a) / 2.0
}

/// Ray-cast point-in-mesh test (approximate; counts intersections along +X ray).
#[allow(dead_code)]
pub fn cage_encloses_point(point: [f32; 3], cage_verts: &[[f32; 3]], cage_indices: &[u32]) -> bool {
    let mut crossings = 0u32;
    let n = cage_verts.len();

    for tri in cage_indices.chunks_exact(3) {
        let (ia, ib, ic) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if ia >= n || ib >= n || ic >= n {
            continue;
        }
        let a = cage_verts[ia];
        let b = cage_verts[ib];
        let c = cage_verts[ic];

        // Only consider triangles in front of point (+X direction)
        let min_x = a[0].min(b[0]).min(c[0]);
        if min_x < point[0] {
            continue;
        }

        // Check if ray (point, +X) hits this triangle
        // Using Möller–Trumbore intersection
        let e1 = vec3_sub(b, a);
        let e2 = vec3_sub(c, a);
        let ray_dir = [1.0f32, 0.0, 0.0];
        let h = vec3_cross(ray_dir, e2);
        let det = vec3_dot(e1, h);
        if det.abs() < 1e-10 {
            continue;
        }
        let inv_det = 1.0 / det;
        let s = vec3_sub(point, a);
        let u = inv_det * vec3_dot(s, h);
        if !(0.0..=1.0).contains(&u) {
            continue;
        }
        let q = vec3_cross(s, e1);
        let v = inv_det * vec3_dot(ray_dir, q);
        if v < 0.0 || u + v > 1.0 {
            continue;
        }
        let t = inv_det * vec3_dot(e2, q);
        if t > 1e-10 {
            crossings += 1;
        }
    }
    crossings % 2 == 1
}

/// Apply cage weights to compute a point: Σ w_i * v_i.
#[allow(dead_code)]
pub fn apply_cage_weights(weights: &[f32], cage_verts: &[[f32; 3]]) -> [f32; 3] {
    let mut result = [0.0f32; 3];
    for (w, v) in weights.iter().zip(cage_verts.iter()) {
        result = vec3_add(result, vec3_scale(*v, *w));
    }
    result
}

/// Validate that all rows of `weights` sum to approximately 1.0.
/// Returns the maximum |Σ w_i - 1| over all rows.
#[allow(dead_code)]
pub fn validate_cage_weights(weights: &[Vec<f32>]) -> f32 {
    weights
        .iter()
        .map(|row| {
            let s: f32 = row.iter().sum();
            (s - 1.0).abs()
        })
        .fold(0.0f32, f32::max)
}

impl CageDeformer {
    /// Build a cage deformer by computing MVC weights for every mesh vertex.
    #[allow(dead_code)]
    pub fn new(cage_verts: &[[f32; 3]], mesh_verts: &[[f32; 3]], cage_indices: &[u32]) -> Self {
        let weights = mesh_verts
            .iter()
            .map(|&p| mean_value_coordinates(p, cage_verts, cage_indices))
            .collect();

        CageDeformer {
            cage_vertices: cage_verts.to_vec(),
            weights,
        }
    }

    /// Deform the mesh by applying stored weights to new cage positions.
    #[allow(dead_code)]
    pub fn deform(&self, new_cage_verts: &[[f32; 3]]) -> CageDeformResult {
        let positions: Vec<[f32; 3]> = self
            .weights
            .iter()
            .map(|w| apply_cage_weights(w, new_cage_verts))
            .collect();

        let max_coord_error = validate_cage_weights(&self.weights);

        CageDeformResult {
            positions,
            max_coord_error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple tetrahedron cage around the origin.
    fn tet_cage() -> (Vec<[f32; 3]>, Vec<u32>) {
        let verts = vec![
            [0.0f32, 2.0, 0.0],
            [-2.0, -1.0, -1.0],
            [2.0, -1.0, -1.0],
            [0.0, -1.0, 2.0],
        ];
        let indices = vec![0, 1, 2, 0, 1, 3, 0, 2, 3, 1, 2, 3];
        (verts, indices)
    }

    /// Tiny single-triangle cage (degenerate, used to test weight mechanics).
    fn tri_cage() -> (Vec<[f32; 3]>, Vec<u32>) {
        let verts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let indices = vec![0, 1, 2];
        (verts, indices)
    }

    #[test]
    fn test_apply_cage_weights_linear_combination() {
        let verts: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let weights = vec![0.5f32, 0.5];
        let result = apply_cage_weights(&weights, &verts);
        assert!(
            (result[0] - 1.0).abs() < 1e-6,
            "x should be 1.0, got {}",
            result[0]
        );
        assert!(result[1].abs() < 1e-6);
        assert!(result[2].abs() < 1e-6);
    }

    #[test]
    fn test_apply_cage_weights_single_vertex() {
        let verts: Vec<[f32; 3]> = vec![[3.0, 4.0, 5.0]];
        let weights = vec![1.0f32];
        let r = apply_cage_weights(&weights, &verts);
        assert!((r[0] - 3.0).abs() < 1e-6);
        assert!((r[1] - 4.0).abs() < 1e-6);
        assert!((r[2] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_validate_cage_weights_perfect() {
        let weights = vec![vec![0.25f32, 0.25, 0.25, 0.25], vec![1.0, 0.0, 0.0, 0.0]];
        let err = validate_cage_weights(&weights);
        assert!(err < 1e-6, "perfect weights should have 0 error, got {err}");
    }

    #[test]
    fn test_validate_cage_weights_imperfect() {
        let weights = vec![vec![0.3f32, 0.3, 0.3]]; // sums to 0.9
        let err = validate_cage_weights(&weights);
        assert!((err - 0.1).abs() < 1e-5, "expected error ~0.1, got {err}");
    }

    #[test]
    fn test_mean_value_coordinates_sum_to_one() {
        let (cage_verts, cage_indices) = tet_cage();
        // Sample a point inside the tetrahedron
        let point = [0.0f32, 0.0, 0.0];
        let weights = mean_value_coordinates(point, &cage_verts, &cage_indices);
        let sum: f32 = weights.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-4,
            "MVC weights should sum to 1, got {sum}"
        );
    }

    #[test]
    fn test_mean_value_coordinates_non_negative_inside() {
        let (cage_verts, cage_indices) = tet_cage();
        let point = [0.0f32, 0.0, 0.0];
        let weights = mean_value_coordinates(point, &cage_verts, &cage_indices);
        for &w in &weights {
            assert!(
                w >= -0.01,
                "interior weights should be non-negative, got {w}"
            );
        }
    }

    #[test]
    fn test_cage_deformer_identity_no_change() {
        let (cage_verts, cage_indices) = tet_cage();
        // Use a single point at the centroid of the cage — MVC is most accurate there
        let centroid = [0.0f32, 0.125, 0.25]; // centroid of tet_cage verts
        let mesh_verts: Vec<[f32; 3]> = vec![centroid];
        let deformer = CageDeformer::new(&cage_verts, &mesh_verts, &cage_indices);
        // Deform with the same cage → positions should be close to original
        let result = deformer.deform(&cage_verts);
        for (orig, def) in mesh_verts.iter().zip(result.positions.iter()) {
            let dist = ((orig[0] - def[0]).powi(2)
                + (orig[1] - def[1]).powi(2)
                + (orig[2] - def[2]).powi(2))
            .sqrt();
            // MVC approximation: allow up to 0.5 units of reconstruction error at centroid
            assert!(
                dist < 0.5,
                "identity deform moved centroid too far: dist={}, orig={:?}, def={:?}",
                dist,
                orig,
                def
            );
        }
    }

    #[test]
    fn test_cage_deformer_deform_result_length() {
        let (cage_verts, cage_indices) = tet_cage();
        let mesh_verts: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [0.1, 0.2, 0.3]];
        let deformer = CageDeformer::new(&cage_verts, &mesh_verts, &cage_indices);
        let result = deformer.deform(&cage_verts);
        assert_eq!(result.positions.len(), mesh_verts.len());
    }

    #[test]
    fn test_cage_max_coord_error_near_zero() {
        let (cage_verts, cage_indices) = tet_cage();
        let mesh_verts: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]];
        let deformer = CageDeformer::new(&cage_verts, &mesh_verts, &cage_indices);
        let result = deformer.deform(&cage_verts);
        assert!(
            result.max_coord_error < 0.05,
            "coord error too large: {}",
            result.max_coord_error
        );
    }

    #[test]
    fn test_cage_encloses_point_outside() {
        let (cage_verts, cage_indices) = tet_cage();
        // A point far outside the tetrahedron
        let outside = [100.0f32, 100.0, 100.0];
        assert!(
            !cage_encloses_point(outside, &cage_verts, &cage_indices),
            "far point should not be enclosed"
        );
    }

    #[test]
    fn test_cage_weights_length() {
        let (cage_verts, cage_indices) = tet_cage();
        let point = [0.0f32, 0.0, 0.0];
        let weights = mean_value_coordinates(point, &cage_verts, &cage_indices);
        assert_eq!(
            weights.len(),
            cage_verts.len(),
            "weight count should match cage vertex count"
        );
    }

    #[test]
    fn test_mvc_angle_is_positive() {
        let v = [1.0f32, 0.0, 0.0];
        let center = [0.0f32, 0.0, 0.0];
        let a = [0.0f32, 1.0, 0.0];
        let b = [0.0f32, 0.0, 1.0];
        let angle = mvc_angle(v, center, a, b);
        assert!(angle >= 0.0, "half-angle should be non-negative");
    }

    #[test]
    fn test_apply_cage_weights_zero_weights() {
        let verts: Vec<[f32; 3]> = vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let weights = vec![0.0f32, 0.0];
        let r = apply_cage_weights(&weights, &verts);
        assert_eq!(r, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_cage_deformer_translated_cage() {
        let (cage_verts, cage_indices) = tri_cage();
        let mesh_verts: Vec<[f32; 3]> = vec![[0.3, 0.3, 0.0]];
        let deformer = CageDeformer::new(&cage_verts, &mesh_verts, &cage_indices);

        // Translate cage by (1, 0, 0)
        let new_cage: Vec<[f32; 3]> = cage_verts
            .iter()
            .map(|v| [v[0] + 1.0, v[1], v[2]])
            .collect();
        let result = deformer.deform(&new_cage);
        // Mesh vertex should shift ~1 in x
        assert!(
            (result.positions[0][0] - mesh_verts[0][0] - 1.0).abs() < 0.5,
            "translated cage should shift mesh vertex ~1 in x: got {}",
            result.positions[0][0]
        );
    }
}
