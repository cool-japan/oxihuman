#![allow(dead_code)]

/// A node in a morph blend tree.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphBlendNode {
    pub name: String,
    pub weight: f32,
    pub children: Vec<usize>,
}

/// Tree structure for hierarchical morph blending.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphBlendTree {
    nodes: Vec<MorphBlendNode>,
}

#[allow(dead_code)]
pub fn new_morph_blend_tree() -> MorphBlendTree {
    MorphBlendTree { nodes: Vec::new() }
}

#[allow(dead_code)]
pub fn add_morph_blend_node(tree: &mut MorphBlendTree, name: &str, weight: f32) -> usize {
    let idx = tree.nodes.len();
    tree.nodes.push(MorphBlendNode { name: name.to_string(), weight, children: Vec::new() });
    idx
}

#[allow(dead_code)]
pub fn evaluate_morph_blend(tree: &MorphBlendTree) -> f32 {
    if tree.nodes.is_empty() { return 0.0; }
    fn eval_node(tree: &MorphBlendTree, idx: usize) -> f32 {
        let node = &tree.nodes[idx];
        if node.children.is_empty() {
            return node.weight;
        }
        let child_sum: f32 = node.children.iter().map(|&c| eval_node(tree, c)).sum();
        node.weight * child_sum
    }
    eval_node(tree, 0)
}

#[allow(dead_code)]
pub fn blend_node_count(tree: &MorphBlendTree) -> usize {
    tree.nodes.len()
}

#[allow(dead_code)]
pub fn blend_node_weight(tree: &MorphBlendTree, idx: usize) -> f32 {
    tree.nodes.get(idx).map(|n| n.weight).unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn blend_tree_to_json_mbt(tree: &MorphBlendTree) -> String {
    let entries: Vec<String> = tree.nodes.iter().map(|n| {
        format!("{{\"name\":\"{}\",\"weight\":{:.4}}}", n.name, n.weight)
    }).collect();
    format!("{{\"nodes\":[{}]}}", entries.join(","))
}

#[allow(dead_code)]
pub fn blend_tree_reset(tree: &mut MorphBlendTree) {
    tree.nodes.clear();
}

#[allow(dead_code)]
pub fn blend_tree_depth(tree: &MorphBlendTree) -> usize {
    if tree.nodes.is_empty() { return 0; }
    fn depth(tree: &MorphBlendTree, idx: usize) -> usize {
        let node = &tree.nodes[idx];
        if node.children.is_empty() { return 1; }
        1 + node.children.iter().map(|&c| depth(tree, c)).max().unwrap_or(0)
    }
    depth(tree, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() { let t = new_morph_blend_tree(); assert_eq!(blend_node_count(&t), 0); }

    #[test]
    fn test_add_node() {
        let mut t = new_morph_blend_tree();
        let i = add_morph_blend_node(&mut t, "root", 1.0);
        assert_eq!(i, 0);
        assert_eq!(blend_node_count(&t), 1);
    }

    #[test]
    fn test_evaluate_single() {
        let mut t = new_morph_blend_tree();
        add_morph_blend_node(&mut t, "root", 0.5);
        assert!((evaluate_morph_blend(&t) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_empty() {
        let t = new_morph_blend_tree();
        assert!((evaluate_morph_blend(&t)).abs() < 1e-6);
    }

    #[test]
    fn test_node_weight() {
        let mut t = new_morph_blend_tree();
        add_morph_blend_node(&mut t, "a", 0.7);
        assert!((blend_node_weight(&t, 0) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_node_weight_missing() {
        let t = new_morph_blend_tree();
        assert!((blend_node_weight(&t, 99)).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let mut t = new_morph_blend_tree();
        add_morph_blend_node(&mut t, "root", 1.0);
        let j = blend_tree_to_json_mbt(&t);
        assert!(j.contains("root"));
    }

    #[test]
    fn test_reset() {
        let mut t = new_morph_blend_tree();
        add_morph_blend_node(&mut t, "a", 1.0);
        blend_tree_reset(&mut t);
        assert_eq!(blend_node_count(&t), 0);
    }

    #[test]
    fn test_depth_empty() {
        let t = new_morph_blend_tree();
        assert_eq!(blend_tree_depth(&t), 0);
    }

    #[test]
    fn test_depth_single() {
        let mut t = new_morph_blend_tree();
        add_morph_blend_node(&mut t, "root", 1.0);
        assert_eq!(blend_tree_depth(&t), 1);
    }
}
