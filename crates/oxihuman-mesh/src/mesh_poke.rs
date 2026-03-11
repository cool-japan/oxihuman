// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Result of a poke-face operation.
#[allow(dead_code)]
pub struct PokeFaceResult {
    pub new_verts: Vec<[f32; 3]>,
    pub new_tris: Vec<[u32; 3]>,
}

/// Poke a polygon face by adding a center vertex and splitting into triangles.
#[allow(dead_code)]
pub fn poke_face(verts: &[[f32; 3]], face: &[u32], base_idx: u32) -> PokeFaceResult {
    if face.is_empty() {
        return PokeFaceResult { new_verts: Vec::new(), new_tris: Vec::new() };
    }
    // compute centroid
    let n = face.len() as f32;
    let center = face.iter().fold([0.0f32; 3], |acc, &vi| {
        if let Some(v) = verts.get(vi as usize) {
            [acc[0] + v[0] / n, acc[1] + v[1] / n, acc[2] + v[2] / n]
        } else {
            acc
        }
    });
    let center_idx = base_idx;
    let mut new_tris = Vec::with_capacity(face.len());
    for i in 0..face.len() {
        let j = (i + 1) % face.len();
        new_tris.push([face[i], face[j], center_idx]);
    }
    PokeFaceResult { new_verts: vec![center], new_tris }
}

/// Poke a triangle by adding a center vertex, returning 3 new triangles.
#[allow(dead_code)]
pub fn poke_tri(
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
    center: [f32; 3],
    base: u32,
) -> Vec<[u32; 3]> {
    let _ = (v0, v1, v2, center);
    // base+0=v0, base+1=v1, base+2=v2, base+3=center
    vec![
        [base, base + 1, base + 3],
        [base + 1, base + 2, base + 3],
        [base + 2, base, base + 3],
    ]
}

/// Poke a quad face (4 vertices + center) into 4 triangles.
#[allow(dead_code)]
pub fn poke_quad(v0: u32, v1: u32, v2: u32, v3: u32, center: u32) -> Vec<[u32; 3]> {
    vec![
        [v0, v1, center],
        [v1, v2, center],
        [v2, v3, center],
        [v3, v0, center],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri_verts() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [1.0, 2.0, 0.0],
        ]
    }

    #[test]
    fn test_poke_face_triangle_tris_count() {
        let verts = tri_verts();
        let result = poke_face(&verts, &[0, 1, 2], 3);
        assert_eq!(result.new_tris.len(), 3);
    }

    #[test]
    fn test_poke_face_triangle_center_vert() {
        let verts = tri_verts();
        let result = poke_face(&verts, &[0, 1, 2], 3);
        assert_eq!(result.new_verts.len(), 1);
    }

    #[test]
    fn test_poke_face_center_position() {
        let verts = tri_verts();
        let result = poke_face(&verts, &[0, 1, 2], 10);
        let c = result.new_verts[0];
        // centroid of (0,0,0),(2,0,0),(1,2,0) is (1, 2/3, 0)
        assert!((c[0] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_poke_face_empty() {
        let verts = tri_verts();
        let result = poke_face(&verts, &[], 0);
        assert!(result.new_tris.is_empty());
    }

    #[test]
    fn test_poke_tri_returns_3() {
        let r = poke_tri(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0],
            [0.5, 0.33, 0.0], 0,
        );
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn test_poke_tri_indices_include_center() {
        let r = poke_tri(
            [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0],
            [0.5, 0.33, 0.0], 0,
        );
        // center is index base+3 = 3
        let has_center = r.iter().any(|t| t.contains(&3));
        assert!(has_center);
    }

    #[test]
    fn test_poke_quad_returns_4() {
        let r = poke_quad(0, 1, 2, 3, 4);
        assert_eq!(r.len(), 4);
    }

    #[test]
    fn test_poke_quad_all_include_center() {
        let r = poke_quad(0, 1, 2, 3, 4);
        assert!(r.iter().all(|t| t.contains(&4)));
    }

    #[test]
    fn test_poke_face_quad_produces_4_tris() {
        let verts = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let result = poke_face(&verts, &[0, 1, 2, 3], 4);
        assert_eq!(result.new_tris.len(), 4);
    }
}
