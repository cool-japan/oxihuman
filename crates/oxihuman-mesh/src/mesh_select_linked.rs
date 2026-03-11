// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Flood-fill face selection (select linked).

use std::collections::{HashMap, HashSet, VecDeque};

/// Build a face adjacency map: face_index → list of adjacent face indices sharing an edge.
pub fn build_face_adjacency(indices: &[u32]) -> HashMap<usize, Vec<usize>> {
    let face_count = indices.len() / 3;
    /* edge → list of face indices */
    let mut edge_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for fi in 0..face_count {
        let base = fi * 3;
        let tri = &indices[base..base + 3];
        for e in 0..3 {
            let a = tri[e];
            let b = tri[(e + 1) % 3];
            let key = (a.min(b), a.max(b));
            edge_faces.entry(key).or_default().push(fi);
        }
    }
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for faces in edge_faces.values() {
        if faces.len() == 2 {
            adj.entry(faces[0]).or_default().push(faces[1]);
            adj.entry(faces[1]).or_default().push(faces[0]);
        }
    }
    adj
}

/// BFS flood-fill from seed face, returning all connected face indices.
pub fn select_linked_faces(indices: &[u32], seed_face: usize) -> Vec<usize> {
    let adj = build_face_adjacency(indices);
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(seed_face);
    visited.insert(seed_face);
    while let Some(fi) = queue.pop_front() {
        if let Some(neighbors) = adj.get(&fi) {
            for &n in neighbors {
                if visited.insert(n) {
                    queue.push_back(n);
                }
            }
        }
    }
    let mut result: Vec<usize> = visited.into_iter().collect();
    result.sort_unstable();
    result
}

/// Collect all unique vertex indices used by the selected faces.
pub fn select_linked_vertices(indices: &[u32], selected_faces: &[usize]) -> Vec<u32> {
    let mut verts = HashSet::new();
    for &fi in selected_faces {
        let base = fi * 3;
        if base + 2 < indices.len() {
            verts.insert(indices[base]);
            verts.insert(indices[base + 1]);
            verts.insert(indices[base + 2]);
        }
    }
    let mut result: Vec<u32> = verts.into_iter().collect();
    result.sort_unstable();
    result
}

/// Count the number of disconnected mesh islands.
pub fn count_islands(indices: &[u32]) -> usize {
    let face_count = indices.len() / 3;
    let adj = build_face_adjacency(indices);
    let mut visited = HashSet::new();
    let mut islands = 0;
    for fi in 0..face_count {
        if visited.contains(&fi) {
            continue;
        }
        islands += 1;
        let mut queue = VecDeque::new();
        queue.push_back(fi);
        visited.insert(fi);
        while let Some(f) = queue.pop_front() {
            if let Some(neighbors) = adj.get(&f) {
                for &n in neighbors {
                    if visited.insert(n) {
                        queue.push_back(n);
                    }
                }
            }
        }
    }
    islands
}

/// Returns true if face `a` and face `b` share an edge.
pub fn face_shares_edge(indices: &[u32], a: usize, b: usize) -> bool {
    let base_a = a * 3;
    let base_b = b * 3;
    if base_a + 2 >= indices.len() || base_b + 2 >= indices.len() {
        return false;
    }
    let ta = &indices[base_a..base_a + 3];
    let tb = &indices[base_b..base_b + 3];
    let mut shared = 0;
    for &va in ta {
        if tb.contains(&va) {
            shared += 1;
        }
    }
    shared >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris() -> Vec<u32> {
        /* two adjacent triangles sharing edge 1-2 */
        vec![0, 1, 2, 1, 3, 2]
    }

    fn two_islands() -> Vec<u32> {
        /* two disconnected triangles */
        vec![0, 1, 2, 3, 4, 5]
    }

    #[test]
    fn test_select_linked_both_faces() {
        let idx = two_tris();
        let sel = select_linked_faces(&idx, 0);
        assert_eq!(sel.len(), 2);
    }

    #[test]
    fn test_face_shares_edge_true() {
        let idx = two_tris();
        assert!(face_shares_edge(&idx, 0, 1));
    }

    #[test]
    fn test_face_shares_edge_false() {
        let idx = two_islands();
        assert!(!face_shares_edge(&idx, 0, 1));
    }

    #[test]
    fn test_count_islands_one() {
        let idx = two_tris();
        assert_eq!(count_islands(&idx), 1);
    }

    #[test]
    fn test_count_islands_two() {
        let idx = two_islands();
        assert_eq!(count_islands(&idx), 2);
    }

    #[test]
    fn test_select_linked_vertices() {
        let idx = two_tris();
        let faces = select_linked_faces(&idx, 0);
        let verts = select_linked_vertices(&idx, &faces);
        /* triangles share 4 unique vertices: 0,1,2,3 */
        assert_eq!(verts.len(), 4);
    }
}
