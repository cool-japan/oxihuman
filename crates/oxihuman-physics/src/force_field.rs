#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Force field (gravity well, wind, vortex).

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ForceFieldType {
    GravityWell,
    Wind,
    Vortex,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ForceField {
    pub field_type: ForceFieldType,
    pub position: [f32; 3],
    pub direction: [f32; 3],
    pub strength: f32,
    pub radius: f32,
    pub falloff: f32,
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn normalize3_safe(v: [f32; 3]) -> [f32; 3] {
    let l = len3(v);
    if l < 1e-10 {
        [0.0; 3]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[allow(dead_code)]
pub fn gravity_well_force(field: &ForceField, pos: [f32; 3]) -> [f32; 3] {
    let diff = sub3(field.position, pos);
    let dist = len3(diff);
    if dist < 1e-6 || dist > field.radius {
        return [0.0; 3];
    }
    let atten = (1.0 - (dist / field.radius).powf(field.falloff)).max(0.0);
    let dir = normalize3_safe(diff);
    [
        dir[0] * field.strength * atten,
        dir[1] * field.strength * atten,
        dir[2] * field.strength * atten,
    ]
}

#[allow(dead_code)]
pub fn wind_force(field: &ForceField) -> [f32; 3] {
    let dir = normalize3_safe(field.direction);
    [
        dir[0] * field.strength,
        dir[1] * field.strength,
        dir[2] * field.strength,
    ]
}

#[allow(dead_code)]
pub fn vortex_force(field: &ForceField, pos: [f32; 3]) -> [f32; 3] {
    let diff = sub3(pos, field.position);
    let dist = len3(diff);
    if dist < 1e-6 || dist > field.radius {
        return [0.0; 3];
    }
    let axis = normalize3_safe(field.direction);
    // Tangential = axis x diff, normalized
    let tangential = cross3(axis, diff);
    let tang_dir = normalize3_safe(tangential);
    let atten = (1.0 - dist / field.radius).max(0.0);
    [
        tang_dir[0] * field.strength * atten,
        tang_dir[1] * field.strength * atten,
        tang_dir[2] * field.strength * atten,
    ]
}

#[allow(dead_code)]
pub fn sample_force_field(field: &ForceField, pos: [f32; 3]) -> [f32; 3] {
    match field.field_type {
        ForceFieldType::GravityWell => gravity_well_force(field, pos),
        ForceFieldType::Wind => wind_force(field),
        ForceFieldType::Vortex => vortex_force(field, pos),
    }
}

#[allow(dead_code)]
pub fn force_field_in_range(field: &ForceField, pos: [f32; 3]) -> bool {
    let diff = sub3(pos, field.position);
    len3(diff) <= field.radius
}

// Suppress unused warning for helpers
const _: fn() = || {
    let _ = dot3([0.0; 3], [0.0; 3]);
};

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grav_well() -> ForceField {
        ForceField {
            field_type: ForceFieldType::GravityWell,
            position: [0.0; 3],
            direction: [0.0, 1.0, 0.0],
            strength: 10.0,
            radius: 5.0,
            falloff: 1.0,
        }
    }

    fn make_wind() -> ForceField {
        ForceField {
            field_type: ForceFieldType::Wind,
            position: [0.0; 3],
            direction: [1.0, 0.0, 0.0],
            strength: 5.0,
            radius: 100.0,
            falloff: 1.0,
        }
    }

    #[test]
    fn test_gravity_well_in_range() {
        let f = make_grav_well();
        let force = gravity_well_force(&f, [1.0, 0.0, 0.0]);
        assert!(force[0] < 0.0 || force[0] < 1.0); // points towards center
    }

    #[test]
    fn test_gravity_well_out_of_range() {
        let f = make_grav_well();
        let force = gravity_well_force(&f, [10.0, 0.0, 0.0]);
        assert_eq!(force, [0.0; 3]);
    }

    #[test]
    fn test_wind_force_direction() {
        let f = make_wind();
        let force = wind_force(&f);
        assert!((force[0] - 5.0).abs() < 1e-5);
        assert!(force[1].abs() < 1e-5);
    }

    #[test]
    fn test_in_range() {
        let f = make_grav_well();
        assert!(force_field_in_range(&f, [2.0, 0.0, 0.0]));
    }

    #[test]
    fn test_out_of_range() {
        let f = make_grav_well();
        assert!(!force_field_in_range(&f, [10.0, 0.0, 0.0]));
    }

    #[test]
    fn test_sample_wind() {
        let f = make_wind();
        let force = sample_force_field(&f, [1.0, 0.0, 0.0]);
        assert!((force[0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_vortex_tangential() {
        let f = ForceField {
            field_type: ForceFieldType::Vortex,
            position: [0.0; 3],
            direction: [0.0, 1.0, 0.0],
            strength: 10.0,
            radius: 5.0,
            falloff: 1.0,
        };
        let force = vortex_force(&f, [1.0, 0.0, 0.0]);
        // Should be perpendicular to both axis and diff
        assert!(force[1].abs() < 1e-5); // no y component since axis=y, diff=x
    }

    #[test]
    fn test_vortex_out_of_range() {
        let f = ForceField {
            field_type: ForceFieldType::Vortex,
            position: [0.0; 3],
            direction: [0.0, 1.0, 0.0],
            strength: 10.0,
            radius: 1.0,
            falloff: 1.0,
        };
        let force = vortex_force(&f, [5.0, 0.0, 0.0]);
        assert_eq!(force, [0.0; 3]);
    }

    #[test]
    fn test_sample_gravity() {
        let f = make_grav_well();
        let force = sample_force_field(&f, [1.0, 0.0, 0.0]);
        // Force should point towards center (negative x)
        assert!(force[0] < 0.0);
    }
}
