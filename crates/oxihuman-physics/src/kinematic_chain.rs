// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A link in a kinematic chain (articulated body).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ChainLink {
    pub length: f32,
    pub angle: f32,
    pub min_angle: f32,
    pub max_angle: f32,
}

/// A kinematic chain for forward/inverse kinematics.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct KinematicChain {
    links: Vec<ChainLink>,
    base: [f32; 2],
}

#[allow(dead_code)]
impl KinematicChain {
    pub fn new(base: [f32; 2]) -> Self {
        Self {
            links: Vec::new(),
            base,
        }
    }

    pub fn add_link(&mut self, length: f32, angle: f32, min_angle: f32, max_angle: f32) {
        self.links.push(ChainLink {
            length,
            angle,
            min_angle,
            max_angle,
        });
    }

    pub fn num_links(&self) -> usize {
        self.links.len()
    }

    pub fn total_length(&self) -> f32 {
        self.links.iter().map(|l| l.length).sum()
    }

    pub fn end_effector(&self) -> [f32; 2] {
        let mut x = self.base[0];
        let mut y = self.base[1];
        let mut cumulative_angle = 0.0f32;
        for link in &self.links {
            cumulative_angle += link.angle;
            x += link.length * cumulative_angle.cos();
            y += link.length * cumulative_angle.sin();
        }
        [x, y]
    }

    pub fn joint_positions(&self) -> Vec<[f32; 2]> {
        let mut positions = vec![self.base];
        let mut x = self.base[0];
        let mut y = self.base[1];
        let mut cumulative_angle = 0.0f32;
        for link in &self.links {
            cumulative_angle += link.angle;
            x += link.length * cumulative_angle.cos();
            y += link.length * cumulative_angle.sin();
            positions.push([x, y]);
        }
        positions
    }

    pub fn set_angle(&mut self, link_idx: usize, angle: f32) {
        if let Some(link) = self.links.get_mut(link_idx) {
            link.angle = angle.clamp(link.min_angle, link.max_angle);
        }
    }

    pub fn get_angle(&self, link_idx: usize) -> Option<f32> {
        self.links.get(link_idx).map(|l| l.angle)
    }

    pub fn can_reach(&self, target: [f32; 2]) -> bool {
        let dx = target[0] - self.base[0];
        let dy = target[1] - self.base[1];
        let dist = (dx * dx + dy * dy).sqrt();
        dist <= self.total_length()
    }

    pub fn distance_to_target(&self, target: [f32; 2]) -> f32 {
        let ee = self.end_effector();
        let dx = target[0] - ee[0];
        let dy = target[1] - ee[1];
        (dx * dx + dy * dy).sqrt()
    }

    pub fn base(&self) -> [f32; 2] {
        self.base
    }

    pub fn set_base(&mut self, base: [f32; 2]) {
        self.base = base;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new() {
        let chain = KinematicChain::new([0.0, 0.0]);
        assert_eq!(chain.num_links(), 0);
        assert_eq!(chain.base(), [0.0, 0.0]);
    }

    #[test]
    fn test_add_link() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.add_link(1.0, 0.0, -PI, PI);
        assert_eq!(chain.num_links(), 1);
    }

    #[test]
    fn test_total_length() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.add_link(2.0, 0.0, -PI, PI);
        chain.add_link(3.0, 0.0, -PI, PI);
        assert!((chain.total_length() - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_end_effector_straight() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.add_link(3.0, 0.0, -PI, PI);
        chain.add_link(2.0, 0.0, -PI, PI);
        let ee = chain.end_effector();
        assert!((ee[0] - 5.0).abs() < 1e-5);
        assert!(ee[1].abs() < 1e-5);
    }

    #[test]
    fn test_end_effector_bent() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.add_link(1.0, 0.0, -PI, PI);
        chain.add_link(1.0, PI / 2.0, -PI, PI);
        let ee = chain.end_effector();
        assert!((ee[0] - 1.0).abs() < 1e-5);
        assert!((ee[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_joint_positions() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.add_link(1.0, 0.0, -PI, PI);
        let positions = chain.joint_positions();
        assert_eq!(positions.len(), 2);
    }

    #[test]
    fn test_set_angle() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.add_link(1.0, 0.0, -1.0, 1.0);
        chain.set_angle(0, 0.5);
        assert!((chain.get_angle(0).unwrap() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_set_angle_clamped() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.add_link(1.0, 0.0, -1.0, 1.0);
        chain.set_angle(0, 5.0);
        assert!((chain.get_angle(0).unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_can_reach() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.add_link(3.0, 0.0, -PI, PI);
        assert!(chain.can_reach([2.0, 0.0]));
        assert!(!chain.can_reach([5.0, 0.0]));
    }

    #[test]
    fn test_distance_to_target() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.add_link(1.0, 0.0, -PI, PI);
        let dist = chain.distance_to_target([1.0, 0.0]);
        assert!(dist < 1e-5);
    }

    #[test]
    fn test_set_base() {
        let mut chain = KinematicChain::new([0.0, 0.0]);
        chain.set_base([5.0, 5.0]);
        assert_eq!(chain.base(), [5.0, 5.0]);
    }
}
