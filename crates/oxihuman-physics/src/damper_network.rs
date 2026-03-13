// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! A network of spring-damper elements connecting simulated nodes.

use std::f32::consts::PI;

/// A single node in the damper network.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DamperNode {
    pub id: u32,
    pub pos: f32,
    pub vel: f32,
    pub inv_mass: f32,
}

#[allow(dead_code)]
impl DamperNode {
    pub fn new(id: u32, pos: f32, inv_mass: f32) -> Self {
        Self {
            id,
            pos,
            vel: 0.0,
            inv_mass,
        }
    }

    pub fn is_fixed(&self) -> bool {
        self.inv_mass == 0.0
    }
}

/// A spring-damper link between two nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DamperLink {
    pub node_a: u32,
    pub node_b: u32,
    pub spring_k: f32,
    pub damping: f32,
    pub rest_length: f32,
}

/// A network of nodes connected by spring-damper links.
#[allow(dead_code)]
pub struct DamperNetwork {
    pub nodes: Vec<DamperNode>,
    pub links: Vec<DamperLink>,
}

#[allow(dead_code)]
impl DamperNetwork {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            links: Vec::new(),
        }
    }

    pub fn add_node(&mut self, pos: f32, inv_mass: f32) -> u32 {
        let id = self.nodes.len() as u32;
        self.nodes.push(DamperNode::new(id, pos, inv_mass));
        id
    }

    pub fn add_link(&mut self, node_a: u32, node_b: u32, spring_k: f32, damping: f32) {
        let rest = if let (Some(a), Some(b)) = (self.node(node_a), self.node(node_b)) {
            (b.pos - a.pos).abs()
        } else {
            0.0
        };
        self.links.push(DamperLink {
            node_a,
            node_b,
            spring_k,
            damping,
            rest_length: rest,
        });
    }

    pub fn node(&self, id: u32) -> Option<&DamperNode> {
        self.nodes.get(id as usize)
    }

    pub fn node_mut(&mut self, id: u32) -> Option<&mut DamperNode> {
        self.nodes.get_mut(id as usize)
    }

    /// Compute the force on each node from spring-damper links.
    fn compute_forces(&self) -> Vec<f32> {
        let mut forces = vec![0.0_f32; self.nodes.len()];
        for link in &self.links {
            let a = link.node_a as usize;
            let b = link.node_b as usize;
            if a >= self.nodes.len() || b >= self.nodes.len() {
                continue;
            }
            let dx = self.nodes[b].pos - self.nodes[a].pos;
            let dv = self.nodes[b].vel - self.nodes[a].vel;
            let spring_f = link.spring_k * (dx - link.rest_length);
            let damper_f = link.damping * dv;
            let f = spring_f + damper_f;
            forces[a] += f;
            forces[b] -= f;
        }
        forces
    }

    /// Integrate the network by one time step using semi-implicit Euler.
    pub fn step(&mut self, dt: f32) {
        let forces = self.compute_forces();
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if node.is_fixed() {
                continue;
            }
            node.vel += forces[i] * node.inv_mass * dt;
            node.pos += node.vel * dt;
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }

    /// Total elastic potential energy across all links.
    pub fn elastic_energy(&self) -> f32 {
        self.links
            .iter()
            .map(|link| {
                if let (Some(a), Some(b)) = (self.node(link.node_a), self.node(link.node_b)) {
                    let dx = (b.pos - a.pos) - link.rest_length;
                    0.5 * link.spring_k * dx * dx
                } else {
                    0.0
                }
            })
            .sum()
    }

    /// Total kinetic energy of all free nodes.
    pub fn kinetic_energy(&self) -> f32 {
        self.nodes
            .iter()
            .filter(|n| !n.is_fixed())
            .map(|n| 0.5 * (1.0 / n.inv_mass) * n.vel * n.vel)
            .sum()
    }

    /// Natural frequency of a two-node spring (omega = sqrt(k/m)).
    pub fn natural_frequency(&self, link_idx: usize) -> f32 {
        if link_idx >= self.links.len() {
            return 0.0;
        }
        let link = &self.links[link_idx];
        if let (Some(a), Some(b)) = (self.node(link.node_a), self.node(link.node_b)) {
            let reduced_mass = if a.inv_mass + b.inv_mass < 1e-9 {
                return 0.0;
            } else {
                1.0 / (a.inv_mass + b.inv_mass)
            };
            (link.spring_k / reduced_mass).sqrt() / (2.0 * PI)
        } else {
            0.0
        }
    }
}

impl Default for DamperNetwork {
    fn default() -> Self {
        Self::new()
    }
}

pub fn new_damper_network() -> DamperNetwork {
    DamperNetwork::new()
}

pub fn dn_add_node(net: &mut DamperNetwork, pos: f32, inv_mass: f32) -> u32 {
    net.add_node(pos, inv_mass)
}

pub fn dn_add_link(net: &mut DamperNetwork, a: u32, b: u32, k: f32, c: f32) {
    net.add_link(a, b, k, c);
}

pub fn dn_step(net: &mut DamperNetwork, dt: f32) {
    net.step(dt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_network_empty() {
        let net = new_damper_network();
        assert_eq!(net.node_count(), 0);
        assert_eq!(net.link_count(), 0);
    }

    #[test]
    fn add_node_increments_count() {
        let mut net = new_damper_network();
        dn_add_node(&mut net, 0.0, 1.0);
        assert_eq!(net.node_count(), 1);
    }

    #[test]
    fn fixed_node() {
        let mut net = new_damper_network();
        let id = dn_add_node(&mut net, 0.0, 0.0);
        assert!(net.node(id).expect("should succeed").is_fixed());
    }

    #[test]
    fn add_link_increments_count() {
        let mut net = new_damper_network();
        let a = dn_add_node(&mut net, 0.0, 1.0);
        let b = dn_add_node(&mut net, 1.0, 1.0);
        dn_add_link(&mut net, a, b, 100.0, 1.0);
        assert_eq!(net.link_count(), 1);
    }

    #[test]
    fn spring_restores_to_rest() {
        let mut net = new_damper_network();
        let a = dn_add_node(&mut net, 0.0, 0.0); // fixed
        let b = dn_add_node(&mut net, 2.0, 1.0); // rest len = 2.0 → displaced to 2.0 initially, no force
        dn_add_link(&mut net, a, b, 100.0, 0.5);
        // displace b
        net.node_mut(b).expect("should succeed").pos = 3.0;
        let old_pos = net.node(b).expect("should succeed").pos;
        dn_step(&mut net, 0.01);
        // b should move toward a
        assert!(net.node(b).expect("should succeed").pos < old_pos);
    }

    #[test]
    fn step_does_not_move_fixed_node() {
        let mut net = new_damper_network();
        let a = dn_add_node(&mut net, 0.0, 0.0);
        let b = dn_add_node(&mut net, 2.0, 1.0);
        dn_add_link(&mut net, a, b, 100.0, 0.0);
        dn_step(&mut net, 0.01);
        assert_eq!(net.node(a).expect("should succeed").pos, 0.0);
    }

    #[test]
    fn elastic_energy_at_rest_is_zero() {
        let mut net = new_damper_network();
        let a = dn_add_node(&mut net, 0.0, 0.0);
        let b = dn_add_node(&mut net, 1.0, 1.0);
        dn_add_link(&mut net, a, b, 100.0, 0.0);
        assert!(net.elastic_energy() < 1e-8);
    }

    #[test]
    fn kinetic_energy_after_step() {
        let mut net = new_damper_network();
        let a = dn_add_node(&mut net, 0.0, 0.0);
        let b = dn_add_node(&mut net, 2.0, 1.0);
        dn_add_link(&mut net, a, b, 100.0, 0.0);
        net.node_mut(b).expect("should succeed").pos = 3.0;
        dn_step(&mut net, 0.01);
        assert!(net.kinetic_energy() > 0.0);
    }

    #[test]
    fn natural_frequency_positive() {
        let mut net = new_damper_network();
        let a = dn_add_node(&mut net, 0.0, 0.0);
        let b = dn_add_node(&mut net, 1.0, 1.0);
        dn_add_link(&mut net, a, b, 100.0, 0.0);
        assert!(net.natural_frequency(0) > 0.0);
    }

    #[test]
    fn rest_length_stored() {
        let mut net = new_damper_network();
        let a = dn_add_node(&mut net, 0.0, 0.0);
        let b = dn_add_node(&mut net, 3.0, 1.0);
        dn_add_link(&mut net, a, b, 10.0, 0.0);
        assert!((net.links[0].rest_length - 3.0).abs() < 1e-5);
    }
}
