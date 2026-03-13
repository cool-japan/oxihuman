// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Convert an edge loop into a face region selection.
//! All faces on one side of the loop boundary are selected.

use std::collections::{HashMap, HashSet, VecDeque};

/// Result of the loop-to-region conversion.
#[derive(Debug, Clone, Default)]
pub struct LoopToRegionResult {
    pub selected_faces: Vec<usize>,
    pub boundary_edges: usize,
    pub region_face_count: usize,
}

/// Builds a face adjacency graph: face index → neighbouring face indices.
pub fn face_adjacency_map(indices: &[u32]) -> HashMap<usize, Vec<usize>> {
    let n = indices.len() / 3;
    let mut edge_to_face: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for i in 0..n {
        let ia = indices[i * 3];
        let ib = indices[i * 3 + 1];
        let ic = indices[i * 3 + 2];
        for (a, b) in [(ia, ib), (ib, ic), (ic, ia)] {
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_face.entry(key).or_default().push(i);
        }
    }
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for faces in edge_to_face.values() {
        if faces.len() == 2 {
            let fa = faces[0];
            let fb = faces[1];
            adj.entry(fa).or_default().push(fb);
            adj.entry(fb).or_default().push(fa);
        }
    }
    adj
}

/// Checks whether a face contains any edge from the given loop edge set.
pub fn face_touches_loop(
    indices: &[u32],
    face_idx: usize,
    loop_edges: &HashSet<(u32, u32)>,
) -> bool {
    if face_idx * 3 + 2 >= indices.len() {
        return false;
    }
    let ia = indices[face_idx * 3];
    let ib = indices[face_idx * 3 + 1];
    let ic = indices[face_idx * 3 + 2];
    for (a, b) in [(ia, ib), (ib, ic), (ic, ia)] {
        let key = if a < b { (a, b) } else { (b, a) };
        if loop_edges.contains(&key) {
            return true;
        }
    }
    false
}

/// Expands a face region by BFS, stopping at loop edges.
pub fn flood_fill_region(
    indices: &[u32],
    start_face: usize,
    loop_edges: &HashSet<(u32, u32)>,
    adjacency: &HashMap<usize, Vec<usize>>,
) -> Vec<usize> {
    let n = indices.len() / 3;
    let mut visited: HashSet<usize> = HashSet::new();
    let mut queue: VecDeque<usize> = VecDeque::new();
    if start_face >= n {
        return vec![];
    }
    queue.push_back(start_face);
    visited.insert(start_face);
    while let Some(fi) = queue.pop_front() {
        if let Some(neighbours) = adjacency.get(&fi) {
            for &nf in neighbours {
                if visited.contains(&nf) {
                    continue;
                }
                /* don't cross loop boundary */
                let ia_fi = if fi * 3 + 2 < indices.len() {
                    Some((indices[fi * 3], indices[fi * 3 + 1], indices[fi * 3 + 2]))
                } else {
                    None
                };
                let ia_nf = if nf * 3 + 2 < indices.len() {
                    Some((indices[nf * 3], indices[nf * 3 + 1], indices[nf * 3 + 2]))
                } else {
                    None
                };
                let shared_edge_on_loop = if let (Some(fi_verts), Some(nf_verts)) = (ia_fi, ia_nf) {
                    /* find shared edge */
                    let fi_edges: Vec<(u32, u32)> = vec![
                        if fi_verts.0 < fi_verts.1 {
                            (fi_verts.0, fi_verts.1)
                        } else {
                            (fi_verts.1, fi_verts.0)
                        },
                        if fi_verts.1 < fi_verts.2 {
                            (fi_verts.1, fi_verts.2)
                        } else {
                            (fi_verts.2, fi_verts.1)
                        },
                        if fi_verts.2 < fi_verts.0 {
                            (fi_verts.2, fi_verts.0)
                        } else {
                            (fi_verts.0, fi_verts.2)
                        },
                    ];
                    let nf_edges: Vec<(u32, u32)> = vec![
                        if nf_verts.0 < nf_verts.1 {
                            (nf_verts.0, nf_verts.1)
                        } else {
                            (nf_verts.1, nf_verts.0)
                        },
                        if nf_verts.1 < nf_verts.2 {
                            (nf_verts.1, nf_verts.2)
                        } else {
                            (nf_verts.2, nf_verts.1)
                        },
                        if nf_verts.2 < nf_verts.0 {
                            (nf_verts.2, nf_verts.0)
                        } else {
                            (nf_verts.0, nf_verts.2)
                        },
                    ];
                    fi_edges
                        .iter()
                        .any(|e| nf_edges.contains(e) && loop_edges.contains(e))
                } else {
                    false
                };
                if !shared_edge_on_loop {
                    visited.insert(nf);
                    queue.push_back(nf);
                }
            }
        }
    }
    let mut result: Vec<usize> = visited.into_iter().collect();
    result.sort_unstable();
    result
}

/// Converts an edge loop (as vertex-pair list) into a face region.
pub fn loop_to_region(
    indices: &[u32],
    loop_edges: &[(u32, u32)],
    seed_face: usize,
) -> LoopToRegionResult {
    let edge_set: HashSet<(u32, u32)> = loop_edges
        .iter()
        .map(|&(a, b)| if a < b { (a, b) } else { (b, a) })
        .collect();
    let adjacency = face_adjacency_map(indices);
    let selected = flood_fill_region(indices, seed_face, &edge_set, &adjacency);
    let region_face_count = selected.len();
    LoopToRegionResult {
        selected_faces: selected,
        boundary_edges: loop_edges.len(),
        region_face_count,
    }
}

/// Returns unique vertex indices from a list of face indices.
pub fn vertices_from_faces(indices: &[u32], faces: &[usize]) -> Vec<u32> {
    let mut verts: HashSet<u32> = HashSet::new();
    for &fi in faces {
        if fi * 3 + 2 < indices.len() {
            verts.insert(indices[fi * 3]);
            verts.insert(indices[fi * 3 + 1]);
            verts.insert(indices[fi * 3 + 2]);
        }
    }
    let mut v: Vec<u32> = verts.into_iter().collect();
    v.sort_unstable();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_mesh() -> Vec<u32> {
        /* 2×2 grid, 4 triangles */
        vec![
            0u32, 1, 3, /* tri 0 */
            0, 3, 2, /* tri 1 */
            1, 4, 3, /* tri 2 */
            3, 4, 5, /* tri 3 */
        ]
    }

    #[test]
    fn face_adjacency_has_entries() {
        let idx = grid_mesh();
        let adj = face_adjacency_map(&idx);
        assert!(!adj.is_empty());
    }

    #[test]
    fn flood_fill_reaches_adjacent() {
        let idx = grid_mesh();
        let adj = face_adjacency_map(&idx);
        let region = flood_fill_region(&idx, 0, &HashSet::new(), &adj);
        assert!(region.len() > 1);
    }

    #[test]
    fn loop_to_region_returns_faces() {
        let idx = grid_mesh();
        let res = loop_to_region(&idx, &[], 0);
        assert!(!res.selected_faces.is_empty());
    }

    #[test]
    fn vertices_from_faces_non_empty() {
        let idx = grid_mesh();
        let verts = vertices_from_faces(&idx, &[0, 1]);
        assert!(!verts.is_empty());
    }

    #[test]
    fn face_touches_loop_false_empty() {
        let idx = grid_mesh();
        let loop_set: HashSet<(u32, u32)> = HashSet::new();
        assert!(!face_touches_loop(&idx, 0, &loop_set));
    }

    #[test]
    fn face_touches_loop_true() {
        let idx = grid_mesh();
        let mut loop_set = HashSet::new();
        loop_set.insert((0u32, 1u32));
        assert!(face_touches_loop(&idx, 0, &loop_set));
    }

    #[test]
    fn loop_to_region_empty_loop_selects_all() {
        let idx = grid_mesh();
        let res = loop_to_region(&idx, &[], 0);
        assert_eq!(res.selected_faces.len(), idx.len() / 3);
    }

    #[test]
    fn result_region_count_matches_selected() {
        let idx = grid_mesh();
        let res = loop_to_region(&idx, &[], 0);
        assert_eq!(res.region_face_count, res.selected_faces.len());
    }

    #[test]
    fn flood_fill_empty_mesh() {
        let idx: Vec<u32> = vec![];
        let adj = face_adjacency_map(&idx);
        let region = flood_fill_region(&idx, 0, &HashSet::new(), &adj);
        assert!(region.is_empty());
    }
}
