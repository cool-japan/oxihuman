// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Icosphere (subdivided icosahedron) mesh generation.

#![allow(dead_code)]

/// Return the base vertices of an icosahedron.
#[allow(dead_code)]
pub fn icosahedron_verts() -> Vec<[f32; 3]> {
    // Golden ratio
    let phi = (1.0 + 5.0f32.sqrt()) * 0.5;
    let norm = (1.0f32 + phi * phi).sqrt();
    let a = 1.0 / norm;
    let b = phi / norm;
    vec![
        [-a, b, 0.0],
        [a, b, 0.0],
        [-a, -b, 0.0],
        [a, -b, 0.0],
        [0.0, -a, b],
        [0.0, a, b],
        [0.0, -a, -b],
        [0.0, a, -b],
        [b, 0.0, -a],
        [b, 0.0, a],
        [-b, 0.0, -a],
        [-b, 0.0, a],
    ]
}

/// Return the base faces of an icosahedron (20 triangles).
#[allow(dead_code)]
pub fn icosahedron_faces() -> Vec<[u32; 3]> {
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

fn normalize_v(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-8);
    [v[0] / len, v[1] / len, v[2] / len]
}

fn midpoint(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    normalize_v([
        (a[0] + b[0]) * 0.5,
        (a[1] + b[1]) * 0.5,
        (a[2] + b[2]) * 0.5,
    ])
}

/// Return the estimated vertex count for an icosphere after `subdivisions` passes.
/// Starts with 12 verts and 20 faces; each subdivision: V' = V + E, F' = 4*F.
/// Approx formula: 10 * 4^n + 2
#[allow(dead_code)]
pub fn icosphere_vert_count(subdivisions: u32) -> usize {
    let faces = 20usize * (4usize.pow(subdivisions));
    faces / 2 + 2
}

/// Generate an icosphere mesh by subdividing an icosahedron.
/// Returns (vertices, triangles). Vertices are projected onto a unit sphere then scaled by `radius`.
#[allow(dead_code)]
pub fn make_icosphere(radius: f32, subdivisions: u32) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    let base_verts = icosahedron_verts();
    // Use only the first 12 (not the dummy zero vertex)
    let mut verts: Vec<[f32; 3]> = base_verts[..12].to_vec();
    let mut faces: Vec<[u32; 3]> = icosahedron_faces();

    for _ in 0..subdivisions {
        let mut new_faces: Vec<[u32; 3]> = Vec::with_capacity(faces.len() * 4);
        use std::collections::HashMap;
        let mut mid_cache: HashMap<(u32, u32), u32> = HashMap::new();

        let mut get_mid = |a: u32, b: u32, verts: &mut Vec<[f32; 3]>| -> u32 {
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&idx) = mid_cache.get(&key) {
                return idx;
            }
            let m = midpoint(verts[a as usize], verts[b as usize]);
            let idx = verts.len() as u32;
            verts.push(m);
            mid_cache.insert(key, idx);
            idx
        };

        for face in &faces {
            let a = face[0];
            let b = face[1];
            let c = face[2];
            let ab = get_mid(a, b, &mut verts);
            let bc = get_mid(b, c, &mut verts);
            let ca = get_mid(c, a, &mut verts);
            new_faces.push([a, ab, ca]);
            new_faces.push([b, bc, ab]);
            new_faces.push([c, ca, bc]);
            new_faces.push([ab, bc, ca]);
        }
        faces = new_faces;
    }

    // Scale to radius
    let scaled: Vec<[f32; 3]> = verts
        .iter()
        .map(|v| {
            let n = normalize_v(*v);
            [n[0] * radius, n[1] * radius, n[2] * radius]
        })
        .collect();

    (scaled, faces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icosahedron_verts_count() {
        assert_eq!(icosahedron_verts().len(), 12);
    }

    #[test]
    fn test_icosahedron_faces_count() {
        assert_eq!(icosahedron_faces().len(), 20);
    }

    #[test]
    fn test_make_icosphere_zero_subdivisions() {
        let (verts, tris) = make_icosphere(1.0, 0);
        assert_eq!(verts.len(), 12);
        assert_eq!(tris.len(), 20);
    }

    #[test]
    fn test_make_icosphere_one_subdivision() {
        let (verts, tris) = make_icosphere(1.0, 1);
        assert_eq!(tris.len(), 80);
        // 42 verts expected
        assert_eq!(verts.len(), 42);
    }

    #[test]
    fn test_icosphere_radius() {
        let (verts, _) = make_icosphere(2.0, 0);
        for v in &verts {
            let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!((r - 2.0).abs() < 1e-4, "r={}", r);
        }
    }

    #[test]
    fn test_icosphere_indices_in_range() {
        let (verts, tris) = make_icosphere(1.0, 1);
        let nv = verts.len() as u32;
        for tri in &tris {
            assert!(tri[0] < nv);
            assert!(tri[1] < nv);
            assert!(tri[2] < nv);
        }
    }

    #[test]
    fn test_icosphere_two_subdivisions() {
        let (_, tris) = make_icosphere(1.0, 2);
        assert_eq!(tris.len(), 320);
    }

    #[test]
    fn test_icosphere_vert_count_formula() {
        // 0 subdivisions: 10*1 + 2 = 12
        assert_eq!(icosphere_vert_count(0), 12);
        // 1 subdivision: 10*4 + 2 = 42
        assert_eq!(icosphere_vert_count(1), 42);
    }

    #[test]
    fn test_unit_sphere_vert_distance() {
        let (verts, _) = make_icosphere(1.0, 0);
        for v in &verts {
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            assert!((len - 1.0).abs() < 1e-4);
        }
    }
}
