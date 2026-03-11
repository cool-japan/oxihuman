// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Flatten selected faces to their best-fit plane using PCA-style projection.

/// Result of the flatten operation.
#[derive(Debug, Clone, Default)]
pub struct FlattenFaceResult {
    pub vertices_projected: usize,
    pub plane_normal: [f32; 3],
    pub plane_origin: [f32; 3],
}

/// Computes the centroid of a set of vertex indices.
pub fn vertex_centroid(positions: &[[f32; 3]], vertex_ids: &[u32]) -> [f32; 3] {
    if vertex_ids.is_empty() {
        return [0.0; 3];
    }
    let mut cx = 0.0f32;
    let mut cy = 0.0f32;
    let mut cz = 0.0f32;
    let mut count = 0usize;
    for &vi in vertex_ids {
        let vi = vi as usize;
        if vi < positions.len() {
            cx += positions[vi][0];
            cy += positions[vi][1];
            cz += positions[vi][2];
            count += 1;
        }
    }
    if count == 0 {
        return [0.0; 3];
    }
    [cx / count as f32, cy / count as f32, cz / count as f32]
}

/// Normalises a 3-D vector; returns zero vector if magnitude is too small.
pub fn safe_normalize_ff(v: [f32; 3]) -> [f32; 3] {
    let mag = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if mag < 1e-10 {
        return [0.0, 1.0, 0.0];
    }
    [v[0] / mag, v[1] / mag, v[2] / mag]
}

/// Estimates the best-fit plane normal by averaging face normals.
pub fn best_fit_plane_normal(
    positions: &[[f32; 3]],
    indices: &[u32],
    selected_faces: &[usize],
) -> ([f32; 3], [f32; 3]) {
    let mut nx = 0.0f32;
    let mut ny = 0.0f32;
    let mut nz = 0.0f32;
    let mut ox = 0.0f32;
    let mut oy = 0.0f32;
    let mut oz = 0.0f32;
    let mut count = 0usize;
    for &fi in selected_faces {
        if fi * 3 + 2 >= indices.len() {
            continue;
        }
        let ia = indices[fi * 3] as usize;
        let ib = indices[fi * 3 + 1] as usize;
        let ic = indices[fi * 3 + 2] as usize;
        if ia >= positions.len() || ib >= positions.len() || ic >= positions.len() {
            continue;
        }
        let a = positions[ia];
        let b = positions[ib];
        let c = positions[ic];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        nx += ab[1] * ac[2] - ab[2] * ac[1];
        ny += ab[2] * ac[0] - ab[0] * ac[2];
        nz += ab[0] * ac[1] - ab[1] * ac[0];
        ox += (a[0] + b[0] + c[0]) / 3.0;
        oy += (a[1] + b[1] + c[1]) / 3.0;
        oz += (a[2] + b[2] + c[2]) / 3.0;
        count += 1;
    }
    if count == 0 {
        return ([0.0, 0.0, 1.0], [0.0; 3]);
    }
    let n = count as f32;
    let normal = safe_normalize_ff([nx / n, ny / n, nz / n]);
    let origin = [ox / n, oy / n, oz / n];
    (normal, origin)
}

/// Projects a single point onto a plane defined by `origin` and `normal`.
pub fn project_to_plane(point: [f32; 3], origin: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let dx = point[0] - origin[0];
    let dy = point[1] - origin[1];
    let dz = point[2] - origin[2];
    let dot = dx * normal[0] + dy * normal[1] + dz * normal[2];
    [
        point[0] - dot * normal[0],
        point[1] - dot * normal[1],
        point[2] - dot * normal[2],
    ]
}

/// Flattens all vertices belonging to selected faces onto the best-fit plane.
pub fn flatten_selected_faces(
    positions: &mut [[f32; 3]],
    indices: &[u32],
    selected_faces: &[usize],
) -> FlattenFaceResult {
    let (normal, origin) = best_fit_plane_normal(positions, indices, selected_faces);
    /* collect unique vertices */
    let mut verts: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for &fi in selected_faces {
        if fi * 3 + 2 < indices.len() {
            verts.insert(indices[fi * 3]);
            verts.insert(indices[fi * 3 + 1]);
            verts.insert(indices[fi * 3 + 2]);
        }
    }
    let mut projected = 0usize;
    for &vi in &verts {
        let vi = vi as usize;
        if vi < positions.len() {
            positions[vi] = project_to_plane(positions[vi], origin, normal);
            projected += 1;
        }
    }
    FlattenFaceResult {
        vertices_projected: projected,
        plane_normal: normal,
        plane_origin: origin,
    }
}

/// Computes the planarity error (average distance from the best-fit plane).
pub fn planarity_error(positions: &[[f32; 3]], indices: &[u32], selected_faces: &[usize]) -> f32 {
    let (normal, origin) = best_fit_plane_normal(positions, indices, selected_faces);
    let mut total_dist = 0.0f32;
    let mut count = 0usize;
    let mut verts: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for &fi in selected_faces {
        if fi * 3 + 2 < indices.len() {
            verts.insert(indices[fi * 3]);
            verts.insert(indices[fi * 3 + 1]);
            verts.insert(indices[fi * 3 + 2]);
        }
    }
    for &vi in &verts {
        let vi = vi as usize;
        if vi < positions.len() {
            let p = positions[vi];
            let dx = p[0] - origin[0];
            let dy = p[1] - origin[1];
            let dz = p[2] - origin[2];
            let dist = (dx * normal[0] + dy * normal[1] + dz * normal[2]).abs();
            total_dist += dist;
            count += 1;
        }
    }
    if count == 0 {
        0.0
    } else {
        total_dist / count as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_tri() -> (Vec<[f32; 3]>, Vec<u32>) {
        let pos = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        (pos, idx)
    }

    #[test]
    fn vertex_centroid_triangle() {
        let (pos, _) = flat_tri();
        let c = vertex_centroid(&pos, &[0, 1, 2]);
        assert!((c[0] - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn safe_normalize_unit() {
        let v = safe_normalize_ff([3.0, 0.0, 0.0]);
        assert!((v[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn project_to_plane_already_on_plane() {
        let p = [1.0f32, 0.0, 0.0];
        let origin = [0.0f32; 3];
        let normal = [0.0f32, 0.0, 1.0];
        let projected = project_to_plane(p, origin, normal);
        assert!((projected[2]).abs() < 1e-5);
    }

    #[test]
    fn best_fit_plane_z_axis() {
        let (pos, idx) = flat_tri();
        let (normal, _) = best_fit_plane_normal(&pos, &idx, &[0]);
        /* z-component should be ≈ 1 */
        assert!(normal[2].abs() > 0.9);
    }

    #[test]
    fn flatten_selected_faces_projects() {
        let (mut pos, idx) = flat_tri();
        let res = flatten_selected_faces(&mut pos, &idx, &[0]);
        assert_eq!(res.vertices_projected, 3);
    }

    #[test]
    fn planarity_error_zero_for_flat() {
        let (pos, idx) = flat_tri();
        let err = planarity_error(&pos, &idx, &[0]);
        assert!(err < 1e-5);
    }

    #[test]
    fn vertex_centroid_empty() {
        let pos: Vec<[f32; 3]> = vec![];
        let c = vertex_centroid(&pos, &[]);
        assert_eq!(c, [0.0; 3]);
    }

    #[test]
    fn safe_normalize_zero_gives_up() {
        let v = safe_normalize_ff([0.0, 0.0, 0.0]);
        assert_eq!(v, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn flatten_empty_selection_no_crash() {
        let (mut pos, idx) = flat_tri();
        let res = flatten_selected_faces(&mut pos, &idx, &[]);
        assert_eq!(res.vertices_projected, 0);
    }
}
