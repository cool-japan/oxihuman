// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Rip-vertex operation: duplicate a vertex and separate connected faces.

/// Result of the rip-vertex operation.
#[derive(Debug, Clone)]
pub struct RipVertexResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    /// Index of the newly inserted duplicate vertex.
    pub new_vertex_index: u32,
    pub affected_faces: usize,
}

/// Returns all face indices that use the given vertex.
pub fn faces_using_vertex(indices: &[u32], vertex: u32) -> Vec<usize> {
    let n = indices.len() / 3;
    let mut out = Vec::new();
    for i in 0..n {
        if indices[i * 3] == vertex || indices[i * 3 + 1] == vertex || indices[i * 3 + 2] == vertex
        {
            out.push(i);
        }
    }
    out
}

/// Rips the vertex by duplicating it and reassigning a subset of its faces to
/// the new duplicate.
///
/// `faces_to_rip` — face indices (into the triangle list) that will use the
/// new duplicate vertex instead of the original.
pub fn rip_vertex(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex: u32,
    faces_to_rip: &[usize],
) -> Option<RipVertexResult> {
    let vi = vertex as usize;
    if vi >= positions.len() {
        return None;
    }
    let rip_set: std::collections::HashSet<usize> = faces_to_rip.iter().copied().collect();
    /* duplicate the vertex */
    let new_vertex = positions.len() as u32;
    let mut new_positions = positions.to_vec();
    new_positions.push(positions[vi]);
    let n = indices.len() / 3;
    let mut new_indices = Vec::with_capacity(indices.len());
    let mut affected = 0usize;
    for i in 0..n {
        let mut ia = indices[i * 3];
        let mut ib = indices[i * 3 + 1];
        let mut ic = indices[i * 3 + 2];
        if rip_set.contains(&i) {
            if ia == vertex {
                ia = new_vertex;
                affected += 1;
            }
            if ib == vertex {
                ib = new_vertex;
                affected += 1;
            }
            if ic == vertex {
                ic = new_vertex;
                affected += 1;
            }
        }
        new_indices.push(ia);
        new_indices.push(ib);
        new_indices.push(ic);
    }
    Some(RipVertexResult {
        new_positions,
        new_indices,
        new_vertex_index: new_vertex,
        affected_faces: affected,
    })
}

/// Rips all faces at a vertex, creating a fully separated duplicate.
pub fn rip_all_faces(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex: u32,
) -> Option<RipVertexResult> {
    let all_faces = faces_using_vertex(indices, vertex);
    if all_faces.is_empty() {
        return None;
    }
    rip_vertex(positions, indices, vertex, &all_faces)
}

/// Computes the average position of all faces adjacent to a vertex.
pub fn vertex_neighborhood_centroid(
    positions: &[[f32; 3]],
    indices: &[u32],
    vertex: u32,
) -> [f32; 3] {
    let faces = faces_using_vertex(indices, vertex);
    if faces.is_empty() {
        return if (vertex as usize) < positions.len() {
            positions[vertex as usize]
        } else {
            [0.0; 3]
        };
    }
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    let mut count = 0usize;
    for fi in faces {
        for k in 0..3 {
            let vi = indices[fi * 3 + k] as usize;
            if vi < positions.len() {
                cx += positions[vi][0];
                cy += positions[vi][1];
                cz += positions[vi][2];
                count += 1;
            }
        }
    }
    if count == 0 {
        return [0.0; 3];
    }
    [cx / count as f32, cy / count as f32, cz / count as f32]
}

/// Returns true if the vertex is a boundary vertex (has an edge shared by only one face).
pub fn is_boundary_vertex_rv(indices: &[u32], vertex: u32, vertex_count: usize) -> bool {
    if (vertex as usize) >= vertex_count {
        return false;
    }
    let n = indices.len() / 3;
    let mut edge_count: std::collections::HashMap<(u32, u32), usize> =
        std::collections::HashMap::new();
    for i in 0..n {
        let ia = indices[i * 3];
        let ib = indices[i * 3 + 1];
        let ic = indices[i * 3 + 2];
        for (a, b) in [(ia, ib), (ib, ic), (ic, ia)] {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count
        .iter()
        .any(|((a, b), &c)| c == 1 && (*a == vertex || *b == vertex))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        /* both tris share vertex 0 */
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        (pos, idx)
    }

    #[test]
    fn faces_using_vertex_count() {
        let (_, idx) = two_tris();
        let faces = faces_using_vertex(&idx, 0);
        assert_eq!(faces.len(), 2);
    }

    #[test]
    fn rip_vertex_creates_new_vertex() {
        let (pos, idx) = two_tris();
        let res = rip_vertex(&pos, &idx, 0, &[0]).unwrap();
        assert_eq!(res.new_positions.len(), pos.len() + 1);
    }

    #[test]
    fn rip_all_faces_separates_vertex() {
        let (pos, idx) = two_tris();
        let res = rip_all_faces(&pos, &idx, 0).unwrap();
        /* new vertex duplicated */
        assert_eq!(res.new_vertex_index, 4);
    }

    #[test]
    fn rip_out_of_bounds_returns_none() {
        let (pos, idx) = two_tris();
        let res = rip_vertex(&pos, &idx, 99, &[0]);
        assert!(res.is_none());
    }

    #[test]
    fn vertex_neighborhood_centroid_finite() {
        let (pos, idx) = two_tris();
        let c = vertex_neighborhood_centroid(&pos, &idx, 0);
        for v in c {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn is_boundary_vertex_out_of_bounds() {
        let (_, idx) = two_tris();
        assert!(!is_boundary_vertex_rv(&idx, 99, 4));
    }

    #[test]
    fn rip_vertex_affected_faces() {
        let (pos, idx) = two_tris();
        let res = rip_vertex(&pos, &idx, 0, &[0]).unwrap();
        assert!(res.affected_faces > 0);
    }

    #[test]
    fn faces_using_no_vertex_empty() {
        let (_, idx) = two_tris();
        let faces = faces_using_vertex(&idx, 9);
        assert!(faces.is_empty());
    }

    #[test]
    fn rip_all_no_faces_returns_none() {
        let (pos, _) = two_tris();
        let idx: Vec<u32> = vec![];
        let res = rip_all_faces(&pos, &idx, 0);
        assert!(res.is_none());
    }
}
