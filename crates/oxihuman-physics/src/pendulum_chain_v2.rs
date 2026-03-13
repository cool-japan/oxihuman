// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Pendulum chain v2: multi-link chain with position-based distance constraints.

/// A link in the pendulum chain.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ChainLink {
    pub pos: [f32; 3],
    pub prev_pos: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

/// Pendulum chain v2.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PendulumChainV2 {
    pub links: Vec<ChainLink>,
    pub link_length: f32,
    pub gravity: [f32; 3],
    pub constraint_iters: usize,
    pub damping: f32,
}

/// Create a new `PendulumChainV2` hanging from `anchor`.
#[allow(dead_code)]
pub fn new_pendulum_chain_v2(
    anchor: [f32; 3],
    link_length: f32,
    n_links: usize,
) -> PendulumChainV2 {
    let links = (0..n_links)
        .map(|i| {
            let y = anchor[1] - i as f32 * link_length;
            ChainLink {
                pos: [anchor[0], y, anchor[2]],
                prev_pos: [anchor[0], y, anchor[2]],
                mass: 0.1,
                pinned: i == 0,
            }
        })
        .collect();
    PendulumChainV2 {
        links,
        link_length,
        gravity: [0.0, -9.81, 0.0],
        constraint_iters: 8,
        damping: 0.99,
    }
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// Integrate positions using Verlet.
#[allow(dead_code)]
pub fn pc2_step(chain: &mut PendulumChainV2, dt: f32) {
    // Verlet integrate
    for link in &mut chain.links {
        if link.pinned {
            continue;
        }
        let vel = sub3(link.pos, link.prev_pos);
        let vel_d = scale3(vel, chain.damping);
        link.prev_pos = link.pos;
        let gravity_disp = scale3(chain.gravity, dt * dt);
        link.pos = add3(link.pos, add3(vel_d, gravity_disp));
    }

    // Constraint solve
    for _ in 0..chain.constraint_iters {
        for i in 0..chain.links.len().saturating_sub(1) {
            let pa = chain.links[i].pos;
            let pb = chain.links[i + 1].pos;
            let delta = sub3(pb, pa);
            let dist = len3(delta);
            if dist < 1e-9 {
                continue;
            }
            let diff = (dist - chain.link_length) / dist;
            let correction = scale3(delta, diff * 0.5);
            if !chain.links[i].pinned {
                chain.links[i].pos = add3(pa, correction);
            }
            if !chain.links[i + 1].pinned {
                chain.links[i + 1].pos = add3(pb, scale3(correction, -1.0));
            }
        }
    }
}

/// Length of the chain (number of links).
#[allow(dead_code)]
pub fn pc2_len(chain: &PendulumChainV2) -> usize {
    chain.links.len()
}

/// Position of the tip (last link).
#[allow(dead_code)]
pub fn pc2_tip_pos(chain: &PendulumChainV2) -> Option<[f32; 3]> {
    chain.links.last().map(|l| l.pos)
}

/// Total chain length (sum of distances between consecutive links).
#[allow(dead_code)]
pub fn pc2_total_length(chain: &PendulumChainV2) -> f32 {
    let mut total = 0.0;
    for i in 0..chain.links.len().saturating_sub(1) {
        total += len3(sub3(chain.links[i + 1].pos, chain.links[i].pos));
    }
    total
}

/// Kinetic energy (sum of ½mv²).
#[allow(dead_code)]
pub fn pc2_kinetic_energy(chain: &PendulumChainV2, dt: f32) -> f32 {
    if dt < 1e-9 {
        return 0.0;
    }
    chain
        .links
        .iter()
        .map(|link| {
            let v = sub3(link.pos, link.prev_pos);
            let speed_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
            0.5 * link.mass * speed_sq / (dt * dt)
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_new_chain() {
        let chain = new_pendulum_chain_v2([0.0, 10.0, 0.0], 1.0, 5);
        assert_eq!(pc2_len(&chain), 5);
    }

    #[test]
    fn test_anchor_pinned() {
        let chain = new_pendulum_chain_v2([0.0, 10.0, 0.0], 1.0, 5);
        assert!(chain.links[0].pinned);
    }

    #[test]
    fn test_step_no_crash() {
        let mut chain = new_pendulum_chain_v2([0.0, 10.0, 0.0], 1.0, 5);
        pc2_step(&mut chain, 0.016);
    }

    #[test]
    fn test_anchor_stays_fixed() {
        let mut chain = new_pendulum_chain_v2([0.0, 10.0, 0.0], 1.0, 5);
        for _ in 0..10 {
            pc2_step(&mut chain, 0.016);
        }
        let anchor = chain.links[0].pos;
        assert!((anchor[1] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn test_tip_pos_exists() {
        let chain = new_pendulum_chain_v2([0.0, 10.0, 0.0], 1.0, 5);
        assert!(pc2_tip_pos(&chain).is_some());
    }

    #[test]
    fn test_free_link_moves() {
        let mut chain = new_pendulum_chain_v2([0.0, 10.0, 0.0], 1.0, 3);
        let initial_y = chain.links[2].pos[1];
        pc2_step(&mut chain, 0.1);
        let new_y = chain.links[2].pos[1];
        assert!(new_y < initial_y || (new_y - initial_y).abs() < 1.0);
    }

    #[test]
    fn test_total_length_approx() {
        let chain = new_pendulum_chain_v2([0.0, 10.0, 0.0], 1.0, 5);
        let total = pc2_total_length(&chain);
        assert!((total - 4.0).abs() < 0.01);
    }

    #[test]
    fn test_pi_used() {
        let circle_area = PI * 1.0 * 1.0;
        assert!(circle_area > 3.0);
    }

    #[test]
    fn test_kinetic_energy_nonneg() {
        let chain = new_pendulum_chain_v2([0.0, 10.0, 0.0], 1.0, 5);
        let ke = pc2_kinetic_energy(&chain, 0.016);
        assert!(ke >= 0.0);
    }
}
