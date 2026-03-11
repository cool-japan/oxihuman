// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Rope-cloth hybrid simulation.
//!
//! Combines 1-D rope segments and 2-D cloth quads under a unified Position-Based
//! Dynamics (PBD) framework.  Particles share the same pool; rope segments add
//! stretch constraints along single edges while cloth quads add structural,
//! shear, and (optionally) bending constraints over a quad face.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Math helpers
// ---------------------------------------------------------------------------

#[inline]
fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Configuration for the rope-cloth simulation.
#[allow(dead_code)]
pub struct RopeClothConfig {
    /// Gravity vector applied each step.
    pub gravity: [f32; 3],
    /// Default mass of each particle.
    pub particle_mass: f32,
    /// PBD solver iterations per timestep.
    pub solver_iterations: u32,
    /// Rope stretch stiffness (0–1).
    pub rope_stiffness: f32,
    /// Cloth structural stiffness (0–1).
    pub cloth_stiffness: f32,
    /// Cloth shear stiffness (0–1).
    pub cloth_shear_stiffness: f32,
    /// Air damping coefficient.
    pub damping: f32,
}

/// A single particle in the unified rope-cloth system.
#[allow(dead_code)]
#[derive(Clone)]
pub struct RopeClothParticle {
    /// Current world-space position.
    pub position: [f32; 3],
    /// Previous position (Verlet).
    pub prev_position: [f32; 3],
    /// Inverse mass (0 = pinned/static).
    pub inv_mass: f32,
    /// Accumulated external force for the current step.
    pub force: [f32; 3],
}

/// A distance constraint between two particles.
#[allow(dead_code)]
#[derive(Clone)]
pub struct RopeClothConstraint {
    /// First particle index.
    pub p0: usize,
    /// Second particle index.
    pub p1: usize,
    /// Rest length.
    pub rest_length: f32,
    /// Compliance (PBD stiffness); lower = stiffer.
    pub compliance: f32,
}

/// Indicates whether a constraint originates from a rope segment or cloth quad.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConstraintKind {
    /// Stretch constraint along a rope segment.
    Rope,
    /// Structural, shear, or bending constraint within a cloth quad.
    Cloth,
}

/// Annotated constraint for the hybrid body.
#[allow(dead_code)]
#[derive(Clone)]
pub struct AnnotatedConstraint {
    /// The underlying distance constraint.
    pub constraint: RopeClothConstraint,
    /// Whether this comes from a rope or cloth element.
    pub kind: ConstraintKind,
}

/// A rope segment (a span of consecutive particle indices with constraints).
#[allow(dead_code)]
#[derive(Clone)]
pub struct RopeSegmentRecord {
    /// Inclusive start particle index.
    pub start: usize,
    /// Inclusive end particle index.
    pub end: usize,
}

/// A cloth quad (four particle indices, CCW order).
#[allow(dead_code)]
#[derive(Clone)]
pub struct ClothQuadRecord {
    /// Four corner particle indices: (i0, i1, i2, i3) in CCW order.
    pub indices: [usize; 4],
}

/// The main hybrid rope-cloth simulation body.
#[allow(dead_code)]
pub struct RopeClothBody {
    /// All particles.
    pub particles: Vec<RopeClothParticle>,
    /// All distance constraints (rope + cloth).
    pub constraints: Vec<AnnotatedConstraint>,
    /// Metadata for each rope segment added.
    pub rope_segments: Vec<RopeSegmentRecord>,
    /// Metadata for each cloth quad added.
    pub cloth_quads: Vec<ClothQuadRecord>,
    /// Simulation configuration.
    pub config: RopeClothConfig,
}

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Result of energy query: (kinetic, potential) pair.
pub type EnergyPair = (f32, f32);

// ---------------------------------------------------------------------------
// Config / constructor
// ---------------------------------------------------------------------------

/// Return a default `RopeClothConfig`.
#[allow(dead_code)]
pub fn default_rope_cloth_config() -> RopeClothConfig {
    RopeClothConfig {
        gravity: [0.0, -9.81, 0.0],
        particle_mass: 1.0,
        solver_iterations: 8,
        rope_stiffness: 0.95,
        cloth_stiffness: 0.85,
        cloth_shear_stiffness: 0.5,
        damping: 0.99,
    }
}

/// Create a new empty `RopeClothBody` with the given configuration.
#[allow(dead_code)]
pub fn new_rope_cloth_body(config: RopeClothConfig) -> RopeClothBody {
    RopeClothBody {
        particles: Vec::new(),
        constraints: Vec::new(),
        rope_segments: Vec::new(),
        cloth_quads: Vec::new(),
        config,
    }
}

// ---------------------------------------------------------------------------
// Particle helper
// ---------------------------------------------------------------------------

fn new_particle(pos: [f32; 3], inv_mass: f32) -> RopeClothParticle {
    RopeClothParticle {
        position: pos,
        prev_position: pos,
        inv_mass,
        force: [0.0; 3],
    }
}

fn make_constraint(
    p0: usize,
    p1: usize,
    positions: &[[f32; 3]],
    compliance: f32,
    kind: ConstraintKind,
) -> AnnotatedConstraint {
    let d = sub3(positions[p1], positions[p0]);
    AnnotatedConstraint {
        constraint: RopeClothConstraint {
            p0,
            p1,
            rest_length: len3(d),
            compliance,
        },
        kind,
    }
}

// ---------------------------------------------------------------------------
// Rope segment
// ---------------------------------------------------------------------------

/// Add a rope segment consisting of `count` new particles starting at `start_pos`
/// and ending at `end_pos`.  Particles are equally spaced along the segment.
/// Returns the range `[start_idx, end_idx]` of newly created particles.
#[allow(dead_code)]
pub fn add_rope_segment(
    body: &mut RopeClothBody,
    start_pos: [f32; 3],
    end_pos: [f32; 3],
    count: usize,
) -> (usize, usize) {
    let n = count.max(2);
    let start_idx = body.particles.len();
    let inv_mass = if body.config.particle_mass > 0.0 {
        1.0 / body.config.particle_mass
    } else {
        0.0
    };

    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let pos = add3(scale3(start_pos, 1.0 - t), scale3(end_pos, t));
        body.particles.push(new_particle(pos, inv_mass));
    }

    let compliance = 1.0 - body.config.rope_stiffness.clamp(0.0, 1.0);
    for i in start_idx..(start_idx + n - 1) {
        let positions: Vec<[f32; 3]> = body.particles.iter().map(|p| p.position).collect();
        let c = make_constraint(i, i + 1, &positions, compliance, ConstraintKind::Rope);
        body.constraints.push(c);
    }

    let end_idx = start_idx + n - 1;
    body.rope_segments.push(RopeSegmentRecord {
        start: start_idx,
        end: end_idx,
    });
    (start_idx, end_idx)
}

// ---------------------------------------------------------------------------
// Cloth quad
// ---------------------------------------------------------------------------

/// Add a cloth quad defined by four existing particle indices `[i0, i1, i2, i3]`
/// (CCW winding).  Structural (edges) and shear (diagonals) constraints are added.
#[allow(dead_code)]
pub fn add_cloth_quad(body: &mut RopeClothBody, indices: [usize; 4]) {
    let struct_compliance = 1.0 - body.config.cloth_stiffness.clamp(0.0, 1.0);
    let shear_compliance = 1.0 - body.config.cloth_shear_stiffness.clamp(0.0, 1.0);

    let positions: Vec<[f32; 3]> = body.particles.iter().map(|p| p.position).collect();

    // 4 structural edges
    let edges = [(0, 1), (1, 2), (2, 3), (3, 0)];
    for (a, b) in edges {
        let c = make_constraint(
            indices[a],
            indices[b],
            &positions,
            struct_compliance,
            ConstraintKind::Cloth,
        );
        body.constraints.push(c);
    }
    // 2 shear diagonals
    let diags = [(0, 2), (1, 3)];
    for (a, b) in diags {
        let c = make_constraint(
            indices[a],
            indices[b],
            &positions,
            shear_compliance,
            ConstraintKind::Cloth,
        );
        body.constraints.push(c);
    }

    body.cloth_quads.push(ClothQuadRecord { indices });
}

// ---------------------------------------------------------------------------
// Counting
// ---------------------------------------------------------------------------

/// Return the total number of particles.
#[allow(dead_code)]
pub fn rope_cloth_particle_count(body: &RopeClothBody) -> usize {
    body.particles.len()
}

/// Return the number of rope segments.
#[allow(dead_code)]
pub fn rope_segment_count(body: &RopeClothBody) -> usize {
    body.rope_segments.len()
}

/// Return the number of cloth quads.
#[allow(dead_code)]
pub fn cloth_quad_count(body: &RopeClothBody) -> usize {
    body.cloth_quads.len()
}

// ---------------------------------------------------------------------------
// Pinning
// ---------------------------------------------------------------------------

/// Pin particle `idx` (set inv_mass to 0 so it cannot move).
#[allow(dead_code)]
pub fn pin_rope_cloth_particle(body: &mut RopeClothBody, idx: usize) {
    if let Some(p) = body.particles.get_mut(idx) {
        p.inv_mass = 0.0;
    }
}

/// Unpin particle `idx`, restoring the configured particle mass.
#[allow(dead_code)]
pub fn unpin_rope_cloth_particle(body: &mut RopeClothBody, idx: usize) {
    if let Some(p) = body.particles.get_mut(idx) {
        let mass = body.config.particle_mass;
        p.inv_mass = if mass > 0.0 { 1.0 / mass } else { 0.0 };
    }
}

// ---------------------------------------------------------------------------
// Gravity
// ---------------------------------------------------------------------------

/// Accumulate gravity force on all unpinned particles.
#[allow(dead_code)]
pub fn apply_gravity_rope_cloth(body: &mut RopeClothBody) {
    let g = body.config.gravity;
    let mass = body.config.particle_mass;
    for p in &mut body.particles {
        if p.inv_mass > 0.0 {
            p.force = add3(p.force, scale3(g, mass));
        }
    }
}

// ---------------------------------------------------------------------------
// PBD update step
// ---------------------------------------------------------------------------

/// Advance the simulation by one timestep `dt` using PBD.
///
/// 1. Apply external forces + gravity via Verlet integration.
/// 2. Solve distance constraints for `solver_iterations` passes.
/// 3. Apply velocity damping.
#[allow(dead_code)]
pub fn update_rope_cloth(body: &mut RopeClothBody, dt: f32) {
    let g = body.config.gravity;
    let damp = body.config.damping;
    let iters = body.config.solver_iterations;

    // Step 1: Verlet integrate + apply accumulated external forces.
    for p in &mut body.particles {
        if p.inv_mass <= 0.0 {
            p.force = [0.0; 3];
            continue;
        }
        let acc = add3(g, scale3(p.force, p.inv_mass));
        let vel = scale3(sub3(p.position, p.prev_position), damp);
        let new_pos = add3(add3(p.position, vel), scale3(acc, dt * dt));
        p.prev_position = p.position;
        p.position = new_pos;
        p.force = [0.0; 3];
    }

    // Step 2: PBD constraint projection.
    for _ in 0..iters {
        for ac in &body.constraints {
            let c = &ac.constraint;
            let p0 = &body.particles[c.p0];
            let p1 = &body.particles[c.p1];
            let w0 = p0.inv_mass;
            let w1 = p1.inv_mass;
            let w_sum = w0 + w1;
            if w_sum <= 0.0 {
                continue;
            }
            let diff = sub3(p1.position, p0.position);
            let cur_len = len3(diff);
            if cur_len < 1e-12 {
                continue;
            }
            let dir = scale3(diff, 1.0 / cur_len);
            let error = cur_len - c.rest_length;
            // Compliance correction (XPBD-like: alpha = compliance / dt^2)
            let alpha = c.compliance / (dt * dt);
            let delta = -error / (w_sum + alpha);
            let d0 = scale3(dir, -delta * w0);
            let d1 = scale3(dir, delta * w1);
            body.particles[c.p0].position = add3(body.particles[c.p0].position, d0);
            body.particles[c.p1].position = add3(body.particles[c.p1].position, d1);
        }
    }
}

// ---------------------------------------------------------------------------
// Energy
// ---------------------------------------------------------------------------

/// Compute approximate (kinetic, elastic potential) energies of the body.
///
/// Kinetic energy uses Verlet velocity estimate.  Potential = sum of spring
/// elastic energy (½ k x²) where stiffness k = 1 / compliance.
#[allow(dead_code)]
pub fn rope_cloth_energy(body: &RopeClothBody, dt: f32) -> EnergyPair {
    let mass = body.config.particle_mass;
    let mut kinetic = 0.0f32;
    for p in &body.particles {
        if p.inv_mass <= 0.0 {
            continue;
        }
        let vel = scale3(sub3(p.position, p.prev_position), 1.0 / dt.max(1e-9));
        kinetic += 0.5 * mass * dot3(vel, vel);
    }
    let mut potential = 0.0f32;
    for ac in &body.constraints {
        let c = &ac.constraint;
        let d = sub3(body.particles[c.p1].position, body.particles[c.p0].position);
        let cur = len3(d);
        let stretch = cur - c.rest_length;
        let k = if c.compliance > 1e-12 {
            1.0 / c.compliance
        } else {
            1e6
        };
        potential += 0.5 * k * stretch * stretch;
    }
    (kinetic, potential)
}

// ---------------------------------------------------------------------------
// Reset
// ---------------------------------------------------------------------------

/// Reset all particles to their rest positions (prev = current), clear forces.
#[allow(dead_code)]
pub fn reset_rope_cloth(body: &mut RopeClothBody) {
    for p in &mut body.particles {
        p.prev_position = p.position;
        p.force = [0.0; 3];
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_body() -> RopeClothBody {
        let cfg = default_rope_cloth_config();
        new_rope_cloth_body(cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_rope_cloth_config();
        assert_eq!(cfg.solver_iterations, 8);
        assert!((cfg.gravity[1] + 9.81).abs() < 1e-4);
    }

    #[test]
    fn test_new_body_empty() {
        let body = simple_body();
        assert_eq!(body.particles.len(), 0);
        assert_eq!(body.constraints.len(), 0);
    }

    #[test]
    fn test_add_rope_segment_count() {
        let mut body = simple_body();
        let (s, e) = add_rope_segment(&mut body, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 5);
        assert_eq!(s, 0);
        assert_eq!(e, 4);
        assert_eq!(rope_cloth_particle_count(&body), 5);
    }

    #[test]
    fn test_add_rope_segment_constraints() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0, 0.0, 0.0], [4.0, 0.0, 0.0], 5);
        // 4 stretch constraints for 5 particles
        let rope_constraints = body
            .constraints
            .iter()
            .filter(|ac| ac.kind == ConstraintKind::Rope)
            .count();
        assert_eq!(rope_constraints, 4);
    }

    #[test]
    fn test_rope_segment_count() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0; 3], [1.0, 0.0, 0.0], 3);
        add_rope_segment(&mut body, [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], 3);
        assert_eq!(rope_segment_count(&body), 2);
    }

    #[test]
    fn test_add_cloth_quad() {
        let mut body = simple_body();
        // Create 4 particles manually
        let cfg = default_rope_cloth_config();
        let inv_mass = 1.0 / cfg.particle_mass;
        let positions = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        for pos in positions {
            body.particles.push(RopeClothParticle {
                position: pos,
                prev_position: pos,
                inv_mass,
                force: [0.0; 3],
            });
        }
        add_cloth_quad(&mut body, [0, 1, 2, 3]);
        assert_eq!(cloth_quad_count(&body), 1);
        // 4 edges + 2 diagonals = 6 cloth constraints
        let cloth_count = body
            .constraints
            .iter()
            .filter(|ac| ac.kind == ConstraintKind::Cloth)
            .count();
        assert_eq!(cloth_count, 6);
    }

    #[test]
    fn test_cloth_quad_count() {
        let mut body = simple_body();
        let inv_mass = 1.0;
        for i in 0..8 {
            let pos = [i as f32, 0.0, 0.0];
            body.particles.push(RopeClothParticle {
                position: pos,
                prev_position: pos,
                inv_mass,
                force: [0.0; 3],
            });
        }
        add_cloth_quad(&mut body, [0, 1, 2, 3]);
        add_cloth_quad(&mut body, [4, 5, 6, 7]);
        assert_eq!(cloth_quad_count(&body), 2);
    }

    #[test]
    fn test_pin_unpin() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0; 3], [1.0, 0.0, 0.0], 3);
        pin_rope_cloth_particle(&mut body, 0);
        assert_eq!(body.particles[0].inv_mass, 0.0);
        unpin_rope_cloth_particle(&mut body, 0);
        assert!(body.particles[0].inv_mass > 0.0);
    }

    #[test]
    fn test_apply_gravity_pinned_unchanged() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0; 3], [1.0, 0.0, 0.0], 3);
        pin_rope_cloth_particle(&mut body, 0);
        apply_gravity_rope_cloth(&mut body);
        assert_eq!(body.particles[0].force, [0.0; 3]);
    }

    #[test]
    fn test_apply_gravity_free_particle() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0; 3], [1.0, 0.0, 0.0], 3);
        apply_gravity_rope_cloth(&mut body);
        // Free particle should have a downward force component
        assert!(body.particles[1].force[1] < 0.0);
    }

    #[test]
    fn test_update_rope_cloth_drops() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0, 1.0, 0.0], [1.0, 1.0, 0.0], 3);
        // Pin ends so middle particle can drop
        pin_rope_cloth_particle(&mut body, 0);
        pin_rope_cloth_particle(&mut body, 2);
        let init_y = body.particles[1].position[1];
        for _ in 0..10 {
            update_rope_cloth(&mut body, 0.01);
        }
        // Middle particle should have dropped
        assert!(
            body.particles[1].position[1] < init_y,
            "y={}",
            body.particles[1].position[1]
        );
    }

    #[test]
    fn test_pinned_particle_does_not_move() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 3);
        pin_rope_cloth_particle(&mut body, 0);
        let start_pos = body.particles[0].position;
        for _ in 0..20 {
            update_rope_cloth(&mut body, 0.01);
        }
        let end_pos = body.particles[0].position;
        assert!((end_pos[0] - start_pos[0]).abs() < 1e-6);
        assert!((end_pos[1] - start_pos[1]).abs() < 1e-6);
    }

    #[test]
    fn test_rope_cloth_energy_positive() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0, 5.0, 0.0], [1.0, 5.0, 0.0], 4);
        pin_rope_cloth_particle(&mut body, 0);
        for _ in 0..5 {
            update_rope_cloth(&mut body, 0.01);
        }
        let (ke, pe) = rope_cloth_energy(&body, 0.01);
        assert!(ke >= 0.0);
        assert!(pe >= 0.0);
    }

    #[test]
    fn test_reset_rope_cloth() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0; 3], [1.0, 0.0, 0.0], 3);
        for _ in 0..10 {
            update_rope_cloth(&mut body, 0.01);
        }
        let moved_pos = body.particles[1].position;
        reset_rope_cloth(&mut body);
        // After reset, prev_position = current position (no velocity).
        assert_eq!(body.particles[1].prev_position, moved_pos);
        assert_eq!(body.particles[1].force, [0.0; 3]);
    }

    #[test]
    fn test_rope_cloth_particle_count_combined() {
        let mut body = simple_body();
        add_rope_segment(&mut body, [0.0; 3], [1.0, 0.0, 0.0], 3);
        // Add extra particles for cloth
        let inv_mass = 1.0;
        for i in 3..7 {
            let pos = [i as f32, 0.0, 0.0];
            body.particles.push(RopeClothParticle {
                position: pos,
                prev_position: pos,
                inv_mass,
                force: [0.0; 3],
            });
        }
        add_cloth_quad(&mut body, [3, 4, 5, 6]);
        assert_eq!(rope_cloth_particle_count(&body), 7);
    }
}
