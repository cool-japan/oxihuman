// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Uniform force field: constant force applied over a region.

/// Axis-aligned box region.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct FieldRegion {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Uniform force field.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UniformForceField {
    pub force: [f32; 3],
    pub region: Option<FieldRegion>,
    pub enabled: bool,
    pub scale: f32,
}

/// Create an unbounded uniform force field.
#[allow(dead_code)]
pub fn new_uniform_field(force: [f32; 3]) -> UniformForceField {
    UniformForceField {
        force,
        region: None,
        enabled: true,
        scale: 1.0,
    }
}

/// Create a bounded uniform force field.
#[allow(dead_code)]
pub fn new_bounded_field(force: [f32; 3], region: FieldRegion) -> UniformForceField {
    UniformForceField {
        force,
        region: Some(region),
        enabled: true,
        scale: 1.0,
    }
}

/// Whether a position is inside the field region.
#[allow(dead_code)]
pub fn field_contains(field: &UniformForceField, pos: [f32; 3]) -> bool {
    match &field.region {
        None => true,
        Some(r) => {
            (r.min[0]..=r.max[0]).contains(&pos[0])
                && (r.min[1]..=r.max[1]).contains(&pos[1])
                && (r.min[2]..=r.max[2]).contains(&pos[2])
        }
    }
}

/// Compute force on a body at position `pos` with given mass.
#[allow(dead_code)]
pub fn field_force_at(field: &UniformForceField, pos: [f32; 3], mass: f32) -> [f32; 3] {
    if !field.enabled || !field_contains(field, pos) {
        return [0.0; 3];
    }
    let s = field.scale * mass;
    [field.force[0] * s, field.force[1] * s, field.force[2] * s]
}

/// Acceleration (force/mass) at position.
#[allow(dead_code)]
pub fn field_acceleration_at(field: &UniformForceField, pos: [f32; 3]) -> [f32; 3] {
    if !field.enabled || !field_contains(field, pos) {
        return [0.0; 3];
    }
    [
        field.force[0] * field.scale,
        field.force[1] * field.scale,
        field.force[2] * field.scale,
    ]
}

/// Enable or disable the field.
#[allow(dead_code)]
pub fn field_set_enabled(field: &mut UniformForceField, enabled: bool) {
    field.enabled = enabled;
}

/// Set field scale.
#[allow(dead_code)]
pub fn field_set_scale(field: &mut UniformForceField, scale: f32) {
    field.scale = scale;
}

/// Force magnitude.
#[allow(dead_code)]
pub fn field_magnitude(field: &UniformForceField) -> f32 {
    let f = field.force;
    (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt() * field.scale.abs()
}

/// Work done moving mass through displacement.
#[allow(dead_code)]
pub fn field_work(field: &UniformForceField, displacement: [f32; 3], mass: f32) -> f32 {
    let f = field_force_at(field, [0.0; 3], mass);
    f[0] * displacement[0] + f[1] * displacement[1] + f[2] * displacement[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unbounded_force() {
        let f = new_uniform_field([0.0, -9.81, 0.0]);
        let force = field_force_at(&f, [100.0, 100.0, 100.0], 2.0);
        assert!((force[1] - (-19.62_f32)).abs() < 1e-4);
    }

    #[test]
    fn test_outside_region_zero() {
        let region = FieldRegion {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        let f = new_bounded_field([1.0, 0.0, 0.0], region);
        let force = field_force_at(&f, [5.0, 0.0, 0.0], 1.0);
        assert_eq!(force, [0.0; 3]);
    }

    #[test]
    fn test_inside_region_force() {
        let region = FieldRegion {
            min: [0.0; 3],
            max: [10.0; 3],
        };
        let f = new_bounded_field([1.0, 0.0, 0.0], region);
        let force = field_force_at(&f, [5.0, 5.0, 5.0], 1.0);
        assert!((force[0] - 1.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_disabled_field() {
        let mut f = new_uniform_field([1.0, 0.0, 0.0]);
        field_set_enabled(&mut f, false);
        let force = field_force_at(&f, [0.0; 3], 1.0);
        assert_eq!(force, [0.0; 3]);
    }

    #[test]
    fn test_scale() {
        let mut f = new_uniform_field([1.0, 0.0, 0.0]);
        field_set_scale(&mut f, 2.0);
        let force = field_force_at(&f, [0.0; 3], 1.0);
        assert!((force[0] - 2.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_magnitude() {
        let f = new_uniform_field([3.0, 4.0, 0.0]);
        assert!((field_magnitude(&f) - 5.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_work() {
        let f = new_uniform_field([1.0, 0.0, 0.0]);
        let w = field_work(&f, [5.0, 0.0, 0.0], 1.0);
        assert!((w - 5.0_f32).abs() < 1e-5);
    }

    #[test]
    fn test_contains_boundary() {
        let region = FieldRegion {
            min: [0.0; 3],
            max: [1.0; 3],
        };
        let f = new_bounded_field([1.0, 0.0, 0.0], region);
        assert!(field_contains(&f, [0.0, 0.0, 0.0]));
        assert!(field_contains(&f, [1.0, 1.0, 1.0]));
        assert!(!field_contains(&f, [1.1, 0.0, 0.0]));
    }

    #[test]
    fn test_acceleration() {
        let f = new_uniform_field([0.0, -9.81, 0.0]);
        let a = field_acceleration_at(&f, [0.0; 3]);
        assert!((a[1] - (-9.81_f32)).abs() < 1e-4);
    }
}
