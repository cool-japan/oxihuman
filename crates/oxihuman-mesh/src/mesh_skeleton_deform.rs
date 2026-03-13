// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// A bone transform: position + a simple 3x3 rotation matrix (row-major).
#[allow(dead_code)]
#[derive(Clone)]
pub struct BoneTransformSd {
    pub position: [f32; 3],
    pub rotation: [[f32; 3]; 3],
}

impl BoneTransformSd {
    /// Identity transform.
    #[allow(dead_code)]
    pub fn identity() -> Self {
        Self {
            position: [0.0; 3],
            rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }
}

/// A skeleton bone.
#[allow(dead_code)]
pub struct BoneSd {
    pub name: String,
    pub parent: Option<usize>,
    pub bind: BoneTransformSd,
    pub current: BoneTransformSd,
}

/// Vertex skin weight entry.
#[allow(dead_code)]
#[derive(Clone)]
pub struct SkeletonWeight {
    pub bone: usize,
    pub weight: f32,
}

/// Apply skeleton deformation using linear blend skinning.
#[allow(dead_code)]
pub fn skeleton_deform(
    positions: &[[f32; 3]],
    weights: &[Vec<SkeletonWeight>],
    bones: &[BoneSd],
) -> Vec<[f32; 3]> {
    positions
        .iter()
        .enumerate()
        .map(|(vi, &p)| {
            let ws = &weights[vi];
            if ws.is_empty() {
                return p;
            }
            let mut out = [0.0_f32; 3];
            for sw in ws {
                if sw.bone >= bones.len() {
                    continue;
                }
                let bt = &bones[sw.bone].current;
                let rotated = mat3_mul_vec(bt.rotation, p);
                out[0] += sw.weight * (rotated[0] + bt.position[0]);
                out[1] += sw.weight * (rotated[1] + bt.position[1]);
                out[2] += sw.weight * (rotated[2] + bt.position[2]);
            }
            out
        })
        .collect()
}

fn mat3_mul_vec(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// Normalize skin weights per vertex.
#[allow(dead_code)]
pub fn normalize_skeleton_weights(weights: &mut [Vec<SkeletonWeight>]) {
    for ws in weights.iter_mut() {
        let sum: f32 = ws.iter().map(|w| w.weight).sum();
        if sum > 1e-9 {
            for w in ws.iter_mut() {
                w.weight /= sum;
            }
        }
    }
}

/// Count vertices with any skin weight.
#[allow(dead_code)]
pub fn skinned_vertex_count_sd(weights: &[Vec<SkeletonWeight>]) -> usize {
    weights.iter().filter(|ws| !ws.is_empty()).count()
}

/// Serialize deform info to JSON.
#[allow(dead_code)]
pub fn skeleton_deform_to_json(n_verts: usize, n_bones: usize) -> String {
    format!(r#"{{"vertices":{},"bones":{}}}"#, n_verts, n_bones)
}

/// Compute average displacement between original and deformed.
#[allow(dead_code)]
pub fn deform_avg_displacement(original: &[[f32; 3]], deformed: &[[f32; 3]]) -> f32 {
    if original.is_empty() {
        return 0.0;
    }
    let sum: f32 = original
        .iter()
        .zip(deformed.iter())
        .map(|(a, b)| {
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            let dz = a[2] - b[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum();
    sum / original.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_bone() -> BoneSd {
        BoneSd {
            name: "root".to_string(),
            parent: None,
            bind: BoneTransformSd::identity(),
            current: BoneTransformSd::identity(),
        }
    }

    #[test]
    fn identity_no_displacement() {
        let pos = vec![[1.0_f32, 2.0, 3.0]];
        let weights = vec![vec![SkeletonWeight {
            bone: 0,
            weight: 1.0,
        }]];
        let bones = vec![identity_bone()];
        let deformed = skeleton_deform(&pos, &weights, &bones);
        assert!((deformed[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn no_weight_no_change() {
        let pos = vec![[1.0_f32, 2.0, 3.0]];
        let weights = vec![vec![]];
        let bones = vec![identity_bone()];
        let deformed = skeleton_deform(&pos, &weights, &bones);
        assert_eq!(deformed[0], pos[0]);
    }

    #[test]
    fn normalize_weights_sum_one() {
        let mut weights = vec![vec![
            SkeletonWeight {
                bone: 0,
                weight: 2.0,
            },
            SkeletonWeight {
                bone: 1,
                weight: 2.0,
            },
        ]];
        normalize_skeleton_weights(&mut weights);
        let sum: f32 = weights[0].iter().map(|w| w.weight).sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn skinned_count() {
        let weights = vec![
            vec![SkeletonWeight {
                bone: 0,
                weight: 1.0,
            }],
            vec![],
        ];
        assert_eq!(skinned_vertex_count_sd(&weights), 1);
    }

    #[test]
    fn json_has_bones() {
        let j = skeleton_deform_to_json(10, 3);
        assert!(j.contains("\"bones\":3"));
    }

    #[test]
    fn avg_displacement_identity() {
        let pos = vec![[0.0_f32; 3]; 3];
        assert!((deform_avg_displacement(&pos, &pos) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn translate_bone() {
        let mut bone = identity_bone();
        bone.current.position = [1.0, 0.0, 0.0];
        let pos = vec![[0.0_f32; 3]];
        let weights = vec![vec![SkeletonWeight {
            bone: 0,
            weight: 1.0,
        }]];
        let bones = vec![bone];
        let deformed = skeleton_deform(&pos, &weights, &bones);
        assert!((deformed[0][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn identity_transform() {
        let t = BoneTransformSd::identity();
        assert_eq!(t.position, [0.0; 3]);
    }

    #[test]
    fn empty_positions() {
        let deformed = skeleton_deform(&[], &[], &[]);
        assert!(deformed.is_empty());
    }

    #[test]
    fn avg_displacement_nonzero() {
        let a = vec![[0.0_f32; 3]];
        let b = vec![[1.0_f32, 0.0, 0.0]];
        assert!((deform_avg_displacement(&a, &b) - 1.0).abs() < 1e-5);
    }
}
