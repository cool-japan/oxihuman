// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export geometry node graph data.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum GeoNodeType { Input, Output, Transform, Merge, Split, Custom }

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeoNode {
    pub name: String,
    pub node_type: GeoNodeType,
    pub inputs: Vec<u32>,
    pub outputs: Vec<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GeometryNodeExport {
    pub nodes: Vec<GeoNode>,
    pub links: Vec<[u32; 2]>,
}

#[allow(dead_code)]
pub fn new_geometry_node_export() -> GeometryNodeExport {
    GeometryNodeExport { nodes: Vec::new(), links: Vec::new() }
}

#[allow(dead_code)]
pub fn gne_add_node(gne: &mut GeometryNodeExport, name: &str, nt: GeoNodeType) -> u32 {
    gne.nodes.push(GeoNode { name: name.to_string(), node_type: nt, inputs: Vec::new(), outputs: Vec::new() });
    (gne.nodes.len() - 1) as u32
}

#[allow(dead_code)]
pub fn gne_add_link(gne: &mut GeometryNodeExport, from: u32, to: u32) {
    gne.links.push([from, to]);
    if (from as usize) < gne.nodes.len() { gne.nodes[from as usize].outputs.push(to); }
    if (to as usize) < gne.nodes.len() { gne.nodes[to as usize].inputs.push(from); }
}

#[allow(dead_code)]
pub fn gne_node_count(gne: &GeometryNodeExport) -> usize { gne.nodes.len() }

#[allow(dead_code)]
pub fn gne_link_count(gne: &GeometryNodeExport) -> usize { gne.links.len() }

#[allow(dead_code)]
pub fn gne_input_nodes(gne: &GeometryNodeExport) -> Vec<usize> {
    gne.nodes.iter().enumerate().filter(|(_, n)| n.node_type == GeoNodeType::Input).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn gne_output_nodes(gne: &GeometryNodeExport) -> Vec<usize> {
    gne.nodes.iter().enumerate().filter(|(_, n)| n.node_type == GeoNodeType::Output).map(|(i, _)| i).collect()
}

#[allow(dead_code)]
pub fn gne_validate(gne: &GeometryNodeExport) -> bool {
    gne.links.iter().all(|l| (l[0] as usize) < gne.nodes.len() && (l[1] as usize) < gne.nodes.len())
}

#[allow(dead_code)]
pub fn gne_to_json(gne: &GeometryNodeExport) -> String {
    format!("{{\"nodes\":{},\"links\":{}}}", gne.nodes.len(), gne.links.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> GeometryNodeExport {
        let mut g = new_geometry_node_export();
        let a = gne_add_node(&mut g, "input", GeoNodeType::Input);
        let b = gne_add_node(&mut g, "transform", GeoNodeType::Transform);
        let c = gne_add_node(&mut g, "output", GeoNodeType::Output);
        gne_add_link(&mut g, a, b);
        gne_add_link(&mut g, b, c);
        g
    }

    #[test] fn test_new() { assert_eq!(gne_node_count(&new_geometry_node_export()), 0); }
    #[test] fn test_add_node() { assert_eq!(gne_node_count(&sample()), 3); }
    #[test] fn test_link_count() { assert_eq!(gne_link_count(&sample()), 2); }
    #[test] fn test_input_nodes() { assert_eq!(gne_input_nodes(&sample()).len(), 1); }
    #[test] fn test_output_nodes() { assert_eq!(gne_output_nodes(&sample()).len(), 1); }
    #[test] fn test_validate() { assert!(gne_validate(&sample())); }
    #[test] fn test_to_json() { assert!(gne_to_json(&sample()).contains("nodes")); }
    #[test] fn test_node_name() { let g = sample(); assert_eq!(g.nodes[0].name, "input"); }
    #[test] fn test_outputs() { let g = sample(); assert!(!g.nodes[0].outputs.is_empty()); }
    #[test] fn test_inputs() { let g = sample(); assert!(!g.nodes[1].inputs.is_empty()); }
}
