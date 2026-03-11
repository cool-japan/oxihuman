// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
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
}
