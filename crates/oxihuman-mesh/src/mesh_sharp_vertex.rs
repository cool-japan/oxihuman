// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sharp vertex (corner) marking for subdivision surfaces.

/// Marks sharp (corner) vertices by index.
pub struct SharpVertexSet {
    pub marked: Vec<u32>,
    pub sharpness: Vec<f32>,
}

/// Create a new empty sharp vertex set.
pub fn new_sharp_vertex_set() -> SharpVertexSet {
    SharpVertexSet {
        marked: Vec::new(),
        sharpness: Vec::new(),
    }
}

/// Mark a vertex as sharp with given sharpness (clamped to [0, 10]).
pub fn mark_sharp_vertex(set: &mut SharpVertexSet, vertex: u32, sharpness: f32) {
    if !set.marked.contains(&vertex) {
        set.marked.push(vertex);
        set.sharpness.push(sharpness.clamp(0.0, 10.0));
    }
}

/// Unmark a vertex; returns true if it was marked.
pub fn unmark_sharp_vertex(set: &mut SharpVertexSet, vertex: u32) -> bool {
    if let Some(pos) = set.marked.iter().position(|&v| v == vertex) {
        set.marked.remove(pos);
        set.sharpness.remove(pos);
        true
    } else {
        false
    }
}

/// Check whether a vertex is marked sharp.
pub fn is_sharp_vertex(set: &SharpVertexSet, vertex: u32) -> bool {
    set.marked.contains(&vertex)
}

/// Get sharpness of a vertex; returns None if not marked.
pub fn get_vertex_sharpness(set: &SharpVertexSet, vertex: u32) -> Option<f32> {
    set.marked
        .iter()
        .position(|&v| v == vertex)
        .map(|i| set.sharpness[i])
}

/// Count of marked sharp vertices.
pub fn sharp_vertex_count(set: &SharpVertexSet) -> usize {
    set.marked.len()
}

/// Maximum sharpness among all marked vertices; None if empty.
pub fn max_sharpness(set: &SharpVertexSet) -> Option<f32> {
    set.sharpness.iter().cloned().reduce(f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_is_empty() {
        let s = new_sharp_vertex_set();
        assert_eq!(sharp_vertex_count(&s), 0 /* empty */);
    }

    #[test]
    fn mark_vertex_increases_count() {
        let mut s = new_sharp_vertex_set();
        mark_sharp_vertex(&mut s, 5, 2.0);
        assert_eq!(sharp_vertex_count(&s), 1 /* one marked */);
    }

    #[test]
    fn is_sharp_returns_true_for_marked() {
        let mut s = new_sharp_vertex_set();
        mark_sharp_vertex(&mut s, 3, 1.5);
        assert!(is_sharp_vertex(&s, 3) /* marked */);
    }

    #[test]
    fn is_sharp_returns_false_for_unmarked() {
        let s = new_sharp_vertex_set();
        assert!(!is_sharp_vertex(&s, 99) /* not marked */);
    }

    #[test]
    fn duplicate_mark_not_added() {
        let mut s = new_sharp_vertex_set();
        mark_sharp_vertex(&mut s, 7, 1.0);
        mark_sharp_vertex(&mut s, 7, 2.0);
        assert_eq!(sharp_vertex_count(&s), 1 /* deduped */);
    }

    #[test]
    fn unmark_returns_true_and_removes() {
        let mut s = new_sharp_vertex_set();
        mark_sharp_vertex(&mut s, 4, 3.0);
        let removed = unmark_sharp_vertex(&mut s, 4);
        assert!(removed /* was present */);
        assert_eq!(sharp_vertex_count(&s), 0 /* removed */);
    }

    #[test]
    fn unmark_missing_returns_false() {
        let mut s = new_sharp_vertex_set();
        assert!(!unmark_sharp_vertex(&mut s, 99) /* not found */);
    }

    #[test]
    fn get_vertex_sharpness_returns_value() {
        let mut s = new_sharp_vertex_set();
        mark_sharp_vertex(&mut s, 2, 4.5);
        let sh = get_vertex_sharpness(&s, 2);
        assert!((sh.unwrap() - 4.5).abs() < 1e-6 /* correct sharpness */);
    }

    #[test]
    fn max_sharpness_none_when_empty() {
        let s = new_sharp_vertex_set();
        assert!(max_sharpness(&s).is_none() /* empty */);
    }

    #[test]
    fn max_sharpness_finds_maximum() {
        let mut s = new_sharp_vertex_set();
        mark_sharp_vertex(&mut s, 0, 1.0);
        mark_sharp_vertex(&mut s, 1, 5.0);
        mark_sharp_vertex(&mut s, 2, 3.0);
        assert!((max_sharpness(&s).unwrap() - 5.0).abs() < 1e-6 /* max is 5 */);
    }
}
