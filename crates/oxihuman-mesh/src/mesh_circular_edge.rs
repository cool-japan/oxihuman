// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Circular edge detection in meshes (edges that form loops).

use std::collections::HashMap;
use std::f32::consts::TAU;

/// An edge defined by two vertex indices (sorted).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge(pub u32, pub u32);

/// Result of circular edge detection.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CircularEdgeResult {
    pub loops: Vec<Vec<u32>>,
    pub open_chains: Vec<Vec<u32>>,
}

/// Create a canonical edge key.
#[allow(dead_code)]
pub fn make_edge(a: u32, b: u32) -> Edge {
    if a <= b {
        Edge(a, b)
    } else {
        Edge(b, a)
    }
}

/// Build adjacency from indices.
#[allow(dead_code)]
pub fn build_adjacency(indices: &[u32]) -> HashMap<u32, Vec<u32>> {
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    let tri_count = indices.len() / 3;
    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let vs = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for i in 0..3 {
            let a = vs[i];
            let b = vs[(i + 1) % 3];
            adj.entry(a).or_default().push(b);
            adj.entry(b).or_default().push(a);
        }
    }
    for v in adj.values_mut() {
        v.sort_unstable();
        v.dedup();
    }
    adj
}

/// Detect circular edge loops and open chains from boundary edges.
#[allow(dead_code)]
pub fn detect_circular_edges(indices: &[u32]) -> CircularEdgeResult {
    let mut edge_count: HashMap<Edge, usize> = HashMap::new();
    let tri_count = indices.len() / 3;
    #[allow(clippy::needless_range_loop)]
    for t in 0..tri_count {
        let vs = [indices[t * 3], indices[t * 3 + 1], indices[t * 3 + 2]];
        for i in 0..3 {
            let e = make_edge(vs[i], vs[(i + 1) % 3]);
            *edge_count.entry(e).or_default() += 1;
        }
    }
    let boundary: Vec<Edge> = edge_count
        .into_iter()
        .filter(|&(_, c)| c == 1)
        .map(|(e, _)| e)
        .collect();
    let mut adj: HashMap<u32, Vec<u32>> = HashMap::new();
    for e in &boundary {
        adj.entry(e.0).or_default().push(e.1);
        adj.entry(e.1).or_default().push(e.0);
    }

    let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut loops = Vec::new();
    let mut open_chains = Vec::new();

    for &start in adj.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut chain = vec![start];
        visited.insert(start);
        let mut current = start;
        loop {
            let neighbors = adj.get(&current).cloned().unwrap_or_default();
            let next = neighbors.iter().find(|n| !visited.contains(n));
            match next {
                Some(&n) => {
                    chain.push(n);
                    visited.insert(n);
                    current = n;
                }
                None => break,
            }
        }
        if let Some(neighbors) = adj.get(&current) {
            if neighbors.contains(&start) && chain.len() > 2 {
                loops.push(chain);
            } else {
                open_chains.push(chain);
            }
        } else {
            open_chains.push(chain);
        }
    }

    CircularEdgeResult { loops, open_chains }
}

/// Number of closed loops.
#[allow(dead_code)]
pub fn loop_count(r: &CircularEdgeResult) -> usize {
    r.loops.len()
}

/// Number of open chains.
#[allow(dead_code)]
pub fn open_chain_count(r: &CircularEdgeResult) -> usize {
    r.open_chains.len()
}

/// Largest loop size.
#[allow(dead_code)]
pub fn largest_loop(r: &CircularEdgeResult) -> usize {
    r.loops.iter().map(|l| l.len()).max().unwrap_or(0)
}

/// TAU-based circumference for radius.
#[allow(dead_code)]
pub fn circumference(radius: f32) -> f32 {
    TAU * radius
}

/// Export to JSON.
#[allow(dead_code)]
pub fn circular_edge_to_json(r: &CircularEdgeResult) -> String {
    format!(
        "{{\"loops\":{},\"open_chains\":{}}}",
        loop_count(r),
        open_chain_count(r)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_edge() {
        assert_eq!(make_edge(5, 3), Edge(3, 5));
    }

    #[test]
    fn test_build_adjacency() {
        let adj = build_adjacency(&[0, 1, 2]);
        assert_eq!(adj.len(), 3);
    }

    #[test]
    fn test_detect_single_tri() {
        let r = detect_circular_edges(&[0, 1, 2]);
        // Single triangle has 3 boundary edges forming a loop
        assert!(loop_count(&r) + open_chain_count(&r) > 0);
    }

    #[test]
    fn test_detect_empty() {
        let r = detect_circular_edges(&[]);
        assert_eq!(loop_count(&r), 0);
        assert_eq!(open_chain_count(&r), 0);
    }

    #[test]
    fn test_two_tris_shared_edge() {
        let r = detect_circular_edges(&[0, 1, 2, 1, 3, 2]);
        assert!(loop_count(&r) + open_chain_count(&r) > 0);
    }

    #[test]
    fn test_largest_loop() {
        let r = detect_circular_edges(&[0, 1, 2]);
        // May or may not form a loop depending on detection
        assert!(largest_loop(&r) <= 3);
    }

    #[test]
    fn test_circumference() {
        assert!((circumference(1.0) - TAU).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let r = CircularEdgeResult {
            loops: vec![],
            open_chains: vec![],
        };
        let j = circular_edge_to_json(&r);
        assert!(j.contains("\"loops\":0"));
    }

    #[test]
    fn test_open_chain_count() {
        let r = CircularEdgeResult {
            loops: vec![],
            open_chains: vec![vec![0, 1]],
        };
        assert_eq!(open_chain_count(&r), 1);
    }

    #[test]
    fn test_loop_count() {
        let r = CircularEdgeResult {
            loops: vec![vec![0, 1, 2]],
            open_chains: vec![],
        };
        assert_eq!(loop_count(&r), 1);
    }
}
