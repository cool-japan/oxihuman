// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export shader/material node graphs for rendering engines.

/// Node in a material graph.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MatGraphNode {
    pub id: u32,
    pub name: String,
    pub node_type: String,
    pub inputs: Vec<(String, f32)>,
}

/// Connection between nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MatGraphConnection {
    pub from_node: u32,
    pub from_output: String,
    pub to_node: u32,
    pub to_input: String,
}

/// Material graph export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialGraphExport {
    pub nodes: Vec<MatGraphNode>,
    pub connections: Vec<MatGraphConnection>,
}

#[allow(dead_code)]
pub fn new_material_graph() -> MaterialGraphExport {
    MaterialGraphExport { nodes: Vec::new(), connections: Vec::new() }
}

#[allow(dead_code)]
pub fn mg_add_node(graph: &mut MaterialGraphExport, id: u32, name: &str, ntype: &str) -> usize {
    let idx = graph.nodes.len();
    graph.nodes.push(MatGraphNode { id, name: name.to_string(), node_type: ntype.to_string(), inputs: Vec::new() });
    idx
}

#[allow(dead_code)]
pub fn mg_add_input(graph: &mut MaterialGraphExport, node_idx: usize, name: &str, value: f32) {
    if node_idx < graph.nodes.len() {
        graph.nodes[node_idx].inputs.push((name.to_string(), value));
    }
}

#[allow(dead_code)]
pub fn mg_connect(graph: &mut MaterialGraphExport, from: u32, from_out: &str, to: u32, to_in: &str) {
    graph.connections.push(MatGraphConnection {
        from_node: from, from_output: from_out.to_string(),
        to_node: to, to_input: to_in.to_string(),
    });
}

#[allow(dead_code)]
pub fn mg_node_count(graph: &MaterialGraphExport) -> usize { graph.nodes.len() }

#[allow(dead_code)]
pub fn mg_connection_count(graph: &MaterialGraphExport) -> usize { graph.connections.len() }

#[allow(dead_code)]
pub fn mg_find_node(graph: &MaterialGraphExport, id: u32) -> Option<usize> {
    graph.nodes.iter().position(|n| n.id == id)
}

#[allow(dead_code)]
pub fn mg_to_json(graph: &MaterialGraphExport) -> String {
    let nodes: Vec<String> = graph.nodes.iter().map(|n| {
        format!(r#"{{"id":{},"name":"{}","type":"{}","inputs":{}}}"#, n.id, n.name, n.node_type, n.inputs.len())
    }).collect();
    let conns: Vec<String> = graph.connections.iter().map(|c| {
        format!(r#"{{"from":{},"to":{}}}"#, c.from_node, c.to_node)
    }).collect();
    format!(r#"{{"nodes":[{}],"connections":[{}]}}"#, nodes.join(","), conns.join(","))
}

#[allow(dead_code)]
pub fn mg_validate(graph: &MaterialGraphExport) -> bool {
    let ids: std::collections::HashSet<u32> = graph.nodes.iter().map(|n| n.id).collect();
    graph.connections.iter().all(|c| ids.contains(&c.from_node) && ids.contains(&c.to_node))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        let g = new_material_graph();
        assert_eq!(mg_node_count(&g), 0);
    }

    #[test]
    fn test_add_node() {
        let mut g = new_material_graph();
        mg_add_node(&mut g, 1, "Diffuse", "shader");
        assert_eq!(mg_node_count(&g), 1);
    }

    #[test]
    fn test_add_input() {
        let mut g = new_material_graph();
        let idx = mg_add_node(&mut g, 1, "PBR", "shader");
        mg_add_input(&mut g, idx, "roughness", 0.5);
        assert_eq!(g.nodes[0].inputs.len(), 1);
    }

    #[test]
    fn test_connect() {
        let mut g = new_material_graph();
        mg_add_node(&mut g, 1, "Tex", "texture");
        mg_add_node(&mut g, 2, "Shader", "shader");
        mg_connect(&mut g, 1, "color", 2, "base_color");
        assert_eq!(mg_connection_count(&g), 1);
    }

    #[test]
    fn test_find_node() {
        let mut g = new_material_graph();
        mg_add_node(&mut g, 42, "N", "t");
        assert_eq!(mg_find_node(&g, 42), Some(0));
    }

    #[test]
    fn test_find_missing() {
        let g = new_material_graph();
        assert_eq!(mg_find_node(&g, 99), None);
    }

    #[test]
    fn test_to_json() {
        let mut g = new_material_graph();
        mg_add_node(&mut g, 1, "A", "t");
        let json = mg_to_json(&g);
        assert!(json.contains("nodes"));
    }

    #[test]
    fn test_validate_ok() {
        let mut g = new_material_graph();
        mg_add_node(&mut g, 1, "A", "t");
        mg_add_node(&mut g, 2, "B", "t");
        mg_connect(&mut g, 1, "out", 2, "in");
        assert!(mg_validate(&g));
    }

    #[test]
    fn test_validate_fail() {
        let mut g = new_material_graph();
        mg_add_node(&mut g, 1, "A", "t");
        mg_connect(&mut g, 1, "out", 99, "in");
        assert!(!mg_validate(&g));
    }

    #[test]
    fn test_multiple_inputs() {
        let mut g = new_material_graph();
        let idx = mg_add_node(&mut g, 1, "PBR", "shader");
        mg_add_input(&mut g, idx, "roughness", 0.5);
        mg_add_input(&mut g, idx, "metallic", 1.0);
        assert_eq!(g.nodes[0].inputs.len(), 2);
    }

}
