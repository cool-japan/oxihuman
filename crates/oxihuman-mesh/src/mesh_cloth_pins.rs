// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cloth pinning vertex groups.

/// A single cloth pin entry.
#[derive(Debug, Clone)]
pub struct ClothPinEntry {
    pub vertex_index: usize,
    pub strength: f32,
}

/// Collection of cloth pin entries for a mesh.
#[derive(Debug, Clone)]
pub struct ClothPinGroup {
    pub name: String,
    pub pins: Vec<ClothPinEntry>,
}

impl ClothPinGroup {
    /// Create an empty pin group.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            pins: Vec::new(),
        }
    }

    /// Add or update a pin at the given vertex with the given strength.
    pub fn pin(&mut self, vertex: usize, strength: f32) {
        if let Some(e) = self.pins.iter_mut().find(|e| e.vertex_index == vertex) {
            e.strength = strength.clamp(0.0, 1.0);
        } else {
            self.pins.push(ClothPinEntry {
                vertex_index: vertex,
                strength: strength.clamp(0.0, 1.0),
            });
        }
    }

    /// Remove a pin, returning its strength if found.
    pub fn unpin(&mut self, vertex: usize) -> Option<f32> {
        if let Some(pos) = self.pins.iter().position(|e| e.vertex_index == vertex) {
            Some(self.pins.remove(pos).strength)
        } else {
            None
        }
    }

    /// Return true if a vertex is pinned.
    pub fn is_pinned(&self, vertex: usize) -> bool {
        self.pins.iter().any(|e| e.vertex_index == vertex)
    }

    /// Return pin count.
    pub fn pin_count(&self) -> usize {
        self.pins.len()
    }
}

/// Compute average pin strength.
pub fn average_strength(group: &ClothPinGroup) -> f32 {
    if group.pins.is_empty() {
        return 0.0;
    }
    let sum: f32 = group.pins.iter().map(|e| e.strength).sum();
    sum / group.pins.len() as f32
}

/// Count fully-pinned vertices (strength == 1.0).
pub fn fully_pinned_count(group: &ClothPinGroup) -> usize {
    group
        .pins
        .iter()
        .filter(|e| (e.strength - 1.0).abs() < 1e-5)
        .count()
}

/// Return pin vertex indices as a sorted vec.
pub fn pinned_vertex_indices(group: &ClothPinGroup) -> Vec<usize> {
    let mut indices: Vec<usize> = group.pins.iter().map(|e| e.vertex_index).collect();
    indices.sort_unstable();
    indices
}

/// Scale all pin strengths by a factor (clamped to `[0,1]`).
pub fn scale_strengths(group: &mut ClothPinGroup, factor: f32) {
    for e in &mut group.pins {
        e.strength = (e.strength * factor).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_group() -> ClothPinGroup {
        let mut g = ClothPinGroup::new("anchors");
        g.pin(0, 1.0);
        g.pin(1, 0.5);
        g.pin(2, 0.8);
        g
    }

    #[test]
    fn test_pin_count() {
        /* pin count is correct */
        let g = basic_group();
        assert_eq!(g.pin_count(), 3);
    }

    #[test]
    fn test_is_pinned() {
        /* is_pinned detects pinned vertices */
        let g = basic_group();
        assert!(g.is_pinned(1));
        assert!(!g.is_pinned(99));
    }

    #[test]
    fn test_unpin() {
        /* unpin removes the entry */
        let mut g = basic_group();
        let strength = g.unpin(1);
        assert!(strength.is_some());
        assert_eq!(g.pin_count(), 2);
    }

    #[test]
    fn test_average_strength() {
        /* average strength is computed correctly */
        let g = basic_group();
        let avg = average_strength(&g);
        assert!((avg - (1.0 + 0.5 + 0.8) / 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_fully_pinned_count() {
        /* fully pinned count is correct */
        let g = basic_group();
        assert_eq!(fully_pinned_count(&g), 1);
    }

    #[test]
    fn test_pinned_vertex_indices_sorted() {
        /* indices are returned sorted */
        let mut g = ClothPinGroup::new("test");
        g.pin(5, 1.0);
        g.pin(2, 0.5);
        g.pin(9, 0.3);
        let indices = pinned_vertex_indices(&g);
        assert_eq!(indices, vec![2, 5, 9]);
    }

    #[test]
    fn test_scale_strengths() {
        /* scaling halves strengths */
        let mut g = basic_group();
        scale_strengths(&mut g, 0.5);
        let pin0_strength = g
            .pins
            .iter()
            .find(|e| e.vertex_index == 0)
            .expect("should succeed")
            .strength;
        assert!((pin0_strength - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_update_existing_pin() {
        /* pinning existing vertex updates strength */
        let mut g = basic_group();
        g.pin(0, 0.3);
        let s = g
            .pins
            .iter()
            .find(|e| e.vertex_index == 0)
            .expect("should succeed")
            .strength;
        assert!((s - 0.3).abs() < 1e-6);
        assert_eq!(g.pin_count(), 3);
    }
}
