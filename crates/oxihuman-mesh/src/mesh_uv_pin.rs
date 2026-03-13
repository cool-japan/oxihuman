// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! UV vertex pinning for relaxation algorithms.

/// Stores pinned UV vertices.
pub struct UvPinSet {
    pub pinned: Vec<u32>,
    pub pin_uvs: Vec<[f32; 2]>,
}

/// Create a new empty UV pin set.
pub fn new_uv_pin_set() -> UvPinSet {
    UvPinSet {
        pinned: Vec::new(),
        pin_uvs: Vec::new(),
    }
}

/// Pin a UV vertex at a specific UV coordinate.
pub fn pin_vertex(set: &mut UvPinSet, vertex: u32, uv: [f32; 2]) {
    if let Some(pos) = set.pinned.iter().position(|&v| v == vertex) {
        set.pin_uvs[pos] = uv;
    } else {
        set.pinned.push(vertex);
        set.pin_uvs.push(uv);
    }
}

/// Unpin a vertex; returns true if it was pinned.
pub fn unpin_vertex(set: &mut UvPinSet, vertex: u32) -> bool {
    if let Some(pos) = set.pinned.iter().position(|&v| v == vertex) {
        set.pinned.remove(pos);
        set.pin_uvs.remove(pos);
        true
    } else {
        false
    }
}

/// Check if a vertex is pinned.
pub fn is_pinned(set: &UvPinSet, vertex: u32) -> bool {
    set.pinned.contains(&vertex)
}

/// Get the pinned UV for a vertex; None if not pinned.
pub fn get_pin_uv(set: &UvPinSet, vertex: u32) -> Option<[f32; 2]> {
    set.pinned
        .iter()
        .position(|&v| v == vertex)
        .map(|i| set.pin_uvs[i])
}

/// Number of pinned vertices.
pub fn pin_count(set: &UvPinSet) -> usize {
    set.pinned.len()
}

/// Clear all pinned vertices.
pub fn clear_pins(set: &mut UvPinSet) {
    set.pinned.clear();
    set.pin_uvs.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_empty() {
        let s = new_uv_pin_set();
        assert_eq!(pin_count(&s), 0 /* empty */);
    }

    #[test]
    fn pin_vertex_adds_entry() {
        let mut s = new_uv_pin_set();
        pin_vertex(&mut s, 0, [0.5, 0.5]);
        assert_eq!(pin_count(&s), 1 /* one pin */);
    }

    #[test]
    fn is_pinned_true_after_pin() {
        let mut s = new_uv_pin_set();
        pin_vertex(&mut s, 3, [0.0, 1.0]);
        assert!(is_pinned(&s, 3) /* pinned */);
    }

    #[test]
    fn is_pinned_false_for_other() {
        let mut s = new_uv_pin_set();
        pin_vertex(&mut s, 3, [0.0, 1.0]);
        assert!(!is_pinned(&s, 4) /* not pinned */);
    }

    #[test]
    fn get_pin_uv_returns_coordinate() {
        let mut s = new_uv_pin_set();
        pin_vertex(&mut s, 1, [0.25, 0.75]);
        let uv = get_pin_uv(&s, 1).expect("should succeed");
        assert!((uv[0] - 0.25).abs() < 1e-6 /* U correct */);
        assert!((uv[1] - 0.75).abs() < 1e-6 /* V correct */);
    }

    #[test]
    fn overwrite_updates_uv() {
        let mut s = new_uv_pin_set();
        pin_vertex(&mut s, 2, [0.0, 0.0]);
        pin_vertex(&mut s, 2, [1.0, 1.0]);
        assert_eq!(pin_count(&s), 1 /* no duplicate */);
        let uv = get_pin_uv(&s, 2).expect("should succeed");
        assert!((uv[0] - 1.0).abs() < 1e-6 /* updated U */);
    }

    #[test]
    fn unpin_removes_entry() {
        let mut s = new_uv_pin_set();
        pin_vertex(&mut s, 5, [0.5, 0.5]);
        let ok = unpin_vertex(&mut s, 5);
        assert!(ok /* was pinned */);
        assert_eq!(pin_count(&s), 0 /* removed */);
    }

    #[test]
    fn unpin_missing_returns_false() {
        let mut s = new_uv_pin_set();
        assert!(!unpin_vertex(&mut s, 99) /* not found */);
    }

    #[test]
    fn clear_removes_all() {
        let mut s = new_uv_pin_set();
        pin_vertex(&mut s, 0, [0.0, 0.0]);
        pin_vertex(&mut s, 1, [1.0, 0.0]);
        clear_pins(&mut s);
        assert_eq!(pin_count(&s), 0 /* cleared */);
    }

    #[test]
    fn get_missing_none() {
        let s = new_uv_pin_set();
        assert!(get_pin_uv(&s, 7).is_none() /* none */);
    }
}
