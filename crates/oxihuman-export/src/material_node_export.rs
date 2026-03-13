// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Material node graph export.

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Shader,
    Texture,
    Math,
    Mix,
    Output,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialNode {
    pub id: u32,
    pub name: String,
    pub node_type: NodeType,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MaterialNodeExport {
    pub nodes: Vec<MaterialNode>,
    pub connections: Vec<(u32, u32)>,
}

#[allow(dead_code)]
pub fn new_material_node_export() -> MaterialNodeExport {
    MaterialNodeExport { nodes: Vec::new(), connections: Vec::new() }
}

#[allow(dead_code)]
pub fn mn_add_node(exp: &mut MaterialNodeExport, node: MaterialNode) {
    exp.nodes.push(node);
}

#[allow(dead_code)]
pub fn mn_connect(exp: &mut MaterialNodeExport, from_id: u32, to_id: u32) {
    exp.connections.push((from_id, to_id));
}

#[allow(dead_code)]
pub fn mn_node_count(exp: &MaterialNodeExport) -> usize {
    exp.nodes.len()
}

#[allow(dead_code)]
pub fn mn_connection_count(exp: &MaterialNodeExport) -> usize {
    exp.connections.len()
}

#[allow(dead_code)]
pub fn mn_get_node(exp: &MaterialNodeExport, id: u32) -> Option<&MaterialNode> {
    exp.nodes.iter().find(|n| n.id == id)
}

#[allow(dead_code)]
pub fn mn_to_json(exp: &MaterialNodeExport) -> String {
    format!(
        r#"{{"nodes":{},"connections":{}}}"#,
        exp.nodes.len(),
        exp.connections.len()
    )
}

#[allow(dead_code)]
pub fn mn_node_type_name(nt: &NodeType) -> &'static str {
    match nt {
        NodeType::Shader => "Shader",
        NodeType::Texture => "Texture",
        NodeType::Math => "Math",
        NodeType::Mix => "Mix",
        NodeType::Output => "Output",
    }
}

#[allow(dead_code)]
pub fn mn_validate(exp: &MaterialNodeExport) -> bool {
    exp.nodes.iter().all(|n| !n.name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(id: u32, name: &str, nt: NodeType) -> MaterialNode {
        MaterialNode {
            id,
            name: name.to_string(),
            node_type: nt,
            inputs: vec!["Base Color".to_string()],
            outputs: vec!["BSDF".to_string()],
        }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_material_node_export();
        assert_eq!(mn_node_count(&exp), 0);
        assert_eq!(mn_connection_count(&exp), 0);
    }

    #[test]
    fn add_node_increments() {
        let mut exp = new_material_node_export();
        mn_add_node(&mut exp, make_node(1, "shader", NodeType::Shader));
        assert_eq!(mn_node_count(&exp), 1);
    }

    #[test]
    fn connect_nodes() {
        let mut exp = new_material_node_export();
        mn_add_node(&mut exp, make_node(1, "tex", NodeType::Texture));
        mn_add_node(&mut exp, make_node(2, "shader", NodeType::Shader));
        mn_connect(&mut exp, 1, 2);
        assert_eq!(mn_connection_count(&exp), 1);
    }

    #[test]
    fn get_node_by_id() {
        let mut exp = new_material_node_export();
        mn_add_node(&mut exp, make_node(42, "out", NodeType::Output));
        assert!(mn_get_node(&exp, 42).is_some());
        assert!(mn_get_node(&exp, 99).is_none());
    }

    #[test]
    fn type_names_correct() {
        assert_eq!(mn_node_type_name(&NodeType::Shader), "Shader");
        assert_eq!(mn_node_type_name(&NodeType::Output), "Output");
        assert_eq!(mn_node_type_name(&NodeType::Mix), "Mix");
    }

    #[test]
    fn validate_ok() {
        let mut exp = new_material_node_export();
        mn_add_node(&mut exp, make_node(1, "node", NodeType::Math));
        assert!(mn_validate(&exp));
    }

    #[test]
    fn validate_empty_name_fails() {
        let mut exp = new_material_node_export();
        mn_add_node(&mut exp, make_node(1, "", NodeType::Math));
        assert!(!mn_validate(&exp));
    }

    #[test]
    fn to_json_has_fields() {
        let exp = new_material_node_export();
        let json = mn_to_json(&exp);
        assert!(json.contains("nodes"));
        assert!(json.contains("connections"));
    }
}
