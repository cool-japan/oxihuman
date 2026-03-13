// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// 2D planar forward kinematics chain.
#[derive(Debug, Clone)]
pub struct ForwardKinematicsChain {
    pub joint_angles: Vec<f32>,
    pub link_lengths: Vec<f32>,
}

/// Create a new FK chain from link lengths (one angle per link, initially 0).
pub fn new_fk_chain(lengths: Vec<f32>) -> ForwardKinematicsChain {
    let n = lengths.len();
    ForwardKinematicsChain {
        joint_angles: vec![0.0; n],
        link_lengths: lengths,
    }
}

/// Set the angle (radians) for joint i.
pub fn fk_set_angle(chain: &mut ForwardKinematicsChain, i: usize, angle: f32) {
    if i < chain.joint_angles.len() {
        chain.joint_angles[i] = angle;
    }
}

/// 2D end-effector position (cumulative FK).
pub fn fk_end_effector_2d(chain: &ForwardKinematicsChain) -> [f32; 2] {
    let n = chain.link_lengths.len();
    if n == 0 {
        return [0.0, 0.0];
    }
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut cumulative_angle = 0.0f32;
    for i in 0..n {
        cumulative_angle += chain.joint_angles[i];
        x += chain.link_lengths[i] * cumulative_angle.cos();
        y += chain.link_lengths[i] * cumulative_angle.sin();
    }
    [x, y]
}

/// 2D position of joint i (sum of all links up to and including i).
pub fn fk_joint_position_2d(chain: &ForwardKinematicsChain, i: usize) -> [f32; 2] {
    let n = i.min(chain.link_lengths.len());
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut cumulative_angle = 0.0f32;
    for j in 0..n {
        cumulative_angle += chain.joint_angles[j];
        x += chain.link_lengths[j] * cumulative_angle.cos();
        y += chain.link_lengths[j] * cumulative_angle.sin();
    }
    [x, y]
}

/// Total chain length (sum of all link lengths).
pub fn fk_chain_length(chain: &ForwardKinematicsChain) -> f32 {
    chain.link_lengths.iter().sum()
}

/// Number of joints in the chain.
pub fn fk_joint_count(chain: &ForwardKinematicsChain) -> usize {
    chain.link_lengths.len()
}

/// Reachable workspace radius (= chain length when fully extended).
pub fn fk_workspace_radius(chain: &ForwardKinematicsChain) -> f32 {
    fk_chain_length(chain)
}

/// Distance from origin to end effector.
pub fn fk_end_effector_dist(chain: &ForwardKinematicsChain) -> f32 {
    let pos = fk_end_effector_2d(chain);
    (pos[0] * pos[0] + pos[1] * pos[1]).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn test_new_fk_chain() {
        /* chain with 3 links */
        let chain = new_fk_chain(vec![1.0, 1.0, 1.0]);
        assert_eq!(fk_joint_count(&chain), 3);
        assert!((fk_chain_length(&chain) - 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_fk_end_effector_straight() {
        /* all angles = 0 -> end effector on +x axis */
        let chain = new_fk_chain(vec![1.0, 1.0]);
        let pos = fk_end_effector_2d(&chain);
        assert!((pos[0] - 2.0).abs() < 1e-5);
        assert!(pos[1].abs() < 1e-5);
    }

    #[test]
    fn test_fk_end_effector_bent() {
        /* first joint 90°: link1 along +y, link2 along -x */
        let mut chain = new_fk_chain(vec![1.0, 1.0]);
        fk_set_angle(&mut chain, 0, FRAC_PI_2);
        fk_set_angle(&mut chain, 1, FRAC_PI_2);
        let pos = fk_end_effector_2d(&chain);
        /* should be at approx (−1, 1) */
        assert!((pos[1] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_fk_set_angle_out_of_bounds() {
        /* no panic */
        let mut chain = new_fk_chain(vec![1.0]);
        fk_set_angle(&mut chain, 10, 1.0);
    }

    #[test]
    fn test_fk_joint_position_origin() {
        /* position of joint 0 = origin */
        let chain = new_fk_chain(vec![1.0, 1.0]);
        let p = fk_joint_position_2d(&chain, 0);
        assert!(p[0].abs() < 1e-9 && p[1].abs() < 1e-9);
    }

    #[test]
    fn test_fk_chain_length_single() {
        let chain = new_fk_chain(vec![2.5]);
        assert!((fk_chain_length(&chain) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_fk_joint_count_empty() {
        let chain = new_fk_chain(vec![]);
        assert_eq!(fk_joint_count(&chain), 0);
        let pos = fk_end_effector_2d(&chain);
        assert!(pos[0].abs() < 1e-9);
    }

    #[test]
    fn test_fk_workspace_radius() {
        let chain = new_fk_chain(vec![1.0, 2.0, 0.5]);
        assert!((fk_workspace_radius(&chain) - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_fk_end_effector_dist_straight() {
        /* straight chain -> dist equals chain length */
        let chain = new_fk_chain(vec![1.0, 1.0, 1.0]);
        let d = fk_end_effector_dist(&chain);
        assert!((d - 3.0).abs() < 1e-5);
    }
}
