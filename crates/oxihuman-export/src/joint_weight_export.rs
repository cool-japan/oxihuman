// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Joint weight export: per-vertex joint influence weights.

/// A single joint influence on a vertex.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointInfluence {
    pub joint_index: u32,
    pub weight: f32,
}

/// Per-vertex joint weight data.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointWeightExport {
    pub vertex_influences: Vec<Vec<JointInfluence>>,
    pub joint_count: u32,
}

/// Create a new joint weight export.
#[allow(dead_code)]
pub fn new_joint_weight_export(vertex_count: usize, joint_count: u32) -> JointWeightExport {
    JointWeightExport {
        vertex_influences: vec![Vec::new(); vertex_count],
        joint_count,
    }
}

/// Add a joint influence to a vertex.
#[allow(dead_code)]
pub fn add_joint_influence(jw: &mut JointWeightExport, vertex: usize, joint: u32, weight: f32) {
    if vertex < jw.vertex_influences.len() {
        jw.vertex_influences[vertex].push(JointInfluence {
            joint_index: joint,
            weight,
        });
    }
}

/// Vertex count.
#[allow(dead_code)]
pub fn jw_vertex_count(jw: &JointWeightExport) -> usize {
    jw.vertex_influences.len()
}

/// Max influences per vertex.
#[allow(dead_code)]
pub fn jw_max_influences(jw: &JointWeightExport) -> usize {
    jw.vertex_influences
        .iter()
        .map(|v| v.len())
        .max()
        .unwrap_or(0)
}

/// Normalize weights per vertex to sum to 1.
#[allow(dead_code)]
pub fn normalize_joint_weights(jw: &mut JointWeightExport) {
    for infl in &mut jw.vertex_influences {
        let sum: f32 = infl.iter().map(|i| i.weight).sum();
        if sum > 1e-12 {
            for i in infl {
                i.weight /= sum;
            }
        }
    }
}

/// Count vertices with at least one influence.
#[allow(dead_code)]
pub fn skinned_vertex_count(jw: &JointWeightExport) -> usize {
    jw.vertex_influences
        .iter()
        .filter(|v| !v.is_empty())
        .count()
}

/// Validate: all weights non-negative and all joint indices < joint_count.
#[allow(dead_code)]
pub fn validate_joint_weights(jw: &JointWeightExport) -> bool {
    jw.vertex_influences.iter().all(|infl| {
        infl.iter()
            .all(|i| i.weight >= 0.0 && i.joint_index < jw.joint_count)
    })
}

/// Export to JSON.
#[allow(dead_code)]
pub fn joint_weight_to_json(jw: &JointWeightExport) -> String {
    format!(
        "{{\"vertex_count\":{},\"joint_count\":{},\"skinned_vertices\":{}}}",
        jw_vertex_count(jw),
        jw.joint_count,
        skinned_vertex_count(jw)
    )
}

/// Export to flat arrays (joints and weights for up to max_inf influences).
#[allow(dead_code)]
pub fn to_flat_arrays(jw: &JointWeightExport, max_inf: usize) -> (Vec<u32>, Vec<f32>) {
    let n = jw_vertex_count(jw);
    let mut joints = vec![0u32; n * max_inf];
    let mut weights = vec![0.0f32; n * max_inf];
    for (v, infl) in jw.vertex_influences.iter().enumerate() {
        for (k, ji) in infl.iter().take(max_inf).enumerate() {
            joints[v * max_inf + k] = ji.joint_index;
            weights[v * max_inf + k] = ji.weight;
        }
    }
    (joints, weights)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_vertex_jw() -> JointWeightExport {
        let mut jw = new_joint_weight_export(2, 4);
        add_joint_influence(&mut jw, 0, 0, 0.6);
        add_joint_influence(&mut jw, 0, 1, 0.4);
        add_joint_influence(&mut jw, 1, 2, 1.0);
        jw
    }

    #[test]
    fn test_new_joint_weight_export() {
        let jw = new_joint_weight_export(5, 10);
        assert_eq!(jw_vertex_count(&jw), 5);
    }

    #[test]
    fn test_add_joint_influence() {
        let jw = two_vertex_jw();
        assert_eq!(jw.vertex_influences[0].len(), 2);
    }

    #[test]
    fn test_jw_max_influences() {
        let jw = two_vertex_jw();
        assert_eq!(jw_max_influences(&jw), 2);
    }

    #[test]
    fn test_normalize_joint_weights() {
        let mut jw = two_vertex_jw();
        normalize_joint_weights(&mut jw);
        let sum: f32 = jw.vertex_influences[0].iter().map(|i| i.weight).sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_skinned_vertex_count() {
        let jw = two_vertex_jw();
        assert_eq!(skinned_vertex_count(&jw), 2);
    }

    #[test]
    fn test_validate_joint_weights() {
        let jw = two_vertex_jw();
        assert!(validate_joint_weights(&jw));
    }

    #[test]
    fn test_validate_invalid_joint_index() {
        let mut jw = new_joint_weight_export(1, 2);
        add_joint_influence(&mut jw, 0, 99, 1.0);
        assert!(!validate_joint_weights(&jw));
    }

    #[test]
    fn test_joint_weight_to_json() {
        let jw = two_vertex_jw();
        let j = joint_weight_to_json(&jw);
        assert!(j.contains("\"vertex_count\":2"));
    }

    #[test]
    fn test_to_flat_arrays() {
        let jw = two_vertex_jw();
        let (joints, weights) = to_flat_arrays(&jw, 2);
        assert_eq!(joints.len(), 4);
        assert!((weights[0] - 0.6).abs() < 1e-5);
    }

    #[test]
    fn test_weight_in_range() {
        let mut jw = two_vertex_jw();
        normalize_joint_weights(&mut jw);
        for infl in &jw.vertex_influences {
            for i in infl {
                assert!((0.0..=1.0).contains(&i.weight));
            }
        }
    }
}
