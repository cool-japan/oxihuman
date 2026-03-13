// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use crate::mesh::MeshBuffers;
use crate::normals::compute_normals;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// SpringParams
// ---------------------------------------------------------------------------

/// Parameters controlling the spring-mass simulation.
pub struct SpringParams {
    /// Spring stiffness k.
    pub stiffness: f32,
    /// Velocity damping factor applied each substep (0..1).
    pub damping: f32,
    /// Vertex mass.
    pub mass: f32,
    /// Gravity vector in world space.
    pub gravity: [f32; 3],
    /// Number of integration substeps per `step()` call.
    pub substeps: usize,
    /// If `true`, boundary vertices are pinned and do not move.
    pub fixed_boundary: bool,
}

impl Default for SpringParams {
    fn default() -> Self {
        Self {
            stiffness: 50.0,
            damping: 0.9,
            mass: 1.0,
            gravity: [0.0, -9.8, 0.0],
            substeps: 4,
            fixed_boundary: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Helper math (inline, no external deps)
// ---------------------------------------------------------------------------

#[inline]
fn vec3_sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn vec3_add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn vec3_scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn vec3_dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn vec3_len_sq(a: [f32; 3]) -> f32 {
    vec3_dot(a, a)
}

#[inline]
fn vec3_len(a: [f32; 3]) -> f32 {
    vec3_len_sq(a).sqrt()
}

// ---------------------------------------------------------------------------
// Build topology helpers
// ---------------------------------------------------------------------------

/// Build springs from mesh edge topology (deduplicated).
/// Each spring is `(vertex_a, vertex_b, rest_length)`.
pub fn build_edge_springs(mesh: &MeshBuffers) -> Vec<(usize, usize, f32)> {
    // Use a HashMap keyed by (min_idx, max_idx) to deduplicate edges.
    let mut edge_map: HashMap<(u32, u32), ()> = HashMap::new();
    let mut springs = Vec::new();

    for tri in mesh.indices.chunks_exact(3) {
        let verts = [tri[0], tri[1], tri[2]];
        for i in 0..3 {
            let a = verts[i];
            let b = verts[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if edge_map.insert(key, ()).is_none() {
                // New edge — compute rest length from initial positions.
                let pa = mesh.positions[key.0 as usize];
                let pb = mesh.positions[key.1 as usize];
                let rest_len = vec3_len(vec3_sub(pa, pb));
                springs.push((key.0 as usize, key.1 as usize, rest_len));
            }
        }
    }

    springs
}

/// Detect boundary vertices: a vertex is on the boundary if at least one of
/// its edges belongs to only one triangle face.
pub fn find_boundary_vertices(mesh: &MeshBuffers) -> Vec<bool> {
    let n = mesh.positions.len();
    // Count how many faces each directed edge appears in.
    let mut edge_face_count: HashMap<(u32, u32), u32> = HashMap::new();

    for tri in mesh.indices.chunks_exact(3) {
        let verts = [tri[0], tri[1], tri[2]];
        for i in 0..3 {
            let a = verts[i];
            let b = verts[(i + 1) % 3];
            // Use undirected key (min, max) for manifold-edge counting.
            let key = if a < b { (a, b) } else { (b, a) };
            *edge_face_count.entry(key).or_insert(0) += 1;
        }
    }

    let mut is_boundary = vec![false; n];
    for ((a, b), count) in &edge_face_count {
        if *count == 1 {
            // This edge is on the boundary — mark both endpoints.
            is_boundary[*a as usize] = true;
            is_boundary[*b as usize] = true;
        }
    }
    is_boundary
}

// ---------------------------------------------------------------------------
// SpringSystem
// ---------------------------------------------------------------------------

/// A spring-mass system attached to mesh vertices for soft-body simulation.
pub struct SpringSystem {
    /// Rest positions (used to reset).
    pub rest_positions: Vec<[f32; 3]>,
    /// Current positions.
    pub positions: Vec<[f32; 3]>,
    /// Current velocities.
    pub velocities: Vec<[f32; 3]>,
    /// Springs: (vertex_a, vertex_b, rest_length).
    pub springs: Vec<(usize, usize, f32)>,
    /// Fixed (pinned) vertices that do not move.
    pub fixed: Vec<bool>,
    /// Simulation parameters.
    pub params: SpringParams,
}

impl SpringSystem {
    /// Construct from a mesh and simulation parameters.
    pub fn from_mesh(mesh: &MeshBuffers, params: SpringParams) -> Self {
        let n = mesh.positions.len();
        let rest_positions = mesh.positions.clone();
        let positions = mesh.positions.clone();
        let velocities = vec![[0.0f32; 3]; n];
        let springs = build_edge_springs(mesh);

        let fixed = if params.fixed_boundary {
            find_boundary_vertices(mesh)
        } else {
            vec![false; n]
        };

        Self {
            rest_positions,
            positions,
            velocities,
            springs,
            fixed,
            params,
        }
    }

    /// Number of vertices in the system.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Number of springs in the system.
    pub fn spring_count(&self) -> usize {
        self.springs.len()
    }

    /// Total kinetic energy: sum of 0.5 * mass * |v|^2 over all vertices.
    pub fn kinetic_energy(&self) -> f32 {
        let half_m = 0.5 * self.params.mass;
        self.velocities
            .iter()
            .map(|v| half_m * vec3_len_sq(*v))
            .sum()
    }

    /// Returns `true` when the kinetic energy is below `threshold`.
    pub fn is_settled(&self, threshold: f32) -> bool {
        self.kinetic_energy() < threshold
    }

    /// Pin or unpin a single vertex.
    pub fn set_fixed(&mut self, vertex: usize, fixed: bool) {
        if vertex < self.fixed.len() {
            self.fixed[vertex] = fixed;
        }
    }

    /// Apply an instantaneous velocity impulse to a vertex.
    pub fn apply_impulse(&mut self, vertex: usize, force: [f32; 3]) {
        if vertex < self.velocities.len() && !self.fixed[vertex] {
            self.velocities[vertex] = vec3_add(self.velocities[vertex], force);
        }
    }

    /// Accumulate gravity into all non-fixed vertex velocities.
    pub fn apply_gravity_impulse(&mut self, dt: f32) {
        let g = self.params.gravity;
        for (i, vel) in self.velocities.iter_mut().enumerate() {
            if !self.fixed[i] {
                *vel = vec3_add(*vel, vec3_scale(g, dt));
            }
        }
    }

    /// Advance the simulation by `dt` seconds (uses `params.substeps` substeps).
    pub fn step(&mut self, dt: f32) {
        let sub_dt = dt / self.params.substeps as f32;
        for _ in 0..self.params.substeps {
            self.substep(sub_dt);
        }
    }

    /// Advance the simulation by `n` steps of `dt` seconds each.
    pub fn step_n(&mut self, dt: f32, n: usize) {
        for _ in 0..n {
            self.step(dt);
        }
    }

    /// Return all vertices to their rest positions and zero all velocities.
    pub fn reset(&mut self) {
        self.positions = self.rest_positions.clone();
        let n = self.positions.len();
        self.velocities = vec![[0.0f32; 3]; n];
    }

    /// Build a new `MeshBuffers` from the template, with positions replaced by
    /// the current simulated positions, and normals recomputed.
    pub fn to_mesh(&self, template: &MeshBuffers) -> MeshBuffers {
        let mut out = template.clone();
        out.positions = self.positions.clone();
        compute_normals(&mut out);
        out
    }

    // -----------------------------------------------------------------------
    // Internal integration step
    // -----------------------------------------------------------------------

    fn substep(&mut self, dt: f32) {
        let n = self.positions.len();
        let mut forces = vec![[0.0f32; 3]; n];

        // Spring forces.
        let k = self.params.stiffness;
        for &(a, b, rest_len) in &self.springs {
            let pa = self.positions[a];
            let pb = self.positions[b];
            let diff = vec3_sub(pb, pa);
            let cur_len = vec3_len(diff);
            if cur_len < 1e-10 {
                continue;
            }
            let unit = vec3_scale(diff, 1.0 / cur_len);
            let stretch = cur_len - rest_len;
            let f = vec3_scale(unit, k * stretch);
            forces[a] = vec3_add(forces[a], f);
            forces[b] = vec3_sub(forces[b], f);
        }

        let g = self.params.gravity;
        let m = self.params.mass;
        let damping = self.params.damping;

        // Integrate each non-fixed vertex: semi-implicit Euler.
        for (i, (pos, vel)) in self
            .positions
            .iter_mut()
            .zip(self.velocities.iter_mut())
            .enumerate()
        {
            if self.fixed[i] {
                continue;
            }
            let acc = [
                forces[i][0] / m + g[0],
                forces[i][1] / m + g[1],
                forces[i][2] / m + g[2],
            ];
            let new_vel = vec3_add(*vel, vec3_scale(acc, dt));
            // Apply damping.
            let new_vel = vec3_scale(new_vel, damping);
            *vel = new_vel;
            *pos = vec3_add(*pos, vec3_scale(new_vel, dt));
        }
    }
}

// ---------------------------------------------------------------------------
// High-level jiggle deform
// ---------------------------------------------------------------------------

/// Apply an impulse to a single vertex and simulate until the mesh settles
/// (kinetic energy < 0.001) or 1000 steps have elapsed, then return the
/// deformed mesh.
pub fn jiggle_deform(
    mesh: &MeshBuffers,
    impulse_vertex: usize,
    impulse: [f32; 3],
    params: &SpringParams,
) -> MeshBuffers {
    // Build a clone of params — SpringParams does not implement Clone so we
    // reconstruct manually.
    let p = SpringParams {
        stiffness: params.stiffness,
        damping: params.damping,
        mass: params.mass,
        gravity: params.gravity,
        substeps: params.substeps,
        fixed_boundary: params.fixed_boundary,
    };

    let mut system = SpringSystem::from_mesh(mesh, p);
    system.apply_impulse(impulse_vertex, impulse);

    const MAX_STEPS: usize = 1000;
    const DT: f32 = 1.0 / 60.0;

    for _ in 0..MAX_STEPS {
        if system.is_settled(0.001) {
            break;
        }
        system.step(DT);
    }

    system.to_mesh(mesh)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_morph::engine::MeshBuffers as MB;

    /// Build a simple 2-triangle mesh (4 vertices, 2 faces).
    ///
    /// ```
    ///  3---2
    ///  |  /|
    ///  | / |
    ///  |/  |
    ///  0---1
    /// ```
    fn two_tri_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0], // 0
                [1.0, 0.0, 0.0], // 1
                [1.0, 1.0, 0.0], // 2
                [0.0, 1.0, 0.0], // 3
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0]; 4],
            // Two triangles sharing edge 1-3.
            indices: vec![0, 1, 3, 1, 2, 3],
            has_suit: false,
        })
    }

    /// Default params but with gravity disabled so tests are deterministic.
    fn no_gravity_params() -> SpringParams {
        SpringParams {
            gravity: [0.0, 0.0, 0.0],
            fixed_boundary: false,
            ..Default::default()
        }
    }

    // -----------------------------------------------------------------------

    #[test]
    fn test_spring_system_from_mesh() {
        let mesh = two_tri_mesh();
        let sys = SpringSystem::from_mesh(&mesh, SpringParams::default());
        assert_eq!(sys.rest_positions.len(), 4);
        assert_eq!(sys.positions.len(), 4);
        assert_eq!(sys.velocities.len(), 4);
    }

    #[test]
    fn test_vertex_count() {
        let mesh = two_tri_mesh();
        let sys = SpringSystem::from_mesh(&mesh, SpringParams::default());
        assert_eq!(sys.vertex_count(), 4);
    }

    #[test]
    fn test_spring_count() {
        let mesh = two_tri_mesh();
        let sys = SpringSystem::from_mesh(&mesh, SpringParams::default());
        // 2 triangles → up to 5 unique edges (0-1, 1-3, 0-3, 1-2, 2-3).
        assert!(sys.spring_count() >= 4);
        assert!(sys.spring_count() <= 5);
    }

    #[test]
    fn test_reset() {
        let mesh = two_tri_mesh();
        let mut sys = SpringSystem::from_mesh(&mesh, no_gravity_params());
        sys.apply_impulse(0, [1.0, 0.0, 0.0]);
        sys.step(0.1);
        sys.reset();
        for i in 0..sys.vertex_count() {
            for j in 0..3 {
                assert!(
                    (sys.positions[i][j] - sys.rest_positions[i][j]).abs() < 1e-6,
                    "position not reset at vertex {i}"
                );
                assert!(
                    sys.velocities[i][j].abs() < 1e-6,
                    "velocity not zeroed at vertex {i}"
                );
            }
        }
    }

    #[test]
    fn test_step_moves_unfixed_vertices() {
        let mesh = two_tri_mesh();
        let params = SpringParams {
            fixed_boundary: false,
            gravity: [0.0, -9.8, 0.0],
            damping: 1.0, // no damping so movement is clear
            substeps: 1,
            ..Default::default()
        };
        let mut sys = SpringSystem::from_mesh(&mesh, params);
        let orig = sys.positions.clone();
        sys.step(0.05);
        // With gravity, at least some vertices should have moved.
        let moved = sys
            .positions
            .iter()
            .zip(orig.iter())
            .any(|(a, b)| vec3_len(vec3_sub(*a, *b)) > 1e-6);
        assert!(moved, "no vertices moved after step with gravity");
    }

    #[test]
    fn test_fixed_vertex_stays_fixed() {
        let mesh = two_tri_mesh();
        let params = SpringParams {
            fixed_boundary: false,
            gravity: [0.0, -9.8, 0.0],
            ..Default::default()
        };
        let mut sys = SpringSystem::from_mesh(&mesh, params);
        // Manually pin vertex 0.
        sys.set_fixed(0, true);
        let orig0 = sys.positions[0];
        sys.step_n(0.016, 20);
        // Vertex 0 must not have moved.
        for (j, &orig) in orig0.iter().enumerate() {
            assert!(
                (sys.positions[0][j] - orig).abs() < 1e-6,
                "fixed vertex moved at component {j}"
            );
        }
    }

    #[test]
    fn test_apply_impulse() {
        let mesh = two_tri_mesh();
        let params = no_gravity_params();
        let mut sys = SpringSystem::from_mesh(&mesh, params);
        // Vertex 0 is not fixed (fixed_boundary=false).
        sys.apply_impulse(0, [5.0, 0.0, 0.0]);
        assert!((sys.velocities[0][0] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_kinetic_energy() {
        let mesh = two_tri_mesh();
        let params = no_gravity_params();
        let mut sys = SpringSystem::from_mesh(&mesh, params);
        // At rest, KE should be zero.
        assert!(sys.kinetic_energy() < 1e-10);
        sys.apply_impulse(0, [1.0, 0.0, 0.0]);
        assert!(sys.kinetic_energy() > 0.0);
    }

    #[test]
    fn test_is_settled() {
        let mesh = two_tri_mesh();
        let params = no_gravity_params();
        let mut sys = SpringSystem::from_mesh(&mesh, params);
        // At rest, settled with any positive threshold.
        assert!(sys.is_settled(1e-3));
        sys.apply_impulse(0, [100.0, 0.0, 0.0]);
        // Large impulse → not settled.
        assert!(!sys.is_settled(1e-3));
    }

    #[test]
    fn test_build_edge_springs() {
        let mesh = two_tri_mesh();
        let springs = build_edge_springs(&mesh);
        // Verify no duplicate edges.
        let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
        for &(a, b, _) in &springs {
            let key = (a.min(b), a.max(b));
            assert!(seen.insert(key), "duplicate edge ({a},{b})");
        }
        // All rest lengths should be positive.
        for &(_, _, len) in &springs {
            assert!(len > 0.0, "non-positive rest length");
        }
    }

    #[test]
    fn test_find_boundary_vertices() {
        let mesh = two_tri_mesh();
        let boundary = find_boundary_vertices(&mesh);
        assert_eq!(boundary.len(), 4);
        // All 4 vertices are on the boundary of the 2-triangle mesh.
        for (i, &b) in boundary.iter().enumerate() {
            assert!(b, "vertex {i} should be boundary");
        }
    }

    #[test]
    fn test_to_mesh() {
        let mesh = two_tri_mesh();
        let mut sys = SpringSystem::from_mesh(
            &mesh,
            SpringParams {
                fixed_boundary: false,
                gravity: [0.0, -9.8, 0.0],
                ..Default::default()
            },
        );
        sys.step(0.1);
        let out = sys.to_mesh(&mesh);
        // Output mesh should have same topology.
        assert_eq!(out.indices, mesh.indices);
        assert_eq!(out.positions.len(), mesh.positions.len());
        // Normals must be recomputed (not all-zero).
        let all_zero = out
            .normals
            .iter()
            .all(|n| n[0].abs() < 1e-10 && n[1].abs() < 1e-10 && n[2].abs() < 1e-10);
        assert!(!all_zero, "normals should be non-zero after recompute");
    }

    #[test]
    fn test_jiggle_deform() {
        let mesh = two_tri_mesh();
        let params = SpringParams {
            fixed_boundary: false,
            gravity: [0.0, 0.0, 0.0],
            damping: 0.5,
            stiffness: 50.0,
            ..Default::default()
        };
        let result = jiggle_deform(&mesh, 0, [0.5, 0.0, 0.0], &params);
        // Result must have same topology and vertex count.
        assert_eq!(result.indices, mesh.indices);
        assert_eq!(result.positions.len(), mesh.positions.len());
    }
}
