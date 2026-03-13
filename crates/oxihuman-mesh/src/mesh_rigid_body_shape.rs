// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rigid body collision shape — specifies physics collision geometry for mesh rigid bodies.

/// Shape kind used for rigid body simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyShapeKind {
    Box,
    Sphere,
    Capsule,
    Cylinder,
    Cone,
    ConvexHull,
    Mesh,
}

/// Rigid body collision shape descriptor.
#[derive(Debug, Clone)]
pub struct RigidBodyShape {
    pub kind: RigidBodyShapeKind,
    /// [width_half, height_half, depth_half] or [radius, length, 0] for capsule/cylinder.
    pub dimensions: [f32; 3],
    pub mass: f32,
    pub friction: f32,
    pub restitution: f32,
    pub label: String,
}

/// Create a rigid body box shape.
pub fn new_box_shape(half_extents: [f32; 3], mass: f32, label: &str) -> RigidBodyShape {
    RigidBodyShape {
        kind: RigidBodyShapeKind::Box,
        dimensions: half_extents,
        mass: mass.max(0.0),
        friction: 0.5,
        restitution: 0.0,
        label: label.to_owned(),
    }
}

/// Create a rigid body sphere shape.
pub fn new_sphere_shape(radius: f32, mass: f32, label: &str) -> RigidBodyShape {
    RigidBodyShape {
        kind: RigidBodyShapeKind::Sphere,
        dimensions: [radius.max(0.0), 0.0, 0.0],
        mass: mass.max(0.0),
        friction: 0.5,
        restitution: 0.0,
        label: label.to_owned(),
    }
}

/// Create a convex hull rigid body shape.
pub fn new_convex_hull_shape(aabb_half: [f32; 3], mass: f32, label: &str) -> RigidBodyShape {
    RigidBodyShape {
        kind: RigidBodyShapeKind::ConvexHull,
        dimensions: aabb_half,
        mass: mass.max(0.0),
        friction: 0.5,
        restitution: 0.0,
        label: label.to_owned(),
    }
}

/// Return a name string for the shape kind.
pub fn shape_kind_name(s: &RigidBodyShape) -> &'static str {
    match s.kind {
        RigidBodyShapeKind::Box => "box",
        RigidBodyShapeKind::Sphere => "sphere",
        RigidBodyShapeKind::Capsule => "capsule",
        RigidBodyShapeKind::Cylinder => "cylinder",
        RigidBodyShapeKind::Cone => "cone",
        RigidBodyShapeKind::ConvexHull => "convex_hull",
        RigidBodyShapeKind::Mesh => "mesh",
    }
}

/// Is the body considered static (mass == 0)?
pub fn is_static(s: &RigidBodyShape) -> bool {
    s.mass < 1e-8
}

/// Approximate inertia scalar (simplified sphere formula).
pub fn approx_inertia(s: &RigidBodyShape) -> f32 {
    let r = s.dimensions[0];
    (2.0 / 5.0) * s.mass * r * r
}

/// Serialize to JSON-style string.
pub fn rigid_body_shape_to_json(s: &RigidBodyShape) -> String {
    format!(
        r#"{{"label":"{}", "kind":"{}", "mass":{:.4}, "friction":{:.4}, "restitution":{:.4}}}"#,
        s.label,
        shape_kind_name(s),
        s.mass,
        s.friction,
        s.restitution
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_shape_correct_kind() {
        /* box shape should have Box kind */
        let s = new_box_shape([1.0; 3], 1.0, "b");
        assert_eq!(s.kind, RigidBodyShapeKind::Box);
    }

    #[test]
    fn sphere_shape_radius_stored() {
        /* radius stored in dimensions[0] */
        let s = new_sphere_shape(2.0, 1.0, "s");
        assert!((s.dimensions[0] - 2.0).abs() < 1e-5);
    }

    #[test]
    fn negative_mass_clamped_to_zero() {
        /* negative mass should become 0 */
        let s = new_box_shape([1.0; 3], -5.0, "b");
        assert!(s.mass < 1e-8);
    }

    #[test]
    fn is_static_zero_mass() {
        /* mass 0 body is static */
        let s = new_box_shape([1.0; 3], 0.0, "b");
        assert!(is_static(&s));
    }

    #[test]
    fn is_static_nonzero_mass_false() {
        /* mass 1 body is not static */
        let s = new_box_shape([1.0; 3], 1.0, "b");
        assert!(!is_static(&s));
    }

    #[test]
    fn shape_kind_name_convex_hull() {
        /* convex hull name is "convex_hull" */
        let s = new_convex_hull_shape([1.0; 3], 1.0, "h");
        assert_eq!(shape_kind_name(&s), "convex_hull");
    }

    #[test]
    fn approx_inertia_positive_for_nonzero_mass() {
        /* inertia should be positive for nonzero mass */
        let s = new_sphere_shape(1.0, 5.0, "s");
        assert!(approx_inertia(&s) > 0.0);
    }

    #[test]
    fn json_contains_label() {
        /* JSON includes label */
        let s = new_box_shape([1.0; 3], 1.0, "myShape");
        assert!(rigid_body_shape_to_json(&s).contains("myShape"));
    }

    #[test]
    fn default_friction_is_half() {
        /* default friction is 0.5 */
        let s = new_box_shape([1.0; 3], 1.0, "b");
        assert!((s.friction - 0.5).abs() < 1e-5);
    }
}
