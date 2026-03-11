// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Seam path detection and tracing for UV unwrapping.

#[allow(dead_code)]
pub struct SeamPath {
    pub edges: Vec<[u32; 2]>,
}

#[allow(dead_code)]
pub fn new_seam_path() -> SeamPath {
    SeamPath { edges: Vec::new() }
}

#[allow(dead_code)]
pub fn sp_add_edge(path: &mut SeamPath, a: u32, b: u32) {
    path.edges.push([a, b]);
}

#[allow(dead_code)]
pub fn sp_edge_count(path: &SeamPath) -> usize {
    path.edges.len()
}

#[allow(dead_code)]
pub fn sp_has_edge(path: &SeamPath, a: u32, b: u32) -> bool {
    path.edges.iter().any(|e| (e[0] == a && e[1] == b) || (e[0] == b && e[1] == a))
}

#[allow(dead_code)]
pub fn sp_vertex_set(path: &SeamPath) -> Vec<u32> {
    let mut verts: Vec<u32> = path.edges.iter().flat_map(|e| e.iter().copied()).collect();
    verts.sort_unstable();
    verts.dedup();
    verts
}

#[allow(dead_code)]
pub fn sp_is_closed(path: &SeamPath) -> bool {
    let mut count = std::collections::HashMap::new();
    for e in &path.edges {
        *count.entry(e[0]).or_insert(0u32) += 1;
        *count.entry(e[1]).or_insert(0u32) += 1;
    }
    count.values().all(|&c| c % 2 == 0)
}

#[allow(dead_code)]
pub fn sp_clear(path: &mut SeamPath) {
    path.edges.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        assert_eq!(sp_edge_count(&new_seam_path()), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut p = new_seam_path();
        sp_add_edge(&mut p, 0, 1);
        assert_eq!(sp_edge_count(&p), 1);
    }

    #[test]
    fn test_has_edge_forward() {
        let mut p = new_seam_path();
        sp_add_edge(&mut p, 2, 3);
        assert!(sp_has_edge(&p, 2, 3));
    }

    #[test]
    fn test_has_edge_reverse() {
        let mut p = new_seam_path();
        sp_add_edge(&mut p, 2, 3);
        assert!(sp_has_edge(&p, 3, 2));
    }

    #[test]
    fn test_has_edge_false() {
        let mut p = new_seam_path();
        sp_add_edge(&mut p, 2, 3);
        assert!(!sp_has_edge(&p, 0, 1));
    }

    #[test]
    fn test_vertex_set() {
        let mut p = new_seam_path();
        sp_add_edge(&mut p, 0, 1);
        sp_add_edge(&mut p, 1, 2);
        let vs = sp_vertex_set(&p);
        assert_eq!(vs, vec![0, 1, 2]);
    }

    #[test]
    fn test_is_closed_loop() {
        let mut p = new_seam_path();
        sp_add_edge(&mut p, 0, 1);
        sp_add_edge(&mut p, 1, 2);
        sp_add_edge(&mut p, 2, 0);
        assert!(sp_is_closed(&p));
    }

    #[test]
    fn test_is_not_closed_open_path() {
        let mut p = new_seam_path();
        sp_add_edge(&mut p, 0, 1);
        sp_add_edge(&mut p, 1, 2);
        assert!(!sp_is_closed(&p));
    }

    #[test]
    fn test_clear() {
        let mut p = new_seam_path();
        sp_add_edge(&mut p, 0, 1);
        sp_clear(&mut p);
        assert_eq!(sp_edge_count(&p), 0);
    }
}
