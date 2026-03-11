// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

pub struct MeshBoundary {
    pub loops: Vec<Vec<usize>>,
}

pub fn new_mesh_boundary() -> MeshBoundary {
    MeshBoundary { loops: Vec::new() }
}

pub fn boundary_add_loop(b: &mut MeshBoundary, loop_verts: Vec<usize>) {
    b.loops.push(loop_verts);
}

pub fn boundary_hole_count(b: &MeshBoundary) -> usize {
    b.loops.len()
}

pub fn boundary_total_vertices(b: &MeshBoundary) -> usize {
    b.loops.iter().map(|l| l.len()).sum()
}

pub fn detect_boundary_edges(faces: &[[usize; 3]]) -> Vec<(usize, usize)> {
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for face in faces {
        for i in 0..3 {
            let a = face[i];
            let b = face[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    edge_count
        .into_iter()
        .filter(|(_, count)| *count == 1)
        .map(|(k, _)| k)
        .collect()
}

pub fn is_closed_mesh(faces: &[[usize; 3]]) -> bool {
    detect_boundary_edges(faces).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mesh_boundary() {
        /* starts empty */
        let b = new_mesh_boundary();
        assert_eq!(boundary_hole_count(&b), 0);
    }

    #[test]
    fn test_boundary_add_loop() {
        /* adds a loop */
        let mut b = new_mesh_boundary();
        boundary_add_loop(&mut b, vec![0, 1, 2]);
        assert_eq!(boundary_hole_count(&b), 1);
    }

    #[test]
    fn test_boundary_total_vertices() {
        /* sums vertices across loops */
        let mut b = new_mesh_boundary();
        boundary_add_loop(&mut b, vec![0, 1, 2]);
        boundary_add_loop(&mut b, vec![3, 4]);
        assert_eq!(boundary_total_vertices(&b), 5);
    }

    #[test]
    fn test_detect_boundary_edges_open_mesh() {
        /* single triangle has 3 boundary edges */
        let faces = vec![[0usize, 1, 2]];
        let edges = detect_boundary_edges(&faces);
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_is_closed_mesh_open() {
        /* single triangle is not closed */
        let faces = vec![[0usize, 1, 2]];
        assert!(!is_closed_mesh(&faces));
    }

    #[test]
    fn test_detect_boundary_edges_shared_edge() {
        /* two triangles sharing edge: that edge is not boundary */
        let faces = vec![[0usize, 1, 2], [0, 2, 3]];
        let edges = detect_boundary_edges(&faces);
        /* 4 boundary edges remain (each tri contributes 3 minus 1 shared) */
        assert_eq!(edges.len(), 4);
    }
}
