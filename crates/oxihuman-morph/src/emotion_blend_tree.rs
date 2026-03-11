// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Emotion node blend tree stub.

/// Blend operation for combining child nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendOp {
    Add,
    Multiply,
    Override,
}

/// A node in the emotion blend tree.
#[derive(Debug, Clone)]
pub struct EmotionNode {
    pub name: String,
    pub weight: f32,
    pub children: Vec<usize>,
    pub blend_op: BlendOp,
}

/// Emotion blend tree.
#[derive(Debug, Clone)]
pub struct EmotionBlendTree {
    pub nodes: Vec<EmotionNode>,
    pub root: Option<usize>,
    pub output_dim: usize,
    pub enabled: bool,
}

impl EmotionBlendTree {
    pub fn new(output_dim: usize) -> Self {
        EmotionBlendTree {
            nodes: Vec::new(),
            root: None,
            output_dim,
            enabled: true,
        }
    }
}

/// Create a new emotion blend tree.
pub fn new_emotion_blend_tree(output_dim: usize) -> EmotionBlendTree {
    EmotionBlendTree::new(output_dim)
}

/// Add a node and return its index.
pub fn ebt_add_node(tree: &mut EmotionBlendTree, node: EmotionNode) -> usize {
    let idx = tree.nodes.len();
    tree.nodes.push(node);
    idx
}

/// Set the root node index.
pub fn ebt_set_root(tree: &mut EmotionBlendTree, root: usize) {
    tree.root = Some(root);
}

/// Evaluate the tree to produce output weights (stub: zeroed).
pub fn ebt_evaluate(tree: &EmotionBlendTree) -> Vec<f32> {
    /* Stub: returns zeroed output */
    vec![0.0; tree.output_dim]
}

/// Return node count.
pub fn ebt_node_count(tree: &EmotionBlendTree) -> usize {
    tree.nodes.len()
}

/// Enable or disable the tree.
pub fn ebt_set_enabled(tree: &mut EmotionBlendTree, enabled: bool) {
    tree.enabled = enabled;
}

/// Serialize to JSON-like string.
pub fn ebt_to_json(tree: &EmotionBlendTree) -> String {
    format!(
        r#"{{"node_count":{},"output_dim":{},"has_root":{},"enabled":{}}}"#,
        tree.nodes.len(),
        tree.output_dim,
        tree.root.is_some(),
        tree.enabled
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_output_dim() {
        let t = new_emotion_blend_tree(10);
        assert_eq!(t.output_dim, 10 /* output_dim must match */,);
    }

    #[test]
    fn test_no_nodes_initially() {
        let t = new_emotion_blend_tree(5);
        assert_eq!(ebt_node_count(&t), 0 /* no nodes initially */,);
    }

    #[test]
    fn test_add_node_returns_index() {
        let mut t = new_emotion_blend_tree(5);
        let idx = ebt_add_node(
            &mut t,
            EmotionNode {
                name: "joy".into(),
                weight: 1.0,
                children: vec![],
                blend_op: BlendOp::Add,
            },
        );
        assert_eq!(idx, 0 /* first node must have index 0 */,);
    }

    #[test]
    fn test_set_root() {
        let mut t = new_emotion_blend_tree(5);
        let idx = ebt_add_node(
            &mut t,
            EmotionNode {
                name: "root".into(),
                weight: 1.0,
                children: vec![],
                blend_op: BlendOp::Add,
            },
        );
        ebt_set_root(&mut t, idx);
        assert_eq!(t.root, Some(0) /* root must be set */,);
    }

    #[test]
    fn test_evaluate_output_length() {
        let t = new_emotion_blend_tree(8);
        let out = ebt_evaluate(&t);
        assert_eq!(out.len(), 8 /* output length must match output_dim */,);
    }

    #[test]
    fn test_evaluate_zeroed() {
        let t = new_emotion_blend_tree(3);
        let out = ebt_evaluate(&t);
        assert!(out.iter().all(|&v| v.abs() < 1e-6), /* stub must return zeros */);
    }

    #[test]
    fn test_set_enabled() {
        let mut t = new_emotion_blend_tree(3);
        ebt_set_enabled(&mut t, false);
        assert!(!t.enabled /* must be disabled */,);
    }

    #[test]
    fn test_to_json_contains_node_count() {
        let t = new_emotion_blend_tree(4);
        let j = ebt_to_json(&t);
        assert!(j.contains("\"node_count\""), /* json must contain node_count */);
    }

    #[test]
    fn test_enabled_default() {
        let t = new_emotion_blend_tree(1);
        assert!(t.enabled /* must be enabled by default */,);
    }

    #[test]
    fn test_no_root_initially() {
        let t = new_emotion_blend_tree(2);
        assert!(t.root.is_none() /* root must be None initially */,);
    }
}
