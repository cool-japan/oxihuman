// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cloth tearing / fracture simulation.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    len3(sub3(a, b))
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A constraint between two particles.
#[derive(Debug, Clone)]
pub struct TearConstraint {
    pub particle_a: usize,
    pub particle_b: usize,
    pub rest_length: f32,
    pub max_stretch: f32,
    pub broken: bool,
}

/// Tearable cloth mesh.
#[derive(Debug, Clone)]
pub struct TearableMesh {
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
    pub constraints: Vec<TearConstraint>,
    pub pinned: Vec<bool>,
    pub mass: f32,
}

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

/// Create a flat tearable grid of rows x cols particles.
pub fn new_tearable_grid(rows: usize, cols: usize, spacing: f32, max_stretch: f32) -> TearableMesh {
    let n = rows * cols;
    let mut positions = Vec::with_capacity(n);
    let mut velocities = Vec::with_capacity(n);
    let mut pinned = Vec::with_capacity(n);

    for r in 0..rows {
        for c in 0..cols {
            positions.push([c as f32 * spacing, -(r as f32) * spacing, 0.0]);
            velocities.push([0.0, 0.0, 0.0]);
            pinned.push(r == 0); // pin top row
        }
    }

    let mut constraints = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let idx = r * cols + c;
            // Horizontal constraint
            if c + 1 < cols {
                let nbr = r * cols + c + 1;
                constraints.push(TearConstraint {
                    particle_a: idx,
                    particle_b: nbr,
                    rest_length: spacing,
                    max_stretch,
                    broken: false,
                });
            }
            // Vertical constraint
            if r + 1 < rows {
                let nbr = (r + 1) * cols + c;
                constraints.push(TearConstraint {
                    particle_a: idx,
                    particle_b: nbr,
                    rest_length: spacing,
                    max_stretch,
                    broken: false,
                });
            }
            // Diagonal shear constraints
            if r + 1 < rows && c + 1 < cols {
                let nbr = (r + 1) * cols + c + 1;
                constraints.push(TearConstraint {
                    particle_a: idx,
                    particle_b: nbr,
                    rest_length: spacing * std::f32::consts::SQRT_2,
                    max_stretch,
                    broken: false,
                });
            }
        }
    }

    TearableMesh {
        positions,
        velocities,
        constraints,
        pinned,
        mass: 1.0,
    }
}

// ---------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------

/// Integrate positions, apply PBD constraints, check for tears.
pub fn step_tear_simulation(mesh: &mut TearableMesh, gravity: [f32; 3], dt: f32, substeps: u32) {
    let sub_dt = dt / substeps as f32;
    let inv_mass = if mesh.mass > 1e-10 {
        1.0 / mesh.mass
    } else {
        0.0
    };

    for _ in 0..substeps {
        // Semi-implicit Euler integration
        for i in 0..mesh.positions.len() {
            if mesh.pinned[i] {
                continue;
            }
            mesh.velocities[i] = add3(mesh.velocities[i], scale3(gravity, sub_dt));
            mesh.positions[i] = add3(mesh.positions[i], scale3(mesh.velocities[i], sub_dt));
        }

        // PBD constraint solving (1 iteration per substep for performance)
        for constraint in mesh.constraints.iter_mut() {
            if constraint.broken {
                continue;
            }
            let pa = constraint.particle_a;
            let pb = constraint.particle_b;
            if pa >= mesh.positions.len() || pb >= mesh.positions.len() {
                continue;
            }

            let delta = sub3(mesh.positions[pb], mesh.positions[pa]);
            let dist = len3(delta);
            if dist < 1e-10 {
                continue;
            }

            let stretch = dist / constraint.rest_length;

            // Check for tear
            if stretch > constraint.max_stretch {
                constraint.broken = true;
                continue;
            }

            let correction = (dist - constraint.rest_length) / dist;
            let half = scale3(delta, correction * 0.5);

            let pinned_a = mesh.pinned[pa];
            let pinned_b = mesh.pinned[pb];

            match (pinned_a, pinned_b) {
                (false, false) => {
                    mesh.positions[pa] = add3(mesh.positions[pa], scale3(half, inv_mass));
                    mesh.positions[pb] = sub3(mesh.positions[pb], scale3(half, inv_mass));
                }
                (false, true) => {
                    mesh.positions[pa] = add3(mesh.positions[pa], scale3(half, 2.0 * inv_mass));
                }
                (true, false) => {
                    mesh.positions[pb] = sub3(mesh.positions[pb], scale3(half, 2.0 * inv_mass));
                }
                (true, true) => {}
            }
        }
    }
}

/// Apply an impulse to all particles within radius of pos.
pub fn apply_force_at(mesh: &mut TearableMesh, pos: [f32; 3], radius: f32, force: [f32; 3]) {
    let inv_mass = if mesh.mass > 1e-10 {
        1.0 / mesh.mass
    } else {
        0.0
    };
    for i in 0..mesh.positions.len() {
        if mesh.pinned[i] {
            continue;
        }
        let d = dist3(mesh.positions[i], pos);
        if d <= radius {
            let factor = (1.0 - d / radius.max(1e-10)).max(0.0);
            mesh.velocities[i] = add3(mesh.velocities[i], scale3(force, factor * inv_mass));
        }
    }
}

/// Break all constraints near a point within radius.
pub fn tear_at_point(mesh: &mut TearableMesh, pos: [f32; 3], radius: f32) {
    for constraint in mesh.constraints.iter_mut() {
        if constraint.broken {
            continue;
        }
        let pa = constraint.particle_a;
        let pb = constraint.particle_b;
        if pa >= mesh.positions.len() || pb >= mesh.positions.len() {
            continue;
        }
        let mid = scale3(add3(mesh.positions[pa], mesh.positions[pb]), 0.5);
        if dist3(mid, pos) <= radius {
            constraint.broken = true;
        }
    }
}

/// Count broken constraints.
pub fn count_broken_constraints(mesh: &TearableMesh) -> usize {
    mesh.constraints.iter().filter(|c| c.broken).count()
}

/// Count intact constraints.
pub fn count_intact_constraints(mesh: &TearableMesh) -> usize {
    mesh.constraints.iter().filter(|c| !c.broken).count()
}

/// Check if all constraints are broken.
pub fn is_fully_torn(mesh: &TearableMesh) -> bool {
    mesh.constraints.is_empty() || mesh.constraints.iter().all(|c| c.broken)
}

/// Compute stretch ratio of a constraint given current positions.
pub fn compute_stretch_ratio(constraint: &TearConstraint, positions: &[[f32; 3]]) -> f32 {
    let pa = constraint.particle_a;
    let pb = constraint.particle_b;
    if pa >= positions.len() || pb >= positions.len() || constraint.rest_length < 1e-10 {
        return 1.0;
    }
    let d = dist3(positions[pa], positions[pb]);
    d / constraint.rest_length
}

/// Find indices of constraints that are near their breaking point.
pub fn find_overloaded_constraints(mesh: &TearableMesh, threshold: f32) -> Vec<usize> {
    mesh.constraints
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            if c.broken {
                return false;
            }
            let ratio = compute_stretch_ratio(c, &mesh.positions);
            ratio >= threshold
        })
        .map(|(i, _)| i)
        .collect()
}

/// Stats: (particles, intact, broken).
pub fn tearable_mesh_stats(mesh: &TearableMesh) -> (usize, usize, usize) {
    let intact = count_intact_constraints(mesh);
    let broken = count_broken_constraints(mesh);
    (mesh.positions.len(), intact, broken)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn small_grid() -> TearableMesh {
        new_tearable_grid(3, 3, 1.0, 1.5)
    }

    #[test]
    fn test_new_tearable_grid_particles() {
        let mesh = new_tearable_grid(3, 4, 1.0, 1.5);
        assert_eq!(mesh.positions.len(), 12);
    }

    #[test]
    fn test_new_tearable_grid_constraints_positive() {
        let mesh = small_grid();
        assert!(!mesh.constraints.is_empty());
    }

    #[test]
    fn test_new_tearable_grid_pinned_top_row() {
        let mesh = new_tearable_grid(3, 3, 1.0, 1.5);
        // First 3 particles (top row) should be pinned
        for i in 0..3 {
            assert!(mesh.pinned[i], "Particle {i} should be pinned");
        }
    }

    #[test]
    fn test_new_tearable_grid_bottom_unpinned() {
        let mesh = new_tearable_grid(3, 3, 1.0, 1.5);
        for i in 3..9 {
            assert!(!mesh.pinned[i], "Particle {i} should not be pinned");
        }
    }

    #[test]
    fn test_step_simulation_moves_particles() {
        let mut mesh = new_tearable_grid(3, 3, 1.0, 1.5);
        let initial_y = mesh.positions[4][1]; // middle particle
        step_tear_simulation(&mut mesh, [0.0, -9.8, 0.0], 0.016, 4);
        // Unpinned particles should have moved downward
        assert!(mesh.positions[4][1] < initial_y);
    }

    #[test]
    fn test_step_simulation_pinned_dont_move() {
        let mut mesh = new_tearable_grid(3, 3, 1.0, 1.5);
        let initial_pos = mesh.positions[0];
        step_tear_simulation(&mut mesh, [0.0, -9.8, 0.0], 0.016, 4);
        assert_eq!(
            mesh.positions[0], initial_pos,
            "Pinned particle should not move"
        );
    }

    #[test]
    fn test_count_broken_initially_zero() {
        let mesh = small_grid();
        assert_eq!(count_broken_constraints(&mesh), 0);
    }

    #[test]
    fn test_count_intact_initially_all() {
        let mesh = small_grid();
        let total = mesh.constraints.len();
        assert_eq!(count_intact_constraints(&mesh), total);
    }

    #[test]
    fn test_tear_at_point() {
        let mut mesh = small_grid();
        // Tear at center
        tear_at_point(&mut mesh, [1.0, -1.0, 0.0], 1.5);
        assert!(count_broken_constraints(&mesh) > 0);
    }

    #[test]
    fn test_is_fully_torn_false_initially() {
        let mesh = small_grid();
        assert!(!is_fully_torn(&mesh));
    }

    #[test]
    fn test_is_fully_torn_true_when_all_broken() {
        let mut mesh = small_grid();
        for c in mesh.constraints.iter_mut() {
            c.broken = true;
        }
        assert!(is_fully_torn(&mesh));
    }

    #[test]
    fn test_compute_stretch_ratio_rest() {
        let mesh = small_grid();
        let ratio = compute_stretch_ratio(&mesh.constraints[0], &mesh.positions);
        assert!(
            (ratio - 1.0).abs() < 0.01,
            "At rest, stretch ratio should be ~1.0"
        );
    }

    #[test]
    fn test_find_overloaded_constraints_none_at_rest() {
        let mesh = small_grid();
        // At rest, stretch ratio = 1.0, threshold 1.2 -> none overloaded
        let overloaded = find_overloaded_constraints(&mesh, 1.2);
        assert!(overloaded.is_empty());
    }

    #[test]
    fn test_tearable_mesh_stats() {
        let mesh = small_grid();
        let (particles, intact, broken) = tearable_mesh_stats(&mesh);
        assert_eq!(particles, 9);
        assert!(intact > 0);
        assert_eq!(broken, 0);
    }

    #[test]
    fn test_apply_force_at_changes_velocity() {
        let mut mesh = small_grid();
        apply_force_at(&mut mesh, [1.0, -1.0, 0.0], 2.0, [0.0, 0.0, 10.0]);
        // At least some unpinned particles should have non-zero z velocity
        let any_z = mesh.velocities.iter().any(|v| v[2].abs() > 0.0);
        assert!(any_z);
    }

    #[test]
    fn test_large_gravity_causes_tears() {
        let mut mesh = new_tearable_grid(3, 3, 1.0, 1.01); // very low max_stretch
                                                           // Run many steps with large gravity to cause tearing
        for _ in 0..100 {
            step_tear_simulation(&mut mesh, [0.0, -100.0, 0.0], 0.016, 4);
        }
        assert!(count_broken_constraints(&mesh) > 0);
    }
}
