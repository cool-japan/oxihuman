// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cloth-mesh collision geometry helpers.

/// Axis-aligned bounding box for quick broad-phase rejection.
#[derive(Debug, Clone, PartialEq)]
pub struct ColliderAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// A cloth collision mesh with per-triangle data.
#[derive(Debug, Clone)]
pub struct ClothCollider {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub friction: f32,
    pub restitution: f32,
}

impl ClothCollider {
    /// Create a new collider from mesh data.
    pub fn new(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Self {
        Self {
            positions,
            indices,
            friction: 0.5,
            restitution: 0.2,
        }
    }

    /// Return the number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// Build an AABB for the collider.
pub fn collider_aabb(c: &ClothCollider) -> ColliderAabb {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in &c.positions {
        for k in 0..3 {
            if p[k] < min[k] {
                min[k] = p[k];
            }
            if p[k] > max[k] {
                max[k] = p[k];
            }
        }
    }
    ColliderAabb { min, max }
}

/// Test if a point is inside the AABB.
pub fn point_in_aabb(aabb: &ColliderAabb, p: [f32; 3]) -> bool {
    (0..3).all(|k| p[k] >= aabb.min[k] && p[k] <= aabb.max[k])
}

/// Compute the closest point on a triangle to a query point.
pub fn closest_point_on_tri(a: [f32; 3], b: [f32; 3], c: [f32; 3], p: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let d1 = dot3(ab, ap);
    let d2 = dot3(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = [p[0] - b[0], p[1] - b[1], p[2] - b[2]];
    let d3 = dot3(ab, bp);
    let d4 = dot3(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let cp = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
    let d5 = dot3(ab, cp);
    let d6 = dot3(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return [a[0] + v * ab[0], a[1] + v * ab[1], a[2] + v * ab[2]];
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return [a[0] + w * ac[0], a[1] + w * ac[1], a[2] + w * ac[2]];
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return [
            b[0] + w * (c[0] - b[0]),
            b[1] + w * (c[1] - b[1]),
            b[2] + w * (c[2] - b[2]),
        ];
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    [
        a[0] + v * ab[0] + w * ac[0],
        a[1] + v * ab[1] + w * ac[1],
        a[2] + v * ab[2] + w * ac[2],
    ]
}

/// Inflate a collider AABB by `margin` on each side.
pub fn inflate_aabb(aabb: &ColliderAabb, margin: f32) -> ColliderAabb {
    ColliderAabb {
        min: [
            aabb.min[0] - margin,
            aabb.min[1] - margin,
            aabb.min[2] - margin,
        ],
        max: [
            aabb.max[0] + margin,
            aabb.max[1] + margin,
            aabb.max[2] + margin,
        ],
    }
}

/// Validate that the collider has consistent geometry.
pub fn validate_collider(c: &ClothCollider) -> bool {
    !c.positions.is_empty()
        && c.indices.len().is_multiple_of(3)
        && c.indices.iter().all(|&i| (i as usize) < c.positions.len())
        && (0.0..=1.0).contains(&c.friction)
        && (0.0..=1.0).contains(&c.restitution)
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_collider() -> ClothCollider {
        ClothCollider::new(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![0, 1, 2],
        )
    }

    #[test]
    fn triangle_count_correct() {
        /* one triangle */
        assert_eq!(simple_collider().triangle_count(), 1);
    }

    #[test]
    fn aabb_computed() {
        /* check min/max */
        let aabb = collider_aabb(&simple_collider());
        assert!(aabb.min[0] <= 0.0);
        assert!(aabb.max[0] >= 1.0);
    }

    #[test]
    fn point_in_aabb_true() {
        /* origin is inside */
        let aabb = collider_aabb(&simple_collider());
        assert!(point_in_aabb(&aabb, [0.5, 0.4, 0.0]));
    }

    #[test]
    fn point_in_aabb_false() {
        /* far point is outside */
        let aabb = collider_aabb(&simple_collider());
        assert!(!point_in_aabb(&aabb, [5.0, 5.0, 5.0]));
    }

    #[test]
    fn inflate_aabb_grows() {
        /* inflated min should be smaller */
        let aabb = collider_aabb(&simple_collider());
        let big = inflate_aabb(&aabb, 1.0);
        assert!(big.min[0] < aabb.min[0]);
        assert!(big.max[0] > aabb.max[0]);
    }

    #[test]
    fn closest_point_on_tri_vertex() {
        /* point far away → nearest is vertex a */
        let pt = closest_point_on_tri(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [-5.0, -5.0, 0.0],
        );
        assert!((pt[0]).abs() < 1e-5);
        assert!((pt[1]).abs() < 1e-5);
    }

    #[test]
    fn validate_collider_ok() {
        /* default collider should be valid */
        assert!(validate_collider(&simple_collider()));
    }

    #[test]
    fn validate_collider_bad_index() {
        /* out-of-range index → invalid */
        let mut c = simple_collider();
        c.indices[0] = 99;
        assert!(!validate_collider(&c));
    }

    #[test]
    fn validate_friction_range() {
        /* friction outside 0-1 → invalid */
        let mut c = simple_collider();
        c.friction = 1.5;
        assert!(!validate_collider(&c));
    }

    #[test]
    fn new_defaults() {
        /* default friction is 0.5 */
        let c = simple_collider();
        assert!((c.friction - 0.5).abs() < 1e-6);
    }
}
