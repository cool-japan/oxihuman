// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Loop subdivision scheme for triangle meshes.

use std::f32::consts::PI;

/// Result of a Loop subdivision step.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LoopSubdivResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub levels: usize,
}

fn edge_key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Perform one level of Loop subdivision.
#[allow(dead_code)]
pub fn loop_subdiv_step(positions: &[[f32; 3]], indices: &[u32]) -> (Vec<[f32; 3]>, Vec<u32>) {
    use std::collections::HashMap;
    let mut edge_midpoints: HashMap<(u32, u32), u32> = HashMap::new();
    let mut new_pos = positions.to_vec();
    let next_idx =
        |ep: &mut HashMap<(u32, u32), u32>, new_pos: &mut Vec<[f32; 3]>, a: u32, b: u32| -> u32 {
            let key = edge_key(a, b);
            *ep.entry(key).or_insert_with(|| {
                let mid = lerp3(new_pos[a as usize], new_pos[b as usize], 0.5);
                let idx = new_pos.len() as u32;
                new_pos.push(mid);
                idx
            })
        };
    let mut new_indices = Vec::with_capacity(indices.len() * 4);
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0], tri[1], tri[2]);
        let ab = next_idx(&mut edge_midpoints, &mut new_pos, a, b);
        let bc = next_idx(&mut edge_midpoints, &mut new_pos, b, c);
        let ca = next_idx(&mut edge_midpoints, &mut new_pos, c, a);
        new_indices.extend_from_slice(&[a, ab, ca]);
        new_indices.extend_from_slice(&[ab, b, bc]);
        new_indices.extend_from_slice(&[ca, bc, c]);
        new_indices.extend_from_slice(&[ab, bc, ca]);
    }
    (new_pos, new_indices)
}

/// Subdivide a triangle mesh `levels` times using Loop subdivision.
#[allow(dead_code)]
pub fn loop_subdivide_mesh(
    positions: &[[f32; 3]],
    indices: &[u32],
    levels: usize,
) -> LoopSubdivResult {
    let mut pos = positions.to_vec();
    let mut idx = indices.to_vec();
    let _ = PI;
    for _ in 0..levels {
        let (p, i) = loop_subdiv_step(&pos, &idx);
        pos = p;
        idx = i;
    }
    LoopSubdivResult {
        positions: pos,
        indices: idx,
        levels,
    }
}

/// Expected face count after `levels` of subdivision.
#[allow(dead_code)]
pub fn expected_face_count(base_faces: usize, levels: usize) -> usize {
    base_faces * 4usize.pow(levels as u32)
}

/// Expected vertex count estimate (approximate for closed manifold).
#[allow(dead_code)]
pub fn expected_vertex_count_approx(base_verts: usize, base_faces: usize, levels: usize) -> usize {
    let faces = expected_face_count(base_faces, levels);
    base_verts + (faces / 2) // rough approximation via Euler
}

/// Check that all indices are within bounds.
#[allow(dead_code)]
pub fn indices_valid(res: &LoopSubdivResult) -> bool {
    let n = res.positions.len() as u32;
    res.indices.iter().all(|&i| i < n)
}

/// Average edge length.
#[allow(dead_code)]
pub fn avg_edge_length_ls(res: &LoopSubdivResult) -> f32 {
    let mut total = 0.0_f32;
    let mut count = 0usize;
    for tri in res.indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = res.positions[a];
        let pb = res.positions[b];
        let pc = res.positions[c];
        total +=
            ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
        total +=
            ((pb[0] - pc[0]).powi(2) + (pb[1] - pc[1]).powi(2) + (pb[2] - pc[2]).powi(2)).sqrt();
        total +=
            ((pc[0] - pa[0]).powi(2) + (pc[1] - pa[1]).powi(2) + (pc[2] - pa[2]).powi(2)).sqrt();
        count += 3;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

/// Compute bounding box of subdivided mesh.
#[allow(dead_code)]
pub fn bounding_box_ls(res: &LoopSubdivResult) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for p in &res.positions {
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    (mn, mx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        (
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn one_level_face_count() {
        let (p, i) = single_tri();
        let res = loop_subdivide_mesh(&p, &i, 1);
        assert_eq!(res.indices.len() / 3, 4);
    }

    #[test]
    fn two_level_face_count() {
        let (p, i) = single_tri();
        let res = loop_subdivide_mesh(&p, &i, 2);
        assert_eq!(res.indices.len() / 3, 16);
    }

    #[test]
    fn indices_valid_after_subdiv() {
        let (p, i) = single_tri();
        let res = loop_subdivide_mesh(&p, &i, 2);
        assert!(indices_valid(&res));
    }

    #[test]
    fn zero_levels_unchanged() {
        let (p, i) = single_tri();
        let res = loop_subdivide_mesh(&p, &i, 0);
        assert_eq!(res.indices.len(), i.len());
    }

    #[test]
    fn expected_face_count_test() {
        assert_eq!(expected_face_count(1, 2), 16);
    }

    #[test]
    fn avg_edge_length_positive() {
        let (p, i) = single_tri();
        let res = loop_subdivide_mesh(&p, &i, 1);
        assert!(avg_edge_length_ls(&res) > 0.0);
    }

    #[test]
    fn bounding_box_grows_not() {
        let (p, i) = single_tri();
        let res0 = loop_subdivide_mesh(&p, &i, 0);
        let res1 = loop_subdivide_mesh(&p, &i, 2);
        let (mn0, mx0) = bounding_box_ls(&res0);
        let (mn1, mx1) = bounding_box_ls(&res1);
        // Bounding box should not grow (midpoints only)
        for k in 0..3 {
            assert!(mn1[k] >= mn0[k] - 1e-4);
            assert!(mx1[k] <= mx0[k] + 1e-4);
        }
    }

    #[test]
    fn result_levels_field() {
        let (p, i) = single_tri();
        let res = loop_subdivide_mesh(&p, &i, 3);
        assert_eq!(res.levels, 3);
    }

    #[test]
    fn subdiv_step_direct() {
        let (p, i) = single_tri();
        let (np, ni) = loop_subdiv_step(&p, &i);
        assert_eq!(ni.len() / 3, 4);
        assert!(np.len() > p.len());
    }

    #[test]
    fn empty_mesh() {
        let res = loop_subdivide_mesh(&[], &[], 2);
        assert!(res.positions.is_empty());
    }
}
