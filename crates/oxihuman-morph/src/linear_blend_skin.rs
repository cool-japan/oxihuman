// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Linear blend skinning (LBS) stub.

/// A skinning influence (bone index + weight).
#[derive(Debug, Clone, Copy)]
pub struct SkinInfluence {
    pub bone_index: usize,
    pub weight: f32,
}

/// LBS vertex binding — up to 4 influences per vertex.
#[derive(Debug, Clone)]
pub struct LbsVertex {
    pub influences: Vec<SkinInfluence>,
}

impl LbsVertex {
    pub fn new() -> Self {
        LbsVertex {
            influences: Vec::new(),
        }
    }
}

impl Default for LbsVertex {
    fn default() -> Self {
        LbsVertex::new()
    }
}

/// Linear blend skin mesh.
#[derive(Debug, Clone)]
pub struct LinearBlendSkin {
    pub vertices: Vec<LbsVertex>,
    pub bone_count: usize,
}

impl LinearBlendSkin {
    pub fn new(vertex_count: usize, bone_count: usize) -> Self {
        LinearBlendSkin {
            vertices: (0..vertex_count).map(|_| LbsVertex::default()).collect(),
            bone_count,
        }
    }
}

/// Create a new LBS mesh.
pub fn new_lbs(vertex_count: usize, bone_count: usize) -> LinearBlendSkin {
    LinearBlendSkin::new(vertex_count, bone_count)
}

/// Add an influence to a vertex.
pub fn lbs_add_influence(lbs: &mut LinearBlendSkin, vertex: usize, bone: usize, weight: f32) {
    if vertex < lbs.vertices.len() && bone < lbs.bone_count {
        lbs.vertices[vertex].influences.push(SkinInfluence {
            bone_index: bone,
            weight,
        });
    }
}

/// Normalize influence weights so they sum to 1.0 for each vertex.
pub fn lbs_normalize(lbs: &mut LinearBlendSkin) {
    for v in &mut lbs.vertices {
        let sum: f32 = v.influences.iter().map(|i| i.weight).sum();
        if sum > 1e-9 {
            for inf in &mut v.influences {
                inf.weight /= sum;
            }
        }
    }
}

/// Return vertex count.
pub fn lbs_vertex_count(lbs: &LinearBlendSkin) -> usize {
    lbs.vertices.len()
}

/// Return influence count for a vertex.
pub fn lbs_influence_count(lbs: &LinearBlendSkin, vertex: usize) -> usize {
    if vertex < lbs.vertices.len() {
        lbs.vertices[vertex].influences.len()
    } else {
        0
    }
}

/// Return a JSON-like string.
pub fn lbs_to_json(lbs: &LinearBlendSkin) -> String {
    format!(
        r#"{{"vertices":{},"bones":{}}}"#,
        lbs.vertices.len(),
        lbs.bone_count
    )
}

/// Check if all vertex influence weights sum to approximately 1.0.
pub fn lbs_is_normalized(lbs: &LinearBlendSkin) -> bool {
    lbs.vertices
        .iter()
        .filter(|v| !v.influences.is_empty())
        .all(|v| {
            let s: f32 = v.influences.iter().map(|i| i.weight).sum();
            (s - 1.0).abs() < 1e-4
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_lbs_vertex_count() {
        let lbs = new_lbs(20, 5);
        assert_eq!(
            lbs_vertex_count(&lbs),
            20, /* vertex count must match */
        );
    }

    #[test]
    fn test_add_influence_increases_count() {
        let mut lbs = new_lbs(5, 3);
        lbs_add_influence(&mut lbs, 0, 0, 0.5);
        lbs_add_influence(&mut lbs, 0, 1, 0.5);
        assert_eq!(
            lbs_influence_count(&lbs, 0),
            2, /* two influences added */
        );
    }

    #[test]
    fn test_normalize_makes_sum_one() {
        let mut lbs = new_lbs(2, 3);
        lbs_add_influence(&mut lbs, 0, 0, 2.0);
        lbs_add_influence(&mut lbs, 0, 1, 2.0);
        lbs_normalize(&mut lbs);
        assert!(lbs_is_normalized(&lbs), /* normalized weights should sum to 1 */);
    }

    #[test]
    fn test_add_out_of_bounds_ignored() {
        let mut lbs = new_lbs(2, 3);
        lbs_add_influence(&mut lbs, 99, 0, 1.0);
        assert_eq!(
            lbs_influence_count(&lbs, 0),
            0, /* out-of-bounds vertex ignored */
        );
    }

    #[test]
    fn test_add_invalid_bone_ignored() {
        let mut lbs = new_lbs(2, 3);
        lbs_add_influence(&mut lbs, 0, 99, 1.0);
        assert_eq!(
            lbs_influence_count(&lbs, 0),
            0, /* out-of-bounds bone ignored */
        );
    }

    #[test]
    fn test_to_json_contains_bones() {
        let lbs = new_lbs(4, 6);
        let j = lbs_to_json(&lbs);
        assert!(j.contains("bones") /* JSON must contain bones */,);
    }

    #[test]
    fn test_empty_vertices_are_normalized() {
        let lbs = new_lbs(3, 2);
        assert!(lbs_is_normalized(&lbs), /* empty vertices trivially normalized */);
    }

    #[test]
    fn test_influence_count_out_of_bounds() {
        let lbs = new_lbs(2, 3);
        assert_eq!(
            lbs_influence_count(&lbs, 99),
            0, /* out-of-bounds returns 0 */
        );
    }

    #[test]
    fn test_bone_count_stored() {
        let lbs = new_lbs(5, 8);
        assert_eq!(lbs.bone_count, 8 /* bone count must match */,);
    }

    #[test]
    fn test_single_influence_is_normalized() {
        let mut lbs = new_lbs(1, 1);
        lbs_add_influence(&mut lbs, 0, 0, 1.0);
        assert!(lbs_is_normalized(&lbs), /* single influence should be normalized */);
    }
}
