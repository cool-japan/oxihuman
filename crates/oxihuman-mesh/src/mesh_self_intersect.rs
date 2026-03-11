// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Self-intersection detection for triangle meshes.

/// A pair of triangle face indices that intersect each other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntersectingPair {
    pub face_a: usize,
    pub face_b: usize,
}

/// Result of a self-intersection detection pass.
#[derive(Debug, Clone, Default)]
pub struct SelfIntersectResult {
    pub pairs: Vec<IntersectingPair>,
}

impl SelfIntersectResult {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_clean(&self) -> bool {
        self.pairs.is_empty()
    }
}

/// Detect self-intersecting triangle pairs in a mesh (brute-force O(n²)).
pub fn detect_self_intersections(positions: &[[f32; 3]], indices: &[u32]) -> SelfIntersectResult {
    let n_faces = indices.len() / 3;
    let mut result = SelfIntersectResult::new();
    for i in 0..n_faces {
        let ta = get_tri(positions, indices, i);
        for j in (i + 1)..n_faces {
            if shares_vertex(indices, i, j) {
                continue;
            }
            let tb = get_tri(positions, indices, j);
            if triangles_intersect(ta, tb) {
                result.pairs.push(IntersectingPair {
                    face_a: i,
                    face_b: j,
                });
            }
        }
    }
    result
}

/// Count of intersecting pairs.
pub fn intersection_pair_count(r: &SelfIntersectResult) -> usize {
    r.pairs.len()
}

/// True if the mesh has no detected self-intersections.
pub fn is_self_intersection_free(r: &SelfIntersectResult) -> bool {
    r.is_clean()
}

/// All face indices involved in at least one intersection.
pub fn intersecting_faces(r: &SelfIntersectResult) -> Vec<usize> {
    let mut faces: Vec<usize> = r.pairs.iter().flat_map(|p| [p.face_a, p.face_b]).collect();
    faces.sort_unstable();
    faces.dedup();
    faces
}

/// Fraction of faces involved in intersections.
pub fn intersection_face_fraction(r: &SelfIntersectResult, total_faces: usize) -> f32 {
    if total_faces == 0 {
        return 0.0;
    }
    intersecting_faces(r).len() as f32 / total_faces as f32
}

fn get_tri(positions: &[[f32; 3]], indices: &[u32], i: usize) -> [[f32; 3]; 3] {
    [
        positions[indices[i * 3] as usize],
        positions[indices[i * 3 + 1] as usize],
        positions[indices[i * 3 + 2] as usize],
    ]
}

fn shares_vertex(indices: &[u32], i: usize, j: usize) -> bool {
    let ta = &indices[i * 3..i * 3 + 3];
    let tb = &indices[j * 3..j * 3 + 3];
    ta.iter().any(|v| tb.contains(v))
}

fn triangles_intersect(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> bool {
    /* Möller triangle-triangle intersection (simplified AABB pre-check) */
    let aabb_a = tri_aabb(a);
    let aabb_b = tri_aabb(b);
    if !aabbs_overlap(aabb_a, aabb_b) {
        return false;
    }
    /* full SAT is complex; use signed volume test as approximation */
    let n_a = tri_normal(a);
    let d_a = dot3(n_a, a[0]);
    let signs_b: Vec<f32> = b.iter().map(|v| dot3(n_a, *v) - d_a).collect();
    if signs_b.iter().all(|&s| s > 1e-8) || signs_b.iter().all(|&s| s < -1e-8) {
        return false;
    }
    let n_b = tri_normal(b);
    let d_b = dot3(n_b, b[0]);
    let signs_a: Vec<f32> = a.iter().map(|v| dot3(n_b, *v) - d_b).collect();
    if signs_a.iter().all(|&s| s > 1e-8) || signs_a.iter().all(|&s| s < -1e-8) {
        return false;
    }
    true
}

fn tri_aabb(t: [[f32; 3]; 3]) -> ([f32; 3], [f32; 3]) {
    let mn = [
        t[0][0].min(t[1][0]).min(t[2][0]),
        t[0][1].min(t[1][1]).min(t[2][1]),
        t[0][2].min(t[1][2]).min(t[2][2]),
    ];
    let mx = [
        t[0][0].max(t[1][0]).max(t[2][0]),
        t[0][1].max(t[1][1]).max(t[2][1]),
        t[0][2].max(t[1][2]).max(t[2][2]),
    ];
    (mn, mx)
}

fn aabbs_overlap(a: ([f32; 3], [f32; 3]), b: ([f32; 3], [f32; 3])) -> bool {
    for i in 0..3 {
        if a.1[i] < b.0[i] || b.1[i] < a.0[i] {
            return false;
        }
    }
    true
}

fn tri_normal(t: [[f32; 3]; 3]) -> [f32; 3] {
    let ab = sub3(t[1], t[0]);
    let ac = sub3(t[2], t[0]);
    cross3(ab, ac)
}

fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let p = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [2.5, 1.0, 0.0],
        ];
        let i = vec![0, 1, 2, 3, 4, 5];
        (p, i)
    }

    #[test]
    fn test_clean_mesh_no_intersections() {
        /* separated triangles should not self-intersect */
        let (p, i) = clean_mesh();
        let r = detect_self_intersections(&p, &i);
        assert!(is_self_intersection_free(&r));
    }

    #[test]
    fn test_result_is_clean_empty() {
        /* empty result is clean */
        let r = SelfIntersectResult::new();
        assert!(r.is_clean());
    }

    #[test]
    fn test_intersection_pair_count_zero() {
        /* clean mesh pair count is zero */
        let (p, i) = clean_mesh();
        let r = detect_self_intersections(&p, &i);
        assert_eq!(intersection_pair_count(&r), 0);
    }

    #[test]
    fn test_intersecting_faces_empty() {
        /* no pairs means no faces listed */
        let r = SelfIntersectResult::new();
        assert!(intersecting_faces(&r).is_empty());
    }

    #[test]
    fn test_intersection_face_fraction_zero() {
        /* fraction is zero for a clean mesh */
        let r = SelfIntersectResult::new();
        assert!((intersection_face_fraction(&r, 10) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_fraction_zero_faces() {
        /* zero total faces does not panic */
        let r = SelfIntersectResult::new();
        assert!((intersection_face_fraction(&r, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_manual_pair_insertion() {
        /* manually inserted pairs are counted */
        let mut r = SelfIntersectResult::new();
        r.pairs.push(IntersectingPair {
            face_a: 0,
            face_b: 2,
        });
        assert_eq!(intersection_pair_count(&r), 1);
        assert!(!is_self_intersection_free(&r));
    }

    #[test]
    fn test_intersecting_faces_dedup() {
        /* faces in multiple pairs appear only once */
        let mut r = SelfIntersectResult::new();
        r.pairs.push(IntersectingPair {
            face_a: 0,
            face_b: 1,
        });
        r.pairs.push(IntersectingPair {
            face_a: 0,
            face_b: 2,
        });
        let faces = intersecting_faces(&r);
        assert_eq!(faces.iter().filter(|&&f| f == 0).count(), 1);
    }

    #[test]
    fn test_aabbs_overlap_disjoint() {
        /* disjoint AABBs do not overlap */
        let a = ([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
        let b = ([2.0, 2.0, 2.0], [3.0, 3.0, 3.0]);
        assert!(!aabbs_overlap(a, b));
    }

    #[test]
    fn test_shares_vertex_detection() {
        /* triangles sharing a vertex are skipped */
        let indices: Vec<u32> = vec![0, 1, 2, 2, 3, 4];
        assert!(shares_vertex(&indices, 0, 1));
    }
}
