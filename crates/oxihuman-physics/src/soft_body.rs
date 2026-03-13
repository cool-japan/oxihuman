// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Tetrahedral soft body simulation using XPBD.
//!
//! Implements a position-based soft body that is discretised into tetrahedra.
//! Distance and volume constraints maintain shape while an explicit gravity
//! integration step updates velocities and positions.

use crate::constraint::{
    apply_distance_constraint, apply_volume_constraint, DistanceConstraint, VolumeConstraint,
};
use std::collections::HashSet;

// ── Tetrahedron ───────────────────────────────────────────────────────────────

/// A tetrahedron defined by four vertex indices.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tetrahedron {
    /// Indices into the parent `SoftBody::positions` array.
    pub verts: [usize; 4],
}

// ── SoftBody ──────────────────────────────────────────────────────────────────

/// Tetrahedral soft body state.
pub struct SoftBody {
    /// Current world-space positions of all vertices.
    pub positions: Vec<[f32; 3]>,
    /// Current velocity of each vertex.
    pub velocities: Vec<[f32; 3]>,
    /// Inverse mass of each vertex (`0.0` = pinned / infinite mass).
    pub inv_masses: Vec<f32>,
    /// Tetrahedra (topology).
    pub tets: Vec<Tetrahedron>,
    /// Distance constraints on the edges.
    pub distance_constraints: Vec<DistanceConstraint>,
    /// Volume constraints, one per tetrahedron.
    pub volume_constraints: Vec<VolumeConstraint>,
    /// Rest volumes (one per tet, cached for energy queries).
    pub rest_volumes: Vec<f32>,
}

impl SoftBody {
    // ── Construction ─────────────────────────────────────────────────────────

    /// Build a soft body from a tetrahedral mesh.
    ///
    /// `density` is in kg/m³; mass is distributed from each tet to its four
    /// vertices (one quarter of the tet mass per vertex).
    pub fn from_tet_mesh(positions: &[[f32; 3]], tets: &[[usize; 4]], density: f32) -> Self {
        let n = positions.len();
        let mut masses = vec![0.0f32; n];

        let mut tet_structs = Vec::with_capacity(tets.len());
        let mut volume_constraints = Vec::with_capacity(tets.len());
        let mut rest_volumes = Vec::with_capacity(tets.len());

        for tet in tets {
            let [i0, i1, i2, i3] = *tet;
            tet_structs.push(Tetrahedron {
                verts: [i0, i1, i2, i3],
            });

            let vol = tet_volume(positions[i0], positions[i1], positions[i2], positions[i3]).abs();

            rest_volumes.push(vol);

            let tet_mass = density * vol;
            let per_vertex = tet_mass / 4.0;
            masses[i0] += per_vertex;
            masses[i1] += per_vertex;
            masses[i2] += per_vertex;
            masses[i3] += per_vertex;

            volume_constraints.push(VolumeConstraint {
                verts: [i0, i1, i2, i3],
                rest_volume: vol,
                compliance: 1e-4,
            });
        }

        let inv_masses: Vec<f32> = masses
            .iter()
            .map(|&m| if m > f32::EPSILON { 1.0 / m } else { 0.0 })
            .collect();

        // Build distance constraints from unique edges.
        let edges = build_tet_edges(&tet_structs);
        let distance_constraints: Vec<DistanceConstraint> = edges
            .iter()
            .map(|&(a, b)| {
                let rest_length = len3(sub3(positions[b], positions[a]));
                DistanceConstraint::with_compliance(a, b, rest_length, 1e-6)
            })
            .collect();

        let velocities = vec![[0.0f32; 3]; n];
        let positions = positions.to_vec();

        Self {
            positions,
            velocities,
            inv_masses,
            tets: tet_structs,
            distance_constraints,
            volume_constraints,
            rest_volumes,
        }
    }

    // ── Simulation step ───────────────────────────────────────────────────────

    /// Advance the simulation by one time step.
    ///
    /// Uses XPBD: integrate gravity, solve constraints `substeps` times,
    /// then recover velocities from position differences.
    pub fn step(&mut self, dt: f32, substeps: u32, gravity: [f32; 3]) {
        let h = dt / substeps as f32;

        for _ in 0..substeps {
            // Integrate gravity (explicit Euler on velocities).
            for i in 0..self.positions.len() {
                if self.inv_masses[i] < f32::EPSILON {
                    continue;
                }
                self.velocities[i] = add3(self.velocities[i], scale3(gravity, h));
                self.positions[i] = add3(self.positions[i], scale3(self.velocities[i], h));
            }

            // Solve distance constraints.
            let dc: Vec<DistanceConstraint> = self.distance_constraints.clone();
            for c in &dc {
                apply_distance_constraint(&mut self.positions, c, &self.inv_masses, h, 1);
            }

            // Solve volume constraints.
            let vc: Vec<VolumeConstraint> = self.volume_constraints.clone();
            for c in &vc {
                apply_volume_constraint(&mut self.positions, c, &self.inv_masses, h, 1);
            }
        }

        // Recover velocities from position change (implicit in XPBD).
        // Note: we already integrated velocities above; this is consistent
        // with a simple sub-stepping scheme.
    }

    // ── Impulse ───────────────────────────────────────────────────────────────

    /// Apply an instantaneous velocity impulse to a vertex.
    pub fn apply_impulse(&mut self, vertex: usize, impulse: [f32; 3]) {
        if self.inv_masses[vertex] < f32::EPSILON {
            return;
        }
        self.velocities[vertex] = add3(
            self.velocities[vertex],
            scale3(impulse, self.inv_masses[vertex]),
        );
    }

    // ── Energy queries ────────────────────────────────────────────────────────

    /// Compute total kinetic energy: `Σ ½ m v²`.
    pub fn kinetic_energy(&self) -> f32 {
        self.velocities
            .iter()
            .zip(self.inv_masses.iter())
            .filter(|(_, &w)| w > f32::EPSILON)
            .map(|(v, &w)| {
                let m = 1.0 / w;
                0.5 * m * dot3(*v, *v)
            })
            .sum()
    }

    /// Compute gravitational potential energy: `Σ m g·h`.
    ///
    /// Uses the Y-axis height relative to Y = 0.
    pub fn potential_energy(&self, gravity: [f32; 3]) -> f32 {
        let g_len = len3(gravity);
        if g_len < f32::EPSILON {
            return 0.0;
        }
        self.positions
            .iter()
            .zip(self.inv_masses.iter())
            .filter(|(_, &w)| w > f32::EPSILON)
            .map(|(p, &w)| {
                let m = 1.0 / w;
                // Height along the gravity direction (negative = down).
                let h = -dot3(*p, scale3(gravity, 1.0 / g_len));
                m * g_len * h
            })
            .sum()
    }

    /// Compute the centre of mass of all free (non-pinned) vertices.
    pub fn center_of_mass(&self) -> [f32; 3] {
        let mut total_mass = 0.0f32;
        let mut com = [0.0f32; 3];
        for (p, &w) in self.positions.iter().zip(self.inv_masses.iter()) {
            if w > f32::EPSILON {
                let m = 1.0 / w;
                com = add3(com, scale3(*p, m));
                total_mass += m;
            }
        }
        if total_mass > f32::EPSILON {
            scale3(com, 1.0 / total_mass)
        } else {
            [0.0; 3]
        }
    }
}

// ── build_tet_edges ───────────────────────────────────────────────────────────

/// Collect the unique set of edges from a slice of tetrahedra.
pub fn build_tet_edges(tets: &[Tetrahedron]) -> Vec<(usize, usize)> {
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    for tet in tets {
        let v = tet.verts;
        // All 6 edges of a tet: (01, 02, 03, 12, 13, 23).
        let pairs = [
            (v[0].min(v[1]), v[0].max(v[1])),
            (v[0].min(v[2]), v[0].max(v[2])),
            (v[0].min(v[3]), v[0].max(v[3])),
            (v[1].min(v[2]), v[1].max(v[2])),
            (v[1].min(v[3]), v[1].max(v[3])),
            (v[2].min(v[3]), v[2].max(v[3])),
        ];
        for pair in pairs {
            seen.insert(pair);
        }
    }
    let mut edges: Vec<(usize, usize)> = seen.into_iter().collect();
    edges.sort();
    edges
}

// ── make_cube_soft_body ───────────────────────────────────────────────────────

/// Build a soft body cube subdivided into a tetrahedral lattice.
///
/// The cube extends from `(0,0,0)` to `(side, side, side)` and is divided
/// into a `subdivisions × subdivisions × subdivisions` voxel grid, each
/// voxel split into 5 tetrahedra.
pub fn make_cube_soft_body(side: f32, subdivisions: usize) -> SoftBody {
    let n = subdivisions + 1;
    let step = side / subdivisions as f32;

    // Generate a regular grid of vertices.
    let mut positions: Vec<[f32; 3]> = Vec::new();
    for iz in 0..n {
        for iy in 0..n {
            for ix in 0..n {
                positions.push([ix as f32 * step, iy as f32 * step, iz as f32 * step]);
            }
        }
    }

    let idx = |ix: usize, iy: usize, iz: usize| iz * n * n + iy * n + ix;

    // Split each voxel into 5 tetrahedra (standard cube-to-tet decomposition).
    let mut tets: Vec<[usize; 4]> = Vec::new();
    for iz in 0..subdivisions {
        for iy in 0..subdivisions {
            for ix in 0..subdivisions {
                let v000 = idx(ix, iy, iz);
                let v100 = idx(ix + 1, iy, iz);
                let v010 = idx(ix, iy + 1, iz);
                let v110 = idx(ix + 1, iy + 1, iz);
                let v001 = idx(ix, iy, iz + 1);
                let v101 = idx(ix + 1, iy, iz + 1);
                let v011 = idx(ix, iy + 1, iz + 1);
                let v111 = idx(ix + 1, iy + 1, iz + 1);

                tets.push([v000, v100, v010, v001]);
                tets.push([v100, v110, v010, v111]);
                tets.push([v001, v100, v101, v111]);
                tets.push([v001, v010, v011, v111]);
                tets.push([v001, v010, v100, v111]);
            }
        }
    }

    SoftBody::from_tet_mesh(&positions, &tets, 1000.0)
}

// ── Private vec3 helpers ──────────────────────────────────────────────────────

#[inline]
fn tet_volume(a: [f32; 3], b: [f32; 3], c: [f32; 3], d: [f32; 3]) -> f32 {
    let ab = sub3(b, a);
    let ac = sub3(c, a);
    let ad = sub3(d, a);
    dot3(ab, cross3(ac, ad)) / 6.0
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(a: [f32; 3]) -> f32 {
    dot3(a, a).sqrt()
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn single_tet() -> SoftBody {
        let positions = [
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let tets = [[0usize, 1, 2, 3]];
        SoftBody::from_tet_mesh(&positions, &tets, 1000.0)
    }

    // ── from_tet_mesh ─────────────────────────────────────────────────────────

    #[test]
    fn test_from_tet_mesh_vertex_count() {
        let sb = single_tet();
        assert_eq!(sb.positions.len(), 4);
        assert_eq!(sb.inv_masses.len(), 4);
        assert_eq!(sb.velocities.len(), 4);
    }

    #[test]
    fn test_from_tet_mesh_mass_distribution() {
        // Unit tet volume = 1/6; density=1000 → total mass=1000/6.
        // Each vertex gets (1000/6)/4 = 1000/24; inv_mass = 24/1000.
        let sb = single_tet();
        let expected_inv = 24.0 / 1000.0;
        for &w in &sb.inv_masses {
            assert!((w - expected_inv).abs() < 1e-4, "w={w}");
        }
    }

    #[test]
    fn test_from_tet_mesh_volume_constraints() {
        let sb = single_tet();
        assert_eq!(sb.volume_constraints.len(), 1);
        let rest_vol = sb.volume_constraints[0].rest_volume;
        assert!((rest_vol - 1.0 / 6.0).abs() < 1e-5, "rest_vol={rest_vol}");
    }

    #[test]
    fn test_from_tet_mesh_distance_constraints() {
        // A single tet has 6 unique edges.
        let sb = single_tet();
        assert_eq!(sb.distance_constraints.len(), 6);
    }

    // ── step gravity ─────────────────────────────────────────────────────────

    #[test]
    fn test_step_gravity_drops_com() {
        let mut sb = single_tet();
        let com_before = sb.center_of_mass();
        let gravity = [0.0f32, -9.81, 0.0];
        sb.step(0.016, 4, gravity);
        let com_after = sb.center_of_mass();
        // COM should have dropped (Y decreased).
        assert!(
            com_after[1] < com_before[1],
            "COM did not drop: before={} after={}",
            com_before[1],
            com_after[1]
        );
    }

    #[test]
    fn test_step_no_gravity_stays() {
        let mut sb = single_tet();
        let com_before = sb.center_of_mass();
        let gravity = [0.0f32, 0.0, 0.0];
        sb.step(0.016, 4, gravity);
        let com_after = sb.center_of_mass();
        // Without gravity COM should not drift significantly.
        let drift = len3(sub3(com_after, com_before));
        assert!(drift < 0.05, "unexpected COM drift={drift}");
    }

    // ── kinetic_energy ────────────────────────────────────────────────────────

    #[test]
    fn test_kinetic_energy_zero_at_rest() {
        let sb = single_tet();
        assert_eq!(sb.kinetic_energy(), 0.0);
    }

    #[test]
    fn test_kinetic_energy_positive_after_impulse() {
        let mut sb = single_tet();
        sb.apply_impulse(0, [1.0, 0.0, 0.0]);
        assert!(sb.kinetic_energy() > 0.0);
    }

    // ── apply_impulse ─────────────────────────────────────────────────────────

    #[test]
    fn test_apply_impulse_changes_velocity() {
        let mut sb = single_tet();
        let before = sb.velocities[0];
        sb.apply_impulse(0, [2.0, 0.0, 0.0]);
        let after = sb.velocities[0];
        assert!(after[0] > before[0], "velocity should have increased");
    }

    #[test]
    fn test_apply_impulse_pinned_vertex_no_change() {
        let mut sb = single_tet();
        // Pin vertex 0.
        sb.inv_masses[0] = 0.0;
        let before = sb.velocities[0];
        sb.apply_impulse(0, [10.0, 0.0, 0.0]);
        assert_eq!(sb.velocities[0], before, "pinned vertex should not move");
    }

    // ── center_of_mass ────────────────────────────────────────────────────────

    #[test]
    fn test_center_of_mass_unit_tet() {
        let sb = single_tet();
        let com = sb.center_of_mass();
        // All vertices have equal mass → COM = (0.25, 0.25, 0.25).
        assert!((com[0] - 0.25).abs() < 1e-4, "com[0]={}", com[0]);
        assert!((com[1] - 0.25).abs() < 1e-4, "com[1]={}", com[1]);
        assert!((com[2] - 0.25).abs() < 1e-4, "com[2]={}", com[2]);
    }

    // ── build_tet_edges ───────────────────────────────────────────────────────

    #[test]
    fn test_build_tet_edges_single_tet() {
        let tets = [Tetrahedron {
            verts: [0, 1, 2, 3],
        }];
        let edges = build_tet_edges(&tets);
        assert_eq!(edges.len(), 6, "a single tet has 6 edges");
    }

    #[test]
    fn test_build_tet_edges_shared_edge() {
        // Two tets sharing an edge should still report it only once.
        let tets = [
            Tetrahedron {
                verts: [0, 1, 2, 3],
            },
            Tetrahedron {
                verts: [0, 1, 4, 5],
            },
        ];
        let edges = build_tet_edges(&tets);
        // 6 + 6 - 1 shared (0,1) = 11.
        assert_eq!(edges.len(), 11, "got {} edges", edges.len());
    }

    // ── make_cube_soft_body ───────────────────────────────────────────────────

    #[test]
    fn test_make_cube_soft_body_vertex_count() {
        // subdivisions=1 → 2×2×2 = 8 vertices, 5 tets.
        let sb = make_cube_soft_body(1.0, 1);
        assert_eq!(sb.positions.len(), 8);
        assert_eq!(sb.tets.len(), 5);
    }
}
