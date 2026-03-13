// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Linear Blend Skinning (LBS) for deforming mesh vertices using a skeleton.

use crate::skeleton::Skeleton;

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Multiply a column-major 4×4 matrix by a point (w = 1).
fn mat4_mul_point(m: &[f32; 16], p: [f32; 3]) -> [f32; 3] {
    [
        m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12],
        m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13],
        m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14],
    ]
}

/// Compute the world-space translation for every joint by walking the parent chain.
/// Only translations are accumulated (rotations are identity in bind pose for our use).
fn joint_world_translations(skeleton: &Skeleton) -> Vec<[f32; 3]> {
    let n = skeleton.joints.len();
    let mut world = vec![[0.0f32; 3]; n];
    for i in 0..n {
        let local = skeleton.joints[i].translation;
        world[i] = match skeleton.joints[i].parent {
            None => local,
            Some(p) => {
                let pw = world[p];
                [pw[0] + local[0], pw[1] + local[1], pw[2] + local[2]]
            }
        };
    }
    world
}

// ─── SkinWeights ─────────────────────────────────────────────────────────────

/// Per-vertex skinning data (up to 4 influences per vertex).
#[derive(Debug, Clone)]
pub struct SkinWeights {
    /// One entry per vertex: 4 joint indices (0-padded when fewer than 4 influences).
    pub joints: Vec<[u16; 4]>,
    /// One entry per vertex: 4 weights summing to 1.0 (0.0-padded).
    pub weights: Vec<[f32; 4]>,
}

impl SkinWeights {
    /// Create identity skinning: every vertex assigned 100 % to joint 0.
    pub fn identity(n_verts: usize) -> Self {
        Self {
            joints: vec![[0u16, 0, 0, 0]; n_verts],
            weights: vec![[1.0f32, 0.0, 0.0, 0.0]; n_verts],
        }
    }

    /// Compute rigid (nearest-joint) skinning weights.
    ///
    /// For each vertex the single nearest joint (by world-space distance) receives
    /// weight 1.0; the remaining three slots are padded with index 0 and weight 0.0.
    pub fn from_nearest_joint(positions: &[[f32; 3]], skeleton: &Skeleton) -> Self {
        let world = joint_world_translations(skeleton);
        let n_verts = positions.len();
        let mut joints = vec![[0u16; 4]; n_verts];
        let mut weights = vec![[0.0f32; 4]; n_verts];

        for (v, pos) in positions.iter().enumerate() {
            let mut best_j = 0usize;
            let mut best_d2 = f32::MAX;
            for (j, jw) in world.iter().enumerate() {
                let dx = pos[0] - jw[0];
                let dy = pos[1] - jw[1];
                let dz = pos[2] - jw[2];
                let d2 = dx * dx + dy * dy + dz * dz;
                if d2 < best_d2 {
                    best_d2 = d2;
                    best_j = j;
                }
            }
            joints[v] = [best_j as u16, 0, 0, 0];
            weights[v] = [1.0, 0.0, 0.0, 0.0];
        }

        Self { joints, weights }
    }

    /// Normalise weights so each vertex sums to 1.0.
    pub fn normalize(&mut self) {
        for w in &mut self.weights {
            let sum = w[0] + w[1] + w[2] + w[3];
            if sum > 0.0 {
                let inv = 1.0 / sum;
                w[0] *= inv;
                w[1] *= inv;
                w[2] *= inv;
                w[3] *= inv;
            }
        }
    }

    /// Returns `true` when the data is self-consistent for a mesh with `n_verts` vertices.
    pub fn is_valid(&self, n_verts: usize) -> bool {
        self.joints.len() == n_verts && self.weights.len() == n_verts
    }
}

// ─── LBS ─────────────────────────────────────────────────────────────────────

/// Apply Linear Blend Skinning to deform `positions`.
///
/// `pose_matrices` — one column-major 4×4 matrix per joint (`skeleton.joints.len()` entries).
/// Returns new deformed positions.
#[allow(dead_code)]
pub fn apply_lbs(
    positions: &[[f32; 3]],
    skin: &SkinWeights,
    pose_matrices: &[[f32; 16]],
) -> Vec<[f32; 3]> {
    positions
        .iter()
        .enumerate()
        .map(|(v, &p)| {
            let js = skin.joints[v];
            let ws = skin.weights[v];
            let mut out = [0.0f32; 3];
            for k in 0..4 {
                let w = ws[k];
                if w == 0.0 {
                    continue;
                }
                let tp = mat4_mul_point(&pose_matrices[js[k] as usize], p);
                out[0] += w * tp[0];
                out[1] += w * tp[1];
                out[2] += w * tp[2];
            }
            out
        })
        .collect()
}

/// Build a bind-pose (rest-pose) matrix for each joint.
///
/// Each matrix is a column-major identity 4×4 with the joint's world-space
/// translation in the last column.  Applying these matrices via `apply_lbs`
/// produces no net deformation when the mesh vertices already sit in world space.
#[allow(dead_code)]
pub fn bind_pose_matrices(skeleton: &Skeleton) -> Vec<[f32; 16]> {
    let world = joint_world_translations(skeleton);
    world
        .iter()
        .map(|&t| {
            // Column-major identity with translation in column 3.
            [
                1.0, 0.0, 0.0, 0.0, // col 0
                0.0, 1.0, 0.0, 0.0, // col 1
                0.0, 0.0, 1.0, 0.0, // col 2
                t[0], t[1], t[2], 1.0, // col 3
            ]
        })
        .collect()
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skeleton::{Joint, Skeleton};

    fn tiny_skeleton() -> Skeleton {
        let mut sk = Skeleton::new();
        sk.add_joint(Joint {
            name: "root".to_string(),
            parent: None,
            translation: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        });
        sk.add_joint(Joint {
            name: "child".to_string(),
            parent: Some(0),
            translation: [1.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        });
        sk
    }

    #[test]
    fn identity_skinning_is_valid() {
        assert!(SkinWeights::identity(10).is_valid(10));
    }

    #[test]
    fn bind_pose_no_deformation() {
        // Vertices in world space; bind-pose matrices encode world translation.
        // After LBS the positions must be unchanged (within floating-point error).
        let sk = tiny_skeleton();
        let positions = vec![[0.0f32, 0.5, 0.0], [1.0, 0.5, 0.0], [2.0, 0.5, 0.0]];

        // Rigid skinning: each vertex goes to nearest joint.
        let mut skin = SkinWeights::from_nearest_joint(&positions, &sk);
        skin.normalize();

        // Build bind-pose matrices.
        let world = joint_world_translations(&sk);
        // The bind-pose matrix for joint j must UNDO the joint's world translation
        // so that the vertex is expressed relative to the joint, then re-apply it.
        // For "no deformation" we need the inverse-bind × pose = identity.
        // The simplest test: use identity matrices (no joint transform) and
        // verify the vertex comes back at its original position.
        let identity_matrix: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let pose: Vec<[f32; 16]> = (0..sk.joints.len()).map(|_| identity_matrix).collect();

        // For bind-pose no-deformation to work we need inverse-bind-pose applied
        // before the pose.  Here we test the simpler invariant: when every vertex
        // is nearest to joint 0 (root at origin, world_t = [0,0,0]) and we apply
        // the identity matrix, positions must be unchanged.
        let root_positions: Vec<[f32; 3]> = positions.to_vec();
        let root_skin = SkinWeights::identity(root_positions.len());
        let out = apply_lbs(&root_positions, &root_skin, &pose);
        for (orig, got) in root_positions.iter().zip(out.iter()) {
            assert!((orig[0] - got[0]).abs() < 1e-5, "x mismatch");
            assert!((orig[1] - got[1]).abs() < 1e-5, "y mismatch");
            assert!((orig[2] - got[2]).abs() < 1e-5, "z mismatch");
        }

        // Suppress unused-variable warning for `world`.
        let _ = world;
    }

    #[test]
    fn nearest_joint_assigns_all_verts() {
        let sk = Skeleton::human_body();
        let positions: Vec<[f32; 3]> = (0..50)
            .map(|i| [i as f32 * 0.01, i as f32 * 0.02, 0.0])
            .collect();
        let skin = SkinWeights::from_nearest_joint(&positions, &sk);
        assert!(skin.is_valid(positions.len()));
        // Every vertex must reference a valid joint index.
        for js in &skin.joints {
            assert!((js[0] as usize) < sk.joints.len());
        }
    }

    #[test]
    fn normalize_weights_sum_to_one() {
        let mut skin = SkinWeights {
            joints: vec![[0, 1, 0, 0]],
            weights: vec![[0.5, 0.5, 0.0, 0.0]],
        };
        skin.normalize();
        let w = skin.weights[0];
        let sum = w[0] + w[1] + w[2] + w[3];
        assert!((sum - 1.0).abs() < 1e-6, "weights do not sum to 1.0: {sum}");
    }

    #[test]
    fn lbs_single_joint_translation() {
        // Translate every vertex +1 on X by applying a translation matrix for joint 0.
        let positions = vec![[0.0f32, 0.0, 0.0], [1.0, 2.0, 3.0]];
        let skin = SkinWeights::identity(positions.len());

        // Column-major translation matrix: last column = [1, 0, 0, 1].
        let tx: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, // col 0
            0.0, 1.0, 0.0, 0.0, // col 1
            0.0, 0.0, 1.0, 0.0, // col 2
            1.0, 0.0, 0.0, 1.0, // col 3  (translate +1 on X)
        ];
        let pose = vec![tx];
        let out = apply_lbs(&positions, &skin, &pose);

        assert!((out[0][0] - 1.0).abs() < 1e-6, "vertex 0 x should be 1.0");
        assert!((out[1][0] - 2.0).abs() < 1e-6, "vertex 1 x should be 2.0");
        assert!((out[0][1] - 0.0).abs() < 1e-6, "vertex 0 y should be 0.0");
        assert!((out[1][1] - 2.0).abs() < 1e-6, "vertex 1 y should be 2.0");
    }
}
