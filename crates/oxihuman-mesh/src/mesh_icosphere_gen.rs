// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

fn phi() -> f32 {
    (1.0 + 5.0_f32.sqrt()) * 0.5
}

pub fn icosahedron_vertices() -> Vec<[f32; 3]> {
    let t = phi();
    let verts = [
        [-1.0, t, 0.0],
        [1.0, t, 0.0],
        [-1.0, -t, 0.0],
        [1.0, -t, 0.0],
        [0.0, -1.0, t],
        [0.0, 1.0, t],
        [0.0, -1.0, -t],
        [0.0, 1.0, -t],
        [t, 0.0, -1.0],
        [t, 0.0, 1.0],
        [-t, 0.0, -1.0],
        [-t, 0.0, 1.0],
    ];
    verts
        .iter()
        .map(|v| {
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            [v[0] / len, v[1] / len, v[2] / len]
        })
        .collect()
}

pub fn icosahedron_faces() -> Vec<[usize; 3]> {
    vec![
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ]
}

pub fn icosphere_subdivide(verts: &mut Vec<[f32; 3]>, faces: &mut Vec<[usize; 3]>) {
    use std::collections::HashMap;
    let mut midpoints: HashMap<(usize, usize), usize> = HashMap::new();
    let mut new_faces = Vec::new();

    let get_mid = |a: usize,
                   b: usize,
                   verts: &mut Vec<[f32; 3]>,
                   midpoints: &mut HashMap<(usize, usize), usize>|
     -> usize {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = midpoints.get(&key) {
            return idx;
        }
        let va = verts[a];
        let vb = verts[b];
        let mut mid = [
            (va[0] + vb[0]) * 0.5,
            (va[1] + vb[1]) * 0.5,
            (va[2] + vb[2]) * 0.5,
        ];
        let len = (mid[0] * mid[0] + mid[1] * mid[1] + mid[2] * mid[2]).sqrt();
        mid[0] /= len;
        mid[1] /= len;
        mid[2] /= len;
        let idx = verts.len();
        verts.push(mid);
        midpoints.insert(key, idx);
        idx
    };

    for &[a, b, c] in faces.iter() {
        let ab = get_mid(a, b, verts, &mut midpoints);
        let bc = get_mid(b, c, verts, &mut midpoints);
        let ca = get_mid(c, a, verts, &mut midpoints);
        new_faces.push([a, ab, ca]);
        new_faces.push([b, bc, ab]);
        new_faces.push([c, ca, bc]);
        new_faces.push([ab, bc, ca]);
    }
    *faces = new_faces;
}

pub fn icosphere_build(subdivisions: u32) -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
    let mut verts = icosahedron_vertices();
    let mut faces = icosahedron_faces();
    for _ in 0..subdivisions {
        icosphere_subdivide(&mut verts, &mut faces);
    }
    (verts, faces)
}

pub fn icosphere_vertex_count(subdivisions: u32) -> usize {
    10 * 4_usize.pow(subdivisions) + 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icosahedron_vertex_count() {
        /* 12 vertices */
        assert_eq!(icosahedron_vertices().len(), 12);
    }

    #[test]
    fn test_icosahedron_face_count() {
        /* 20 faces */
        assert_eq!(icosahedron_faces().len(), 20);
    }

    #[test]
    fn test_icosahedron_vertices_unit() {
        /* all vertices on unit sphere */
        for v in icosahedron_vertices() {
            let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!((r - 1.0).abs() < 1e-5, "not unit: {}", r);
        }
    }

    #[test]
    fn test_icosphere_build_subdivision_0() {
        /* subdivision 0 = icosahedron */
        let (verts, faces) = icosphere_build(0);
        assert_eq!(verts.len(), 12);
        assert_eq!(faces.len(), 20);
    }

    #[test]
    fn test_icosphere_build_subdivision_1() {
        /* subdivision 1 has 42 vertices */
        let (verts, _) = icosphere_build(1);
        assert_eq!(verts.len(), icosphere_vertex_count(1));
    }

    #[test]
    fn test_icosphere_vertex_count_formula() {
        /* formula matches build */
        let (verts, _) = icosphere_build(2);
        assert_eq!(verts.len(), icosphere_vertex_count(2));
    }
}
