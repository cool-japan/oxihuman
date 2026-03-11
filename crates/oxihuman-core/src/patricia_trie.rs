#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple radix/patricia-like trie for string lookup.

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TrieNode {
    pub prefix: String,
    pub children: Vec<TrieNode>,
    pub value: Option<String>,
}

#[allow(dead_code)]
pub fn new_trie_node() -> TrieNode {
    TrieNode {
        prefix: String::new(),
        children: Vec::new(),
        value: None,
    }
}

#[allow(dead_code)]
pub fn trie_insert(root: &mut TrieNode, key: &str, val: &str) {
    insert_node(root, key, val);
}

fn insert_node(node: &mut TrieNode, remaining: &str, val: &str) {
    // Check if any child's prefix matches
    for i in 0..node.children.len() {
        let common_len = {
            let cp = node.children[i].prefix.chars()
                .zip(remaining.chars())
                .take_while(|(a, b)| a == b)
                .count();
            // convert char count to byte count
            node.children[i].prefix.char_indices().nth(cp).map(|(b, _)| b).unwrap_or(node.children[i].prefix.len())
        };
        if common_len == 0 {
            continue;
        }
        let child_prefix_len = node.children[i].prefix.len();
        if common_len == child_prefix_len {
            // Recurse into child
            let rest = remaining[common_len..].to_string();
            if rest.is_empty() {
                node.children[i].value = Some(val.to_string());
            } else {
                insert_node(&mut node.children[i], &rest, val);
            }
            return;
        } else {
            // Split child at common_len
            let old_suffix = node.children[i].prefix[common_len..].to_string();
            let common_str = node.children[i].prefix[..common_len].to_string();
            let new_suffix = remaining[common_len..].to_string();

            let old_child = TrieNode {
                prefix: old_suffix,
                children: std::mem::take(&mut node.children[i].children),
                value: node.children[i].value.take(),
            };
            node.children[i].prefix = common_str;
            node.children[i].children = vec![old_child];
            if new_suffix.is_empty() {
                node.children[i].value = Some(val.to_string());
            } else {
                let new_leaf = TrieNode {
                    prefix: new_suffix,
                    children: Vec::new(),
                    value: Some(val.to_string()),
                };
                node.children[i].children.push(new_leaf);
            }
            return;
        }
    }
    // No matching child: add new leaf
    node.children.push(TrieNode {
        prefix: remaining.to_string(),
        children: Vec::new(),
        value: Some(val.to_string()),
    });
}

fn common_prefix<'a>(a: &'a str, b: &str) -> &'a str {
    let len = a
        .chars()
        .zip(b.chars())
        .take_while(|(ca, cb)| ca == cb)
        .count();
    let byte_len = a
        .char_indices()
        .nth(len)
        .map(|(i, _)| i)
        .unwrap_or(a.len());
    &a[..byte_len]
}

#[allow(dead_code)]
pub fn trie_find<'a>(root: &'a TrieNode, key: &str) -> Option<&'a str> {
    find_node(root, key)
}

fn find_node<'a>(node: &'a TrieNode, remaining: &str) -> Option<&'a str> {
    for child in &node.children {
        if remaining.starts_with(&child.prefix as &str) {
            let rest = &remaining[child.prefix.len()..];
            if rest.is_empty() {
                return child.value.as_deref();
            }
            return find_node(child, rest);
        }
    }
    None
}

#[allow(dead_code)]
pub fn trie_starts_with(root: &TrieNode, prefix: &str) -> Vec<String> {
    let mut results = Vec::new();
    collect_with_prefix(root, prefix, "", &mut results);
    results
}

fn collect_with_prefix(
    node: &TrieNode,
    remaining_prefix: &str,
    current_path: &str,
    results: &mut Vec<String>,
) {
    for child in &node.children {
        let child_path = format!("{}{}", current_path, child.prefix);
        if remaining_prefix.is_empty() {
            // Collect all descendants
            if child.value.is_some() {
                results.push(child_path.clone());
            }
            collect_with_prefix(child, "", &child_path, results);
        } else if remaining_prefix.starts_with(&child.prefix as &str) {
            let rest = &remaining_prefix[child.prefix.len()..];
            collect_with_prefix(child, rest, &child_path, results);
        } else if child.prefix.starts_with(remaining_prefix) {
            // prefix is exhausted within this node
            if child.value.is_some() {
                results.push(child_path.clone());
            }
            collect_with_prefix(child, "", &child_path, results);
        }
    }
}

#[allow(dead_code)]
pub fn trie_remove(root: &mut TrieNode, key: &str) -> bool {
    remove_node(root, key)
}

fn remove_node(node: &mut TrieNode, remaining: &str) -> bool {
    for i in 0..node.children.len() {
        if remaining.starts_with(&node.children[i].prefix as &str) {
            let rest = remaining[node.children[i].prefix.len()..].to_string();
            if rest.is_empty() {
                if node.children[i].value.is_some() {
                    node.children[i].value = None;
                    if node.children[i].children.is_empty() {
                        node.children.remove(i);
                    }
                    return true;
                }
                return false;
            }
            return remove_node(&mut node.children[i], &rest);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_find() {
        let mut root = new_trie_node();
        trie_insert(&mut root, "hello", "world");
        assert_eq!(trie_find(&root, "hello"), Some("world"));
    }

    #[test]
    fn find_missing() {
        let root = new_trie_node();
        assert!(trie_find(&root, "foo").is_none());
    }

    #[test]
    fn multiple_keys() {
        let mut root = new_trie_node();
        trie_insert(&mut root, "cat", "feline");
        trie_insert(&mut root, "car", "vehicle");
        assert_eq!(trie_find(&root, "cat"), Some("feline"));
        assert_eq!(trie_find(&root, "car"), Some("vehicle"));
    }

    #[test]
    fn update_value() {
        let mut root = new_trie_node();
        trie_insert(&mut root, "key", "old");
        trie_insert(&mut root, "key", "new");
        assert_eq!(trie_find(&root, "key"), Some("new"));
    }

    #[test]
    fn remove_existing() {
        let mut root = new_trie_node();
        trie_insert(&mut root, "remove_me", "val");
        assert!(trie_remove(&mut root, "remove_me"));
        assert!(trie_find(&root, "remove_me").is_none());
    }

    #[test]
    fn remove_missing_returns_false() {
        let mut root = new_trie_node();
        assert!(!trie_remove(&mut root, "ghost"));
    }

    #[test]
    fn starts_with_basic() {
        let mut root = new_trie_node();
        trie_insert(&mut root, "apple", "1");
        trie_insert(&mut root, "application", "2");
        trie_insert(&mut root, "banana", "3");
        let results = trie_starts_with(&root, "app");
        assert!(results.contains(&"apple".to_string()) || results.contains(&"application".to_string()));
    }

    #[test]
    fn starts_with_no_match() {
        let mut root = new_trie_node();
        trie_insert(&mut root, "hello", "h");
        let results = trie_starts_with(&root, "xyz");
        assert!(results.is_empty());
    }

    #[test]
    fn empty_prefix_matches_all() {
        let mut root = new_trie_node();
        trie_insert(&mut root, "a", "1");
        trie_insert(&mut root, "b", "2");
        let results = trie_starts_with(&root, "");
        assert!(results.len() >= 2);
    }

    #[test]
    fn partial_key_not_found() {
        let mut root = new_trie_node();
        trie_insert(&mut root, "hello", "world");
        assert!(trie_find(&root, "hell").is_none());
    }
}
