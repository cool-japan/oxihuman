// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Named vertex weight groups for mesh deformation.

use std::collections::HashMap;

/// An entry in a vertex weight group.
#[derive(Debug, Clone)]
pub struct VertexWeight {
    pub vertex_index: usize,
    pub weight: f32,
}

/// A named group of weighted vertices.
#[derive(Debug, Clone)]
pub struct VertexWeightGroup {
    pub name: String,
    pub entries: Vec<VertexWeight>,
}

impl VertexWeightGroup {
    /// Create an empty named group.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            entries: Vec::new(),
        }
    }

    /// Add or update a vertex weight.
    pub fn set_weight(&mut self, vertex: usize, weight: f32) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.vertex_index == vertex) {
            e.weight = weight.clamp(0.0, 1.0);
        } else {
            self.entries.push(VertexWeight {
                vertex_index: vertex,
                weight: weight.clamp(0.0, 1.0),
            });
        }
    }

    /// Get the weight for a vertex (0.0 if not present).
    pub fn get_weight(&self, vertex: usize) -> f32 {
        self.entries
            .iter()
            .find(|e| e.vertex_index == vertex)
            .map_or(0.0, |e| e.weight)
    }

    /// Return entry count.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Collection of named vertex weight groups for a mesh.
#[derive(Debug, Clone)]
pub struct VertexWeightGroupSet {
    pub groups: HashMap<String, VertexWeightGroup>,
}

impl VertexWeightGroupSet {
    /// Create an empty set.
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// Add a group (or replace existing).
    pub fn add_group(&mut self, group: VertexWeightGroup) {
        self.groups.insert(group.name.clone(), group);
    }

    /// Return group count.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }

    /// Get a mutable reference to a named group.
    pub fn get_group_mut(&mut self, name: &str) -> Option<&mut VertexWeightGroup> {
        self.groups.get_mut(name)
    }
}

impl Default for VertexWeightGroupSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize weights across groups so they sum to ≤ 1 per vertex.
pub fn normalize_across_groups(set: &mut VertexWeightGroupSet, vertex: usize) {
    let sum: f32 = set.groups.values().map(|g| g.get_weight(vertex)).sum();
    if sum > 1e-6 {
        for g in set.groups.values_mut() {
            if let Some(e) = g.entries.iter_mut().find(|e| e.vertex_index == vertex) {
                e.weight /= sum;
            }
        }
    }
}

/// Return the name of the group with the highest weight for a vertex.
pub fn dominant_group(set: &VertexWeightGroupSet, vertex: usize) -> Option<&str> {
    set.groups
        .values()
        .max_by(|a, b| {
            a.get_weight(vertex)
                .partial_cmp(&b.get_weight(vertex))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .filter(|g| g.get_weight(vertex) > 0.0)
        .map(|g| g.name.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_set() -> VertexWeightGroupSet {
        let mut set = VertexWeightGroupSet::new();
        let mut g1 = VertexWeightGroup::new("bones");
        g1.set_weight(0, 0.8);
        g1.set_weight(1, 0.5);
        let mut g2 = VertexWeightGroup::new("muscles");
        g2.set_weight(0, 0.3);
        set.add_group(g1);
        set.add_group(g2);
        set
    }

    #[test]
    fn test_group_count() {
        /* set has correct group count */
        let s = build_set();
        assert_eq!(s.group_count(), 2);
    }

    #[test]
    fn test_get_weight() {
        /* get_weight returns stored value */
        let s = build_set();
        let g = s.groups.get("bones").expect("should succeed");
        assert!((g.get_weight(0) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight_clamps() {
        /* weights are clamped to [0,1] */
        let mut g = VertexWeightGroup::new("test");
        g.set_weight(0, 2.5);
        assert!((g.get_weight(0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight_updates_existing() {
        /* set_weight updates existing entry rather than adding duplicate */
        let mut g = VertexWeightGroup::new("test");
        g.set_weight(0, 0.5);
        g.set_weight(0, 0.9);
        assert_eq!(g.entry_count(), 1);
        assert!((g.get_weight(0) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_get_weight_missing_vertex_zero() {
        /* get_weight for absent vertex returns 0.0 */
        let g = VertexWeightGroup::new("empty");
        assert_eq!(g.get_weight(5), 0.0);
    }

    #[test]
    fn test_normalize_across_groups() {
        /* normalize brings weights below 1 */
        let mut s = build_set();
        normalize_across_groups(&mut s, 0);
        let total: f32 = s.groups.values().map(|g| g.get_weight(0)).sum();
        assert!((total - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_dominant_group() {
        /* dominant group has highest weight */
        let s = build_set();
        let dom = dominant_group(&s, 0);
        assert!(dom == Some("bones"));
    }

    #[test]
    fn test_dominant_group_zero_weight() {
        /* dominant group returns None when all weights are 0 */
        let s = VertexWeightGroupSet::new();
        assert!(dominant_group(&s, 0).is_none());
    }
}
