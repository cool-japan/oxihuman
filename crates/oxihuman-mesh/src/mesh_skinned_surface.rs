// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Skin surface through profile curves.

/// Skin surface built by interpolating through an ordered sequence of profile
/// curves (like loft but with chord-length parameterization).
#[derive(Debug, Clone)]
pub struct SkinnedSurface {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    /// Number of profiles used.
    pub profile_count: usize,
    /// Number of points per profile.
    pub ring_len: usize,
}

/// Compute chord-length parameter values for a sequence of profile curves.
pub fn chord_length_params(profiles: &[Vec<[f32; 3]>]) -> Vec<f32> {
    if profiles.is_empty() {
        return vec![];
    }
    let mut params = vec![0.0f32];
    for i in 1..profiles.len() {
        let c0 = profile_centroid_skin(&profiles[i - 1]);
        let c1 = profile_centroid_skin(&profiles[i]);
        let d =
            ((c1[0] - c0[0]).powi(2) + (c1[1] - c0[1]).powi(2) + (c1[2] - c0[2]).powi(2)).sqrt();
        params.push(params.last().copied().unwrap_or(0.0) + d.max(1e-6));
    }
    let total = *params.last().unwrap_or(&1.0);
    if total > 1e-9 {
        params.iter_mut().for_each(|p| *p /= total);
    }
    params
}

fn profile_centroid_skin(profile: &[[f32; 3]]) -> [f32; 3] {
    if profile.is_empty() {
        return [0.0; 3];
    }
    let n = profile.len() as f32;
    let mut s = [0.0f32; 3];
    for &p in profile {
        s[0] += p[0];
        s[1] += p[1];
        s[2] += p[2];
    }
    [s[0] / n, s[1] / n, s[2] / n]
}

/// Build a skinned surface through the given profiles.
/// All profiles must have the same ring length.
pub fn build_skinned_surface(profiles: &[Vec<[f32; 3]>]) -> SkinnedSurface {
    if profiles.len() < 2 {
        return SkinnedSurface {
            verts: vec![],
            tris: vec![],
            profile_count: 0,
            ring_len: 0,
        };
    }
    let ring_len = profiles[0].len();
    if ring_len < 2 || profiles.iter().any(|p| p.len() != ring_len) {
        return SkinnedSurface {
            verts: vec![],
            tris: vec![],
            profile_count: 0,
            ring_len: 0,
        };
    }
    let n_prof = profiles.len();
    let mut verts = Vec::with_capacity(n_prof * ring_len);
    for prof in profiles {
        verts.extend_from_slice(prof);
    }
    let mut tris = Vec::new();
    for p in 0..(n_prof - 1) {
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
    SkinnedSurface {
        verts,
        tris,
        profile_count: n_prof,
        ring_len,
    }
}

/// Return the vertex count.
pub fn skinned_vertex_count(surf: &SkinnedSurface) -> usize {
    surf.verts.len()
}

/// Return the triangle count.
pub fn skinned_tri_count(surf: &SkinnedSurface) -> usize {
    surf.tris.len()
}

/// Validate triangle index bounds.
pub fn validate_skinned_surface(surf: &SkinnedSurface) -> bool {
    let n = surf.verts.len() as u32;
    surf.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn circle(y: f32, r: f32, n: usize) -> Vec<[f32; 3]> {
        use std::f32::consts::TAU;
        (0..n)
            .map(|i| {
                let a = TAU * i as f32 / n as f32;
                [r * a.cos(), y, r * a.sin()]
            })
            .collect()
    }

    #[test]
    fn test_skinned_vertex_count() {
        let p0 = circle(0.0, 1.0, 8);
        let p1 = circle(1.0, 1.0, 8);
        let p2 = circle(2.0, 1.0, 8);
        let s = build_skinned_surface(&[p0, p1, p2]);
        assert_eq!(skinned_vertex_count(&s), 24);
    }

    #[test]
    fn test_skinned_tri_count() {
        /* 2 segments * 8 quads * 2 tris = 32 */
        let p0 = circle(0.0, 1.0, 8);
        let p1 = circle(1.0, 1.0, 8);
        let p2 = circle(2.0, 1.0, 8);
        let s = build_skinned_surface(&[p0, p1, p2]);
        assert_eq!(skinned_tri_count(&s), 32);
    }

    #[test]
    fn test_skinned_empty_on_single_profile() {
        let p0 = circle(0.0, 1.0, 6);
        let s = build_skinned_surface(&[p0]);
        assert_eq!(skinned_vertex_count(&s), 0);
    }

    #[test]
    fn test_skinned_empty_on_mismatch() {
        let p0 = circle(0.0, 1.0, 6);
        let p1 = circle(1.0, 1.0, 8);
        let s = build_skinned_surface(&[p0, p1]);
        assert_eq!(skinned_vertex_count(&s), 0);
    }

    #[test]
    fn test_validate_skinned_surface() {
        let p0 = circle(0.0, 1.0, 5);
        let p1 = circle(1.0, 1.0, 5);
        let s = build_skinned_surface(&[p0, p1]);
        assert!(validate_skinned_surface(&s));
    }

    #[test]
    fn test_chord_length_params_length() {
        let profiles = vec![
            circle(0.0, 1.0, 4),
            circle(1.0, 1.0, 4),
            circle(3.0, 1.0, 4),
        ];
        let params = chord_length_params(&profiles);
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_chord_length_params_endpoints() {
        let profiles = vec![
            circle(0.0, 1.0, 4),
            circle(2.0, 1.0, 4),
            circle(6.0, 1.0, 4),
        ];
        let params = chord_length_params(&profiles);
        assert!((params[0]).abs() < 1e-6);
        assert!((params.last().unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_chord_length_params_empty() {
        let p = chord_length_params(&[]);
        assert!(p.is_empty());
    }

    #[test]
    fn test_skinned_profile_count() {
        let p0 = circle(0.0, 1.0, 4);
        let p1 = circle(1.0, 1.0, 4);
        let p2 = circle(2.0, 1.0, 4);
        let s = build_skinned_surface(&[p0, p1, p2]);
        assert_eq!(s.profile_count, 3);
        assert_eq!(s.ring_len, 4);
    }
}
