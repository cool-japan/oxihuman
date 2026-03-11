// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Export deformer weight maps (e.g. lattice FFD, cage, proximity weights).

/// A named deformer weight map.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DeformerWeightMap {
    pub deformer_name: String,
    pub weights: Vec<f32>,
}

/// A collection of deformer weight maps for a mesh.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DeformWeightsExport {
    pub vertex_count: usize,
    pub maps: Vec<DeformerWeightMap>,
}

/// Create a new export structure.
#[allow(dead_code)]
pub fn new_deform_weights_export(vertex_count: usize) -> DeformWeightsExport {
    DeformWeightsExport {
        vertex_count,
        maps: vec![],
    }
}

/// Add a weight map.
#[allow(dead_code)]
pub fn add_weight_map(export: &mut DeformWeightsExport, name: &str, weights: Vec<f32>) {
    export.maps.push(DeformerWeightMap {
        deformer_name: name.to_string(),
        weights,
    });
}

/// Find a weight map by deformer name.
#[allow(dead_code)]
pub fn find_map<'a>(export: &'a DeformWeightsExport, name: &str) -> Option<&'a DeformerWeightMap> {
    export.maps.iter().find(|m| m.deformer_name == name)
}

/// Get weight for a specific vertex from a named deformer.
#[allow(dead_code)]
pub fn vertex_weight(export: &DeformWeightsExport, name: &str, vertex: usize) -> f32 {
    find_map(export, name)
        .and_then(|m| m.weights.get(vertex).cloned())
        .unwrap_or(0.0)
}

/// Normalise each vertex's weight across all deformers so they sum to 1.
#[allow(dead_code)]
pub fn normalise_across_deformers(export: &mut DeformWeightsExport) {
    let n = export.vertex_count;
    for vi in 0..n {
        let total: f32 = export
            .maps
            .iter()
            .map(|m| m.weights.get(vi).cloned().unwrap_or(0.0))
            .sum();
        if total > 1e-8 {
            for m in &mut export.maps {
                if let Some(w) = m.weights.get_mut(vi) {
                    *w /= total;
                }
            }
        }
    }
}

/// Serialise all maps to a flat buffer (interleaved per vertex).
#[allow(dead_code)]
pub fn serialise_deform_weights(export: &DeformWeightsExport) -> Vec<f32> {
    let mut buf = Vec::with_capacity(export.vertex_count * export.maps.len());
    for vi in 0..export.vertex_count {
        for m in &export.maps {
            buf.push(m.weights.get(vi).cloned().unwrap_or(0.0));
        }
    }
    buf
}

/// Count deformers.
#[allow(dead_code)]
pub fn deformer_count(export: &DeformWeightsExport) -> usize {
    export.maps.len()
}

/// Clamp all weights to [0, 1].
#[allow(dead_code)]
pub fn clamp_all_weights(export: &mut DeformWeightsExport) {
    for m in &mut export.maps {
        for w in &mut m.weights {
            *w = w.clamp(0.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_deformer_export() -> DeformWeightsExport {
        let mut e = new_deform_weights_export(3);
        add_weight_map(&mut e, "ffd", vec![1.0, 0.5, 0.0]);
        add_weight_map(&mut e, "cage", vec![0.0, 0.5, 1.0]);
        e
    }

    #[test]
    fn test_new_export() {
        let e = new_deform_weights_export(5);
        assert_eq!(e.vertex_count, 5);
        assert!(e.maps.is_empty());
    }

    #[test]
    fn test_add_map() {
        let e = two_deformer_export();
        assert_eq!(deformer_count(&e), 2);
    }

    #[test]
    fn test_find_map_found() {
        let e = two_deformer_export();
        assert!(find_map(&e, "ffd").is_some());
    }

    #[test]
    fn test_find_map_not_found() {
        let e = two_deformer_export();
        assert!(find_map(&e, "lattice").is_none());
    }

    #[test]
    fn test_vertex_weight() {
        let e = two_deformer_export();
        assert!((vertex_weight(&e, "ffd", 0) - 1.0).abs() < 1e-6);
        assert!((vertex_weight(&e, "cage", 2) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalise_across_deformers() {
        let mut e = two_deformer_export();
        normalise_across_deformers(&mut e);
        let sum_v1: f32 = e.maps.iter().map(|m| m.weights[1]).sum();
        assert!((sum_v1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_serialise_length() {
        let e = two_deformer_export();
        // 3 vertices * 2 deformers = 6
        assert_eq!(serialise_deform_weights(&e).len(), 6);
    }

    #[test]
    fn test_clamp_all_weights() {
        let mut e = new_deform_weights_export(2);
        add_weight_map(&mut e, "d", vec![-0.5, 1.5]);
        clamp_all_weights(&mut e);
        assert!((0.0..=1.0).contains(&e.maps[0].weights[0]));
        assert!((0.0..=1.0).contains(&e.maps[0].weights[1]));
    }

    #[test]
    fn test_vertex_weight_missing_deformer() {
        let e = two_deformer_export();
        assert_eq!(vertex_weight(&e, "none", 0), 0.0);
    }

    #[test]
    fn test_vertex_weight_out_of_bounds() {
        let e = two_deformer_export();
        assert_eq!(vertex_weight(&e, "ffd", 999), 0.0);
    }
}
