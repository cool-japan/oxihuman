// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cocos Creator scene export stub.

/// A Cocos scene node.
#[derive(Debug, Clone)]
pub struct CocosNode {
    pub name: String,
    pub position: [f32; 3],
    pub children: Vec<CocosNode>,
}

impl CocosNode {
    /// Create a new scene node.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            position: [0.0; 3],
            children: Vec::new(),
        }
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: CocosNode) {
        self.children.push(child);
    }

    /// Total node count including self and descendants.
    pub fn total_node_count(&self) -> usize {
        1 + self
            .children
            .iter()
            .map(|c| c.total_node_count())
            .sum::<usize>()
    }
}

/// A Cocos Creator scene.
#[derive(Debug, Clone)]
pub struct CocosScene {
    pub name: String,
    pub root: CocosNode,
}

impl CocosScene {
    /// Create a new scene.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            root: CocosNode::new("Canvas"),
        }
    }

    /// Total node count.
    pub fn total_node_count(&self) -> usize {
        self.root.total_node_count()
    }
}

/// Serialize scene to JSON string (stub).
pub fn export_cocos_json(scene: &CocosScene) -> String {
    format!(
        "{{\"scene\":\"{}\",\"node_count\":{}}}",
        scene.name,
        scene.total_node_count()
    )
}

/// Find node by name (depth-first search).
pub fn find_node<'a>(node: &'a CocosNode, name: &str) -> Option<&'a CocosNode> {
    if node.name == name {
        return Some(node);
    }
    node.children.iter().find_map(|c| find_node(c, name))
}

/// Depth of the scene graph.
pub fn scene_depth(node: &CocosNode) -> usize {
    if node.children.is_empty() {
        return 1;
    }
    1 + node.children.iter().map(scene_depth).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_scene() -> CocosScene {
        let mut scene = CocosScene::new("MainScene");
        let mut child = CocosNode::new("Player");
        child.add_child(CocosNode::new("Weapon"));
        scene.root.add_child(child);
        scene
    }

    #[test]
    fn test_total_node_count() {
        /* total node count traverses hierarchy */
        let s = sample_scene();
        assert_eq!(s.total_node_count(), 3);
    }

    #[test]
    fn test_export_cocos_json_not_empty() {
        /* JSON output is non-empty */
        let s = sample_scene();
        assert!(!export_cocos_json(&s).is_empty());
    }

    #[test]
    fn test_export_cocos_json_contains_name() {
        /* JSON contains scene name */
        let s = sample_scene();
        assert!(export_cocos_json(&s).contains("MainScene"));
    }

    #[test]
    fn test_find_node_found() {
        /* find_node locates existing node */
        let s = sample_scene();
        assert!(find_node(&s.root, "Player").is_some());
    }

    #[test]
    fn test_find_node_deep() {
        /* find_node finds deeply nested nodes */
        let s = sample_scene();
        assert!(find_node(&s.root, "Weapon").is_some());
    }

    #[test]
    fn test_find_node_not_found() {
        /* find_node returns None for missing node */
        let s = sample_scene();
        assert!(find_node(&s.root, "Enemy").is_none());
    }

    #[test]
    fn test_scene_depth() {
        /* depth of scene graph is correct */
        let s = sample_scene();
        assert_eq!(scene_depth(&s.root), 3);
    }

    #[test]
    fn test_empty_scene_node_count() {
        /* new scene has one root node */
        let s = CocosScene::new("Empty");
        assert_eq!(s.total_node_count(), 1);
    }
}
