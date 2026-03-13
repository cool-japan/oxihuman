// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Adaptive remeshing: refine faces above a size threshold.

#[allow(dead_code)]
pub struct AdaptiveRemesherV2 {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<[u32; 3]>,
    pub max_edge_len: f32,
}

#[allow(dead_code)]
pub fn new_adaptive_remesher_v2(
    positions: Vec<[f32; 3]>,
    indices: Vec<[u32; 3]>,
    max_edge_len: f32,
) -> AdaptiveRemesherV2 {
    AdaptiveRemesherV2 { positions, indices, max_edge_len }
}

#[allow(dead_code)]
pub fn ar_face_count_v2(r: &AdaptiveRemesherV2) -> usize { r.indices.len() }

#[allow(dead_code)]
pub fn ar_vertex_count_v2(r: &AdaptiveRemesherV2) -> usize { r.positions.len() }

#[allow(dead_code)]
pub fn ar_max_edge_len_v2(r: &AdaptiveRemesherV2) -> f32 { r.max_edge_len }

fn edge_len(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn ar_needs_refinement_v2(r: &AdaptiveRemesherV2) -> bool {
    for tri in &r.indices {
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        if a < r.positions.len() && b < r.positions.len() && c < r.positions.len()
            && (edge_len(r.positions[a], r.positions[b]) > r.max_edge_len
                || edge_len(r.positions[b], r.positions[c]) > r.max_edge_len
                || edge_len(r.positions[c], r.positions[a]) > r.max_edge_len)
        {
            return true;
        }
    }
    false
}

#[allow(dead_code)]
pub fn ar_refine_step_v2(_r: &mut AdaptiveRemesherV2) {
    /* stub: mark as refined */
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_mesh() -> AdaptiveRemesherV2 {
        new_adaptive_remesher_v2(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]],
            vec![[0, 1, 2]],
            1.0,
        )
    }

    #[test]
    fn test_face_count() { assert_eq!(ar_face_count_v2(&tiny_mesh()), 1); }

    #[test]
    fn test_vertex_count() { assert_eq!(ar_vertex_count_v2(&tiny_mesh()), 3); }

    #[test]
    fn test_max_edge_len_getter() {
        assert!((ar_max_edge_len_v2(&tiny_mesh()) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_needs_refinement_false_for_tiny() {
        assert!(!ar_needs_refinement_v2(&tiny_mesh()));
    }

    #[test]
    fn test_needs_refinement_true_for_large() {
        let r = new_adaptive_remesher_v2(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
            1.0,
        );
        assert!(ar_needs_refinement_v2(&r));
    }

    #[test]
    fn test_refine_step_no_crash() {
        let mut r = tiny_mesh();
        ar_refine_step_v2(&mut r);
        assert_eq!(ar_face_count_v2(&r), 1);
    }

    #[test]
    fn test_empty_mesh_no_refinement() {
        let r = new_adaptive_remesher_v2(vec![], vec![], 1.0);
        assert!(!ar_needs_refinement_v2(&r));
    }

    #[test]
    fn test_initial_state() {
        let r = new_adaptive_remesher_v2(vec![[0.0, 0.0, 0.0]], vec![], 0.5);
        assert_eq!(ar_vertex_count_v2(&r), 1);
        assert_eq!(ar_face_count_v2(&r), 0);
    }
}
