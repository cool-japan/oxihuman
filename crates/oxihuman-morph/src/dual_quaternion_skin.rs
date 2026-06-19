// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dual quaternion skinning (DQS) stub.

/// A dual quaternion: real part q0, dual part qe.
#[derive(Debug, Clone, Copy)]
pub struct DualQuat {
    pub q0: [f32; 4],
    pub qe: [f32; 4],
}

impl DualQuat {
    pub fn identity() -> Self {
        DualQuat {
            q0: [0.0, 0.0, 0.0, 1.0],
            qe: [0.0; 4],
        }
    }
}

/// DQS vertex binding.
#[derive(Debug, Clone)]
pub struct DqsVertex {
    pub bone_indices: [usize; 4],
    pub weights: [f32; 4],
}

impl Default for DqsVertex {
    fn default() -> Self {
        DqsVertex {
            bone_indices: [0; 4],
            weights: [0.0; 4],
        }
    }
}

/// Dual quaternion skinning mesh.
#[derive(Debug, Clone)]
pub struct DualQuaternionSkin {
    pub vertices: Vec<DqsVertex>,
    pub bone_dqs: Vec<DualQuat>,
}

impl DualQuaternionSkin {
    pub fn new(vertex_count: usize, bone_count: usize) -> Self {
        DualQuaternionSkin {
            vertices: (0..vertex_count).map(|_| DqsVertex::default()).collect(),
            bone_dqs: (0..bone_count).map(|_| DualQuat::identity()).collect(),
        }
    }
}

/// Create a new DQS mesh.
pub fn new_dqs(vertex_count: usize, bone_count: usize) -> DualQuaternionSkin {
    DualQuaternionSkin::new(vertex_count, bone_count)
}

/// Set bone influences for a vertex.
#[allow(clippy::too_many_arguments)]
pub fn dqs_set_vertex(
    dqs: &mut DualQuaternionSkin,
    vertex: usize,
    b0: usize,
    w0: f32,
    b1: usize,
    w1: f32,
    b2: usize,
    w2: f32,
) {
    if vertex < dqs.vertices.len() {
        dqs.vertices[vertex] = DqsVertex {
            bone_indices: [b0, b1, b2, 0],
            weights: [w0, w1, w2, 0.0],
        };
    }
}

/// Set a bone's dual quaternion transform.
pub fn dqs_set_bone(dqs: &mut DualQuaternionSkin, bone: usize, dq: DualQuat) {
    if bone < dqs.bone_dqs.len() {
        dqs.bone_dqs[bone] = dq;
    }
}

/// Normalize a dual quaternion.
pub fn dqs_normalize(dq: &DualQuat) -> DualQuat {
    let len =
        (dq.q0[0] * dq.q0[0] + dq.q0[1] * dq.q0[1] + dq.q0[2] * dq.q0[2] + dq.q0[3] * dq.q0[3])
            .sqrt();
    if len < 1e-9 {
        return DualQuat::identity();
    }
    DualQuat {
        q0: [
            dq.q0[0] / len,
            dq.q0[1] / len,
            dq.q0[2] / len,
            dq.q0[3] / len,
        ],
        qe: [
            dq.qe[0] / len,
            dq.qe[1] / len,
            dq.qe[2] / len,
            dq.qe[3] / len,
        ],
    }
}

/// Return vertex count.
pub fn dqs_vertex_count(dqs: &DualQuaternionSkin) -> usize {
    dqs.vertices.len()
}

/// Return bone count.
pub fn dqs_bone_count(dqs: &DualQuaternionSkin) -> usize {
    dqs.bone_dqs.len()
}

// ---------------------------------------------------------------------------
// Internal quaternion helpers (all represented as [x, y, z, w])
// ---------------------------------------------------------------------------

/// Multiply two quaternions: result = a * b (Hamilton product).
/// Convention: [x, y, z, w] where w is the scalar part.
#[inline]
fn q_mul(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let (ax, ay, az, aw) = (a[0], a[1], a[2], a[3]);
    let (bx, by, bz, bw) = (b[0], b[1], b[2], b[3]);
    [
        aw * bx + ax * bw + ay * bz - az * by,
        aw * by - ax * bz + ay * bw + az * bx,
        aw * bz + ax * by - ay * bx + az * bw,
        aw * bw - ax * bx - ay * by - az * bz,
    ]
}

/// Conjugate of a unit quaternion: negate the vector part.
#[inline]
fn q_conj(q: [f32; 4]) -> [f32; 4] {
    [-q[0], -q[1], -q[2], q[3]]
}

/// Dot product of the real (scalar) parts of two quaternions' `q0` components.
#[inline]
fn q0_dot(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// Scale a dual quaternion by a scalar.
#[inline]
fn dq_scale(dq: DualQuat, s: f32) -> DualQuat {
    DualQuat {
        q0: [dq.q0[0] * s, dq.q0[1] * s, dq.q0[2] * s, dq.q0[3] * s],
        qe: [dq.qe[0] * s, dq.qe[1] * s, dq.qe[2] * s, dq.qe[3] * s],
    }
}

/// Add two dual quaternions component-wise.
#[inline]
fn dq_add(a: DualQuat, b: DualQuat) -> DualQuat {
    DualQuat {
        q0: [
            a.q0[0] + b.q0[0],
            a.q0[1] + b.q0[1],
            a.q0[2] + b.q0[2],
            a.q0[3] + b.q0[3],
        ],
        qe: [
            a.qe[0] + b.qe[0],
            a.qe[1] + b.qe[1],
            a.qe[2] + b.qe[2],
            a.qe[3] + b.qe[3],
        ],
    }
}

/// Zero dual quaternion (additive identity for blending).
#[inline]
fn dq_zero() -> DualQuat {
    DualQuat {
        q0: [0.0; 4],
        qe: [0.0; 4],
    }
}

// ---------------------------------------------------------------------------
// Public DQS transform functions
// ---------------------------------------------------------------------------

/// Transform a single vertex position using Dual-Quaternion Linear Blending (DLB).
///
/// Algorithm:
/// 1. Gather the 4 `(bone_index, weight)` pairs for `vertex_idx`.
/// 2. Accumulate a blended DQ: `blend += w_k * bone_dqs[bone_indices[k]]`, applying the
///    antipodal sign fix (`dot(blend.q0, dq_k.q0) < 0` → negate `dq_k` before adding).
/// 3. Normalize the blended DQ.
/// 4. Apply rotation via quaternion sandwich: `q0 * [px,py,pz,0] * conj(q0)`.
/// 5. Extract translation from the dual part: `t = 2 * qe * conj(q0)`, take `.xyz`.
/// 6. Return `rotated_pos + t`.
///
/// If `vertex_idx` is out of range, returns `pos` unchanged.
pub fn dqs_transform_vertex(
    skin: &DualQuaternionSkin,
    vertex_idx: usize,
    pos: [f32; 3],
) -> [f32; 3] {
    let Some(vtx) = skin.vertices.get(vertex_idx) else {
        return pos;
    };

    let mut blend = dq_zero();

    for k in 0..4 {
        let w = vtx.weights[k];
        if w.abs() < 1e-12 {
            continue;
        }
        let bone_idx = vtx.bone_indices[k];
        let Some(&dq_k) = skin.bone_dqs.get(bone_idx) else {
            continue;
        };

        // Antipodal sign fix: ensure all blended quaternions live in the same hemisphere.
        let dq_k_signed = if q0_dot(blend.q0, dq_k.q0) < 0.0 {
            dq_scale(dq_k, -1.0)
        } else {
            dq_k
        };

        blend = dq_add(blend, dq_scale(dq_k_signed, w));
    }

    // Normalize the blended dual quaternion.
    let n = dqs_normalize(&blend);

    // Step 4: rotate position via sandwich product q0 * p * conj(q0).
    let p_quat: [f32; 4] = [pos[0], pos[1], pos[2], 0.0];
    let q0 = n.q0;
    let rotated_quat = q_mul(q_mul(q0, p_quat), q_conj(q0));
    let rot_pos = [rotated_quat[0], rotated_quat[1], rotated_quat[2]];

    // Step 5: extract translation from dual part.
    // t_quat = 2 * qe * conj(q0)  →  xyz are the translation vector.
    let qe_scaled = [n.qe[0] * 2.0, n.qe[1] * 2.0, n.qe[2] * 2.0, n.qe[3] * 2.0];
    let t_quat = q_mul(qe_scaled, q_conj(q0));

    // Step 6: result = rotated_pos + translation_xyz.
    [
        rot_pos[0] + t_quat[0],
        rot_pos[1] + t_quat[1],
        rot_pos[2] + t_quat[2],
    ]
}

/// Transform every position in `positions` using `dqs_transform_vertex`.
///
/// The `i`-th element of `positions` is transformed using vertex binding `i`.
/// If `positions` is longer than `skin.vertices`, extra positions are passed through unchanged.
pub fn dqs_transform_all(skin: &DualQuaternionSkin, positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
    positions
        .iter()
        .enumerate()
        .map(|(i, &p)| dqs_transform_vertex(skin, i, p))
        .collect()
}

/// Return a JSON-like string.
pub fn dqs_to_json(dqs: &DualQuaternionSkin) -> String {
    format!(
        r#"{{"vertices":{},"bones":{}}}"#,
        dqs.vertices.len(),
        dqs.bone_dqs.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dqs_vertex_count() {
        let d = new_dqs(15, 4);
        assert_eq!(dqs_vertex_count(&d), 15 /* vertex count must match */,);
    }

    #[test]
    fn test_new_dqs_bone_count() {
        let d = new_dqs(5, 8);
        assert_eq!(dqs_bone_count(&d), 8 /* bone count must match */,);
    }

    #[test]
    fn test_identity_dq_real_part() {
        let dq = DualQuat::identity();
        assert!((dq.q0[3] - 1.0).abs() < 1e-5, /* identity w component should be 1 */);
    }

    #[test]
    fn test_normalize_identity_stays_identity() {
        let dq = DualQuat::identity();
        let n = dqs_normalize(&dq);
        assert!((n.q0[3] - 1.0).abs() < 1e-5, /* normalized identity w should still be 1 */);
    }

    #[test]
    fn test_set_bone_updates() {
        let mut d = new_dqs(2, 2);
        let dq = DualQuat {
            q0: [0.0, 0.0, 0.707, 0.707],
            qe: [0.0; 4],
        };
        dqs_set_bone(&mut d, 0, dq);
        assert!((d.bone_dqs[0].q0[2] - 0.707).abs() < 1e-3, /* bone DQ z component must match */);
    }

    #[test]
    fn test_set_bone_out_of_bounds_ignored() {
        let mut d = new_dqs(2, 2);
        dqs_set_bone(&mut d, 99, DualQuat::identity());
        assert_eq!(dqs_bone_count(&d), 2 /* bone count unchanged */,);
    }

    #[test]
    fn test_set_vertex_out_of_bounds_ignored() {
        let mut d = new_dqs(2, 2);
        dqs_set_vertex(&mut d, 99, 0, 1.0, 0, 0.0, 0, 0.0);
        assert_eq!(dqs_vertex_count(&d), 2 /* vertex count unchanged */,);
    }

    #[test]
    fn test_to_json_contains_vertices() {
        let d = new_dqs(6, 3);
        let j = dqs_to_json(&d);
        assert!(j.contains("vertices") /* JSON must contain vertices */,);
    }

    #[test]
    fn test_identity_dual_part_zero() {
        let dq = DualQuat::identity();
        for &v in &dq.qe {
            assert!((v).abs() < 1e-6, /* identity dual part should be zero */);
        }
    }

    #[test]
    fn test_normalize_zero_length_returns_identity() {
        let dq = DualQuat {
            q0: [0.0; 4],
            qe: [0.0; 4],
        };
        let n = dqs_normalize(&dq);
        assert!((n.q0[3] - 1.0).abs() < 1e-5, /* zero DQ normalizes to identity */);
    }

    // ----- dqs_transform_vertex tests -----

    /// A single identity bone with full weight must leave the position unchanged.
    #[test]
    fn test_transform_identity_bone_no_change() {
        let mut skin = new_dqs(1, 1);
        // vertex 0: fully bound to bone 0 with weight 1.0
        dqs_set_vertex(&mut skin, 0, 0, 1.0, 0, 0.0, 0, 0.0);
        // bone 0 is the identity DQ (set by default)
        let pos = [1.0f32, 2.0, 3.0];
        let out = dqs_transform_vertex(&skin, 0, pos);
        assert!(
            (out[0] - pos[0]).abs() < 1e-4,
            "identity: x unchanged, got {}",
            out[0]
        );
        assert!(
            (out[1] - pos[1]).abs() < 1e-4,
            "identity: y unchanged, got {}",
            out[1]
        );
        assert!(
            (out[2] - pos[2]).abs() < 1e-4,
            "identity: z unchanged, got {}",
            out[2]
        );
    }

    /// A pure-translation bone (no rotation, translation = [1, 0, 0]) must shift the position.
    ///
    /// The dual part of a unit DQ that encodes translation `t` is:
    ///   qe = 0.5 * t_quat * q0
    /// For identity rotation q0 = [0,0,0,1] and t = [tx, 0, 0]:
    ///   t_quat = [tx, 0, 0, 0]
    ///   qe     = 0.5 * [tx,0,0,0] * [0,0,0,1] = [tx/2, 0, 0, 0]
    #[test]
    fn test_transform_pure_translation_bone() {
        let tx = 3.0f32;
        let mut skin = new_dqs(1, 1);
        dqs_set_vertex(&mut skin, 0, 0, 1.0, 0, 0.0, 0, 0.0);

        // Encode translation tx along X into the dual part.
        let dq_translate = DualQuat {
            q0: [0.0, 0.0, 0.0, 1.0], // identity rotation
            qe: [tx / 2.0, 0.0, 0.0, 0.0],
        };
        dqs_set_bone(&mut skin, 0, dq_translate);

        let pos = [1.0f32, 2.0, 3.0];
        let out = dqs_transform_vertex(&skin, 0, pos);
        assert!(
            (out[0] - (pos[0] + tx)).abs() < 1e-3,
            "translation: expected x={}, got {}",
            pos[0] + tx,
            out[0]
        );
        assert!(
            (out[1] - pos[1]).abs() < 1e-3,
            "translation: y unchanged, got {}",
            out[1]
        );
        assert!(
            (out[2] - pos[2]).abs() < 1e-3,
            "translation: z unchanged, got {}",
            out[2]
        );
    }

    /// A 180° rotation around Z must map [1,0,0] → [-1,0,0] and [0,1,0] → [0,-1,0].
    /// q0 for 180° around Z: [0, 0, sin(90°), cos(90°)] = [0, 0, 1, 0].
    #[test]
    fn test_transform_180_rotation_z() {
        let mut skin = new_dqs(1, 1);
        dqs_set_vertex(&mut skin, 0, 0, 1.0, 0, 0.0, 0, 0.0);

        // 180° rotation around Z: q0 = [0, 0, 1, 0] (unit quaternion)
        let dq_rot = DualQuat {
            q0: [0.0, 0.0, 1.0, 0.0],
            qe: [0.0; 4],
        };
        dqs_set_bone(&mut skin, 0, dq_rot);

        // [1, 0, 0] rotated 180° around Z should become [-1, 0, 0]
        let out = dqs_transform_vertex(&skin, 0, [1.0, 0.0, 0.0]);
        assert!(
            (out[0] - (-1.0)).abs() < 1e-3,
            "rot180z x: expected -1, got {}",
            out[0]
        );
        assert!(
            (out[1]).abs() < 1e-3,
            "rot180z y: expected 0, got {}",
            out[1]
        );
        assert!(
            (out[2]).abs() < 1e-3,
            "rot180z z: expected 0, got {}",
            out[2]
        );
    }

    /// dqs_transform_all must apply dqs_transform_vertex to every position.
    #[test]
    fn test_transform_all_identity() {
        let skin = new_dqs(3, 1);
        // All vertices default to bone_indices=[0,0,0,0] weights=[0,0,0,0], so blend = zero DQ
        // which normalises to identity → positions unchanged.
        // But default weights are [0.0;4] → all zero → blend remains zero → normalises identity.
        let positions: Vec<[f32; 3]> = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        // We need at least one bone with identity set and weight 1 on each vertex.
        // The default DqsVertex has all weights=0 so blend will be zero → normalise gives identity.
        let out = dqs_transform_all(&skin, &positions);
        assert_eq!(out.len(), 3, "transform_all output length must match input");
        // With zero blend, dqs_normalize returns identity, which maps pos to pos.
        for (i, (o, p)) in out.iter().zip(positions.iter()).enumerate() {
            for c in 0..3 {
                assert!(
                    (o[c] - p[c]).abs() < 1e-3,
                    "transform_all[{i}][{c}] mismatch"
                );
            }
        }
    }
}
