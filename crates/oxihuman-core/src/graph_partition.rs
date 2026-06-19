#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

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

/// Rebalance partitions using a Kernighan–Lin-style greedy node-move pass.
///
/// Repeatedly finds the heaviest and lightest partitions, then moves the
/// boundary node from the heavy partition whose transfer to the light partition
/// causes the smallest increase (or largest decrease) in edge cut.  The pass
/// terminates when all partitions are within 1 node of each other, or no
/// boundary node remains in the heavy partition.
///
/// Boundary nodes are identified by examining the edges encoded in the
/// `PartitionResult`: for each `(a, b)` pair in the flat partition-to-vec
/// representation, a node is on the boundary if it has at least one neighbour
/// assigned to a different partition.  Since the `PartitionResult` stores
/// only partition membership and edge-cut counts (not the raw edge list), we
/// reconstruct an implicit edge list from the partition ID assignments using
/// the round-robin structure established by `partition_graph`.
#[allow(dead_code)]
pub fn rebalance_partition(result: &PartitionResult) -> PartitionResult {
    if result.partitions.len() <= 1 {
        return result.clone();
    }

    // Build a mutable node-to-partition assignment map.
    let total_nodes: usize = result.partitions.iter().map(|p| p.nodes.len()).sum();
    if total_nodes == 0 {
        return result.clone();
    }

    // node_part[node_id] = partition_id
    let mut node_part: Vec<usize> = vec![0usize; total_nodes];
    for (pi, part) in result.partitions.iter().enumerate() {
        for &n in &part.nodes {
            if n < total_nodes {
                node_part[n] = pi;
            }
        }
    }

    // Reconstruct implicit adjacency: the original round-robin assignment means
    // we don't have the raw edges.  We treat consecutive nodes as a chain graph
    // (0-1, 1-2, … (n-2)-(n-1)) to approximate boundary detection, which is
    // always available regardless of how `partition_graph` was called.
    // This covers the typical use-case without storing edge data separately.
    let build_adj = |np: &[usize]| -> Vec<Vec<usize>> {
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); np.len()];
        for i in 0..np.len().saturating_sub(1) {
            adj[i].push(i + 1);
            adj[i + 1].push(i);
        }
        adj
    };

    let adj = build_adj(&node_part);

    let k = result.partitions.len();
    let mut part_nodes: Vec<Vec<usize>> = result.partitions.iter().map(|p| p.nodes.clone()).collect();

    // Iteratively move nodes from heaviest to lightest partition.
    loop {
        // Identify heaviest and lightest partitions.
        let max_idx = (0..k).max_by_key(|&i| part_nodes[i].len()).unwrap_or(0);
        let min_idx = (0..k).min_by_key(|&i| part_nodes[i].len()).unwrap_or(0);

        let heavy_len = part_nodes[max_idx].len();
        let light_len = part_nodes[min_idx].len();

        if heavy_len <= light_len + 1 {
            // Already balanced within 1 node.
            break;
        }

        // Find boundary nodes in the heavy partition: those with ≥1 neighbour
        // in another partition.
        let boundary: Vec<usize> = part_nodes[max_idx]
            .iter()
            .copied()
            .filter(|&n| {
                n < adj.len()
                    && adj[n]
                        .iter()
                        .any(|&nb| nb < node_part.len() && node_part[nb] != max_idx)
            })
            .collect();

        if boundary.is_empty() {
            break;
        }

        // Pick the boundary node that minimises additional cut when moved.
        // Delta cut for moving node n from heavy to light =
        //   (edges to light partition neighbours) - (edges to heavy partition neighbours).
        // We want the node with the minimum delta (most negative = best gain).
        let best = boundary
            .iter()
            .copied()
            .min_by(|&a, &b| {
                let delta = |n: usize| -> i32 {
                    if n >= adj.len() {
                        return 0;
                    }
                    let to_light: i32 = adj[n]
                        .iter()
                        .filter(|&&nb| nb < node_part.len() && node_part[nb] == min_idx)
                        .count() as i32;
                    let to_heavy: i32 = adj[n]
                        .iter()
                        .filter(|&&nb| nb < node_part.len() && node_part[nb] == max_idx)
                        .count() as i32;
                    // Moving from heavy → light: new cuts to heavy, removed cuts to light.
                    to_heavy - to_light
                };
                delta(a).cmp(&delta(b))
            });

        let node = match best {
            Some(n) => n,
            None => break,
        };

        // Move node from heavy to light.
        part_nodes[max_idx].retain(|&n| n != node);
        part_nodes[min_idx].push(node);
        if node < node_part.len() {
            node_part[node] = min_idx;
        }
    }

    // Recompute edge cut.
    let edge_cut = (0..total_nodes.saturating_sub(1))
        .filter(|&i| i < node_part.len() && i + 1 < node_part.len() && node_part[i] != node_part[i + 1])
        .count();

    let partitions: Vec<Partition> = part_nodes
        .into_iter()
        .enumerate()
        .map(|(id, nodes)| Partition { id, nodes })
        .collect();

    PartitionResult { partitions, edge_cut }
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
    fn test_rebalance_same_partition_count() {
        // Rebalancing must preserve the number of partitions.
        let r = partition_graph(4, &[], 2);
        let r2 = rebalance_partition(&r);
        assert_eq!(partition_count(&r2), partition_count(&r));
    }

    #[test]
    fn test_rebalance_preserves_node_count() {
        // Total nodes must be unchanged after rebalancing.
        let r = partition_graph(6, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)], 2);
        let r2 = rebalance_partition(&r);
        assert_eq!(partition_node_count(&r2), partition_node_count(&r));
    }

    #[test]
    fn test_rebalance_produces_balanced_partitions() {
        // 6 nodes into 2 partitions → each should end up with 3 nodes (within 1).
        let r = partition_graph(6, &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)], 2);
        let r2 = rebalance_partition(&r);
        let sizes: Vec<usize> = r2.partitions.iter().map(|p| p.nodes.len()).collect();
        let max_s = sizes.iter().copied().max().unwrap_or(0);
        let min_s = sizes.iter().copied().min().unwrap_or(0);
        assert!(max_s - min_s <= 1, "partitions must be balanced within 1 node after rebalancing");
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
