// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Poke (fan triangulate) faces — inserts a centre vertex in each face.

#[derive(Debug, Clone)]
pub struct PokeResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub poked_face_count: usize,
    pub new_vertex_count: usize,
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

/// Poke a single triangle face (insert centroid, create 3 triangles).
pub fn poke_triangle(positions: &[[f32; 3]], tri: [u32; 3]) -> (Vec<[f32; 3]>, Vec<u32>) {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let cen = centroid(&[a, b, c]);
    let ci = positions.len() as u32;
    let mut new_pts = positions.to_vec();
    new_pts.push(cen);
    let (va, vb, vc) = (tri[0], tri[1], tri[2]);
    let idx = vec![va, vb, ci, vb, vc, ci, vc, va, ci];
    (new_pts, idx)
}

/// Poke a polygon face (insert centroid, fan triangulate).
pub fn poke_polygon(positions: &[[f32; 3]], face: &[u32]) -> (Vec<[f32; 3]>, Vec<u32>) {
    if face.len() < 3 {
        return (positions.to_vec(), vec![]);
    }
    let poly_pts: Vec<[f32; 3]> = face.iter().map(|&i| positions[i as usize]).collect();
    let cen = centroid(&poly_pts);
    let ci = positions.len() as u32;
    let mut new_pts = positions.to_vec();
    new_pts.push(cen);
    let n = face.len();
    let mut idx = Vec::with_capacity(n * 3);
    for i in 0..n {
        let j = (i + 1) % n;
        idx.extend_from_slice(&[face[i], face[j], ci]);
    }
    (new_pts, idx)
}

/// Apply poke to a list of triangle face indices in a mesh.
pub fn poke_faces(positions: &[[f32; 3]], indices: &[u32], face_indices: &[usize]) -> PokeResult {
    let mut new_positions = positions.to_vec();
    let mut new_indices = Vec::new();
    let mut poked_face_count = 0usize;
    let face_count = indices.len() / 3;
    for &fi in face_indices {
        if fi >= face_count {
            continue;
        }
        let base = fi * 3;
        let tri = [indices[base], indices[base + 1], indices[base + 2]];
        let (pts, idx) = poke_triangle(&new_positions, tri);
        let old_len = new_positions.len();
        new_positions.extend_from_slice(&pts[old_len..]);
        new_indices.extend_from_slice(&idx);
        poked_face_count += 1;
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
    PokeResult {
        new_positions,
        new_indices,
        poked_face_count,
        new_vertex_count,
    }
}

/// Poke all faces in a mesh.
pub fn poke_all_faces(positions: &[[f32; 3]], indices: &[u32]) -> PokeResult {
    let face_count = indices.len() / 3;
    let face_indices: Vec<usize> = (0..face_count).collect();
    poke_faces(positions, indices, &face_indices)
}

/// Estimate triangle count after poking n faces.
pub fn poke_triangle_estimate(n: usize) -> usize {
    n * 3
}

/// Validate that poke won't produce degenerate faces.
pub fn validate_poke_input(positions: &[[f32; 3]], indices: &[u32]) -> bool {
    indices
        .chunks(3)
        .all(|tri| tri.len() == 3 && tri.iter().all(|&v| (v as usize) < positions.len()))
}

/// Poke a specific vertex to adjust centre position.
pub fn poke_with_offset(
    positions: &[[f32; 3]],
    tri: [u32; 3],
    offset: [f32; 3],
) -> (Vec<[f32; 3]>, Vec<u32>) {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let cen_raw = centroid(&[a, b, c]);
    let cen = [
        cen_raw[0] + offset[0],
        cen_raw[1] + offset[1],
        cen_raw[2] + offset[2],
    ];
    let ci = positions.len() as u32;
    let mut new_pts = positions.to_vec();
    new_pts.push(cen);
    let idx = vec![tri[0], tri[1], ci, tri[1], tri[2], ci, tri[2], tri[0], ci];
    (new_pts, idx)
}

/// Count new triangles produced by poking all faces.
pub fn count_poke_result_triangles(original_tri_count: usize) -> usize {
    original_tri_count * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poke_triangle_vertex_count() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let (new_pos, _) = poke_triangle(&pos, [0, 1, 2]);
        assert_eq!(new_pos.len(), 4);
    }

    #[test]
    fn test_poke_triangle_index_count() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let (_, idx) = poke_triangle(&pos, [0, 1, 2]);
        assert_eq!(idx.len(), 9);
    }

    #[test]
    fn test_poke_polygon() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let face = vec![0u32, 1, 2, 3];
        let (new_pos, idx) = poke_polygon(&pos, &face);
        assert_eq!(new_pos.len(), 5);
        assert_eq!(idx.len(), 12);
    }

    #[test]
    fn test_poke_faces_runs() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let res = poke_faces(&pos, &idx, &[0]);
        assert_eq!(res.poked_face_count, 1);
        assert_eq!(res.new_vertex_count, 1);
    }

    #[test]
    fn test_poke_all_faces() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        let res = poke_all_faces(&pos, &idx);
        assert_eq!(res.poked_face_count, 1);
    }

    #[test]
    fn test_poke_triangle_estimate() {
        assert_eq!(poke_triangle_estimate(4), 12);
    }

    #[test]
    fn test_validate_poke_input() {
        let pos = vec![[0.0; 3], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        assert!(validate_poke_input(&pos, &idx));
    }

    #[test]
    fn test_poke_with_offset() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let (new_pos, _) = poke_with_offset(&pos, [0, 1, 2], [0.0, 0.0, 0.1]);
        assert!(new_pos.last().unwrap()[2] > 0.0);
    }
}
