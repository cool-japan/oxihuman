// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Surface of revolution mesh generator.

/// A surface of revolution obtained by rotating a profile curve around the Y-axis.
#[derive(Debug, Clone)]
pub struct RevolutionSurface {
    pub verts: Vec<[f32; 3]>,
    pub tris: Vec<[u32; 3]>,
    pub profile_len: usize,
    pub angular_steps: usize,
}

/// Build a surface of revolution by sweeping `profile` (in the XY plane) around
/// the Y-axis through `angle_rad` radians in `angular_steps` steps.
pub fn build_revolution_surface(
    profile: &[[f32; 3]],
    angular_steps: usize,
    angle_rad: f32,
) -> RevolutionSurface {
    let n = profile.len();
    if n < 2 || angular_steps < 1 {
        return RevolutionSurface {
            verts: vec![],
            tris: vec![],
            profile_len: 0,
            angular_steps: 0,
        };
    }
    let rings = angular_steps + 1;
    let mut verts = Vec::with_capacity(rings * n);
    for step in 0..rings {
        let theta = angle_rad * step as f32 / angular_steps as f32;
        let (sin_t, cos_t) = theta.sin_cos();
        for &p in profile {
            let r = p[0]; /* radius = x-component of profile */
            verts.push([r * cos_t, p[1], r * sin_t]);
        }
    }
    let mut tris = Vec::new();
    for s in 0..(rings - 1) {
        for i in 0..(n - 1) {
            let a = (s * n + i) as u32;
            let b = (s * n + i + 1) as u32;
            let c = ((s + 1) * n + i) as u32;
            let d = ((s + 1) * n + i + 1) as u32;
            tris.push([a, c, b]);
            tris.push([b, c, d]);
        }
    }
    RevolutionSurface {
        verts,
        tris,
        profile_len: n,
        angular_steps,
    }
}

/// Return the vertex count.
pub fn revolution_vertex_count(surf: &RevolutionSurface) -> usize {
    surf.verts.len()
}

/// Return the triangle count.
pub fn revolution_tri_count(surf: &RevolutionSurface) -> usize {
    surf.tris.len()
}

/// Validate all triangle indices are in range.
pub fn validate_revolution_surface(surf: &RevolutionSurface) -> bool {
    let n = surf.verts.len() as u32;
    surf.tris.iter().all(|t| t[0] < n && t[1] < n && t[2] < n)
}

/// Compute the approximate surface area (sum of quad areas).
pub fn revolution_surface_area(surf: &RevolutionSurface) -> f32 {
    let mut area = 0.0f32;
    for tri in &surf.tris {
        let a = surf.verts[tri[0] as usize];
        let b = surf.verts[tri[1] as usize];
        let c = surf.verts[tri[2] as usize];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let cross = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        area += (cross[0].powi(2) + cross[1].powi(2) + cross[2].powi(2)).sqrt() * 0.5;
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    fn cylinder_profile(n: usize) -> Vec<[f32; 3]> {
        /* vertical segment at x=1 */
        (0..n).map(|i| [1.0, i as f32, 0.0]).collect()
    }

    #[test]
    fn test_revolution_vertex_count() {
        /* 5-point profile, 8 steps → 9 * 5 = 45 verts */
        let s = build_revolution_surface(&cylinder_profile(5), 8, TAU);
        assert_eq!(revolution_vertex_count(&s), 45);
    }

    #[test]
    fn test_revolution_tri_count() {
        /* (8 steps) * (4 segments) * 2 = 64 tris */
        let s = build_revolution_surface(&cylinder_profile(5), 8, TAU);
        assert_eq!(revolution_tri_count(&s), 64);
    }

    #[test]
    fn test_revolution_empty_on_too_short() {
        let s = build_revolution_surface(&[[1.0, 0.0, 0.0]], 8, TAU);
        assert_eq!(revolution_vertex_count(&s), 0);
    }

    #[test]
    fn test_revolution_empty_on_zero_steps() {
        let s = build_revolution_surface(&cylinder_profile(3), 0, TAU);
        assert_eq!(revolution_vertex_count(&s), 0);
    }

    #[test]
    fn test_validate_revolution_surface() {
        let s = build_revolution_surface(&cylinder_profile(4), 6, TAU);
        assert!(validate_revolution_surface(&s));
    }

    #[test]
    fn test_revolution_surface_area_positive() {
        let s = build_revolution_surface(&cylinder_profile(3), 12, TAU);
        assert!(revolution_surface_area(&s) > 0.0);
    }

    #[test]
    fn test_revolution_profile_len_stored() {
        let s = build_revolution_surface(&cylinder_profile(7), 4, TAU);
        assert_eq!(s.profile_len, 7);
        assert_eq!(s.angular_steps, 4);
    }

    #[test]
    fn test_revolution_half_turn() {
        use std::f32::consts::PI;
        /* half revolution → angular_steps + 1 = 5 rings */
        let s = build_revolution_surface(&cylinder_profile(3), 4, PI);
        assert_eq!(revolution_vertex_count(&s), 15);
    }
}
