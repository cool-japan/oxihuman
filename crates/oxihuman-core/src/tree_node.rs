#![allow(dead_code)]

/// A tree node holding a string label and children.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<TreeNode>,
}

/// A tree is just a collection of root nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Tree {
    pub roots: Vec<TreeNode>,
}

/// Creates a new empty tree.
#[allow(dead_code)]
pub fn new_tree() -> Tree {
    Tree { roots: Vec::new() }
}

/// Adds a child with the given label to a node, returns mutable ref to the child.
#[allow(dead_code)]
pub fn add_child<'a>(node: &'a mut TreeNode, label: &str) -> &'a mut TreeNode {
    node.children.push(TreeNode {
        label: label.to_string(),
        children: Vec::new(),
    });
    let len = node.children.len();
    &mut node.children[len - 1]
}

/// Returns the number of direct children of a node.
#[allow(dead_code)]
pub fn child_count(node: &TreeNode) -> usize {
    node.children.len()
}

/// Returns the depth of the tree rooted at this node.
#[allow(dead_code)]
pub fn tree_depth(node: &TreeNode) -> usize {
    if node.children.is_empty() {
        return 1;
    }
    1 + node
        .children
        .iter()
        .map(tree_depth)
        .max()
        .unwrap_or(0)
}

/// Finds a node by label (DFS), returns a reference.
#[allow(dead_code)]
pub fn find_node<'a>(node: &'a TreeNode, label: &str) -> Option<&'a TreeNode> {
    if node.label == label {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_node(child, label) {
            return Some(found);
        }
    }
    None
}

/// Removes the first child with the given label, returns true if removed.
#[allow(dead_code)]
pub fn remove_node(node: &mut TreeNode, label: &str) -> bool {
    if let Some(pos) = node.children.iter().position(|c| c.label == label) {
        node.children.remove(pos);
        return true;
    }
    for child in &mut node.children {
        if remove_node(child, label) {
            return true;
        }
    }
    false
}

/// Serializes the tree node to a simple JSON-like string.
#[allow(dead_code)]
pub fn tree_to_json(node: &TreeNode) -> String {
    if node.children.is_empty() {
        return format!("{{\"label\":\"{}\"}}", node.label);
    }
    let children_json: Vec<String> = node.children.iter().map(tree_to_json).collect();
    format!(
        "{{\"label\":\"{}\",\"children\":[{}]}}",
        node.label,
        children_json.join(",")
    )
}

/// Returns the total number of nodes in the subtree.
#[allow(dead_code)]
pub fn tree_size(node: &TreeNode) -> usize {
    1 + node.children.iter().map(tree_size).sum::<usize>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_root(label: &str) -> TreeNode {
        TreeNode {
            label: label.to_string(),
            children: Vec::new(),
        }
    }

    #[test]
    fn test_new_tree() {
        let t = new_tree();
        assert!(t.roots.is_empty());
    }

    #[test]
    fn test_add_child() {
        let mut root = make_root("root");
        add_child(&mut root, "child1");
        assert_eq!(child_count(&root), 1);
    }

    #[test]
    fn test_tree_depth() {
        let mut root = make_root("root");
        let c1 = add_child(&mut root, "c1");
        add_child(c1, "c1_1");
        assert_eq!(tree_depth(&root), 3);
    }

    #[test]
    fn test_find_node() {
        let mut root = make_root("root");
        let c = add_child(&mut root, "target");
        add_child(c, "deep");
        assert!(find_node(&root, "target").is_some());
        assert!(find_node(&root, "deep").is_some());
        assert!(find_node(&root, "missing").is_none());
    }

    #[test]
    fn test_remove_node() {
        let mut root = make_root("root");
        add_child(&mut root, "a");
        add_child(&mut root, "b");
        assert!(remove_node(&mut root, "a"));
        assert_eq!(child_count(&root), 1);
    }

    #[test]
    fn test_tree_to_json() {
        let root = make_root("leaf");
        let json = tree_to_json(&root);
        assert!(json.contains("leaf"));
    }

    #[test]
    fn test_tree_size() {
        let mut root = make_root("root");
        add_child(&mut root, "a");
        add_child(&mut root, "b");
        assert_eq!(tree_size(&root), 3);
    }

    #[test]
    fn test_depth_leaf() {
        let leaf = make_root("leaf");
        assert_eq!(tree_depth(&leaf), 1);
    }

    #[test]
    fn test_remove_deep() {
        let mut root = make_root("root");
        let c = add_child(&mut root, "c");
        add_child(c, "deep");
        assert!(remove_node(&mut root, "deep"));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut root = make_root("root");
        assert!(!remove_node(&mut root, "nope"));
    }
}
