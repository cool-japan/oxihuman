// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Linear Blend Skinning (LBS) and Dual Quaternion Blend Skinning (DQS).
//!
//! Vertices carry a list of (bone_index, weight) pairs.  Bone transforms are
//! represented as 4×4 column-major matrices (index layout: `m[col * 4 + row]`).
//! Dual quaternions are stored as `[qr_w, qr_x, qr_y, qr_z, qd_w, qd_x, qd_y, qd_z]`.

/// Skinning algorithm selector.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinningMethod {
    /// Classic linear blend skinning (can exhibit candy-wrapper artefacts).
    Linear,
    /// Dual quaternion blend skinning (volume-preserving).
    DualQuaternion,
}

/// Per-vertex skinning data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinVertex {
    /// Rest-pose position `[x, y, z]`.
    pub position: [f32; 3],
    /// Bone indices parallel to `weights`.
    pub bone_indices: Vec<usize>,
    /// Blend weights (should sum to 1.0 after normalisation).
    pub weights: Vec<f32>,
}

/// Output of a skinning pass.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinningResult {
    /// Deformed vertex positions (`[x, y, z]` per vertex).
    pub positions: Vec<[f32; 3]>,
    pub method: SkinningMethod,
}

// ── Type aliases ─────────────────────────────────────────────────────────────

/// Column-major 4×4 transform matrix.
pub type Mat4 = [f32; 16];

/// Dual quaternion: `[qr_w, qr_x, qr_y, qr_z, qd_w, qd_x, qd_y, qd_z]`.
pub type DualQuat = [f32; 8];

// ── Public API ────────────────────────────────────────────────────────────────

/// Return the default skinning method (`Linear`).
#[allow(dead_code)]
pub fn default_skinning_method() -> SkinningMethod {
    SkinningMethod::Linear
}

/// Apply linear blend skinning to a single vertex.
///
/// `matrices` is a flat slice of 4×4 column-major bone matrices (len = 16 × B).
#[allow(dead_code)]
pub fn lbs_transform_vertex(vertex: &SkinVertex, matrices: &[Mat4]) -> [f32; 3] {
    let mut out = [0.0f32; 3];
    let p = vertex.position;
    for (&bi, &w) in vertex.bone_indices.iter().zip(vertex.weights.iter()) {
        if bi >= matrices.len() || w == 0.0 {
            continue;
        }
        let m = &matrices[bi];
        // mat4 × point (homogeneous w=1)
        out[0] += w * (m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12]);
        out[1] += w * (m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13]);
        out[2] += w * (m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14]);
    }
    out
}

/// Apply dual quaternion skinning to a single vertex.
///
/// `dqs` – one dual quaternion per bone.
#[allow(dead_code)]
pub fn dqs_transform_vertex(vertex: &SkinVertex, dqs: &[DualQuat]) -> [f32; 3] {
    // Blend dual quaternions linearly then normalise the real part.
    let mut blended = [0.0f32; 8];
    let pivot_sign = if vertex.bone_indices.is_empty() {
        1.0f32
    } else {
        dqs[vertex.bone_indices[0]][0].signum()
    };
    for (&bi, &w) in vertex.bone_indices.iter().zip(vertex.weights.iter()) {
        if bi >= dqs.len() || w == 0.0 {
            continue;
        }
        let dq = &dqs[bi];
        // Antipodality correction.
        let sign = if dq[0] * pivot_sign < 0.0 { -1.0 } else { 1.0 };
        for k in 0..8 {
            blended[k] += w * sign * dq[k];
        }
    }
    // Normalise real quaternion part.
    let qr_len = (blended[0] * blended[0]
        + blended[1] * blended[1]
        + blended[2] * blended[2]
        + blended[3] * blended[3])
        .sqrt();
    if qr_len < 1e-8 {
        return vertex.position;
    }
    for b in &mut blended {
        *b /= qr_len;
    }
    dq_transform_point(&blended, vertex.position)
}

/// Skin all vertices using LBS.
#[allow(dead_code)]
pub fn apply_lbs(vertices: &[SkinVertex], matrices: &[Mat4]) -> SkinningResult {
    let positions = vertices
        .iter()
        .map(|v| lbs_transform_vertex(v, matrices))
        .collect();
    SkinningResult {
        positions,
        method: SkinningMethod::Linear,
    }
}

/// Skin all vertices using DQS.
#[allow(dead_code)]
pub fn apply_dqs(vertices: &[SkinVertex], dqs: &[DualQuat]) -> SkinningResult {
    let positions = vertices
        .iter()
        .map(|v| dqs_transform_vertex(v, dqs))
        .collect();
    SkinningResult {
        positions,
        method: SkinningMethod::DualQuaternion,
    }
}

/// Normalise per-vertex blend weights so they sum to 1.0.
#[allow(dead_code)]
pub fn normalize_weights(vertices: &mut [SkinVertex]) {
    for v in vertices.iter_mut() {
        let total: f32 = v.weights.iter().sum();
        if total > 1e-8 {
            for w in v.weights.iter_mut() {
                *w /= total;
            }
        }
    }
}

/// Return the number of vertices in a skinning result.
#[allow(dead_code)]
pub fn skinning_vertex_count(result: &SkinningResult) -> usize {
    result.positions.len()
}

/// Return the maximum number of bone influences across all vertices.
#[allow(dead_code)]
pub fn max_influences(vertices: &[SkinVertex]) -> usize {
    vertices.iter().map(|v| v.bone_indices.len()).max().unwrap_or(0)
}

/// Return a clone of vertices as their "bind pose" (identity transform).
///
/// Useful for resetting deformation.
#[allow(dead_code)]
pub fn skin_bind_pose(vertices: &[SkinVertex]) -> Vec<[f32; 3]> {
    vertices.iter().map(|v| v.position).collect()
}

/// Return the deformed positions from a `SkinningResult`.
#[allow(dead_code)]
pub fn skin_current_pose(result: &SkinningResult) -> &[[f32; 3]] {
    &result.positions
}

/// Validate that each vertex has at least one influence and weights sum ≈ 1.0.
///
/// Returns a list of problematic vertex indices.
#[allow(dead_code)]
pub fn validate_skin_weights(vertices: &[SkinVertex]) -> Vec<usize> {
    let mut bad = Vec::new();
    for (i, v) in vertices.iter().enumerate() {
        if v.bone_indices.is_empty() || v.weights.is_empty() {
            bad.push(i);
            continue;
        }
        let total: f32 = v.weights.iter().sum();
        if (total - 1.0).abs() > 0.01 {
            bad.push(i);
        }
    }
    bad
}

/// Blend two skinning results linearly by `alpha` (0 = a, 1 = b).
#[allow(dead_code)]
pub fn blend_skinning_results(a: &SkinningResult, b: &SkinningResult, alpha: f32) -> SkinningResult {
    let len = a.positions.len().min(b.positions.len());
    let positions = (0..len)
        .map(|i| {
            let pa = a.positions[i];
            let pb = b.positions[i];
            let t = alpha.clamp(0.0, 1.0);
            [
                pa[0] + t * (pb[0] - pa[0]),
                pa[1] + t * (pb[1] - pa[1]),
                pa[2] + t * (pb[2] - pa[2]),
            ]
        })
        .collect();
    SkinningResult {
        positions,
        method: a.method,
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Transform a point by a dual quaternion (assumes unit real part).
fn dq_transform_point(dq: &DualQuat, p: [f32; 3]) -> [f32; 3] {
    // Recover translation from dual part: t = 2 * qd * conj(qr)
    let (rw, rx, ry, rz) = (dq[0], dq[1], dq[2], dq[3]);
    let (dw, dx, dy, dz) = (dq[4], dq[5], dq[6], dq[7]);
    let tx = 2.0 * (-dw * rx + dx * rw - dy * rz + dz * ry);
    let ty = 2.0 * (-dw * ry + dx * rz + dy * rw - dz * rx);
    let tz = 2.0 * (-dw * rz - dx * ry + dy * rx + dz * rw);
    // Rotate point by real quaternion then add translation.
    let rotated = quat_rotate_point([rw, rx, ry, rz], p);
    [rotated[0] + tx, rotated[1] + ty, rotated[2] + tz]
}

fn quat_rotate_point(q: [f32; 4], p: [f32; 3]) -> [f32; 3] {
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    let (px, py, pz) = (p[0], p[1], p[2]);
    // v' = q * v * q^-1
    let ix = w * px + y * pz - z * py;
    let iy = w * py + z * px - x * pz;
    let iz = w * pz + x * py - y * px;
    let iw = -x * px - y * py - z * pz;
    [
        ix * w + iw * -x + iy * -z - iz * -y,
        iy * w + iw * -y + iz * -x - ix * -z,
        iz * w + iw * -z + ix * -y - iy * -x,
    ]
}

/// Identity 4×4 matrix (column-major).
#[allow(dead_code)]
pub fn identity_mat4() -> Mat4 {
    [
        1.0, 0.0, 0.0, 0.0, // col 0
        0.0, 1.0, 0.0, 0.0, // col 1
        0.0, 0.0, 1.0, 0.0, // col 2
        0.0, 0.0, 0.0, 1.0, // col 3
    ]
}

/// Identity dual quaternion.
#[allow(dead_code)]
pub fn identity_dq() -> DualQuat {
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vertex(pos: [f32; 3]) -> SkinVertex {
        SkinVertex {
            position: pos,
            bone_indices: vec![0],
            weights: vec![1.0],
        }
    }

    #[test]
    fn test_default_skinning_method() {
        assert_eq!(default_skinning_method(), SkinningMethod::Linear);
    }

    #[test]
    fn test_lbs_identity_transform() {
        let v = make_vertex([1.0, 2.0, 3.0]);
        let m = identity_mat4();
        let out = lbs_transform_vertex(&v, &[m]);
        assert!((out[0] - 1.0).abs() < 1e-5);
        assert!((out[1] - 2.0).abs() < 1e-5);
        assert!((out[2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_lbs_translation() {
        let v = make_vertex([0.0, 0.0, 0.0]);
        let mut m = identity_mat4();
        m[12] = 5.0; // tx
        m[13] = 3.0; // ty
        let out = lbs_transform_vertex(&v, &[m]);
        assert!((out[0] - 5.0).abs() < 1e-5);
        assert!((out[1] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_dqs_identity_transform() {
        let v = make_vertex([1.0, 2.0, 3.0]);
        let dq = identity_dq();
        let out = dqs_transform_vertex(&v, &[dq]);
        assert!((out[0] - 1.0).abs() < 1e-4);
        assert!((out[1] - 2.0).abs() < 1e-4);
        assert!((out[2] - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_apply_lbs_vertex_count() {
        let verts: Vec<SkinVertex> = (0..5).map(|i| make_vertex([i as f32, 0.0, 0.0])).collect();
        let m = identity_mat4();
        let result = apply_lbs(&verts, &[m]);
        assert_eq!(skinning_vertex_count(&result), 5);
        assert_eq!(result.method, SkinningMethod::Linear);
    }

    #[test]
    fn test_apply_dqs_vertex_count() {
        let verts: Vec<SkinVertex> = (0..3).map(|i| make_vertex([i as f32, 0.0, 0.0])).collect();
        let dq = identity_dq();
        let result = apply_dqs(&verts, &[dq]);
        assert_eq!(skinning_vertex_count(&result), 3);
        assert_eq!(result.method, SkinningMethod::DualQuaternion);
    }

    #[test]
    fn test_normalize_weights_already_normalized() {
        let mut verts = vec![SkinVertex {
            position: [0.0; 3],
            bone_indices: vec![0, 1],
            weights: vec![0.5, 0.5],
        }];
        normalize_weights(&mut verts);
        let total: f32 = verts[0].weights.iter().sum();
        assert!((total - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_weights_unnormalized() {
        let mut verts = vec![SkinVertex {
            position: [0.0; 3],
            bone_indices: vec![0, 1],
            weights: vec![2.0, 2.0],
        }];
        normalize_weights(&mut verts);
        let total: f32 = verts[0].weights.iter().sum();
        assert!((total - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_influences() {
        let verts = vec![
            SkinVertex { position: [0.0; 3], bone_indices: vec![0], weights: vec![1.0] },
            SkinVertex { position: [0.0; 3], bone_indices: vec![0, 1, 2], weights: vec![0.4, 0.4, 0.2] },
        ];
        assert_eq!(max_influences(&verts), 3);
    }

    #[test]
    fn test_max_influences_empty() {
        assert_eq!(max_influences(&[]), 0);
    }

    #[test]
    fn test_skin_bind_pose() {
        let verts: Vec<SkinVertex> = (0..3)
            .map(|i| make_vertex([i as f32, 0.0, 0.0]))
            .collect();
        let bp = skin_bind_pose(&verts);
        assert_eq!(bp.len(), 3);
        assert!((bp[1][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_skin_current_pose() {
        let verts = vec![make_vertex([1.0, 0.0, 0.0])];
        let m = identity_mat4();
        let result = apply_lbs(&verts, &[m]);
        let pose = skin_current_pose(&result);
        assert_eq!(pose.len(), 1);
    }

    #[test]
    fn test_validate_skin_weights_valid() {
        let verts = vec![make_vertex([0.0; 3])];
        let bad = validate_skin_weights(&verts);
        assert!(bad.is_empty());
    }

    #[test]
    fn test_validate_skin_weights_empty_bones() {
        let verts = vec![SkinVertex {
            position: [0.0; 3],
            bone_indices: vec![],
            weights: vec![],
        }];
        let bad = validate_skin_weights(&verts);
        assert_eq!(bad, vec![0]);
    }

    #[test]
    fn test_validate_skin_weights_bad_sum() {
        let verts = vec![SkinVertex {
            position: [0.0; 3],
            bone_indices: vec![0],
            weights: vec![0.5],
        }];
        let bad = validate_skin_weights(&verts);
        assert_eq!(bad, vec![0]);
    }

    #[test]
    fn test_blend_skinning_results_alpha_0() {
        let a = SkinningResult { positions: vec![[0.0, 0.0, 0.0]], method: SkinningMethod::Linear };
        let b = SkinningResult { positions: vec![[10.0, 0.0, 0.0]], method: SkinningMethod::Linear };
        let out = blend_skinning_results(&a, &b, 0.0);
        assert!((out.positions[0][0]).abs() < 1e-5);
    }

    #[test]
    fn test_blend_skinning_results_alpha_1() {
        let a = SkinningResult { positions: vec![[0.0, 0.0, 0.0]], method: SkinningMethod::Linear };
        let b = SkinningResult { positions: vec![[10.0, 0.0, 0.0]], method: SkinningMethod::Linear };
        let out = blend_skinning_results(&a, &b, 1.0);
        assert!((out.positions[0][0] - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_blend_skinning_results_alpha_half() {
        let a = SkinningResult { positions: vec![[0.0, 0.0, 0.0]], method: SkinningMethod::Linear };
        let b = SkinningResult { positions: vec![[10.0, 0.0, 0.0]], method: SkinningMethod::Linear };
        let out = blend_skinning_results(&a, &b, 0.5);
        assert!((out.positions[0][0] - 5.0).abs() < 1e-5);
    }
}
