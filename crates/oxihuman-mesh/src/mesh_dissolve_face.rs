// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dissolve selected faces: remove the selected triangles and close the hole
//! with a new polygon fill (fan triangulation).

/// Result of the dissolve-face pass.
#[derive(Debug, Clone, Default)]
pub struct DissolveFaceResult {
    pub faces_before: usize,
    pub faces_dissolved: usize,
    pub faces_after: usize,
    pub fill_faces_added: usize,
}

/// Collects the boundary edge loop of a set of selected face indices.
/// An edge is on the boundary if it is used by exactly one of the selected faces.
pub fn boundary_of_face_set(indices: &[u32], selected: &[usize]) -> Vec<(u32, u32)> {
    let sel_set: std::collections::HashSet<usize> = selected.iter().copied().collect();
    let mut edge_count: std::collections::HashMap<(u32, u32), usize> =
        std::collections::HashMap::new();
    for &fi in &sel_set {
        if fi * 3 + 2 >= indices.len() {
            continue;
        }
        let ia = indices[fi * 3];
        let ib = indices[fi * 3 + 1];
        let ic = indices[fi * 3 + 2];
        for (a, b) in [(ia, ib), (ib, ic), (ic, ia)] {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter(|(_, c)| *c == 1)
        .map(|(k, _)| k)
        .collect()
}

/// Converts an unsorted edge list into an ordered vertex loop (if possible).
pub fn edges_to_vertex_loop(edges: &[(u32, u32)]) -> Vec<u32> {
    if edges.is_empty() {
        return vec![];
    }
    let mut adj: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
    for &(a, b) in edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }
    let start = edges[0].0;
    let mut loop_verts = vec![start];
    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    visited.insert(start);
    let mut current = start;
    loop {
        let next = adj
            .get(&current)
            .and_then(|ns| ns.iter().find(|&&n| !visited.contains(&n)));
        match next {
            Some(&v) => {
                loop_verts.push(v);
                visited.insert(v);
                current = v;
            }
            None => break,
        }
    }
    loop_verts
}

/// Performs a fan triangulation of a vertex loop.
pub fn fan_triangulate_df(loop_verts: &[u32]) -> Vec<u32> {
    if loop_verts.len() < 3 {
        return vec![];
    }
    let pivot = loop_verts[0];
    let mut tris = Vec::new();
    for i in 1..loop_verts.len().saturating_sub(1) {
        tris.push(pivot);
        tris.push(loop_verts[i]);
        tris.push(loop_verts[i + 1]);
    }
    tris
}

/// Dissolves the selected faces and replaces them with a fan fill.
pub fn dissolve_faces(indices: &[u32], selected: &[usize]) -> (Vec<u32>, DissolveFaceResult) {
    let faces_before = indices.len() / 3;
    let sel_set: std::collections::HashSet<usize> = selected.iter().copied().collect();
    /* keep unselected faces */
    let mut new_indices = Vec::with_capacity(indices.len());
    for i in 0..faces_before {
        if !sel_set.contains(&i) {
            new_indices.push(indices[i * 3]);
            new_indices.push(indices[i * 3 + 1]);
            new_indices.push(indices[i * 3 + 2]);
        }
    }
    /* compute fill */
    let boundary_edges = boundary_of_face_set(indices, selected);
    let loop_verts = edges_to_vertex_loop(&boundary_edges);
    let fill = fan_triangulate_df(&loop_verts);
    let fill_added = fill.len() / 3;
    new_indices.extend(fill);
    let result = DissolveFaceResult {
        faces_before,
        faces_dissolved: sel_set.len(),
        faces_after: new_indices.len() / 3,
        fill_faces_added: fill_added,
    };
    (new_indices, result)
}

/// Returns the count of faces that would be dissolved.
pub fn dissolve_count(selected: &[usize]) -> usize {
    selected.len()
}

/// Checks whether all selected indices are within bounds.
pub fn selection_valid(indices: &[u32], selected: &[usize]) -> bool {
    let n = indices.len() / 3;
    selected.iter().all(|&i| i < n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tris() -> Vec<u32> {
        vec![0u32, 1, 2, 1, 3, 2]
    }

    #[test]
    fn boundary_of_single_face() {
        let idx = two_tris();
        let boundary = boundary_of_face_set(&idx, &[0]);
        /* selected face has 3 boundary edges */
        assert_eq!(boundary.len(), 3);
    }

    #[test]
    fn edges_to_vertex_loop_length() {
        let edges = vec![(0u32, 1), (1, 2), (2, 0)];
        let lp = edges_to_vertex_loop(&edges);
        assert_eq!(lp.len(), 3);
    }

    #[test]
    fn fan_triangulate_quad() {
        let lp = vec![0u32, 1, 2, 3];
        let tris = fan_triangulate_df(&lp);
        assert_eq!(tris.len() / 3, 2);
    }

    #[test]
    fn dissolve_removes_selected() {
        let idx = two_tris();
        let (out, stats) = dissolve_faces(&idx, &[0]);
        assert_eq!(stats.faces_dissolved, 1);
        /* remaining plus fill */
        assert_eq!(out.len() / 3, stats.faces_after);
    }

    #[test]
    fn selection_valid_in_bounds() {
        let idx = two_tris();
        assert!(selection_valid(&idx, &[0, 1]));
    }

    #[test]
    fn selection_valid_out_of_bounds() {
        let idx = two_tris();
        assert!(!selection_valid(&idx, &[9]));
    }

    #[test]
    fn dissolve_count_matches() {
        assert_eq!(dissolve_count(&[0, 2, 4]), 3);
    }

    #[test]
    fn fan_triangulate_empty() {
        assert!(fan_triangulate_df(&[]).is_empty());
    }

    #[test]
    fn dissolve_all_faces_empty_output_except_fill() {
        let idx = vec![0u32, 1, 2];
        let (out, stats) = dissolve_faces(&idx, &[0]);
        assert_eq!(stats.faces_dissolved, 1);
        /* fill should cover the triangle */
        assert_eq!(out.len() % 3, 0);
    }
}
