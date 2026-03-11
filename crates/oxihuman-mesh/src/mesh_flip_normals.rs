// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Flip face normals.

#[derive(Debug, Clone)]
pub struct FlipNormalsResult {
    pub indices: Vec<u32>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub flipped_face_count: usize,
}

/// Flip the winding of all triangles in an index buffer.
pub fn flip_all_windings(indices: &[u32]) -> Vec<u32> {
    let mut out = indices.to_vec();
    for tri in out.chunks_mut(3) {
        if tri.len() == 3 {
            tri.swap(1, 2);
        }
    }
    out
}

/// Flip the winding of selected triangle faces.
pub fn flip_selected_windings(indices: &[u32], face_indices: &[usize]) -> Vec<u32> {
    let mut out = indices.to_vec();
    for &fi in face_indices {
        let base = fi * 3;
        if base + 2 < out.len() {
            out.swap(base + 1, base + 2);
        }
    }
    out
}

/// Negate a set of normals (flip them).
pub fn negate_normals(normals: &[[f32; 3]]) -> Vec<[f32; 3]> {
    normals.iter().map(|&n| [-n[0], -n[1], -n[2]]).collect()
}

/// Flip normals of selected vertices.
pub fn negate_selected_normals(normals: &[[f32; 3]], vertex_indices: &[usize]) -> Vec<[f32; 3]> {
    let mut out = normals.to_vec();
    for &vi in vertex_indices {
        if vi < out.len() {
            out[vi] = [-out[vi][0], -out[vi][1], -out[vi][2]];
        }
    }
    out
}

/// Flip all normals and winding orders together.
pub fn flip_normals(indices: &[u32], normals: Option<&[[f32; 3]]>) -> FlipNormalsResult {
    let flipped_indices = flip_all_windings(indices);
    let flipped_normals = normals.map(negate_normals);
    let flipped_face_count = indices.len() / 3;
    FlipNormalsResult {
        indices: flipped_indices,
        normals: flipped_normals,
        flipped_face_count,
    }
}

/// Flip only a selection of faces.
pub fn flip_normals_selected(
    indices: &[u32],
    normals: Option<&[[f32; 3]]>,
    face_indices: &[usize],
) -> FlipNormalsResult {
    let flipped_indices = flip_selected_windings(indices, face_indices);
    let flipped_normals = normals.map(|n| n.to_vec());
    let flipped_face_count = face_indices.len();
    FlipNormalsResult {
        indices: flipped_indices,
        normals: flipped_normals,
        flipped_face_count,
    }
}

/// Check if two triangle windings are consistent (shared edge is opposite direction).
pub fn windings_consistent(a: [u32; 3], b: [u32; 3]) -> bool {
    for i in 0..3 {
        let v0 = a[i];
        let v1 = a[(i + 1) % 3];
        for j in 0..3 {
            let w0 = b[j];
            let w1 = b[(j + 1) % 3];
            if v0 == w1 && v1 == w0 {
                return true;
            }
        }
    }
    false
}

/// Count inconsistent windings in a mesh (simple heuristic).
pub fn count_inconsistent_faces(indices: &[u32]) -> usize {
    let face_count = indices.len() / 3;
    let mut inconsistent = 0usize;
    for i in 0..face_count {
        for j in i + 1..face_count.min(i + 10) {
            let a = [indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]];
            let b = [indices[j * 3], indices[j * 3 + 1], indices[j * 3 + 2]];
            if !windings_consistent(a, b) {
                /* check they share an edge first */
                let shared: Vec<u32> = a.iter().filter(|v| b.contains(v)).cloned().collect();
                if shared.len() == 2 {
                    inconsistent += 1;
                }
            }
        }
    }
    inconsistent
}

/// Normalize a vector.
fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-10 {
        [0.0; 3]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// Compute the face normal for a triangle.
pub fn tri_normal(positions: &[[f32; 3]], tri: [u32; 3]) -> [f32; 3] {
    let a = positions[tri[0] as usize];
    let b = positions[tri[1] as usize];
    let c = positions[tri[2] as usize];
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    normalize3([
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_all_windings() {
        let idx = vec![0u32, 1, 2];
        let flipped = flip_all_windings(&idx);
        assert_eq!(flipped, vec![0, 2, 1]);
    }

    #[test]
    fn test_flip_all_windings_multiple() {
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        let flipped = flip_all_windings(&idx);
        assert_eq!(flipped[1], 2);
        assert_eq!(flipped[4], 5);
    }

    #[test]
    fn test_negate_normals() {
        let n = vec![[1.0f32, 0.0, 0.0]];
        let neg = negate_normals(&n);
        assert!((neg[0][0] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_flip_normals() {
        let idx = vec![0u32, 1, 2];
        let nrm = vec![[0.0f32, 0.0, 1.0]];
        let res = flip_normals(&idx, Some(&nrm));
        assert_eq!(res.flipped_face_count, 1);
        let n = res.normals.unwrap();
        assert!(n[0][2] < 0.0);
    }

    #[test]
    fn test_flip_selected() {
        let idx = vec![0u32, 1, 2, 3, 4, 5];
        let res = flip_normals_selected(&idx, None, &[0]);
        assert_eq!(res.indices[1], 2);
        assert_eq!(res.indices[4], 4);
    }

    #[test]
    fn test_windings_consistent() {
        let a = [0u32, 1, 2];
        let b = [1u32, 0, 3]; /* shares 0-1 in opposite direction */
        assert!(windings_consistent(a, b));
    }

    #[test]
    fn test_tri_normal_z() {
        let pos = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let n = tri_normal(&pos, [0, 1, 2]);
        assert!(n[2].abs() > 0.5);
    }

    #[test]
    fn test_negate_selected_normals() {
        let n = vec![[0.0, 0.0, 1.0f32], [0.0, 0.0, 1.0]];
        let res = negate_selected_normals(&n, &[0]);
        assert!(res[0][2] < 0.0);
        assert!(res[1][2] > 0.0);
    }
}
