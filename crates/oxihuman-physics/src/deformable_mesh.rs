// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Deformable mesh: particle-based mesh deformation with rest-shape constraints.

/// A single mesh particle.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeformParticle {
    pub position: [f32; 3],
    pub rest_position: [f32; 3],
    pub velocity: [f32; 3],
    pub mass: f32,
    pub fixed: bool,
}

/// Deformable mesh.
#[derive(Debug)]
#[allow(dead_code)]
pub struct DeformableMesh {
    pub particles: Vec<DeformParticle>,
    pub edges: Vec<(usize, usize)>,
    pub rest_lengths: Vec<f32>,
    pub stiffness: f32,
}

/// Create a deformable mesh.
#[allow(dead_code)]
pub fn new_deformable_mesh(stiffness: f32) -> DeformableMesh {
    DeformableMesh {
        particles: Vec::new(),
        edges: Vec::new(),
        rest_lengths: Vec::new(),
        stiffness,
    }
}

/// Add a particle; returns its index.
#[allow(dead_code)]
pub fn dm_add_particle(mesh: &mut DeformableMesh, pos: [f32; 3], mass: f32, fixed: bool) -> usize {
    let idx = mesh.particles.len();
    mesh.particles.push(DeformParticle {
        position: pos,
        rest_position: pos,
        velocity: [0.0; 3],
        mass,
        fixed,
    });
    idx
}

/// Add an edge between particles; records rest length.
#[allow(dead_code)]
pub fn dm_add_edge(mesh: &mut DeformableMesh, i: usize, j: usize) {
    let pi = mesh.particles[i].position;
    let pj = mesh.particles[j].position;
    let d = [pj[0] - pi[0], pj[1] - pi[1], pj[2] - pi[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    mesh.edges.push((i, j));
    mesh.rest_lengths.push(len);
}

/// Compute spring forces for all edges into an accumulator.
#[allow(dead_code)]
pub fn dm_spring_forces(mesh: &DeformableMesh) -> Vec<[f32; 3]> {
    let mut forces = vec![[0.0_f32; 3]; mesh.particles.len()];
    for (edge_idx, &(i, j)) in mesh.edges.iter().enumerate() {
        let pi = mesh.particles[i].position;
        let pj = mesh.particles[j].position;
        let d = [pj[0] - pi[0], pj[1] - pi[1], pj[2] - pi[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1e-12 {
            continue;
        }
        let rest = mesh.rest_lengths[edge_idx];
        let stretch = len - rest;
        let mag = mesh.stiffness * stretch / len;
        for k in 0..3 {
            forces[i][k] += mag * d[k];
            forces[j][k] -= mag * d[k];
        }
    }
    forces
}

/// Integrate all non-fixed particles.
#[allow(dead_code)]
pub fn dm_integrate(mesh: &mut DeformableMesh, forces: &[[f32; 3]], dt: f32) {
    for (p, f) in mesh.particles.iter_mut().zip(forces.iter()) {
        if p.fixed || p.mass < 1e-12 {
            continue;
        }
        let inv_m = 1.0 / p.mass;
        p.velocity[0] += f[0] * inv_m * dt;
        p.velocity[1] += f[1] * inv_m * dt;
        p.velocity[2] += f[2] * inv_m * dt;
        p.position[0] += p.velocity[0] * dt;
        p.position[1] += p.velocity[1] * dt;
        p.position[2] += p.velocity[2] * dt;
    }
}

/// Total kinetic energy.
#[allow(dead_code)]
pub fn dm_kinetic_energy(mesh: &DeformableMesh) -> f32 {
    mesh.particles
        .iter()
        .map(|p| {
            let v2 = p.velocity[0] * p.velocity[0]
                + p.velocity[1] * p.velocity[1]
                + p.velocity[2] * p.velocity[2];
            0.5 * p.mass * v2
        })
        .sum()
}

/// Total elastic potential energy.
#[allow(dead_code)]
pub fn dm_potential_energy(mesh: &DeformableMesh) -> f32 {
    mesh.edges
        .iter()
        .zip(mesh.rest_lengths.iter())
        .map(|(&(i, j), &rest)| {
            let pi = mesh.particles[i].position;
            let pj = mesh.particles[j].position;
            let d = [pj[0] - pi[0], pj[1] - pi[1], pj[2] - pi[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let stretch = len - rest;
            0.5 * mesh.stiffness * stretch * stretch
        })
        .sum()
}

/// Particle count.
#[allow(dead_code)]
pub fn dm_particle_count(mesh: &DeformableMesh) -> usize {
    mesh.particles.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_particles() {
        let mut mesh = new_deformable_mesh(100.0);
        dm_add_particle(&mut mesh, [0.0, 0.0, 0.0], 1.0, false);
        dm_add_particle(&mut mesh, [1.0, 0.0, 0.0], 1.0, false);
        assert_eq!(dm_particle_count(&mesh), 2);
    }

    #[test]
    fn test_add_edge_rest_length() {
        let mut mesh = new_deformable_mesh(100.0);
        dm_add_particle(&mut mesh, [0.0, 0.0, 0.0], 1.0, false);
        dm_add_particle(&mut mesh, [3.0, 4.0, 0.0], 1.0, false);
        dm_add_edge(&mut mesh, 0, 1);
        assert!((mesh.rest_lengths[0] - 5.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_spring_force_compression() {
        let mut mesh = new_deformable_mesh(100.0);
        dm_add_particle(&mut mesh, [0.0, 0.0, 0.0], 1.0, false);
        dm_add_particle(&mut mesh, [0.5, 0.0, 0.0], 1.0, false); // rest=0.5, no stretch
        dm_add_edge(&mut mesh, 0, 1);
        let forces = dm_spring_forces(&mesh);
        // No stretch → zero force
        assert!(forces[0][0].abs() < 1e-5);
    }

    #[test]
    fn test_spring_force_stretch() {
        let mut mesh = new_deformable_mesh(100.0);
        dm_add_particle(&mut mesh, [0.0, 0.0, 0.0], 1.0, false);
        dm_add_particle(&mut mesh, [2.0, 0.0, 0.0], 1.0, false); // rest=2 at creation
        dm_add_edge(&mut mesh, 0, 1);
        // Now stretch the second particle
        mesh.particles[1].position = [3.0, 0.0, 0.0];
        let forces = dm_spring_forces(&mesh);
        assert!(forces[0][0] > 0.0); // i pulled toward j
    }

    #[test]
    fn test_integrate_moves_particle() {
        let mut mesh = new_deformable_mesh(100.0);
        dm_add_particle(&mut mesh, [0.0, 0.0, 0.0], 1.0, false);
        let forces = vec![[10.0_f32, 0.0, 0.0]];
        dm_integrate(&mut mesh, &forces, 0.1);
        assert!(mesh.particles[0].position[0] > 0.0);
    }

    #[test]
    fn test_fixed_not_moved() {
        let mut mesh = new_deformable_mesh(100.0);
        dm_add_particle(&mut mesh, [0.0, 0.0, 0.0], 1.0, true);
        let forces = vec![[100.0_f32, 0.0, 0.0]];
        dm_integrate(&mut mesh, &forces, 1.0);
        assert_eq!(mesh.particles[0].position[0], 0.0);
    }

    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let mut mesh = new_deformable_mesh(10.0);
        dm_add_particle(&mut mesh, [0.0, 0.0, 0.0], 1.0, false);
        assert!((dm_kinetic_energy(&mesh)).abs() < 1e-10);
    }

    #[test]
    fn test_potential_energy_zero_at_rest() {
        let mut mesh = new_deformable_mesh(100.0);
        dm_add_particle(&mut mesh, [0.0, 0.0, 0.0], 1.0, false);
        dm_add_particle(&mut mesh, [1.0, 0.0, 0.0], 1.0, false);
        dm_add_edge(&mut mesh, 0, 1);
        assert!(dm_potential_energy(&mesh).abs() < 1e-10);
    }

    #[test]
    fn test_potential_energy_positive_when_stretched() {
        let mut mesh = new_deformable_mesh(100.0);
        dm_add_particle(&mut mesh, [0.0, 0.0, 0.0], 1.0, false);
        dm_add_particle(&mut mesh, [1.0, 0.0, 0.0], 1.0, false);
        dm_add_edge(&mut mesh, 0, 1);
        mesh.particles[1].position = [2.0, 0.0, 0.0];
        assert!(dm_potential_energy(&mesh) > 0.0);
    }
}
