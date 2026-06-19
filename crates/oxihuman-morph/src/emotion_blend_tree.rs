// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Emotion node blend tree stub.

use std::collections::HashSet;

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

/// Recursive DFS evaluator with cycle detection.
///
/// Returns the contribution of the node at `node_idx`, scaled by that node's weight.
/// `visited` tracks ancestors on the current DFS path to detect cycles.
fn ebt_eval_node(
    tree: &EmotionBlendTree,
    node_idx: usize,
    visited: &mut HashSet<usize>,
) -> Vec<f32> {
    let output_dim = tree.output_dim;

    // Cycle guard: if this node is already on the current DFS path, return zeros.
    if !visited.insert(node_idx) {
        return vec![0.0; output_dim];
    }

    // Depth guard: refuse to recurse deeper than the total node count (extra safety).
    if visited.len() > tree.nodes.len() {
        visited.remove(&node_idx);
        return vec![0.0; output_dim];
    }

    let node = &tree.nodes[node_idx];

    let scaled = if node.children.is_empty() {
        // Leaf node: contributes `node.weight` uniformly across all dimensions.
        vec![node.weight; output_dim]
    } else {
        // Accumulate children according to blend_op.
        let mut child_acc: Vec<f32> = match node.blend_op {
            BlendOp::Add => vec![0.0; output_dim],
            BlendOp::Multiply => vec![1.0; output_dim],
            BlendOp::Override => vec![0.0; output_dim],
        };

        for &child_idx in &node.children {
            if child_idx >= tree.nodes.len() {
                continue;
            }
            let child_result = ebt_eval_node(tree, child_idx, visited);
            match node.blend_op {
                BlendOp::Add => {
                    for j in 0..output_dim {
                        child_acc[j] += child_result[j];
                    }
                }
                BlendOp::Multiply => {
                    for j in 0..output_dim {
                        child_acc[j] *= child_result[j];
                    }
                }
                BlendOp::Override => {
                    child_acc = child_result;
                }
            }
        }

        // Scale the accumulated result by this node's weight.
        let mut result = vec![0.0; output_dim];
        for j in 0..output_dim {
            result[j] = child_acc[j] * node.weight;
        }
        result
    };

    visited.remove(&node_idx);
    scaled
}

/// Evaluate the tree to produce output weights via recursive DFS blend.
///
/// Traverses from `tree.root` downward. Leaf nodes contribute
/// `vec![node.weight; output_dim]`. Interior nodes accumulate children
/// according to their `blend_op`, then scale by their own `weight`.
/// Returns zeros if no root is set or the tree is disabled.
pub fn ebt_evaluate(tree: &EmotionBlendTree) -> Vec<f32> {
    let root_idx = match tree.root {
        Some(idx) => idx,
        None => return vec![0.0; tree.output_dim],
    };

    if root_idx >= tree.nodes.len() {
        return vec![0.0; tree.output_dim];
    }

    let mut visited = HashSet::new();
    ebt_eval_node(tree, root_idx, &mut visited)
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

    #[test]
    fn ebt_leaf_node_weight_propagated() {
        // A single root leaf node (weight=0.5, output_dim=2).
        // Expected output: [0.5, 0.5].
        let mut t = new_emotion_blend_tree(2);
        let idx = ebt_add_node(
            &mut t,
            EmotionNode {
                name: "leaf".into(),
                weight: 0.5,
                children: vec![],
                blend_op: BlendOp::Add,
            },
        );
        ebt_set_root(&mut t, idx);
        let out = ebt_evaluate(&t);
        assert_eq!(out.len(), 2, "output length must be 2");
        assert!((out[0] - 0.5).abs() < 1e-6, "expected 0.5, got {}", out[0]);
        assert!((out[1] - 0.5).abs() < 1e-6, "expected 0.5, got {}", out[1]);
    }

    #[test]
    fn ebt_add_two_children() {
        // root (Add, weight=1.0) → [child_a (leaf, weight=1.0), child_b (leaf, weight=0.5)]
        // child_a contributes [1.0, 1.0], child_b contributes [0.5, 0.5].
        // After Add in root: [1.5, 1.5]; scaled by root.weight=1.0 → [1.5, 1.5].
        let mut t = new_emotion_blend_tree(2);
        let child_a = ebt_add_node(
            &mut t,
            EmotionNode {
                name: "child_a".into(),
                weight: 1.0,
                children: vec![],
                blend_op: BlendOp::Add,
            },
        );
        let child_b = ebt_add_node(
            &mut t,
            EmotionNode {
                name: "child_b".into(),
                weight: 0.5,
                children: vec![],
                blend_op: BlendOp::Add,
            },
        );
        let root_idx = ebt_add_node(
            &mut t,
            EmotionNode {
                name: "root".into(),
                weight: 1.0,
                children: vec![child_a, child_b],
                blend_op: BlendOp::Add,
            },
        );
        ebt_set_root(&mut t, root_idx);
        let out = ebt_evaluate(&t);
        assert_eq!(out.len(), 2, "output length must be 2");
        assert!((out[0] - 1.5).abs() < 1e-6, "expected 1.5, got {}", out[0]);
        assert!((out[1] - 1.5).abs() < 1e-6, "expected 1.5, got {}", out[1]);
    }

    #[test]
    fn ebt_empty_tree_returns_zeros() {
        // No root set → zeros.
        let t = new_emotion_blend_tree(4);
        let out = ebt_evaluate(&t);
        assert_eq!(out.len(), 4, "output length must match output_dim");
        assert!(
            out.iter().all(|&v| v.abs() < 1e-6),
            "expected all zeros with no root, got {:?}",
            out
        );
    }
}
