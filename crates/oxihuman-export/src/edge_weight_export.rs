// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge weight export: per-edge scalar weights for graph/mesh algorithms.

/// An edge with associated weight.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightedEdge {
    pub v0: u32,
    pub v1: u32,
    pub weight: f32,
}

/// Collection of weighted edges.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeWeightExport {
    pub edges: Vec<WeightedEdge>,
}

/// Create an empty edge weight export.
#[allow(dead_code)]
pub fn new_edge_weight_export() -> EdgeWeightExport {
    EdgeWeightExport { edges: Vec::new() }
}

/// Add an edge.
#[allow(dead_code)]
pub fn add_weighted_edge(ew: &mut EdgeWeightExport, v0: u32, v1: u32, weight: f32) {
    ew.edges.push(WeightedEdge { v0, v1, weight });
}

/// Total edge count.
#[allow(dead_code)]
pub fn edge_weight_count(ew: &EdgeWeightExport) -> usize {
    ew.edges.len()
}

/// Average weight.
#[allow(dead_code)]
pub fn avg_edge_weight(ew: &EdgeWeightExport) -> f32 {
    if ew.edges.is_empty() {
        return 0.0;
    }
    ew.edges.iter().map(|e| e.weight).sum::<f32>() / ew.edges.len() as f32
}

/// Maximum weight.
#[allow(dead_code)]
pub fn max_edge_weight(ew: &EdgeWeightExport) -> f32 {
    ew.edges.iter().map(|e| e.weight).fold(0.0_f32, f32::max)
}

/// Minimum weight.
#[allow(dead_code)]
pub fn min_edge_weight(ew: &EdgeWeightExport) -> f32 {
    ew.edges.iter().map(|e| e.weight).fold(f32::MAX, f32::min)
}

/// Normalize weights so max = 1.0.
#[allow(dead_code)]
pub fn normalize_edge_weights(ew: &mut EdgeWeightExport) {
    let max_w = max_edge_weight(ew);
    if max_w < 1e-12 {
        return;
    }
    for e in &mut ew.edges {
        e.weight /= max_w;
    }
}

/// Count edges with weight above threshold.
#[allow(dead_code)]
pub fn count_heavy_edges(ew: &EdgeWeightExport, threshold: f32) -> usize {
    ew.edges.iter().filter(|e| e.weight > threshold).count()
}

/// Build from mesh edge lengths.
#[allow(dead_code)]
pub fn from_mesh_edges(positions: &[[f32; 3]], indices: &[u32]) -> EdgeWeightExport {
    let mut ew = new_edge_weight_export();
    let face_count = indices.len() / 3;
    for f in 0..face_count {
        let i0 = indices[f * 3];
        let i1 = indices[f * 3 + 1];
        let i2 = indices[f * 3 + 2];
        for (a, b) in [(i0, i1), (i1, i2), (i2, i0)] {
            let pa = positions[a as usize];
            let pb = positions[b as usize];
            let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            add_weighted_edge(&mut ew, a, b, len);
        }
    }
    ew
}

/// Export to JSON.
#[allow(dead_code)]
pub fn edge_weight_to_json(ew: &EdgeWeightExport) -> String {
    format!(
        "{{\"edge_count\":{},\"avg_weight\":{:.6}}}",
        edge_weight_count(ew),
        avg_edge_weight(ew)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_edge_weight_export() {
        let ew = new_edge_weight_export();
        assert_eq!(edge_weight_count(&ew), 0);
    }

    #[test]
    fn test_add_weighted_edge() {
        let mut ew = new_edge_weight_export();
        add_weighted_edge(&mut ew, 0, 1, 1.5);
        assert_eq!(edge_weight_count(&ew), 1);
    }

    #[test]
    fn test_avg_edge_weight() {
        let mut ew = new_edge_weight_export();
        add_weighted_edge(&mut ew, 0, 1, 2.0);
        add_weighted_edge(&mut ew, 1, 2, 4.0);
        assert!((avg_edge_weight(&ew) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_edge_weight() {
        let mut ew = new_edge_weight_export();
        add_weighted_edge(&mut ew, 0, 1, 0.5);
        add_weighted_edge(&mut ew, 1, 2, 1.5);
        assert!((max_edge_weight(&ew) - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_edge_weights() {
        let mut ew = new_edge_weight_export();
        add_weighted_edge(&mut ew, 0, 1, 2.0);
        add_weighted_edge(&mut ew, 1, 2, 4.0);
        normalize_edge_weights(&mut ew);
        assert!((max_edge_weight(&ew) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_count_heavy_edges() {
        let mut ew = new_edge_weight_export();
        add_weighted_edge(&mut ew, 0, 1, 0.5);
        add_weighted_edge(&mut ew, 1, 2, 1.5);
        assert_eq!(count_heavy_edges(&ew, 1.0), 1);
    }

    #[test]
    fn test_from_mesh_edges() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![0u32, 1, 2];
        let ew = from_mesh_edges(&positions, &indices);
        assert!(edge_weight_count(&ew) > 0);
    }

    #[test]
    fn test_edge_weight_to_json() {
        let ew = new_edge_weight_export();
        let j = edge_weight_to_json(&ew);
        assert!(j.contains("\"edge_count\":0"));
    }

    #[test]
    fn test_empty_avg() {
        let ew = new_edge_weight_export();
        assert!((avg_edge_weight(&ew)).abs() < 1e-9);
    }

    #[test]
    fn test_weight_in_range() {
        let mut ew = new_edge_weight_export();
        add_weighted_edge(&mut ew, 0, 1, 0.5);
        normalize_edge_weights(&mut ew);
        assert!((0.0..=1.0).contains(&ew.edges[0].weight));
    }
}
