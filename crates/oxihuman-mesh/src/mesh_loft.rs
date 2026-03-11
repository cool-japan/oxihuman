// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Loft surface generation between profile curves.

#![allow(dead_code)]

/// Result of a loft operation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LoftResult {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
}

/// Compute the centroid of a profile curve.
#[allow(dead_code)]
pub fn profile_centroid(profile: &[[f32; 3]]) -> [f32; 3] {
    if profile.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let n = profile.len() as f32;
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for v in profile {
        cx += v[0];
        cy += v[1];
        cz += v[2];
    }
    [cx / n, cy / n, cz / n]
}

/// Loft a surface through a sequence of profile curves.
/// Each profile is a closed polygon of vertices.
/// Profiles must all have the same number of points.
#[allow(dead_code)]
pub fn loft_profiles(profiles: &[Vec<[f32; 3]>]) -> LoftResult {
    if profiles.len() < 2 {
        return LoftResult {
            verts: vec![],
            tris: vec![],
        };
    }
    let ring_len = profiles[0].len();
    let mut verts: Vec<[f32; 3]> = Vec::new();
    for profile in profiles {
        for &v in profile {
            verts.push(v);
        }
    }
    let mut tris: Vec<[u32; 3]> = Vec::new();
    let n_profiles = profiles.len();
    for p in 0..(n_profiles - 1) {
        for i in 0..ring_len {
            let next_i = (i + 1) % ring_len;
            let a = (p * ring_len + i) as u32;
            let b = (p * ring_len + next_i) as u32;
            let c = ((p + 1) * ring_len + i) as u32;
            let d = ((p + 1) * ring_len + next_i) as u32;
            tris.push([a, b, c]);
            tris.push([b, d, c]);
        }
    }
    LoftResult { verts, tris }
}

/// Return the vertex count of a loft result.
#[allow(dead_code)]
pub fn loft_vertex_count(result: &LoftResult) -> usize {
    result.verts.len()
}

/// Return the triangle count of a loft result.
#[allow(dead_code)]
pub fn loft_triangle_count(result: &LoftResult) -> usize {
    result.tris.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring(y: f32, r: f32, n: usize) -> Vec<[f32; 3]> {
        use std::f32::consts::PI;
        (0..n)
            .map(|i| {
                let angle = 2.0 * PI * i as f32 / n as f32;
                [r * angle.cos(), y, r * angle.sin()]
            })
            .collect()
    }

    #[test]
    fn test_loft_two_profiles() {
        let p0 = ring(0.0, 1.0, 4);
        let p1 = ring(1.0, 1.0, 4);
        let result = loft_profiles(&[p0, p1]);
        assert_eq!(loft_vertex_count(&result), 8);
        assert_eq!(loft_triangle_count(&result), 8);
    }

    #[test]
    fn test_loft_three_profiles() {
        let p0 = ring(0.0, 1.0, 6);
        let p1 = ring(1.0, 0.8, 6);
        let p2 = ring(2.0, 0.5, 6);
        let result = loft_profiles(&[p0, p1, p2]);
        assert_eq!(loft_vertex_count(&result), 18);
    }

    #[test]
    fn test_loft_empty() {
        let result = loft_profiles(&[]);
        assert_eq!(loft_vertex_count(&result), 0);
        assert_eq!(loft_triangle_count(&result), 0);
    }

    #[test]
    fn test_loft_single_profile() {
        let p0 = ring(0.0, 1.0, 4);
        let result = loft_profiles(&[p0]);
        assert_eq!(loft_vertex_count(&result), 0);
    }

    #[test]
    fn test_profile_centroid_square() {
        let profile = vec![
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
        ];
        let c = profile_centroid(&profile);
        assert!((c[0]).abs() < 1e-5);
        assert!((c[1]).abs() < 1e-5);
        assert!((c[2]).abs() < 1e-5);
    }

    #[test]
    fn test_profile_centroid_empty() {
        let c = profile_centroid(&[]);
        assert_eq!(c, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_loft_tri_indices_in_range() {
        let p0 = ring(0.0, 1.0, 5);
        let p1 = ring(1.0, 1.0, 5);
        let result = loft_profiles(&[p0, p1]);
        let nv = loft_vertex_count(&result) as u32;
        for tri in &result.tris {
            assert!(tri[0] < nv);
            assert!(tri[1] < nv);
            assert!(tri[2] < nv);
        }
    }

    #[test]
    fn test_loft_triangle_count_formula() {
        let p0 = ring(0.0, 1.0, 8);
        let p1 = ring(1.0, 1.0, 8);
        let p2 = ring(2.0, 1.0, 8);
        let result = loft_profiles(&[p0, p1, p2]);
        // 2 segments * 8 verts * 2 tris per quad
        assert_eq!(loft_triangle_count(&result), 32);
    }

    #[test]
    fn test_profile_centroid_single() {
        let profile = vec![[3.0, 5.0, 7.0]];
        let c = profile_centroid(&profile);
        assert!((c[0] - 3.0).abs() < 1e-5);
        assert!((c[1] - 5.0).abs() < 1e-5);
        assert!((c[2] - 7.0).abs() < 1e-5);
    }
}
