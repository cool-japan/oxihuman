// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Mesh bounding volumes: AABB, bounding sphere, and OBB approximation.
//!
//! Used by physics, LOD selection, and frustum culling.

// ─────────────────────────────────────────────────────────────────────────────
// AABB
// ─────────────────────────────────────────────────────────────────────────────

/// Axis-Aligned Bounding Box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Aabb {
    /// Compute AABB from a slice of positions.
    ///
    /// Returns `None` if `positions` is empty.
    pub fn from_positions(positions: &[[f32; 3]]) -> Option<Self> {
        if positions.is_empty() {
            return None;
        }
        let mut min = positions[0];
        let mut max = positions[0];
        for p in positions.iter().skip(1) {
            for i in 0..3 {
                if p[i] < min[i] {
                    min[i] = p[i];
                }
                if p[i] > max[i] {
                    max[i] = p[i];
                }
            }
        }
        Some(Aabb { min, max })
    }

    /// Returns `None` if no positions, else computes min/max.
    /// Skips NaN/Inf values.
    pub fn from_positions_finite(positions: &[[f32; 3]]) -> Option<Self> {
        let finite: Vec<[f32; 3]> = positions
            .iter()
            .copied()
            .filter(|p| p.iter().all(|v| v.is_finite()))
            .collect();
        Self::from_positions(&finite)
    }

    /// Center point of the AABB.
    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    /// Half-extents (half the size on each axis).
    pub fn half_extents(&self) -> [f32; 3] {
        [
            (self.max[0] - self.min[0]) * 0.5,
            (self.max[1] - self.min[1]) * 0.5,
            (self.max[2] - self.min[2]) * 0.5,
        ]
    }

    /// Total size on each axis.
    pub fn size(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    /// Volume of the AABB.
    pub fn volume(&self) -> f32 {
        let s = self.size();
        s[0] * s[1] * s[2]
    }

    /// Check if a point is inside (or on the boundary of) the AABB.
    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        p[0] >= self.min[0]
            && p[0] <= self.max[0]
            && p[1] >= self.min[1]
            && p[1] <= self.max[1]
            && p[2] >= self.min[2]
            && p[2] <= self.max[2]
    }

    /// Check if two AABBs overlap.
    pub fn overlaps(&self, other: &Aabb) -> bool {
        self.max[0] >= other.min[0]
            && self.min[0] <= other.max[0]
            && self.max[1] >= other.min[1]
            && self.min[1] <= other.max[1]
            && self.max[2] >= other.min[2]
            && self.min[2] <= other.max[2]
    }

    /// Expand this AABB to also contain `other`.
    pub fn union(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bounding Sphere
// ─────────────────────────────────────────────────────────────────────────────

/// Bounding sphere.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingSphere {
    pub center: [f32; 3],
    pub radius: f32,
}

#[inline]
fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[inline]
fn midpoint3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ]
}

impl BoundingSphere {
    /// Ritter's algorithm: fast approximate bounding sphere.
    ///
    /// Returns `None` if `positions` is empty.
    pub fn from_positions(positions: &[[f32; 3]]) -> Option<Self> {
        if positions.is_empty() {
            return None;
        }
        if positions.len() == 1 {
            return Some(BoundingSphere {
                center: positions[0],
                radius: 0.0,
            });
        }

        // Step 1: pick p0 = positions[0]
        let p0 = positions[0];

        // Step 2: find p1 = point farthest from p0
        let p1 = *positions
            .iter()
            .max_by(|a, b| {
                dist3(**a, p0)
                    .partial_cmp(&dist3(**b, p0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&p0);

        // Step 3: find p2 = point farthest from p1
        let p2 = *positions
            .iter()
            .max_by(|a, b| {
                dist3(**a, p1)
                    .partial_cmp(&dist3(**b, p1))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(&p1);

        // Step 4: initial sphere
        let mut center = midpoint3(p1, p2);
        let mut radius = dist3(p1, p2) * 0.5;

        // Step 5: expand to include all points
        for &p in positions {
            let d = dist3(p, center);
            if d > radius {
                // Expand sphere
                let new_radius = (radius + d) * 0.5;
                let t = (d - radius) / (2.0 * d);
                center = [
                    center[0] + t * (p[0] - center[0]),
                    center[1] + t * (p[1] - center[1]),
                    center[2] + t * (p[2] - center[2]),
                ];
                radius = new_radius;
            }
        }

        Some(BoundingSphere { center, radius })
    }

    /// Check if a point is inside (or on the boundary of) the sphere.
    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        dist3(p, self.center) <= self.radius + f32::EPSILON
    }

    /// Check if two spheres overlap.
    pub fn overlaps(&self, other: &BoundingSphere) -> bool {
        let d = dist3(self.center, other.center);
        d <= self.radius + other.radius
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// OBB
// ─────────────────────────────────────────────────────────────────────────────

/// Simple OBB approximation using AABB-aligned axes.
///
/// For a true OBB you would need PCA on the covariance matrix; this version
/// uses world-aligned axes (equivalent to an AABB expressed in OBB form).
#[derive(Debug, Clone, Copy)]
pub struct Obb {
    pub center: [f32; 3],
    /// Half-extents along each local axis.
    pub half_extents: [f32; 3],
    /// Column-major 3×3 rotation matrix (local axes as columns). Identity here.
    pub axes: [[f32; 3]; 3],
}

impl Obb {
    /// Build an OBB aligned with world axes (same as AABB, but in OBB form).
    pub fn from_aabb(aabb: &Aabb) -> Self {
        Obb {
            center: aabb.center(),
            half_extents: aabb.half_extents(),
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// Volume of the OBB.
    pub fn volume(&self) -> f32 {
        let he = self.half_extents;
        8.0 * he[0] * he[1] * he[2]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// compute_bounds
// ─────────────────────────────────────────────────────────────────────────────

/// All three bounding volumes computed in a single pass.
pub struct BoundsResult {
    pub aabb: Option<Aabb>,
    pub sphere: Option<BoundingSphere>,
    pub obb: Option<Obb>,
}

/// Compute all three bounding volumes at once (more efficient than separate calls).
pub fn compute_bounds(positions: &[[f32; 3]]) -> BoundsResult {
    let aabb = Aabb::from_positions(positions);
    let sphere = BoundingSphere::from_positions(positions);
    let obb = aabb.as_ref().map(Obb::from_aabb);
    BoundsResult { aabb, sphere, obb }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_cube_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
        ]
    }

    #[test]
    fn aabb_from_unit_cube() {
        let aabb = Aabb::from_positions(&unit_cube_positions()).unwrap();
        assert_eq!(aabb.min, [0.0, 0.0, 0.0]);
        assert_eq!(aabb.max, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn aabb_center_correct() {
        // [0..2] cube → center = [1,1,1]
        let positions: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 0.0, 2.0],
            [2.0, 0.0, 2.0],
            [0.0, 2.0, 2.0],
            [2.0, 2.0, 2.0],
        ];
        let aabb = Aabb::from_positions(&positions).unwrap();
        let c = aabb.center();
        assert!((c[0] - 1.0).abs() < 1e-6);
        assert!((c[1] - 1.0).abs() < 1e-6);
        assert!((c[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn aabb_volume_correct() {
        // AABB of size [2, 3, 4] → volume = 24
        let aabb = Aabb {
            min: [0.0, 0.0, 0.0],
            max: [2.0, 3.0, 4.0],
        };
        assert!((aabb.volume() - 24.0).abs() < 1e-5);
    }

    #[test]
    fn aabb_contains_point() {
        let aabb = Aabb::from_positions(&unit_cube_positions()).unwrap();
        let center = aabb.center();
        assert!(aabb.contains_point(center), "center should be inside");
        assert!(!aabb.contains_point([2.0, 0.5, 0.5]), "outside X");
        assert!(!aabb.contains_point([0.5, -1.0, 0.5]), "outside Y");
    }

    #[test]
    fn aabb_overlaps() {
        let a = Aabb {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        };
        let b = Aabb {
            min: [0.5, 0.5, 0.5],
            max: [1.5, 1.5, 1.5],
        };
        let c = Aabb {
            min: [2.0, 2.0, 2.0],
            max: [3.0, 3.0, 3.0],
        };
        assert!(a.overlaps(&b), "a and b should overlap");
        assert!(!a.overlaps(&c), "a and c should not overlap");
    }

    #[test]
    fn aabb_union() {
        let a = Aabb {
            min: [0.0, 0.0, 0.0],
            max: [1.0, 1.0, 1.0],
        };
        let b = Aabb {
            min: [2.0, 2.0, 2.0],
            max: [3.0, 3.0, 3.0],
        };
        let u = a.union(&b);
        assert_eq!(u.min, [0.0, 0.0, 0.0]);
        assert_eq!(u.max, [3.0, 3.0, 3.0]);
        assert!(u.contains_point([0.5, 0.5, 0.5]));
        assert!(u.contains_point([2.5, 2.5, 2.5]));
    }

    #[test]
    fn bounding_sphere_contains_center() {
        let positions = unit_cube_positions();
        let sphere = BoundingSphere::from_positions(&positions).unwrap();
        // The centroid of the unit cube is [0.5, 0.5, 0.5]
        assert!(
            sphere.contains_point([0.5, 0.5, 0.5]),
            "sphere should contain the centroid"
        );
    }

    #[test]
    fn bounding_sphere_contains_all_points() {
        let positions = unit_cube_positions();
        let sphere = BoundingSphere::from_positions(&positions).unwrap();
        for &p in &positions {
            assert!(
                sphere.contains_point(p),
                "sphere should contain point {:?}",
                p
            );
        }
    }

    #[test]
    fn obb_from_aabb_volume() {
        let aabb = Aabb {
            min: [0.0, 0.0, 0.0],
            max: [2.0, 3.0, 4.0],
        };
        let obb = Obb::from_aabb(&aabb);
        assert!(
            (obb.volume() - aabb.volume()).abs() < 1e-4,
            "OBB volume {} should match AABB volume {}",
            obb.volume(),
            aabb.volume()
        );
    }

    #[test]
    fn aabb_from_empty_returns_none() {
        assert!(Aabb::from_positions(&[]).is_none());
    }

    #[test]
    fn compute_bounds_all_three() {
        let positions = unit_cube_positions();
        let result = compute_bounds(&positions);
        assert!(
            result.aabb.is_some(),
            "aabb should be Some for non-empty input"
        );
        assert!(
            result.sphere.is_some(),
            "sphere should be Some for non-empty input"
        );
        assert!(
            result.obb.is_some(),
            "obb should be Some for non-empty input"
        );
    }
}
