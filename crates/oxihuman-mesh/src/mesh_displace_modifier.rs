#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Displacement modifier: offset vertices along their normals by a texture/map value.

#[allow(dead_code)]
pub fn displace_vertex(v: [f32; 3], n: [f32; 3], amount: f32) -> [f32; 3] {
    [
        v[0] + n[0] * amount,
        v[1] + n[1] * amount,
        v[2] + n[2] * amount,
    ]
}

#[allow(dead_code)]
pub fn bilinear_sample(map: &[f32], width: usize, u: f32, v: f32) -> f32 {
    if map.is_empty() || width == 0 {
        return 0.0;
    }
    let height = map.len() / width;
    if height == 0 {
        return 0.0;
    }
    let uc = u.clamp(0.0, 1.0);
    let vc = v.clamp(0.0, 1.0);
    let px = (uc * (width - 1) as f32) as usize;
    let py = (vc * (height - 1) as f32) as usize;
    let px = px.min(width - 1);
    let py = py.min(height - 1);
    map[py * width + px]
}

#[allow(dead_code)]
pub fn apply_displace(
    verts: &[[f32; 3]],
    normals: &[[f32; 3]],
    map: &[f32],
    strength: f32,
) -> Vec<[f32; 3]> {
    verts
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let n = if i < normals.len() {
                normals[i]
            } else {
                [0.0, 1.0, 0.0]
            };
            let amount = if i < map.len() { map[i] } else { 0.0 };
            displace_vertex(*v, n, amount * strength)
        })
        .collect()
}

// ── New required API ──────────────────────────────────────────────────────────

pub struct DisplaceParams {
    pub strength: f32,
    pub midlevel: f32,
    pub direction: u8,
}

pub fn new_displace_params(strength: f32) -> DisplaceParams {
    DisplaceParams {
        strength,
        midlevel: 0.5,
        direction: 0,
    }
}

pub fn displace_midlevel_offset(val: f32, midlevel: f32) -> f32 {
    val - midlevel
}

pub fn displace_along_normal(pos: [f32; 3], normal: [f32; 3], amount: f32) -> [f32; 3] {
    [
        pos[0] + normal[0] * amount,
        pos[1] + normal[1] * amount,
        pos[2] + normal[2] * amount,
    ]
}

pub fn displace_along_axis(pos: [f32; 3], amount: f32, axis: u8) -> [f32; 3] {
    let mut out = pos;
    match axis {
        1 => out[0] += amount,
        2 => out[1] += amount,
        3 => out[2] += amount,
        _ => {}
    }
    out
}

pub fn displace_vertex_new(
    pos: [f32; 3],
    normal: [f32; 3],
    texture_val: f32,
    params: &DisplaceParams,
) -> [f32; 3] {
    let offset = displace_midlevel_offset(texture_val, params.midlevel);
    let amount = offset * params.strength;
    if params.direction == 0 {
        displace_along_normal(pos, normal, amount)
    } else {
        displace_along_axis(pos, amount, params.direction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displace_vertex_along_y() {
        let v = displace_vertex([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0);
        assert!((v[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn displace_vertex_zero_amount() {
        let v = displace_vertex([1.0, 2.0, 3.0], [0.0, 1.0, 0.0], 0.0);
        assert!((v[0] - 1.0).abs() < 1e-6);
        assert!((v[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn bilinear_sample_center() {
        let map = vec![0.0, 0.0, 0.0, 1.0];
        let v = bilinear_sample(&map, 2, 1.0, 1.0);
        assert!((v - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bilinear_sample_empty_returns_zero() {
        let map: Vec<f32> = vec![];
        assert!((bilinear_sample(&map, 2, 0.5, 0.5)).abs() < 1e-6);
    }

    #[test]
    fn bilinear_sample_zero_width_returns_zero() {
        let map = vec![1.0, 2.0];
        assert!((bilinear_sample(&map, 0, 0.5, 0.5)).abs() < 1e-6);
    }

    #[test]
    fn apply_displace_preserves_count() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let normals = vec![[0.0, 1.0, 0.0], [0.0, 1.0, 0.0]];
        let map = vec![1.0, 0.5];
        let out = apply_displace(&verts, &normals, &map, 1.0);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn apply_displace_strength_scales() {
        let verts = vec![[0.0, 0.0, 0.0]];
        let normals = vec![[0.0, 1.0, 0.0]];
        let map = vec![1.0];
        let out = apply_displace(&verts, &normals, &map, 3.0);
        assert!((out[0][1] - 3.0).abs() < 1e-6);
    }

    #[test]
    fn apply_displace_zero_strength_no_change() {
        let verts = vec![[1.0, 2.0, 3.0]];
        let normals = vec![[1.0, 0.0, 0.0]];
        let map = vec![1.0];
        let out = apply_displace(&verts, &normals, &map, 0.0);
        assert!((out[0][0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn apply_displace_uses_default_normal_when_missing() {
        let verts = vec![[0.0, 0.0, 0.0]];
        let normals: Vec<[f32; 3]> = vec![];
        let map = vec![1.0];
        let out = apply_displace(&verts, &normals, &map, 1.0);
        // default normal is [0,1,0]
        assert!((out[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bilinear_sample_top_left_zero() {
        let map = vec![0.5, 0.0, 0.0, 0.0];
        let v = bilinear_sample(&map, 2, 0.0, 0.0);
        assert!((v - 0.5).abs() < 1e-6);
    }
}
