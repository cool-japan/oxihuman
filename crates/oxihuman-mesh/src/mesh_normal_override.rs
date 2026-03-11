// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-vertex normal override layer.

/// Stores overridden normals per vertex index.
pub struct NormalOverrideLayer {
    pub vertex_indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
}

/// Create a new empty normal override layer.
pub fn new_normal_override_layer() -> NormalOverrideLayer {
    NormalOverrideLayer {
        vertex_indices: Vec::new(),
        normals: Vec::new(),
    }
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Set an overridden normal for a vertex (normalised automatically).
pub fn set_override_normal(layer: &mut NormalOverrideLayer, vertex: u32, normal: [f32; 3]) {
    let n = norm3(normal);
    if let Some(pos) = layer.vertex_indices.iter().position(|&v| v == vertex) {
        layer.normals[pos] = n;
    } else {
        layer.vertex_indices.push(vertex);
        layer.normals.push(n);
    }
}

/// Get the overridden normal for a vertex; None if not overridden.
pub fn get_override_normal(layer: &NormalOverrideLayer, vertex: u32) -> Option<[f32; 3]> {
    layer
        .vertex_indices
        .iter()
        .position(|&v| v == vertex)
        .map(|i| layer.normals[i])
}

/// Remove override for a vertex; returns true if it existed.
pub fn remove_override(layer: &mut NormalOverrideLayer, vertex: u32) -> bool {
    if let Some(pos) = layer.vertex_indices.iter().position(|&v| v == vertex) {
        layer.vertex_indices.remove(pos);
        layer.normals.remove(pos);
        true
    } else {
        false
    }
}

/// Number of overridden vertices.
pub fn override_count(layer: &NormalOverrideLayer) -> usize {
    layer.vertex_indices.len()
}

/// Validate all stored normals are unit length.
pub fn validate_overrides(layer: &NormalOverrideLayer) -> bool {
    layer.normals.iter().all(|n| {
        let sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
        (sq - 1.0).abs() < 1e-4
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_layer_empty() {
        let l = new_normal_override_layer();
        assert_eq!(override_count(&l), 0 /* empty */);
    }

    #[test]
    fn set_and_get() {
        let mut l = new_normal_override_layer();
        set_override_normal(&mut l, 0, [0.0, 1.0, 0.0]);
        let n = get_override_normal(&l, 0).unwrap();
        assert!((n[1] - 1.0).abs() < 1e-6 /* Y up */);
    }

    #[test]
    fn overwrite_does_not_duplicate() {
        let mut l = new_normal_override_layer();
        set_override_normal(&mut l, 5, [1.0, 0.0, 0.0]);
        set_override_normal(&mut l, 5, [0.0, 0.0, 1.0]);
        assert_eq!(override_count(&l), 1 /* one entry */);
    }

    #[test]
    fn get_missing_none() {
        let l = new_normal_override_layer();
        assert!(get_override_normal(&l, 99).is_none() /* not found */);
    }

    #[test]
    fn remove_present_returns_true() {
        let mut l = new_normal_override_layer();
        set_override_normal(&mut l, 3, [0.0, 1.0, 0.0]);
        assert!(remove_override(&mut l, 3) /* removed */);
        assert_eq!(override_count(&l), 0 /* empty */);
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut l = new_normal_override_layer();
        assert!(!remove_override(&mut l, 7) /* not found */);
    }

    #[test]
    fn normals_are_normalised() {
        let mut l = new_normal_override_layer();
        set_override_normal(&mut l, 0, [5.0, 0.0, 0.0]);
        let n = get_override_normal(&l, 0).unwrap();
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-6 /* unit length */);
    }

    #[test]
    fn validate_all_valid() {
        let mut l = new_normal_override_layer();
        set_override_normal(&mut l, 0, [1.0, 0.0, 0.0]);
        set_override_normal(&mut l, 1, [0.0, 1.0, 0.0]);
        assert!(validate_overrides(&l) /* valid */);
    }

    #[test]
    fn multiple_vertices_independent() {
        let mut l = new_normal_override_layer();
        for i in 0u32..5 {
            set_override_normal(&mut l, i, [1.0, 0.0, 0.0]);
        }
        assert_eq!(override_count(&l), 5 /* five entries */);
    }
}
