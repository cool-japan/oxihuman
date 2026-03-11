#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! AABB shape bounds utilities.

/// An axis-aligned bounding box.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct ShapeBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// Create a new `ShapeBounds` from min/max corners.
#[allow(dead_code)]
pub fn new_shape_bounds(min: [f32; 3], max: [f32; 3]) -> ShapeBounds {
    ShapeBounds { min, max }
}

/// Create bounds for a sphere.
#[allow(dead_code)]
pub fn sphere_bounds(center: [f32; 3], radius: f32) -> ShapeBounds {
    ShapeBounds {
        min: [center[0] - radius, center[1] - radius, center[2] - radius],
        max: [center[0] + radius, center[1] + radius, center[2] + radius],
    }
}

/// Create bounds for an axis-aligned box.
#[allow(dead_code)]
pub fn box_bounds(center: [f32; 3], half_extents: [f32; 3]) -> ShapeBounds {
    ShapeBounds {
        min: [center[0] - half_extents[0], center[1] - half_extents[1], center[2] - half_extents[2]],
        max: [center[0] + half_extents[0], center[1] + half_extents[1], center[2] + half_extents[2]],
    }
}

/// Create bounds for a capsule (aligned along Y axis).
#[allow(dead_code)]
pub fn capsule_bounds(center: [f32; 3], radius: f32, half_height: f32) -> ShapeBounds {
    ShapeBounds {
        min: [center[0] - radius, center[1] - half_height - radius, center[2] - radius],
        max: [center[0] + radius, center[1] + half_height + radius, center[2] + radius],
    }
}

/// Compute the union of two bounds.
#[allow(dead_code)]
pub fn bounds_union(a: &ShapeBounds, b: &ShapeBounds) -> ShapeBounds {
    ShapeBounds {
        min: [a.min[0].min(b.min[0]), a.min[1].min(b.min[1]), a.min[2].min(b.min[2])],
        max: [a.max[0].max(b.max[0]), a.max[1].max(b.max[1]), a.max[2].max(b.max[2])],
    }
}

/// Return true if two bounds intersect.
#[allow(dead_code)]
pub fn bounds_intersect(a: &ShapeBounds, b: &ShapeBounds) -> bool {
    a.min[0] <= b.max[0] && a.max[0] >= b.min[0]
    && a.min[1] <= b.max[1] && a.max[1] >= b.min[1]
    && a.min[2] <= b.max[2] && a.max[2] >= b.min[2]
}

/// Return true if a point is inside the bounds.
#[allow(dead_code)]
pub fn bounds_contains_point(bounds: &ShapeBounds, p: [f32; 3]) -> bool {
    (bounds.min[0]..=bounds.max[0]).contains(&p[0])
    && (bounds.min[1]..=bounds.max[1]).contains(&p[1])
    && (bounds.min[2]..=bounds.max[2]).contains(&p[2])
}

/// Expand bounds by a margin in all directions.
#[allow(dead_code)]
pub fn bounds_expand(bounds: &ShapeBounds, margin: f32) -> ShapeBounds {
    ShapeBounds {
        min: [bounds.min[0] - margin, bounds.min[1] - margin, bounds.min[2] - margin],
        max: [bounds.max[0] + margin, bounds.max[1] + margin, bounds.max[2] + margin],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_shape_bounds() {
        let b = new_shape_bounds([0.0; 3], [1.0; 3]);
        assert_eq!(b.min, [0.0; 3]);
    }

    #[test]
    fn test_sphere_bounds() {
        let b = sphere_bounds([0.0; 3], 1.0);
        assert!((b.min[0] + 1.0).abs() < 1e-6);
        assert!((b.max[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_box_bounds() {
        let b = box_bounds([0.0; 3], [2.0, 1.0, 0.5]);
        assert!((b.max[0] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_capsule_bounds() {
        let b = capsule_bounds([0.0; 3], 0.5, 1.0);
        assert!((b.max[1] - 1.5).abs() < 1e-6);
    }

    #[test]
    fn test_bounds_union() {
        let a = new_shape_bounds([-1.0; 3], [0.0; 3]);
        let b = new_shape_bounds([0.0; 3], [1.0; 3]);
        let u = bounds_union(&a, &b);
        assert!((u.min[0] + 1.0).abs() < 1e-6);
        assert!((u.max[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_bounds_intersect() {
        let a = new_shape_bounds([0.0; 3], [2.0; 3]);
        let b = new_shape_bounds([1.0; 3], [3.0; 3]);
        assert!(bounds_intersect(&a, &b));
        let c = new_shape_bounds([5.0; 3], [6.0; 3]);
        assert!(!bounds_intersect(&a, &c));
    }

    #[test]
    fn test_bounds_contains_point() {
        let b = new_shape_bounds([0.0; 3], [1.0; 3]);
        assert!(bounds_contains_point(&b, [0.5, 0.5, 0.5]));
        assert!(!bounds_contains_point(&b, [2.0, 0.5, 0.5]));
    }

    #[test]
    fn test_bounds_expand() {
        let b = new_shape_bounds([0.0; 3], [1.0; 3]);
        let expanded = bounds_expand(&b, 0.5);
        assert!((expanded.min[0] + 0.5).abs() < 1e-6);
        assert!((expanded.max[0] - 1.5).abs() < 1e-6);
    }
}
