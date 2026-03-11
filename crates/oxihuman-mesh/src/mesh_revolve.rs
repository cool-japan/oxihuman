// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Surface of revolution: revolve a 2-D profile curve around the Y axis.

/// A surface of revolution mesh.
#[allow(dead_code)]
pub struct RevolveSurface {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub steps: usize,
}

/// Revolve a profile (list of (r, y) pairs) around the Y axis.
/// `steps` is the number of angular divisions.
/// `angle` is the sweep angle in radians (use `TAU` for full revolution).
#[allow(dead_code)]
pub fn revolve_profile(profile: &[[f32; 2]], steps: usize, angle: f32) -> RevolveSurface {
    let ns = steps.max(3);
    let np = profile.len();
    if np < 2 {
        return RevolveSurface {
            positions: vec![],
            indices: vec![],
            normals: vec![],
            steps: ns,
        };
    }
    let mut positions = Vec::with_capacity(np * (ns + 1));
    for &[r, y] in profile {
        for si in 0..=ns {
            let theta = (si as f32 / ns as f32) * angle;
            positions.push([r * theta.cos(), y, r * theta.sin()]);
        }
    }
    let stride = ns + 1;
    let mut indices: Vec<u32> = Vec::new();
    for pi in 0..(np - 1) {
        for si in 0..ns {
            let a = (pi * stride + si) as u32;
            let b = (pi * stride + si + 1) as u32;
            let c = ((pi + 1) * stride + si) as u32;
            let d = ((pi + 1) * stride + si + 1) as u32;
            indices.extend_from_slice(&[a, b, c, b, d, c]);
        }
    }
    let normals = revolve_normals(&positions, &indices);
    RevolveSurface {
        positions,
        indices,
        normals,
        steps: ns,
    }
}

/// Create a simple profile for a cylinder of given `radius` and `height`.
#[allow(dead_code)]
pub fn cylinder_profile(radius: f32, height: f32) -> Vec<[f32; 2]> {
    vec![[radius, 0.0], [radius, height]]
}

/// Create a cone profile (radius shrinks from base to tip).
#[allow(dead_code)]
pub fn cone_profile(base_radius: f32, height: f32, steps: usize) -> Vec<[f32; 2]> {
    let n = steps.max(2);
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            [base_radius * (1.0 - t), t * height]
        })
        .collect()
}

/// Triangle count of a revolve surface.
#[allow(dead_code)]
pub fn revolve_triangle_count(surf: &RevolveSurface) -> usize {
    surf.indices.len() / 3
}

fn revolve_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut acc = vec![[0.0f32; 3]; positions.len()];
    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let pa = positions[a];
        let pb = positions[b];
        let pc = positions[c];
        let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
        let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
        let n3 = [
            ab[1] * ac[2] - ab[2] * ac[1],
            ab[2] * ac[0] - ab[0] * ac[2],
            ab[0] * ac[1] - ab[1] * ac[0],
        ];
        for &i in &[a, b, c] {
            acc[i][0] += n3[0];
            acc[i][1] += n3[1];
            acc[i][2] += n3[2];
        }
    }
    acc.iter()
        .map(|&v| {
            let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            if l < 1e-8 {
                [0.0, 1.0, 0.0]
            } else {
                [v[0] / l, v[1] / l, v[2] / l]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    #[test]
    fn revolve_cylinder_vertex_count() {
        let prof = cylinder_profile(1.0, 2.0);
        let surf = revolve_profile(&prof, 8, TAU);
        assert_eq!(surf.positions.len(), 2 * 9);
    }

    #[test]
    fn revolve_empty_profile() {
        let surf = revolve_profile(&[], 8, TAU);
        assert!(surf.positions.is_empty());
    }

    #[test]
    fn revolve_single_point_profile() {
        let surf = revolve_profile(&[[1.0, 0.0]], 8, TAU);
        assert!(surf.positions.is_empty());
    }

    #[test]
    fn revolve_triangle_count_correct() {
        let prof = cylinder_profile(1.0, 1.0);
        let surf = revolve_profile(&prof, 6, TAU);
        assert_eq!(revolve_triangle_count(&surf), 12);
    }

    #[test]
    fn cone_profile_tip_radius_zero() {
        let prof = cone_profile(1.0, 2.0, 4);
        assert!((prof.last().unwrap()[0]).abs() < 1e-6);
    }

    #[test]
    fn cone_profile_length() {
        let prof = cone_profile(1.0, 2.0, 4);
        assert_eq!(prof.len(), 5);
    }

    #[test]
    fn revolve_partial_angle() {
        let prof = cylinder_profile(1.0, 1.0);
        let half = revolve_profile(&prof, 8, TAU / 2.0);
        let full = revolve_profile(&prof, 8, TAU);
        assert_eq!(half.positions.len(), full.positions.len());
    }

    #[test]
    fn tau_double_pi() {
        let v = TAU - 2.0 * std::f32::consts::PI;
        assert!(v.abs() < 1e-5);
    }

    #[test]
    fn normals_unit_length() {
        let prof = cylinder_profile(1.0, 2.0);
        let surf = revolve_profile(&prof, 8, TAU);
        for n in &surf.normals {
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            assert!(l < 1e-6 || (l - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn steps_stored() {
        let prof = cylinder_profile(1.0, 1.0);
        let surf = revolve_profile(&prof, 12, TAU);
        assert_eq!(surf.steps, 12);
    }
}
