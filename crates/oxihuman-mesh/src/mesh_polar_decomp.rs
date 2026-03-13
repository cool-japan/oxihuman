// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Polar decomposition for mesh deformation analysis (R, S = rotation, scale).

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Structures
// ---------------------------------------------------------------------------

/// Result of a polar decomposition M = R * S.
#[allow(dead_code)]
pub struct PolarDecomp {
    /// 3×3 rotation matrix
    pub rotation: [[f32; 3]; 3],
    /// 3×3 symmetric stretch matrix
    pub symmetric: [[f32; 3]; 3],
}

/// Deformation gradient for a triangle.
#[allow(dead_code)]
pub struct DeformationGradient {
    /// F = dx/dX
    pub f: [[f32; 3]; 3],
    /// det(F)
    pub det_f: f32,
    /// volume ratio (= det_f for 3-D)
    pub volume_ratio: f32,
}

// ---------------------------------------------------------------------------
// Mat3 helpers
// ---------------------------------------------------------------------------

/// Multiply two 3×3 matrices.
#[allow(dead_code)]
pub fn mat3_mul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut out = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

/// Transpose a 3×3 matrix.
#[allow(dead_code)]
pub fn mat3_transpose(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Determinant of a 3×3 matrix.
#[allow(dead_code)]
pub fn mat3_det(m: [[f32; 3]; 3]) -> f32 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// Identity matrix.
#[allow(dead_code)]
pub fn mat3_identity() -> [[f32; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

/// Scale all elements of a 3×3 matrix by scalar s.
#[allow(dead_code)]
pub fn mat3_scale(m: [[f32; 3]; 3], s: f32) -> [[f32; 3]; 3] {
    let mut out = m;
    for row in out.iter_mut() {
        for v in row.iter_mut() {
            *v *= s;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Polar decomposition
// ---------------------------------------------------------------------------

/// Polar decompose M = R * S using iterative QR-like method (Higham's iteration).
/// R is the rotation part, S = R^T * M is the symmetric stretch.
#[allow(dead_code)]
pub fn polar_decompose(m: [[f32; 3]; 3]) -> PolarDecomp {
    // Higham's iteration: R_{k+1} = 0.5*(R_k + R_k^{-T})
    let mut r = m;
    for _ in 0..32 {
        let rt = mat3_transpose(r);
        let det = mat3_det(r);
        if det.abs() < 1e-12 {
            break;
        }
        let inv_rt = mat3_inv(rt);
        // r_next = 0.5 * (r + inv_rt)
        let mut r_next = [[0.0f32; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                r_next[i][j] = 0.5 * (r[i][j] + inv_rt[i][j]);
            }
        }
        // Check convergence
        let mut diff = 0.0f32;
        for i in 0..3 {
            for j in 0..3 {
                diff += (r_next[i][j] - r[i][j]).abs();
            }
        }
        r = r_next;
        if diff < 1e-7 {
            break;
        }
    }
    // S = R^T * M
    let rt = mat3_transpose(r);
    let symmetric = mat3_mul(rt, m);
    PolarDecomp {
        rotation: r,
        symmetric,
    }
}

/// Invert a 3×3 matrix (used internally for polar decomposition).
fn mat3_inv(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let det = mat3_det(m);
    if det.abs() < 1e-12 {
        return mat3_identity();
    }
    let inv_det = 1.0 / det;
    [
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det,
        ],
    ]
}

// ---------------------------------------------------------------------------
// Deformation gradient
// ---------------------------------------------------------------------------

/// Compute deformation gradient F for a triangle given rest and deformed positions.
#[allow(dead_code)]
pub fn compute_deformation_gradient(
    rest_pos: &[[f32; 3]],
    def_pos: &[[f32; 3]],
    face: [usize; 3],
) -> DeformationGradient {
    let [i0, i1, i2] = face;
    // Edge vectors in rest and deformed config
    let dr1 = sub3(rest_pos[i1], rest_pos[i0]);
    let dr2 = sub3(rest_pos[i2], rest_pos[i0]);
    let dd1 = sub3(def_pos[i1], def_pos[i0]);
    let dd2 = sub3(def_pos[i2], def_pos[i0]);

    // Build Dm and Ds (3x2 as two column vectors embedded in 3x3 with a normal third column)
    let rest_normal = cross3(dr1, dr2);
    let def_normal = cross3(dd1, dd2);

    // Form 3x3 matrices: columns are [dr1, dr2, rest_normal_hat] and [dd1, dd2, def_normal_hat]
    let rn_len = vec3_len(rest_normal).max(1e-12);
    let dn_len = vec3_len(def_normal).max(1e-12);
    let rn_hat = [
        rest_normal[0] / rn_len,
        rest_normal[1] / rn_len,
        rest_normal[2] / rn_len,
    ];
    let dn_hat = [
        def_normal[0] / dn_len,
        def_normal[1] / dn_len,
        def_normal[2] / dn_len,
    ];

    // Dm = [dr1 | dr2 | rn_hat] (columns)
    let dm = cols_to_mat3(dr1, dr2, rn_hat);
    let ds = cols_to_mat3(dd1, dd2, dn_hat);

    // F = Ds * Dm^{-1}
    let dm_inv = mat3_inv(dm);
    let f = mat3_mul(ds, dm_inv);
    let det_f = mat3_det(f);
    DeformationGradient {
        f,
        det_f,
        volume_ratio: det_f,
    }
}

fn cols_to_mat3(c0: [f32; 3], c1: [f32; 3], c2: [f32; 3]) -> [[f32; 3]; 3] {
    [
        [c0[0], c1[0], c2[0]],
        [c0[1], c1[1], c2[1]],
        [c0[2], c1[2], c2[2]],
    ]
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn vec3_len(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

// ---------------------------------------------------------------------------
// Analysis helpers
// ---------------------------------------------------------------------------

/// Compute ||R^T R - I||_F, a measure of how close R is to orthogonal.
#[allow(dead_code)]
pub fn rotation_error(r: &[[f32; 3]; 3]) -> f32 {
    let rt = mat3_transpose(*r);
    let rtr = mat3_mul(rt, *r);
    let id = mat3_identity();
    let mut err = 0.0f32;
    for i in 0..3 {
        for j in 0..3 {
            let d = rtr[i][j] - id[i][j];
            err += d * d;
        }
    }
    err.sqrt()
}

/// Ratio of maximum to minimum diagonal of the symmetric stretch matrix.
#[allow(dead_code)]
pub fn stretch_ratio(pd: &PolarDecomp) -> f32 {
    let diag = [pd.symmetric[0][0], pd.symmetric[1][1], pd.symmetric[2][2]];
    let max_d = diag.iter().cloned().fold(f32::NEG_INFINITY, f32::max).abs();
    let min_d = diag
        .iter()
        .cloned()
        .fold(f32::INFINITY, f32::min)
        .abs()
        .max(1e-12);
    max_d / min_d
}

/// Compute per-face deformation gradients for all triangles.
#[allow(dead_code)]
pub fn per_face_deformation(
    rest: &[[f32; 3]],
    deformed: &[[f32; 3]],
    indices: &[u32],
) -> Vec<DeformationGradient> {
    let tri_count = indices.len() / 3;
    let mut result = Vec::with_capacity(tri_count);
    for t in 0..tri_count {
        let face = [
            indices[t * 3] as usize,
            indices[t * 3 + 1] as usize,
            indices[t * 3 + 2] as usize,
        ];
        result.push(compute_deformation_gradient(rest, deformed, face));
    }
    result
}

/// Divergence of the deformation field: |det_f - 1| per element.
#[allow(dead_code)]
pub fn deformation_field_divergence(gradients: &[DeformationGradient]) -> Vec<f32> {
    gradients.iter().map(|g| (g.det_f - 1.0).abs()).collect()
}

/// Returns true if the deformation is close to rigid body (symmetric part near identity).
#[allow(dead_code)]
pub fn rigid_body_deformation(pd: &PolarDecomp) -> bool {
    let id = mat3_identity();
    let mut diff = 0.0f32;
    for (row_s, row_id) in pd.symmetric.iter().zip(id.iter()) {
        for (s, i) in row_s.iter().zip(row_id.iter()) {
            let d = s - i;
            diff += d * d;
        }
    }
    diff.sqrt() < 0.05
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> [[f32; 3]; 3] {
        mat3_identity()
    }

    #[test]
    fn test_mat3_mul_identity() {
        let a = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let id = identity();
        let result = mat3_mul(a, id);
        for i in 0..3 {
            for j in 0..3 {
                assert!((result[i][j] - a[i][j]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_mat3_det_identity() {
        let det = mat3_det(identity());
        assert!((det - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_mat3_det_zero() {
        let m = [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let det = mat3_det(m);
        assert!(det.abs() < 1e-5);
    }

    #[test]
    fn test_mat3_transpose() {
        let m = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let t = mat3_transpose(m);
        for i in 0..3 {
            for j in 0..3 {
                assert!((t[i][j] - m[j][i]).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_mat3_identity() {
        let id = mat3_identity();
        for (i, row) in id.iter().enumerate() {
            for (j, val) in row.iter().enumerate() {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((val - expected).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn test_mat3_scale() {
        let id = identity();
        let scaled = mat3_scale(id, 2.0);
        assert!((scaled[0][0] - 2.0).abs() < 1e-5);
        assert!((scaled[0][1]).abs() < 1e-5);
    }

    #[test]
    fn test_polar_decompose_identity() {
        let id = identity();
        let pd = polar_decompose(id);
        assert!(
            rotation_error(&pd.rotation) < 1e-4,
            "rotation should be near orthogonal"
        );
    }

    #[test]
    fn test_rotation_error_identity() {
        let id = identity();
        let err = rotation_error(&id);
        assert!(err < 1e-5, "identity rotation error should be 0, got {err}");
    }

    #[test]
    fn test_rigid_body_deformation_identity() {
        let id = identity();
        let pd = polar_decompose(id);
        assert!(
            rigid_body_deformation(&pd),
            "identity deformation should be rigid"
        );
    }

    #[test]
    fn test_stretch_ratio_identity() {
        let id = identity();
        let pd = polar_decompose(id);
        let sr = stretch_ratio(&pd);
        assert!((0.99..=1.5).contains(&sr), "stretch ratio near 1, got {sr}");
    }

    #[test]
    fn test_compute_deformation_gradient_rigid() {
        // rest and deformed are the same → F should be near identity
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let dg = compute_deformation_gradient(&pos, &pos, [0, 1, 2]);
        assert!(
            (dg.det_f - 1.0).abs() < 0.1,
            "det F should be near 1, got {}",
            dg.det_f
        );
    }

    #[test]
    fn test_per_face_deformation_count() {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let idx: Vec<u32> = vec![0, 1, 2, 1, 3, 2];
        let grads = per_face_deformation(&pos, &pos, &idx);
        assert_eq!(grads.len(), 2);
    }

    #[test]
    fn test_deformation_field_divergence_identity() {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx: Vec<u32> = vec![0, 1, 2];
        let grads = per_face_deformation(&pos, &pos, &idx);
        let div = deformation_field_divergence(&grads);
        assert!(!div.is_empty());
        // |det_f - 1| should be small for no deformation
        assert!(
            div[0] < 0.5,
            "divergence from identity should be small, got {}",
            div[0]
        );
    }

    #[test]
    fn test_polar_decompose_scale_matrix() {
        // Scale by 2 in X only
        let m = [[2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let pd = polar_decompose(m);
        // Rotation should still be near orthogonal
        assert!(rotation_error(&pd.rotation) < 1e-3);
        // Not rigid body (scaled)
        assert!(!rigid_body_deformation(&pd));
    }
}
