// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Project mesh vertices onto a sphere surface.

use std::f32::consts::PI;

/// Project a single position onto a sphere of radius `r` centred at `centre`.
#[allow(dead_code)]
pub fn project_to_sphere(pos: [f32; 3], centre: [f32; 3], r: f32) -> [f32; 3] {
    let d = [pos[0] - centre[0], pos[1] - centre[1], pos[2] - centre[2]];
    let len = (d[0].powi(2) + d[1].powi(2) + d[2].powi(2)).sqrt();
    if len < 1e-9 {
        return [centre[0] + r, centre[1], centre[2]];
    }
    [
        centre[0] + d[0] / len * r,
        centre[1] + d[1] / len * r,
        centre[2] + d[2] / len * r,
    ]
}

/// Project all mesh positions onto a sphere.
#[allow(dead_code)]
pub fn project_mesh_to_sphere(positions: &[[f32; 3]], centre: [f32; 3], r: f32) -> Vec<[f32; 3]> {
    positions
        .iter()
        .map(|&p| project_to_sphere(p, centre, r))
        .collect()
}

/// Convert a position on a unit sphere to spherical coordinates `(theta, phi)`.
/// `theta` ∈ [0, π], `phi` ∈ [-π, π].
#[allow(dead_code)]
pub fn cart_to_spherical(pos: [f32; 3]) -> (f32, f32) {
    let len = (pos[0].powi(2) + pos[1].powi(2) + pos[2].powi(2))
        .sqrt()
        .max(1e-9);
    let theta = (pos[2] / len).clamp(-1.0, 1.0).acos();
    let phi = pos[1].atan2(pos[0]);
    (theta, phi)
}

/// Convert spherical `(theta, phi, r)` back to Cartesian.
#[allow(dead_code)]
pub fn spherical_to_cart(theta: f32, phi: f32, r: f32) -> [f32; 3] {
    let x = r * theta.sin() * phi.cos();
    let y = r * theta.sin() * phi.sin();
    let z = r * theta.cos();
    [x, y, z]
}

/// Compute spherical UV coordinates from a unit-sphere position.
/// Returns `[u, v]` with u,v ∈ [0, 1].
#[allow(dead_code)]
pub fn spherical_uv(pos: [f32; 3]) -> [f32; 2] {
    let (theta, phi) = cart_to_spherical(pos);
    let u = (phi + PI) / (2.0 * PI);
    let v = theta / PI;
    [u, v]
}

/// Compute mean distance of positions from `centre`.
#[allow(dead_code)]
pub fn mean_radius(positions: &[[f32; 3]], centre: [f32; 3]) -> f32 {
    if positions.is_empty() {
        return 0.0;
    }
    let sum: f32 = positions
        .iter()
        .map(|p| {
            ((p[0] - centre[0]).powi(2) + (p[1] - centre[1]).powi(2) + (p[2] - centre[2]).powi(2))
                .sqrt()
        })
        .sum();
    sum / positions.len() as f32
}

/// Check that all projected positions lie on the sphere (within tolerance).
#[allow(dead_code)]
pub fn all_on_sphere(positions: &[[f32; 3]], centre: [f32; 3], r: f32, tol: f32) -> bool {
    positions.iter().all(|p| {
        let d =
            ((p[0] - centre[0]).powi(2) + (p[1] - centre[1]).powi(2) + (p[2] - centre[2]).powi(2))
                .sqrt();
        (d - r).abs() < tol
    })
}

/// Generate UVs for all projected positions.
#[allow(dead_code)]
pub fn sphere_uvs(positions: &[[f32; 3]], centre: [f32; 3], r: f32) -> Vec<[f32; 2]> {
    let proj = project_mesh_to_sphere(positions, centre, r);
    proj.iter()
        .map(|&p| {
            let local = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
            let unit = {
                let l = r.max(1e-9);
                [local[0] / l, local[1] / l, local[2] / l]
            };
            spherical_uv(unit)
        })
        .collect()
}

/// Return the angle between two sphere-projected positions as seen from `centre`.
#[allow(dead_code)]
pub fn angle_between_projected(a: [f32; 3], b: [f32; 3], centre: [f32; 3]) -> f32 {
    let da: [f32; 3] = [a[0] - centre[0], a[1] - centre[1], a[2] - centre[2]];
    let db: [f32; 3] = [b[0] - centre[0], b[1] - centre[1], b[2] - centre[2]];
    let la = (da[0].powi(2) + da[1].powi(2) + da[2].powi(2))
        .sqrt()
        .max(1e-9);
    let lb = (db[0].powi(2) + db[1].powi(2) + db[2].powi(2))
        .sqrt()
        .max(1e-9);
    let dot = (da[0] * db[0] + da[1] * db[1] + da[2] * db[2]) / (la * lb);
    dot.clamp(-1.0, 1.0).acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_to_unit_sphere() {
        let p = project_to_sphere([3.0, 0.0, 0.0], [0.0; 3], 1.0);
        assert!((p[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn project_mesh_smoke() {
        let pos = vec![[2.0, 0.0, 0.0], [0.0, 2.0, 0.0]];
        let proj = project_mesh_to_sphere(&pos, [0.0; 3], 1.0);
        assert!(all_on_sphere(&proj, [0.0; 3], 1.0, 1e-5));
    }

    #[test]
    fn spherical_uv_in_range() {
        let uv = spherical_uv([1.0, 0.0, 0.0]);
        assert!((0.0..=1.0).contains(&uv[0]));
        assert!((0.0..=1.0).contains(&uv[1]));
    }

    #[test]
    fn cart_spherical_roundtrip() {
        let (t, p) = cart_to_spherical([0.0, 1.0, 0.0]);
        let c = spherical_to_cart(t, p, 1.0);
        assert!((c[0] - 0.0).abs() < 1e-5);
        assert!((c[1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn mean_radius_unit_sphere() {
        let pos = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let r = mean_radius(&pos, [0.0; 3]);
        assert!((r - 1.0).abs() < 1e-5);
    }

    #[test]
    fn mean_radius_empty() {
        assert_eq!(mean_radius(&[], [0.0; 3]), 0.0);
    }

    #[test]
    fn angle_between_orthogonal() {
        let a = project_to_sphere([1.0, 0.0, 0.0], [0.0; 3], 1.0);
        let b = project_to_sphere([0.0, 1.0, 0.0], [0.0; 3], 1.0);
        let ang = angle_between_projected(a, b, [0.0; 3]);
        assert!((ang - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
    }

    #[test]
    fn sphere_uvs_count() {
        let pos = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let uvs = sphere_uvs(&pos, [0.0; 3], 1.0);
        assert_eq!(uvs.len(), 2);
    }

    #[test]
    fn pi_constant_used() {
        assert!((PI - std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn project_origin_fallback() {
        let p = project_to_sphere([0.0, 0.0, 0.0], [0.0; 3], 2.0);
        let r = (p[0].powi(2) + p[1].powi(2) + p[2].powi(2)).sqrt();
        assert!((r - 2.0).abs() < 1e-5);
    }
}
