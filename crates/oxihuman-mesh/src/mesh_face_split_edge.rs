// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Split a face by inserting a new edge between two of its vertices.

/// Result of a face-split-by-edge operation.
#[derive(Debug, Clone)]
pub struct FaceSplitEdgeResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub split_face_index: usize,
    pub inserted_midpoint: Option<u32>,
}

/// Linearly interpolates between two 3-D positions.
pub fn lerp3_fse(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Finds the face that contains the directed edge `(v0, v1)`.
/// Returns the face index and the third vertex index.
pub fn find_face_with_edge(indices: &[u32], v0: u32, v1: u32) -> Option<(usize, u32)> {
    let n = indices.len() / 3;
    for i in 0..n {
        let ia = indices[i * 3];
        let ib = indices[i * 3 + 1];
        let ic = indices[i * 3 + 2];
        /* check all three directed edges */
        for (ea, eb, ec) in [(ia, ib, ic), (ib, ic, ia), (ic, ia, ib)] {
            if ea == v0 && eb == v1 {
                return Some((i, ec));
            }
        }
    }
    None
}

/// Splits the face containing edge `(v0, v1)` by inserting a midpoint.
/// Returns updated position and index buffers.
pub fn split_face_by_edge_midpoint(
    positions: &[[f32; 3]],
    indices: &[u32],
    v0: u32,
    v1: u32,
) -> Option<FaceSplitEdgeResult> {
    let (face_idx, v2) = find_face_with_edge(indices, v0, v1)?;
    let n = indices.len() / 3;
    let mid_pos = lerp3_fse(positions[v0 as usize], positions[v1 as usize], 0.5);
    let mid_idx = positions.len() as u32;
    let mut new_positions = positions.to_vec();
    new_positions.push(mid_pos);
    /* replace old face with two new faces */
    let mut new_indices = Vec::with_capacity(indices.len() + 3);
    for i in 0..n {
        if i == face_idx {
            /* face (v0, v1, v2) becomes (v0, mid, v2) and (mid, v1, v2) */
            new_indices.extend_from_slice(&[v0, mid_idx, v2]);
            new_indices.extend_from_slice(&[mid_idx, v1, v2]);
        } else {
            new_indices.push(indices[i * 3]);
            new_indices.push(indices[i * 3 + 1]);
            new_indices.push(indices[i * 3 + 2]);
        }
    }
    Some(FaceSplitEdgeResult {
        new_positions,
        new_indices,
        split_face_index: face_idx,
        inserted_midpoint: Some(mid_idx),
    })
}

/// Counts faces sharing the given edge.
pub fn faces_sharing_edge(indices: &[u32], v0: u32, v1: u32) -> usize {
    let n = indices.len() / 3;
    let mut count = 0usize;
    for i in 0..n {
        let ia = indices[i * 3];
        let ib = indices[i * 3 + 1];
        let ic = indices[i * 3 + 2];
        let has_v0 = ia == v0 || ib == v0 || ic == v0;
        let has_v1 = ia == v1 || ib == v1 || ic == v1;
        if has_v0 && has_v1 {
            count += 1;
        }
    }
    count
}

/// Checks whether the given undirected edge exists in the mesh.
pub fn edge_exists_fse(indices: &[u32], v0: u32, v1: u32) -> bool {
    faces_sharing_edge(indices, v0, v1) > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    #[test]
    fn lerp_midpoint() {
        let mid = lerp3_fse([0.0, 0.0, 0.0], [2.0, 0.0, 0.0], 0.5);
        assert!((mid[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn find_face_with_edge_found() {
        let (_, idx) = one_tri();
        let res = find_face_with_edge(&idx, 0, 1);
        assert!(res.is_some());
    }

    #[test]
    fn find_face_with_edge_missing() {
        let (_, idx) = one_tri();
        let res = find_face_with_edge(&idx, 9, 10);
        assert!(res.is_none());
    }

    #[test]
    fn split_increases_face_count() {
        let (pos, idx) = one_tri();
        let res = split_face_by_edge_midpoint(&pos, &idx, 0, 1).unwrap();
        assert_eq!(res.new_indices.len() / 3, 2);
    }

    #[test]
    fn split_adds_one_vertex() {
        let (pos, idx) = one_tri();
        let res = split_face_by_edge_midpoint(&pos, &idx, 0, 1).unwrap();
        assert_eq!(res.new_positions.len(), pos.len() + 1);
    }

    #[test]
    fn midpoint_inserted_index() {
        let (pos, idx) = one_tri();
        let res = split_face_by_edge_midpoint(&pos, &idx, 0, 1).unwrap();
        assert_eq!(res.inserted_midpoint, Some(3));
    }

    #[test]
    fn faces_sharing_edge_one() {
        let (_, idx) = one_tri();
        assert_eq!(faces_sharing_edge(&idx, 0, 1), 1);
    }

    #[test]
    fn edge_exists_true() {
        let (_, idx) = one_tri();
        assert!(edge_exists_fse(&idx, 1, 2));
    }

    #[test]
    fn edge_exists_false() {
        let (_, idx) = one_tri();
        assert!(!edge_exists_fse(&idx, 5, 6));
    }
}
