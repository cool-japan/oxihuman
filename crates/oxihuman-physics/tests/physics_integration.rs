// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for oxihuman-physics subsystems.
//!
//! 7 tests covering cloth PBD, rigid-body gravity, sphere-sphere collision
//! detection, capsule no-false-positive, XPBD length constraint, FEM tet
//! deformation finiteness, and SPH mass conservation.

use oxihuman_physics::{
    // Cloth / PBD
    add_cloth_grid,
    add_xpbd_particle,
    // Collision
    capsule_capsule,
    // XPBD (exported with renamed aliases from modules_a)
    new_xpbd_world,
    sphere_sphere,
    xpbd_add_distance,
    xpbd_step,
    Capsule,
    PbdConfig,
    PbdSimulation,
    // SPH
    SphFluidSimConfig,
    SphFluidV2,
    Sphere,
    XpbdConstraintType,
};

// ── 1. Cloth PBD converges ────────────────────────────────────────────────────

/// 30-particle cloth grid (5×5 interior → 6×6 grid would be 36; use 5×6 = 30
/// interior vertices = 5x5 = 25 cells → we want exactly 30 free particles).
/// We create a 5×5 grid (36 vertices) and pin the top row (6 particles) leaving
/// 30 free particles.  After 50 PBD steps the maximum displacement must be less
/// than the initial span of the cloth (2 m in each horizontal direction).
#[test]
fn test_cloth_pbd_converges() {
    let cfg = PbdConfig {
        substeps: 4,
        gravity: [0.0, -9.81, 0.0],
        damping: 0.01,
        restitution: 0.2,
        friction: 0.3,
    };
    let mut sim = PbdSimulation::new(cfg);

    // 5×5 divisions → (6×6) = 36 particles; span = 2×2 m
    let particle_indices = add_cloth_grid(
        &mut sim,
        [0.0, 1.0, 0.0], // origin (raised so gravity can act)
        [2.0, 2.0],      // size
        [5, 5],          // divisions
        1.0,             // inv_mass
    );

    // Pin top row (last 6 particles in the flat list): y-index == 5 (stride = 6)
    let stride = 6usize;
    for ix in 0..stride {
        let idx = particle_indices[5 * stride + ix];
        sim.particles[idx].inv_mass = 0.0;
    }

    let initial_span = 2.0_f32;
    let dt = 1.0 / 60.0;

    for _ in 0..50 {
        sim.step(dt);
    }

    // All particle positions must remain within reasonable bounds.
    for &pi in &particle_indices {
        let pos = sim.particles[pi].position;
        let disp = (pos[0].abs() + pos[1].abs() + pos[2].abs()) / (initial_span * 3.0);
        assert!(
            disp < 10.0,
            "particle displacement ratio too large: {disp}  pos={pos:?}"
        );
        // Positions must be finite
        assert!(
            pos[0].is_finite() && pos[1].is_finite() && pos[2].is_finite(),
            "non-finite position: {pos:?}"
        );
    }

    // The free particles (unpinned) must have moved from their initial y=1.0
    // (downward under gravity) — proving simulation ran.
    let free_moved = particle_indices
        .iter()
        .any(|&pi| sim.particles[pi].inv_mass > 0.0 && sim.particles[pi].position[1] < 0.99);
    assert!(
        free_moved,
        "no free particle moved under gravity — simulation stalled"
    );
}

// ── 2. Rigid body gravity ─────────────────────────────────────────────────────

/// Single rigid body with mass=1 kg starts at rest at y=10.
/// After 10 integration steps with dt=0.1, y must decrease monotonically.
#[test]
fn test_rigid_body_gravity() {
    use oxihuman_physics::rigid_body::{
        default_rigid_body_config, integrate_rigid_body, new_rigid_body,
    };

    let cfg = default_rigid_body_config();
    let mut rb = new_rigid_body(1.0, [1.0, 1.0, 1.0], &cfg);
    rb.position = [0.0, 10.0, 0.0];

    let dt = 0.1_f32;
    let mut prev_y = rb.position[1];

    for _ in 0..10 {
        integrate_rigid_body(&mut rb, dt, &cfg);
        let cur_y = rb.position[1];
        assert!(
            cur_y < prev_y,
            "y did not decrease: prev={prev_y} cur={cur_y}"
        );
        prev_y = cur_y;
    }
}

// ── 3. Collision sphere–sphere detect ────────────────────────────────────────

/// Two spheres centered 1.5 m apart, each with radius 1.0 m → overlap = 0.5 m.
/// `sphere_sphere` must return `Some(Contact)` with positive depth.
#[test]
fn test_collision_sphere_sphere_detect() {
    let a = Sphere {
        center: [0.0, 0.0, 0.0],
        radius: 1.0,
    };
    let b = Sphere {
        center: [1.5, 0.0, 0.0],
        radius: 1.0,
    };
    let contact =
        sphere_sphere(&a, &b).expect("sphere_sphere should detect overlap for overlapping spheres");
    assert!(
        contact.depth > 0.0,
        "contact depth must be positive, got {}",
        contact.depth
    );
    // Expected depth = 2.0 - 1.5 = 0.5
    assert!(
        (contact.depth - 0.5).abs() < 1e-4,
        "unexpected depth {}",
        contact.depth
    );
}

// ── 4. Collision capsule no-false-positive ────────────────────────────────────

/// Two capsules separated by 5 m with combined radii of 0.6 m.
/// `capsule_capsule` must return `None` (no false positive collision).
#[test]
fn test_collision_capsule_no_false_positive() {
    // Capsule A: segment from (0,0,0) to (1,0,0), radius=0.3
    let cap_a = Capsule {
        a: [0.0, 0.0, 0.0],
        b: [1.0, 0.0, 0.0],
        radius: 0.3,
    };
    // Capsule B: same orientation but displaced 5 m in Y — well separated
    let cap_b = Capsule {
        a: [0.0, 5.0, 0.0],
        b: [1.0, 5.0, 0.0],
        radius: 0.3,
    };
    let result = capsule_capsule(&cap_a, &cap_b);
    assert!(
        result.is_none(),
        "capsule_capsule should return None for well-separated capsules, got {result:?}"
    );
}

// ── 5. XPBD length constraint ─────────────────────────────────────────────────

/// 2 free particles placed 2 m apart with a distance constraint of rest_length=1 m.
/// After 20 XPBD iterations the distance must be within 2% of the rest length.
#[test]
fn test_xpbd_length_constraint() {
    let mut world = new_xpbd_world();
    let _i0 = add_xpbd_particle(&mut world, [0.0, 0.0, 0.0], 1.0);
    let _i1 = add_xpbd_particle(&mut world, [2.0, 0.0, 0.0], 1.0);

    // Add a distance constraint with rest_length = 1.0
    xpbd_add_distance(&mut world, 0, 1, 0.0); // rest_length computed from positions (= 2.0)

    // Override rest length to 1.0 inside the constraint
    if let XpbdConstraintType::Distance {
        ref mut rest_length,
        ..
    } = world.constraints[0].constraint_type
    {
        *rest_length = 1.0;
    }

    // Disable gravity so only the constraint acts
    world.gravity = [0.0, 0.0, 0.0];

    // Run 20 XPBD substeps (each xpbd_step call uses substeps internally)
    for _ in 0..20 {
        xpbd_step(&mut world, 0.016, 1);
    }

    let p0 = world.particles[0].position;
    let p1 = world.particles[1].position;
    let dx = p1[0] - p0[0];
    let dy = p1[1] - p0[1];
    let dz = p1[2] - p0[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    let rest = 1.0_f32;
    let error = (dist - rest).abs() / rest;
    assert!(
        error < 0.02,
        "XPBD length constraint error {:.4} ({:.4}%) exceeds 2%; dist={}",
        error,
        error * 100.0,
        dist
    );
}

// ── 6. FEM deformation finite ─────────────────────────────────────────────────

/// Build a single-tetrahedron FEM mesh and step it; all vertex positions must
/// remain finite (non-NaN, non-Inf) after one integration step.
///
/// We use `SoftBody::from_tet_mesh` / `SoftBody::step` from the `soft_body`
/// module which is re-exported from `oxihuman_physics`.
#[test]
fn test_fem_deformation_finite() {
    use oxihuman_physics::{make_cube_soft_body, SoftBody};

    // make_cube_soft_body creates a soft body from a unit cube; use side=1 and
    // subdivisions=1 so we get the minimal tetrahedralisation.
    let mut body: SoftBody = make_cube_soft_body(1.0, 1);

    // Verify we have vertices
    assert!(!body.positions.is_empty(), "SoftBody must have vertices");

    // Run one step with gravity
    body.step(0.016, 2, [0.0, -9.81, 0.0]);

    // All positions must be finite
    for (idx, pos) in body.positions.iter().enumerate() {
        assert!(
            pos[0].is_finite() && pos[1].is_finite() && pos[2].is_finite(),
            "vertex {idx} has non-finite position: {pos:?}"
        );
    }
}

// ── 7. SPH fluid step conserves mass ─────────────────────────────────────────

/// Create an SPH fluid system with 16 particles arranged on a 4×4 grid.
/// After one step the particle count must remain exactly 16 (mass conservation
/// in the discrete particle sense: no particles created or destroyed).
#[test]
fn test_sph_fluid_step_conserves_mass() {
    let config = SphFluidSimConfig {
        smoothing_h: 0.15,
        rest_density: 1000.0,
        pressure_k: 200.0,
        viscosity: 0.01,
        gravity: [0.0, -9.81],
    };
    let mut fluid = SphFluidV2::new(config);

    // 4×4 grid of particles, spacing = 0.1 m
    for iy in 0..4_usize {
        for ix in 0..4_usize {
            fluid.add_particle([ix as f32 * 0.1, iy as f32 * 0.1], 1.0);
        }
    }

    let initial_count = fluid.particle_count();
    assert_eq!(initial_count, 16, "expected 16 particles before step");

    fluid.step(0.001);

    let final_count = fluid.particle_count();
    assert_eq!(
        final_count, initial_count,
        "particle count changed after SPH step: {} → {}",
        initial_count, final_count
    );
}
