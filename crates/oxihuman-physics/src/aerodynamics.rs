// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Aerodynamic drag and lift forces for cloth and hair simulation.

// ── Data structures ───────────────────────────────────────────────────────────

/// Aerodynamic configuration parameters.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AeroConfig {
    /// Air density in kg/m³, default `1.225`.
    pub air_density: f32,
    /// Drag coefficient `C_d`, default `1.0` for a cloth panel.
    pub drag_coeff: f32,
    /// Lift coefficient `C_l`, default `0.0` (no lift for flat cloth).
    pub lift_coeff: f32,
    /// Dynamic viscosity in Pa·s, default `1.8e-5`.
    pub viscosity: f32,
}

impl Default for AeroConfig {
    fn default() -> Self {
        Self {
            air_density: 1.225,
            drag_coeff: 1.0,
            lift_coeff: 0.0,
            viscosity: 1.8e-5,
        }
    }
}

/// Combined aerodynamic force result.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AeroForce {
    /// Drag force vector (opposes relative velocity).
    pub drag: [f32; 3],
    /// Lift force vector.
    pub lift: [f32; 3],
    /// Total force = drag + lift.
    pub total: [f32; 3],
    /// Reynolds number for this flow condition.
    pub reynolds: f32,
}

// ── Vector helpers ────────────────────────────────────────────────────────────

#[allow(dead_code)]
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
fn len3(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}

// ── Core computations ─────────────────────────────────────────────────────────

/// Scalar drag force magnitude: `0.5 * rho * Cd * A * v²`.
#[allow(dead_code)]
pub fn drag_force_magnitude(speed: f32, area: f32, cfg: &AeroConfig) -> f32 {
    0.5 * cfg.air_density * cfg.drag_coeff * area * speed * speed
}

/// Reynolds number: `speed * length * rho / viscosity`.
#[allow(dead_code)]
pub fn reynolds_number(speed: f32, length: f32, cfg: &AeroConfig) -> f32 {
    speed * length * cfg.air_density / cfg.viscosity
}

/// Terminal velocity of a body falling under gravity:
/// `sqrt(2 * m * g / (rho * Cd * A))`.
/// Uses `g = 9.81 m/s²`.
#[allow(dead_code)]
pub fn terminal_velocity(mass: f32, area: f32, cfg: &AeroConfig) -> f32 {
    let g = 9.81_f32;
    let denom = cfg.air_density * cfg.drag_coeff * area;
    if denom < 1e-12 {
        return f32::INFINITY;
    }
    (2.0 * mass * g / denom).sqrt()
}

/// Stokes drag on a small sphere: `6π * viscosity * radius * speed`.
#[allow(dead_code)]
pub fn stokes_drag(speed: f32, radius: f32, cfg: &AeroConfig) -> f32 {
    6.0 * std::f32::consts::PI * cfg.viscosity * radius * speed
}

/// Compute aerodynamic drag and lift forces on a surface element.
///
/// * `velocity_rel` — relative wind velocity (wind_vel − particle_vel).
/// * `face_normal`  — unit normal of the surface element.
/// * `face_area`    — area of the surface element (m²).
///
/// Drag: `F_d = -0.5 * rho * Cd * A * |v|² * v̂`
/// Lift: `F_l = 0.5 * rho * Cl * A * |v|² * ((v̂ × n̂) × v̂)`
#[allow(dead_code)]
pub fn compute_aero_force(
    velocity_rel: [f32; 3],
    face_normal: [f32; 3],
    face_area: f32,
    cfg: &AeroConfig,
) -> AeroForce {
    let speed = len3(velocity_rel);
    if speed < 1e-9 {
        return AeroForce {
            drag: [0.0; 3],
            lift: [0.0; 3],
            total: [0.0; 3],
            reynolds: 0.0,
        };
    }

    let v_norm = scale3(velocity_rel, 1.0 / speed);
    let v_sq = speed * speed;
    let half_rho_a = 0.5 * cfg.air_density * face_area;

    // Drag opposes relative velocity
    let drag_mag = half_rho_a * cfg.drag_coeff * v_sq;
    let drag = scale3(v_norm, -drag_mag);

    // Lift: (v̂ × n̂) × v̂
    let lift_dir_raw = cross3(cross3(v_norm, face_normal), v_norm);
    let lift_mag = half_rho_a * cfg.lift_coeff * v_sq;
    let lift = scale3(lift_dir_raw, lift_mag);

    let total = add3(drag, lift);

    // Characteristic length: sqrt of area
    let char_len = face_area.sqrt();
    let re = reynolds_number(speed, char_len, cfg);

    AeroForce {
        drag,
        lift,
        total,
        reynolds: re,
    }
}

/// Apply aerodynamic forces to a set of particles given per-face triangle
/// indices.
///
/// For each triangle:
/// 1. Compute face centre relative velocity and face normal + area.
/// 2. Compute `AeroForce`.
/// 3. Distribute total force equally to the 3 triangle vertices.
/// 4. Integrate: `v += F * inv_mass * dt`.
#[allow(dead_code)]
pub fn apply_aero_to_particles(
    positions: &[[f32; 3]],
    velocities: &mut [[f32; 3]],
    wind_velocities: &[[f32; 3]],
    inv_masses: &[f32],
    face_indices: &[[usize; 3]],
    cfg: &AeroConfig,
    dt: f32,
) {
    for face in face_indices {
        let [i0, i1, i2] = *face;
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
            continue;
        }

        // Average relative velocity at face centre
        let v_rel = [
            (wind_velocities[i0][0] + wind_velocities[i1][0] + wind_velocities[i2][0]) / 3.0
                - (velocities[i0][0] + velocities[i1][0] + velocities[i2][0]) / 3.0,
            (wind_velocities[i0][1] + wind_velocities[i1][1] + wind_velocities[i2][1]) / 3.0
                - (velocities[i0][1] + velocities[i1][1] + velocities[i2][1]) / 3.0,
            (wind_velocities[i0][2] + wind_velocities[i1][2] + wind_velocities[i2][2]) / 3.0
                - (velocities[i0][2] + velocities[i1][2] + velocities[i2][2]) / 3.0,
        ];

        // Face normal and area via cross product of edges
        let e0 = sub3(positions[i1], positions[i0]);
        let e1 = sub3(positions[i2], positions[i0]);
        let cross = cross3(e0, e1);
        let cross_len = len3(cross);
        if cross_len < 1e-12 {
            continue;
        }
        let face_normal = scale3(cross, 1.0 / cross_len);
        let face_area = 0.5 * cross_len;

        let force = compute_aero_force(v_rel, face_normal, face_area, cfg);

        // Distribute equally to each vertex
        let f_third = scale3(force.total, 1.0 / 3.0);
        for &vi in &[i0, i1, i2] {
            let dv = scale3(f_third, inv_masses[vi] * dt);
            velocities[vi] = add3(velocities[vi], dv);
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> AeroConfig {
        AeroConfig::default()
    }

    // 1. drag_force_magnitude formula check
    #[test]
    fn drag_formula() {
        let cfg = default_cfg();
        let d = drag_force_magnitude(10.0, 1.0, &cfg);
        let expected = 0.5 * 1.225 * 1.0 * 1.0 * 100.0;
        assert!((d - expected).abs() < 1e-3, "got {d}, expected {expected}");
    }

    // 2. drag doubles with v²
    #[test]
    fn drag_doubles_with_v_squared() {
        let cfg = default_cfg();
        let d1 = drag_force_magnitude(1.0, 1.0, &cfg);
        let d2 = drag_force_magnitude(2.0, 1.0, &cfg);
        assert!((d2 - 4.0 * d1).abs() < 1e-5, "d1={d1}, d2={d2}");
    }

    // 3. drag doubles with area
    #[test]
    fn drag_doubles_with_area() {
        let cfg = default_cfg();
        let d1 = drag_force_magnitude(5.0, 1.0, &cfg);
        let d2 = drag_force_magnitude(5.0, 2.0, &cfg);
        assert!((d2 - 2.0 * d1).abs() < 1e-5, "d1={d1}, d2={d2}");
    }

    // 4. reynolds_number formula
    #[test]
    fn reynolds_formula() {
        let cfg = default_cfg();
        let re = reynolds_number(10.0, 1.0, &cfg);
        let expected = 10.0 * 1.0 * 1.225 / 1.8e-5;
        assert!((re - expected).abs() < 1.0, "got {re}, expected {expected}");
    }

    // 5. terminal_velocity is positive
    #[test]
    fn terminal_velocity_positive() {
        let cfg = default_cfg();
        let tv = terminal_velocity(1.0, 0.01, &cfg);
        assert!(tv > 0.0, "terminal velocity should be positive");
    }

    // 6. stokes_drag linear in speed
    #[test]
    fn stokes_drag_linearity() {
        let cfg = default_cfg();
        let d1 = stokes_drag(1.0, 0.01, &cfg);
        let d2 = stokes_drag(2.0, 0.01, &cfg);
        assert!(
            (d2 - 2.0 * d1).abs() < 1e-10,
            "stokes not linear: d1={d1}, d2={d2}"
        );
    }

    // 7. stokes_drag linear in radius
    #[test]
    fn stokes_drag_linear_radius() {
        let cfg = default_cfg();
        let d1 = stokes_drag(1.0, 0.01, &cfg);
        let d2 = stokes_drag(1.0, 0.02, &cfg);
        assert!((d2 - 2.0 * d1).abs() < 1e-10, "stokes not linear in radius");
    }

    // 8. compute_aero_force zero velocity → zero force
    #[test]
    fn aero_zero_velocity_zero_force() {
        let cfg = default_cfg();
        let f = compute_aero_force([0.0; 3], [0.0, 1.0, 0.0], 1.0, &cfg);
        assert_eq!(f.total, [0.0; 3]);
        assert_eq!(f.drag, [0.0; 3]);
    }

    // 9. drag opposes motion (drag force should be opposite to v_rel)
    #[test]
    fn drag_opposes_motion() {
        let cfg = default_cfg();
        let v_rel = [1.0_f32, 0.0, 0.0];
        let f = compute_aero_force(v_rel, [0.0, 1.0, 0.0], 1.0, &cfg);
        // drag x component should be negative (opposing +x velocity)
        assert!(
            f.drag[0] < 0.0,
            "drag should oppose motion, got {:?}",
            f.drag
        );
    }

    // 10. total = drag + lift
    #[test]
    fn total_equals_drag_plus_lift() {
        let cfg = AeroConfig {
            lift_coeff: 0.5,
            ..Default::default()
        };
        let f = compute_aero_force([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, &cfg);
        let expected = [
            f.drag[0] + f.lift[0],
            f.drag[1] + f.lift[1],
            f.drag[2] + f.lift[2],
        ];
        assert!((f.total[0] - expected[0]).abs() < 1e-6);
        assert!((f.total[1] - expected[1]).abs() < 1e-6);
        assert!((f.total[2] - expected[2]).abs() < 1e-6);
    }

    // 11. apply_aero_to_particles runs on a single triangle without panic
    #[test]
    fn apply_aero_single_triangle() {
        let cfg = default_cfg();
        let positions = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mut velocities = vec![[0.0_f32; 3]; 3];
        let wind_velocities = vec![[0.0_f32, 0.0, 5.0]; 3];
        let inv_masses = vec![1.0_f32; 3];
        let face_indices = vec![[0usize, 1, 2]];
        apply_aero_to_particles(
            &positions,
            &mut velocities,
            &wind_velocities,
            &inv_masses,
            &face_indices,
            &cfg,
            0.016,
        );
        // velocities should be finite
        for v in &velocities {
            assert!(v[0].is_finite() && v[1].is_finite() && v[2].is_finite());
        }
    }

    // 12. high speed gives large drag
    #[test]
    fn high_speed_large_drag() {
        let cfg = default_cfg();
        let low = drag_force_magnitude(1.0, 1.0, &cfg);
        let high = drag_force_magnitude(100.0, 1.0, &cfg);
        assert!(high > low * 100.0, "high-speed drag should dominate");
    }

    // 13. compute_aero_force reynolds is positive for nonzero velocity
    #[test]
    fn aero_reynolds_positive() {
        let cfg = default_cfg();
        let f = compute_aero_force([5.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0, &cfg);
        assert!(
            f.reynolds > 0.0,
            "Reynolds should be positive, got {}",
            f.reynolds
        );
    }

    // 14. dot product of drag and velocity_rel should be negative (opposing)
    #[test]
    fn drag_dot_velocity_negative() {
        let cfg = default_cfg();
        let v_rel = [2.0_f32, 1.0, 0.5];
        let f = compute_aero_force(v_rel, [0.0, 1.0, 0.0], 1.0, &cfg);
        let d = dot3(f.drag, v_rel);
        assert!(d < 0.0, "drag should oppose velocity, dot={d}");
    }
}
