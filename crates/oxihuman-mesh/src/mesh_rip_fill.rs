// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Rip-and-fill region: rip a set of vertices and fill the resulting hole.

/// Result of a rip-fill operation.
#[derive(Debug, Clone)]
pub struct RipFillResult {
    pub new_positions: Vec<[f32; 3]>,
    pub new_indices: Vec<u32>,
    pub fill_faces_added: usize,
    pub ripped_vertices: usize,
}

/// Duplicates a vertex.
pub fn duplicate_vertex(positions: &mut Vec<[f32; 3]>, vertex: u32) -> u32 {
    let vi = vertex as usize;
    if vi >= positions.len() {
        return vertex;
    }
    let pos = positions[vi];
    positions.push(pos);
    (positions.len() - 1) as u32
}

/// Computes a face normal (unnormalised) from three positions.
pub fn face_normal_rf(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ]
}

/// Generates a simple fan fill from a boundary loop of vertices.
/// `boundary` should be an ordered list of vertex indices forming the hole rim.
pub fn fan_fill_boundary(positions: &[[f32; 3]], boundary: &[u32]) -> Vec<u32> {
    if boundary.len() < 3 {
        return vec![];
    }
    /* place fill centroid */
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    for &vi in boundary {
        let vi = vi as usize;
        if vi < positions.len() {
            cx += positions[vi][0];
            cy += positions[vi][1];
            cz += positions[vi][2];
        }
    }
    let n = boundary.len() as f32;
    let _ = [cx / n, cy / n, cz / n]; /* centroid for future use */
    /* fan from boundary[0] */
    let pivot = boundary[0];
    let mut tris = Vec::new();
    for i in 1..boundary.len().saturating_sub(1) {
        tris.push(pivot);
        tris.push(boundary[i]);
        tris.push(boundary[i + 1]);
    }
    tris
}

/// Returns all faces that use any of the rip vertices.
pub fn faces_with_any_vertex(indices: &[u32], verts: &[u32]) -> Vec<usize> {
    let vset: std::collections::HashSet<u32> = verts.iter().copied().collect();
    let n = indices.len() / 3;
    let mut out = Vec::new();
    for i in 0..n {
        let ia = indices[i * 3];
        let ib = indices[i * 3 + 1];
        let ic = indices[i * 3 + 2];
        if vset.contains(&ia) || vset.contains(&ib) || vset.contains(&ic) {
            out.push(i);
        }
    }
    out
}

/// Performs a rip-fill: duplicates `rip_verts`, removes their connected faces,
/// then adds a fan fill from the resulting boundary.
#[allow(clippy::too_many_arguments)]
pub fn rip_fill(
    positions: &[[f32; 3]],
    indices: &[u32],
    rip_verts: &[u32],
    boundary_loop: &[u32],
) -> RipFillResult {
    let mut new_positions = positions.to_vec();
    let ripped = rip_verts.len();
    /* duplicate the ripped vertices */
    let mut new_verts = Vec::with_capacity(ripped);
    for &v in rip_verts {
        let nv = duplicate_vertex(&mut new_positions, v);
        new_verts.push(nv);
    }
    /* keep faces that don't use any rip vert */
    let bad_faces: std::collections::HashSet<usize> = faces_with_any_vertex(indices, rip_verts)
        .into_iter()
        .collect();
    let n = indices.len() / 3;
    let mut new_indices = Vec::with_capacity(indices.len());
    for i in 0..n {
        if !bad_faces.contains(&i) {
            new_indices.push(indices[i * 3]);
            new_indices.push(indices[i * 3 + 1]);
            new_indices.push(indices[i * 3 + 2]);
        }
    }
    /* fill boundary hole */
    let fill = fan_fill_boundary(&new_positions, boundary_loop);
    let fill_faces = fill.len() / 3;
    new_indices.extend(fill);
    RipFillResult {
        new_positions,
        new_indices,
        fill_faces_added: fill_faces,
        ripped_vertices: ripped,
    }
}

/// Validates that boundary indices are within position count.
pub fn boundary_valid(boundary: &[u32], vertex_count: usize) -> bool {
    boundary.iter().all(|&v| (v as usize) < vertex_count)
}

/// Returns the total surface area covered by fill triangles.
pub fn fill_area(positions: &[[f32; 3]], fill_indices: &[u32]) -> f32 {
    let n = fill_indices.len() / 3;
    let mut area = 0.0f32;
    for i in 0..n {
        let ia = fill_indices[i * 3] as usize;
        let ib = fill_indices[i * 3 + 1] as usize;
        let ic = fill_indices[i * 3 + 2] as usize;
        if ia < positions.len() && ib < positions.len() && ic < positions.len() {
            let n_vec = face_normal_rf(positions[ia], positions[ib], positions[ic]);
            let mag = (n_vec[0] * n_vec[0] + n_vec[1] * n_vec[1] + n_vec[2] * n_vec[2]).sqrt();
            area += mag * 0.5;
        }
    }
    area
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square_positions() -> Vec<[f32; 3]> {
        vec![
            [0.0f32, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn duplicate_vertex_adds_one() {
        let mut pos = square_positions();
        let new_v = duplicate_vertex(&mut pos, 0);
        assert_eq!(pos.len(), 5);
        assert_eq!(new_v, 4);
    }

    #[test]
    fn duplicate_out_of_bounds_returns_same() {
        let mut pos = square_positions();
        let v = duplicate_vertex(&mut pos, 99);
        assert_eq!(v, 99);
    }

    #[test]
    fn fan_fill_three_boundary() {
        let pos = square_positions();
        let boundary = vec![0u32, 1, 2, 3];
        let fill = fan_fill_boundary(&pos, &boundary);
        /* fan from 3 vertices minus 1 = 2 triangles */
        assert!(!fill.is_empty());
    }

    #[test]
    fn fan_fill_too_short() {
        let pos = square_positions();
        let boundary = vec![0u32, 1];
        let fill = fan_fill_boundary(&pos, &boundary);
        assert!(fill.is_empty());
    }

    #[test]
    fn faces_with_any_vertex_count() {
        let idx = vec![0u32, 1, 2, 1, 2, 3];
        let result = faces_with_any_vertex(&idx, &[1]);
        assert_eq!(result.len(), 2); /* both faces use vertex 1 */
    }

    #[test]
    fn boundary_valid_all_in_range() {
        let b = vec![0u32, 1, 2];
        assert!(boundary_valid(&b, 4));
    }

    #[test]
    fn boundary_valid_out_of_range() {
        let b = vec![0u32, 9];
        assert!(!boundary_valid(&b, 4));
    }

    #[test]
    fn fill_area_positive() {
        let pos = square_positions();
        let fill = vec![0u32, 1, 2];
        let area = fill_area(&pos, &fill);
        assert!(area > 0.0);
    }

    #[test]
    fn rip_fill_increases_vertices() {
        let pos = square_positions();
        let idx = vec![0u32, 1, 2, 0, 2, 3];
        let res = rip_fill(&pos, &idx, &[0], &[1, 2, 3]);
        assert!(res.new_positions.len() > pos.len());
    }
}
