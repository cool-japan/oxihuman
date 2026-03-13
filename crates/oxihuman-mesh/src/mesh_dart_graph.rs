// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Dart (half-edge-like) graph representation for combinatorial mesh topology.

/// A dart in the combinatorial map.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Dart {
    pub vertex: u32,
    pub face: u32,
    pub twin: usize,
    pub next: usize,
}

/// Dart graph for a triangle mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DartGraph {
    pub darts: Vec<Dart>,
}

/// Build a dart graph from triangle indices.
#[allow(dead_code)]
pub fn build_dart_graph(indices: &[u32]) -> DartGraph {
    let tri_count = indices.len() / 3;
    let dart_count = tri_count * 3;
    let mut darts = Vec::with_capacity(dart_count);

    for t in 0..tri_count {
        let base = t * 3;
        let f = t as u32;
        for k in 0..3 {
            darts.push(Dart {
                vertex: indices[base + k],
                face: f,
                twin: usize::MAX,
                next: base + (k + 1) % 3,
            });
        }
    }

    // Collect (vertex, next_vertex) pairs for twin matching
    use std::collections::HashMap;
    let pairs: Vec<(u32, u32)> = (0..darts.len())
        .map(|i| {
            let next_vert = darts[darts[i].next].vertex;
            (darts[i].vertex, next_vert)
        })
        .collect();

    let mut edge_map: HashMap<(u32, u32), usize> = HashMap::new();
    let mut twins = vec![usize::MAX; darts.len()];
    for (i, &(v, nv)) in pairs.iter().enumerate() {
        let twin_key = (nv, v);
        if let Some(&twin_idx) = edge_map.get(&twin_key) {
            twins[i] = twin_idx;
            twins[twin_idx] = i;
        }
        edge_map.insert((v, nv), i);
    }
    for (i, t) in twins.into_iter().enumerate() {
        darts[i].twin = t;
    }

    DartGraph { darts }
}

/// Total dart count.
#[allow(dead_code)]
pub fn dart_count(graph: &DartGraph) -> usize {
    graph.darts.len()
}

/// Get dart at index.
#[allow(dead_code)]
pub fn get_dart(graph: &DartGraph, idx: usize) -> Option<&Dart> {
    graph.darts.get(idx)
}

/// Check if a dart has a twin (is an interior edge).
#[allow(dead_code)]
pub fn has_twin(graph: &DartGraph, idx: usize) -> bool {
    graph.darts.get(idx).is_some_and(|d| d.twin != usize::MAX)
}

/// Count boundary darts (no twin).
#[allow(dead_code)]
pub fn boundary_dart_count(graph: &DartGraph) -> usize {
    graph.darts.iter().filter(|d| d.twin == usize::MAX).count()
}

/// Face count.
#[allow(dead_code)]
pub fn face_count(graph: &DartGraph) -> usize {
    graph.darts.len() / 3
}

/// Convert to JSON.
#[allow(dead_code)]
pub fn dart_graph_to_json(graph: &DartGraph) -> String {
    format!(
        "{{\"dart_count\":{},\"face_count\":{},\"boundary_darts\":{}}}",
        dart_count(graph),
        face_count(graph),
        boundary_dart_count(graph)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_single_triangle() {
        let g = build_dart_graph(&[0, 1, 2]);
        assert_eq!(dart_count(&g), 3);
    }

    #[test]
    fn test_face_count() {
        let g = build_dart_graph(&[0, 1, 2, 1, 3, 2]);
        assert_eq!(face_count(&g), 2);
    }

    #[test]
    fn test_boundary_darts_single() {
        let g = build_dart_graph(&[0, 1, 2]);
        assert_eq!(boundary_dart_count(&g), 3); // all boundary
    }

    #[test]
    fn test_twin_pairing() {
        let g = build_dart_graph(&[0, 1, 2, 2, 1, 3]);
        // Edge 1-2 should have twins
        let paired = g.darts.iter().filter(|d| d.twin != usize::MAX).count();
        assert!(paired >= 2);
    }

    #[test]
    fn test_get_dart() {
        let g = build_dart_graph(&[0, 1, 2]);
        assert!(get_dart(&g, 0).is_some());
        assert!(get_dart(&g, 10).is_none());
    }

    #[test]
    fn test_has_twin() {
        let g = build_dart_graph(&[0, 1, 2]);
        assert!(!has_twin(&g, 0));
    }

    #[test]
    fn test_empty() {
        let g = build_dart_graph(&[]);
        assert_eq!(dart_count(&g), 0);
    }

    #[test]
    fn test_dart_vertex() {
        let g = build_dart_graph(&[5, 10, 15]);
        assert_eq!(g.darts[0].vertex, 5);
        assert_eq!(g.darts[1].vertex, 10);
    }

    #[test]
    fn test_next_cycling() {
        let g = build_dart_graph(&[0, 1, 2]);
        let d0 = &g.darts[0];
        let d1 = &g.darts[d0.next];
        let d2 = &g.darts[d1.next];
        assert_eq!(d2.next, 0);
    }

    #[test]
    fn test_to_json() {
        let g = build_dart_graph(&[0, 1, 2]);
        let j = dart_graph_to_json(&g);
        assert!(j.contains("\"dart_count\":3"));
    }
}
