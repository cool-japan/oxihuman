// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NodeConnection {
    pub from_node: u32,
    pub from_socket: u32,
    pub to_node: u32,
    pub to_socket: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompositorNode {
    pub id: u32,
    pub node_type: String,
    pub name: String,
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NodeTreeExport {
    pub name: String,
    pub nodes: Vec<CompositorNode>,
    pub connections: Vec<NodeConnection>,
}

#[allow(dead_code)]
pub fn new_node_tree_export(name: &str) -> NodeTreeExport {
    NodeTreeExport {
        name: name.to_string(),
        nodes: Vec::new(),
        connections: Vec::new(),
    }
}

#[allow(dead_code)]
pub fn nt_add_node(exp: &mut NodeTreeExport, node: CompositorNode) {
    exp.nodes.push(node);
}

#[allow(dead_code)]
pub fn nt_connect(exp: &mut NodeTreeExport, from_node: u32, from_socket: u32, to_node: u32, to_socket: u32) {
    exp.connections.push(NodeConnection { from_node, from_socket, to_node, to_socket });
}

#[allow(dead_code)]
pub fn nt_node_count(exp: &NodeTreeExport) -> usize {
    exp.nodes.len()
}

#[allow(dead_code)]
pub fn nt_connection_count(exp: &NodeTreeExport) -> usize {
    exp.connections.len()
}

#[allow(dead_code)]
pub fn nt_get_node(exp: &NodeTreeExport, id: u32) -> Option<&CompositorNode> {
    exp.nodes.iter().find(|n| n.id == id)
}

#[allow(dead_code)]
pub fn nt_to_json(exp: &NodeTreeExport) -> String {
    format!(
        r#"{{"name":"{}","node_count":{},"connection_count":{}}}"#,
        exp.name,
        exp.nodes.len(),
        exp.connections.len()
    )
}

#[allow(dead_code)]
pub fn nt_validate(exp: &NodeTreeExport) -> bool {
    !exp.name.is_empty()
}

#[allow(dead_code)]
pub fn nt_remove_node(exp: &mut NodeTreeExport, id: u32) {
    exp.nodes.retain(|n| n.id != id);
    exp.connections.retain(|c| c.from_node != id && c.to_node != id);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: u32, t: &str) -> CompositorNode {
        CompositorNode {
            id,
            node_type: t.to_string(),
            name: format!("Node{id}"),
            x: 0.0,
            y: 0.0,
        }
    }

    #[test]
    fn test_new_export() {
        let e = new_node_tree_export("Compositor");
        assert_eq!(nt_node_count(&e), 0);
        assert_eq!(nt_connection_count(&e), 0);
    }

    #[test]
    fn test_add_node() {
        let mut e = new_node_tree_export("Compositor");
        nt_add_node(&mut e, make_node(1, "RenderLayers"));
        assert_eq!(nt_node_count(&e), 1);
    }

    #[test]
    fn test_connect() {
        let mut e = new_node_tree_export("Compositor");
        nt_add_node(&mut e, make_node(1, "RenderLayers"));
        nt_add_node(&mut e, make_node(2, "Composite"));
        nt_connect(&mut e, 1, 0, 2, 0);
        assert_eq!(nt_connection_count(&e), 1);
    }

    #[test]
    fn test_get_node() {
        let mut e = new_node_tree_export("Compositor");
        nt_add_node(&mut e, make_node(5, "Blur"));
        let n = nt_get_node(&e, 5).expect("should succeed");
        assert_eq!(n.node_type, "Blur");
    }

    #[test]
    fn test_get_node_missing() {
        let e = new_node_tree_export("Compositor");
        assert!(nt_get_node(&e, 99).is_none());
    }

    #[test]
    fn test_remove_node() {
        let mut e = new_node_tree_export("Compositor");
        nt_add_node(&mut e, make_node(1, "A"));
        nt_add_node(&mut e, make_node(2, "B"));
        nt_connect(&mut e, 1, 0, 2, 0);
        nt_remove_node(&mut e, 1);
        assert_eq!(nt_node_count(&e), 1);
        assert_eq!(nt_connection_count(&e), 0);
    }

    #[test]
    fn test_validate() {
        let e = new_node_tree_export("Tree");
        assert!(nt_validate(&e));
    }

    #[test]
    fn test_to_json() {
        let mut e = new_node_tree_export("Tree");
        nt_add_node(&mut e, make_node(1, "Input"));
        let j = nt_to_json(&e);
        assert!(j.contains("node_count"));
        assert!(j.contains("Tree"));
    }
}
