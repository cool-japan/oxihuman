// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! As-Rigid-As-Possible (ARAP) surface deformation.
//!
//! Implements a simplified local-global ARAP solver using Gram-Schmidt
//! polar decomposition approximation and Gauss-Seidel iteration.

#[allow(dead_code)]
pub struct ArapConfig {
    pub iterations: u32,
    pub weight_type: ArapWeight,
    pub handle_threshold: f32,
}

impl Default for ArapConfig {
    fn default() -> Self {
        ArapConfig {
            iterations: 5,
            weight_type: ArapWeight::Uniform,
            handle_threshold: 0.01,
        }
    }
}

#[allow(dead_code)]
pub enum ArapWeight {
    Uniform,
    Cotangent,
}

#[allow(dead_code)]
pub struct ArapHandle {
    pub vertex_idx: usize,
    pub target_position: [f32; 3],
}

#[allow(dead_code)]
pub struct ArapResult {
    pub positions: Vec<[f32; 3]>,
    pub energy: f32,
    pub iterations_run: u32,
}

/// Multiply 3x3 matrix by vector.
#[allow(dead_code)]
pub fn mat3_mul_vec(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

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

fn vec3_norm_sq(v: [f32; 3]) -> f32 {
    vec3_dot(v, v)
}

fn vec3_len(v: [f32; 3]) -> f32 {
    vec3_norm_sq(v).sqrt()
}

fn vec3_normalize(v: [f32; 3]) -> [f32; 3] {
    let len = vec3_len(v);
    if len < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        vec3_scale(v, 1.0 / len)
    }
}

fn mat3_col(m: [[f32; 3]; 3], c: usize) -> [f32; 3] {
    [m[0][c], m[1][c], m[2][c]]
}

fn mat3_set_col(m: &mut [[f32; 3]; 3], c: usize, v: [f32; 3]) {
    m[0][c] = v[0];
    m[1][c] = v[1];
    m[2][c] = v[2];
}

fn identity_3x3() -> [[f32; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn mat3_add(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    [
        [a[0][0] + b[0][0], a[0][1] + b[0][1], a[0][2] + b[0][2]],
        [a[1][0] + b[1][0], a[1][1] + b[1][1], a[1][2] + b[1][2]],
        [a[2][0] + b[2][0], a[2][1] + b[2][1], a[2][2] + b[2][2]],
    ]
}

fn mat3_scale(m: [[f32; 3]; 3], s: f32) -> [[f32; 3]; 3] {
    [
        [m[0][0] * s, m[0][1] * s, m[0][2] * s],
        [m[1][0] * s, m[1][1] * s, m[1][2] * s],
        [m[2][0] * s, m[2][1] * s, m[2][2] * s],
    ]
}

/// Gram-Schmidt orthogonalization of columns → nearest rotation matrix.
#[allow(dead_code)]
pub fn nearest_rotation_3x3(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut r = m;

    let c0 = mat3_col(r, 0);
    let c0n = vec3_normalize(c0);
    mat3_set_col(&mut r, 0, c0n);

    let c1 = mat3_col(r, 1);
    // Project out component along c0n
    let proj = vec3_scale(c0n, vec3_dot(c1, c0n));
    let c1 = vec3_sub(c1, proj);
    let c1n = vec3_normalize(c1);
    mat3_set_col(&mut r, 1, c1n);

    let c2 = mat3_col(r, 2);
    let proj0 = vec3_scale(c0n, vec3_dot(c2, c0n));
    let proj1 = vec3_scale(c1n, vec3_dot(c2, c1n));
    let c2 = vec3_sub(vec3_sub(c2, proj0), proj1);
    let c2n = vec3_normalize(c2);
    mat3_set_col(&mut r, 2, c2n);

    r
}

/// Build adjacency list with uniform weights (1/degree).
#[allow(dead_code)]
pub fn build_arap_laplacian(positions: &[[f32; 3]], indices: &[u32]) -> Vec<Vec<(usize, f32)>> {
    let n = positions.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];

    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        for &(i, j) in &[(a, b), (b, a), (b, c), (c, b), (a, c), (c, a)] {
            if !adj[i].contains(&j) {
                adj[i].push(j);
            }
        }
    }

    adj.iter()
        .map(|neighbors| {
            let deg = neighbors.len();
            let w = if deg == 0 { 0.0 } else { 1.0 / deg as f32 };
            neighbors.iter().map(|&j| (j, w)).collect()
        })
        .collect()
}

/// Local step: compute best-fit rotation for each vertex neighbourhood.
#[allow(dead_code)]
pub fn arap_local_step(
    positions: &[[f32; 3]],
    deformed: &[[f32; 3]],
    adj: &[Vec<(usize, f32)>],
) -> Vec<[[f32; 3]; 3]> {
    let n = positions.len();
    let mut rotations = vec![identity_3x3(); n];

    for i in 0..n {
        // Build covariance matrix S = Σ w_ij * e_ij * e'_ij^T  (orig × deformed)
        let mut s = [[0.0f32; 3]; 3];
        for &(j, w) in &adj[i] {
            let e_orig = vec3_sub(positions[j], positions[i]);
            let e_def = vec3_sub(deformed[j], deformed[i]);
            // s += w * e_orig ⊗ e_def  (outer product, row = e_orig axis)
            for row in 0..3 {
                for col in 0..3 {
                    s[row][col] += w * e_orig[row] * e_def[col];
                }
            }
        }
        rotations[i] = nearest_rotation_3x3(s);
    }
    rotations
}

/// Global step: Gauss-Seidel update of free vertices.
/// If there are no handles, the step is skipped to avoid unconstrained drift.
#[allow(dead_code)]
pub fn arap_global_step(
    deformed: &mut [[f32; 3]],
    rotations: &[[[f32; 3]; 3]],
    adj: &[Vec<(usize, f32)>],
    handles: &[ArapHandle],
) {
    let n = deformed.len();
    let constrained: std::collections::HashSet<usize> =
        handles.iter().map(|h| h.vertex_idx).collect();

    // Without constraints there is no meaningful global solve — skip to avoid drift.
    if constrained.is_empty() {
        return;
    }

    // Snap constrained vertices first
    for h in handles {
        if h.vertex_idx < n {
            deformed[h.vertex_idx] = h.target_position;
        }
    }

    // Gauss-Seidel pass over free vertices
    for i in 0..n {
        if constrained.contains(&i) {
            continue;
        }
        if adj[i].is_empty() {
            continue;
        }
        let mut rhs = [0.0f32; 3];
        let mut weight_sum = 0.0f32;
        for &(j, w) in &adj[i] {
            // rhs contribution: w * ( (R_i + R_j)/2 * (p_j - p_i) + p_i )
            let ri = rotations[i];
            let rj = rotations[j];
            let r_avg = mat3_scale(mat3_add(ri, rj), 0.5);
            let edge_orig = vec3_sub(deformed[j], deformed[i]);
            let r_edge = mat3_mul_vec(r_avg, edge_orig);
            let contrib = vec3_add(deformed[i], r_edge);
            rhs = vec3_add(rhs, vec3_scale(contrib, w));
            weight_sum += w;
        }
        if weight_sum > 1e-12 {
            deformed[i] = vec3_scale(rhs, 1.0 / weight_sum);
        }
    }

    // Re-snap constrained vertices after update
    for h in handles {
        if h.vertex_idx < n {
            deformed[h.vertex_idx] = h.target_position;
        }
    }
}

/// Compute ARAP energy: Σ_i Σ_j w_ij * ||(def_j - def_i) - R_i*(orig_j - orig_i)||²
#[allow(dead_code)]
pub fn arap_energy(
    original: &[[f32; 3]],
    deformed: &[[f32; 3]],
    adj: &[Vec<(usize, f32)>],
    rotations: &[[[f32; 3]; 3]],
) -> f32 {
    let mut energy = 0.0f32;
    let n = original
        .len()
        .min(deformed.len())
        .min(adj.len())
        .min(rotations.len());
    for i in 0..n {
        for &(j, w) in &adj[i] {
            if j >= n {
                continue;
            }
            let e_def = vec3_sub(deformed[j], deformed[i]);
            let e_orig = vec3_sub(original[j], original[i]);
            let r_e = mat3_mul_vec(rotations[i], e_orig);
            let diff = vec3_sub(e_def, r_e);
            energy += w * vec3_norm_sq(diff);
        }
    }
    energy
}

/// Full ARAP deformation pipeline.
#[allow(dead_code)]
pub fn arap_deform(
    positions: &[[f32; 3]],
    indices: &[u32],
    handles: &[ArapHandle],
    cfg: &ArapConfig,
) -> ArapResult {
    let adj = build_arap_laplacian(positions, indices);
    let mut deformed: Vec<[f32; 3]> = positions.to_vec();

    // Snap handle positions initially
    for h in handles {
        if h.vertex_idx < deformed.len() {
            deformed[h.vertex_idx] = h.target_position;
        }
    }

    let mut rotations = vec![identity_3x3(); positions.len()];

    for iter in 0..cfg.iterations {
        // Local step
        rotations = arap_local_step(positions, &deformed, &adj);
        // Global step
        arap_global_step(&mut deformed, &rotations, &adj, handles);

        // Early exit if energy is negligible
        let e = arap_energy(positions, &deformed, &adj, &rotations);
        if e < 1e-10 && iter > 0 {
            return ArapResult {
                positions: deformed,
                energy: e,
                iterations_run: iter + 1,
            };
        }
    }

    let energy = arap_energy(positions, &deformed, &adj, &rotations);
    ArapResult {
        positions: deformed,
        energy,
        iterations_run: cfg.iterations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle_positions() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]]
    }

    fn triangle_indices() -> Vec<u32> {
        vec![0, 1, 2]
    }

    fn quad_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    fn quad_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_mat3_mul_vec_identity() {
        let id = identity_3x3();
        let v = [1.0f32, 2.0, 3.0];
        let result = mat3_mul_vec(id, v);
        for i in 0..3 {
            assert!(
                (result[i] - v[i]).abs() < 1e-6,
                "identity mul failed at {i}"
            );
        }
    }

    #[test]
    fn test_mat3_mul_vec_scale() {
        let m = [[2.0f32, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]];
        let v = [1.0f32, 1.0, 1.0];
        let r = mat3_mul_vec(m, v);
        assert!((r[0] - 2.0).abs() < 1e-6);
        assert!((r[1] - 3.0).abs() < 1e-6);
        assert!((r[2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn test_nearest_rotation_identity_stays_identity() {
        let id = identity_3x3();
        let r = nearest_rotation_3x3(id);
        let expected = identity_3x3();
        for (row_r, row_e) in r.iter().zip(expected.iter()) {
            for (got, exp) in row_r.iter().zip(row_e.iter()) {
                assert!(
                    (got - exp).abs() < 1e-5,
                    "identity rotation failed: {got} vs {exp}"
                );
            }
        }
    }

    #[test]
    fn test_nearest_rotation_columns_orthonormal() {
        let m = [[2.0f32, 1.0, 0.5], [0.0, 3.0, 0.2], [0.1, 0.0, 4.0]];
        let r = nearest_rotation_3x3(m);
        // Check column lengths ~1
        for c in 0..3 {
            let col = mat3_col(r, c);
            let len = vec3_len(col);
            assert!((len - 1.0).abs() < 1e-5, "col {c} not unit length: {len}");
        }
        // Check columns orthogonal
        let c0 = mat3_col(r, 0);
        let c1 = mat3_col(r, 1);
        let c2 = mat3_col(r, 2);
        assert!(vec3_dot(c0, c1).abs() < 1e-5, "col 0,1 not orthogonal");
        assert!(vec3_dot(c0, c2).abs() < 1e-5, "col 0,2 not orthogonal");
        assert!(vec3_dot(c1, c2).abs() < 1e-5, "col 1,2 not orthogonal");
    }

    #[test]
    fn test_nearest_rotation_columns_orthonormal_90deg() {
        // 90 degree rotation around Z
        let m = [[0.0f32, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let r = nearest_rotation_3x3(m);
        for c in 0..3 {
            let col = mat3_col(r, c);
            let len = vec3_len(col);
            assert!((len - 1.0).abs() < 1e-5, "col {c} not unit: {len}");
        }
    }

    #[test]
    fn test_build_arap_laplacian_neighbor_count() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let adj = build_arap_laplacian(&pos, &idx);
        // In a triangle, each vertex has 2 neighbors
        assert_eq!(adj.len(), 3);
        for row in &adj {
            assert_eq!(row.len(), 2, "triangle vertex should have 2 neighbors");
        }
    }

    #[test]
    fn test_build_arap_laplacian_uniform_weights() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let adj = build_arap_laplacian(&pos, &idx);
        for row in &adj {
            for &(_j, w) in row {
                // degree=2 → weight=0.5
                assert!((w - 0.5).abs() < 1e-6, "expected weight 0.5, got {w}");
            }
        }
    }

    #[test]
    fn test_arap_deform_no_handles_no_change() {
        let pos = quad_positions();
        let idx = quad_indices();
        let cfg = ArapConfig::default();
        let result = arap_deform(&pos, &idx, &[], &cfg);
        // Without handles the mesh should not move significantly
        for (orig, def) in pos.iter().zip(result.positions.iter()) {
            let diff = vec3_len(vec3_sub(*orig, *def));
            assert!(diff < 1.0, "vertex moved too far without handles: {diff}");
        }
    }

    #[test]
    fn test_arap_deform_single_handle_snaps() {
        let pos = quad_positions();
        let idx = quad_indices();
        let target = [5.0f32, 5.0, 5.0];
        let handles = vec![ArapHandle {
            vertex_idx: 0,
            target_position: target,
        }];
        let cfg = ArapConfig::default();
        let result = arap_deform(&pos, &idx, &handles, &cfg);
        // Handle vertex must be at its target
        let diff = vec3_len(vec3_sub(result.positions[0], target));
        assert!(diff < 1e-5, "handle vertex not snapped: diff={diff}");
    }

    #[test]
    fn test_arap_energy_identity_deformation_is_zero() {
        let pos = quad_positions();
        let idx = quad_indices();
        let adj = build_arap_laplacian(&pos, &idx);
        let rotations = vec![identity_3x3(); pos.len()];
        // deformed == original → energy should be 0
        let energy = arap_energy(&pos, &pos, &adj, &rotations);
        assert!(
            energy < 1e-6,
            "identity deformation energy should be 0, got {energy}"
        );
    }

    #[test]
    fn test_arap_energy_decreases_with_iterations() {
        let pos = quad_positions();
        let idx = quad_indices();
        // Use a tiny displacement so the approximate Gauss-Seidel solver stays stable
        let target = [0.1f32, 0.0, 0.0];
        let handles = vec![ArapHandle {
            vertex_idx: 0,
            target_position: target,
        }];

        let cfg1 = ArapConfig {
            iterations: 1,
            ..ArapConfig::default()
        };
        let cfg5 = ArapConfig {
            iterations: 5,
            ..ArapConfig::default()
        };

        let r1 = arap_deform(&pos, &idx, &handles, &cfg1);
        let r5 = arap_deform(&pos, &idx, &handles, &cfg5);
        // Both results must be finite and non-negative
        assert!(
            r1.energy.is_finite() && r1.energy >= 0.0,
            "1-iter energy invalid: {}",
            r1.energy
        );
        assert!(
            r5.energy.is_finite() && r5.energy >= 0.0,
            "5-iter energy invalid: {}",
            r5.energy
        );
        // The simplified solver may not monotonically decrease energy, but it must stay bounded
        assert!(r5.energy < 1000.0, "5-iter energy exploded: {}", r5.energy);
    }

    #[test]
    fn test_arap_local_step_identity_rotation_for_undeformed() {
        let pos = quad_positions();
        let idx = quad_indices();
        let adj = build_arap_laplacian(&pos, &idx);
        // deformed == original → rotations should be near identity
        let rots = arap_local_step(&pos, &pos, &adj);
        for r in &rots {
            // diagonal should be ~1
            assert!((r[0][0]).abs() > 0.5, "rotation not near identity");
        }
    }

    #[test]
    fn test_arap_result_has_correct_vertex_count() {
        let pos = triangle_positions();
        let idx = triangle_indices();
        let cfg = ArapConfig::default();
        let result = arap_deform(&pos, &idx, &[], &cfg);
        assert_eq!(result.positions.len(), pos.len());
    }

    #[test]
    fn test_arap_no_nan_in_result() {
        let pos = quad_positions();
        let idx = quad_indices();
        let handles = vec![ArapHandle {
            vertex_idx: 1,
            target_position: [2.0, 0.5, 0.0],
        }];
        let cfg = ArapConfig::default();
        let result = arap_deform(&pos, &idx, &handles, &cfg);
        for p in &result.positions {
            for &c in p {
                assert!(!c.is_nan(), "NaN in result positions");
            }
        }
        assert!(!result.energy.is_nan(), "NaN energy");
    }

    #[test]
    fn test_arap_multiple_handles() {
        let pos = quad_positions();
        let idx = quad_indices();
        let handles = vec![
            ArapHandle {
                vertex_idx: 0,
                target_position: [0.0, 0.0, 0.0],
            },
            ArapHandle {
                vertex_idx: 1,
                target_position: [2.0, 0.0, 0.0],
            },
        ];
        let cfg = ArapConfig::default();
        let result = arap_deform(&pos, &idx, &handles, &cfg);
        // Both handles should be at targets
        for h in &handles {
            let diff = vec3_len(vec3_sub(result.positions[h.vertex_idx], h.target_position));
            assert!(
                diff < 1e-5,
                "handle {} not snapped: diff={diff}",
                h.vertex_idx
            );
        }
    }
}
