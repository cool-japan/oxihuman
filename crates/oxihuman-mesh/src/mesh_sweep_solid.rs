// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sweep a 2-D profile along a 3-D path to produce a solid mesh.

use std::f32::consts::{PI, TAU};

/// A 2-D profile (polygon in the XY plane).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Profile2DSolid {
    pub points: Vec<[f32; 2]>,
}

/// A 3-D path point with optional orientation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PathPointSolid {
    pub position: [f32; 3],
    pub up: [f32; 3],
}

/// Result of a solid sweep.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SweepSolidResult {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub cap_start: bool,
    pub cap_end: bool,
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2))
        .sqrt()
        .max(1e-9);
    [v[0] / l, v[1] / l, v[2] / l]
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Create a regular n-gon profile with radius `r`.
#[allow(dead_code)]
pub fn regular_polygon_profile(sides: usize, r: f32) -> Profile2DSolid {
    let pts = (0..sides)
        .map(|i| {
            let a = TAU * i as f32 / sides as f32;
            [r * a.cos(), r * a.sin()]
        })
        .collect();
    Profile2DSolid { points: pts }
}

/// Sweep a profile along a path (no end caps).
#[allow(dead_code)]
pub fn sweep_solid(profile: &Profile2DSolid, path: &[PathPointSolid]) -> SweepSolidResult {
    if path.len() < 2 || profile.points.is_empty() {
        return SweepSolidResult {
            positions: vec![],
            indices: vec![],
            cap_start: false,
            cap_end: false,
        };
    }
    let np = profile.points.len();
    let ns = path.len();
    let mut positions = Vec::new();
    for (si, seg) in path.iter().enumerate() {
        let fwd = if si + 1 < ns {
            normalize3([
                path[si + 1].position[0] - seg.position[0],
                path[si + 1].position[1] - seg.position[1],
                path[si + 1].position[2] - seg.position[2],
            ])
        } else {
            normalize3([
                seg.position[0] - path[si - 1].position[0],
                seg.position[1] - path[si - 1].position[1],
                seg.position[2] - path[si - 1].position[2],
            ])
        };
        let right = normalize3(cross3(fwd, normalize3(seg.up)));
        let up2 = normalize3(cross3(right, fwd));
        for pt in &profile.points {
            let x = pt[0];
            let y = pt[1];
            positions.push([
                seg.position[0] + right[0] * x + up2[0] * y,
                seg.position[1] + right[1] * x + up2[1] * y,
                seg.position[2] + right[2] * x + up2[2] * y,
            ]);
        }
    }
    let mut indices = Vec::new();
    for si in 0..(ns as u32 - 1) {
        for pi in 0..np as u32 {
            let a = si * np as u32 + pi;
            let b = si * np as u32 + (pi + 1) % np as u32;
            let c = (si + 1) * np as u32 + pi;
            let d = (si + 1) * np as u32 + (pi + 1) % np as u32;
            indices.extend_from_slice(&[a, b, d, a, d, c]);
        }
    }
    SweepSolidResult {
        positions,
        indices,
        cap_start: false,
        cap_end: false,
    }
}

/// Count triangles in the sweep result.
#[allow(dead_code)]
pub fn sweep_triangle_count(res: &SweepSolidResult) -> usize {
    res.indices.len() / 3
}

/// Validate indices are within bounds.
#[allow(dead_code)]
pub fn sweep_indices_valid(res: &SweepSolidResult) -> bool {
    let n = res.positions.len() as u32;
    res.indices.iter().all(|&i| i < n)
}

/// Compute bounding box of sweep.
#[allow(dead_code)]
pub fn sweep_bounds(res: &SweepSolidResult) -> ([f32; 3], [f32; 3]) {
    let mut mn = [f32::MAX; 3];
    let mut mx = [f32::MIN; 3];
    for p in &res.positions {
        #[allow(clippy::needless_range_loop)]
        for k in 0..3 {
            if p[k] < mn[k] {
                mn[k] = p[k];
            }
            if p[k] > mx[k] {
                mx[k] = p[k];
            }
        }
    }
    (mn, mx)
}

/// Compute approximate surface area of sweep.
#[allow(dead_code)]
pub fn sweep_surface_area(res: &SweepSolidResult) -> f32 {
    let _ = PI;
    res.indices
        .chunks_exact(3)
        .map(|tri| {
            let a = res.positions[tri[0] as usize];
            let b = res.positions[tri[1] as usize];
            let c = res.positions[tri[2] as usize];
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            (n[0].powi(2) + n[1].powi(2) + n[2].powi(2)).sqrt() * 0.5
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight_path() -> Vec<PathPointSolid> {
        (0..5)
            .map(|i| PathPointSolid {
                position: [i as f32, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
            })
            .collect()
    }

    fn square_profile() -> Profile2DSolid {
        Profile2DSolid {
            points: vec![[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]],
        }
    }

    #[test]
    fn sweep_vertex_count() {
        let res = sweep_solid(&square_profile(), &straight_path());
        assert_eq!(res.positions.len(), 5 * 4);
    }

    #[test]
    fn sweep_triangle_count_test() {
        let res = sweep_solid(&square_profile(), &straight_path());
        assert_eq!(sweep_triangle_count(&res), 4 * 4 * 2);
    }

    #[test]
    fn sweep_indices_valid_test() {
        let res = sweep_solid(&square_profile(), &straight_path());
        assert!(sweep_indices_valid(&res));
    }

    #[test]
    fn polygon_profile_count() {
        let p = regular_polygon_profile(6, 1.0);
        assert_eq!(p.points.len(), 6);
    }

    #[test]
    fn polygon_profile_radius() {
        let p = regular_polygon_profile(4, 2.0);
        let r = (p.points[0][0].powi(2) + p.points[0][1].powi(2)).sqrt();
        assert!((r - 2.0).abs() < 1e-5);
    }

    #[test]
    fn surface_area_positive() {
        let res = sweep_solid(&square_profile(), &straight_path());
        assert!(sweep_surface_area(&res) > 0.0);
    }

    #[test]
    fn bounding_box_extends_along_path() {
        let res = sweep_solid(&square_profile(), &straight_path());
        let (mn, mx) = sweep_bounds(&res);
        assert!(mx[0] > mn[0]);
    }

    #[test]
    fn empty_path() {
        let res = sweep_solid(&square_profile(), &[]);
        assert!(res.positions.is_empty());
    }

    #[test]
    fn empty_profile() {
        let empty = Profile2DSolid { points: vec![] };
        let res = sweep_solid(&empty, &straight_path());
        assert!(res.positions.is_empty());
    }

    #[test]
    fn tau_constant_used() {
        assert!((TAU - 2.0 * PI).abs() < 1e-5);
    }
}
