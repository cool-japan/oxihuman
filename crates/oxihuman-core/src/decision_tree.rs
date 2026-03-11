// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple boolean decision tree with leaf actions.

#![allow(dead_code)]

use std::collections::HashMap;

/// A condition that evaluates to bool given a context.
#[allow(dead_code)]
pub type ConditionFn = fn(&HashMap<String, bool>) -> bool;

/// A node in the decision tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DecisionNode {
    /// Branch: condition key, true-branch index, false-branch index.
    Branch {
        condition_key: String,
        true_branch: usize,
        false_branch: usize,
    },
    /// Leaf: action label.
    Leaf(String),
}

/// A binary decision tree backed by a flat Vec of nodes.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct DecisionTree {
    nodes: Vec<DecisionNode>,
}

/// Create a new empty decision tree.
#[allow(dead_code)]
pub fn new_decision_tree() -> DecisionTree {
    DecisionTree::default()
}

/// Add a leaf node and return its index.
#[allow(dead_code)]
pub fn dt_add_leaf(tree: &mut DecisionTree, action: &str) -> usize {
    let idx = tree.nodes.len();
    tree.nodes.push(DecisionNode::Leaf(action.to_string()));
    idx
}

/// Add a branch node and return its index.
#[allow(dead_code)]
pub fn dt_add_branch(
    tree: &mut DecisionTree,
    condition_key: &str,
    true_branch: usize,
    false_branch: usize,
) -> usize {
    let idx = tree.nodes.len();
    tree.nodes.push(DecisionNode::Branch {
        condition_key: condition_key.to_string(),
        true_branch,
        false_branch,
    });
    idx
}

/// Evaluate the tree starting at `root` using the given boolean context.
/// Returns the leaf action label, or `None` if the tree is empty or invalid.
#[allow(dead_code)]
pub fn dt_evaluate(
    tree: &DecisionTree,
    root: usize,
    context: &HashMap<String, bool>,
) -> Option<String> {
    if tree.nodes.is_empty() {
        return None;
    }
    let mut current = root;
    loop {
        match tree.nodes.get(current)? {
            DecisionNode::Leaf(action) => return Some(action.clone()),
            DecisionNode::Branch {
                condition_key,
                true_branch,
                false_branch,
            } => {
                let cond = context.get(condition_key).copied().unwrap_or(false);
                current = if cond { *true_branch } else { *false_branch };
            }
        }
    }
}

/// Return the number of nodes in the tree.
#[allow(dead_code)]
pub fn dt_node_count(tree: &DecisionTree) -> usize {
    tree.nodes.len()
}

/// Count only leaf nodes.
#[allow(dead_code)]
pub fn dt_leaf_count(tree: &DecisionTree) -> usize {
    tree.nodes
        .iter()
        .filter(|n| matches!(n, DecisionNode::Leaf(_)))
        .count()
}

/// Count only branch nodes.
#[allow(dead_code)]
pub fn dt_branch_count(tree: &DecisionTree) -> usize {
    tree.nodes
        .iter()
        .filter(|n| matches!(n, DecisionNode::Branch { .. }))
        .count()
}

/// Clear all nodes from the tree.
#[allow(dead_code)]
pub fn dt_clear(tree: &mut DecisionTree) {
    tree.nodes.clear();
}

/// Get the action label if a node is a leaf.
#[allow(dead_code)]
pub fn dt_leaf_action(tree: &DecisionTree, idx: usize) -> Option<&str> {
    match tree.nodes.get(idx)? {
        DecisionNode::Leaf(a) => Some(a.as_str()),
        _ => None,
    }
}

/// Collect all leaf actions in tree order.
#[allow(dead_code)]
pub fn dt_all_actions(tree: &DecisionTree) -> Vec<String> {
    tree.nodes
        .iter()
        .filter_map(|n| {
            if let DecisionNode::Leaf(a) = n {
                Some(a.clone())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_tree() -> (DecisionTree, usize) {
        let mut tree = new_decision_tree();
        let leaf_a = dt_add_leaf(&mut tree, "action_a");
        let leaf_b = dt_add_leaf(&mut tree, "action_b");
        let root = dt_add_branch(&mut tree, "is_hot", leaf_a, leaf_b);
        (tree, root)
    }

    #[test]
    fn test_empty_tree() {
        let tree = new_decision_tree();
        assert_eq!(dt_node_count(&tree), 0);
        assert_eq!(dt_leaf_count(&tree), 0);
    }

    #[test]
    fn test_add_leaf() {
        let mut tree = new_decision_tree();
        let idx = dt_add_leaf(&mut tree, "fire");
        assert_eq!(idx, 0);
        assert_eq!(dt_node_count(&tree), 1);
        assert_eq!(dt_leaf_count(&tree), 1);
    }

    #[test]
    fn test_add_branch() {
        let mut tree = new_decision_tree();
        let a = dt_add_leaf(&mut tree, "a");
        let b = dt_add_leaf(&mut tree, "b");
        let root = dt_add_branch(&mut tree, "cond", a, b);
        assert_eq!(dt_branch_count(&tree), 1);
        assert_eq!(root, 2);
    }

    #[test]
    fn test_evaluate_true_branch() {
        let (tree, root) = make_simple_tree();
        let mut ctx = HashMap::new();
        ctx.insert("is_hot".to_string(), true);
        let result = dt_evaluate(&tree, root, &ctx);
        assert_eq!(result, Some("action_a".to_string()));
    }

    #[test]
    fn test_evaluate_false_branch() {
        let (tree, root) = make_simple_tree();
        let mut ctx = HashMap::new();
        ctx.insert("is_hot".to_string(), false);
        let result = dt_evaluate(&tree, root, &ctx);
        assert_eq!(result, Some("action_b".to_string()));
    }

    #[test]
    fn test_evaluate_missing_key_defaults_false() {
        let (tree, root) = make_simple_tree();
        let ctx = HashMap::new();
        let result = dt_evaluate(&tree, root, &ctx);
        assert_eq!(result, Some("action_b".to_string()));
    }

    #[test]
    fn test_leaf_action() {
        let mut tree = new_decision_tree();
        let idx = dt_add_leaf(&mut tree, "run");
        assert_eq!(dt_leaf_action(&tree, idx), Some("run"));
    }

    #[test]
    fn test_all_actions() {
        let (tree, _) = make_simple_tree();
        let actions = dt_all_actions(&tree);
        assert_eq!(actions.len(), 2);
        assert!(actions.contains(&"action_a".to_string()));
        assert!(actions.contains(&"action_b".to_string()));
    }

    #[test]
    fn test_clear() {
        let (mut tree, _) = make_simple_tree();
        dt_clear(&mut tree);
        assert_eq!(dt_node_count(&tree), 0);
    }

    #[test]
    fn test_evaluate_empty_returns_none() {
        let tree = new_decision_tree();
        let ctx = HashMap::new();
        assert!(dt_evaluate(&tree, 0, &ctx).is_none());
    }
}
