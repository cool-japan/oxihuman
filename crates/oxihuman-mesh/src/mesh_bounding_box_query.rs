// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Tight axis-aligned bounding box query for meshes stub.

/// An axis-aligned bounding box (AABB).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TightAabb {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl TightAabb {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> [f32; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    pub fn extents(&self) -> [f32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }

    pub fn volume(&self) -> f32 {
        let e = self.extents();
        e[0].max(0.0) * e[1].max(0.0) * e[2].max(0.0)
    }

    pub fn surface_area(&self) -> f32 {
        let e = self.extents();
        2.0 * (e[0] * e[1] + e[1] * e[2] + e[2] * e[0])
    }

    pub fn contains_point(&self, p: [f32; 3]) -> bool {
        (0..3).all(|k| p[k] >= self.min[k] && p[k] <= self.max[k])
    }

    pub fn overlaps(&self, other: &TightAabb) -> bool {
        (0..3).all(|k| self.min[k] <= other.max[k] && other.min[k] <= self.max[k])
    }

    pub fn expand_by(&self, delta: f32) -> TightAabb {
        TightAabb {
            min: [
                self.min[0] - delta,
                self.min[1] - delta,
                self.min[2] - delta,
            ],
            max: [
                self.max[0] + delta,
                self.max[1] + delta,
                self.max[2] + delta,
            ],
        }
    }
}

/// Compute the tight AABB of a set of vertices.
pub fn compute_tight_aabb(verts: &[[f32; 3]]) -> Option<TightAabb> {
    if verts.is_empty() {
        return None;
    }
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for v in verts {
        for k in 0..3 {
            if v[k] < mn[k] {
                mn[k] = v[k];
            }
            if v[k] > mx[k] {
                mx[k] = v[k];
            }
        }
    }
    Some(TightAabb::new(mn, mx))
}

/// Compute per-triangle bounding boxes.
pub fn triangle_aabbs(verts: &[[f32; 3]], tris: &[[u32; 3]]) -> Vec<TightAabb> {
    tris.iter()
        .filter_map(|tri| {
            let (i0, i1, i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
            if i0 >= verts.len() || i1 >= verts.len() || i2 >= verts.len() {
                return None;
            }
            let pts = [verts[i0], verts[i1], verts[i2]];
            let mut mn = [f32::MAX; 3];
            let mut mx = [f32::MIN; 3];
            for p in &pts {
                for k in 0..3 {
                    if p[k] < mn[k] {
                        mn[k] = p[k];
                    }
                    if p[k] > mx[k] {
                        mx[k] = p[k];
                    }
                }
            }
            Some(TightAabb::new(mn, mx))
        })
        .collect()
}

/// Diagonal length of the AABB (= maximum extent).
pub fn aabb_diagonal(verts: &[[f32; 3]]) -> f32 {
    if let Some(aabb) = compute_tight_aabb(verts) {
        let e = aabb.extents();
        (e[0] * e[0] + e[1] * e[1] + e[2] * e[2]).sqrt()
    } else {
        0.0
    }
}

/// Longest axis index (0=X, 1=Y, 2=Z) of the AABB.
pub fn aabb_longest_axis(verts: &[[f32; 3]]) -> usize {
    let Some(aabb) = compute_tight_aabb(verts) else {
        return 0;
    };
    let e = aabb.extents();
    if e[0] >= e[1] && e[0] >= e[2] {
        0
    } else if e[1] >= e[2] {
        1
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_center() {
        let aabb = TightAabb::new([0.0; 3], [2.0, 2.0, 2.0]);
        let c = aabb.center();
        assert!((c[0] - 1.0).abs() < 1e-5 /* center x=1 */);
    }

    #[test]
    fn test_aabb_volume() {
        let aabb = TightAabb::new([0.0; 3], [2.0, 3.0, 4.0]);
        assert!((aabb.volume() - 24.0).abs() < 1e-5 /* 2×3×4 = 24 */);
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb = TightAabb::new([0.0; 3], [1.0; 3]);
        assert!(aabb.contains_point([0.5, 0.5, 0.5]) /* inside */);
        assert!(!aabb.contains_point([2.0, 0.0, 0.0]) /* outside */);
    }

    #[test]
    fn test_aabb_overlaps() {
        let a = TightAabb::new([0.0; 3], [2.0; 3]);
        let b = TightAabb::new([1.0; 3], [3.0; 3]);
        assert!(a.overlaps(&b) /* overlapping */);
    }

    #[test]
    fn test_aabb_no_overlap() {
        let a = TightAabb::new([0.0; 3], [1.0; 3]);
        let b = TightAabb::new([2.0; 3], [3.0; 3]);
        assert!(!a.overlaps(&b) /* non-overlapping */);
    }

    #[test]
    fn test_compute_tight_aabb_empty() {
        assert!(compute_tight_aabb(&[]).is_none() /* empty vertices */);
    }

    #[test]
    fn test_compute_tight_aabb_values() {
        let verts = vec![[0.0f32, 1.0, 2.0], [3.0, 0.0, -1.0]];
        let aabb = compute_tight_aabb(&verts).unwrap();
        assert_eq!(aabb.min, [0.0, 0.0, -1.0] /* correct min */);
        assert_eq!(aabb.max, [3.0, 1.0, 2.0] /* correct max */);
    }

    #[test]
    fn test_aabb_diagonal_nonzero() {
        let verts = vec![[0.0f32; 3], [1.0, 0.0, 0.0]];
        assert!(aabb_diagonal(&verts) > 0.0 /* non-zero diagonal */);
    }

    #[test]
    fn test_aabb_longest_axis() {
        let verts = vec![[0.0f32, 0.0, 0.0], [5.0, 1.0, 1.0]]; /* X is longest */
        assert_eq!(aabb_longest_axis(&verts), 0 /* X axis */);
    }
}
