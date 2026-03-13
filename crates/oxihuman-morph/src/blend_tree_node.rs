#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

/// Type of a blend tree node.
#[derive(Debug, Clone)]
pub enum BlendNodeKind {
    Clip(String),
    Lerp,
    Additive,
    Override,
}

/// A single node in the blend tree.
#[derive(Debug, Clone)]
pub struct BlendTreeNodeEntry {
    pub id: u32,
    pub node_type: BlendNodeKind,
    pub weight: f32,
    pub children: Vec<u32>,
}

/// An animation blend tree.
#[derive(Debug, Clone)]
pub struct AnimBlendTree {
    pub nodes: Vec<BlendTreeNodeEntry>,
    pub root: Option<u32>,
}

#[allow(dead_code)]
pub fn new_anim_blend_tree() -> AnimBlendTree {
    AnimBlendTree {
        nodes: Vec::new(),
        root: None,
    }
}

#[allow(dead_code)]
pub fn add_clip_node(tree: &mut AnimBlendTree, clip: &str, weight: f32) -> u32 {
    let id = tree.nodes.len() as u32;
    tree.nodes.push(BlendTreeNodeEntry {
        id,
        node_type: BlendNodeKind::Clip(clip.to_string()),
        weight,
        children: Vec::new(),
    });
    if tree.root.is_none() {
        tree.root = Some(id);
    }
    id
}

#[allow(dead_code)]
pub fn add_lerp_node(tree: &mut AnimBlendTree, weight: f32) -> u32 {
    let id = tree.nodes.len() as u32;
    tree.nodes.push(BlendTreeNodeEntry {
        id,
        node_type: BlendNodeKind::Lerp,
        weight,
        children: Vec::new(),
    });
    id
}

#[allow(dead_code)]
pub fn connect_node(tree: &mut AnimBlendTree, parent: u32, child: u32) {
    if let Some(node) = tree.nodes.iter_mut().find(|n| n.id == parent) {
        node.children.push(child);
    }
}

#[allow(dead_code)]
pub fn evaluate_anim_blend_tree(tree: &AnimBlendTree) -> f32 {
    if tree.nodes.is_empty() {
        return 0.0;
    }
    let total: f32 = tree.nodes.iter().map(|n| n.weight).sum();
    let count = tree.nodes.len() as f32;
    if count > 0.0 {
        total / count
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_tree() {
        let t = new_anim_blend_tree();
        assert!(t.nodes.is_empty());
        assert!(t.root.is_none());
    }

    #[test]
    fn test_add_clip_node_sets_root() {
        let mut t = new_anim_blend_tree();
        let id = add_clip_node(&mut t, "walk", 1.0);
        assert_eq!(id, 0);
        assert_eq!(t.root, Some(0));
    }

    #[test]
    fn test_add_lerp_node() {
        let mut t = new_anim_blend_tree();
        let id = add_lerp_node(&mut t, 0.5);
        assert_eq!(t.nodes.len(), 1);
        assert!((t.nodes[id as usize].weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_connect_node() {
        let mut t = new_anim_blend_tree();
        let parent = add_lerp_node(&mut t, 1.0);
        let child = add_clip_node(&mut t, "run", 0.6);
        connect_node(&mut t, parent, child);
        assert!(t.nodes[parent as usize].children.contains(&child));
    }

    #[test]
    fn test_evaluate_empty() {
        let t = new_anim_blend_tree();
        assert!((evaluate_anim_blend_tree(&t)).abs() < 1e-6);
    }

    #[test]
    fn test_evaluate_single_node() {
        let mut t = new_anim_blend_tree();
        add_clip_node(&mut t, "idle", 0.8);
        let v = evaluate_anim_blend_tree(&t);
        assert!((v - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_node_count() {
        let mut t = new_anim_blend_tree();
        add_clip_node(&mut t, "a", 0.5);
        add_lerp_node(&mut t, 0.3);
        assert_eq!(t.nodes.len(), 2);
    }

    #[test]
    fn test_clip_name_stored() {
        let mut t = new_anim_blend_tree();
        add_clip_node(&mut t, "sprint", 1.0);
        if let BlendNodeKind::Clip(ref name) = t.nodes[0].node_type {
            assert_eq!(name, "sprint");
        } else {
            panic!("wrong node type");
        }
    }

    #[test]
    fn test_second_clip_no_root_change() {
        let mut t = new_anim_blend_tree();
        add_clip_node(&mut t, "walk", 1.0);
        add_clip_node(&mut t, "run", 0.5);
        assert_eq!(t.root, Some(0)); // root stays as first
    }
}
