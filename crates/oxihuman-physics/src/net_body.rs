// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Net body: flexible net with nodes and cable links.

/// A node in the net.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct NetNode {
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

/// A link (cable) between two net nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NetLink {
    pub a: usize,
    pub b: usize,
    pub rest_len: f32,
    pub stiffness: f32,
    pub damping: f32,
}

/// Net body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NetBody {
    pub nodes: Vec<NetNode>,
    pub links: Vec<NetLink>,
    pub gravity: [f32; 3],
}

/// Create a new `NetBody` from node positions and links.
#[allow(dead_code)]
pub fn new_net_body(node_positions: &[[f32; 3]], mass: f32) -> NetBody {
    let nodes = node_positions
        .iter()
        .map(|&p| NetNode {
            pos: p,
            vel: [0.0; 3],
            mass: mass.max(1e-9),
            pinned: false,
        })
        .collect();
    NetBody {
        nodes,
        links: Vec::new(),
        gravity: [0.0, -9.81, 0.0],
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

/// Add a link between two nodes.
#[allow(dead_code)]
pub fn net_add_link(body: &mut NetBody, a: usize, b: usize, stiffness: f32) {
    if a < body.nodes.len() && b < body.nodes.len() {
        let rest_len = len3(sub3(body.nodes[b].pos, body.nodes[a].pos));
        body.links.push(NetLink {
            a,
            b,
            rest_len: rest_len.max(1e-9),
            stiffness,
            damping: 1.0,
        });
    }
}

/// Pin a node (it won't move).
#[allow(dead_code)]
pub fn net_pin(body: &mut NetBody, idx: usize) {
    if idx < body.nodes.len() {
        body.nodes[idx].pinned = true;
    }
}

/// Simulate one step.
#[allow(dead_code)]
#[allow(clippy::needless_range_loop)]
pub fn net_step(body: &mut NetBody, dt: f32) {
    let nn = body.nodes.len();
    let mut forces = vec![[0.0f32; 3]; nn];

    for i in 0..nn {
        forces[i] = add3(forces[i], scale3(body.gravity, body.nodes[i].mass));
    }

    for link in &body.links {
        let pa = body.nodes[link.a].pos;
        let pb = body.nodes[link.b].pos;
        let delta = sub3(pb, pa);
        let dist = len3(delta);
        if dist < 1e-9 {
            continue;
        }
        let stretch = dist - link.rest_len;
        if stretch <= 0.0 {
            continue;
        } // cables only in tension
        let dir = scale3(delta, 1.0 / dist);
        let f = scale3(dir, link.stiffness * stretch);
        forces[link.a] = add3(forces[link.a], f);
        forces[link.b] = add3(forces[link.b], scale3(f, -1.0));
    }

    for i in 0..nn {
        if body.nodes[i].pinned {
            continue;
        }
        let a = scale3(forces[i], 1.0 / body.nodes[i].mass);
        body.nodes[i].vel = add3(body.nodes[i].vel, scale3(a, dt));
        body.nodes[i].pos = add3(body.nodes[i].pos, scale3(body.nodes[i].vel, dt));
    }
}

/// Average node height (y-coordinate).
#[allow(dead_code)]
pub fn net_avg_y(body: &NetBody) -> f32 {
    if body.nodes.is_empty() {
        return 0.0;
    }
    let sum: f32 = body.nodes.iter().map(|n| n.pos[1]).sum();
    sum / body.nodes.len() as f32
}

/// Node count.
#[allow(dead_code)]
pub fn net_node_count(body: &NetBody) -> usize {
    body.nodes.len()
}

/// Link count.
#[allow(dead_code)]
pub fn net_link_count(body: &NetBody) -> usize {
    body.links.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_net_body() {
        let body = new_net_body(&[[0.0; 3], [1.0, 0.0, 0.0]], 0.1);
        assert_eq!(net_node_count(&body), 2);
    }

    #[test]
    fn test_add_link() {
        let mut body = new_net_body(&[[0.0; 3], [1.0, 0.0, 0.0]], 0.1);
        net_add_link(&mut body, 0, 1, 100.0);
        assert_eq!(net_link_count(&body), 1);
    }

    #[test]
    fn test_pin_node() {
        let mut body = new_net_body(&[[0.0; 3], [1.0, 0.0, 0.0]], 0.1);
        net_pin(&mut body, 0);
        assert!(body.nodes[0].pinned);
    }

    #[test]
    fn test_step_no_crash() {
        let mut body = new_net_body(&[[0.0, 1.0, 0.0], [0.0, 0.0, 0.0]], 0.1);
        net_add_link(&mut body, 0, 1, 100.0);
        net_step(&mut body, 0.01);
    }

    #[test]
    fn test_pinned_does_not_move() {
        let mut body = new_net_body(&[[0.0, 5.0, 0.0]], 1.0);
        net_pin(&mut body, 0);
        net_step(&mut body, 1.0);
        assert!((body.nodes[0].pos[1] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_gravity_moves_free_node() {
        let mut body = new_net_body(&[[0.0, 10.0, 0.0]], 1.0);
        net_step(&mut body, 0.1);
        assert!(body.nodes[0].pos[1] < 10.0);
    }

    #[test]
    fn test_link_only_tension() {
        let mut body = new_net_body(&[[0.0, 0.0, 0.0], [0.01, 0.0, 0.0]], 1.0);
        net_add_link(&mut body, 0, 1, 1000.0);
        // rest_len ≈ 0.01, positions exactly at rest → no force
        net_step(&mut body, 0.01);
        assert!(net_link_count(&body) == 1);
    }

    #[test]
    fn test_avg_y_flat() {
        let body = new_net_body(&[[0.0; 3], [1.0, 0.0, 0.0]], 0.1);
        assert!((net_avg_y(&body)).abs() < 1e-6);
    }
}
