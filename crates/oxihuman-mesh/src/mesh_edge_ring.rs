// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0 / #![allow(dead_code)]

/// An edge defined by two vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshEdgeRing {
    pub v0: u32,
    pub v1: u32,
}

#[allow(dead_code)]
impl MeshEdgeRing {
    pub fn new(v0: u32, v1: u32) -> Self {
        if v0 < v1 { Self { v0, v1 } } else { Self { v0: v1, v1: v0 } }
    }
}

/// Extract all unique edges from triangle indices.
#[allow(dead_code)]
pub fn extract_all_edges(indices: &[u32]) -> Vec<MeshEdgeRing> {
    let mut edges = Vec::new();
    for tri in indices.chunks_exact(3) {
        for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let e = MeshEdgeRing::new(a, b);
            if !edges.contains(&e) {
                edges.push(e);
            }
        }
    }
    edges
}

/// Find the edge ring starting from a given edge, following quads.
#[allow(dead_code)]
pub fn find_edge_ring(indices: &[u32], start_edge: MeshEdgeRing) -> Vec<MeshEdgeRing> {
    let mut ring = vec![start_edge];
    let mut current = start_edge;
    for _ in 0..indices.len() {
        let mut found = false;
        for tri in indices.chunks_exact(3) {
            let t = [tri[0], tri[1], tri[2]];
            if t.contains(&current.v0) && t.contains(&current.v1) {
                let other = t.iter().find(|&&v| v != current.v0 && v != current.v1);
                if let Some(&o) = other {
                    let next = MeshEdgeRing::new(current.v0, o);
                    if !ring.contains(&next) {
                        ring.push(next);
                        current = next;
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            break;
        }
    }
    ring
}

/// Count unique edges.
#[allow(dead_code)]
pub fn edge_count(indices: &[u32]) -> usize {
    extract_all_edges(indices).len()
}

/// Check if two vertices share an edge.
#[allow(dead_code)]
pub fn shares_edge(indices: &[u32], v0: u32, v1: u32) -> bool {
    let e = MeshEdgeRing::new(v0, v1);
    extract_all_edges(indices).contains(&e)
}

/// Get edges adjacent to a vertex.
#[allow(dead_code)]
pub fn edges_at_vertex(indices: &[u32], vertex: u32) -> Vec<MeshEdgeRing> {
    extract_all_edges(indices)
        .into_iter()
        .filter(|e| e.v0 == vertex || e.v1 == vertex)
        .collect()
}

/// Serialize edge list to JSON.
#[allow(dead_code)]
pub fn edges_to_json(edges: &[MeshEdgeRing]) -> String {
    let mut s = String::from("[");
    for (i, e) in edges.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&format!("[{},{}]", e.v0, e.v1));
    }
    s.push(']');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_new_sorted() {
        let e = MeshEdgeRing::new(5, 2);
        assert_eq!(e.v0, 2);
        assert_eq!(e.v1, 5);
    }

    #[test]
    fn test_extract_edges_tri() {
        let edges = extract_all_edges(&[0, 1, 2]);
        assert_eq!(edges.len(), 3);
    }

    #[test]
    fn test_extract_edges_quad() {
        let edges = extract_all_edges(&[0, 1, 2, 0, 2, 3]);
        assert_eq!(edges.len(), 5);
    }

    #[test]
    fn test_edge_count() {
        assert_eq!(edge_count(&[0, 1, 2]), 3);
    }

    #[test]
    fn test_shares_edge() {
        assert!(shares_edge(&[0, 1, 2], 0, 1));
        assert!(!shares_edge(&[0, 1, 2], 0, 5));
    }

    #[test]
    fn test_edges_at_vertex() {
        let edges = edges_at_vertex(&[0, 1, 2], 0);
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_find_edge_ring() {
        let ring = find_edge_ring(&[0, 1, 2], MeshEdgeRing::new(0, 1));
        assert!(!ring.is_empty());
    }

    #[test]
    fn test_empty() {
        let edges = extract_all_edges(&[]);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_edges_to_json() {
        let edges = extract_all_edges(&[0, 1, 2]);
        let json = edges_to_json(&edges);
        assert!(json.starts_with('['));
    }

    #[test]
    fn test_edge_equality() {
        let e1 = MeshEdgeRing::new(0, 1);
        let e2 = MeshEdgeRing::new(1, 0);
        assert_eq!(e1, e2);
    }
}
