// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Edge loop export for mesh topology information.

/// An edge loop (ordered sequence of vertex indices).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeLoopExport {
    pub name: String,
    pub vertices: Vec<u32>,
}

/// Collection of edge loops.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeLoopBundle {
    pub loops: Vec<EdgeLoopExport>,
}

/// Create new bundle.
#[allow(dead_code)]
pub fn new_edge_loop_bundle() -> EdgeLoopBundle {
    EdgeLoopBundle { loops: vec![] }
}

/// Add an edge loop.
#[allow(dead_code)]
pub fn add_edge_loop(b: &mut EdgeLoopBundle, name: &str, verts: &[u32]) {
    b.loops.push(EdgeLoopExport {
        name: name.to_string(),
        vertices: verts.to_vec(),
    });
}

/// Loop count.
#[allow(dead_code)]
pub fn el_loop_count(b: &EdgeLoopBundle) -> usize {
    b.loops.len()
}

/// Total vertex count across all loops.
#[allow(dead_code)]
pub fn el_total_vertices(b: &EdgeLoopBundle) -> usize {
    b.loops.iter().map(|l| l.vertices.len()).sum()
}

/// Get loop by name.
#[allow(dead_code)]
pub fn get_edge_loop<'a>(b: &'a EdgeLoopBundle, name: &str) -> Option<&'a EdgeLoopExport> {
    b.loops.iter().find(|l| l.name == name)
}

/// Check if a loop is closed (first == last).
#[allow(dead_code)]
pub fn is_closed_loop(l: &EdgeLoopExport) -> bool {
    l.vertices.len() >= 2 && l.vertices.first() == l.vertices.last()
}

/// Largest loop vertex count.
#[allow(dead_code)]
pub fn largest_loop_size(b: &EdgeLoopBundle) -> usize {
    b.loops.iter().map(|l| l.vertices.len()).max().unwrap_or(0)
}

/// Validate.
#[allow(dead_code)]
pub fn el_validate(b: &EdgeLoopBundle) -> bool {
    b.loops.iter().all(|l| !l.vertices.is_empty())
}

/// Export to JSON.
#[allow(dead_code)]
pub fn edge_loop_bundle_to_json(b: &EdgeLoopBundle) -> String {
    format!(
        "{{\"loop_count\":{},\"total_vertices\":{}}}",
        el_loop_count(b),
        el_total_vertices(b)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let b = new_edge_loop_bundle();
        assert_eq!(el_loop_count(&b), 0);
    }
    #[test]
    fn test_add() {
        let mut b = new_edge_loop_bundle();
        add_edge_loop(&mut b, "waist", &[0, 1, 2]);
        assert_eq!(el_loop_count(&b), 1);
    }
    #[test]
    fn test_total_verts() {
        let mut b = new_edge_loop_bundle();
        add_edge_loop(&mut b, "a", &[0, 1]);
        add_edge_loop(&mut b, "b", &[2, 3, 4]);
        assert_eq!(el_total_vertices(&b), 5);
    }
    #[test]
    fn test_get() {
        let mut b = new_edge_loop_bundle();
        add_edge_loop(&mut b, "neck", &[0]);
        assert!(get_edge_loop(&b, "neck").is_some());
    }
    #[test]
    fn test_get_missing() {
        let b = new_edge_loop_bundle();
        assert!(get_edge_loop(&b, "x").is_none());
    }
    #[test]
    fn test_is_closed() {
        let l = EdgeLoopExport {
            name: "c".to_string(),
            vertices: vec![0, 1, 2, 0],
        };
        assert!(is_closed_loop(&l));
    }
    #[test]
    fn test_not_closed() {
        let l = EdgeLoopExport {
            name: "c".to_string(),
            vertices: vec![0, 1, 2],
        };
        assert!(!is_closed_loop(&l));
    }
    #[test]
    fn test_largest() {
        let mut b = new_edge_loop_bundle();
        add_edge_loop(&mut b, "a", &[0, 1]);
        add_edge_loop(&mut b, "b", &[0, 1, 2, 3]);
        assert_eq!(largest_loop_size(&b), 4);
    }
    #[test]
    fn test_validate() {
        let mut b = new_edge_loop_bundle();
        add_edge_loop(&mut b, "a", &[0]);
        assert!(el_validate(&b));
    }
    #[test]
    fn test_to_json() {
        let b = new_edge_loop_bundle();
        assert!(edge_loop_bundle_to_json(&b).contains("\"loop_count\":0"));
    }
}
