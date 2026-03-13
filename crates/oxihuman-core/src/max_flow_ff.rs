// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Ford-Fulkerson max-flow using BFS (Edmonds-Karp).

/// A flow network.
pub struct FlowGraph {
    pub n: usize,
    /// `capacity[u][v]`
    pub cap: Vec<Vec<f32>>,
}

impl FlowGraph {
    pub fn new(n: usize) -> Self {
        FlowGraph {
            n,
            cap: vec![vec![0.0; n]; n],
        }
    }
}

/// Create a new flow graph with `n` vertices.
pub fn new_flow_graph(n: usize) -> FlowGraph {
    FlowGraph::new(n)
}

/// Add a directed edge with capacity `c`.
pub fn fg_add_edge(g: &mut FlowGraph, u: usize, v: usize, c: f32) {
    if u < g.n && v < g.n {
        g.cap[u][v] += c;
    }
}

/// Return the number of nodes.
pub fn fg_node_count(g: &FlowGraph) -> usize {
    g.n
}

/// Run Edmonds-Karp (BFS-based Ford-Fulkerson). Returns max-flow value.
#[allow(clippy::needless_range_loop)]
pub fn max_flow(g: &FlowGraph, src: usize, sink: usize) -> f32 {
    if src >= g.n || sink >= g.n || src == sink {
        return 0.0;
    }
    let n = g.n;
    let mut residual = g.cap.clone();
    let mut total = 0.0f32;

    loop {
        /* BFS to find augmenting path */
        let mut prev = vec![usize::MAX; n];
        let mut visited = vec![false; n];
        let mut queue = std::collections::VecDeque::new();
        visited[src] = true;
        queue.push_back(src);

        while let Some(u) = queue.pop_front() {
            if u == sink {
                break;
            }
            for v in 0..n {
                if !visited[v] && residual[u][v] > 1e-9 {
                    visited[v] = true;
                    prev[v] = u;
                    queue.push_back(v);
                }
            }
        }

        if !visited[sink] {
            break; /* no augmenting path */
        }

        /* find bottleneck */
        let mut flow = f32::INFINITY;
        let mut cur = sink;
        while cur != src {
            let p = prev[cur];
            flow = flow.min(residual[p][cur]);
            cur = p;
        }

        /* update residual graph */
        cur = sink;
        while cur != src {
            let p = prev[cur];
            residual[p][cur] -= flow;
            residual[cur][p] += flow;
            cur = p;
        }

        total += flow;
    }
    total
}

/// Return `true` if `sink` is reachable from `src` in the residual graph
/// (i.e., flow is not yet at maximum).
#[allow(clippy::needless_range_loop)]
pub fn fg_has_augmenting_path(g: &FlowGraph, src: usize, sink: usize) -> bool {
    /* thin wrapper — just try one BFS */
    let small = FlowGraph {
        n: g.n,
        cap: g.cap.clone(),
    };
    /* a dummy call; real check: residual capacity > 0 on some path */
    let _ = small;
    /* Simple BFS on capacities > 0 */
    let mut visited = vec![false; g.n];
    let mut queue = std::collections::VecDeque::new();
    if src < g.n {
        visited[src] = true;
        queue.push_back(src);
    }
    while let Some(u) = queue.pop_front() {
        if u == sink {
            return true;
        }
        for v in 0..g.n {
            if !visited[v] && g.cap[u][v] > 1e-9 {
                visited[v] = true;
                queue.push_back(v);
            }
        }
    }
    false
}

/// Return total capacity from `src` (sum of outgoing edges).
pub fn fg_total_capacity_from(g: &FlowGraph, src: usize) -> f32 {
    if src >= g.n {
        return 0.0;
    }
    g.cap[src].iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_flow() {
        let mut g = new_flow_graph(4);
        fg_add_edge(&mut g, 0, 1, 3.0);
        fg_add_edge(&mut g, 0, 2, 2.0);
        fg_add_edge(&mut g, 1, 3, 3.0);
        fg_add_edge(&mut g, 2, 3, 2.0);
        assert!((max_flow(&g, 0, 3) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_no_path_returns_zero() {
        let g = new_flow_graph(3);
        assert_eq!(max_flow(&g, 0, 2), 0.0);
    }

    #[test]
    fn test_bottleneck_edge() {
        let mut g = new_flow_graph(3);
        fg_add_edge(&mut g, 0, 1, 100.0);
        fg_add_edge(&mut g, 1, 2, 1.0);
        assert!((max_flow(&g, 0, 2) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_parallel_paths() {
        let mut g = new_flow_graph(4);
        fg_add_edge(&mut g, 0, 1, 5.0);
        fg_add_edge(&mut g, 0, 2, 5.0);
        fg_add_edge(&mut g, 1, 3, 5.0);
        fg_add_edge(&mut g, 2, 3, 5.0);
        assert!((max_flow(&g, 0, 3) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn test_node_count() {
        let g = new_flow_graph(6);
        assert_eq!(fg_node_count(&g), 6);
    }

    #[test]
    fn test_capacity_from() {
        let mut g = new_flow_graph(3);
        fg_add_edge(&mut g, 0, 1, 3.0);
        fg_add_edge(&mut g, 0, 2, 4.0);
        assert!((fg_total_capacity_from(&g, 0) - 7.0).abs() < 1e-5);
    }

    #[test]
    fn test_src_equals_sink() {
        let g = new_flow_graph(3);
        /* not a meaningful query, but should not crash */
        let _ = max_flow(&g, 1, 1);
    }

    #[test]
    fn test_diamond_graph() {
        /* classic 6-node flow */
        let mut g = new_flow_graph(6);
        fg_add_edge(&mut g, 0, 1, 10.0);
        fg_add_edge(&mut g, 0, 2, 10.0);
        fg_add_edge(&mut g, 1, 3, 10.0);
        fg_add_edge(&mut g, 2, 4, 10.0);
        fg_add_edge(&mut g, 3, 5, 10.0);
        fg_add_edge(&mut g, 4, 5, 10.0);
        fg_add_edge(&mut g, 1, 4, 5.0);
        fg_add_edge(&mut g, 2, 3, 5.0);
        assert!(max_flow(&g, 0, 5) >= 20.0 - 1e-3);
    }
}
