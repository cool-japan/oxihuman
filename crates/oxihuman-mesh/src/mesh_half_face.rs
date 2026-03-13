// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Half-face data structure for tetrahedral and polygonal mesh topology.

/// A half-face: one oriented side of a polygonal face.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct HalfFace {
    pub face_idx: u32,
    pub vertices: [u32; 3],
    pub twin: Option<u32>,
}

/// Collection of half-faces built from triangle indices.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HalfFaceMesh {
    pub half_faces: Vec<HalfFace>,
    pub face_count: usize,
}

/// Build half-face mesh from triangle index buffer.
#[allow(dead_code)]
pub fn build_half_face_mesh(indices: &[u32]) -> HalfFaceMesh {
    use std::collections::HashMap;
    let face_count = indices.len() / 3;
    let mut half_faces: Vec<HalfFace> = (0..face_count as u32)
        .map(|f| {
            let base = f as usize * 3;
            HalfFace {
                face_idx: f,
                vertices: [indices[base], indices[base + 1], indices[base + 2]],
                twin: None,
            }
        })
        .collect();
    // Match twins by canonical face (sorted vertices).
    let mut face_map: HashMap<[u32; 3], u32> = HashMap::new();
    for (i, hf) in half_faces.iter().enumerate() {
        let mut key = hf.vertices;
        key.sort_unstable();
        face_map.entry(key).or_insert(i as u32);
    }
    #[allow(clippy::needless_range_loop)]
    for i in 0..half_faces.len() {
        let mut key = half_faces[i].vertices;
        key.sort_unstable();
        if let Some(&twin_idx) = face_map.get(&key) {
            if twin_idx != i as u32 {
                half_faces[i].twin = Some(twin_idx);
            }
        }
    }
    HalfFaceMesh {
        half_faces,
        face_count,
    }
}

/// Total half-face count.
#[allow(dead_code)]
pub fn half_face_count(hfm: &HalfFaceMesh) -> usize {
    hfm.half_faces.len()
}

/// Count half-faces with a twin.
#[allow(dead_code)]
pub fn paired_half_face_count(hfm: &HalfFaceMesh) -> usize {
    hfm.half_faces.iter().filter(|hf| hf.twin.is_some()).count()
}

/// Count boundary half-faces (no twin).
#[allow(dead_code)]
pub fn boundary_half_face_count(hfm: &HalfFaceMesh) -> usize {
    hfm.half_faces.iter().filter(|hf| hf.twin.is_none()).count()
}

/// Get vertices of a half-face.
#[allow(dead_code)]
pub fn half_face_vertices(hfm: &HalfFaceMesh, idx: usize) -> Option<[u32; 3]> {
    hfm.half_faces.get(idx).map(|hf| hf.vertices)
}

/// Check if mesh is closed (all half-faces have twins).
#[allow(dead_code)]
pub fn is_closed_half_face_mesh(hfm: &HalfFaceMesh) -> bool {
    hfm.half_faces.iter().all(|hf| hf.twin.is_some())
}

/// Export to JSON.
#[allow(dead_code)]
pub fn half_face_mesh_to_json(hfm: &HalfFaceMesh) -> String {
    format!(
        "{{\"half_face_count\":{},\"face_count\":{},\"boundary\":{}}}",
        half_face_count(hfm),
        hfm.face_count,
        boundary_half_face_count(hfm)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_tri_indices() -> Vec<u32> {
        vec![0, 1, 2, 2, 1, 3]
    }

    #[test]
    fn test_build_half_face_mesh_empty() {
        let hfm = build_half_face_mesh(&[]);
        assert_eq!(half_face_count(&hfm), 0);
    }

    #[test]
    fn test_build_half_face_mesh_single() {
        let hfm = build_half_face_mesh(&[0, 1, 2]);
        assert_eq!(half_face_count(&hfm), 1);
    }

    #[test]
    fn test_paired_half_faces() {
        let hfm = build_half_face_mesh(&two_tri_indices());
        assert_eq!(half_face_count(&hfm), 2);
    }

    #[test]
    fn test_half_face_vertices() {
        let hfm = build_half_face_mesh(&[0, 1, 2]);
        let v = half_face_vertices(&hfm, 0);
        assert!(v.is_some_and(|verts| verts[0] == 0));
    }

    #[test]
    fn test_half_face_vertices_oob() {
        let hfm = build_half_face_mesh(&[]);
        assert!(half_face_vertices(&hfm, 0).is_none());
    }

    #[test]
    fn test_boundary_half_face_count_open_mesh() {
        let hfm = build_half_face_mesh(&[0, 1, 2]);
        assert_eq!(boundary_half_face_count(&hfm), 1);
    }

    #[test]
    fn test_is_closed_single_tri() {
        let hfm = build_half_face_mesh(&[0, 1, 2]);
        assert!(!is_closed_half_face_mesh(&hfm));
    }

    #[test]
    fn test_half_face_mesh_to_json() {
        let hfm = build_half_face_mesh(&[0, 1, 2]);
        let j = half_face_mesh_to_json(&hfm);
        assert!(j.contains("\"half_face_count\":1"));
    }

    #[test]
    fn test_boundary_count_non_negative() {
        let hfm = build_half_face_mesh(&two_tri_indices());
        let bc = boundary_half_face_count(&hfm);
        let pc = paired_half_face_count(&hfm);
        assert_eq!(bc + pc, half_face_count(&hfm));
    }
}
