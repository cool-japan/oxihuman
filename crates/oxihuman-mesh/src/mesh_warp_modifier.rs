#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Simple warp (lattice-based vertex displacement).

#[allow(dead_code)]
pub struct WarpModifier {
    pub from_pos: [f32; 3],
    pub to_pos: [f32; 3],
    pub falloff_radius: f32,
    pub strength: f32,
}

fn dist3(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[allow(dead_code)]
pub fn warp_falloff(dist: f32, radius: f32) -> f32 {
    if radius <= 0.0 {
        return 0.0;
    }
    let t = (dist / radius).clamp(0.0, 1.0);
    1.0 - t * t
}

#[allow(dead_code)]
pub fn warp_vertex(v: [f32; 3], warp: &WarpModifier) -> [f32; 3] {
    let d = dist3(v, warp.from_pos);
    let w = warp_falloff(d, warp.falloff_radius) * warp.strength;
    [
        v[0] + (warp.to_pos[0] - warp.from_pos[0]) * w,
        v[1] + (warp.to_pos[1] - warp.from_pos[1]) * w,
        v[2] + (warp.to_pos[2] - warp.from_pos[2]) * w,
    ]
}

#[allow(dead_code)]
pub fn apply_warp(verts: &[[f32; 3]], warp: &WarpModifier) -> Vec<[f32; 3]> {
    verts.iter().map(|v| warp_vertex(*v, warp)).collect()
}

// ── New required API ──────────────────────────────────────────────────────────

pub struct WarpParams {
    pub from_center: [f32; 3],
    pub to_center: [f32; 3],
    pub radius: f32,
    pub strength: f32,
}

pub fn new_warp_params(from: [f32; 3], to: [f32; 3], radius: f32) -> WarpParams {
    WarpParams {
        from_center: from,
        to_center: to,
        radius,
        strength: 1.0,
    }
}

fn dist3_wp(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

pub fn warp_falloff_linear(dist: f32, radius: f32) -> f32 {
    if radius <= 0.0 {
        return 0.0;
    }
    (1.0 - dist / radius).clamp(0.0, 1.0)
}

pub fn warp_falloff_smooth(dist: f32, radius: f32) -> f32 {
    let t = warp_falloff_linear(dist, radius);
    t * t * (3.0 - 2.0 * t)
}

pub fn warp_influence(p: [f32; 3], params: &WarpParams) -> f32 {
    let dist = dist3_wp(p, params.from_center);
    warp_falloff_linear(dist, params.radius)
}

pub fn warp_vertex_new(p: [f32; 3], params: &WarpParams) -> [f32; 3] {
    let infl = warp_influence(p, params) * params.strength;
    let delta = [
        params.to_center[0] - params.from_center[0],
        params.to_center[1] - params.from_center[1],
        params.to_center[2] - params.from_center[2],
    ];
    [
        p[0] + delta[0] * infl,
        p[1] + delta[1] * infl,
        p[2] + delta[2] * infl,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_warp() -> WarpModifier {
        WarpModifier {
            from_pos: [0.0, 0.0, 0.0],
            to_pos: [1.0, 0.0, 0.0],
            falloff_radius: 2.0,
            strength: 1.0,
        }
    }

    #[test]
    fn vertex_at_origin_fully_warped() {
        let w = make_warp();
        let v = warp_vertex([0.0, 0.0, 0.0], &w);
        assert!((v[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn vertex_far_from_origin_unchanged() {
        let w = make_warp();
        let v = warp_vertex([100.0, 0.0, 0.0], &w);
        assert!((v[0] - 100.0).abs() < 1e-5);
    }

    #[test]
    fn falloff_at_zero_is_one() {
        assert!((warp_falloff(0.0, 2.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn falloff_at_radius_is_zero() {
        assert!((warp_falloff(2.0, 2.0)).abs() < 1e-6);
    }

    #[test]
    fn falloff_beyond_radius_is_zero() {
        assert!((warp_falloff(5.0, 2.0)).abs() < 1e-6);
    }

    #[test]
    fn apply_warp_preserves_count() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];
        let w = make_warp();
        let out = apply_warp(&verts, &w);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn zero_strength_no_displacement() {
        let w = WarpModifier {
            from_pos: [0.0, 0.0, 0.0],
            to_pos: [1.0, 0.0, 0.0],
            falloff_radius: 2.0,
            strength: 0.0,
        };
        let v = warp_vertex([0.0, 0.0, 0.0], &w);
        assert!((v[0]).abs() < 1e-6);
    }

    #[test]
    fn falloff_zero_radius_returns_zero() {
        assert!((warp_falloff(0.1, 0.0)).abs() < 1e-6);
    }

    #[test]
    fn warp_moves_in_correct_direction() {
        let w = WarpModifier {
            from_pos: [0.0, 0.0, 0.0],
            to_pos: [0.0, 2.0, 0.0],
            falloff_radius: 2.0,
            strength: 1.0,
        };
        let v = warp_vertex([0.0, 0.0, 0.0], &w);
        assert!(v[1] > 0.0);
    }
}
