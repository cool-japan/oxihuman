// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Face inset tool.

#[derive(Debug, Clone)]
pub struct InsetResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub inset_face_count: usize,
    pub new_vertex_count: usize,
}

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

fn centroid(pts: &[[f32; 3]]) -> [f32; 3] {
    if pts.is_empty() {
        return [0.0; 3];
    }
    let n = pts.len() as f32;
    let s = pts
        .iter()
        .fold([0.0f32; 3], |a, &p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]);
    [s[0] / n, s[1] / n, s[2] / n]
}

pub fn inset_triangle(
    positions: &[[f32; 3]],
    tri: [u32; 3],
    amount: f32,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let cen = centroid(&[a, b, c]);
    let t = amount.clamp(0.0, 1.0);
    let ia = lerp3(a, cen, t);
    let ib = lerp3(b, cen, t);
    let ic = lerp3(c, cen, t);
    let base = positions.len() as u32;
    let mut new_pts = positions.to_vec();
    new_pts.push(ia);
    new_pts.push(ib);
    new_pts.push(ic);
    let (ai, bi, ci) = (base, base + 1, base + 2);
    let (oa, ob, oc) = (tri[0], tri[1], tri[2]);
    let new_idx = vec![
        ai, bi, ci, oa, ob, bi, oa, bi, ai, ob, oc, ci, ob, ci, bi, oc, oa, ai, oc, ai, ci,
    ];
    (new_pts, new_idx)
}

pub fn inset_polygon(
    positions: &[[f32; 3]],
    face: &[u32],
    amount: f32,
) -> (Vec<[f32; 3]>, Vec<u32>) {
    if face.len() < 3 {
        return (positions.to_vec(), vec![]);
    }
    let poly_pts: Vec<[f32; 3]> = face.iter().map(|&i| positions[i as usize]).collect();
    let cen = centroid(&poly_pts);
    let t = amount.clamp(0.0, 1.0);
    let inset_pts: Vec<[f32; 3]> = poly_pts.iter().map(|&p| lerp3(p, cen, t)).collect();
    let base = positions.len() as u32;
    let n = face.len();
    let mut new_pts = positions.to_vec();
    new_pts.extend_from_slice(&inset_pts);
    let mut new_idx = Vec::new();
    for i in 1..n - 1 {
        new_idx.extend_from_slice(&[base, base + i as u32, base + i as u32 + 1]);
    }
    for i in 0..n {
        let j = (i + 1) % n;
        let oa = face[i];
        let ob = face[j];
        let ia = base + i as u32;
        let ib = base + j as u32;
        new_idx.extend_from_slice(&[oa, ob, ib, oa, ib, ia]);
    }
    (new_pts, new_idx)
}

pub fn inset_faces(
    positions: &[[f32; 3]],
    indices: &[u32],
    face_indices: &[usize],
    amount: f32,
) -> InsetResult {
    let mut new_positions = positions.to_vec();
    let mut new_indices = Vec::new();
    let mut inset_face_count = 0usize;
    let face_count = indices.len() / 3;
    for &fi in face_indices {
        if fi >= face_count {
            continue;
        }
        let base = fi * 3;
        let tri = [indices[base], indices[base + 1], indices[base + 2]];
        let (pts, idx) = inset_triangle(&new_positions, tri, amount);
        let old_len = new_positions.len();
        new_positions.extend_from_slice(&pts[old_len..]);
        new_indices.extend_from_slice(&idx);
        inset_face_count += 1;
    }
    for (fi, tri) in indices.chunks(3).enumerate() {
        if tri.len() < 3 {
            continue;
        }
        if face_indices.contains(&fi) {
            continue;
        }
        new_indices.extend_from_slice(tri);
    }
    let new_vertex_count = new_positions.len() - positions.len();
    InsetResult {
        new_positions,
        new_indices,
        inset_face_count,
        new_vertex_count,
    }
}

pub fn inset_amount_from_depth(depth: f32, max_depth: f32) -> f32 {
    (depth * max_depth).clamp(0.0, max_depth)
}

pub fn validate_inset_amount(amount: f32) -> bool {
    (0.0..=1.0).contains(&amount)
}

pub fn inset_area_ratio(amount: f32) -> f32 {
    (1.0 - amount) * (1.0 - amount)
}

pub fn inset_vertex_estimate(polygon_size: usize) -> usize {
    polygon_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inset_triangle_inner() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let (new_pos, _) = inset_triangle(&pos, [0, 1, 2], 0.5);
        assert_eq!(new_pos.len(), 6);
    }

    #[test]
    fn test_inset_polygon_inner() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let face = vec![0u32, 1, 2, 3];
        let (new_pos, _) = inset_polygon(&pos, &face, 0.3);
        assert_eq!(new_pos.len(), 8);
    }

    #[test]
    fn test_validate_inset_amount() {
        assert!(validate_inset_amount(0.5));
        assert!(validate_inset_amount(0.0));
        assert!(!validate_inset_amount(1.1));
    }

    #[test]
    fn test_inset_area_ratio_zero() {
        assert!((inset_area_ratio(0.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_inset_area_ratio_half() {
        assert!((inset_area_ratio(0.5) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn test_inset_vertex_estimate() {
        assert_eq!(inset_vertex_estimate(4), 4);
    }

    #[test]
    fn test_inset_amount_from_depth() {
        let a = inset_amount_from_depth(0.5, 0.8);
        assert!((a - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_inset_faces_runs() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let res = inset_faces(&pos, &idx, &[0], 0.3);
        assert_eq!(res.inset_face_count, 1);
        assert!(res.new_positions.len() > 3);
    }
}
