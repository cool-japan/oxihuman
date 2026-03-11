// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Blend tree node export: serialise animator blend trees.

/// Node type in a blend tree.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlendNodeType {
    Clip,
    Blend1D,
    Blend2D,
    StateMachine,
}

/// A blend tree node.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendTreeNode {
    pub id: u32,
    pub name: String,
    pub node_type: BlendNodeType,
    pub children: Vec<u32>,
    pub weight: f32,
}

/// Blend tree export container.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendTreeExport {
    pub nodes: Vec<BlendTreeNode>,
    pub root_id: Option<u32>,
}

/// Create an empty blend tree export.
#[allow(dead_code)]
pub fn new_blend_tree_export() -> BlendTreeExport {
    BlendTreeExport {
        nodes: Vec::new(),
        root_id: None,
    }
}

/// Add a node.
#[allow(dead_code)]
pub fn add_blend_node(exp: &mut BlendTreeExport, node: BlendTreeNode) {
    exp.nodes.push(node);
}

/// Set root.
#[allow(dead_code)]
pub fn set_blend_root(exp: &mut BlendTreeExport, id: u32) {
    exp.root_id = Some(id);
}

/// Node count.
#[allow(dead_code)]
pub fn blend_node_count(exp: &BlendTreeExport) -> usize {
    exp.nodes.len()
}

/// Find node by id.
#[allow(dead_code)]
pub fn find_blend_node(exp: &BlendTreeExport, id: u32) -> Option<&BlendTreeNode> {
    exp.nodes.iter().find(|n| n.id == id)
}

/// Total child connections.
#[allow(dead_code)]
pub fn total_connections(exp: &BlendTreeExport) -> usize {
    exp.nodes.iter().map(|n| n.children.len()).sum()
}

/// Nodes of a given type.
#[allow(dead_code)]
pub fn nodes_of_type<'a>(exp: &'a BlendTreeExport, t: &BlendNodeType) -> Vec<&'a BlendTreeNode> {
    exp.nodes.iter().filter(|n| &n.node_type == t).collect()
}

/// Serialise to JSON.
#[allow(dead_code)]
pub fn blend_tree_to_json(exp: &BlendTreeExport) -> String {
    format!(
        "{{\"node_count\":{},\"root_id\":{}}}",
        blend_node_count(exp),
        exp.root_id.map_or("null".to_string(), |id| id.to_string())
    )
}

/// Validate: all child ids exist.
#[allow(dead_code)]
pub fn validate_blend_tree(exp: &BlendTreeExport) -> bool {
    let ids: std::collections::HashSet<u32> = exp.nodes.iter().map(|n| n.id).collect();
    exp.nodes
        .iter()
        .all(|n| n.children.iter().all(|&c| ids.contains(&c)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip_node(id: u32, name: &str) -> BlendTreeNode {
        BlendTreeNode {
            id,
            name: name.to_string(),
            node_type: BlendNodeType::Clip,
            children: Vec::new(),
            weight: 1.0,
        }
    }

    #[test]
    fn new_export_empty() {
        let exp = new_blend_tree_export();
        assert_eq!(blend_node_count(&exp), 0);
    }

    #[test]
    fn add_node_increments() {
        let mut exp = new_blend_tree_export();
        add_blend_node(&mut exp, clip_node(0, "idle"));
        assert_eq!(blend_node_count(&exp), 1);
    }

    #[test]
    fn find_existing() {
        let mut exp = new_blend_tree_export();
        add_blend_node(&mut exp, clip_node(5, "run"));
        assert!(find_blend_node(&exp, 5).is_some());
    }

    #[test]
    fn find_missing_none() {
        let exp = new_blend_tree_export();
        assert!(find_blend_node(&exp, 99).is_none());
    }

    #[test]
    fn set_root() {
        let mut exp = new_blend_tree_export();
        add_blend_node(&mut exp, clip_node(0, "root"));
        set_blend_root(&mut exp, 0);
        assert!(exp.root_id.is_some_and(|id| id == 0));
    }

    #[test]
    fn total_connections_none() {
        let mut exp = new_blend_tree_export();
        add_blend_node(&mut exp, clip_node(0, "a"));
        assert_eq!(total_connections(&exp), 0);
    }

    #[test]
    fn nodes_of_type_filter() {
        let mut exp = new_blend_tree_export();
        add_blend_node(&mut exp, clip_node(0, "a"));
        add_blend_node(
            &mut exp,
            BlendTreeNode {
                id: 1,
                name: "b1d".to_string(),
                node_type: BlendNodeType::Blend1D,
                children: vec![0],
                weight: 0.5,
            },
        );
        assert_eq!(nodes_of_type(&exp, &BlendNodeType::Clip).len(), 1);
    }

    #[test]
    fn validate_no_dangling_children() {
        let mut exp = new_blend_tree_export();
        add_blend_node(&mut exp, clip_node(0, "a"));
        add_blend_node(&mut exp, clip_node(1, "b"));
        assert!(validate_blend_tree(&exp));
    }

    #[test]
    fn json_contains_node_count() {
        let exp = new_blend_tree_export();
        let j = blend_tree_to_json(&exp);
        assert!(j.contains("node_count"));
    }

    #[test]
    fn weight_in_range() {
        let n = clip_node(0, "t");
        assert!((0.0..=1.0).contains(&n.weight));
    }
}
