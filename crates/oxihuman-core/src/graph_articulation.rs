// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Articulation points and bridges in undirected graphs (Tarjan's DFS).

/// An undirected graph.
pub struct ArticGraph {
    pub n: usize,
    pub adj: Vec<Vec<usize>>,
}

impl ArticGraph {
    pub fn new(n: usize) -> Self {
        ArticGraph {
            n,
            adj: vec![Vec::new(); n],
        }
    }
}

/// Create a new articulation-point graph with `n` nodes.
pub fn new_artic_graph(n: usize) -> ArticGraph {
    ArticGraph::new(n)
}

/// Add an undirected edge.
pub fn artic_add_edge(g: &mut ArticGraph, u: usize, v: usize) {
    if u < g.n && v < g.n {
        g.adj[u].push(v);
        g.adj[v].push(u);
    }
}

/// Find all articulation points.
pub fn find_articulation_points(g: &ArticGraph) -> Vec<usize> {
    let n = g.n;
    let mut disc = vec![usize::MAX; n];
    let mut low = vec![0usize; n];
    let mut is_ap = vec![false; n];
    let mut timer = 0usize;

    for start in 0..n {
        if disc[start] == usize::MAX {
            dfs_ap(
                g,
                start,
                usize::MAX,
                &mut disc,
                &mut low,
                &mut is_ap,
                &mut timer,
            );
        }
    }
    is_ap
        .iter()
        .enumerate()
        .filter(|(_, &v)| v)
        .map(|(i, _)| i)
        .collect()
}

fn dfs_ap(
    g: &ArticGraph,
    u: usize,
    parent: usize,
    disc: &mut Vec<usize>,
    low: &mut Vec<usize>,
    is_ap: &mut Vec<bool>,
    timer: &mut usize,
) {
    disc[u] = *timer;
    low[u] = *timer;
    *timer += 1;
    let mut child_count = 0usize;

    for &v in &g.adj[u] {
        if disc[v] == usize::MAX {
            child_count += 1;
            dfs_ap(g, v, u, disc, low, is_ap, timer);
            low[u] = low[u].min(low[v]);
            /* articulation point condition */
            if parent == usize::MAX && child_count > 1 {
                is_ap[u] = true;
            }
            if parent != usize::MAX && low[v] >= disc[u] {
                is_ap[u] = true;
            }
        } else if v != parent {
            low[u] = low[u].min(disc[v]);
        }
    }
}

/// Find all bridges (edges whose removal disconnects the graph).
pub fn find_bridges(g: &ArticGraph) -> Vec<(usize, usize)> {
    let n = g.n;
    let mut disc = vec![usize::MAX; n];
    let mut low = vec![0usize; n];
    let mut bridges = Vec::new();
    let mut timer = 0usize;

    for start in 0..n {
        if disc[start] == usize::MAX {
            dfs_bridge(
                g,
                start,
                usize::MAX,
                &mut disc,
                &mut low,
                &mut bridges,
                &mut timer,
            );
        }
    }
    bridges
}

fn dfs_bridge(
    g: &ArticGraph,
    u: usize,
    parent: usize,
    disc: &mut Vec<usize>,
    low: &mut Vec<usize>,
    bridges: &mut Vec<(usize, usize)>,
    timer: &mut usize,
) {
    disc[u] = *timer;
    low[u] = *timer;
    *timer += 1;
    for &v in &g.adj[u] {
        if disc[v] == usize::MAX {
            dfs_bridge(g, v, u, disc, low, bridges, timer);
            low[u] = low[u].min(low[v]);
            if low[v] > disc[u] {
                bridges.push((u.min(v), u.max(v)));
            }
        } else if v != parent {
            low[u] = low[u].min(disc[v]);
        }
    }
}

/// Return number of connected components.
pub fn artic_component_count(g: &ArticGraph) -> usize {
    let mut visited = vec![false; g.n];
    let mut count = 0;
    for start in 0..g.n {
        if !visited[start] {
            count += 1;
            let mut stack = vec![start];
            while let Some(u) = stack.pop() {
                if visited[u] {
                    continue;
                }
                visited[u] = true;
                for &v in &g.adj[u] {
                    if !visited[v] {
                        stack.push(v);
                    }
                }
            }
        }
    }
    count
}

/// Return `true` if the graph is biconnected (no articulation points, single component).
pub fn is_biconnected(g: &ArticGraph) -> bool {
    if g.n <= 2 {
        return true;
    }
    find_articulation_points(g).is_empty() && artic_component_count(g) == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_articulation_in_cycle() {
        let mut g = new_artic_graph(4);
        artic_add_edge(&mut g, 0, 1);
        artic_add_edge(&mut g, 1, 2);
        artic_add_edge(&mut g, 2, 3);
        artic_add_edge(&mut g, 3, 0);
        /* a cycle has no articulation points */
        assert!(find_articulation_points(&g).is_empty());
    }

    #[test]
    fn test_articulation_in_path() {
        /* 0-1-2: node 1 is AP */
        let mut g = new_artic_graph(3);
        artic_add_edge(&mut g, 0, 1);
        artic_add_edge(&mut g, 1, 2);
        let aps = find_articulation_points(&g);
        assert!(aps.contains(&1));
    }

    #[test]
    fn test_bridge_in_path() {
        let mut g = new_artic_graph(3);
        artic_add_edge(&mut g, 0, 1);
        artic_add_edge(&mut g, 1, 2);
        let bridges = find_bridges(&g);
        assert_eq!(bridges.len(), 2);
    }

    #[test]
    fn test_no_bridges_in_cycle() {
        let mut g = new_artic_graph(4);
        artic_add_edge(&mut g, 0, 1);
        artic_add_edge(&mut g, 1, 2);
        artic_add_edge(&mut g, 2, 3);
        artic_add_edge(&mut g, 3, 0);
        assert!(find_bridges(&g).is_empty());
    }

    #[test]
    fn test_component_count() {
        let mut g = new_artic_graph(4);
        artic_add_edge(&mut g, 0, 1);
        /* 2 and 3 are isolated */
        assert_eq!(artic_component_count(&g), 3);
    }

    #[test]
    fn test_biconnected_true() {
        let mut g = new_artic_graph(4);
        artic_add_edge(&mut g, 0, 1);
        artic_add_edge(&mut g, 1, 2);
        artic_add_edge(&mut g, 2, 3);
        artic_add_edge(&mut g, 3, 0);
        artic_add_edge(&mut g, 0, 2);
        assert!(is_biconnected(&g));
    }

    #[test]
    fn test_biconnected_false() {
        let mut g = new_artic_graph(3);
        artic_add_edge(&mut g, 0, 1);
        artic_add_edge(&mut g, 1, 2);
        assert!(!is_biconnected(&g));
    }

    #[test]
    fn test_empty_graph() {
        let g = new_artic_graph(0);
        assert!(find_articulation_points(&g).is_empty());
    }

    #[test]
    fn test_single_node() {
        let g = new_artic_graph(1);
        assert!(find_articulation_points(&g).is_empty());
    }
}
