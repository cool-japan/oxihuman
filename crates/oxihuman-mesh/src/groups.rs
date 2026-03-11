// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::HashMap;

/// A named group of vertex indices.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VertexGroup {
    pub name: String,
    /// Sorted, deduplicated vertex indices.
    pub indices: Vec<u32>,
    /// Optional per-vertex weights in [0..1] (same length as indices, or empty = all 1.0).
    pub weights: Vec<f32>,
}

impl VertexGroup {
    pub fn new(name: impl Into<String>, mut indices: Vec<u32>) -> Self {
        indices.sort_unstable();
        indices.dedup();
        Self {
            name: name.into(),
            indices,
            weights: Vec::new(),
        }
    }

    pub fn with_weights(name: impl Into<String>, mut indices: Vec<u32>, weights: Vec<f32>) -> Self {
        // Sort by index, keeping weights aligned
        let mut pairs: Vec<(u32, f32)> = indices
            .iter()
            .copied()
            .zip(weights.iter().copied())
            .collect();
        pairs.sort_unstable_by_key(|(i, _)| *i);
        pairs.dedup_by_key(|(i, _)| *i);
        indices = pairs.iter().map(|(i, _)| *i).collect();
        let weights = pairs.iter().map(|(_, w)| *w).collect();
        Self {
            name: name.into(),
            indices,
            weights,
        }
    }

    /// Number of vertices in this group.
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Get the weight for a vertex index (1.0 if not in group, or weights empty).
    pub fn weight_of(&self, vid: u32) -> f32 {
        match self.indices.binary_search(&vid) {
            Ok(pos) => {
                if self.weights.is_empty() {
                    1.0
                } else {
                    self.weights[pos]
                }
            }
            Err(_) => 1.0,
        }
    }

    /// True if the group contains the given vertex index.
    pub fn contains(&self, vid: u32) -> bool {
        self.indices.binary_search(&vid).is_ok()
    }

    /// Merge another group into this one (union). Deduplicates and sorts.
    pub fn union(&self, other: &VertexGroup) -> VertexGroup {
        let mut merged = self.indices.clone();
        merged.extend_from_slice(&other.indices);
        merged.sort_unstable();
        merged.dedup();
        VertexGroup {
            name: self.name.clone(),
            indices: merged,
            weights: Vec::new(),
        }
    }

    /// Intersection with another group.
    pub fn intersect(&self, other: &VertexGroup) -> VertexGroup {
        let indices: Vec<u32> = self
            .indices
            .iter()
            .copied()
            .filter(|v| other.contains(*v))
            .collect();
        VertexGroup {
            name: self.name.clone(),
            indices,
            weights: Vec::new(),
        }
    }

    /// Difference: vertices in self but not in other.
    pub fn difference(&self, other: &VertexGroup) -> VertexGroup {
        let indices: Vec<u32> = self
            .indices
            .iter()
            .copied()
            .filter(|v| !other.contains(*v))
            .collect();
        VertexGroup {
            name: self.name.clone(),
            indices,
            weights: Vec::new(),
        }
    }
}

/// A collection of named vertex groups for a mesh.
pub struct VertexGroupMap {
    groups: HashMap<String, VertexGroup>,
}

impl VertexGroupMap {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    /// Add or replace a group.
    pub fn add(&mut self, group: VertexGroup) {
        self.groups.insert(group.name.to_lowercase(), group);
    }

    /// Get a group by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&VertexGroup> {
        self.groups.get(&name.to_lowercase())
    }

    /// Remove a group.
    pub fn remove(&mut self, name: &str) -> Option<VertexGroup> {
        self.groups.remove(&name.to_lowercase())
    }

    /// All group names.
    pub fn names(&self) -> Vec<&str> {
        self.groups.values().map(|g| g.name.as_str()).collect()
    }

    /// Number of groups.
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }

    /// Build standard human body groups from vertex positions using Y-band heuristics.
    /// Assigns vertices to body regions based on their Y coordinate (height).
    /// `total_height`: the mesh height range (max_y - min_y).
    /// `min_y`: the bottom of the mesh.
    pub fn from_y_bands(positions: &[[f32; 3]], min_y: f32, total_height: f32) -> Self {
        let bands: &[(&str, f32, f32)] = &[
            ("feet", 0.00, 0.08),
            ("lower_legs", 0.08, 0.25),
            ("upper_legs", 0.25, 0.48),
            ("torso", 0.48, 0.72),
            ("shoulders", 0.72, 0.82),
            ("neck", 0.82, 0.90),
            ("head", 0.90, 1.01), // 1.01 to catch floating point at exactly 1.0
        ];

        // Initialise one Vec per band
        let mut band_verts: Vec<Vec<u32>> = vec![Vec::new(); bands.len()];

        for (vid, pos) in positions.iter().enumerate() {
            let vy = pos[1];
            let t = if total_height > 0.0 {
                (vy - min_y) / total_height
            } else {
                0.0
            };
            for (bi, &(_, lo, hi)) in bands.iter().enumerate() {
                if t >= lo && t < hi {
                    band_verts[bi].push(vid as u32);
                    break;
                }
            }
        }

        let mut map = Self::new();
        for (bi, &(name, _, _)) in bands.iter().enumerate() {
            let group = VertexGroup::new(name, band_verts[bi].clone());
            map.add(group);
        }
        map
    }

    /// Serialize all groups to JSON.
    pub fn to_json(&self) -> serde_json::Value {
        let obj: serde_json::Map<String, serde_json::Value> = self
            .groups
            .iter()
            .map(|(key, group)| {
                (
                    key.clone(),
                    serde_json::to_value(group).unwrap_or(serde_json::Value::Null),
                )
            })
            .collect();
        serde_json::Value::Object(obj)
    }
}

impl Default for VertexGroupMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_contains_vertex() {
        let group = VertexGroup::new("test", vec![5, 10, 20]);
        assert!(group.contains(5));
        assert!(!group.contains(99));
    }

    #[test]
    fn group_union_deduplicates() {
        let a = VertexGroup::new("a", vec![0, 1, 2]);
        let b = VertexGroup::new("b", vec![1, 2, 3]);
        let u = a.union(&b);
        assert_eq!(u.indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn group_intersect() {
        let a = VertexGroup::new("a", vec![0, 1, 2]);
        let b = VertexGroup::new("b", vec![1, 2, 3]);
        let i = a.intersect(&b);
        assert_eq!(i.indices, vec![1, 2]);
    }

    #[test]
    fn group_difference() {
        let a = VertexGroup::new("a", vec![0, 1, 2]);
        let b = VertexGroup::new("b", vec![1, 2]);
        let d = a.difference(&b);
        assert_eq!(d.indices, vec![0]);
    }

    #[test]
    fn map_get_case_insensitive() {
        let mut map = VertexGroupMap::new();
        map.add(VertexGroup::new("Head", vec![0, 1]));
        assert!(map.get("head").is_some());
        assert!(map.get("HEAD").is_some());
    }

    #[test]
    fn from_y_bands_creates_seven_groups() {
        // Spread 7 vertices one per band
        let positions: Vec<[f32; 3]> = vec![
            [0.0, 0.04, 0.0], // feet       t=0.02
            [0.0, 0.16, 0.0], // lower_legs t=0.16
            [0.0, 0.36, 0.0], // upper_legs t=0.36
            [0.0, 0.60, 0.0], // torso      t=0.60
            [0.0, 0.77, 0.0], // shoulders  t=0.77
            [0.0, 0.86, 0.0], // neck       t=0.86
            [0.0, 0.95, 0.0], // head       t=0.95
        ];
        let map = VertexGroupMap::from_y_bands(&positions, 0.0, 1.0);
        assert_eq!(map.len(), 7);
    }

    #[test]
    fn from_y_bands_head_at_top() {
        // 2.0-height mesh; vertex at y=1.8 → t=0.9 → head
        let positions: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0], // feet
            [0.0, 1.8, 0.0], // head
        ];
        let map = VertexGroupMap::from_y_bands(&positions, 0.0, 2.0);
        let head = map.get("head").expect("head group missing");
        assert!(head.contains(1), "vertex 1 should be in head group");
    }

    #[test]
    fn weight_of_returns_one_for_simple_group() {
        let group = VertexGroup::new("test", vec![3, 7, 11]);
        assert!((group.weight_of(3) - 1.0).abs() < 1e-6);
        assert!((group.weight_of(7) - 1.0).abs() < 1e-6);
        // Not in group also returns 1.0
        assert!((group.weight_of(99) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn to_json_has_keys() {
        let mut map = VertexGroupMap::new();
        map.add(VertexGroup::new("head", vec![0]));
        map.add(VertexGroup::new("feet", vec![1]));
        let json = map.to_json();
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("head"));
        assert!(obj.contains_key("feet"));
    }
}
