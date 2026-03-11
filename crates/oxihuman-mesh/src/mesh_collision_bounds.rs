// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Collision bounds (box/sphere/hull) — describes simple collision geometry for a mesh.

/// Type of collision primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionBoundsType {
    Box,
    Sphere,
    ConvexHull,
    Capsule,
}

/// Collision bounds descriptor.
#[derive(Debug, Clone)]
pub struct CollisionBounds {
    pub bounds_type: CollisionBoundsType,
    /// Half-extents (for Box) or radius (for Sphere, index 0), height (Capsule, index 1).
    pub extents: [f32; 3],
    pub margin: f32,
    pub label: String,
}

/// Create a box collision bounds.
pub fn new_box_bounds(half_extents: [f32; 3], label: &str) -> CollisionBounds {
    CollisionBounds {
        bounds_type: CollisionBoundsType::Box,
        extents: half_extents,
        margin: 0.04,
        label: label.to_owned(),
    }
}

/// Create a sphere collision bounds.
pub fn new_sphere_bounds(radius: f32, label: &str) -> CollisionBounds {
    CollisionBounds {
        bounds_type: CollisionBoundsType::Sphere,
        extents: [radius.max(0.0), 0.0, 0.0],
        margin: 0.04,
        label: label.to_owned(),
    }
}

/// Create a convex hull collision bounds placeholder.
pub fn new_hull_bounds(aabb_half: [f32; 3], label: &str) -> CollisionBounds {
    CollisionBounds {
        bounds_type: CollisionBoundsType::ConvexHull,
        extents: aabb_half,
        margin: 0.04,
        label: label.to_owned(),
    }
}

/// Approximate volume of the collision bounds.
pub fn bounds_volume(b: &CollisionBounds) -> f32 {
    match b.bounds_type {
        CollisionBoundsType::Box => 8.0 * b.extents[0] * b.extents[1] * b.extents[2],
        CollisionBoundsType::Sphere => (4.0 / 3.0) * std::f32::consts::PI * b.extents[0].powi(3),
        CollisionBoundsType::ConvexHull => {
            /* approximate as bounding box */
            8.0 * b.extents[0] * b.extents[1] * b.extents[2]
        }
        CollisionBoundsType::Capsule => {
            let r = b.extents[0];
            let h = b.extents[1];
            std::f32::consts::PI * r * r * h + (4.0 / 3.0) * std::f32::consts::PI * r.powi(3)
        }
    }
}

/// Name of the bounds type.
pub fn bounds_type_name(b: &CollisionBounds) -> &'static str {
    match b.bounds_type {
        CollisionBoundsType::Box => "box",
        CollisionBoundsType::Sphere => "sphere",
        CollisionBoundsType::ConvexHull => "convex_hull",
        CollisionBoundsType::Capsule => "capsule",
    }
}

/// Serialize to JSON-style string.
pub fn collision_bounds_to_json(b: &CollisionBounds) -> String {
    format!(
        r#"{{"label":"{}", "type":"{}", "extents":[{:.4},{:.4},{:.4}], "margin":{:.4}}}"#,
        b.label,
        bounds_type_name(b),
        b.extents[0],
        b.extents[1],
        b.extents[2],
        b.margin
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_volume_correct() {
        /* 2x2x2 box has volume 8 */
        let b = new_box_bounds([1.0, 1.0, 1.0], "box");
        assert!((bounds_volume(&b) - 8.0).abs() < 1e-4);
    }

    #[test]
    fn sphere_volume_correct() {
        /* unit sphere volume = 4/3 * pi */
        let b = new_sphere_bounds(1.0, "sph");
        let expected = (4.0 / 3.0) * std::f32::consts::PI;
        assert!((bounds_volume(&b) - expected).abs() < 1e-4);
    }

    #[test]
    fn bounds_type_name_box() {
        /* box type name is "box" */
        let b = new_box_bounds([1.0; 3], "b");
        assert_eq!(bounds_type_name(&b), "box");
    }

    #[test]
    fn bounds_type_name_sphere() {
        /* sphere type name is "sphere" */
        let b = new_sphere_bounds(1.0, "s");
        assert_eq!(bounds_type_name(&b), "sphere");
    }

    #[test]
    fn bounds_type_name_hull() {
        /* convex hull type name is "convex_hull" */
        let b = new_hull_bounds([1.0; 3], "h");
        assert_eq!(bounds_type_name(&b), "convex_hull");
    }

    #[test]
    fn json_contains_label() {
        /* JSON output includes label */
        let b = new_box_bounds([1.0; 3], "myBounds");
        assert!(collision_bounds_to_json(&b).contains("myBounds"));
    }

    #[test]
    fn default_margin_is_nonzero() {
        /* default margin should be 0.04 */
        let b = new_box_bounds([1.0; 3], "b");
        assert!((b.margin - 0.04).abs() < 1e-5);
    }

    #[test]
    fn sphere_radius_stored_in_extents_x() {
        /* sphere radius should appear in extents[0] */
        let b = new_sphere_bounds(2.5, "s");
        assert!((b.extents[0] - 2.5).abs() < 1e-5);
    }

    #[test]
    fn hull_is_convex_hull_type() {
        /* hull bounds should have ConvexHull type */
        let b = new_hull_bounds([1.0; 3], "h");
        assert_eq!(b.bounds_type, CollisionBoundsType::ConvexHull);
    }
}
