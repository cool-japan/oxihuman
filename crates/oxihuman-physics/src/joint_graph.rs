// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::collections::HashMap;

/// A graph of joint connections between physics bodies.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct JointGraph {
    adjacency: HashMap<u32, Vec<JointEdge>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct JointEdge {
    pub other_body: u32,
    pub joint_id: u32,
    pub joint_type: JointType,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JointType {
    Fixed,
    Hinge,
    Ball,
    Slider,
    Spring,
}

#[allow(dead_code)]
impl JointGraph {
    pub fn new() -> Self {
        Self {
            adjacency: HashMap::new(),
        }
    }

    pub fn add_joint(&mut self, body_a: u32, body_b: u32, joint_id: u32, joint_type: JointType) {
        self.adjacency.entry(body_a).or_default().push(JointEdge {
            other_body: body_b,
            joint_id,
            joint_type,
        });
        self.adjacency.entry(body_b).or_default().push(JointEdge {
            other_body: body_a,
            joint_id,
            joint_type,
        });
    }

    pub fn remove_joint(&mut self, joint_id: u32) {
        for edges in self.adjacency.values_mut() {
            edges.retain(|e| e.joint_id != joint_id);
        }
    }

    pub fn joints_for_body(&self, body_id: u32) -> &[JointEdge] {
        self.adjacency.get(&body_id).map_or(&[], |v| v.as_slice())
    }

    pub fn are_connected(&self, body_a: u32, body_b: u32) -> bool {
        self.adjacency
            .get(&body_a)
            .is_some_and(|edges| edges.iter().any(|e| e.other_body == body_b))
    }

    pub fn num_bodies(&self) -> usize {
        self.adjacency.len()
    }

    pub fn num_joints(&self) -> usize {
        let total: usize = self.adjacency.values().map(|v| v.len()).sum();
        total / 2
    }

    pub fn degree(&self, body_id: u32) -> usize {
        self.adjacency.get(&body_id).map_or(0, |v| v.len())
    }

    pub fn clear(&mut self) {
        self.adjacency.clear();
    }

    pub fn body_ids(&self) -> Vec<u32> {
        self.adjacency.keys().copied().collect()
    }

    pub fn connected_component(&self, start: u32) -> Vec<u32> {
        let mut visited = Vec::new();
        let mut stack = vec![start];
        while let Some(body) = stack.pop() {
            if visited.contains(&body) {
                continue;
            }
            visited.push(body);
            for edge in self.joints_for_body(body) {
                if !visited.contains(&edge.other_body) {
                    stack.push(edge.other_body);
                }
            }
        }
        visited
    }
}

impl Default for JointGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let g = JointGraph::new();
        assert_eq!(g.num_bodies(), 0);
        assert_eq!(g.num_joints(), 0);
    }

    #[test]
    fn test_add_joint() {
        let mut g = JointGraph::new();
        g.add_joint(1, 2, 100, JointType::Hinge);
        assert!(g.are_connected(1, 2));
        assert!(g.are_connected(2, 1));
    }

    #[test]
    fn test_remove_joint() {
        let mut g = JointGraph::new();
        g.add_joint(1, 2, 100, JointType::Ball);
        g.remove_joint(100);
        assert!(!g.are_connected(1, 2));
    }

    #[test]
    fn test_joints_for_body() {
        let mut g = JointGraph::new();
        g.add_joint(1, 2, 10, JointType::Fixed);
        g.add_joint(1, 3, 11, JointType::Spring);
        assert_eq!(g.joints_for_body(1).len(), 2);
    }

    #[test]
    fn test_num_joints() {
        let mut g = JointGraph::new();
        g.add_joint(1, 2, 10, JointType::Hinge);
        g.add_joint(2, 3, 11, JointType::Ball);
        assert_eq!(g.num_joints(), 2);
    }

    #[test]
    fn test_degree() {
        let mut g = JointGraph::new();
        g.add_joint(1, 2, 10, JointType::Fixed);
        g.add_joint(1, 3, 11, JointType::Fixed);
        assert_eq!(g.degree(1), 2);
        assert_eq!(g.degree(2), 1);
    }

    #[test]
    fn test_clear() {
        let mut g = JointGraph::new();
        g.add_joint(1, 2, 10, JointType::Slider);
        g.clear();
        assert_eq!(g.num_bodies(), 0);
    }

    #[test]
    fn test_connected_component() {
        let mut g = JointGraph::new();
        g.add_joint(1, 2, 10, JointType::Hinge);
        g.add_joint(2, 3, 11, JointType::Ball);
        let component = g.connected_component(1);
        assert_eq!(component.len(), 3);
    }

    #[test]
    fn test_disconnected() {
        let mut g = JointGraph::new();
        g.add_joint(1, 2, 10, JointType::Fixed);
        g.add_joint(3, 4, 11, JointType::Fixed);
        let comp = g.connected_component(1);
        assert_eq!(comp.len(), 2);
        assert!(!comp.contains(&3));
    }

    #[test]
    fn test_not_connected() {
        let g = JointGraph::new();
        assert!(!g.are_connected(1, 2));
    }
}
