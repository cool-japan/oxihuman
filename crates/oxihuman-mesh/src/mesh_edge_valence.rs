// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Edge valence analysis: count how many faces share each edge.

use std::collections::HashMap;

/// Edge valence info.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeValenceResult {
    /// Map from canonical edge (min, max) to face count.
    pub valences: HashMap<(u32, u32), usize>,
}

/// Build edge valence map.
#[allow(dead_code)]
pub fn compute_edge_valences(indices: &[u32]) -> EdgeValenceResult {
    let mut valences: HashMap<(u32, u32), usize> = HashMap::new();
    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let vs = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for k in 0..3 {
            let a = vs[k];
            let b = vs[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            *valences.entry(key).or_insert(0) += 1;
        }
    }
    EdgeValenceResult { valences }
}

/// Total unique edge count.
#[allow(dead_code)]
pub fn total_edge_count(result: &EdgeValenceResult) -> usize {
    result.valences.len()
}

/// Get valence for a specific edge.
#[allow(dead_code)]
pub fn edge_valence(result: &EdgeValenceResult, a: u32, b: u32) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };
    result.valences.get(&key).copied().unwrap_or(0)
}

/// Count boundary edges (valence == 1).
#[allow(dead_code)]
pub fn boundary_edge_count(result: &EdgeValenceResult) -> usize {
    result.valences.values().filter(|&&v| v == 1).count()
}

/// Count manifold edges (valence == 2).
#[allow(dead_code)]
pub fn manifold_edge_count(result: &EdgeValenceResult) -> usize {
    result.valences.values().filter(|&&v| v == 2).count()
}

/// Count non-manifold edges (valence > 2).
#[allow(dead_code)]
pub fn non_manifold_edge_count(result: &EdgeValenceResult) -> usize {
    result.valences.values().filter(|&&v| v > 2).count()
}

/// Maximum valence across all edges.
#[allow(dead_code)]
pub fn max_valence(result: &EdgeValenceResult) -> usize {
    result.valences.values().copied().max().unwrap_or(0)
}

/// Average valence.
#[allow(dead_code)]
pub fn avg_valence(result: &EdgeValenceResult) -> f32 {
    if result.valences.is_empty() {
        return 0.0;
    }
    let sum: usize = result.valences.values().sum();
    sum as f32 / result.valences.len() as f32
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn edge_valence_to_json(result: &EdgeValenceResult) -> String {
    format!(
        "{{\"edges\":{},\"boundary\":{},\"manifold\":{},\"non_manifold\":{}}}",
        total_edge_count(result),
        boundary_edge_count(result),
        manifold_edge_count(result),
        non_manifold_edge_count(result),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tri() -> Vec<u32> {
        vec![0, 1, 2]
    }
    fn two_tris() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }

    #[test]
    fn test_single_tri_edges() {
        let r = compute_edge_valences(&single_tri());
        assert_eq!(total_edge_count(&r), 3);
    }

    #[test]
    fn test_single_tri_all_boundary() {
        let r = compute_edge_valences(&single_tri());
        assert_eq!(boundary_edge_count(&r), 3);
    }

    #[test]
    fn test_two_tris_shared_edge() {
        let r = compute_edge_valences(&two_tris());
        assert_eq!(edge_valence(&r, 1, 2), 2);
    }

    #[test]
    fn test_manifold_count() {
        let r = compute_edge_valences(&two_tris());
        assert_eq!(manifold_edge_count(&r), 1);
    }

    #[test]
    fn test_non_manifold_empty() {
        let r = compute_edge_valences(&two_tris());
        assert_eq!(non_manifold_edge_count(&r), 0);
    }

    #[test]
    fn test_max_valence() {
        let r = compute_edge_valences(&two_tris());
        assert_eq!(max_valence(&r), 2);
    }

    #[test]
    fn test_avg_valence() {
        let r = compute_edge_valences(&single_tri());
        assert!((avg_valence(&r) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_empty() {
        let r = compute_edge_valences(&[]);
        assert_eq!(total_edge_count(&r), 0);
        assert_eq!(max_valence(&r), 0);
    }

    #[test]
    fn test_edge_valence_nonexistent() {
        let r = compute_edge_valences(&single_tri());
        assert_eq!(edge_valence(&r, 10, 20), 0);
    }

    #[test]
    fn test_edge_valence_to_json() {
        let r = compute_edge_valences(&single_tri());
        let json = edge_valence_to_json(&r);
        assert!(json.contains("\"edges\":3"));
    }
}
