// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Hierarchical tree index: parent-child relationships by u32 id.

use std::collections::HashMap;

/// A node in the tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: u32,
    pub label: String,
    pub parent: Option<u32>,
    pub children: Vec<u32>,
}

/// Tree index structure.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TreeIndex {
    nodes: HashMap<u32, TreeNode>,
    roots: Vec<u32>,
    next_id: u32,
}

/// Create a new empty `TreeIndex`.
#[allow(dead_code)]
pub fn new_tree_index() -> TreeIndex {
    TreeIndex::default()
}

/// Add a root node and return its id.
#[allow(dead_code)]
pub fn ti_add_root(ti: &mut TreeIndex, label: &str) -> u32 {
    let id = ti.next_id;
    ti.next_id += 1;
    ti.nodes.insert(
        id,
        TreeNode {
            id,
            label: label.to_string(),
            parent: None,
            children: Vec::new(),
        },
    );
    ti.roots.push(id);
    id
}

/// Add a child under `parent_id` and return its id.
#[allow(dead_code)]
pub fn ti_add_child(ti: &mut TreeIndex, parent_id: u32, label: &str) -> Option<u32> {
    if !ti.nodes.contains_key(&parent_id) {
        return None;
    }
    let id = ti.next_id;
    ti.next_id += 1;
    ti.nodes.insert(
        id,
        TreeNode {
            id,
            label: label.to_string(),
            parent: Some(parent_id),
            children: Vec::new(),
        },
    );
    if let Some(parent) = ti.nodes.get_mut(&parent_id) {
        parent.children.push(id);
    }
    Some(id)
}

/// Get label for a node.
#[allow(dead_code)]
pub fn ti_label(ti: &TreeIndex, id: u32) -> Option<&str> {
    ti.nodes.get(&id).map(|n| n.label.as_str())
}

/// Get parent id.
#[allow(dead_code)]
pub fn ti_parent(ti: &TreeIndex, id: u32) -> Option<u32> {
    ti.nodes.get(&id).and_then(|n| n.parent)
}

/// Get children of a node.
#[allow(dead_code)]
pub fn ti_children(ti: &TreeIndex, id: u32) -> Vec<u32> {
    ti.nodes
        .get(&id)
        .map(|n| n.children.clone())
        .unwrap_or_default()
}

/// Total number of nodes.
#[allow(dead_code)]
pub fn ti_count(ti: &TreeIndex) -> usize {
    ti.nodes.len()
}

/// Depth of a node from root (root = 0).
#[allow(dead_code)]
pub fn ti_depth(ti: &TreeIndex, id: u32) -> usize {
    let mut depth = 0;
    let mut cur = id;
    while let Some(p) = ti.nodes.get(&cur).and_then(|n| n.parent) {
        cur = p;
        depth += 1;
    }
    depth
}

/// Collect all descendants (BFS).
#[allow(dead_code)]
pub fn ti_descendants(ti: &TreeIndex, id: u32) -> Vec<u32> {
    let mut result = Vec::new();
    let mut queue = vec![id];
    while !queue.is_empty() {
        let cur = queue.remove(0);
        if let Some(node) = ti.nodes.get(&cur) {
            for &child in &node.children {
                result.push(child);
                queue.push(child);
            }
        }
    }
    result
}

/// List root node ids.
#[allow(dead_code)]
pub fn ti_roots(ti: &TreeIndex) -> &[u32] {
    &ti.roots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_root() {
        let mut ti = new_tree_index();
        let id = ti_add_root(&mut ti, "root");
        assert_eq!(ti_label(&ti, id), Some("root"));
    }

    #[test]
    fn test_add_child() {
        let mut ti = new_tree_index();
        let root = ti_add_root(&mut ti, "root");
        let child = ti_add_child(&mut ti, root, "child").unwrap();
        assert_eq!(ti_parent(&ti, child), Some(root));
    }

    #[test]
    fn test_children() {
        let mut ti = new_tree_index();
        let r = ti_add_root(&mut ti, "r");
        let c1 = ti_add_child(&mut ti, r, "c1").unwrap();
        let c2 = ti_add_child(&mut ti, r, "c2").unwrap();
        let children = ti_children(&ti, r);
        assert!(children.contains(&c1) && children.contains(&c2));
    }

    #[test]
    fn test_depth() {
        let mut ti = new_tree_index();
        let r = ti_add_root(&mut ti, "r");
        let c = ti_add_child(&mut ti, r, "c").unwrap();
        let gc = ti_add_child(&mut ti, c, "gc").unwrap();
        assert_eq!(ti_depth(&ti, r), 0);
        assert_eq!(ti_depth(&ti, c), 1);
        assert_eq!(ti_depth(&ti, gc), 2);
    }

    #[test]
    fn test_descendants() {
        let mut ti = new_tree_index();
        let r = ti_add_root(&mut ti, "r");
        let c = ti_add_child(&mut ti, r, "c").unwrap();
        ti_add_child(&mut ti, c, "gc").unwrap();
        let desc = ti_descendants(&ti, r);
        assert_eq!(desc.len(), 2);
    }

    #[test]
    fn test_count() {
        let mut ti = new_tree_index();
        let r = ti_add_root(&mut ti, "r");
        ti_add_child(&mut ti, r, "c").unwrap();
        assert_eq!(ti_count(&ti), 2);
    }

    #[test]
    fn test_roots_list() {
        let mut ti = new_tree_index();
        let r1 = ti_add_root(&mut ti, "r1");
        let r2 = ti_add_root(&mut ti, "r2");
        let roots = ti_roots(&ti);
        assert!(roots.contains(&r1) && roots.contains(&r2));
    }

    #[test]
    fn test_invalid_parent() {
        let mut ti = new_tree_index();
        let result = ti_add_child(&mut ti, 999, "x");
        assert!(result.is_none());
    }

    #[test]
    fn test_no_parent_for_root() {
        let mut ti = new_tree_index();
        let r = ti_add_root(&mut ti, "r");
        assert!(ti_parent(&ti, r).is_none());
    }
}
