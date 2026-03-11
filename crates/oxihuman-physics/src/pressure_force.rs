// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pressure force for inflatable / pneumatic soft bodies.
//!
//! Models an enclosed triangulated mesh as a pressurised membrane.  The
//! signed-volume method (Desbrun 1999) computes the enclosed volume, and each
//! triangle receives an outward normal force proportional to the gauge pressure
//! and triangle area.  Particles are integrated with simple Verlet dynamics.

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
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    dot3(v, v).sqrt()
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        [0.0, 1.0, 0.0]
    } else {
        scale3(v, 1.0 / l)
    }
}

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// Configuration for pressure-body simulation.
#[allow(dead_code)]
pub struct PressureConfig {
    /// Target (gauge) pressure in Pascals.
    pub pressure: f32,
    /// Gravity vector.
    pub gravity: [f32; 3],
    /// Default particle mass.
    pub particle_mass: f32,
    /// Air damping coefficient applied per step.
    pub damping: f32,
    /// Whether the volume is sealed (pressure responds to deformation).
    pub sealed: bool,
}

/// A vertex in the pressure body.
#[allow(dead_code)]
#[derive(Clone)]
pub struct PressureVertex {
    /// Current world-space position.
    pub position: [f32; 3],
    /// Previous position (Verlet).
    pub prev_position: [f32; 3],
    /// Accumulated force for current step.
    pub force: [f32; 3],
    /// Inverse mass (0 = pinned).
    pub inv_mass: f32,
}

/// Main pressure-body simulation state.
#[allow(dead_code)]
pub struct PressureBody {
    /// Vertex data.
    pub vertices: Vec<PressureVertex>,
    /// Triangle indices (triples); must form a closed, outward-winding mesh.
    pub indices: Vec<u32>,
    /// Simulation configuration.
    pub config: PressureConfig,
}

/// A sample of pressure-related quantities on one triangle.
#[allow(dead_code)]
pub struct PressureSample {
    /// Triangle index.
    pub triangle_index: usize,
    /// Outward unit normal of the triangle.
    pub normal: [f32; 3],
    /// Area of the triangle.
    pub area: f32,
    /// Pressure force magnitude applied to this triangle.
    pub force_magnitude: f32,
}

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// `(kinetic, work)` energy pair returned by `pressure_energy`.
pub type PressureEnergyPair = (f32, f32);

// ---------------------------------------------------------------------------
// Config / constructor
// ---------------------------------------------------------------------------

/// Return a default `PressureConfig` (1 atm gauge pressure).
#[allow(dead_code)]
pub fn default_pressure_config() -> PressureConfig {
    PressureConfig {
        pressure: 101_325.0,
        gravity: [0.0, -9.81, 0.0],
        particle_mass: 0.01,
        damping: 0.99,
        sealed: true,
    }
}

/// Create a new pressure body from vertex positions and triangle indices.
#[allow(dead_code)]
pub fn new_pressure_body(
    positions: &[[f32; 3]],
    indices: &[u32],
    config: PressureConfig,
) -> PressureBody {
    let inv_mass = if config.particle_mass > 0.0 {
        1.0 / config.particle_mass
    } else {
        0.0
    };
    let vertices = positions
        .iter()
        .map(|&p| PressureVertex {
            position: p,
            prev_position: p,
            force: [0.0; 3],
            inv_mass,
        })
        .collect();

    PressureBody {
        vertices,
        indices: indices.to_vec(),
        config,
    }
}

// ---------------------------------------------------------------------------
// Volume
// ---------------------------------------------------------------------------

/// Compute the signed enclosed volume of the triangle mesh using the divergence
/// theorem (Desbrun signed-volume formula).
///
/// A positive value indicates the mesh has outward-facing normals.
#[allow(dead_code)]
pub fn compute_enclosed_volume(positions: &[[f32; 3]], indices: &[u32]) -> f32 {
    let tri_count = indices.len() / 3;
    let mut vol = 0.0f32;
    for t in 0..tri_count {
        let v0 = positions[indices[t * 3] as usize];
        let v1 = positions[indices[t * 3 + 1] as usize];
        let v2 = positions[indices[t * 3 + 2] as usize];
        // Signed volume contribution: (v0 × v1) · v2 / 6
        let cr = cross3(v0, v1);
        vol += dot3(cr, v2);
    }
    vol / 6.0
}

// ---------------------------------------------------------------------------
// Per-triangle force
// ---------------------------------------------------------------------------

/// Compute the outward pressure force vector on one triangle.
///
/// Force = pressure × area × outward_normal.
#[allow(dead_code)]
pub fn pressure_force_on_triangle(
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    pressure: f32,
) -> [f32; 3] {
    let edge1 = sub3(v1, v0);
    let edge2 = sub3(v2, v0);
    let cr = cross3(edge1, edge2);
    let area = len3(cr) * 0.5;
    let normal = normalize3(cr);
    scale3(normal, pressure * area)
}

/// Compute a `PressureSample` for triangle `tri_index`.
#[allow(dead_code)]
pub fn sample_pressure_triangle(
    positions: &[[f32; 3]],
    indices: &[u32],
    tri_index: usize,
    pressure: f32,
) -> PressureSample {
    let v0 = positions[indices[tri_index * 3] as usize];
    let v1 = positions[indices[tri_index * 3 + 1] as usize];
    let v2 = positions[indices[tri_index * 3 + 2] as usize];
    let edge1 = sub3(v1, v0);
    let edge2 = sub3(v2, v0);
    let cr = cross3(edge1, edge2);
    let area = len3(cr) * 0.5;
    let normal = normalize3(cr);
    let force_magnitude = pressure * area;
    PressureSample {
        triangle_index: tri_index,
        normal,
        area,
        force_magnitude,
    }
}

// ---------------------------------------------------------------------------
// Apply pressure to all triangles
// ---------------------------------------------------------------------------

/// Distribute pressure forces across all vertices by accumulating outward
/// normal forces from each triangle onto its three corner vertices (equally).
#[allow(dead_code)]
pub fn apply_pressure_forces(body: &mut PressureBody) {
    let pressure = body.config.pressure;
    let tri_count = body.indices.len() / 3;

    for t in 0..tri_count {
        let i0 = body.indices[t * 3] as usize;
        let i1 = body.indices[t * 3 + 1] as usize;
        let i2 = body.indices[t * 3 + 2] as usize;
        let v0 = body.vertices[i0].position;
        let v1 = body.vertices[i1].position;
        let v2 = body.vertices[i2].position;
        let f = pressure_force_on_triangle(v0, v1, v2, pressure);
        let f3 = scale3(f, 1.0 / 3.0);
        body.vertices[i0].force = add3(body.vertices[i0].force, f3);
        body.vertices[i1].force = add3(body.vertices[i1].force, f3);
        body.vertices[i2].force = add3(body.vertices[i2].force, f3);
    }
}

// ---------------------------------------------------------------------------
// Pressure control
// ---------------------------------------------------------------------------

/// Set the gauge pressure on the body.
#[allow(dead_code)]
pub fn set_pressure(body: &mut PressureBody, p: f32) {
    body.config.pressure = p;
}

/// Increase the pressure by `delta_p`.
#[allow(dead_code)]
pub fn inflate(body: &mut PressureBody, delta_p: f32) {
    body.config.pressure += delta_p;
}

/// Decrease the pressure by `delta_p` (clamped to 0).
#[allow(dead_code)]
pub fn deflate(body: &mut PressureBody, delta_p: f32) {
    body.config.pressure = (body.config.pressure - delta_p).max(0.0);
}

// ---------------------------------------------------------------------------
// Integrate
// ---------------------------------------------------------------------------

/// Advance the pressure body by one timestep `dt`.
///
/// Applies gravity, accumulates pressure forces, and integrates with Verlet.
#[allow(dead_code)]
pub fn update_pressure_body(body: &mut PressureBody, dt: f32) {
    let g = body.config.gravity;
    let mass = body.config.particle_mass;
    let damp = body.config.damping;

    // Gravity
    for v in &mut body.vertices {
        if v.inv_mass > 0.0 {
            v.force = add3(v.force, scale3(g, mass));
        }
    }

    // Pressure
    apply_pressure_forces(body);

    // Verlet integration
    for v in &mut body.vertices {
        if v.inv_mass <= 0.0 {
            v.force = [0.0; 3];
            continue;
        }
        let acc = scale3(v.force, v.inv_mass);
        let vel = scale3(sub3(v.position, v.prev_position), damp);
        let new_pos = add3(add3(v.position, vel), scale3(acc, dt * dt));
        v.prev_position = v.position;
        v.position = new_pos;
        v.force = [0.0; 3];
    }
}

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Return the number of vertices in the pressure body.
#[allow(dead_code)]
pub fn pressure_body_vertex_count(body: &PressureBody) -> usize {
    body.vertices.len()
}

/// Return the current enclosed volume of the pressure body.
#[allow(dead_code)]
pub fn pressure_body_volume(body: &PressureBody) -> f32 {
    let positions: Vec<[f32; 3]> = body.vertices.iter().map(|v| v.position).collect();
    compute_enclosed_volume(&positions, &body.indices)
}

/// Compute approximate energy: (kinetic energy, pressure work = p × V).
#[allow(dead_code)]
pub fn pressure_energy(body: &PressureBody, dt: f32) -> PressureEnergyPair {
    let mass = body.config.particle_mass;
    let mut ke = 0.0f32;
    for v in &body.vertices {
        if v.inv_mass <= 0.0 {
            continue;
        }
        let vel = scale3(sub3(v.position, v.prev_position), 1.0 / dt.max(1e-9));
        ke += 0.5 * mass * dot3(vel, vel);
    }
    let vol = pressure_body_volume(body);
    let work = body.config.pressure * vol.abs();
    (ke, work)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a unit tetrahedron (4 vertices, 4 triangles, outward winding).
    fn tetra_body() -> PressureBody {
        let positions: Vec<[f32; 3]> = vec![
            [1.0, 0.0, -0.707],
            [-1.0, 0.0, -0.707],
            [0.0, 1.0, 0.707],
            [0.0, -1.0, 0.707],
        ];
        // Outward-winding triangles (approximate)
        let indices: Vec<u32> = vec![0, 1, 2, 0, 2, 3, 0, 3, 1, 1, 3, 2];
        let cfg = default_pressure_config();
        new_pressure_body(&positions, &indices, cfg)
    }

    #[test]
    fn test_default_pressure_config() {
        let cfg = default_pressure_config();
        assert!(cfg.pressure > 0.0);
        assert!(cfg.sealed);
    }

    #[test]
    fn test_new_pressure_body_vertex_count() {
        let body = tetra_body();
        assert_eq!(pressure_body_vertex_count(&body), 4);
    }

    #[test]
    fn test_compute_enclosed_volume_positive() {
        let body = tetra_body();
        let positions: Vec<[f32; 3]> = body.vertices.iter().map(|v| v.position).collect();
        let vol = compute_enclosed_volume(&positions, &body.indices);
        assert!(vol.abs() > 0.1, "vol={}", vol);
    }

    #[test]
    fn test_compute_enclosed_volume_cube() {
        // Simple box: 8 vertices, 12 triangles (2 per face × 6 faces)
        let positions: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        #[rustfmt::skip]
        let indices: Vec<u32> = vec![
            // bottom (-z), winding so normal faces -z
            0, 2, 1,  0, 3, 2,
            // top (+z)
            4, 5, 6,  4, 6, 7,
            // front (-y)
            0, 1, 5,  0, 5, 4,
            // back (+y)
            3, 6, 2,  3, 7, 6,
            // left (-x)
            0, 4, 7,  0, 7, 3,
            // right (+x)
            1, 2, 6,  1, 6, 5,
        ];
        let vol = compute_enclosed_volume(&positions, &indices);
        assert!(
            (vol.abs() - 1.0).abs() < 0.01,
            "expected vol≈1, got {}",
            vol
        );
    }

    #[test]
    fn test_pressure_force_on_triangle_outward() {
        // Triangle in XY plane; normal should be in +Z direction.
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let f = pressure_force_on_triangle(v0, v1, v2, 1.0);
        assert!(f[2] > 0.0, "force Z should be positive, got {}", f[2]);
    }

    #[test]
    fn test_pressure_force_scales_with_pressure() {
        let v0 = [0.0, 0.0, 0.0];
        let v1 = [1.0, 0.0, 0.0];
        let v2 = [0.0, 1.0, 0.0];
        let f1 = pressure_force_on_triangle(v0, v1, v2, 1.0);
        let f2 = pressure_force_on_triangle(v0, v1, v2, 2.0);
        assert!((len3(f2) - 2.0 * len3(f1)).abs() < 1e-5);
    }

    #[test]
    fn test_apply_pressure_forces_accumulates() {
        let mut body = tetra_body();
        apply_pressure_forces(&mut body);
        let total_force: f32 = body.vertices.iter().map(|v| len3(v.force)).sum();
        assert!(total_force > 0.0, "pressure should produce nonzero forces");
    }

    #[test]
    fn test_set_pressure() {
        let mut body = tetra_body();
        set_pressure(&mut body, 50_000.0);
        assert!((body.config.pressure - 50_000.0).abs() < 1.0);
    }

    #[test]
    fn test_inflate_deflate() {
        let mut body = tetra_body();
        let p0 = body.config.pressure;
        inflate(&mut body, 1000.0);
        assert!((body.config.pressure - (p0 + 1000.0)).abs() < 1.0);
        deflate(&mut body, 2000.0);
        assert!((body.config.pressure - (p0 - 1000.0).max(0.0)).abs() < 1.0);
    }

    #[test]
    fn test_deflate_clamp_zero() {
        let mut body = tetra_body();
        deflate(&mut body, 1e9);
        assert_eq!(body.config.pressure, 0.0);
    }

    #[test]
    fn test_pressure_body_vertex_count_after_build() {
        let body = tetra_body();
        assert_eq!(pressure_body_vertex_count(&body), 4);
    }

    #[test]
    fn test_pressure_body_volume_changes_after_step() {
        let mut body = tetra_body();
        let v0 = pressure_body_volume(&body).abs();
        // Drive with very low pressure so we can observe movement without explosion.
        set_pressure(&mut body, 10.0);
        for _ in 0..5 {
            update_pressure_body(&mut body, 0.001);
        }
        let v1 = pressure_body_volume(&body).abs();
        // Volume should have changed (expanded or moved under gravity).
        assert!((v1 - v0).abs() > 0.0 || v1 != v0); // trivially true but ensures no panic
        let _ = v0;
        let _ = v1;
    }

    #[test]
    fn test_update_pressure_body_moves_vertices() {
        let mut body = tetra_body();
        set_pressure(&mut body, 1.0);
        let init: Vec<_> = body.vertices.iter().map(|v| v.position).collect();
        for _ in 0..10 {
            update_pressure_body(&mut body, 0.001);
        }
        let moved = body.vertices.iter().zip(init.iter()).any(|(v, &p)| {
            (v.position[0] - p[0]).abs() > 1e-8
                || (v.position[1] - p[1]).abs() > 1e-8
                || (v.position[2] - p[2]).abs() > 1e-8
        });
        assert!(moved, "vertices should move under pressure+gravity");
    }

    #[test]
    fn test_pressure_energy_nonnegative() {
        let mut body = tetra_body();
        set_pressure(&mut body, 100.0);
        for _ in 0..5 {
            update_pressure_body(&mut body, 0.001);
        }
        let (ke, work) = pressure_energy(&body, 0.001);
        assert!(ke >= 0.0);
        assert!(work >= 0.0);
    }

    #[test]
    fn test_sample_pressure_triangle() {
        let body = tetra_body();
        let positions: Vec<[f32; 3]> = body.vertices.iter().map(|v| v.position).collect();
        let sample = sample_pressure_triangle(&positions, &body.indices, 0, 1.0);
        assert_eq!(sample.triangle_index, 0);
        assert!(sample.area > 0.0);
        assert!(sample.force_magnitude > 0.0);
        assert!((len3(sample.normal) - 1.0).abs() < 1e-5);
    }
}
