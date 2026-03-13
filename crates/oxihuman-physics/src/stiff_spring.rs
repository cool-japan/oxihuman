// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Stiff spring constraint solver using implicit integration.

#![allow(dead_code)]

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Configuration for a stiff spring.
pub struct StiffSpringConfig {
    /// Natural (rest) length of the spring.
    pub rest_length: f32,
    /// Spring stiffness coefficient (N/m).
    pub stiffness: f32,
    /// Damping coefficient.
    pub damping: f32,
    /// Maximum allowable force magnitude (clamped).
    pub max_force: f32,
}

/// One endpoint of a spring (a particle).
pub struct SpringEndpoint {
    /// World-space position.
    pub position: [f32; 3],
    /// Velocity.
    pub velocity: [f32; 3],
    /// Mass in kg.
    pub mass: f32,
    /// If `true`, this endpoint is pinned and does not move.
    pub fixed: bool,
}

/// Forces and metrics output from a spring force calculation.
pub struct StiffSpringResult {
    /// Force applied to endpoint A.
    pub force_a: [f32; 3],
    /// Force applied to endpoint B (equal and opposite to `force_a`).
    pub force_b: [f32; 3],
    /// Current spring length.
    pub spring_length: f32,
    /// Extension (positive = stretched, negative = compressed).
    pub extension: f32,
}

// ---------------------------------------------------------------------------
// Internal math helpers
// ---------------------------------------------------------------------------

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn neg3(v: [f32; 3]) -> [f32; 3] {
    [-v[0], -v[1], -v[2]]
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Create a `StiffSpringConfig` with rest length and stiffness; damping=1, max_force=1e6.
#[allow(dead_code)]
pub fn default_stiff_spring_config(rest_length: f32, stiffness: f32) -> StiffSpringConfig {
    StiffSpringConfig {
        rest_length,
        stiffness,
        damping: 1.0,
        max_force: 1.0e6,
    }
}

/// Create a free (non-fixed) spring endpoint at the given position with zero velocity.
#[allow(dead_code)]
pub fn new_spring_endpoint(pos: [f32; 3], mass: f32) -> SpringEndpoint {
    SpringEndpoint {
        position: pos,
        velocity: [0.0; 3],
        mass,
        fixed: false,
    }
}

/// Compute the current distance between two spring endpoints.
#[allow(dead_code)]
pub fn spring_length(a: &SpringEndpoint, b: &SpringEndpoint) -> f32 {
    len3(sub3(b.position, a.position))
}

/// Compute how much the spring is stretched beyond (or compressed below) its rest length.
#[allow(dead_code)]
pub fn spring_extension(a: &SpringEndpoint, b: &SpringEndpoint, cfg: &StiffSpringConfig) -> f32 {
    spring_length(a, b) - cfg.rest_length
}

/// Compute the elastic potential energy stored in the spring.
#[allow(dead_code)]
pub fn spring_potential_energy(
    a: &SpringEndpoint,
    b: &SpringEndpoint,
    cfg: &StiffSpringConfig,
) -> f32 {
    let ext = spring_extension(a, b, cfg);
    0.5 * cfg.stiffness * ext * ext
}

/// Returns `true` if the spring is currently compressed (shorter than rest length).
#[allow(dead_code)]
pub fn spring_is_compressed(
    a: &SpringEndpoint,
    b: &SpringEndpoint,
    cfg: &StiffSpringConfig,
) -> bool {
    spring_length(a, b) < cfg.rest_length
}

/// Compute the spring force vectors for both endpoints.
///
/// Uses Hooke's law with viscous damping:
/// `F = -(k * extension + c * v_rel) * direction`
#[allow(dead_code)]
pub fn compute_spring_force(
    a: &SpringEndpoint,
    b: &SpringEndpoint,
    cfg: &StiffSpringConfig,
) -> StiffSpringResult {
    let diff = sub3(b.position, a.position);
    let dist = len3(diff);
    let extension = dist - cfg.rest_length;
    let dir = if dist < 1e-12 {
        [0.0, 1.0, 0.0]
    } else {
        normalize3(diff)
    };

    // Relative velocity along the spring axis.
    let rel_vel = sub3(b.velocity, a.velocity);
    let rel_vel_proj = dot3(rel_vel, dir);

    let scalar = cfg.stiffness * extension + cfg.damping * rel_vel_proj;
    let scalar = scalar.clamp(-cfg.max_force, cfg.max_force);

    let force_on_a = scale3(dir, scalar);
    let force_on_b = neg3(force_on_a);

    StiffSpringResult {
        force_a: force_on_a,
        force_b: force_on_b,
        spring_length: dist,
        extension,
    }
}

/// Advance the spring system by one timestep `dt` using explicit Euler integration.
///
/// Fixed endpoints are not moved.
#[allow(dead_code)]
pub fn step_spring(
    a: &mut SpringEndpoint,
    b: &mut SpringEndpoint,
    cfg: &StiffSpringConfig,
    dt: f32,
) {
    let result = compute_spring_force(a, b, cfg);

    if !a.fixed && a.mass > 0.0 {
        let accel = scale3(result.force_a, 1.0 / a.mass);
        a.velocity[0] += accel[0] * dt;
        a.velocity[1] += accel[1] * dt;
        a.velocity[2] += accel[2] * dt;
        a.position[0] += a.velocity[0] * dt;
        a.position[1] += a.velocity[1] * dt;
        a.position[2] += a.velocity[2] * dt;
    }

    if !b.fixed && b.mass > 0.0 {
        let accel = scale3(result.force_b, 1.0 / b.mass);
        b.velocity[0] += accel[0] * dt;
        b.velocity[1] += accel[1] * dt;
        b.velocity[2] += accel[2] * dt;
        b.position[0] += b.velocity[0] * dt;
        b.position[1] += b.velocity[1] * dt;
        b.position[2] += b.velocity[2] * dt;
    }
}

/// Serialize a `StiffSpringResult` to JSON.
#[allow(dead_code)]
pub fn spring_result_to_json(r: &StiffSpringResult) -> String {
    format!(
        "{{\"spring_length\":{},\"extension\":{},\"force_a\":[{},{},{}],\"force_b\":[{},{},{}]}}",
        r.spring_length,
        r.extension,
        r.force_a[0],
        r.force_a[1],
        r.force_a[2],
        r.force_b[0],
        r.force_b[1],
        r.force_b[2],
    )
}

/// Serialize a `StiffSpringConfig` to JSON.
#[allow(dead_code)]
pub fn spring_config_to_json(cfg: &StiffSpringConfig) -> String {
    format!(
        "{{\"rest_length\":{},\"stiffness\":{},\"damping\":{},\"max_force\":{}}}",
        cfg.rest_length, cfg.stiffness, cfg.damping, cfg.max_force,
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_length_basic() {
        let a = new_spring_endpoint([0.0, 0.0, 0.0], 1.0);
        let b = new_spring_endpoint([3.0, 4.0, 0.0], 1.0);
        assert!((spring_length(&a, &b) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn spring_extension_at_rest() {
        let a = new_spring_endpoint([0.0, 0.0, 0.0], 1.0);
        let b = new_spring_endpoint([1.0, 0.0, 0.0], 1.0);
        let cfg = default_stiff_spring_config(1.0, 100.0);
        assert!(spring_extension(&a, &b, &cfg).abs() < 1e-6);
    }

    #[test]
    fn spring_extension_stretched() {
        let a = new_spring_endpoint([0.0, 0.0, 0.0], 1.0);
        let b = new_spring_endpoint([2.0, 0.0, 0.0], 1.0);
        let cfg = default_stiff_spring_config(1.0, 100.0);
        assert!((spring_extension(&a, &b, &cfg) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn compute_spring_force_at_rest_zero() {
        let a = new_spring_endpoint([0.0, 0.0, 0.0], 1.0);
        let b = new_spring_endpoint([1.0, 0.0, 0.0], 1.0);
        let cfg = default_stiff_spring_config(1.0, 100.0);
        let r = compute_spring_force(&a, &b, &cfg);
        // At rest, extension = 0 and velocities = 0, so force should be near zero.
        assert!(r.force_a[0].abs() < 1e-5);
        assert!(r.force_b[0].abs() < 1e-5);
    }

    #[test]
    fn compute_spring_force_stretched_pulls_inward() {
        let a = new_spring_endpoint([0.0, 0.0, 0.0], 1.0);
        let b = new_spring_endpoint([2.0, 0.0, 0.0], 1.0);
        let cfg = default_stiff_spring_config(1.0, 100.0);
        let r = compute_spring_force(&a, &b, &cfg);
        // force_a should pull toward b (positive x), force_b negative x.
        assert!(r.force_a[0] > 0.0);
        assert!(r.force_b[0] < 0.0);
    }

    #[test]
    fn spring_is_compressed_check() {
        let a = new_spring_endpoint([0.0, 0.0, 0.0], 1.0);
        let b = new_spring_endpoint([0.5, 0.0, 0.0], 1.0);
        let cfg = default_stiff_spring_config(1.0, 100.0);
        assert!(spring_is_compressed(&a, &b, &cfg));
    }

    #[test]
    fn potential_energy_stretched() {
        let a = new_spring_endpoint([0.0, 0.0, 0.0], 1.0);
        let b = new_spring_endpoint([2.0, 0.0, 0.0], 1.0);
        let cfg = default_stiff_spring_config(1.0, 100.0);
        // E = 0.5 * 100 * 1^2 = 50.0
        let e = spring_potential_energy(&a, &b, &cfg);
        assert!((e - 50.0).abs() < 1e-4);
    }

    #[test]
    fn step_spring_moves_free_endpoint() {
        let mut a = new_spring_endpoint([0.0, 0.0, 0.0], 1.0);
        a.fixed = true;
        let mut b = new_spring_endpoint([2.0, 0.0, 0.0], 1.0);
        let cfg = default_stiff_spring_config(1.0, 100.0);
        let pos_before = b.position[0];
        step_spring(&mut a, &mut b, &cfg, 0.01);
        // b should move toward a (x decreases).
        assert!(b.position[0] < pos_before);
    }

    #[test]
    fn spring_result_to_json_fields() {
        let a = new_spring_endpoint([0.0, 0.0, 0.0], 1.0);
        let b = new_spring_endpoint([1.0, 0.0, 0.0], 1.0);
        let cfg = default_stiff_spring_config(1.0, 100.0);
        let r = compute_spring_force(&a, &b, &cfg);
        let json = spring_result_to_json(&r);
        assert!(json.contains("spring_length"));
        assert!(json.contains("extension"));
    }

    #[test]
    fn spring_config_to_json_fields() {
        let cfg = default_stiff_spring_config(2.0, 50.0);
        let json = spring_config_to_json(&cfg);
        assert!(json.contains("rest_length"));
        assert!(json.contains("stiffness"));
    }
}
