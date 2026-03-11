// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// A pure quad mesh.
#[allow(dead_code)]
pub struct QuadMesh {
    pub positions: Vec<[f32; 3]>,
    pub quads: Vec<[u32; 4]>,
}

/// Build a flat quad grid.
#[allow(dead_code)]
pub fn build_quad_grid(rows: usize, cols: usize, scale: f32) -> QuadMesh {
    let mut positions = Vec::new();
    let mut quads = Vec::new();
    for r in 0..=rows {
        for c in 0..=cols {
            positions.push([c as f32 * scale, r as f32 * scale, 0.0]);
        }
    }
    let stride = (cols + 1) as u32;
    for r in 0..rows {
        for c in 0..cols {
            let a = r as u32 * stride + c as u32;
            quads.push([a, a + 1, a + stride + 1, a + stride]);
        }
    }
    QuadMesh { positions, quads }
}

/// Count quads in the mesh.
#[allow(dead_code)]
pub fn quad_mesh_face_count(mesh: &QuadMesh) -> usize {
    mesh.quads.len()
}

/// Count vertices.
#[allow(dead_code)]
pub fn quad_mesh_vertex_count(mesh: &QuadMesh) -> usize {
    mesh.positions.len()
}

/// Convert quads to triangle index list.
#[allow(dead_code)]
pub fn quad_mesh_to_tris(mesh: &QuadMesh) -> Vec<u32> {
    let mut tris = Vec::with_capacity(mesh.quads.len() * 6);
    for &[a, b, c, d] in &mesh.quads {
        tris.extend_from_slice(&[a, b, c, a, c, d]);
    }
    tris
}

/// Check all quad indices are in bounds.
#[allow(dead_code)]
pub fn quad_mesh_indices_valid(mesh: &QuadMesh) -> bool {
    let n = mesh.positions.len() as u32;
    mesh.quads.iter().all(|q| q.iter().all(|&i| i < n))
}

/// Compute the centroid of all positions.
#[allow(dead_code)]
pub fn quad_mesh_centroid(mesh: &QuadMesh) -> [f32; 3] {
    if mesh.positions.is_empty() {
        return [0.0; 3];
    }
    let n = mesh.positions.len() as f32;
    let mut s = [0.0_f32; 3];
    for p in &mesh.positions {
        s[0] += p[0];
        s[1] += p[1];
        s[2] += p[2];
    }
    [s[0] / n, s[1] / n, s[2] / n]
}

/// Serialize to JSON summary.
#[allow(dead_code)]
pub fn quad_mesh_to_json(mesh: &QuadMesh) -> String {
    format!(
        r#"{{"vertices":{},"quads":{}}}"#,
        mesh.positions.len(),
        mesh.quads.len()
    )
}

/// Scale all positions.
#[allow(dead_code)]
pub fn quad_mesh_scale(mesh: &mut QuadMesh, f: f32) {
    for p in &mut mesh.positions {
        p[0] *= f;
        p[1] *= f;
        p[2] *= f;
    }
}

/// Flip winding of all quads.
#[allow(dead_code)]
pub fn quad_mesh_flip_winding(mesh: &mut QuadMesh) {
    for q in &mut mesh.quads {
        q.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_count() {
        let m = build_quad_grid(3, 4, 1.0);
        assert_eq!(quad_mesh_vertex_count(&m), 4 * 5);
    }

    #[test]
    fn face_count() {
        let m = build_quad_grid(3, 4, 1.0);
        assert_eq!(quad_mesh_face_count(&m), 12);
    }

    #[test]
    fn tris_count() {
        let m = build_quad_grid(2, 2, 1.0);
        let tris = quad_mesh_to_tris(&m);
        assert_eq!(tris.len(), 4 * 6);
    }

    #[test]
    fn indices_valid() {
        let m = build_quad_grid(3, 3, 1.0);
        assert!(quad_mesh_indices_valid(&m));
    }

    #[test]
    fn centroid_near_center() {
        let m = build_quad_grid(4, 4, 1.0);
        let c = quad_mesh_centroid(&m);
        assert!((c[0] - 2.0).abs() < 0.1);
        assert!((c[1] - 2.0).abs() < 0.1);
    }

    #[test]
    fn json_contains_quads() {
        let m = build_quad_grid(2, 3, 1.0);
        let j = quad_mesh_to_json(&m);
        assert!(j.contains("\"quads\":6"));
    }

    #[test]
    fn scale_doubles() {
        let mut m = build_quad_grid(1, 1, 1.0);
        quad_mesh_scale(&mut m, 2.0);
        let c = quad_mesh_centroid(&m);
        assert!(c[0] > 0.5);
    }

    #[test]
    fn flip_winding_reverses() {
        let m = build_quad_grid(1, 1, 1.0);
        let original = m.quads[0];
        let mut m2 = build_quad_grid(1, 1, 1.0);
        quad_mesh_flip_winding(&mut m2);
        let flipped = m2.quads[0];
        assert_eq!(flipped[0], original[3]);
    }

    #[test]
    fn empty_centroid() {
        let m = QuadMesh {
            positions: vec![],
            quads: vec![],
        };
        let c = quad_mesh_centroid(&m);
        assert_eq!(c, [0.0; 3]);
    }

    #[test]
    fn tris_divisible_by_three() {
        let m = build_quad_grid(3, 3, 1.0);
        let tris = quad_mesh_to_tris(&m);
        assert_eq!(tris.len() % 3, 0);
    }
}
