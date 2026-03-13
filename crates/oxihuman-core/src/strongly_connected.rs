// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tarjan's strongly connected components (SCC) algorithm.

/// A directed graph for SCC computation.
pub struct SccGraph {
    pub n: usize,
    pub adj: Vec<Vec<usize>>,
}

impl SccGraph {
    pub fn new(n: usize) -> Self {
        SccGraph {
            n,
            adj: vec![Vec::new(); n],
        }
    }
}

/// Create a new SCC graph with `n` vertices (0..n).
pub fn new_scc_graph(n: usize) -> SccGraph {
    SccGraph::new(n)
}

/// Add a directed edge from `u` to `v`.
pub fn scc_add_edge(g: &mut SccGraph, u: usize, v: usize) {
    if u < g.n && v < g.n {
        g.adj[u].push(v);
    }
}

/// Run Tarjan's SCC. Returns a list of SCCs, each a sorted Vec of vertex indices.
pub fn tarjan_scc(g: &SccGraph) -> Vec<Vec<usize>> {
    let n = g.n;
    let mut index = vec![usize::MAX; n];
    let mut lowlink = vec![0usize; n];
    let mut on_stack = vec![false; n];
    let mut stack: Vec<usize> = Vec::new();
    let mut sccs: Vec<Vec<usize>> = Vec::new();
    let mut counter = 0usize;

    for start in 0..n {
        if index[start] == usize::MAX {
            /* iterative DFS to avoid stack overflow */
            let mut dfs_stack: Vec<(usize, usize)> = vec![(start, 0)];
            index[start] = counter;
            lowlink[start] = counter;
            counter += 1;
            on_stack[start] = true;
            stack.push(start);

            while let Some((v, ei)) = dfs_stack.last_mut() {
                let v = *v;
                let neighbors = &g.adj[v];
                if *ei < neighbors.len() {
                    let w = neighbors[*ei];
                    *ei += 1;
                    if index[w] == usize::MAX {
                        index[w] = counter;
                        lowlink[w] = counter;
                        counter += 1;
                        on_stack[w] = true;
                        stack.push(w);
                        dfs_stack.push((w, 0));
                    } else if on_stack[w] {
                        let lv = lowlink[v];
                        lowlink[v] = lv.min(index[w]);
                    }
                } else {
                    /* finished v */
                    dfs_stack.pop();
                    if let Some(&(parent, _)) = dfs_stack.last() {
                        let lv = lowlink[parent];
                        lowlink[parent] = lv.min(lowlink[v]);
                    }
                    if lowlink[v] == index[v] {
                        /* root of SCC */
                        let mut scc = Vec::new();
                        #[allow(clippy::while_let_loop)]
                        loop {
                            let Some(w) = stack.pop() else { break };
                            on_stack[w] = false;
                            scc.push(w);
                            if w == v {
                                break;
                            }
                        }
                        scc.sort_unstable();
                        sccs.push(scc);
                    }
                }
            }
        }
    }
    sccs
}

/// Return the number of SCCs.
pub fn scc_count(g: &SccGraph) -> usize {
    tarjan_scc(g).len()
}

/// Return `true` if the graph is strongly connected (single SCC with all nodes).
pub fn is_strongly_connected(g: &SccGraph) -> bool {
    if g.n == 0 {
        return true;
    }
    let sccs = tarjan_scc(g);
    sccs.len() == 1
}

/// Return the largest SCC by node count.
pub fn largest_scc(g: &SccGraph) -> Vec<usize> {
    tarjan_scc(g)
        .into_iter()
        .max_by_key(|s| s.len())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_node_scc() {
        /* a single node is its own SCC */
        let g = new_scc_graph(1);
        let sccs = tarjan_scc(&g);
        assert_eq!(sccs.len(), 1);
        assert_eq!(sccs[0], vec![0]);
    }

    #[test]
    fn test_two_node_cycle() {
        /* 0 <-> 1 forms one SCC */
        let mut g = new_scc_graph(2);
        scc_add_edge(&mut g, 0, 1);
        scc_add_edge(&mut g, 1, 0);
        assert_eq!(scc_count(&g), 1);
    }

    #[test]
    fn test_dag_three_sccs() {
        /* 0 -> 1 -> 2; no back edges */
        let mut g = new_scc_graph(3);
        scc_add_edge(&mut g, 0, 1);
        scc_add_edge(&mut g, 1, 2);
        assert_eq!(scc_count(&g), 3);
    }

    #[test]
    fn test_strongly_connected_true() {
        let mut g = new_scc_graph(3);
        scc_add_edge(&mut g, 0, 1);
        scc_add_edge(&mut g, 1, 2);
        scc_add_edge(&mut g, 2, 0);
        assert!(is_strongly_connected(&g));
    }

    #[test]
    fn test_strongly_connected_false() {
        let mut g = new_scc_graph(3);
        scc_add_edge(&mut g, 0, 1);
        scc_add_edge(&mut g, 1, 2);
        assert!(!is_strongly_connected(&g));
    }

    #[test]
    fn test_empty_graph() {
        let g = new_scc_graph(0);
        assert!(is_strongly_connected(&g));
    }

    #[test]
    fn test_largest_scc() {
        /* SCC {0,1,2} and singleton {3} */
        let mut g = new_scc_graph(4);
        scc_add_edge(&mut g, 0, 1);
        scc_add_edge(&mut g, 1, 2);
        scc_add_edge(&mut g, 2, 0);
        scc_add_edge(&mut g, 2, 3);
        let big = largest_scc(&g);
        assert_eq!(big.len(), 3);
    }

    #[test]
    fn test_self_loop_scc() {
        let mut g = new_scc_graph(2);
        scc_add_edge(&mut g, 0, 0); /* self-loop */
        scc_add_edge(&mut g, 1, 1);
        assert_eq!(scc_count(&g), 2);
    }

    #[test]
    fn test_complex_graph() {
        /* Classic Tarjan example */
        let mut g = new_scc_graph(8);
        let edges = [
            (0, 1),
            (1, 2),
            (2, 0),
            (3, 1),
            (3, 2),
            (3, 4),
            (4, 3),
            (4, 5),
            (5, 6),
            (6, 4),
            (7, 6),
            (7, 7),
        ];
        for (u, v) in edges {
            scc_add_edge(&mut g, u, v);
        }
        let count = scc_count(&g);
        assert!(count >= 3);
    }
}
