//! Wind force computation for cloth simulation.
//!
//! Computes aerodynamic drag forces on cloth triangles.  Each triangle
//! receives a force proportional to the projected area and the relative
//! wind velocity (one-sided lift-drag model).

#![allow(dead_code)]

// ── Math helpers ──────────────────────────────────────────────────────────────

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
fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

// ── Public types ──────────────────────────────────────────────────────────────

/// Configuration for the wind force model.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothWindConfig {
    /// Aerodynamic drag coefficient.
    pub drag_coefficient: f32,
    /// Air density (kg/m³).
    pub air_density: f32,
    /// Base wind velocity vector (m/s).
    pub base_velocity: [f32; 3],
    /// Whether wind is enabled.
    pub enabled: bool,
}

/// A computed wind force for a single cloth triangle.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindForce {
    /// Force applied to vertex A.
    pub fa: [f32; 3],
    /// Force applied to vertex B.
    pub fb: [f32; 3],
    /// Force applied to vertex C.
    pub fc: [f32; 3],
}

/// Result of computing wind forces over a cloth mesh.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClothWindResult {
    /// Per-triangle wind forces.
    pub forces: Vec<WindForce>,
    /// Sum of all forces (net force on the cloth).
    pub total_force: [f32; 3],
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Return default `ClothWindConfig`.
#[allow(dead_code)]
pub fn default_cloth_wind_config() -> ClothWindConfig {
    ClothWindConfig {
        drag_coefficient: 1.0,
        air_density: 1.225,
        base_velocity: [1.0, 0.0, 0.0],
        enabled: true,
    }
}

/// Create a new wind config from the given parameters.
#[allow(dead_code)]
pub fn new_cloth_wind(velocity: [f32; 3], drag_coeff: f32) -> ClothWindConfig {
    ClothWindConfig {
        drag_coefficient: drag_coeff,
        air_density: 1.225,
        base_velocity: velocity,
        enabled: true,
    }
}

/// Update the wind velocity.
#[allow(dead_code)]
pub fn cloth_wind_set_velocity(config: &mut ClothWindConfig, velocity: [f32; 3]) {
    config.base_velocity = velocity;
}

/// Compute wind forces on all cloth triangles defined by `positions` and
/// `indices`.
///
/// Each vertex of each triangle receives one-third of the triangle's total
/// aerodynamic force.
#[allow(dead_code)]
pub fn cloth_wind_compute_forces(
    config: &ClothWindConfig,
    positions: &[[f32; 3]],
    indices: &[u32],
) -> ClothWindResult {
    let mut forces = Vec::new();
    let mut total = [0.0f32; 3];

    if !config.enabled {
        return ClothWindResult { forces, total_force: total };
    }

    let tri_count = indices.len() / 3;
    for t in 0..tri_count {
        let ia = indices[t * 3] as usize;
        let ib = indices[t * 3 + 1] as usize;
        let ic = indices[t * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            forces.push(WindForce { fa: [0.0; 3], fb: [0.0; 3], fc: [0.0; 3] });
            continue;
        }

        let wf = cloth_wind_force_on_triangle(
            config,
            positions[ia],
            positions[ib],
            positions[ic],
        );
        total = add3(total, add3(wf.fa, add3(wf.fb, wf.fc)));
        forces.push(wf);
    }

    ClothWindResult { forces, total_force: total }
}

/// Compute the wind force on a single triangle `(a, b, c)`.
///
/// The force is proportional to the projected area and dynamic pressure.
#[allow(dead_code)]
pub fn cloth_wind_force_on_triangle(
    config: &ClothWindConfig,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
) -> WindForce {
    if !config.enabled {
        return WindForce { fa: [0.0; 3], fb: [0.0; 3], fc: [0.0; 3] };
    }

    // Face normal (not normalised — magnitude is 2 * area)
    let edge1 = sub3(b, a);
    let edge2 = sub3(c, a);
    let n = cross3(edge1, edge2);
    let area2 = len3(n);

    if area2 < 1e-10 {
        return WindForce { fa: [0.0; 3], fb: [0.0; 3], fc: [0.0; 3] };
    }

    let n_unit = scale3(n, 1.0 / area2);
    let w = config.base_velocity;
    let proj = dot3(w, n_unit);

    // Dynamic pressure: 0.5 * rho * |w|^2 applied over projected area
    let area = area2 * 0.5;
    let speed_sq = dot3(w, w);
    let dynamic_p = 0.5 * config.air_density * speed_sq;
    let mag = config.drag_coefficient * dynamic_p * area * proj.abs();

    // Direction is along the normal, signed by the wind projection direction
    let sign = if proj >= 0.0 { 1.0 } else { -1.0 };
    let force = scale3(n_unit, mag * sign);

    // Distribute equally among the three vertices
    let third = scale3(force, 1.0 / 3.0);
    WindForce { fa: third, fb: third, fc: third }
}

/// Return the total net force from a `ClothWindResult`.
#[allow(dead_code)]
pub fn cloth_wind_total_force(result: &ClothWindResult) -> [f32; 3] {
    result.total_force
}

/// Serialise the config to a JSON string.
#[allow(dead_code)]
pub fn cloth_wind_to_json(config: &ClothWindConfig) -> String {
    format!(
        "{{\"drag_coefficient\":{},\"air_density\":{},\"enabled\":{}}}",
        config.drag_coefficient, config.air_density, config.enabled,
    )
}

/// Enable wind forces.
#[allow(dead_code)]
pub fn cloth_wind_enable(config: &mut ClothWindConfig) {
    config.enabled = true;
}

/// Disable wind forces.
#[allow(dead_code)]
pub fn cloth_wind_disable(config: &mut ClothWindConfig) {
    config.enabled = false;
}

/// Return whether wind is enabled.
#[allow(dead_code)]
pub fn cloth_wind_is_enabled(config: &ClothWindConfig) -> bool {
    config.enabled
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_triangle() -> ([f32; 3], [f32; 3], [f32; 3]) {
        ([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0])
    }

    #[test]
    fn test_default_config_enabled() {
        let cfg = default_cloth_wind_config();
        assert!(cfg.enabled);
    }

    #[test]
    fn test_new_cloth_wind_sets_velocity() {
        let cfg = new_cloth_wind([2.0, 0.0, 0.0], 1.0);
        assert!((cfg.base_velocity[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_velocity() {
        let mut cfg = default_cloth_wind_config();
        cloth_wind_set_velocity(&mut cfg, [0.0, 5.0, 0.0]);
        assert!((cfg.base_velocity[1] - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_force_on_triangle_with_wind_perpendicular() {
        // Wind along +Z, triangle in XY plane → normal along Z → max projected area
        let cfg = new_cloth_wind([0.0, 0.0, 1.0], 1.0);
        let a = [0.0f32, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        let wf = cloth_wind_force_on_triangle(&cfg, a, b, c);
        // Force should be in the Z direction
        let fz = wf.fa[2] + wf.fb[2] + wf.fc[2];
        assert!(fz.abs() > 0.0, "expected nonzero Z force, got {}", fz);
    }

    #[test]
    fn test_disabled_wind_zero_force() {
        let mut cfg = default_cloth_wind_config();
        cloth_wind_disable(&mut cfg);
        let (a, b, c) = flat_triangle();
        let wf = cloth_wind_force_on_triangle(&cfg, a, b, c);
        assert_eq!(wf.fa, [0.0; 3]);
        assert_eq!(wf.fb, [0.0; 3]);
        assert_eq!(wf.fc, [0.0; 3]);
    }

    #[test]
    fn test_enable_disable_toggle() {
        let mut cfg = default_cloth_wind_config();
        cloth_wind_disable(&mut cfg);
        assert!(!cloth_wind_is_enabled(&cfg));
        cloth_wind_enable(&mut cfg);
        assert!(cloth_wind_is_enabled(&cfg));
    }

    #[test]
    fn test_compute_forces_empty_mesh() {
        let cfg = default_cloth_wind_config();
        let result = cloth_wind_compute_forces(&cfg, &[], &[]);
        assert_eq!(result.forces.len(), 0);
    }

    #[test]
    fn test_compute_forces_single_triangle() {
        let cfg = new_cloth_wind([0.0, 0.0, 1.0], 1.0);
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let result = cloth_wind_compute_forces(&cfg, &pos, &idx);
        assert_eq!(result.forces.len(), 1);
    }

    #[test]
    fn test_total_force_non_zero_with_wind() {
        let cfg = new_cloth_wind([0.0, 0.0, 5.0], 1.0);
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let result = cloth_wind_compute_forces(&cfg, &pos, &idx);
        let tf = cloth_wind_total_force(&result);
        let mag = (tf[0]*tf[0]+tf[1]*tf[1]+tf[2]*tf[2]).sqrt();
        assert!(mag > 0.0);
    }

    #[test]
    fn test_cloth_wind_to_json() {
        let cfg = default_cloth_wind_config();
        let json = cloth_wind_to_json(&cfg);
        assert!(json.contains("drag_coefficient"));
        assert!(json.contains("enabled"));
    }

    #[test]
    fn test_degenerate_triangle_no_panic() {
        let cfg = default_cloth_wind_config();
        let a = [0.0f32; 3];
        let wf = cloth_wind_force_on_triangle(&cfg, a, a, a);
        assert_eq!(wf.fa, [0.0; 3]);
    }
}
