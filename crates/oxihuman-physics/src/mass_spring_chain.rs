// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

/// 1-D mass-spring chain.
#[allow(dead_code)]
pub struct ChainNode {
    pub pos: f32,
    pub vel: f32,
    pub mass: f32,
    pub pinned: bool,
}

#[allow(dead_code)]
pub struct MassSpringChain {
    pub nodes: Vec<ChainNode>,
    pub stiffness: f32,
    pub rest_length: f32,
    pub damping: f32,
}

#[allow(dead_code)]
impl MassSpringChain {
    pub fn new(n: usize, spacing: f32, mass: f32, stiffness: f32, damping: f32) -> Self {
        let nodes = (0..n)
            .map(|i| ChainNode {
                pos: i as f32 * spacing,
                vel: 0.0,
                mass,
                pinned: i == 0,
            })
            .collect();
        Self {
            nodes,
            stiffness,
            rest_length: spacing,
            damping,
        }
    }
    pub fn step(&mut self, dt: f32, gravity: f32) {
        let n = self.nodes.len();
        let mut forces = vec![0.0f32; n];
        for i in 0..n {
            forces[i] += -gravity * self.nodes[i].mass;
            if i > 0 {
                let dx = self.nodes[i].pos - self.nodes[i - 1].pos;
                let f = self.stiffness * (dx - self.rest_length);
                forces[i] -= f;
                forces[i - 1] += f;
            }
            if i + 1 < n {
                let dx = self.nodes[i + 1].pos - self.nodes[i].pos;
                let f = self.stiffness * (dx - self.rest_length);
                forces[i] += f;
            }
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            if self.nodes[i].pinned {
                continue;
            }
            let acc = forces[i] / self.nodes[i].mass;
            self.nodes[i].vel = self.nodes[i].vel * (1.0 - self.damping * dt) + acc * dt;
            self.nodes[i].pos += self.nodes[i].vel * dt;
        }
    }
    pub fn total_kinetic_energy(&self) -> f32 {
        self.nodes
            .iter()
            .map(|n| 0.5 * n.mass * n.vel * n.vel)
            .sum()
    }
    pub fn total_potential_energy(&self) -> f32 {
        let mut e = 0.0f32;
        for i in 1..self.nodes.len() {
            let dx = self.nodes[i].pos - self.nodes[i - 1].pos - self.rest_length;
            e += 0.5 * self.stiffness * dx * dx;
        }
        e
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn span(&self) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        self.nodes[self.nodes.len() - 1].pos - self.nodes[0].pos
    }
    pub fn max_velocity(&self) -> f32 {
        self.nodes
            .iter()
            .map(|n| n.vel.abs())
            .fold(0.0f32, f32::max)
    }
    pub fn reset_velocities(&mut self) {
        for n in &mut self.nodes {
            n.vel = 0.0;
        }
    }
}

#[allow(dead_code)]
pub fn new_mass_spring_chain(n: usize, spacing: f32, mass: f32, k: f32, d: f32) -> MassSpringChain {
    MassSpringChain::new(n, spacing, mass, k, d)
}
#[allow(dead_code)]
pub fn msc_step(c: &mut MassSpringChain, dt: f32, gravity: f32) {
    c.step(dt, gravity);
}
#[allow(dead_code)]
pub fn msc_kinetic_energy(c: &MassSpringChain) -> f32 {
    c.total_kinetic_energy()
}
#[allow(dead_code)]
pub fn msc_potential_energy(c: &MassSpringChain) -> f32 {
    c.total_potential_energy()
}
#[allow(dead_code)]
pub fn msc_node_count(c: &MassSpringChain) -> usize {
    c.node_count()
}
#[allow(dead_code)]
pub fn msc_span(c: &MassSpringChain) -> f32 {
    c.span()
}
#[allow(dead_code)]
pub fn msc_max_velocity(c: &MassSpringChain) -> f32 {
    c.max_velocity()
}
#[allow(dead_code)]
pub fn msc_reset_velocities(c: &mut MassSpringChain) {
    c.reset_velocities();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_node_count() {
        let c = new_mass_spring_chain(5, 1.0, 1.0, 10.0, 0.1);
        assert_eq!(msc_node_count(&c), 5);
    }
    #[test]
    fn test_first_node_pinned() {
        let c = new_mass_spring_chain(3, 1.0, 1.0, 10.0, 0.0);
        assert!(c.nodes[0].pinned);
    }
    #[test]
    fn test_step_doesnt_panic() {
        let mut c = new_mass_spring_chain(4, 1.0, 1.0, 10.0, 0.1);
        msc_step(&mut c, 0.01, 9.81);
    }
    #[test]
    fn test_pinned_node_doesnt_move() {
        let mut c = new_mass_spring_chain(3, 1.0, 1.0, 10.0, 0.0);
        let pos0 = c.nodes[0].pos;
        for _ in 0..10 {
            msc_step(&mut c, 0.01, 0.0);
        }
        assert_eq!(c.nodes[0].pos, pos0);
    }
    #[test]
    fn test_kinetic_energy_zero_initially() {
        let c = new_mass_spring_chain(3, 1.0, 1.0, 10.0, 0.0);
        assert_eq!(msc_kinetic_energy(&c), 0.0);
    }
    #[test]
    fn test_reset_velocities() {
        let mut c = new_mass_spring_chain(3, 1.0, 1.0, 10.0, 0.0);
        for _ in 0..5 {
            msc_step(&mut c, 0.01, 9.81);
        }
        msc_reset_velocities(&mut c);
        assert_eq!(msc_kinetic_energy(&c), 0.0);
    }
    #[test]
    fn test_potential_energy_at_rest_zero() {
        let c = new_mass_spring_chain(3, 1.0, 1.0, 10.0, 0.0);
        assert!((msc_potential_energy(&c)).abs() < 1e-5);
    }
    #[test]
    fn test_span() {
        let c = new_mass_spring_chain(4, 2.0, 1.0, 10.0, 0.0);
        assert!((msc_span(&c) - 6.0).abs() < 1e-5);
    }
    #[test]
    fn test_max_velocity() {
        let c = new_mass_spring_chain(3, 1.0, 1.0, 10.0, 0.0);
        assert_eq!(msc_max_velocity(&c), 0.0);
    }
    #[test]
    fn test_gravity_causes_motion() {
        let mut c = new_mass_spring_chain(2, 1.0, 1.0, 0.1, 0.0);
        for _ in 0..10 {
            msc_step(&mut c, 0.01, 9.81);
        }
        assert!(msc_max_velocity(&c) > 0.0);
    }
}
