#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Graph partitioning utilities (stub/simple implementation).

/// A single partition containing node IDs.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Partition {
    pub id: usize,
    pub nodes: Vec<usize>,
}

/// Result of a graph partition operation.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct PartitionResult {
    pub partitions: Vec<Partition>,
    pub edge_cut: usize,
}

/// Partition a graph (given as adjacency list) into `k` balanced parts using round-robin assignment.
#[allow(dead_code)]
pub fn partition_graph(node_count: usize, _edges: &[(usize, usize)], k: usize) -> PartitionResult {
    let k = k.max(1);
    let mut parts: Vec<Partition> = (0..k).map(|i| Partition { id: i, nodes: Vec::new() }).collect();
    for n in 0..node_count {
        parts[n % k].nodes.push(n);
    }
    // Count edges that cross partition boundaries
    let node_part: Vec<usize> = (0..node_count).map(|n| n % k).collect();
    let edge_cut = _edges.iter().filter(|(a, b)| {
        if *a < node_count && *b < node_count {
            node_part[*a] != node_part[*b]
        } else {
            false
        }
    }).count();
    PartitionResult { partitions: parts, edge_cut }
}

/// Return the number of partitions.
#[allow(dead_code)]
pub fn partition_count(result: &PartitionResult) -> usize {
    result.partitions.len()
}

/// Return the total number of nodes across all partitions.
#[allow(dead_code)]
pub fn partition_node_count(result: &PartitionResult) -> usize {
    result.partitions.iter().map(|p| p.nodes.len()).sum()
}

/// Return the edge cut count.
#[allow(dead_code)]
pub fn partition_edge_cut(result: &PartitionResult) -> usize {
    result.edge_cut
}

/// Rebalance partitions (no-op stub; returns clone).
#[allow(dead_code)]
pub fn rebalance_partition(result: &PartitionResult) -> PartitionResult {
    result.clone()
}

/// Convert partitions to a flat `Vec<(node, partition_id)>`.
#[allow(dead_code)]
pub fn partition_to_vec(result: &PartitionResult) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for p in &result.partitions {
        for &n in &p.nodes {
            out.push((n, p.id));
        }
    }
    out
}

/// Merge two `PartitionResult`s into one.
#[allow(dead_code)]
pub fn merge_partitions(a: &PartitionResult, b: &PartitionResult) -> PartitionResult {
    let mut parts = a.partitions.clone();
    for p in &b.partitions {
        let mut p2 = p.clone();
        p2.id += a.partitions.len();
        parts.push(p2);
    }
    PartitionResult { partitions: parts, edge_cut: a.edge_cut + b.edge_cut }
}

/// Compute a simple quality score: 1.0 / (1 + edge_cut).
#[allow(dead_code)]
pub fn partition_quality(result: &PartitionResult) -> f32 {
    1.0 / (1 + result.edge_cut) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_graph_basic() {
        let r = partition_graph(6, &[(0,1),(1,2),(2,3),(3,4),(4,5)], 2);
        assert_eq!(partition_count(&r), 2);
        assert_eq!(partition_node_count(&r), 6);
    }

    #[test]
    fn test_partition_k1() {
        let r = partition_graph(4, &[], 1);
        assert_eq!(partition_count(&r), 1);
        assert_eq!(partition_node_count(&r), 4);
    }

    #[test]
    fn test_edge_cut() {
        let r = partition_graph(4, &[(0,1),(1,2),(2,3)], 2);
        // nodes 0,2 -> part 0; nodes 1,3 -> part 1
        assert!(partition_edge_cut(&r) > 0);
    }

    #[test]
    fn test_rebalance_noop() {
        let r = partition_graph(4, &[], 2);
        let r2 = rebalance_partition(&r);
        assert_eq!(partition_count(&r2), partition_count(&r));
    }

    #[test]
    fn test_partition_to_vec() {
        let r = partition_graph(3, &[], 3);
        let v = partition_to_vec(&r);
        assert_eq!(v.len(), 3);
    }

    #[test]
    fn test_merge_partitions() {
        let a = partition_graph(2, &[], 2);
        let b = partition_graph(2, &[], 2);
        let merged = merge_partitions(&a, &b);
        assert_eq!(partition_count(&merged), 4);
    }

    #[test]
    fn test_partition_quality_no_cut() {
        let r = partition_graph(4, &[], 2);
        let q = partition_quality(&r);
        assert!(q > 0.0);
    }

    #[test]
    fn test_empty_graph() {
        let r = partition_graph(0, &[], 3);
        assert_eq!(partition_node_count(&r), 0);
    }
}
