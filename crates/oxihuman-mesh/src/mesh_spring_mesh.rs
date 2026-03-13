// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Spring-based mesh relaxation system.

use std::f32::consts::TAU;

/// A spring connecting two vertices.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Spring {
    pub a: usize,
    pub b: usize,
    pub rest_length: f32,
    pub stiffness: f32,
}

/// Parameters for the spring simulation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SpringMeshParams {
    pub damping: f32,
    pub dt: f32,
    pub iterations: usize,
    pub gravity: [f32; 3],
}

impl Default for SpringMeshParams {
    fn default() -> Self {
        Self {
            damping: 0.9,
            dt: 0.016,
            iterations: 10,
            gravity: [0.0, -9.8, 0.0],
        }
    }
}

/// Simulate one step of spring forces.
#[allow(dead_code)]
pub fn spring_step(
    positions: &mut [[f32; 3]],
    velocities: &mut [[f32; 3]],
    springs: &[Spring],
    params: &SpringMeshParams,
    pinned: &[bool],
) {
    let n = positions.len();
    let mut forces = vec![[0.0_f32; 3]; n];
    for s in springs {
        let d = [
            positions[s.b][0] - positions[s.a][0],
            positions[s.b][1] - positions[s.a][1],
            positions[s.b][2] - positions[s.a][2],
        ];
        let len = (d[0].powi(2) + d[1].powi(2) + d[2].powi(2))
            .sqrt()
            .max(1e-9);
        let f = s.stiffness * (len - s.rest_length);
        for k in 0..3 {
            let fv = f * d[k] / len;
            forces[s.a][k] += fv;
            forces[s.b][k] -= fv;
        }
    }
    for i in 0..n {
        if i < pinned.len() && pinned[i] {
            continue;
        }
        for k in 0..3 {
            forces[i][k] += params.gravity[k];
            velocities[i][k] = (velocities[i][k] + forces[i][k] * params.dt) * params.damping;
            positions[i][k] += velocities[i][k] * params.dt;
        }
    }
}

/// Build springs from a triangle mesh.
#[allow(dead_code)]
pub fn springs_from_mesh(positions: &[[f32; 3]], indices: &[u32], stiffness: f32) -> Vec<Spring> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut springs = Vec::new();
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        for &(u, v) in &[
            (a.min(b), a.max(b)),
            (b.min(c), b.max(c)),
            (a.min(c), a.max(c)),
        ] {
            if seen.insert((u, v)) {
                let d = dist3(positions[u], positions[v]);
                springs.push(Spring {
                    a: u,
                    b: v,
                    rest_length: d,
                    stiffness,
                });
            }
        }
    }
    springs
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Compute total elastic energy.
#[allow(dead_code)]
pub fn spring_energy(positions: &[[f32; 3]], springs: &[Spring]) -> f32 {
    springs
        .iter()
        .map(|s| {
            let d = dist3(positions[s.a], positions[s.b]);
            0.5 * s.stiffness * (d - s.rest_length).powi(2)
        })
        .sum()
}

/// Run full simulation for `params.iterations` steps.
#[allow(dead_code)]
pub fn simulate_spring_mesh(
    positions: &mut [[f32; 3]],
    springs: &[Spring],
    params: &SpringMeshParams,
    pinned: &[bool],
) {
    let mut velocities = vec![[0.0_f32; 3]; positions.len()];
    let _ = TAU;
    for _ in 0..params.iterations {
        spring_step(positions, &mut velocities, springs, params, pinned);
    }
}

/// Maximum velocity magnitude after simulation (used for convergence check).
#[allow(dead_code)]
pub fn max_velocity(velocities: &[[f32; 3]]) -> f32 {
    velocities
        .iter()
        .map(|v| (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt())
        .fold(0.0_f32, f32::max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_pos() -> Vec<[f32; 3]> {
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]
    }

    #[test]
    fn spring_energy_at_rest() {
        let pos = two_pos();
        let s = Spring {
            a: 0,
            b: 1,
            rest_length: 1.0,
            stiffness: 1.0,
        };
        assert!(spring_energy(&pos, &[s]).abs() < 1e-6);
    }

    #[test]
    fn spring_energy_stretched() {
        let pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let s = Spring {
            a: 0,
            b: 1,
            rest_length: 1.0,
            stiffness: 1.0,
        };
        assert!(spring_energy(&pos, &[s]) > 0.0);
    }

    #[test]
    fn spring_step_moves_vertices() {
        let mut pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let mut vel = vec![[0.0_f32; 3]; 2];
        let s = vec![Spring {
            a: 0,
            b: 1,
            rest_length: 1.0,
            stiffness: 100.0,
        }];
        let params = SpringMeshParams {
            gravity: [0.0; 3],
            ..Default::default()
        };
        spring_step(&mut pos, &mut vel, &s, &params, &[false, false]);
        assert!((pos[0][0] - pos[1][0]).abs() < 2.0);
    }

    #[test]
    fn springs_from_mesh_count() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let springs = springs_from_mesh(&pos, &idx, 1.0);
        assert_eq!(springs.len(), 3);
    }

    #[test]
    fn simulate_does_not_panic() {
        let mut pos = two_pos();
        let springs = springs_from_mesh(&pos, &[0u32, 1, 0], 1.0);
        let params = SpringMeshParams::default();
        simulate_spring_mesh(&mut pos, &springs, &params, &[]);
    }

    #[test]
    fn pinned_vertex_unchanged() {
        let mut pos = vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let mut vel = vec![[0.0_f32; 3]; 2];
        let s = vec![Spring {
            a: 0,
            b: 1,
            rest_length: 1.0,
            stiffness: 100.0,
        }];
        let params = SpringMeshParams {
            gravity: [0.0; 3],
            ..Default::default()
        };
        spring_step(&mut pos, &mut vel, &s, &params, &[true, false]);
        assert!((pos[0][0] - 0.0).abs() < 1e-9);
    }

    #[test]
    fn max_velocity_zero_at_rest() {
        let vel = vec![[0.0_f32; 3]; 3];
        assert_eq!(max_velocity(&vel), 0.0);
    }

    #[test]
    fn default_params() {
        let p = SpringMeshParams::default();
        assert!(p.dt > 0.0);
        assert!(p.damping > 0.0);
    }

    #[test]
    fn spring_struct_fields() {
        let s = Spring {
            a: 1,
            b: 2,
            rest_length: 0.5,
            stiffness: 2.0,
        };
        assert_eq!(s.a, 1);
        assert_eq!(s.b, 2);
    }

    #[test]
    fn no_springs_energy_zero() {
        let pos = two_pos();
        assert_eq!(spring_energy(&pos, &[]), 0.0);
    }
}
