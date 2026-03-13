// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Curve network extraction from mesh edges (feature curves, boundaries).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CurveNetwork {
    pub nodes: Vec<[f32; 3]>,
    pub edges: Vec<[u32; 2]>,
}

#[allow(dead_code)]
pub fn new_curve_network() -> CurveNetwork {
    CurveNetwork { nodes: Vec::new(), edges: Vec::new() }
}

#[allow(dead_code)]
pub fn cn_add_node(cn: &mut CurveNetwork, pos: [f32; 3]) -> u32 {
    cn.nodes.push(pos);
    (cn.nodes.len() - 1) as u32
}

#[allow(dead_code)]
pub fn cn_add_edge(cn: &mut CurveNetwork, a: u32, b: u32) {
    cn.edges.push([a, b]);
}

#[allow(dead_code)]
pub fn cn_node_count(cn: &CurveNetwork) -> usize { cn.nodes.len() }

#[allow(dead_code)]
pub fn cn_edge_count(cn: &CurveNetwork) -> usize { cn.edges.len() }

#[allow(dead_code)]
pub fn cn_edge_length(cn: &CurveNetwork, idx: usize) -> f32 {
    if idx >= cn.edges.len() { return 0.0; }
    let e = cn.edges[idx];
    let a = cn.nodes[e[0] as usize];
    let b = cn.nodes[e[1] as usize];
    let d = [b[0]-a[0], b[1]-a[1], b[2]-a[2]];
    (d[0]*d[0]+d[1]*d[1]+d[2]*d[2]).sqrt()
}

#[allow(dead_code)]
pub fn cn_total_length(cn: &CurveNetwork) -> f32 {
    (0..cn.edges.len()).map(|i| cn_edge_length(cn, i)).sum()
}

#[allow(dead_code)]
pub fn cn_node_degree(cn: &CurveNetwork, node: u32) -> usize {
    cn.edges.iter().filter(|e| e[0] == node || e[1] == node).count()
}

#[allow(dead_code)]
pub fn cn_validate(cn: &CurveNetwork) -> bool {
    cn.edges.iter().all(|e| (e[0] as usize) < cn.nodes.len() && (e[1] as usize) < cn.nodes.len())
}

#[allow(dead_code)]
pub fn cn_to_json(cn: &CurveNetwork) -> String {
    format!("{{\"nodes\":{},\"edges\":{},\"total_length\":{:.4}}}", cn.nodes.len(), cn.edges.len(), cn_total_length(cn))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> CurveNetwork {
        let mut cn = new_curve_network();
        cn_add_node(&mut cn, [0.0,0.0,0.0]);
        cn_add_node(&mut cn, [1.0,0.0,0.0]);
        cn_add_node(&mut cn, [2.0,0.0,0.0]);
        cn_add_edge(&mut cn, 0, 1);
        cn_add_edge(&mut cn, 1, 2);
        cn
    }

    #[test] fn test_new() { let cn = new_curve_network(); assert_eq!(cn_node_count(&cn), 0); }
    #[test] fn test_add_node() { let mut cn = new_curve_network(); cn_add_node(&mut cn, [0.0,0.0,0.0]); assert_eq!(cn.nodes.len(), 1); }
    #[test] fn test_node_count() { assert_eq!(cn_node_count(&line()), 3); }
    #[test] fn test_edge_count() { assert_eq!(cn_edge_count(&line()), 2); }
    #[test] fn test_edge_length() { assert!((cn_edge_length(&line(), 0) - 1.0).abs() < 1e-5); }
    #[test] fn test_total_length() { assert!((cn_total_length(&line()) - 2.0).abs() < 1e-5); }
    #[test] fn test_degree() { assert_eq!(cn_node_degree(&line(), 1), 2); }
    #[test] fn test_validate() { assert!(cn_validate(&line())); }
    #[test] fn test_to_json() { assert!(cn_to_json(&line()).contains("total_length")); }
    #[test] fn test_endpoint_degree() { assert_eq!(cn_node_degree(&line(), 0), 1); }
}
