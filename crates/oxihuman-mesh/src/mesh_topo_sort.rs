// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Topological sorting of mesh faces and vertices based on dependency graphs.

use std::collections::VecDeque;

/// Result of a topological sort.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TopoSortResult {
    pub order: Vec<usize>,
    pub has_cycle: bool,
}

/// Build a forward adjacency list from directed edges.
#[allow(dead_code)]
pub fn build_dag(node_count: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut adj = vec![vec![]; node_count];
    for &(u, v) in edges {
        adj[u].push(v);
    }
    adj
}

/// Compute in-degree for each node.
#[allow(dead_code)]
pub fn compute_in_degree(node_count: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let mut deg = vec![0usize; node_count];
    for &(_, v) in edges {
        deg[v] += 1;
    }
    deg
}

/// Kahn's algorithm for topological sort; detects cycles.
#[allow(dead_code)]
pub fn topo_sort(node_count: usize, edges: &[(usize, usize)]) -> TopoSortResult {
    let adj = build_dag(node_count, edges);
    let mut in_deg = compute_in_degree(node_count, edges);
    let mut queue: VecDeque<usize> = (0..node_count).filter(|&i| in_deg[i] == 0).collect();
    let mut order = Vec::with_capacity(node_count);

    while let Some(u) = queue.pop_front() {
        order.push(u);
        for &v in &adj[u] {
            in_deg[v] -= 1;
            if in_deg[v] == 0 {
                queue.push_back(v);
            }
        }
    }

    let has_cycle = order.len() != node_count;
    TopoSortResult { order, has_cycle }
}

/// Sort mesh faces by their dependency on shared edges (faces sharing an edge
/// are treated as dependent on lower-index faces first).
#[allow(dead_code)]
pub fn sort_faces_by_edge_order(face_count: usize, face_indices: &[u32]) -> Vec<usize> {
    // Build a simple face -> face dependency: a face depends on all faces
    // with smaller index that share a vertex.
    use std::collections::HashMap;
    let mut vertex_to_faces: HashMap<u32, Vec<usize>> = HashMap::new();
    for (fi, tri) in face_indices.chunks_exact(3).enumerate() {
        for &v in tri {
            vertex_to_faces.entry(v).or_default().push(fi);
        }
    }
    let mut edges: Vec<(usize, usize)> = Vec::new();
    for faces in vertex_to_faces.values() {
        for &a in faces {
            for &b in faces {
                if a < b {
                    edges.push((a, b));
                }
            }
        }
    }
    edges.sort_unstable();
    edges.dedup();
    let res = topo_sort(face_count, &edges);
    res.order
}

/// Reverse a topological order.
#[allow(dead_code)]
pub fn reverse_topo(order: &[usize]) -> Vec<usize> {
    let mut rev = order.to_vec();
    rev.reverse();
    rev
}

/// Check whether the sorted order covers all nodes.
#[allow(dead_code)]
pub fn order_is_complete(order: &[usize], node_count: usize) -> bool {
    order.len() == node_count
}

/// Assign a depth level to each node in a DAG (longest path from sources).
#[allow(dead_code)]
pub fn assign_levels(node_count: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let adj = build_dag(node_count, edges);
    let res = topo_sort(node_count, edges);
    let mut levels = vec![0usize; node_count];
    for &u in &res.order {
        for &v in &adj[u] {
            if levels[v] < levels[u] + 1 {
                levels[v] = levels[u] + 1;
            }
        }
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_chain() {
        let res = topo_sort(3, &[(0, 1), (1, 2)]);
        assert!(!res.has_cycle);
        assert_eq!(res.order, vec![0, 1, 2]);
    }

    #[test]
    fn test_cycle_detected() {
        let res = topo_sort(3, &[(0, 1), (1, 2), (2, 0)]);
        assert!(res.has_cycle);
    }

    #[test]
    fn test_no_edges() {
        let res = topo_sort(4, &[]);
        assert!(!res.has_cycle);
        assert!(order_is_complete(&res.order, 4));
    }

    #[test]
    fn test_in_degree() {
        let deg = compute_in_degree(3, &[(0, 1), (0, 2), (1, 2)]);
        assert_eq!(deg[0], 0);
        assert_eq!(deg[2], 2);
    }

    #[test]
    fn test_reverse_topo() {
        let order = vec![0, 1, 2];
        let rev = reverse_topo(&order);
        assert_eq!(rev, vec![2, 1, 0]);
    }

    #[test]
    fn test_order_complete() {
        let res = topo_sort(5, &[(0, 1), (2, 3), (3, 4)]);
        assert!(order_is_complete(&res.order, 5));
    }

    #[test]
    fn test_assign_levels_chain() {
        let levels = assign_levels(3, &[(0, 1), (1, 2)]);
        assert_eq!(levels[0], 0);
        assert_eq!(levels[1], 1);
        assert_eq!(levels[2], 2);
    }

    #[test]
    fn test_sort_faces_basic() {
        // Two triangles sharing no vertices
        let order = sort_faces_by_edge_order(2, &[0, 1, 2, 3, 4, 5]);
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn test_build_dag_empty() {
        let adj = build_dag(3, &[]);
        assert!(adj.iter().all(|v| v.is_empty()));
    }

    #[test]
    fn test_diamond_dag() {
        // 0->1, 0->2, 1->3, 2->3
        let res = topo_sort(4, &[(0, 1), (0, 2), (1, 3), (2, 3)]);
        assert!(!res.has_cycle);
        let pos: Vec<usize> = res.order.to_vec();
        let idx = |x: usize| pos.iter().position(|&v| v == x).expect("should succeed");
        assert!(idx(0) < idx(1));
        assert!(idx(0) < idx(2));
        assert!(idx(1) < idx(3));
        assert!(idx(2) < idx(3));
    }
}
