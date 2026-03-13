// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HierarchyNode {
    pub id: u32,
    pub name: String,
    pub parent_id: Option<u32>,
    pub children: Vec<u32>,
    pub object_type: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ObjectHierarchyExport {
    pub nodes: Vec<HierarchyNode>,
}

#[allow(dead_code)]
pub fn new_object_hierarchy_export() -> ObjectHierarchyExport {
    ObjectHierarchyExport { nodes: Vec::new() }
}

#[allow(dead_code)]
pub fn oh_add_node(exp: &mut ObjectHierarchyExport, id: u32, name: &str, object_type: &str) {
    exp.nodes.push(HierarchyNode {
        id,
        name: name.to_string(),
        parent_id: None,
        children: Vec::new(),
        object_type: object_type.to_string(),
    });
}

#[allow(dead_code)]
pub fn oh_set_parent(exp: &mut ObjectHierarchyExport, child_id: u32, parent_id: u32) {
    // Set parent on child
    if let Some(child) = exp.nodes.iter_mut().find(|n| n.id == child_id) {
        child.parent_id = Some(parent_id);
    }
    // Add child to parent's children list
    if let Some(parent) = exp.nodes.iter_mut().find(|n| n.id == parent_id) {
        if !parent.children.contains(&child_id) {
            parent.children.push(child_id);
        }
    }
}

#[allow(dead_code)]
pub fn oh_node_count(exp: &ObjectHierarchyExport) -> usize {
    exp.nodes.len()
}

#[allow(dead_code)]
pub fn oh_get_node(exp: &ObjectHierarchyExport, id: u32) -> Option<&HierarchyNode> {
    exp.nodes.iter().find(|n| n.id == id)
}

#[allow(dead_code)]
pub fn oh_root_nodes(exp: &ObjectHierarchyExport) -> Vec<u32> {
    exp.nodes.iter().filter(|n| n.parent_id.is_none()).map(|n| n.id).collect()
}

#[allow(dead_code)]
pub fn oh_children_of(exp: &ObjectHierarchyExport, id: u32) -> Vec<u32> {
    exp.nodes
        .iter()
        .find(|n| n.id == id)
        .map(|n| n.children.clone())
        .unwrap_or_default()
}

#[allow(dead_code)]
pub fn oh_to_json(exp: &ObjectHierarchyExport) -> String {
    format!(
        r#"{{"node_count":{},"root_count":{}}}"#,
        exp.nodes.len(),
        oh_root_nodes(exp).len()
    )
}

#[allow(dead_code)]
pub fn oh_depth_of(exp: &ObjectHierarchyExport, id: u32) -> usize {
    let mut depth = 0usize;
    let mut current = id;
    while let Some(n) = exp.nodes.iter().find(|n| n.id == current) {
        if let Some(parent) = n.parent_id {
            depth += 1;
            current = parent;
        } else {
            break;
        }
    }
    depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let e = new_object_hierarchy_export();
        assert_eq!(oh_node_count(&e), 0);
    }

    #[test]
    fn test_add_node() {
        let mut e = new_object_hierarchy_export();
        oh_add_node(&mut e, 1, "Root", "MESH");
        assert_eq!(oh_node_count(&e), 1);
    }

    #[test]
    fn test_set_parent() {
        let mut e = new_object_hierarchy_export();
        oh_add_node(&mut e, 1, "Parent", "EMPTY");
        oh_add_node(&mut e, 2, "Child", "MESH");
        oh_set_parent(&mut e, 2, 1);
        let child = oh_get_node(&e, 2).expect("should succeed");
        assert_eq!(child.parent_id, Some(1));
    }

    #[test]
    fn test_root_nodes() {
        let mut e = new_object_hierarchy_export();
        oh_add_node(&mut e, 1, "Root", "EMPTY");
        oh_add_node(&mut e, 2, "Child", "MESH");
        oh_set_parent(&mut e, 2, 1);
        let roots = oh_root_nodes(&e);
        assert_eq!(roots, vec![1]);
    }

    #[test]
    fn test_children_of() {
        let mut e = new_object_hierarchy_export();
        oh_add_node(&mut e, 1, "Root", "EMPTY");
        oh_add_node(&mut e, 2, "Child", "MESH");
        oh_set_parent(&mut e, 2, 1);
        let children = oh_children_of(&e, 1);
        assert_eq!(children, vec![2]);
    }

    #[test]
    fn test_depth_of_root() {
        let mut e = new_object_hierarchy_export();
        oh_add_node(&mut e, 1, "Root", "EMPTY");
        assert_eq!(oh_depth_of(&e, 1), 0);
    }

    #[test]
    fn test_depth_of_child() {
        let mut e = new_object_hierarchy_export();
        oh_add_node(&mut e, 1, "Root", "EMPTY");
        oh_add_node(&mut e, 2, "Child", "MESH");
        oh_set_parent(&mut e, 2, 1);
        assert_eq!(oh_depth_of(&e, 2), 1);
    }

    #[test]
    fn test_to_json() {
        let mut e = new_object_hierarchy_export();
        oh_add_node(&mut e, 1, "Root", "MESH");
        let j = oh_to_json(&e);
        assert!(j.contains("node_count"));
        assert!(j.contains("root_count"));
    }
}
