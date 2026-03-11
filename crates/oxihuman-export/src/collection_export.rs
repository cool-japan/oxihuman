// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Collection export (Blender-style scene collection tree).

/* ── legacy API (kept) ── */

pub struct CollectionObject {
    pub name: String,
    pub object_type: u8,
}

pub struct CollectionExport {
    pub name: String,
    pub objects: Vec<CollectionObject>,
    pub children: Vec<String>,
}

pub fn new_collection_export(name: &str) -> CollectionExport {
    CollectionExport {
        name: name.to_string(),
        objects: Vec::new(),
        children: Vec::new(),
    }
}

/* ── spec functions (wave 150B) ── */

/// Spec-style collection node.
#[derive(Debug, Clone)]
pub struct CollectionNode {
    pub name: String,
    pub objects: Vec<String>,
    pub children: Vec<String>,
}

/// Create a new `CollectionNode`.
pub fn new_collection_node(name: &str) -> CollectionNode {
    CollectionNode {
        name: name.to_string(),
        objects: Vec::new(),
        children: Vec::new(),
    }
}

/// Add a child collection name.
pub fn collection_push_child(node: &mut CollectionNode, child: &str) {
    node.children.push(child.to_string());
}

/// Add an object name.
pub fn collection_push_object(node: &mut CollectionNode, obj: &str) {
    node.objects.push(obj.to_string());
}

/// Serialize to JSON.
pub fn collection_to_json(node: &CollectionNode) -> String {
    format!(
        "{{\"name\":\"{}\",\"objects\":{},\"children\":{}}}",
        node.name,
        node.objects.len(),
        node.children.len()
    )
}

/// Number of objects in this node.
pub fn collection_object_count(node: &CollectionNode) -> usize {
    node.objects.len()
}

/// Number of child collections.
pub fn collection_child_count(node: &CollectionNode) -> usize {
    node.children.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collection_node() {
        let n = new_collection_node("scene");
        assert_eq!(n.name, "scene");
    }

    #[test]
    fn test_push_object() {
        let mut n = new_collection_node("s");
        collection_push_object(&mut n, "Mesh");
        assert_eq!(collection_object_count(&n), 1);
    }

    #[test]
    fn test_push_child() {
        let mut n = new_collection_node("s");
        collection_push_child(&mut n, "sub");
        assert_eq!(collection_child_count(&n), 1);
    }

    #[test]
    fn test_collection_to_json() {
        let n = new_collection_node("s");
        let j = collection_to_json(&n);
        assert!(j.contains("\"name\":\"s\""));
    }

    #[test]
    fn test_counts_empty() {
        let n = new_collection_node("s");
        assert_eq!(collection_object_count(&n), 0);
        assert_eq!(collection_child_count(&n), 0);
    }
}
