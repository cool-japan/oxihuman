// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Topological sort (Kahn's algorithm) for DAGs.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TopoGraph {
    pub nodes: Vec<u32>,
    pub edges: Vec<(u32, u32)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TopoResult {
    pub order: Vec<u32>,
    pub has_cycle: bool,
}

#[allow(dead_code)]
pub fn new_topo_graph() -> TopoGraph {
    TopoGraph {
        nodes: Vec::new(),
        edges: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn topo_add_node(g: &mut TopoGraph, node: u32) {
    if !g.nodes.contains(&node) {
        g.nodes.push(node);
    }
}

#[allow(dead_code)]
pub fn topo_add_edge(g: &mut TopoGraph, from: u32, to: u32) {
    g.edges.push((from, to));
}

#[allow(dead_code)]
pub fn topo_sort(g: &TopoGraph) -> TopoResult {
    let mut in_degree: HashMap<u32, usize> = HashMap::new();
    for &n in &g.nodes {
        in_degree.entry(n).or_insert(0);
    }
    for &(_, to) in &g.edges {
        *in_degree.entry(to).or_insert(0) += 1;
    }

    let mut queue: std::collections::VecDeque<u32> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(&n, _)| n)
        .collect();
    queue.make_contiguous().sort();

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        let mut neighbors: Vec<u32> = g
            .edges
            .iter()
            .filter(|&&(from, _)| from == node)
            .map(|&(_, to)| to)
            .collect();
        neighbors.sort();
        for to in neighbors {
            let d = in_degree.entry(to).or_insert(0);
            *d -= 1;
            if *d == 0 {
                queue.push_back(to);
            }
        }
    }

    let has_cycle = order.len() < g.nodes.len();
    TopoResult { order, has_cycle }
}

#[allow(dead_code)]
pub fn topo_node_count(g: &TopoGraph) -> usize {
    g.nodes.len()
}

#[allow(dead_code)]
pub fn topo_edge_count(g: &TopoGraph) -> usize {
    g.edges.len()
}

#[allow(dead_code)]
pub fn topo_has_cycle(g: &TopoGraph) -> bool {
    topo_sort(g).has_cycle
}

#[allow(dead_code)]
pub fn topo_remove_node(g: &mut TopoGraph, node: u32) {
    g.nodes.retain(|&n| n != node);
    g.edges.retain(|&(f, t)| f != node && t != node);
}

#[allow(dead_code)]
pub fn topo_clear(g: &mut TopoGraph) {
    g.nodes.clear();
    g.edges.clear();
}

#[allow(dead_code)]
pub fn topo_sort_dag(n: usize, edges: &[(usize, usize)]) -> Option<Vec<usize>> {
    let mut in_deg = vec![0usize; n];
    for &(_, to) in edges {
        if to < n {
            in_deg[to] += 1;
        }
    }
    let mut queue: std::collections::VecDeque<usize> = (0..n).filter(|&i| in_deg[i] == 0).collect();
    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &(from, to) in edges {
            if from == node && to < n {
                in_deg[to] -= 1;
                if in_deg[to] == 0 {
                    queue.push_back(to);
                }
            }
        }
    }
    if order.len() == n {
        Some(order)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn topo_has_cycle_dag(n: usize, edges: &[(usize, usize)]) -> bool {
    topo_sort_dag(n, edges).is_none()
}

#[allow(dead_code)]
pub fn topo_layer_count(n: usize, edges: &[(usize, usize)]) -> usize {
    let mut dist = vec![0usize; n];
    let order = match topo_sort_dag(n, edges) {
        Some(o) => o,
        None => return 0,
    };
    for &node in &order {
        for &(from, to) in edges {
            if from == node && to < n {
                dist[to] = dist[to].max(dist[node] + 1);
            }
        }
    }
    dist.iter().max().copied().unwrap_or(0) + 1
}

#[allow(dead_code)]
pub fn topo_sources(n: usize, edges: &[(usize, usize)]) -> Vec<usize> {
    let mut in_deg = vec![0usize; n];
    for &(_, to) in edges {
        if to < n {
            in_deg[to] += 1;
        }
    }
    (0..n).filter(|&i| in_deg[i] == 0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let g = new_topo_graph();
        assert_eq!(topo_node_count(&g), 0);
        assert_eq!(topo_edge_count(&g), 0);
    }

    #[test]
    fn test_add_node() {
        let mut g = new_topo_graph();
        topo_add_node(&mut g, 1);
        topo_add_node(&mut g, 1); // duplicate
        assert_eq!(topo_node_count(&g), 1);
    }

    #[test]
    fn test_simple_sort() {
        let mut g = new_topo_graph();
        topo_add_node(&mut g, 1);
        topo_add_node(&mut g, 2);
        topo_add_node(&mut g, 3);
        topo_add_edge(&mut g, 1, 2);
        topo_add_edge(&mut g, 2, 3);
        let res = topo_sort(&g);
        assert!(!res.has_cycle);
        assert_eq!(res.order, vec![1, 2, 3]);
    }

    #[test]
    fn test_cycle_detection() {
        let mut g = new_topo_graph();
        topo_add_node(&mut g, 1);
        topo_add_node(&mut g, 2);
        topo_add_edge(&mut g, 1, 2);
        topo_add_edge(&mut g, 2, 1);
        assert!(topo_has_cycle(&g));
    }

    #[test]
    fn test_no_cycle() {
        let mut g = new_topo_graph();
        topo_add_node(&mut g, 10);
        topo_add_node(&mut g, 20);
        topo_add_edge(&mut g, 10, 20);
        assert!(!topo_has_cycle(&g));
    }

    #[test]
    fn test_remove_node() {
        let mut g = new_topo_graph();
        topo_add_node(&mut g, 1);
        topo_add_node(&mut g, 2);
        topo_add_edge(&mut g, 1, 2);
        topo_remove_node(&mut g, 1);
        assert_eq!(topo_node_count(&g), 1);
        assert_eq!(topo_edge_count(&g), 0);
    }

    #[test]
    fn test_clear() {
        let mut g = new_topo_graph();
        topo_add_node(&mut g, 1);
        topo_add_edge(&mut g, 1, 2);
        topo_clear(&mut g);
        assert_eq!(topo_node_count(&g), 0);
        assert_eq!(topo_edge_count(&g), 0);
    }

    #[test]
    fn test_edge_count() {
        let mut g = new_topo_graph();
        topo_add_node(&mut g, 1);
        topo_add_node(&mut g, 2);
        topo_add_edge(&mut g, 1, 2);
        topo_add_edge(&mut g, 2, 1);
        assert_eq!(topo_edge_count(&g), 2);
    }

    #[test]
    fn test_diamond_dag() {
        let mut g = new_topo_graph();
        for n in [1, 2, 3, 4] {
            topo_add_node(&mut g, n);
        }
        topo_add_edge(&mut g, 1, 2);
        topo_add_edge(&mut g, 1, 3);
        topo_add_edge(&mut g, 2, 4);
        topo_add_edge(&mut g, 3, 4);
        let res = topo_sort(&g);
        assert!(!res.has_cycle);
        assert_eq!(res.order[0], 1);
        assert_eq!(*res.order.last().unwrap(), 4);
    }

    #[test]
    fn test_single_node() {
        let mut g = new_topo_graph();
        topo_add_node(&mut g, 5);
        let res = topo_sort(&g);
        assert!(!res.has_cycle);
        assert_eq!(res.order, vec![5]);
    }
}
