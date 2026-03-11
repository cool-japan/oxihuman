//! Blend tree for expression composition — layers expressions with weights in a tree structure.

#[allow(dead_code)]
#[derive(Clone)]
pub enum BlendTreeNode {
    Leaf { name: String, weight: f32 },
    Blend { children: Vec<usize>, weights: Vec<f32> },
    Override { base: usize, override_: usize, alpha: f32 },
}

#[allow(dead_code)]
pub struct BlendTreeConfig {
    pub max_nodes: usize,
    pub normalize_on_evaluate: bool,
}

#[allow(dead_code)]
pub struct BlendTree {
    pub nodes: Vec<BlendTreeNode>,
    pub config: BlendTreeConfig,
}

#[allow(dead_code)]
pub fn default_blend_tree_config() -> BlendTreeConfig {
    BlendTreeConfig {
        max_nodes: 256,
        normalize_on_evaluate: true,
    }
}

#[allow(dead_code)]
pub fn new_blend_tree(cfg: &BlendTreeConfig) -> BlendTree {
    BlendTree {
        nodes: Vec::new(),
        config: BlendTreeConfig {
            max_nodes: cfg.max_nodes,
            normalize_on_evaluate: cfg.normalize_on_evaluate,
        },
    }
}

#[allow(dead_code)]
pub fn blend_tree_add_leaf(tree: &mut BlendTree, name: &str, weight: f32) -> usize {
    let idx = tree.nodes.len();
    tree.nodes.push(BlendTreeNode::Leaf {
        name: name.to_string(),
        weight: weight.clamp(0.0, 1.0),
    });
    idx
}

#[allow(dead_code)]
pub fn blend_tree_set_weight(tree: &mut BlendTree, node_idx: usize, weight: f32) {
    if let Some(node) = tree.nodes.get_mut(node_idx) {
        match node {
            BlendTreeNode::Leaf { weight: w, .. } => {
                *w = weight.clamp(0.0, 1.0);
            }
            BlendTreeNode::Override { alpha, .. } => {
                *alpha = weight.clamp(0.0, 1.0);
            }
            BlendTreeNode::Blend { weights, .. } => {
                if let Some(first) = weights.first_mut() {
                    *first = weight.clamp(0.0, 1.0);
                }
            }
        }
    }
}

/// Collect (name, weight) pairs from all leaf nodes.
#[allow(dead_code)]
pub fn blend_tree_evaluate(tree: &BlendTree) -> Vec<(String, f32)> {
    let mut result: Vec<(String, f32)> = Vec::new();
    for node in &tree.nodes {
        if let BlendTreeNode::Leaf { name, weight } = node {
            // Merge duplicate names by summing
            if let Some(entry) = result.iter_mut().find(|(n, _)| n == name) {
                entry.1 += weight;
            } else {
                result.push((name.clone(), *weight));
            }
        }
    }
    result
}

#[allow(dead_code)]
pub fn blend_tree_node_count(tree: &BlendTree) -> usize {
    tree.nodes.len()
}

#[allow(dead_code)]
pub fn blend_tree_normalize(tree: &mut BlendTree) {
    let total: f32 = tree.nodes.iter().map(|n| match n {
        BlendTreeNode::Leaf { weight, .. } => *weight,
        _ => 0.0,
    }).sum();
    if total <= 0.0 {
        return;
    }
    for node in &mut tree.nodes {
        if let BlendTreeNode::Leaf { weight, .. } = node {
            *weight /= total;
        }
    }
}

#[allow(dead_code)]
pub fn blend_tree_clear(tree: &mut BlendTree) {
    tree.nodes.clear();
}

#[allow(dead_code)]
pub fn blend_tree_total_weight(tree: &BlendTree) -> f32 {
    tree.nodes.iter().map(|n| match n {
        BlendTreeNode::Leaf { weight, .. } => *weight,
        _ => 0.0,
    }).sum()
}

#[allow(dead_code)]
pub fn blend_tree_find_leaf(tree: &BlendTree, name: &str) -> Option<usize> {
    tree.nodes.iter().enumerate().find_map(|(i, n)| {
        if let BlendTreeNode::Leaf { name: n_name, .. } = n {
            if n_name == name { Some(i) } else { None }
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_blend_tree_config();
        assert_eq!(cfg.max_nodes, 256);
        assert!(cfg.normalize_on_evaluate);
    }

    #[test]
    fn test_new_blend_tree_empty() {
        let cfg = default_blend_tree_config();
        let tree = new_blend_tree(&cfg);
        assert_eq!(blend_tree_node_count(&tree), 0);
    }

    #[test]
    fn test_add_leaf_and_count() {
        let cfg = default_blend_tree_config();
        let mut tree = new_blend_tree(&cfg);
        let idx = blend_tree_add_leaf(&mut tree, "happy", 0.8);
        assert_eq!(idx, 0);
        assert_eq!(blend_tree_node_count(&tree), 1);
    }

    #[test]
    fn test_evaluate_single_leaf() {
        let cfg = default_blend_tree_config();
        let mut tree = new_blend_tree(&cfg);
        blend_tree_add_leaf(&mut tree, "sad", 0.5);
        let result = blend_tree_evaluate(&tree);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "sad");
        assert!((result[0].1 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight() {
        let cfg = default_blend_tree_config();
        let mut tree = new_blend_tree(&cfg);
        let idx = blend_tree_add_leaf(&mut tree, "angry", 0.2);
        blend_tree_set_weight(&mut tree, idx, 0.9);
        let result = blend_tree_evaluate(&tree);
        assert!((result[0].1 - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_total_weight() {
        let cfg = default_blend_tree_config();
        let mut tree = new_blend_tree(&cfg);
        blend_tree_add_leaf(&mut tree, "a", 0.3);
        blend_tree_add_leaf(&mut tree, "b", 0.4);
        let total = blend_tree_total_weight(&tree);
        assert!((total - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_normalize() {
        let cfg = default_blend_tree_config();
        let mut tree = new_blend_tree(&cfg);
        blend_tree_add_leaf(&mut tree, "x", 0.4);
        blend_tree_add_leaf(&mut tree, "y", 0.6);
        blend_tree_normalize(&mut tree);
        let total = blend_tree_total_weight(&tree);
        assert!((total - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_find_leaf() {
        let cfg = default_blend_tree_config();
        let mut tree = new_blend_tree(&cfg);
        blend_tree_add_leaf(&mut tree, "joy", 0.5);
        blend_tree_add_leaf(&mut tree, "fear", 0.3);
        let idx = blend_tree_find_leaf(&tree, "fear");
        assert_eq!(idx, Some(1));
        assert_eq!(blend_tree_find_leaf(&tree, "missing"), None);
    }

    #[test]
    fn test_clear() {
        let cfg = default_blend_tree_config();
        let mut tree = new_blend_tree(&cfg);
        blend_tree_add_leaf(&mut tree, "a", 1.0);
        blend_tree_add_leaf(&mut tree, "b", 1.0);
        blend_tree_clear(&mut tree);
        assert_eq!(blend_tree_node_count(&tree), 0);
    }

    #[test]
    fn test_weight_clamped_to_one() {
        let cfg = default_blend_tree_config();
        let mut tree = new_blend_tree(&cfg);
        let idx = blend_tree_add_leaf(&mut tree, "big", 2.5);
        // clamped to 1.0 on add
        let result = blend_tree_evaluate(&tree);
        assert!((result[0].1 - 1.0).abs() < 1e-6);
        // clamped to 1.0 on set
        blend_tree_set_weight(&mut tree, idx, 5.0);
        let result2 = blend_tree_evaluate(&tree);
        assert!((result2[0].1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_merges_duplicates() {
        let cfg = default_blend_tree_config();
        let mut tree = new_blend_tree(&cfg);
        blend_tree_add_leaf(&mut tree, "joy", 0.3);
        blend_tree_add_leaf(&mut tree, "joy", 0.4);
        let result = blend_tree_evaluate(&tree);
        assert_eq!(result.len(), 1);
        assert!((result[0].1 - 0.7).abs() < 1e-6);
    }
}
