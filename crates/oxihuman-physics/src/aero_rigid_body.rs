// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Aerodynamic lift and drag forces for rigid body simulation.

// ── Structs ──────────────────────────────────────────────────────────────────

/// Configuration for aerodynamic force computation on a rigid body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RigidAeroConfig {
    pub air_density: f32,
    pub lift_coefficient: f32,
    pub drag_coefficient: f32,
    pub reference_area: f32,
}

/// State of an aerodynamic rigid body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AeroBody {
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    /// Forward/orientation direction of the body.
    pub orientation: [f32; 3],
    pub mass: f32,
}

/// Result of computing aerodynamic forces on a body.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AeroForceResult {
    pub drag: [f32; 3],
    pub lift: [f32; 3],
    pub total_force: [f32; 3],
    pub dynamic_pressure: f32,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

#[inline]
fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

#[inline]
fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[inline]
fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-12 {
        [0.0, 0.0, 0.0]
    } else {
        scale3(v, 1.0 / l)
    }
}

#[inline]
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Build a default `RigidAeroConfig` with standard sea-level air density.
#[allow(dead_code)]
pub fn default_aero_config() -> RigidAeroConfig {
    RigidAeroConfig {
        air_density: 1.225,
        lift_coefficient: 0.3,
        drag_coefficient: 0.47,
        reference_area: 1.0,
    }
}

/// Construct a new `AeroBody` at `pos` with zero velocity.
#[allow(dead_code)]
pub fn new_aero_body(pos: [f32; 3], mass: f32) -> AeroBody {
    AeroBody {
        position: pos,
        velocity: [0.0, 0.0, 0.0],
        orientation: [1.0, 0.0, 0.0],
        mass: mass.max(1e-6),
    }
}

/// Compute dynamic pressure: `0.5 * rho * v^2`.
#[allow(dead_code)]
pub fn rigid_dynamic_pressure(body: &AeroBody, cfg: &RigidAeroConfig) -> f32 {
    let v = body_speed(body);
    0.5 * cfg.air_density * v * v
}

/// Compute the drag force vector on `body`.
///
/// Drag opposes velocity: `F_d = -0.5 * rho * Cd * A * v^2 * v_hat`.
#[allow(dead_code)]
pub fn compute_drag(body: &AeroBody, cfg: &RigidAeroConfig) -> [f32; 3] {
    let v2 = dot3(body.velocity, body.velocity);
    if v2 < 1e-12 {
        return [0.0, 0.0, 0.0];
    }
    let v_hat = normalize3(body.velocity);
    let mag = 0.5 * cfg.air_density * cfg.drag_coefficient * cfg.reference_area * v2;
    scale3(v_hat, -mag)
}

/// Compute the lift force vector on `body`.
///
/// Lift is perpendicular to velocity in the plane of velocity and orientation:
/// `F_l = 0.5 * rho * Cl * A * v^2 * lift_dir`.
#[allow(dead_code)]
pub fn compute_lift(body: &AeroBody, cfg: &RigidAeroConfig) -> [f32; 3] {
    let v2 = dot3(body.velocity, body.velocity);
    if v2 < 1e-12 {
        return [0.0, 0.0, 0.0];
    }
    let v_hat = normalize3(body.velocity);
    // Lift direction: cross(velocity, orientation) x velocity, normalised
    let side = cross3(v_hat, normalize3(body.orientation));
    let lift_dir = normalize3(cross3(side, v_hat));
    let mag = 0.5 * cfg.air_density * cfg.lift_coefficient * cfg.reference_area * v2;
    scale3(lift_dir, mag)
}

/// Compute combined aerodynamic forces on `body`.
#[allow(dead_code)]
pub fn compute_aero_forces(body: &AeroBody, cfg: &RigidAeroConfig) -> AeroForceResult {
    let drag = compute_drag(body, cfg);
    let lift = compute_lift(body, cfg);
    let total_force = add3(drag, lift);
    let dynamic_pressure = rigid_dynamic_pressure(body, cfg);
    AeroForceResult {
        drag,
        lift,
        total_force,
        dynamic_pressure,
    }
}

/// Return the scalar speed of `body`.
#[allow(dead_code)]
pub fn body_speed(body: &AeroBody) -> f32 {
    len3(body.velocity)
}

/// Compute the angle of attack in radians: angle between velocity and orientation.
#[allow(dead_code)]
pub fn angle_of_attack(body: &AeroBody) -> f32 {
    let v_len = len3(body.velocity);
    let o_len = len3(body.orientation);
    if v_len < 1e-12 || o_len < 1e-12 {
        return 0.0;
    }
    let cos_a = (dot3(body.velocity, body.orientation) / (v_len * o_len)).clamp(-1.0, 1.0);
    cos_a.acos()
}

/// Serialize an `AeroForceResult` to a JSON string.
#[allow(dead_code)]
pub fn aero_force_to_json(r: &AeroForceResult) -> String {
    format!(
        "{{\"drag\":[{:.4},{:.4},{:.4}],\"lift\":[{:.4},{:.4},{:.4}],\"dynamic_pressure\":{:.4}}}",
        r.drag[0],
        r.drag[1],
        r.drag[2],
        r.lift[0],
        r.lift[1],
        r.lift[2],
        r.dynamic_pressure,
    )
}

/// Apply aerodynamic forces to `body` for one time step `dt`.
#[allow(dead_code)]
pub fn apply_aero(body: &mut AeroBody, cfg: &RigidAeroConfig, dt: f32) {
    let forces = compute_aero_forces(body, cfg);
    let inv_mass = 1.0 / body.mass;
    body.velocity[0] += forces.total_force[0] * inv_mass * dt;
    body.velocity[1] += forces.total_force[1] * inv_mass * dt;
    body.velocity[2] += forces.total_force[2] * inv_mass * dt;
    body.position[0] += body.velocity[0] * dt;
    body.position[1] += body.velocity[1] * dt;
    body.position[2] += body.velocity[2] * dt;
}

/// Compute the Mach number: speed divided by `speed_of_sound`.
#[allow(dead_code)]
pub fn mach_number(body: &AeroBody, speed_of_sound: f32) -> f32 {
    let s = body_speed(body);
    if speed_of_sound < 1e-12 {
        return 0.0;
    }
    s / speed_of_sound
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_air_density() {
        let cfg = default_aero_config();
        assert!((cfg.air_density - 1.225).abs() < 1e-4);
    }

    #[test]
    fn new_body_zero_velocity() {
        let body = new_aero_body([0.0, 0.0, 0.0], 1.0);
        assert_eq!(body.velocity, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn drag_opposes_velocity() {
        let cfg = default_aero_config();
        let mut body = new_aero_body([0.0, 0.0, 0.0], 1.0);
        body.velocity = [10.0, 0.0, 0.0];
        let drag = compute_drag(&body, &cfg);
        assert!(drag[0] < 0.0, "Drag should oppose positive X velocity");
    }

    #[test]
    fn dynamic_pressure_positive_for_moving_body() {
        let cfg = default_aero_config();
        let mut body = new_aero_body([0.0, 0.0, 0.0], 1.0);
        body.velocity = [5.0, 0.0, 0.0];
        let q = rigid_dynamic_pressure(&body, &cfg);
        assert!(q > 0.0);
    }

    #[test]
    fn mach_number_computed_correctly() {
        let mut body = new_aero_body([0.0, 0.0, 0.0], 1.0);
        body.velocity = [343.0, 0.0, 0.0];
        let m = mach_number(&body, 343.0);
        assert!((m - 1.0).abs() < 1e-5);
    }

    #[test]
    fn apply_aero_changes_position() {
        let cfg = default_aero_config();
        let mut body = new_aero_body([0.0, 0.0, 0.0], 1.0);
        body.velocity = [10.0, 0.0, 0.0];
        let old_pos = body.position;
        apply_aero(&mut body, &cfg, 0.016);
        assert!(
            (body.position[0] - old_pos[0]).abs() > 1e-6,
            "Position should change after apply_aero"
        );
    }

    #[test]
    fn aero_force_to_json_contains_fields() {
        let r = AeroForceResult {
            drag: [-1.0, 0.0, 0.0],
            lift: [0.0, 0.5, 0.0],
            total_force: [-1.0, 0.5, 0.0],
            dynamic_pressure: 61.25,
        };
        let json = aero_force_to_json(&r);
        assert!(json.contains("drag"));
        assert!(json.contains("lift"));
        assert!(json.contains("dynamic_pressure"));
    }

    #[test]
    fn angle_of_attack_zero_when_aligned() {
        let mut body = new_aero_body([0.0, 0.0, 0.0], 1.0);
        body.velocity = [1.0, 0.0, 0.0];
        body.orientation = [1.0, 0.0, 0.0];
        let aoa = angle_of_attack(&body);
        assert!(aoa.abs() < 1e-5);
    }
}
