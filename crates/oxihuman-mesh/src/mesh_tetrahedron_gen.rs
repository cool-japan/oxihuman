// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

pub fn tetrahedron_vertices() -> [[f32; 3]; 4] {
    let s = 1.0_f32 / 3.0_f32.sqrt();
    [
        [1.0, 1.0, 1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
    ]
    .map(|v| [v[0] * s, v[1] * s, v[2] * s])
}

pub fn tetrahedron_faces() -> [[usize; 3]; 4] {
    [[0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2]]
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn tetrahedron_edge_length(v: &[[f32; 3]; 4]) -> f32 {
    dist(v[0], v[1])
}

pub fn tetrahedron_volume(v: &[[f32; 3]; 4]) -> f32 {
    let ab = [v[1][0] - v[0][0], v[1][1] - v[0][1], v[1][2] - v[0][2]];
    let ac = [v[2][0] - v[0][0], v[2][1] - v[0][1], v[2][2] - v[0][2]];
    let ad = [v[3][0] - v[0][0], v[3][1] - v[0][1], v[3][2] - v[0][2]];
    let cross = [
        ac[1] * ad[2] - ac[2] * ad[1],
        ac[2] * ad[0] - ac[0] * ad[2],
        ac[0] * ad[1] - ac[1] * ad[0],
    ];
    (ab[0] * cross[0] + ab[1] * cross[1] + ab[2] * cross[2]).abs() / 6.0
}

fn tri_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt() * 0.5
}

pub fn tetrahedron_surface_area(v: &[[f32; 3]; 4]) -> f32 {
    let faces = tetrahedron_faces();
    faces
        .iter()
        .map(|&[a, b, c]| tri_area(v[a], v[b], v[c]))
        .sum()
}

pub fn tetrahedron_circumradius(edge_len: f32) -> f32 {
    edge_len * (6.0_f32).sqrt() / 4.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tetrahedron_vertices_count() {
        /* 4 vertices */
        let v = tetrahedron_vertices();
        assert_eq!(v.len(), 4);
    }

    #[test]
    fn test_tetrahedron_faces_count() {
        /* 4 faces */
        let f = tetrahedron_faces();
        assert_eq!(f.len(), 4);
    }

    #[test]
    fn test_tetrahedron_equal_edges() {
        /* regular tet: all edges equal */
        let v = tetrahedron_vertices();
        let e01 = super::dist(v[0], v[1]);
        let e02 = super::dist(v[0], v[2]);
        assert!((e01 - e02).abs() < 1e-5);
    }

    #[test]
    fn test_tetrahedron_volume_positive() {
        /* volume > 0 */
        let v = tetrahedron_vertices();
        assert!(tetrahedron_volume(&v) > 0.0);
    }

    #[test]
    fn test_tetrahedron_surface_area_positive() {
        /* surface area > 0 */
        let v = tetrahedron_vertices();
        assert!(tetrahedron_surface_area(&v) > 0.0);
    }

    #[test]
    fn test_tetrahedron_circumradius() {
        /* circumradius = edge * sqrt(6)/4 */
        let v = tetrahedron_vertices();
        let el = tetrahedron_edge_length(&v);
        let cr = tetrahedron_circumradius(el);
        assert!(cr > 0.0);
    }
}
