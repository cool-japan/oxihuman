// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashSet;

/// Represents a directed edge as a pair of vertex indices.
#[allow(dead_code)]
pub type SeamMarkEdge = (u32, u32);

/// A set of marked seam edges.
#[allow(dead_code)]
#[derive(Default)]
pub struct SeamMarkSet {
    pub edges: HashSet<SeamMarkEdge>,
}

/// Mark an edge as a seam.
#[allow(dead_code)]
pub fn mark_seam(set: &mut SeamMarkSet, a: u32, b: u32) {
    let key = if a < b { (a, b) } else { (b, a) };
    set.edges.insert(key);
}

/// Unmark a seam edge.
#[allow(dead_code)]
pub fn unmark_seam(set: &mut SeamMarkSet, a: u32, b: u32) {
    let key = if a < b { (a, b) } else { (b, a) };
    set.edges.remove(&key);
}

/// Check if an edge is marked as a seam.
#[allow(dead_code)]
pub fn is_seam(set: &SeamMarkSet, a: u32, b: u32) -> bool {
    let key = if a < b { (a, b) } else { (b, a) };
    set.edges.contains(&key)
}

/// Count total marked seam edges.
#[allow(dead_code)]
pub fn seam_count(set: &SeamMarkSet) -> usize {
    set.edges.len()
}

/// Clear all seam marks.
#[allow(dead_code)]
pub fn clear_seams(set: &mut SeamMarkSet) {
    set.edges.clear();
}

/// Mark all boundary edges from a mesh as seams.
#[allow(dead_code)]
pub fn mark_boundary_seams(set: &mut SeamMarkSet, indices: &[u32]) {
    let mut edge_count: std::collections::HashMap<SeamMarkEdge, usize> =
        std::collections::HashMap::new();
    let n = indices.len() / 3;
    for fi in 0..n {
        let [a, b, c] = [indices[fi * 3], indices[fi * 3 + 1], indices[fi * 3 + 2]];
        for (u, v) in [(a, b), (b, c), (c, a)] {
            let key = if u < v { (u, v) } else { (v, u) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }
    for (edge, count) in edge_count {
        if count == 1 {
            set.edges.insert(edge);
        }
    }
}

/// Serialize seam set to JSON.
#[allow(dead_code)]
pub fn seam_mark_to_json(set: &SeamMarkSet) -> String {
    format!(r#"{{"seam_count":{}}}"#, set.edges.len())
}

/// List all seam edges as sorted vec.
#[allow(dead_code)]
pub fn seam_edges_sorted(set: &SeamMarkSet) -> Vec<SeamMarkEdge> {
    let mut v: Vec<_> = set.edges.iter().cloned().collect();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_and_check() {
        let mut s = SeamMarkSet::default();
        mark_seam(&mut s, 0, 1);
        assert!(is_seam(&s, 0, 1));
        assert!(is_seam(&s, 1, 0));
    }

    #[test]
    fn unmark() {
        let mut s = SeamMarkSet::default();
        mark_seam(&mut s, 0, 1);
        unmark_seam(&mut s, 0, 1);
        assert!(!is_seam(&s, 0, 1));
    }

    #[test]
    fn count_correct() {
        let mut s = SeamMarkSet::default();
        mark_seam(&mut s, 0, 1);
        mark_seam(&mut s, 1, 2);
        assert_eq!(seam_count(&s), 2);
    }

    #[test]
    fn clear_all() {
        let mut s = SeamMarkSet::default();
        mark_seam(&mut s, 0, 1);
        clear_seams(&mut s);
        assert_eq!(seam_count(&s), 0);
    }

    #[test]
    fn json_has_count() {
        let mut s = SeamMarkSet::default();
        mark_seam(&mut s, 0, 1);
        let j = seam_mark_to_json(&s);
        assert!(j.contains("\"seam_count\":1"));
    }

    #[test]
    fn boundary_seams_triangle() {
        let idx = vec![0_u32, 1, 2];
        let mut s = SeamMarkSet::default();
        mark_boundary_seams(&mut s, &idx);
        assert_eq!(seam_count(&s), 3);
    }

    #[test]
    fn interior_edge_not_seam() {
        // Two triangles sharing edge 0-1
        let idx = vec![0_u32, 1, 2, 0, 2, 3];
        let mut s = SeamMarkSet::default();
        mark_boundary_seams(&mut s, &idx);
        assert!(!is_seam(&s, 0, 2));
    }

    #[test]
    fn sorted_edges() {
        let mut s = SeamMarkSet::default();
        mark_seam(&mut s, 2, 3);
        mark_seam(&mut s, 0, 1);
        let v = seam_edges_sorted(&s);
        assert_eq!(v[0], (0, 1));
    }

    #[test]
    fn duplicate_mark() {
        let mut s = SeamMarkSet::default();
        mark_seam(&mut s, 0, 1);
        mark_seam(&mut s, 0, 1);
        assert_eq!(seam_count(&s), 1);
    }

    #[test]
    fn empty_set_not_seam() {
        let s = SeamMarkSet::default();
        assert!(!is_seam(&s, 0, 1));
    }
}
