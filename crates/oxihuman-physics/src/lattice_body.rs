// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Regular 3D lattice of springs connecting grid nodes.

#![allow(dead_code)]

/// A lattice node (mass point).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LatticeNode {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub pinned: bool,
}

/// A spring connecting two nodes in the lattice.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LatticeSpring {
    pub a: usize,
    pub b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
}

/// A 3D lattice of mass-spring nodes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LatticeBody {
    pub nodes: Vec<LatticeNode>,
    pub springs: Vec<LatticeSpring>,
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
}

/// Create a regular 3D lattice.
#[allow(dead_code)]
pub fn new_lattice_body(
    nx: usize,
    ny: usize,
    nz: usize,
    spacing: f32,
    node_mass: f32,
    stiffness: f32,
    damping: f32,
) -> LatticeBody {
    let mut nodes = Vec::with_capacity(nx * ny * nz);
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                nodes.push(LatticeNode {
                    position: [
                        ix as f32 * spacing,
                        iy as f32 * spacing,
                        iz as f32 * spacing,
                    ],
                    velocity: [0.0; 3],
                    mass: node_mass,
                    pinned: false,
                });
            }
        }
    }
    let mut springs = Vec::new();
    let rest = spacing;
    let node_idx = |ix: usize, iy: usize, iz: usize| iz * ny * nx + iy * nx + ix;
    // Connect neighbors in each axis direction
    for iz in 0..nz {
        for iy in 0..ny {
            for ix in 0..nx {
                let a = node_idx(ix, iy, iz);
                if ix + 1 < nx {
                    springs.push(LatticeSpring {
                        a,
                        b: node_idx(ix + 1, iy, iz),
                        rest_length: rest,
                        stiffness,
                        damping,
                    });
                }
                if iy + 1 < ny {
                    springs.push(LatticeSpring {
                        a,
                        b: node_idx(ix, iy + 1, iz),
                        rest_length: rest,
                        stiffness,
                        damping,
                    });
                }
                if iz + 1 < nz {
                    springs.push(LatticeSpring {
                        a,
                        b: node_idx(ix, iy, iz + 1),
                        rest_length: rest,
                        stiffness,
                        damping,
                    });
                }
            }
        }
    }
    LatticeBody {
        nodes,
        springs,
        nx,
        ny,
        nz,
    }
}

/// Number of nodes.
#[allow(dead_code)]
pub fn lattice_node_count(b: &LatticeBody) -> usize {
    b.nodes.len()
}

/// Number of springs.
#[allow(dead_code)]
pub fn lattice_spring_count(b: &LatticeBody) -> usize {
    b.springs.len()
}

/// Compute the spring force between two nodes.
fn spring_force(
    pa: [f32; 3],
    va: [f32; 3],
    pb: [f32; 3],
    vb: [f32; 3],
    rest: f32,
    stiffness: f32,
    damping: f32,
) -> [f32; 3] {
    let d = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < f32::EPSILON {
        return [0.0; 3];
    }
    let n = [d[0] / len, d[1] / len, d[2] / len];
    let stretch = len - rest;
    let rel_vel = [vb[0] - va[0], vb[1] - va[1], vb[2] - va[2]];
    let rv_n = rel_vel[0] * n[0] + rel_vel[1] * n[1] + rel_vel[2] * n[2];
    let f_mag = stiffness * stretch + damping * rv_n;
    [f_mag * n[0], f_mag * n[1], f_mag * n[2]]
}

/// Step the lattice simulation.
#[allow(dead_code)]
pub fn lattice_step(b: &mut LatticeBody, gravity: [f32; 3], dt: f32) {
    let n = b.nodes.len();
    let mut forces = vec![[0.0f32; 3]; n];
    // Apply spring forces
    for s in &b.springs {
        let pa = b.nodes[s.a].position;
        let va = b.nodes[s.a].velocity;
        let pb = b.nodes[s.b].position;
        let vb = b.nodes[s.b].velocity;
        let f = spring_force(pa, va, pb, vb, s.rest_length, s.stiffness, s.damping);
        forces[s.a][0] += f[0];
        forces[s.a][1] += f[1];
        forces[s.a][2] += f[2];
        forces[s.b][0] -= f[0];
        forces[s.b][1] -= f[1];
        forces[s.b][2] -= f[2];
    }
    // Integrate
    for (i, node) in b.nodes.iter_mut().enumerate() {
        if node.pinned {
            continue;
        }
        let m = node.mass;
        if m < f32::EPSILON {
            continue;
        }
        node.velocity[0] += (forces[i][0] / m + gravity[0]) * dt;
        node.velocity[1] += (forces[i][1] / m + gravity[1]) * dt;
        node.velocity[2] += (forces[i][2] / m + gravity[2]) * dt;
        node.position[0] += node.velocity[0] * dt;
        node.position[1] += node.velocity[1] * dt;
        node.position[2] += node.velocity[2] * dt;
    }
}

/// Pin a node (fix its position).
#[allow(dead_code)]
pub fn lattice_pin(b: &mut LatticeBody, idx: usize) {
    if idx < b.nodes.len() {
        b.nodes[idx].pinned = true;
    }
}

/// Unpin a node.
#[allow(dead_code)]
pub fn lattice_unpin(b: &mut LatticeBody, idx: usize) {
    if idx < b.nodes.len() {
        b.nodes[idx].pinned = false;
    }
}

/// Total kinetic energy of the lattice.
#[allow(dead_code)]
pub fn lattice_kinetic_energy(b: &LatticeBody) -> f32 {
    b.nodes
        .iter()
        .map(|n| {
            let v2 = n.velocity[0] * n.velocity[0]
                + n.velocity[1] * n.velocity[1]
                + n.velocity[2] * n.velocity[2];
            0.5 * n.mass * v2
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_lattice() -> LatticeBody {
        new_lattice_body(3, 3, 3, 1.0, 1.0, 100.0, 5.0)
    }

    #[test]
    fn node_count_correct() {
        let b = small_lattice();
        assert_eq!(lattice_node_count(&b), 27);
    }

    #[test]
    fn spring_count_positive() {
        let b = small_lattice();
        assert!(lattice_spring_count(&b) > 0);
    }

    #[test]
    fn kinetic_energy_zero_at_rest() {
        let b = small_lattice();
        assert_eq!(lattice_kinetic_energy(&b), 0.0);
    }

    #[test]
    fn step_gravity_moves_nodes() {
        let mut b = small_lattice();
        let before_y = b.nodes[0].position[1];
        lattice_step(&mut b, [0.0, -9.81, 0.0], 0.016);
        assert!(b.nodes[0].position[1] < before_y);
    }

    #[test]
    fn pinned_node_does_not_move() {
        let mut b = small_lattice();
        lattice_pin(&mut b, 0);
        let before = b.nodes[0].position;
        lattice_step(&mut b, [0.0, -9.81, 0.0], 0.016);
        assert_eq!(b.nodes[0].position, before);
    }

    #[test]
    fn unpin_allows_movement() {
        let mut b = small_lattice();
        lattice_pin(&mut b, 0);
        lattice_unpin(&mut b, 0);
        let before_y = b.nodes[0].position[1];
        lattice_step(&mut b, [0.0, -9.81, 0.0], 0.016);
        assert!(b.nodes[0].position[1] < before_y);
    }

    #[test]
    fn kinetic_energy_increases_after_gravity() {
        let mut b = small_lattice();
        lattice_step(&mut b, [0.0, -9.81, 0.0], 0.1);
        assert!(lattice_kinetic_energy(&b) > 0.0);
    }

    #[test]
    fn spring_count_for_2x2x2() {
        let b = new_lattice_body(2, 2, 2, 1.0, 1.0, 100.0, 5.0);
        // Each node connects: 3 directions * 8 nodes but avoid double = 12 springs
        assert_eq!(lattice_spring_count(&b), 12);
    }

    #[test]
    fn positions_spaced_correctly() {
        let b = new_lattice_body(2, 1, 1, 2.0, 1.0, 100.0, 5.0);
        assert!((b.nodes[1].position[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn new_lattice_at_rest() {
        let b = small_lattice();
        for n in &b.nodes {
            assert_eq!(n.velocity, [0.0; 3]);
        }
    }
}
