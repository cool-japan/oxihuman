// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! UV seam detection, marking and analysis for mesh parameterisation.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UvSeamEdge {
    pub v0: u32,
    pub v1: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UvSeamSet {
    pub edges: Vec<UvSeamEdge>,
}

#[allow(dead_code)]
pub fn new_uv_seam_set() -> UvSeamSet {
    UvSeamSet { edges: Vec::new() }
}

#[allow(dead_code)]
pub fn add_seam_edge(set: &mut UvSeamSet, v0: u32, v1: u32) {
    let edge = if v0 < v1 {
        UvSeamEdge { v0, v1 }
    } else {
        UvSeamEdge { v0: v1, v1: v0 }
    };
    if !set.edges.contains(&edge) {
        set.edges.push(edge);
    }
}

#[allow(dead_code)]
pub fn seam_edge_count(set: &UvSeamSet) -> usize {
    set.edges.len()
}

#[allow(dead_code)]
pub fn is_seam_edge(set: &UvSeamSet, v0: u32, v1: u32) -> bool {
    let (a, b) = if v0 < v1 { (v0, v1) } else { (v1, v0) };
    set.edges.iter().any(|e| e.v0 == a && e.v1 == b)
}

#[allow(dead_code)]
pub fn remove_seam_edge(set: &mut UvSeamSet, v0: u32, v1: u32) {
    let (a, b) = if v0 < v1 { (v0, v1) } else { (v1, v0) };
    set.edges.retain(|e| !(e.v0 == a && e.v1 == b));
}

#[allow(dead_code)]
pub fn clear_seams(set: &mut UvSeamSet) {
    set.edges.clear();
}

#[allow(dead_code)]
pub fn detect_boundary_seams(positions: &[[f32; 3]], indices: &[u32]) -> UvSeamSet {
    use std::collections::HashMap;
    let mut edge_count: HashMap<(u32, u32), u32> = HashMap::new();
    for tri in indices.chunks(3) {
        if tri.len() < 3 {
            continue;
        }
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    let _ = positions; // used for future normal-based detection
    let mut set = new_uv_seam_set();
    for ((v0, v1), count) in edge_count {
        if count == 1 {
            set.edges.push(UvSeamEdge { v0, v1 });
        }
    }
    set
}

#[allow(dead_code)]
pub fn seam_vertices(set: &UvSeamSet) -> Vec<u32> {
    let mut verts: Vec<u32> = set.edges.iter().flat_map(|e| [e.v0, e.v1]).collect();
    verts.sort_unstable();
    verts.dedup();
    verts
}

#[allow(dead_code)]
pub fn seam_set_to_json(set: &UvSeamSet) -> String {
    format!("{{\"seam_edge_count\":{}}}", set.edges.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_set() {
        let set = new_uv_seam_set();
        assert_eq!(seam_edge_count(&set), 0);
    }

    #[test]
    fn test_add_seam() {
        let mut set = new_uv_seam_set();
        add_seam_edge(&mut set, 0, 1);
        assert_eq!(seam_edge_count(&set), 1);
    }

    #[test]
    fn test_no_duplicate_seams() {
        let mut set = new_uv_seam_set();
        add_seam_edge(&mut set, 0, 1);
        add_seam_edge(&mut set, 1, 0);
        assert_eq!(seam_edge_count(&set), 1);
    }

    #[test]
    fn test_is_seam() {
        let mut set = new_uv_seam_set();
        add_seam_edge(&mut set, 2, 3);
        assert!(is_seam_edge(&set, 2, 3));
        assert!(is_seam_edge(&set, 3, 2));
    }

    #[test]
    fn test_remove_seam() {
        let mut set = new_uv_seam_set();
        add_seam_edge(&mut set, 0, 1);
        remove_seam_edge(&mut set, 0, 1);
        assert_eq!(seam_edge_count(&set), 0);
    }

    #[test]
    fn test_clear_seams() {
        let mut set = new_uv_seam_set();
        add_seam_edge(&mut set, 0, 1);
        add_seam_edge(&mut set, 2, 3);
        clear_seams(&mut set);
        assert_eq!(seam_edge_count(&set), 0);
    }

    #[test]
    fn test_detect_boundary_seam_triangle() {
        let positions = vec![[0.0f32; 3]; 3];
        let indices = vec![0u32, 1, 2];
        let set = detect_boundary_seams(&positions, &indices);
        assert_eq!(seam_edge_count(&set), 3);
    }

    #[test]
    fn test_seam_vertices() {
        let mut set = new_uv_seam_set();
        add_seam_edge(&mut set, 0, 1);
        add_seam_edge(&mut set, 1, 2);
        let verts = seam_vertices(&set);
        assert_eq!(verts.len(), 3);
    }

    #[test]
    fn test_json_output() {
        let set = new_uv_seam_set();
        let j = seam_set_to_json(&set);
        assert!(j.contains("seam_edge_count"));
    }

    #[test]
    fn test_internal_edge_not_boundary() {
        // Two triangles sharing an edge — shared edge should not appear as boundary
        let positions = vec![[0.0f32; 3]; 4];
        let indices = vec![0u32, 1, 2, 0, 2, 3];
        let set = detect_boundary_seams(&positions, &indices);
        assert!(!is_seam_edge(&set, 0, 2));
    }
}
