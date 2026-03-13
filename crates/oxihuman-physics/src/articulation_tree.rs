// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Articulated body tree for multi-body dynamics.

#![allow(dead_code)]

use std::f32::consts::PI;

/// A node in an articulation tree representing a single link.
#[allow(dead_code)]
pub struct ArticulationNode {
    pub id: u32,
    pub parent: Option<u32>,
    pub children: Vec<u32>,
    pub joint_angle: f32,
    pub joint_velocity: f32,
    pub link_length: f32,
    pub mass: f32,
}

/// An articulated body tree.
#[allow(dead_code)]
pub struct ArticulationTree {
    pub nodes: Vec<ArticulationNode>,
}

/// Create a new empty articulation tree.
#[allow(dead_code)]
pub fn new_articulation_tree() -> ArticulationTree {
    ArticulationTree { nodes: Vec::new() }
}

/// Add a link to the tree. Returns the new node's id.
#[allow(dead_code)]
pub fn add_link(
    tree: &mut ArticulationTree,
    parent: Option<u32>,
    length: f32,
    mass: f32,
) -> u32 {
    let id = tree.nodes.len() as u32;
    if let Some(pid) = parent {
        if let Some(p) = tree.nodes.iter_mut().find(|n| n.id == pid) {
            p.children.push(id);
        }
    }
    tree.nodes.push(ArticulationNode {
        id,
        parent,
        children: Vec::new(),
        joint_angle: 0.0,
        joint_velocity: 0.0,
        link_length: length,
        mass,
    });
    id
}

/// Set the joint angle (in radians) for a node with the given id.
#[allow(dead_code)]
pub fn set_joint_angle(tree: &mut ArticulationTree, id: u32, angle: f32) {
    if let Some(node) = tree.nodes.iter_mut().find(|n| n.id == id) {
        // Clamp to [-PI, PI]
        let clamped = angle.clamp(-PI, PI);
        node.joint_angle = clamped;
    }
}

/// Compute the world position of a node's end-effector via forward kinematics.
/// Traverses the chain of parents from root to this node, accumulating rotations.
#[allow(dead_code)]
pub fn node_world_pos(tree: &ArticulationTree, id: u32) -> [f32; 3] {
    // Build the chain from root to this node
    let mut chain: Vec<u32> = Vec::new();
    let mut current_id = id;
    loop {
        chain.push(current_id);
        if let Some(node) = tree.nodes.iter().find(|n| n.id == current_id) {
            match node.parent {
                Some(pid) => current_id = pid,
                None => break,
            }
        } else {
            break;
        }
    }
    chain.reverse();

    // Forward kinematics: accumulate angle and position
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut cumulative_angle = 0.0f32;

    for nid in &chain {
        if let Some(node) = tree.nodes.iter().find(|n| n.id == *nid) {
            cumulative_angle += node.joint_angle;
            x += cumulative_angle.cos() * node.link_length;
            y += cumulative_angle.sin() * node.link_length;
        }
    }
    [x, y, 0.0]
}

/// Return the number of links in the tree.
#[allow(dead_code)]
pub fn link_count(tree: &ArticulationTree) -> usize {
    tree.nodes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree_empty() {
        let t = new_articulation_tree();
        assert_eq!(link_count(&t), 0);
    }

    #[test]
    fn add_root_link() {
        let mut t = new_articulation_tree();
        let id = add_link(&mut t, None, 1.0, 1.0);
        assert_eq!(id, 0);
        assert_eq!(link_count(&t), 1);
    }

    #[test]
    fn add_child_link() {
        let mut t = new_articulation_tree();
        let root = add_link(&mut t, None, 1.0, 1.0);
        let child = add_link(&mut t, Some(root), 0.5, 0.5);
        assert_eq!(link_count(&t), 2);
        assert_eq!(t.nodes[0].children, vec![child]);
    }

    #[test]
    fn set_joint_angle_basic() {
        let mut t = new_articulation_tree();
        let id = add_link(&mut t, None, 1.0, 1.0);
        set_joint_angle(&mut t, id, 1.0);
        assert!((t.nodes[0].joint_angle - 1.0).abs() < 1e-5);
    }

    #[test]
    fn set_joint_angle_clamps_to_pi() {
        let mut t = new_articulation_tree();
        let id = add_link(&mut t, None, 1.0, 1.0);
        set_joint_angle(&mut t, id, 10.0);
        assert!(t.nodes[0].joint_angle <= PI);
    }

    #[test]
    fn world_pos_zero_angle_goes_right() {
        let mut t = new_articulation_tree();
        let id = add_link(&mut t, None, 1.0, 1.0);
        let pos = node_world_pos(&t, id);
        assert!((pos[0] - 1.0).abs() < 1e-5);
        assert!(pos[1].abs() < 1e-5);
    }

    #[test]
    fn world_pos_ninety_degree_goes_up() {
        let mut t = new_articulation_tree();
        let id = add_link(&mut t, None, 1.0, 1.0);
        set_joint_angle(&mut t, id, PI / 2.0);
        let pos = node_world_pos(&t, id);
        assert!(pos[0].abs() < 1e-4);
        assert!((pos[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn two_link_chain_forward_kinematics() {
        let mut t = new_articulation_tree();
        let root = add_link(&mut t, None, 1.0, 1.0);
        let child = add_link(&mut t, Some(root), 1.0, 1.0);
        // Both joints at 0 degrees → end effector at (2, 0, 0)
        let pos = node_world_pos(&t, child);
        assert!((pos[0] - 2.0).abs() < 1e-5);
        assert!(pos[1].abs() < 1e-5);
    }
}
