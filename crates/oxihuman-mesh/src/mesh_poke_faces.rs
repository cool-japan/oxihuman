// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Poke (subdivide) faces by inserting a center vertex.

/// Result of a poke operation on a single face.
#[derive(Debug, Clone)]
pub struct PokeResult {
    pub positions: Vec<[f32; 3]>,
    /// Index triples for the new triangles.
    pub triangles: Vec<[usize; 3]>,
    pub center_idx: usize,
}

/// Poke a single face: insert the centroid and fan-triangulate.
pub fn poke_face(positions: &[[f32; 3]], face: &[usize]) -> PokeResult {
    let n = face.len();
    if n < 3 {
        return PokeResult {
            positions: positions.to_vec(),
            triangles: vec![],
            center_idx: 0,
        };
    }
    let mut out_pos = positions.to_vec();
    let cx: f32 = face.iter().map(|&i| positions[i][0]).sum::<f32>() / n as f32;
    let cy: f32 = face.iter().map(|&i| positions[i][1]).sum::<f32>() / n as f32;
    let cz: f32 = face.iter().map(|&i| positions[i][2]).sum::<f32>() / n as f32;
    let center_idx = out_pos.len();
    out_pos.push([cx, cy, cz]);
    let mut triangles = Vec::with_capacity(n);
    for i in 0..n {
        let a = face[i];
        let b = face[(i + 1) % n];
        triangles.push([center_idx, a, b]);
    }
    PokeResult {
        positions: out_pos,
        triangles,
        center_idx,
    }
}

/// Centroid of a face.
pub fn poke_center(positions: &[[f32; 3]], face: &[usize]) -> [f32; 3] {
    let n = face.len();
    if n == 0 {
        return [0.0; 3];
    }
    let cx: f32 = face.iter().map(|&i| positions[i][0]).sum::<f32>() / n as f32;
    let cy: f32 = face.iter().map(|&i| positions[i][1]).sum::<f32>() / n as f32;
    let cz: f32 = face.iter().map(|&i| positions[i][2]).sum::<f32>() / n as f32;
    [cx, cy, cz]
}

/// Number of triangles produced by poking an n-gon (equals n).
pub fn poke_triangle_count(polygon_len: usize) -> usize {
    polygon_len
}

/// Number of vertices added (always 1: the center).
pub fn poke_vertex_count(base_count: usize) -> usize {
    base_count + 1
}

/// Area of a poked face (sum of sub-triangle areas).
pub fn poke_face_area(positions: &[[f32; 3]], face: &[usize]) -> f32 {
    let result = poke_face(positions, face);
    result
        .triangles
        .iter()
        .map(|&[c, a, b]| {
            let pc = result.positions[c];
            let pa = result.positions[a];
            let pb = result.positions[b];
            let ab = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
            let ac = [pc[0] - pa[0], pc[1] - pa[1], pc[2] - pa[2]];
            let cross = [
                ab[1] * ac[2] - ab[2] * ac[1],
                ab[2] * ac[0] - ab[0] * ac[2],
                ab[0] * ac[1] - ab[1] * ac[0],
            ];
            0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_triangle() -> (Vec<[f32; 3]>, Vec<usize>) {
        let p = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let f = vec![0, 1, 2];
        (p, f)
    }

    #[test]
    fn test_poke_face_triangle_count() {
        /* poking a triangle yields 3 sub-triangles */
        let (p, f) = unit_triangle();
        let r = poke_face(&p, &f);
        assert_eq!(r.triangles.len(), 3);
    }

    #[test]
    fn test_poke_center_is_centroid() {
        let (p, f) = unit_triangle();
        let c = poke_center(&p, &f);
        assert!((c[0] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_poke_triangle_count_fn() {
        assert_eq!(poke_triangle_count(4), 4);
        assert_eq!(poke_triangle_count(3), 3);
    }

    #[test]
    fn test_poke_vertex_count() {
        /* adds exactly 1 vertex */
        assert_eq!(poke_vertex_count(3), 4);
    }

    #[test]
    fn test_poke_face_area_triangle() {
        /* area of unit right triangle in XZ = 0.5 */
        let p = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let f = vec![0, 1, 2];
        let a = poke_face_area(&p, &f);
        assert!((a - 0.5).abs() < 1e-5);
    }
}
