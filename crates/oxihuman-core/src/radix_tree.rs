// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

#[allow(dead_code)]
pub struct RadixNode {
    pub prefix: String,
    pub value: Option<String>,
    pub children: Vec<usize>,
}

#[allow(dead_code)]
pub struct RadixTree {
    pub nodes: Vec<RadixNode>,
}

#[allow(dead_code)]
pub fn new_radix_tree() -> RadixTree {
    let root = RadixNode { prefix: String::new(), value: None, children: Vec::new() };
    RadixTree { nodes: vec![root] }
}

#[allow(dead_code)]
pub fn rt_insert(tree: &mut RadixTree, key: &str, value: &str) {
    let mut node_idx = 0;
    let mut remaining = key;
    loop {
        let prefix = tree.nodes[node_idx].prefix.clone();
        let common = common_prefix_len(&prefix, remaining);
        if common == prefix.len() && common == remaining.len() {
            tree.nodes[node_idx].value = Some(value.to_string());
            return;
        } else if common == prefix.len() {
            remaining = &remaining[common..];
            let children = tree.nodes[node_idx].children.clone();
            let mut found = false;
            for &child in &children {
                let child_prefix = tree.nodes[child].prefix.clone();
                if child_prefix.starts_with(&remaining[..1]) {
                    node_idx = child;
                    found = true;
                    break;
                }
            }
            if !found {
                let new_node = RadixNode {
                    prefix: remaining.to_string(),
                    value: Some(value.to_string()),
                    children: Vec::new(),
                };
                let new_idx = tree.nodes.len();
                tree.nodes.push(new_node);
                tree.nodes[node_idx].children.push(new_idx);
                return;
            }
        } else if common == 0 {
            let new_node = RadixNode {
                prefix: remaining.to_string(),
                value: Some(value.to_string()),
                children: Vec::new(),
            };
            let new_idx = tree.nodes.len();
            tree.nodes.push(new_node);
            tree.nodes[node_idx].children.push(new_idx);
            return;
        } else {
            let split_prefix = prefix[..common].to_string();
            let old_suffix = prefix[common..].to_string();
            let old_value = tree.nodes[node_idx].value.take();
            let old_children = std::mem::take(&mut tree.nodes[node_idx].children);
            let split_child = RadixNode {
                prefix: old_suffix,
                value: old_value,
                children: old_children,
            };
            let split_idx = tree.nodes.len();
            tree.nodes.push(split_child);
            tree.nodes[node_idx].prefix = split_prefix;
            tree.nodes[node_idx].children = vec![split_idx];
            remaining = &remaining[common..];
            if remaining.is_empty() {
                tree.nodes[node_idx].value = Some(value.to_string());
                return;
            }
            let new_node = RadixNode {
                prefix: remaining.to_string(),
                value: Some(value.to_string()),
                children: Vec::new(),
            };
            let new_idx = tree.nodes.len();
            tree.nodes.push(new_node);
            tree.nodes[node_idx].children.push(new_idx);
            return;
        }
    }
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

#[allow(dead_code)]
pub fn rt_get<'a>(tree: &'a RadixTree, key: &str) -> Option<&'a str> {
    let mut node_idx = 0;
    let mut remaining = key;
    loop {
        let prefix = &tree.nodes[node_idx].prefix;
        let common = common_prefix_len(prefix, remaining);
        if common == prefix.len() && common == remaining.len() {
            return tree.nodes[node_idx].value.as_deref();
        } else if common == prefix.len() {
            remaining = &remaining[common..];
            let children = tree.nodes[node_idx].children.clone();
            let mut found = false;
            for &child in &children {
                let cp = common_prefix_len(&tree.nodes[child].prefix, remaining);
                if cp > 0 {
                    node_idx = child;
                    found = true;
                    break;
                }
            }
            if !found {
                return None;
            }
        } else {
            return None;
        }
    }
}

#[allow(dead_code)]
pub fn rt_contains(tree: &RadixTree, key: &str) -> bool {
    rt_get(tree, key).is_some()
}

#[allow(dead_code)]
pub fn rt_node_count(tree: &RadixTree) -> usize {
    tree.nodes.len()
}

#[allow(dead_code)]
pub fn rt_starts_with(tree: &RadixTree, prefix: &str) -> Vec<String> {
    let mut results = Vec::new();
    collect_with_prefix(tree, 0, "", prefix, &mut results);
    results
}

fn collect_with_prefix(
    tree: &RadixTree,
    node_idx: usize,
    current_path: &str,
    prefix: &str,
    results: &mut Vec<String>,
) {
    let node_prefix = &tree.nodes[node_idx].prefix;
    let full_path = format!("{}{}", current_path, node_prefix);
    if full_path.starts_with(prefix) || prefix.starts_with(&full_path as &str) {
        if full_path.starts_with(prefix) && tree.nodes[node_idx].value.is_some() {
            results.push(full_path.clone());
        }
        let children = tree.nodes[node_idx].children.clone();
        for child in children {
            collect_with_prefix(tree, child, &full_path, prefix, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut tree = new_radix_tree();
        rt_insert(&mut tree, "hello", "world");
        assert_eq!(rt_get(&tree, "hello"), Some("world"));
    }

    #[test]
    fn test_get_missing() {
        let tree = new_radix_tree();
        assert_eq!(rt_get(&tree, "missing"), None);
    }

    #[test]
    fn test_contains_true() {
        let mut tree = new_radix_tree();
        rt_insert(&mut tree, "rust", "lang");
        assert!(rt_contains(&tree, "rust"));
    }

    #[test]
    fn test_contains_false() {
        let mut tree = new_radix_tree();
        rt_insert(&mut tree, "rust", "lang");
        assert!(!rt_contains(&tree, "ruby"));
    }

    #[test]
    fn test_node_count_grows() {
        let mut tree = new_radix_tree();
        let initial = rt_node_count(&tree);
        rt_insert(&mut tree, "abc", "1");
        rt_insert(&mut tree, "def", "2");
        assert!(rt_node_count(&tree) > initial);
    }

    #[test]
    fn test_multiple_inserts() {
        let mut tree = new_radix_tree();
        rt_insert(&mut tree, "foo", "1");
        rt_insert(&mut tree, "bar", "2");
        assert_eq!(rt_get(&tree, "foo"), Some("1"));
        assert_eq!(rt_get(&tree, "bar"), Some("2"));
    }

    #[test]
    fn test_overwrite_value() {
        let mut tree = new_radix_tree();
        rt_insert(&mut tree, "key", "old");
        rt_insert(&mut tree, "key", "new");
        assert_eq!(rt_get(&tree, "key"), Some("new"));
    }

    #[test]
    fn test_starts_with() {
        let mut tree = new_radix_tree();
        rt_insert(&mut tree, "hello", "1");
        rt_insert(&mut tree, "world", "2");
        let results = rt_starts_with(&tree, "hello");
        assert!(results.contains(&"hello".to_string()));
    }

    #[test]
    fn test_starts_with_empty_prefix() {
        let mut tree = new_radix_tree();
        rt_insert(&mut tree, "alpha", "a");
        rt_insert(&mut tree, "beta", "b");
        let results = rt_starts_with(&tree, "");
        assert!(results.len() >= 2);
    }
}
