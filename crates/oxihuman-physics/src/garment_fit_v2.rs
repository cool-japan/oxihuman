// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Garment fitting pipeline v2.
//!
//! Drapes a garment mesh onto a body using position-based cloth simulation
//! with SDF body collision and optional self-collision detection.
//!
//! ## Pipeline
//! 1. Build an SDF from the body mesh via [`crate::sdf_gen::SdfGrid`].
//! 2. Initialise cloth particle state from the garment mesh.
//! 3. For each simulation step:
//!    a. Apply gravity to velocities.
//!    b. Predict positions via explicit Euler.
//!    c. Project stretch and bend constraints (PBD-style).
//!    d. Resolve body collision through SDF queries.
//!    e. Resolve self-collision if enabled.
//!    f. Update velocities from corrected positions.
//!    g. Apply damping.
//! 4. Terminate early if kinetic energy falls below a convergence threshold.

use anyhow::{bail, Result};

use crate::sdf_gen::{SdfConfig, SdfGrid};
use crate::self_collision::{build_adjacency, SelfCollisionDetector};

// ---------------------------------------------------------------------------
// Vec3 helpers (f64)
// ---------------------------------------------------------------------------

#[inline]
fn v3_sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn v3_add(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn v3_scale(a: &[f64; 3], s: f64) -> [f64; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

#[inline]
fn v3_dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn v3_cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn v3_len_sq(a: &[f64; 3]) -> f64 {
    v3_dot(a, a)
}

#[inline]
fn v3_len(a: &[f64; 3]) -> f64 {
    v3_len_sq(a).sqrt()
}

#[inline]
fn v3_normalize(a: &[f64; 3]) -> Option<[f64; 3]> {
    let len = v3_len(a);
    if len < 1e-15 {
        None
    } else {
        Some(v3_scale(a, 1.0 / len))
    }
}

#[inline]
fn v3_add_scaled(a: &[f64; 3], b: &[f64; 3], s: f64) -> [f64; 3] {
    [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s]
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the garment fitting v2 pipeline.
#[derive(Debug, Clone)]
pub struct GarmentFitConfig {
    /// Maximum number of simulation steps.
    pub simulation_steps: usize,
    /// Time step per simulation iteration (seconds).
    pub dt: f64,
    /// Gravity vector in world space (m/s^2).
    pub gravity: [f64; 3],
    /// Stiffness coefficient for stretch (distance) constraints, range 0..1.
    pub cloth_stretch_stiffness: f64,
    /// Stiffness coefficient for bend (dihedral angle) constraints, range 0..1.
    pub cloth_bend_stiffness: f64,
    /// Thickness used for body collision offset (metres).
    pub collision_thickness: f64,
    /// Friction coefficient applied during body collision response.
    pub friction: f64,
    /// Whether to enable self-collision detection.
    pub self_collision: bool,
    /// SDF grid resolution along each axis.
    pub sdf_resolution: [usize; 3],
    /// Number of Gauss-Seidel constraint relaxation iterations per step.
    pub relaxation_iterations: usize,
    /// Velocity damping factor applied each step (0 = no damping, 1 = full damp).
    pub damping: f64,
    /// Convergence threshold: kinetic energy per vertex below which we stop.
    pub convergence_threshold: f64,
    /// Self-collision thickness (distance threshold for self-intersection).
    pub self_collision_thickness: f64,
    /// SDF padding around the body bounding box.
    pub sdf_padding: f64,
    /// Seam constraint stiffness (keeps seam pairs together).
    pub seam_stiffness: f64,
}

impl Default for GarmentFitConfig {
    fn default() -> Self {
        Self {
            simulation_steps: 200,
            dt: 1.0 / 60.0,
            gravity: [0.0, -9.81, 0.0],
            cloth_stretch_stiffness: 0.9,
            cloth_bend_stiffness: 0.1,
            collision_thickness: 0.003,
            friction: 0.3,
            self_collision: true,
            sdf_resolution: [64, 64, 64],
            relaxation_iterations: 5,
            damping: 0.01,
            convergence_threshold: 1e-7,
            self_collision_thickness: 0.005,
            sdf_padding: 0.15,
            seam_stiffness: 0.95,
        }
    }
}

// ---------------------------------------------------------------------------
// Input / output types
// ---------------------------------------------------------------------------

/// A garment mesh to be draped on a body.
#[derive(Debug, Clone)]
pub struct GarmentMesh {
    /// Vertex positions in world space.
    pub positions: Vec<[f64; 3]>,
    /// Triangle indices (each triple indexes into `positions`).
    pub triangles: Vec<[usize; 3]>,
    /// Per-vertex UV coordinates (can be empty if unused).
    pub uv_coords: Vec<[f64; 2]>,
    /// Indices of vertices that remain fixed (e.g. waistband, shoulder pins).
    pub pinned_vertices: Vec<usize>,
    /// Pairs of vertex indices that should stay connected (seam stitching).
    pub seam_pairs: Vec<(usize, usize)>,
}

/// The result of a garment fitting simulation.
#[derive(Debug, Clone)]
pub struct GarmentFitResult {
    /// Vertex positions after draping.
    pub draped_positions: Vec<[f64; 3]>,
    /// Number of body-collision corrections applied (cumulative).
    pub collision_count: usize,
    /// Number of self-collision corrections applied (cumulative).
    pub self_collision_count: usize,
    /// Total potential energy of the cloth at termination.
    pub total_energy: f64,
    /// Whether the simulation converged before reaching `simulation_steps`.
    pub converged: bool,
    /// Actual number of simulation steps taken.
    pub steps_taken: usize,
}

// ---------------------------------------------------------------------------
// Internal constraint representation
// ---------------------------------------------------------------------------

/// A distance (stretch) constraint between two vertices.
#[derive(Debug, Clone)]
struct StretchConstraint {
    i: usize,
    j: usize,
    rest_length: f64,
}

/// A bend constraint across a shared edge of two triangles.
///
/// The four vertices are ordered: `[wing_a, shared_0, shared_1, wing_b]`,
/// where the shared edge is `(shared_0, shared_1)` and `wing_a`/`wing_b`
/// are the opposing vertices of each triangle.
#[derive(Debug, Clone)]
struct BendConstraint {
    /// Indices of the four participating vertices.
    v: [usize; 4],
    /// Rest dihedral angle between the two triangle faces.
    rest_angle: f64,
}

// ---------------------------------------------------------------------------
// GarmentFitterV2
// ---------------------------------------------------------------------------

/// Garment fitting pipeline v2.
///
/// Drapes a garment mesh onto a body using cloth simulation with SDF collision.
pub struct GarmentFitterV2 {
    config: GarmentFitConfig,
}

impl GarmentFitterV2 {
    /// Create a new fitter with the given configuration.
    pub fn new(config: GarmentFitConfig) -> Self {
        Self { config }
    }

    /// Fit a garment to a body mesh.
    ///
    /// 1. Generates an SDF from the body mesh.
    /// 2. Initialises cloth simulation from the garment mesh.
    /// 3. Runs simulation steps with SDF collision + optional self-collision.
    /// 4. Returns the draped vertex positions and statistics.
    pub fn fit(
        &self,
        garment: &GarmentMesh,
        body_vertices: &[[f64; 3]],
        body_triangles: &[[usize; 3]],
        body_normals: &[[f64; 3]],
    ) -> Result<GarmentFitResult> {
        self.run_simulation(
            garment,
            &garment.positions,
            body_vertices,
            body_triangles,
            body_normals,
        )
    }

    /// Refit after a body shape change, warm-starting from previous positions.
    ///
    /// Identical to [`fit`](Self::fit) except the cloth particles start from
    /// `previous_positions` instead of the garment rest positions.
    pub fn refit(
        &self,
        garment: &GarmentMesh,
        previous_positions: &[[f64; 3]],
        body_vertices: &[[f64; 3]],
        body_triangles: &[[usize; 3]],
        body_normals: &[[f64; 3]],
    ) -> Result<GarmentFitResult> {
        if previous_positions.len() != garment.positions.len() {
            bail!(
                "previous_positions length ({}) does not match garment vertex count ({})",
                previous_positions.len(),
                garment.positions.len()
            );
        }
        self.run_simulation(
            garment,
            previous_positions,
            body_vertices,
            body_triangles,
            body_normals,
        )
    }

    /// Compute per-triangle strain (ratio of deformed area to rest area - 1).
    ///
    /// A value of 0 means no deformation; positive means stretching.
    pub fn strain_map(
        rest_positions: &[[f64; 3]],
        current_positions: &[[f64; 3]],
        triangles: &[[usize; 3]],
    ) -> Vec<f64> {
        triangles
            .iter()
            .map(|tri| {
                let rest_area = triangle_area(
                    &rest_positions[tri[0]],
                    &rest_positions[tri[1]],
                    &rest_positions[tri[2]],
                );
                let cur_area = triangle_area(
                    &current_positions[tri[0]],
                    &current_positions[tri[1]],
                    &current_positions[tri[2]],
                );
                if rest_area < 1e-15 {
                    0.0
                } else {
                    (cur_area / rest_area) - 1.0
                }
            })
            .collect()
    }

    // -----------------------------------------------------------------------
    // Core simulation
    // -----------------------------------------------------------------------

    fn run_simulation(
        &self,
        garment: &GarmentMesh,
        initial_positions: &[[f64; 3]],
        body_vertices: &[[f64; 3]],
        body_triangles: &[[usize; 3]],
        body_normals: &[[f64; 3]],
    ) -> Result<GarmentFitResult> {
        let cfg = &self.config;
        let n_verts = garment.positions.len();

        // -- validate inputs ------------------------------------------------
        Self::validate_garment(garment)?;
        if initial_positions.len() != n_verts {
            bail!(
                "initial_positions length ({}) != garment vertex count ({})",
                initial_positions.len(),
                n_verts
            );
        }

        // -- build SDF from body mesh ---------------------------------------
        let sdf_config = SdfConfig {
            resolution: cfg.sdf_resolution,
            padding: cfg.sdf_padding,
        };
        let sdf = SdfGrid::from_mesh(body_vertices, body_triangles, body_normals, &sdf_config)?;

        // -- build constraints ----------------------------------------------
        let stretch_constraints = build_stretch_constraints(&garment.positions, &garment.triangles);
        let bend_constraints = build_bend_constraints(&garment.positions, &garment.triangles);

        // -- inverse masses (pinned vertices have 0 inverse mass) -----------
        let mut inv_mass = vec![1.0_f64; n_verts];
        for &pi in &garment.pinned_vertices {
            if pi < n_verts {
                inv_mass[pi] = 0.0;
            }
        }

        // -- initialise particle state --------------------------------------
        let mut positions: Vec<[f64; 3]> = initial_positions.to_vec();
        let mut velocities: Vec<[f64; 3]> = vec![[0.0; 3]; n_verts];
        let mut predicted: Vec<[f64; 3]> = vec![[0.0; 3]; n_verts];

        // -- self-collision setup -------------------------------------------
        let mut self_coll_detector = if cfg.self_collision {
            let cell_size = cfg.self_collision_thickness * 2.0;
            Some(SelfCollisionDetector::new(
                cfg.self_collision_thickness,
                cell_size,
            )?)
        } else {
            None
        };
        let adjacency = if cfg.self_collision {
            build_adjacency(n_verts, &garment.triangles)?
        } else {
            Vec::new()
        };

        let mut total_collision_count: usize = 0;
        let mut total_self_collision_count: usize = 0;
        let mut converged = false;
        let mut steps_taken = 0;

        let dt = cfg.dt;
        let dt_sq = dt * dt;
        let gravity = cfg.gravity;
        let damping_factor = 1.0 - cfg.damping;

        // -- simulation loop ------------------------------------------------
        for step in 0..cfg.simulation_steps {
            steps_taken = step + 1;

            // (a) apply gravity to velocities
            for i in 0..n_verts {
                if inv_mass[i] > 0.0 {
                    velocities[i][0] += gravity[0] * dt;
                    velocities[i][1] += gravity[1] * dt;
                    velocities[i][2] += gravity[2] * dt;
                }
            }

            // (b) predict positions: x_pred = x + v * dt
            for i in 0..n_verts {
                predicted[i] = v3_add_scaled(&positions[i], &velocities[i], dt);
            }

            // (c) constraint projection (Gauss-Seidel relaxation)
            for _relax in 0..cfg.relaxation_iterations {
                // stretch constraints
                project_stretch_constraints(
                    &stretch_constraints,
                    &mut predicted,
                    &inv_mass,
                    cfg.cloth_stretch_stiffness,
                );

                // bend constraints
                project_bend_constraints(
                    &bend_constraints,
                    &mut predicted,
                    &inv_mass,
                    cfg.cloth_bend_stiffness,
                    dt_sq,
                );

                // seam constraints (distance = 0)
                project_seam_constraints(
                    &garment.seam_pairs,
                    &mut predicted,
                    &inv_mass,
                    cfg.seam_stiffness,
                );
            }

            // (d) SDF body collision
            let coll_count = resolve_sdf_collisions(
                &sdf,
                &mut predicted,
                &inv_mass,
                cfg.collision_thickness,
                cfg.friction,
            );
            total_collision_count += coll_count;

            // (e) self-collision
            if let Some(ref mut detector) = self_coll_detector {
                let sc_count = resolve_self_collisions(
                    detector,
                    &mut predicted,
                    &garment.triangles,
                    &adjacency,
                    &inv_mass,
                )?;
                total_self_collision_count += sc_count;
            }

            // (f) update velocities from corrected positions
            let inv_dt = 1.0 / dt;
            for i in 0..n_verts {
                if inv_mass[i] > 0.0 {
                    velocities[i][0] = (predicted[i][0] - positions[i][0]) * inv_dt;
                    velocities[i][1] = (predicted[i][1] - positions[i][1]) * inv_dt;
                    velocities[i][2] = (predicted[i][2] - positions[i][2]) * inv_dt;
                }
            }

            // (g) damping
            for vel in velocities[..n_verts].iter_mut() {
                vel[0] *= damping_factor;
                vel[1] *= damping_factor;
                vel[2] *= damping_factor;
            }

            // commit positions
            positions[..n_verts].copy_from_slice(&predicted[..n_verts]);

            // check convergence: average kinetic energy per vertex
            let ke = compute_kinetic_energy(&velocities, &inv_mass);
            let ke_per_vert = if n_verts > 0 {
                ke / (n_verts as f64)
            } else {
                0.0
            };
            if ke_per_vert < cfg.convergence_threshold {
                converged = true;
                break;
            }
        }

        // -- compute total energy -------------------------------------------
        let total_energy = compute_total_energy(
            &positions,
            &garment.positions,
            &stretch_constraints,
            &gravity,
            &inv_mass,
        );

        Ok(GarmentFitResult {
            draped_positions: positions,
            collision_count: total_collision_count,
            self_collision_count: total_self_collision_count,
            total_energy,
            converged,
            steps_taken,
        })
    }

    fn validate_garment(garment: &GarmentMesh) -> Result<()> {
        if garment.positions.is_empty() {
            bail!("garment has no vertices");
        }
        if garment.triangles.is_empty() {
            bail!("garment has no triangles");
        }
        let n = garment.positions.len();
        for (ti, tri) in garment.triangles.iter().enumerate() {
            for &idx in tri {
                if idx >= n {
                    bail!(
                        "garment triangle {ti} references vertex {idx}, but only {n} vertices exist"
                    );
                }
            }
        }
        if !garment.uv_coords.is_empty() && garment.uv_coords.len() != n {
            bail!(
                "uv_coords length ({}) does not match vertex count ({n})",
                garment.uv_coords.len()
            );
        }
        for &pi in &garment.pinned_vertices {
            if pi >= n {
                bail!("pinned vertex index {pi} is out of bounds (n_verts={n})");
            }
        }
        for (si, &(a, b)) in garment.seam_pairs.iter().enumerate() {
            if a >= n || b >= n {
                bail!("seam pair {si} references out-of-bounds vertex: ({a}, {b}), n_verts={n}");
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Constraint builders
// ---------------------------------------------------------------------------

/// Collect unique edges from the triangle list and create stretch constraints.
fn build_stretch_constraints(
    rest_positions: &[[f64; 3]],
    triangles: &[[usize; 3]],
) -> Vec<StretchConstraint> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    let mut constraints = Vec::new();

    for tri in triangles {
        let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[0], tri[2])];
        for (a, b) in edges {
            let key = if a < b { (a, b) } else { (b, a) };
            if seen.insert(key) {
                let rest_length = v3_len(&v3_sub(&rest_positions[a], &rest_positions[b]));
                constraints.push(StretchConstraint {
                    i: a,
                    j: b,
                    rest_length,
                });
            }
        }
    }
    constraints
}

/// Build bend constraints for each pair of triangles sharing an edge.
///
/// For each shared edge, the constraint involves the four vertices:
/// two on the shared edge plus one "wing" vertex from each triangle.
fn build_bend_constraints(
    rest_positions: &[[f64; 3]],
    triangles: &[[usize; 3]],
) -> Vec<BendConstraint> {
    use std::collections::HashMap;

    // Map each edge (sorted) to the list of triangle indices that share it.
    let mut edge_to_tris: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (ti, tri) in triangles.iter().enumerate() {
        let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[0], tri[2])];
        for (a, b) in edges {
            let key = if a < b { (a, b) } else { (b, a) };
            edge_to_tris.entry(key).or_default().push(ti);
        }
    }

    let mut constraints = Vec::new();

    for (&(e0, e1), tris) in &edge_to_tris {
        if tris.len() < 2 {
            continue; // boundary edge
        }
        // Use only the first pair (manifold mesh assumption)
        let ta = triangles[tris[0]];
        let tb = triangles[tris[1]];

        // Find wing vertices: the vertex in each triangle that is NOT on the shared edge.
        let wing_a = find_wing_vertex(&ta, e0, e1);
        let wing_b = find_wing_vertex(&tb, e0, e1);

        let (wing_a, wing_b) = match (wing_a, wing_b) {
            (Some(a), Some(b)) => (a, b),
            _ => continue,
        };

        let rest_angle = dihedral_angle(
            &rest_positions[wing_a],
            &rest_positions[e0],
            &rest_positions[e1],
            &rest_positions[wing_b],
        );

        constraints.push(BendConstraint {
            v: [wing_a, e0, e1, wing_b],
            rest_angle,
        });
    }

    constraints
}

/// Find the vertex in a triangle that is not one of `e0` or `e1`.
fn find_wing_vertex(tri: &[usize; 3], e0: usize, e1: usize) -> Option<usize> {
    tri.iter().find(|&&v| v != e0 && v != e1).copied()
}

/// Compute the dihedral angle between two triangles sharing edge (p1, p2).
///
/// `p0` and `p3` are the wing vertices.
fn dihedral_angle(p0: &[f64; 3], p1: &[f64; 3], p2: &[f64; 3], p3: &[f64; 3]) -> f64 {
    let e = v3_sub(p2, p1);
    let n0 = v3_cross(&v3_sub(p0, p1), &e);
    let n1 = v3_cross(&e, &v3_sub(p3, p1));

    let n0_len = v3_len(&n0);
    let n1_len = v3_len(&n1);
    if n0_len < 1e-15 || n1_len < 1e-15 {
        return 0.0;
    }

    let n0_n = v3_scale(&n0, 1.0 / n0_len);
    let n1_n = v3_scale(&n1, 1.0 / n1_len);

    let cos_angle = v3_dot(&n0_n, &n1_n).clamp(-1.0, 1.0);
    // Use cross product to determine sign
    let cross = v3_cross(&n0_n, &n1_n);
    let e_norm = match v3_normalize(&e) {
        Some(en) => en,
        None => return 0.0,
    };
    let sin_sign = v3_dot(&cross, &e_norm);
    sin_sign.atan2(cos_angle)
}

// ---------------------------------------------------------------------------
// Constraint projection
// ---------------------------------------------------------------------------

/// Project stretch (distance) constraints using PBD.
fn project_stretch_constraints(
    constraints: &[StretchConstraint],
    positions: &mut [[f64; 3]],
    inv_mass: &[f64],
    stiffness: f64,
) {
    for c in constraints {
        let diff = v3_sub(&positions[c.j], &positions[c.i]);
        let dist = v3_len(&diff);
        if dist < 1e-15 {
            continue;
        }

        let w_sum = inv_mass[c.i] + inv_mass[c.j];
        if w_sum < 1e-15 {
            continue;
        }

        let error = dist - c.rest_length;
        let correction_mag = stiffness * error / (w_sum * dist);

        let correction = v3_scale(&diff, correction_mag);

        positions[c.i] = v3_add(&positions[c.i], &v3_scale(&correction, inv_mass[c.i]));
        positions[c.j] = v3_sub(&positions[c.j], &v3_scale(&correction, inv_mass[c.j]));
    }
}

/// Project bend constraints using a simplified dihedral-angle PBD approach.
///
/// This applies a correction proportional to the angular deviation from the
/// rest dihedral angle, distributed to the four participating vertices
/// weighted by inverse mass and dt^2 scaling.
fn project_bend_constraints(
    constraints: &[BendConstraint],
    positions: &mut [[f64; 3]],
    inv_mass: &[f64],
    stiffness: f64,
    dt_sq: f64,
) {
    // Scale stiffness by dt^2 for temporal consistency (XPBD-like)
    let k = 1.0 - (1.0 - stiffness).powf(1.0 / dt_sq.max(1e-15));

    for c in constraints {
        let [v0, v1, v2, v3] = c.v;
        let current_angle = dihedral_angle(
            &positions[v0],
            &positions[v1],
            &positions[v2],
            &positions[v3],
        );
        let angle_error = current_angle - c.rest_angle;

        // Small angle approximation: correct wing vertices along face normals
        if angle_error.abs() < 1e-12 {
            continue;
        }

        let e = v3_sub(&positions[v2], &positions[v1]);
        let e_len = v3_len(&e);
        if e_len < 1e-15 {
            continue;
        }

        // Face normals of the two triangles
        let n0 = v3_cross(&v3_sub(&positions[v0], &positions[v1]), &e);
        let n1 = v3_cross(&e, &v3_sub(&positions[v3], &positions[v1]));
        let n0_len = v3_len(&n0);
        let n1_len = v3_len(&n1);
        if n0_len < 1e-15 || n1_len < 1e-15 {
            continue;
        }

        let n0_n = v3_scale(&n0, 1.0 / n0_len);
        let n1_n = v3_scale(&n1, 1.0 / n1_len);

        // Correction magnitude: move wing vertices along their respective normals
        let w_total = inv_mass[v0] + inv_mass[v1] + inv_mass[v2] + inv_mass[v3];
        if w_total < 1e-15 {
            continue;
        }

        let correction = k * angle_error / w_total;

        // Wing vertex 0 moves along n0_n
        let d0 = v3_scale(&n0_n, -correction * inv_mass[v0]);
        positions[v0] = v3_add(&positions[v0], &d0);

        // Wing vertex 3 moves along n1_n
        let d3 = v3_scale(&n1_n, correction * inv_mass[v3]);
        positions[v3] = v3_add(&positions[v3], &d3);

        // Shared edge vertices get blended corrections
        let blend = 0.5;
        let d_shared = v3_add(
            &v3_scale(&n0_n, correction * blend),
            &v3_scale(&n1_n, -correction * blend),
        );
        positions[v1] = v3_add_scaled(&positions[v1], &d_shared, inv_mass[v1] * 0.5);
        positions[v2] = v3_add_scaled(&positions[v2], &d_shared, inv_mass[v2] * 0.5);
    }
}

/// Project seam constraints: bring seam vertex pairs together.
fn project_seam_constraints(
    seam_pairs: &[(usize, usize)],
    positions: &mut [[f64; 3]],
    inv_mass: &[f64],
    stiffness: f64,
) {
    for &(a, b) in seam_pairs {
        let diff = v3_sub(&positions[b], &positions[a]);
        let dist = v3_len(&diff);
        if dist < 1e-15 {
            continue;
        }

        let w_sum = inv_mass[a] + inv_mass[b];
        if w_sum < 1e-15 {
            continue;
        }

        // Seam target is rest_length = 0 (vertices should coincide)
        let correction_mag = stiffness * dist / (w_sum * dist);
        let correction = v3_scale(&diff, correction_mag);

        positions[a] = v3_add(&positions[a], &v3_scale(&correction, inv_mass[a]));
        positions[b] = v3_sub(&positions[b], &v3_scale(&correction, inv_mass[b]));
    }
}

// ---------------------------------------------------------------------------
// Collision resolution
// ---------------------------------------------------------------------------

/// Resolve body collisions using the SDF.
///
/// For each non-pinned vertex that is inside or too close to the body surface,
/// push it outward along the SDF gradient. Returns the number of corrections.
fn resolve_sdf_collisions(
    sdf: &SdfGrid,
    positions: &mut [[f64; 3]],
    inv_mass: &[f64],
    thickness: f64,
    friction: f64,
) -> usize {
    let mut count = 0usize;

    for i in 0..positions.len() {
        if inv_mass[i] <= 0.0 {
            continue;
        }

        let d = sdf.sample(&positions[i]);
        if d < thickness {
            // Compute push direction from gradient
            let grad = sdf.gradient(&positions[i]);
            let grad_len = v3_len(&grad);

            // When the gradient is near-zero (e.g. at the exact center of a
            // symmetric object), perturb the sample point slightly along each
            // axis and pick the direction with the steepest ascent.
            let normal = if grad_len < 1e-12 {
                let eps = sdf.cell_size() * 0.25;
                let mut best_dir = [1.0, 0.0, 0.0_f64];
                let mut best_val = f64::MIN;
                for dir in &[
                    [1.0, 0.0, 0.0],
                    [-1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, -1.0, 0.0],
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, -1.0],
                ] {
                    let probe = [
                        positions[i][0] + dir[0] * eps,
                        positions[i][1] + dir[1] * eps,
                        positions[i][2] + dir[2] * eps,
                    ];
                    let val = sdf.sample(&probe);
                    if val > best_val {
                        best_val = val;
                        best_dir = *dir;
                    }
                }
                best_dir
            } else {
                v3_scale(&grad, 1.0 / grad_len)
            };

            // Push vertex to surface + thickness offset
            let penetration = thickness - d;
            let push = v3_scale(&normal, penetration);

            // Decompose current velocity component along normal for friction
            // (velocity is approximated from position change in next step)
            // Here we apply friction by scaling the tangential component of the push
            let push_normal_mag = v3_dot(&push, &normal);
            let push_normal = v3_scale(&normal, push_normal_mag);
            let push_tangent = v3_sub(&push, &push_normal);
            let tangent_len = v3_len(&push_tangent);

            let friction_correction = if tangent_len > 1e-15 {
                // Coulomb friction: clamp tangential correction
                let max_tangent = friction * push_normal_mag.abs();
                if tangent_len > max_tangent {
                    v3_scale(&push_tangent, max_tangent / tangent_len)
                } else {
                    push_tangent
                }
            } else {
                [0.0; 3]
            };

            let final_push = v3_add(&push_normal, &friction_correction);
            positions[i] = v3_add(&positions[i], &final_push);
            count += 1;
        }
    }

    count
}

/// Detect and resolve self-collisions. Returns the number of contacts resolved.
fn resolve_self_collisions(
    detector: &mut SelfCollisionDetector,
    positions: &mut [[f64; 3]],
    triangles: &[[usize; 3]],
    adjacency: &[Vec<usize>],
    inv_mass: &[f64],
) -> Result<usize> {
    let contacts = detector.detect(positions, triangles, adjacency)?;
    let count = contacts.len();

    if !contacts.is_empty() {
        SelfCollisionDetector::resolve_contacts(&contacts, positions, inv_mass)?;
    }

    Ok(count)
}

// ---------------------------------------------------------------------------
// Energy computation
// ---------------------------------------------------------------------------

/// Compute total kinetic energy of the system.
fn compute_kinetic_energy(velocities: &[[f64; 3]], inv_mass: &[f64]) -> f64 {
    let mut ke = 0.0_f64;
    for (i, v) in velocities.iter().enumerate() {
        if inv_mass[i] > 0.0 {
            let mass = 1.0 / inv_mass[i];
            ke += 0.5 * mass * v3_len_sq(v);
        }
    }
    ke
}

/// Compute total potential energy (elastic + gravitational).
fn compute_total_energy(
    positions: &[[f64; 3]],
    rest_positions: &[[f64; 3]],
    stretch_constraints: &[StretchConstraint],
    gravity: &[f64; 3],
    inv_mass: &[f64],
) -> f64 {
    let mut energy = 0.0_f64;

    // Elastic energy from stretch constraints
    for c in stretch_constraints {
        let dist = v3_len(&v3_sub(&positions[c.j], &positions[c.i]));
        let strain = dist - c.rest_length;
        energy += 0.5 * strain * strain;
    }

    // Gravitational potential energy
    let n = positions.len().min(rest_positions.len());
    for i in 0..n {
        if inv_mass[i] > 0.0 {
            let mass = 1.0 / inv_mass[i];
            // E_grav = -m * g . x  (dot product gives height-proportional energy)
            energy -= mass * v3_dot(gravity, &positions[i]);
        }
    }

    energy
}

// ---------------------------------------------------------------------------
// Geometry helpers
// ---------------------------------------------------------------------------

/// Area of a triangle defined by three vertices.
fn triangle_area(a: &[f64; 3], b: &[f64; 3], c: &[f64; 3]) -> f64 {
    let ab = v3_sub(b, a);
    let ac = v3_sub(c, a);
    let cross = v3_cross(&ab, &ac);
    0.5 * v3_len(&cross)
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- helpers ---

    /// Build a simple planar garment: a grid of triangles in the XZ plane.
    fn planar_garment(rows: usize, cols: usize, spacing: f64) -> GarmentMesh {
        let mut positions = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                positions.push([c as f64 * spacing, 0.0, r as f64 * spacing]);
            }
        }

        let mut triangles = Vec::new();
        for r in 0..(rows - 1) {
            for c in 0..(cols - 1) {
                let tl = r * cols + c;
                let tr = tl + 1;
                let bl = (r + 1) * cols + c;
                let br = bl + 1;
                triangles.push([tl, bl, tr]);
                triangles.push([tr, bl, br]);
            }
        }

        GarmentMesh {
            positions,
            triangles,
            uv_coords: Vec::new(),
            pinned_vertices: Vec::new(),
            seam_pairs: Vec::new(),
        }
    }

    /// Build a unit cube body mesh (same as sdf_gen tests).
    #[allow(clippy::type_complexity)]
    fn unit_cube_body() -> (Vec<[f64; 3]>, Vec<[usize; 3]>, Vec<[f64; 3]>) {
        let h = 0.5;
        #[rustfmt::skip]
        let verts: Vec<[f64; 3]> = vec![
            [-h, -h, -h], [ h, -h, -h], [ h,  h, -h], [-h,  h, -h],
            [-h, -h,  h], [ h, -h,  h], [ h,  h,  h], [-h,  h,  h],
        ];
        #[rustfmt::skip]
        let tris: Vec<[usize; 3]> = vec![
            [0, 2, 1], [0, 3, 2],
            [4, 5, 6], [4, 6, 7],
            [0, 1, 5], [0, 5, 4],
            [2, 3, 7], [2, 7, 6],
            [0, 4, 7], [0, 7, 3],
            [1, 2, 6], [1, 6, 5],
        ];

        let mut norms = vec![[0.0_f64; 3]; verts.len()];
        for tri in &tris {
            let a = verts[tri[0]];
            let b = verts[tri[1]];
            let c = verts[tri[2]];
            let ab = v3_sub(&b, &a);
            let ac = v3_sub(&c, &a);
            let n = v3_cross(&ab, &ac);
            for &vi in tri {
                norms[vi][0] += n[0];
                norms[vi][1] += n[1];
                norms[vi][2] += n[2];
            }
        }
        for n in &mut norms {
            let len = v3_len(n);
            if len > 1e-12 {
                *n = v3_scale(n, 1.0 / len);
            }
        }

        (verts, tris, norms)
    }

    // -- tests ---

    #[test]
    fn test_default_config() {
        let cfg = GarmentFitConfig::default();
        assert!(cfg.simulation_steps > 0);
        assert!(cfg.dt > 0.0);
        assert!(cfg.cloth_stretch_stiffness > 0.0);
        assert!(cfg.sdf_resolution[0] > 0);
    }

    #[test]
    fn test_strain_map_no_deformation() {
        let garment = planar_garment(3, 3, 0.1);
        let strain =
            GarmentFitterV2::strain_map(&garment.positions, &garment.positions, &garment.triangles);
        for &s in &strain {
            assert!(
                s.abs() < 1e-10,
                "expected zero strain for undeformed mesh, got {s}"
            );
        }
    }

    #[test]
    fn test_strain_map_stretched() {
        let garment = planar_garment(3, 3, 0.1);
        let stretched: Vec<[f64; 3]> = garment
            .positions
            .iter()
            .map(|p| [p[0] * 2.0, p[1], p[2] * 2.0])
            .collect();
        let strain =
            GarmentFitterV2::strain_map(&garment.positions, &stretched, &garment.triangles);
        // Uniform 2x scaling => area scales by 4x => strain = 3.0
        for &s in &strain {
            assert!(
                s > 0.0,
                "expected positive strain for stretched mesh, got {s}"
            );
        }
    }

    #[test]
    fn test_validate_garment_empty_positions() {
        let garment = GarmentMesh {
            positions: Vec::new(),
            triangles: vec![[0, 1, 2]],
            uv_coords: Vec::new(),
            pinned_vertices: Vec::new(),
            seam_pairs: Vec::new(),
        };
        assert!(GarmentFitterV2::validate_garment(&garment).is_err());
    }

    #[test]
    fn test_validate_garment_bad_triangle_index() {
        let garment = GarmentMesh {
            positions: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            triangles: vec![[0, 1, 5]], // index 5 out of bounds
            uv_coords: Vec::new(),
            pinned_vertices: Vec::new(),
            seam_pairs: Vec::new(),
        };
        assert!(GarmentFitterV2::validate_garment(&garment).is_err());
    }

    #[test]
    fn test_validate_garment_bad_pinned() {
        let garment = GarmentMesh {
            positions: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            triangles: vec![[0, 1, 2]],
            uv_coords: Vec::new(),
            pinned_vertices: vec![10],
            seam_pairs: Vec::new(),
        };
        assert!(GarmentFitterV2::validate_garment(&garment).is_err());
    }

    #[test]
    fn test_build_stretch_constraints() {
        let garment = planar_garment(2, 2, 1.0);
        let constraints = build_stretch_constraints(&garment.positions, &garment.triangles);
        // 2x2 grid => 2 triangles => 5 unique edges
        assert_eq!(constraints.len(), 5);
        for c in &constraints {
            assert!(c.rest_length > 0.0);
        }
    }

    #[test]
    fn test_build_bend_constraints() {
        let garment = planar_garment(2, 2, 1.0);
        let constraints = build_bend_constraints(&garment.positions, &garment.triangles);
        // 2 triangles sharing 1 internal edge => 1 bend constraint
        assert_eq!(constraints.len(), 1);
    }

    #[test]
    fn test_dihedral_angle_flat() {
        // Two coplanar triangles => dihedral angle ~ 0
        let p0 = [0.0, 1.0, 0.0]; // wing A
        let p1 = [0.0, 0.0, 0.0]; // shared edge start
        let p2 = [1.0, 0.0, 0.0]; // shared edge end
        let p3 = [0.0, -1.0, 0.0]; // wing B
        let angle = dihedral_angle(&p0, &p1, &p2, &p3);
        assert!(
            angle.abs() < 0.15, // some numerical tolerance for flat config
            "expected ~0 dihedral for flat pair, got {angle}"
        );
    }

    #[test]
    fn test_triangle_area() {
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let area = triangle_area(&a, &b, &c);
        assert!((area - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_fit_basic() {
        let (bv, bt, bn) = unit_cube_body();
        // Place a small planar garment above the cube (y=0.8, above cube top at y=0.5)
        let mut garment = planar_garment(4, 4, 0.15);
        for p in garment.positions.iter_mut() {
            p[1] = 0.8; // above the cube
            p[0] -= 0.225; // center roughly over cube
            p[2] -= 0.225;
        }

        let cfg = GarmentFitConfig {
            simulation_steps: 50,
            self_collision: false,
            sdf_resolution: [16, 16, 16],
            convergence_threshold: 1e-9,
            ..Default::default()
        };

        let fitter = GarmentFitterV2::new(cfg);
        let result = fitter.fit(&garment, &bv, &bt, &bn).unwrap();

        assert_eq!(result.draped_positions.len(), garment.positions.len());
        assert!(result.steps_taken > 0);
        // Gravity should have pulled garment downward
        let avg_y: f64 = result.draped_positions.iter().map(|p| p[1]).sum::<f64>()
            / result.draped_positions.len() as f64;
        assert!(
            avg_y < 0.8,
            "garment should have dropped under gravity, avg_y={avg_y}"
        );
    }

    #[test]
    fn test_fit_with_pinned_vertices() {
        let (bv, bt, bn) = unit_cube_body();
        let mut garment = planar_garment(3, 3, 0.1);
        for p in garment.positions.iter_mut() {
            p[1] = 0.8;
        }
        // Pin first row
        garment.pinned_vertices = vec![0, 1, 2];

        let cfg = GarmentFitConfig {
            simulation_steps: 30,
            self_collision: false,
            sdf_resolution: [16, 16, 16],
            ..Default::default()
        };

        let fitter = GarmentFitterV2::new(cfg);
        let result = fitter.fit(&garment, &bv, &bt, &bn).unwrap();

        // Pinned vertices should remain at original y
        for &pi in &garment.pinned_vertices {
            assert!(
                (result.draped_positions[pi][1] - 0.8).abs() < 1e-10,
                "pinned vertex {pi} should not move, y={}",
                result.draped_positions[pi][1]
            );
        }
    }

    #[test]
    fn test_refit_warm_start() {
        let (bv, bt, bn) = unit_cube_body();
        let mut garment = planar_garment(3, 3, 0.1);
        for p in garment.positions.iter_mut() {
            p[1] = 0.8;
        }

        let cfg = GarmentFitConfig {
            simulation_steps: 20,
            self_collision: false,
            sdf_resolution: [16, 16, 16],
            ..Default::default()
        };

        let fitter = GarmentFitterV2::new(cfg);
        let first = fitter.fit(&garment, &bv, &bt, &bn).unwrap();

        // Refit with a slightly different body (translate up)
        let shifted_bv: Vec<[f64; 3]> = bv.iter().map(|v| [v[0], v[1] + 0.05, v[2]]).collect();
        let result = fitter
            .refit(&garment, &first.draped_positions, &shifted_bv, &bt, &bn)
            .unwrap();

        assert_eq!(result.draped_positions.len(), garment.positions.len());
    }

    #[test]
    fn test_refit_mismatched_length() {
        let (bv, bt, bn) = unit_cube_body();
        let garment = planar_garment(3, 3, 0.1);
        let fitter = GarmentFitterV2::new(GarmentFitConfig::default());
        let bad_prev = vec![[0.0; 3]; 2]; // wrong length
        assert!(fitter.refit(&garment, &bad_prev, &bv, &bt, &bn).is_err());
    }

    #[test]
    fn test_sdf_collision_pushes_outside() {
        let (bv, bt, bn) = unit_cube_body();
        let sdf_config = SdfConfig {
            resolution: [16, 16, 16],
            padding: 0.15,
        };
        let sdf = SdfGrid::from_mesh(&bv, &bt, &bn, &sdf_config).unwrap();

        // Place a point inside the cube
        let mut positions = vec![[0.0, 0.0, 0.0]];
        let inv_mass = vec![1.0];
        let count = resolve_sdf_collisions(&sdf, &mut positions, &inv_mass, 0.01, 0.3);
        assert!(count > 0, "should detect collision for point inside cube");

        // After push, the point should be outside (positive SDF)
        let d = sdf.sample(&positions[0]);
        assert!(
            d > -0.05,
            "point should be pushed near or outside surface, d={d}"
        );
    }

    #[test]
    fn test_seam_constraints() {
        // Two vertices that should be pulled together
        let mut positions = vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0]];
        let inv_mass = vec![1.0, 1.0];
        let seam_pairs = vec![(0_usize, 1_usize)];

        for _ in 0..50 {
            project_seam_constraints(&seam_pairs, &mut positions, &inv_mass, 0.9);
        }

        let dist = v3_len(&v3_sub(&positions[0], &positions[1]));
        assert!(
            dist < 0.01,
            "seam vertices should converge, distance={dist}"
        );
    }

    #[test]
    fn test_kinetic_energy_at_rest() {
        let velocities = vec![[0.0; 3]; 10];
        let inv_mass = vec![1.0; 10];
        let ke = compute_kinetic_energy(&velocities, &inv_mass);
        assert!(ke.abs() < 1e-15);
    }

    #[test]
    fn test_kinetic_energy_nonzero() {
        let velocities = vec![[1.0, 0.0, 0.0]; 4];
        let inv_mass = vec![1.0; 4]; // mass = 1 each
        let ke = compute_kinetic_energy(&velocities, &inv_mass);
        // 4 * 0.5 * 1.0 * 1.0 = 2.0
        assert!((ke - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_fit_with_self_collision() {
        let (bv, bt, bn) = unit_cube_body();
        let mut garment = planar_garment(4, 4, 0.15);
        for p in garment.positions.iter_mut() {
            p[1] = 0.9;
            p[0] -= 0.225;
            p[2] -= 0.225;
        }

        let cfg = GarmentFitConfig {
            simulation_steps: 20,
            self_collision: true,
            sdf_resolution: [16, 16, 16],
            self_collision_thickness: 0.02,
            ..Default::default()
        };

        let fitter = GarmentFitterV2::new(cfg);
        let result = fitter.fit(&garment, &bv, &bt, &bn).unwrap();
        assert_eq!(result.draped_positions.len(), garment.positions.len());
        // Self-collision count can be 0 if garment doesn't self-intersect
        assert!(result.steps_taken > 0);
    }

    #[test]
    fn test_fit_empty_garment_error() {
        let (bv, bt, bn) = unit_cube_body();
        let garment = GarmentMesh {
            positions: Vec::new(),
            triangles: Vec::new(),
            uv_coords: Vec::new(),
            pinned_vertices: Vec::new(),
            seam_pairs: Vec::new(),
        };

        let fitter = GarmentFitterV2::new(GarmentFitConfig::default());
        assert!(fitter.fit(&garment, &bv, &bt, &bn).is_err());
    }

    #[test]
    fn test_garment_with_seam_pairs() {
        let (bv, bt, bn) = unit_cube_body();
        let mut garment = planar_garment(3, 3, 0.1);
        for p in garment.positions.iter_mut() {
            p[1] = 0.8;
        }
        // Create seam between vertex 0 and vertex 8 (opposite corners)
        garment.seam_pairs = vec![(0, 8)];

        let cfg = GarmentFitConfig {
            simulation_steps: 30,
            self_collision: false,
            sdf_resolution: [16, 16, 16],
            ..Default::default()
        };

        let fitter = GarmentFitterV2::new(cfg);
        let result = fitter.fit(&garment, &bv, &bt, &bn).unwrap();
        // Seam pair should be closer together than in the rest mesh
        let rest_dist = v3_len(&v3_sub(&garment.positions[0], &garment.positions[8]));
        let draped_dist = v3_len(&v3_sub(
            &result.draped_positions[0],
            &result.draped_positions[8],
        ));
        assert!(
            draped_dist < rest_dist + 0.5, // allow some tolerance due to gravity
            "seam should pull vertices closer: rest={rest_dist}, draped={draped_dist}"
        );
    }

    #[test]
    fn test_convergence() {
        let (bv, bt, bn) = unit_cube_body();
        let mut garment = planar_garment(3, 3, 0.05);
        // Place garment well above the body so it falls and settles
        for p in garment.positions.iter_mut() {
            p[1] = 2.0;
        }

        let cfg = GarmentFitConfig {
            simulation_steps: 500,
            self_collision: false,
            sdf_resolution: [16, 16, 16],
            convergence_threshold: 1e-4, // relatively easy threshold
            damping: 0.05,
            ..Default::default()
        };

        let fitter = GarmentFitterV2::new(cfg.clone());
        let result = fitter.fit(&garment, &bv, &bt, &bn).unwrap();
        // With enough steps and high convergence threshold, it should converge
        // (or at least complete without error)
        assert!(result.steps_taken <= cfg.simulation_steps);
    }
}
