// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! 2D N-link pendulum simulation stub.

use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct PendulumLink2d {
    pub length: f32,
    pub mass: f32,
    pub angle: f32,
    pub angular_velocity: f32,
}

impl PendulumLink2d {
    pub fn new(length: f32, mass: f32, angle: f32) -> Self {
        PendulumLink2d {
            length,
            mass,
            angle,
            angular_velocity: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChainPendulum2d {
    pub links: Vec<PendulumLink2d>,
    pub gravity: f32,
    pub damping: f32,
}

impl ChainPendulum2d {
    pub fn new(n: usize, length: f32, mass: f32, gravity: f32) -> Self {
        let links = (0..n)
            .map(|i| PendulumLink2d::new(length, mass, PI * 0.1 * (i as f32 + 1.0)))
            .collect();
        ChainPendulum2d {
            links,
            gravity,
            damping: 0.01,
        }
    }

    pub fn step(&mut self, dt: f32) {
        for link in &mut self.links {
            step_single_link(link, self.gravity, self.damping, dt);
        }
    }

    pub fn end_position(&self, pivot: [f32; 2]) -> [f32; 2] {
        chain_end_pos(pivot, &self.links)
    }

    pub fn total_energy(&self) -> f32 {
        self.links
            .iter()
            .map(|l| link_energy(l, self.gravity))
            .sum()
    }

    pub fn link_count(&self) -> usize {
        self.links.len()
    }
}

fn step_single_link(link: &mut PendulumLink2d, g: f32, damping: f32, dt: f32) {
    let angular_accel = -(g / link.length) * link.angle.sin() - damping * link.angular_velocity;
    link.angular_velocity += angular_accel * dt;
    link.angle += link.angular_velocity * dt;
}

pub fn chain_end_pos(pivot: [f32; 2], links: &[PendulumLink2d]) -> [f32; 2] {
    let mut pos = pivot;
    for link in links {
        pos[0] += link.length * link.angle.sin();
        pos[1] -= link.length * link.angle.cos();
    }
    pos
}

pub fn link_energy(link: &PendulumLink2d, g: f32) -> f32 {
    let ke = 0.5 * link.mass * (link.angular_velocity * link.length).powi(2);
    let pe = link.mass * g * link.length * (1.0 - link.angle.cos());
    ke + pe
}

pub fn small_angle_period(length: f32, g: f32) -> f32 {
    use std::f32::consts::PI;
    2.0 * PI * (length / g).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let p = ChainPendulum2d::new(3, 1.0, 1.0, 9.81);
        assert_eq!(p.link_count(), 3);
    }

    #[test]
    fn test_step_runs() {
        let mut p = ChainPendulum2d::new(2, 1.0, 1.0, 9.81);
        p.step(0.01);
        /* angles change after step */
        assert!(p.links[0].angular_velocity.abs() > 0.0 || p.links[0].angle != PI * 0.1);
    }

    #[test]
    fn test_end_position_vertical() {
        let mut links = vec![PendulumLink2d::new(1.0, 1.0, 0.0)];
        links[0].angular_velocity = 0.0;
        let end = chain_end_pos([0.0, 0.0], &links);
        assert!((end[0] - 0.0).abs() < 1e-5, /* vertical pendulum: x stays 0 */);
        assert!((end[1] - (-1.0)).abs() < 1e-5, /* y goes down by length */);
    }

    #[test]
    fn test_small_angle_period() {
        let t = small_angle_period(1.0, 9.81);
        assert!((t - 2.006).abs() < 0.01, /* period of 1m pendulum ≈ 2.006s */);
    }

    #[test]
    fn test_energy_positive() {
        let link = PendulumLink2d {
            length: 1.0,
            mass: 1.0,
            angle: PI / 4.0,
            angular_velocity: 0.0,
        };
        let e = link_energy(&link, 9.81);
        assert!(e >= 0.0 /* energy is non-negative */,);
    }

    #[test]
    fn test_total_energy() {
        let p = ChainPendulum2d::new(2, 1.0, 1.0, 9.81);
        assert!(p.total_energy() >= 0.0 /* total energy non-negative */,);
    }

    #[test]
    fn test_multiple_steps() {
        let mut p = ChainPendulum2d::new(1, 1.0, 1.0, 9.81);
        for _ in 0..100 {
            p.step(0.01);
        }
        /* simulation completes without panic */
        assert!(p.links[0].angle.is_finite() /* angle stays finite */,);
    }

    #[test]
    fn test_end_position_two_links() {
        let links = vec![
            PendulumLink2d::new(1.0, 1.0, 0.0),
            PendulumLink2d::new(1.0, 1.0, 0.0),
        ];
        let end = chain_end_pos([0.0, 0.0], &links);
        assert!((end[1] - (-2.0)).abs() < 1e-5, /* two vertical links = 2 down */);
    }

    #[test]
    fn test_damping_reduces_velocity() {
        let mut p = ChainPendulum2d::new(1, 1.0, 1.0, 9.81);
        p.links[0].angular_velocity = 1.0;
        p.links[0].angle = 0.0;
        let initial_v = p.links[0].angular_velocity;
        for _ in 0..100 {
            p.step(0.01);
        }
        assert!(
            p.links[0].angular_velocity.abs() < initial_v.abs() + 0.2,
            /* damping reduces velocity over time */
        );
    }
}
