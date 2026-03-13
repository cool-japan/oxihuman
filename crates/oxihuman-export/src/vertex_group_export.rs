// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Vertex group / bone weight group export.

#![allow(dead_code)]

/// A single named vertex group with per-vertex weights.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexGroupEntry {
    /// Unique name for the group.
    pub name: String,
    /// Vertex indices belonging to this group.
    pub indices: Vec<u32>,
    /// Per-vertex weights (same length as `indices`).
    pub weights: Vec<f32>,
}

/// Container for all vertex groups.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexGroupExport {
    /// All vertex groups.
    pub groups: Vec<VertexGroupEntry>,
}

/// Creates a new empty [`VertexGroupExport`].
#[allow(dead_code)]
pub fn new_vertex_group_export() -> VertexGroupExport {
    VertexGroupExport { groups: Vec::new() }
}

/// Adds a group. If a group with the same name exists, it is replaced.
#[allow(dead_code)]
pub fn vg_add_group(export: &mut VertexGroupExport, group: VertexGroupEntry) {
    if let Some(existing) = export.groups.iter_mut().find(|g| g.name == group.name) {
        *existing = group;
    } else {
        export.groups.push(group);
    }
}

/// Returns the number of groups.
#[allow(dead_code)]
pub fn vg_group_count(export: &VertexGroupExport) -> usize {
    export.groups.len()
}

/// Returns the total number of vertex-weight pairs across all groups.
#[allow(dead_code)]
pub fn vg_total_vertices(export: &VertexGroupExport) -> usize {
    export.groups.iter().map(|g| g.indices.len()).sum()
}

/// Returns a reference to the group with the given name.
#[allow(dead_code)]
pub fn vg_get_group<'a>(export: &'a VertexGroupExport, name: &str) -> Option<&'a VertexGroupEntry> {
    export.groups.iter().find(|g| g.name == name)
}

/// Removes the group with the given name. Returns true if removed.
#[allow(dead_code)]
pub fn vg_remove_group(export: &mut VertexGroupExport, name: &str) -> bool {
    let before = export.groups.len();
    export.groups.retain(|g| g.name != name);
    export.groups.len() < before
}

/// Validates all groups: checks that indices and weights have equal length.
#[allow(dead_code)]
pub fn vg_validate(export: &VertexGroupExport) -> bool {
    export.groups.iter().all(|g| g.indices.len() == g.weights.len())
}

/// Serialises to a minimal JSON string.
#[allow(dead_code)]
pub fn vg_to_json(export: &VertexGroupExport) -> String {
    format!("{{\"group_count\":{},\"total_verts\":{}}}", vg_group_count(export), vg_total_vertices(export))
}

/// Normalises weights within each group so they sum to 1.0.
#[allow(dead_code)]
pub fn vg_normalize_weights(export: &mut VertexGroupExport) {
    for group in &mut export.groups {
        let sum: f32 = group.weights.iter().sum();
        if sum > 1e-6 {
            for w in &mut group.weights {
                *w /= sum;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_group(name: &str) -> VertexGroupEntry {
        VertexGroupEntry {
            name: name.to_string(),
            indices: vec![0, 1, 2],
            weights: vec![1.0, 0.5, 0.25],
        }
    }

    #[test]
    fn test_new_export() {
        let export = new_vertex_group_export();
        assert_eq!(vg_group_count(&export), 0);
    }

    #[test]
    fn test_add_group() {
        let mut export = new_vertex_group_export();
        vg_add_group(&mut export, make_group("arm"));
        assert_eq!(vg_group_count(&export), 1);
    }

    #[test]
    fn test_total_vertices() {
        let mut export = new_vertex_group_export();
        vg_add_group(&mut export, make_group("arm"));
        assert_eq!(vg_total_vertices(&export), 3);
    }

    #[test]
    fn test_get_group() {
        let mut export = new_vertex_group_export();
        vg_add_group(&mut export, make_group("leg"));
        assert!(vg_get_group(&export, "leg").is_some());
        assert!(vg_get_group(&export, "head").is_none());
    }

    #[test]
    fn test_remove_group() {
        let mut export = new_vertex_group_export();
        vg_add_group(&mut export, make_group("torso"));
        assert!(vg_remove_group(&mut export, "torso"));
        assert_eq!(vg_group_count(&export), 0);
    }

    #[test]
    fn test_validate_valid() {
        let mut export = new_vertex_group_export();
        vg_add_group(&mut export, make_group("arm"));
        assert!(vg_validate(&export));
    }

    #[test]
    fn test_to_json() {
        let export = new_vertex_group_export();
        let json = vg_to_json(&export);
        assert!(json.contains("group_count"));
    }

    #[test]
    fn test_normalize_weights() {
        let mut export = new_vertex_group_export();
        vg_add_group(&mut export, VertexGroupEntry { name: "g".to_string(), indices: vec![0, 1], weights: vec![2.0, 2.0] });
        vg_normalize_weights(&mut export);
        let sum: f32 = export.groups[0].weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_replace_group_same_name() {
        let mut export = new_vertex_group_export();
        vg_add_group(&mut export, make_group("arm"));
        vg_add_group(&mut export, make_group("arm"));
        assert_eq!(vg_group_count(&export), 1);
    }
}
