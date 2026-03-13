// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Gravity field — point, uniform, and vortex gravity sources.
//!
//! A [`GravityField`] accumulates multiple [`GravitySource`] values and
//! evaluates the total gravitational acceleration at any 3-D point.

// ─── Structures ──────────────────────────────────────────────────────────────

/// Discriminator for gravity source behaviour.
#[allow(dead_code)]
pub enum GravityFieldType {
    /// Constant acceleration in a fixed direction.
    Uniform,
    /// Newtonian point-mass attraction (falls off as 1/r²).
    Point,
    /// Swirling vortex around an axis through a centre.
    Vortex,
}

/// A single gravity source.
#[allow(dead_code)]
pub struct GravitySource {
    pub kind: GravityFieldType,
    /// Direction for Uniform; centre for Point and Vortex.
    pub origin: [f32; 3],
    /// Axis direction for Vortex (unit vector).
    pub axis: [f32; 3],
    /// Gravitational parameter or strength (meaning depends on kind).
    pub strength: f32,
}

/// A collection of gravity sources whose effects are summed.
#[allow(dead_code)]
pub struct GravityField {
    pub sources: Vec<GravitySource>,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Create an empty gravity field.
#[allow(dead_code)]
pub fn new_gravity_field() -> GravityField {
    GravityField {
        sources: Vec::new(),
    }
}

/// Add a gravity source to the field.
#[allow(dead_code)]
pub fn gravity_add_source(field: &mut GravityField, source: GravitySource) {
    field.sources.push(source);
}

/// Compute the total gravitational acceleration at `pos` (m/s²).
#[allow(dead_code)]
pub fn gravity_at_point(field: &GravityField, pos: [f32; 3]) -> [f32; 3] {
    let mut acc = [0.0f32; 3];

    for src in &field.sources {
        let contribution = match src.kind {
            GravityFieldType::Uniform => {
                // direction stored in `axis`, magnitude in `strength`.
                scale3(normalize3(src.axis), src.strength)
            }
            GravityFieldType::Point => {
                let r = sub3(src.origin, pos);
                let r_sq = dot3(r, r);
                if r_sq < 1e-12 {
                    [0.0; 3]
                } else {
                    // a = GM / r² * r̂
                    let r_len = r_sq.sqrt();
                    let gm = src.strength; // treat strength as GM
                    scale3(r, gm / (r_sq * r_len))
                }
            }
            GravityFieldType::Vortex => {
                // Tangential acceleration around `axis` through `origin`.
                let r = sub3(pos, src.origin);
                let axis_n = normalize3(src.axis);
                // Component of r perpendicular to axis.
                let proj = scale3(axis_n, dot3(r, axis_n));
                let perp = sub3(r, proj);
                let perp_len = (dot3(perp, perp)).sqrt();
                if perp_len < 1e-12 {
                    [0.0; 3]
                } else {
                    let tangent = cross3(axis_n, normalize3(perp));
                    scale3(tangent, src.strength)
                }
            }
        };
        acc = add3(acc, contribution);
    }

    acc
}

/// Create a uniform gravity source.
///
/// `direction` is the pull direction (will be normalised internally);
/// `strength` is the acceleration magnitude in m/s².
#[allow(dead_code)]
pub fn new_uniform_gravity(direction: [f32; 3], strength: f32) -> GravitySource {
    GravitySource {
        kind: GravityFieldType::Uniform,
        origin: [0.0; 3],
        axis: direction,
        strength,
    }
}

/// Create a point (Newtonian) gravity source.
///
/// `center` is the position of the mass; `mass` is the gravitational parameter
/// GM (m³/s²).
#[allow(dead_code)]
pub fn new_point_gravity(center: [f32; 3], mass: f32) -> GravitySource {
    GravitySource {
        kind: GravityFieldType::Point,
        origin: center,
        axis: [0.0, 1.0, 0.0],
        strength: mass,
    }
}

/// Create a vortex gravity source.
///
/// `center` is the vortex centre; `axis` is the rotation axis; `strength` is
/// the tangential acceleration magnitude.
#[allow(dead_code)]
pub fn new_vortex_gravity(center: [f32; 3], axis: [f32; 3], strength: f32) -> GravitySource {
    GravitySource {
        kind: GravityFieldType::Vortex,
        origin: center,
        axis,
        strength,
    }
}

/// Number of sources in the field.
#[allow(dead_code)]
pub fn gravity_source_count(field: &GravityField) -> usize {
    field.sources.len()
}

/// Remove all sources from the field.
#[allow(dead_code)]
pub fn gravity_field_clear(field: &mut GravityField) {
    field.sources.clear();
}

/// True if the field has no sources.
#[allow(dead_code)]
pub fn gravity_field_is_empty(field: &GravityField) -> bool {
    field.sources.is_empty()
}

/// Human-readable name of a gravity source type.
#[allow(dead_code)]
pub fn gravity_source_type_name(src: &GravitySource) -> &'static str {
    match src.kind {
        GravityFieldType::Uniform => "Uniform",
        GravityFieldType::Point => "Point",
        GravityFieldType::Vortex => "Vortex",
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_field_is_empty() {
        let field = new_gravity_field();
        assert!(gravity_field_is_empty(&field));
        assert_eq!(gravity_source_count(&field), 0);
    }

    #[test]
    fn test_add_source_increments_count() {
        let mut field = new_gravity_field();
        gravity_add_source(&mut field, new_uniform_gravity([0.0, -1.0, 0.0], 9.81));
        assert_eq!(gravity_source_count(&field), 1);
    }

    #[test]
    fn test_clear_field() {
        let mut field = new_gravity_field();
        gravity_add_source(&mut field, new_uniform_gravity([0.0, -1.0, 0.0], 9.81));
        gravity_field_clear(&mut field);
        assert!(gravity_field_is_empty(&field));
    }

    #[test]
    fn test_uniform_gravity_direction() {
        let mut field = new_gravity_field();
        gravity_add_source(&mut field, new_uniform_gravity([0.0, -1.0, 0.0], 9.81));
        let acc = gravity_at_point(&field, [0.0, 0.0, 0.0]);
        // Should pull downward.
        assert!(acc[1] < -9.0, "acc={:?}", acc);
    }

    #[test]
    fn test_point_gravity_pulls_toward_center() {
        let mut field = new_gravity_field();
        gravity_add_source(&mut field, new_point_gravity([0.0, 0.0, 0.0], 1000.0));
        // A particle at [1,0,0] should be pulled in the -X direction.
        let acc = gravity_at_point(&field, [1.0, 0.0, 0.0]);
        assert!(acc[0] < 0.0, "acc={:?}", acc);
    }

    #[test]
    fn test_point_gravity_zero_at_center_does_not_panic() {
        let mut field = new_gravity_field();
        gravity_add_source(&mut field, new_point_gravity([0.0, 0.0, 0.0], 1000.0));
        // Should return zero, not panic.
        let acc = gravity_at_point(&field, [0.0, 0.0, 0.0]);
        assert_eq!(acc, [0.0; 3]);
    }

    #[test]
    fn test_vortex_gravity_nonzero_off_axis() {
        let mut field = new_gravity_field();
        gravity_add_source(
            &mut field,
            new_vortex_gravity([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 5.0),
        );
        // Particle at [1,0,0] is perpendicular to Y axis, should get a
        // tangential kick.
        let acc = gravity_at_point(&field, [1.0, 0.0, 0.0]);
        let mag = (acc[0] * acc[0] + acc[1] * acc[1] + acc[2] * acc[2]).sqrt();
        assert!(mag > 1.0, "acc={:?}", acc);
    }

    #[test]
    fn test_empty_field_returns_zero() {
        let field = new_gravity_field();
        let acc = gravity_at_point(&field, [3.0, 4.0, 5.0]);
        assert_eq!(acc, [0.0; 3]);
    }

    #[test]
    fn test_source_type_names() {
        let u = new_uniform_gravity([0.0, -1.0, 0.0], 1.0);
        let p = new_point_gravity([0.0, 0.0, 0.0], 1.0);
        let v = new_vortex_gravity([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 1.0);
        assert_eq!(gravity_source_type_name(&u), "Uniform");
        assert_eq!(gravity_source_type_name(&p), "Point");
        assert_eq!(gravity_source_type_name(&v), "Vortex");
    }

    #[test]
    fn test_two_sources_sum() {
        let mut field = new_gravity_field();
        gravity_add_source(&mut field, new_uniform_gravity([0.0, -1.0, 0.0], 9.81));
        gravity_add_source(&mut field, new_uniform_gravity([0.0, -1.0, 0.0], 9.81));
        let acc = gravity_at_point(&field, [0.0, 0.0, 0.0]);
        // Approximately double the single-source value.
        assert!((acc[1] + 2.0 * 9.81).abs() < 1e-3, "acc={:?}", acc);
    }
}
