// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Seam marking v2 — seams as edge pairs for UV unwrapping.

use std::collections::HashSet;

/// An edge represented as an ordered pair of vertex indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeamEdge2 {
    pub a: usize,
    pub b: usize,
}

impl SeamEdge2 {
    /// Create a canonical (ordered) edge.
    pub fn new(a: usize, b: usize) -> Self {
        if a <= b {
            Self { a, b }
        } else {
            Self { a: b, b: a }
        }
    }
}

/// Collection of seam edges.
#[derive(Debug, Clone)]
pub struct SeamSet2 {
    pub seams: HashSet<SeamEdge2>,
}

impl SeamSet2 {
    /// Create an empty seam set.
    pub fn new() -> Self {
        Self {
            seams: HashSet::new(),
        }
    }

    /// Mark an edge as a seam.
    pub fn mark(&mut self, a: usize, b: usize) {
        self.seams.insert(SeamEdge2::new(a, b));
    }

    /// Unmark an edge.
    pub fn unmark(&mut self, a: usize, b: usize) {
        self.seams.remove(&SeamEdge2::new(a, b));
    }

    /// Check if an edge is a seam.
    pub fn is_seam(&self, a: usize, b: usize) -> bool {
        self.seams.contains(&SeamEdge2::new(a, b))
    }

    /// Return seam count.
    pub fn seam_count(&self) -> usize {
        self.seams.len()
    }

    /// Clear all seams.
    pub fn clear(&mut self) {
        self.seams.clear();
    }
}

impl Default for SeamSet2 {
    fn default() -> Self {
        Self::new()
    }
}

/// Mark all boundary edges as seams.
pub fn mark_boundary_seams2(seam_set: &mut SeamSet2, indices: &[u32]) {
    use std::collections::HashMap;
    let mut edge_count: HashMap<SeamEdge2, usize> = HashMap::new();
    let face_count = indices.len() / 3;
    for fi in 0..face_count {
        let a = indices[fi * 3] as usize;
        let b = indices[fi * 3 + 1] as usize;
        let c = indices[fi * 3 + 2] as usize;
        for edge in [
            SeamEdge2::new(a, b),
            SeamEdge2::new(b, c),
            SeamEdge2::new(c, a),
        ] {
            *edge_count.entry(edge).or_insert(0) += 1;
        }
    }
    for (edge, count) in &edge_count {
        if *count == 1 {
            seam_set.seams.insert(*edge);
        }
    }
}

/// Sorted seam edge list.
pub fn sorted_seams2(seam_set: &SeamSet2) -> Vec<SeamEdge2> {
    let mut v: Vec<SeamEdge2> = seam_set.seams.iter().copied().collect();
    v.sort_by_key(|e| (e.a, e.b));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_and_check() {
        /* marked edge is detected as seam */
        let mut s = SeamSet2::new();
        s.mark(0, 1);
        assert!(s.is_seam(0, 1));
        assert!(s.is_seam(1, 0));
    }

    #[test]
    fn test_unmark() {
        /* unmark removes seam */
        let mut s = SeamSet2::new();
        s.mark(0, 1);
        s.unmark(1, 0);
        assert!(!s.is_seam(0, 1));
    }

    #[test]
    fn test_seam_count() {
        /* seam count is correct */
        let mut s = SeamSet2::new();
        s.mark(0, 1);
        s.mark(2, 3);
        assert_eq!(s.seam_count(), 2);
    }

    #[test]
    fn test_clear() {
        /* clear removes all seams */
        let mut s = SeamSet2::new();
        s.mark(0, 1);
        s.clear();
        assert_eq!(s.seam_count(), 0);
    }

    #[test]
    fn test_canonical_edge_order() {
        /* canonical edge normalizes vertex order */
        let e1 = SeamEdge2::new(3, 1);
        let e2 = SeamEdge2::new(1, 3);
        assert_eq!(e1, e2);
    }

    #[test]
    fn test_mark_boundary_seams2() {
        /* boundary edges (shared by only one face) are marked */
        let mut s = SeamSet2::new();
        let idx = vec![0u32, 1, 2];
        mark_boundary_seams2(&mut s, &idx);
        assert_eq!(s.seam_count(), 3);
    }

    #[test]
    fn test_shared_edge_not_boundary() {
        /* interior edge shared by two faces is not marked as boundary */
        let mut s = SeamSet2::new();
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        mark_boundary_seams2(&mut s, &idx);
        assert!(!s.is_seam(0, 2));
    }

    #[test]
    fn test_sorted_seams2() {
        /* sorted seams returns ordered list */
        let mut s = SeamSet2::new();
        s.mark(5, 2);
        s.mark(0, 1);
        let sorted = sorted_seams2(&s);
        assert!(sorted[0].a <= sorted[1].a);
    }
}
