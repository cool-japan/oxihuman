// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Potree hierarchical point cloud export stub.

#[allow(dead_code)]
pub struct PotreeNode {
    pub center: [f32; 3],
    pub size: f32,
    pub points: Vec<[f32; 3]>,
    pub children: Vec<usize>,
}

#[allow(dead_code)]
pub struct PotreeExport {
    pub nodes: Vec<PotreeNode>,
    pub max_depth: u32,
}

#[allow(dead_code)]
pub fn new_potree_export(max_depth: u32) -> PotreeExport {
    PotreeExport { nodes: Vec::new(), max_depth }
}

#[allow(dead_code)]
pub fn pe_add_root(exp: &mut PotreeExport, center: [f32; 3], size: f32) -> usize {
    let idx = exp.nodes.len();
    exp.nodes.push(PotreeNode { center, size, points: Vec::new(), children: Vec::new() });
    idx
}

#[allow(dead_code)]
pub fn pe_add_point_to_node(exp: &mut PotreeExport, node_idx: usize, point: [f32; 3]) {
    if node_idx < exp.nodes.len() {
        exp.nodes[node_idx].points.push(point);
    }
}

#[allow(dead_code)]
pub fn pe_total_points(exp: &PotreeExport) -> usize {
    exp.nodes.iter().map(|n| n.points.len()).sum()
}

#[allow(dead_code)]
pub fn pe_node_count(exp: &PotreeExport) -> usize {
    exp.nodes.len()
}

#[allow(dead_code)]
pub fn pe_max_depth(exp: &PotreeExport) -> u32 {
    exp.max_depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let exp = new_potree_export(8);
        assert_eq!(pe_node_count(&exp), 0);
    }

    #[test]
    fn test_add_root() {
        let mut exp = new_potree_export(8);
        let idx = pe_add_root(&mut exp, [0.0; 3], 10.0);
        assert_eq!(idx, 0);
        assert_eq!(pe_node_count(&exp), 1);
    }

    #[test]
    fn test_add_point() {
        let mut exp = new_potree_export(8);
        let idx = pe_add_root(&mut exp, [0.0; 3], 10.0);
        pe_add_point_to_node(&mut exp, idx, [1.0, 2.0, 3.0]);
        assert_eq!(pe_total_points(&exp), 1);
    }

    #[test]
    fn test_total_points() {
        let mut exp = new_potree_export(8);
        let r0 = pe_add_root(&mut exp, [0.0; 3], 10.0);
        let r1 = pe_add_root(&mut exp, [10.0; 3], 10.0);
        pe_add_point_to_node(&mut exp, r0, [0.0; 3]);
        pe_add_point_to_node(&mut exp, r0, [1.0; 3]);
        pe_add_point_to_node(&mut exp, r1, [5.0; 3]);
        assert_eq!(pe_total_points(&exp), 3);
    }

    #[test]
    fn test_node_count() {
        let mut exp = new_potree_export(4);
        pe_add_root(&mut exp, [0.0; 3], 5.0);
        pe_add_root(&mut exp, [5.0; 3], 5.0);
        assert_eq!(pe_node_count(&exp), 2);
    }

    #[test]
    fn test_max_depth() {
        let exp = new_potree_export(12);
        assert_eq!(pe_max_depth(&exp), 12);
    }

    #[test]
    fn test_add_point_invalid_node() {
        let mut exp = new_potree_export(4);
        pe_add_point_to_node(&mut exp, 99, [0.0; 3]);
        assert_eq!(pe_total_points(&exp), 0);
    }

    #[test]
    fn test_empty_export() {
        let exp = new_potree_export(0);
        assert_eq!(pe_total_points(&exp), 0);
        assert_eq!(pe_node_count(&exp), 0);
    }
}
