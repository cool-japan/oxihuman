// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Articulated rigid body tree (Featherstone stub).

/// A body in the articulated tree.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RigidTreeBody {
    pub name: String,
    pub parent: Option<usize>,
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    pub inertia: [f32; 3],
    pub joint_pos: [f32; 3],
    pub joint_axis: [f32; 3],
    pub joint_angle: f32,
    pub joint_vel: f32,
}

impl RigidTreeBody {
    #[allow(dead_code)]
    pub fn new(name: &str, mass: f32) -> Self {
        Self {
            name: name.to_string(),
            parent: None,
            pos: [0.0; 3],
            vel: [0.0; 3],
            mass,
            inertia: [1.0, 1.0, 1.0],
            joint_pos: [0.0; 3],
            joint_axis: [0.0, 0.0, 1.0],
            joint_angle: 0.0,
            joint_vel: 0.0,
        }
    }
}

/// Articulated rigid body tree.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct RigidTree {
    pub bodies: Vec<RigidTreeBody>,
}

impl RigidTree {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a root body.
    #[allow(dead_code)]
    pub fn add_root(&mut self, body: RigidTreeBody) -> usize {
        let idx = self.bodies.len();
        let mut b = body;
        b.parent = None;
        self.bodies.push(b);
        idx
    }

    /// Add a child body attached to parent.
    #[allow(dead_code)]
    pub fn add_child(&mut self, mut body: RigidTreeBody, parent: usize) -> usize {
        let idx = self.bodies.len();
        body.parent = Some(parent);
        self.bodies.push(body);
        idx
    }

    /// Number of bodies.
    #[allow(dead_code)]
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    /// Get children of a body.
    #[allow(dead_code)]
    pub fn children(&self, parent: usize) -> Vec<usize> {
        self.bodies
            .iter()
            .enumerate()
            .filter(|(_, b)| b.parent == Some(parent))
            .map(|(i, _)| i)
            .collect()
    }

    /// Total mass of the tree.
    #[allow(dead_code)]
    pub fn total_mass(&self) -> f32 {
        self.bodies.iter().map(|b| b.mass).sum()
    }

    /// Forward kinematics: propagate positions from root.
    #[allow(dead_code)]
    pub fn forward_kinematics(&mut self) {
        let n = self.bodies.len();
        for i in 0..n {
            if let Some(p) = self.bodies[i].parent {
                if p < i {
                    let parent_pos = self.bodies[p].pos;
                    let joint = self.bodies[i].joint_pos;
                    let angle = self.bodies[i].joint_angle;
                    let axis = self.bodies[i].joint_axis;
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();
                    let rot_offset = [
                        joint[0] * cos_a - joint[1] * sin_a * axis[2],
                        joint[0] * sin_a * axis[2] + joint[1] * cos_a,
                        joint[2],
                    ];
                    for k in 0..3 {
                        self.bodies[i].pos[k] = parent_pos[k] + rot_offset[k];
                    }
                }
            }
        }
    }

    /// Apply torque to a joint (stub).
    #[allow(dead_code)]
    pub fn apply_joint_torque(&mut self, body_idx: usize, torque: f32, dt: f32) {
        if body_idx >= self.bodies.len() {
            return;
        }
        let b = &mut self.bodies[body_idx];
        let inertia_z = b.inertia[2].max(1e-10);
        b.joint_vel += torque / inertia_z * dt;
        b.joint_angle += b.joint_vel * dt;
    }

    /// Set joint angle for a body.
    #[allow(dead_code)]
    pub fn set_joint_angle(&mut self, body_idx: usize, angle: f32) {
        if body_idx < self.bodies.len() {
            self.bodies[body_idx].joint_angle = angle;
        }
    }

    /// Get joint angle.
    #[allow(dead_code)]
    pub fn joint_angle(&self, body_idx: usize) -> f32 {
        if body_idx < self.bodies.len() {
            self.bodies[body_idx].joint_angle
        } else {
            0.0
        }
    }

    /// Simple default humanoid skeleton.
    #[allow(dead_code)]
    pub fn default_humanoid() -> Self {
        let mut tree = Self::new();
        let torso = RigidTreeBody::new("torso", 30.0);
        let ti = tree.add_root(torso);
        let mut head = RigidTreeBody::new("head", 5.0);
        head.joint_pos = [0.0, 0.3, 0.0];
        tree.add_child(head, ti);
        let mut arm_l = RigidTreeBody::new("arm_l", 4.0);
        arm_l.joint_pos = [-0.2, 0.2, 0.0];
        tree.add_child(arm_l, ti);
        let mut arm_r = RigidTreeBody::new("arm_r", 4.0);
        arm_r.joint_pos = [0.2, 0.2, 0.0];
        tree.add_child(arm_r, ti);
        tree
    }
}

/// Build a simple 2-link chain.
#[allow(dead_code)]
pub fn build_chain(link_mass: f32, n: usize) -> RigidTree {
    let mut tree = RigidTree::new();
    let mut prev = {
        let root = RigidTreeBody::new("link_0", link_mass);
        tree.add_root(root)
    };
    for i in 1..n {
        let mut body = RigidTreeBody::new(&format!("link_{}", i), link_mass);
        body.joint_pos = [0.0, 1.0, 0.0];
        prev = tree.add_child(body, prev);
    }
    tree
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_root_and_child() {
        let mut tree = RigidTree::new();
        let root = RigidTreeBody::new("root", 10.0);
        let ri = tree.add_root(root);
        let child = RigidTreeBody::new("child", 5.0);
        tree.add_child(child, ri);
        assert_eq!(tree.body_count(), 2);
    }

    #[test]
    fn total_mass_sums_all_bodies() {
        let tree = RigidTree::default_humanoid();
        let m = tree.total_mass();
        assert!(m > 30.0);
    }

    #[test]
    fn children_of_root() {
        let tree = RigidTree::default_humanoid();
        let ch = tree.children(0);
        assert!(!ch.is_empty());
    }

    #[test]
    fn set_and_get_joint_angle() {
        let mut tree = RigidTree::default_humanoid();
        tree.set_joint_angle(1, std::f32::consts::FRAC_PI_4);
        assert!((tree.joint_angle(1) - std::f32::consts::FRAC_PI_4).abs() < 1e-5);
    }

    #[test]
    fn apply_torque_changes_angle() {
        let mut tree = RigidTree::default_humanoid();
        tree.apply_joint_torque(1, 10.0, 0.1);
        assert!(tree.joint_angle(1).abs() > 1e-6);
    }

    #[test]
    fn forward_kinematics_runs_without_panic() {
        let mut tree = RigidTree::default_humanoid();
        tree.forward_kinematics();
    }

    #[test]
    fn build_chain_length() {
        let tree = build_chain(1.0, 5);
        assert_eq!(tree.body_count(), 5);
    }

    #[test]
    fn chain_total_mass() {
        let tree = build_chain(2.0, 3);
        assert!((tree.total_mass() - 6.0).abs() < 1e-5);
    }

    #[test]
    fn root_has_no_parent() {
        let tree = RigidTree::default_humanoid();
        assert!(tree.bodies[0].parent.is_none());
    }

    #[test]
    fn child_has_correct_parent() {
        let mut tree = RigidTree::new();
        let ri = tree.add_root(RigidTreeBody::new("root", 1.0));
        tree.add_child(RigidTreeBody::new("child", 1.0), ri);
        assert_eq!(tree.bodies[1].parent, Some(0));
    }
}
