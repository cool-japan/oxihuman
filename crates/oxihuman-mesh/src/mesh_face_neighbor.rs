#![allow(dead_code)]
//! Face neighbor detection for triangle meshes.

use std::collections::HashMap;

/// Stores face adjacency information.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FaceNeighbor {
    /// For each face index, the list of neighboring face indices.
    adjacency: Vec<Vec<usize>>,
}

/// Build face neighbor information from triangle indices.
#[allow(dead_code)]
pub fn build_face_neighbors(indices: &[u32]) -> FaceNeighbor {
    let face_count = indices.len() / 3;
    let mut edge_to_faces: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for fi in 0..face_count {
        let a = indices[fi * 3];
        let b = indices[fi * 3 + 1];
        let c = indices[fi * 3 + 2];
        for &(e0, e1) in &[(a, b), (b, c), (c, a)] {
            let key = if e0 < e1 { (e0, e1) } else { (e1, e0) };
            edge_to_faces.entry(key).or_default().push(fi);
        }
    }
    let mut adjacency = vec![Vec::new(); face_count];
    for faces in edge_to_faces.values() {
        for i in 0..faces.len() {
            for j in (i + 1)..faces.len() {
                if !adjacency[faces[i]].contains(&faces[j]) {
                    adjacency[faces[i]].push(faces[j]);
                }
                if !adjacency[faces[j]].contains(&faces[i]) {
                    adjacency[faces[j]].push(faces[i]);
                }
            }
        }
    }
    FaceNeighbor { adjacency }
}

/// Return the neighbors of a given face.
#[allow(dead_code)]
pub fn neighbors_of_face(fn_data: &FaceNeighbor, face_idx: usize) -> &[usize] {
    fn_data.adjacency.get(face_idx).map_or(&[], |v| v.as_slice())
}

/// Return the neighbor count for a face.
#[allow(dead_code)]
pub fn face_neighbor_count(fn_data: &FaceNeighbor, face_idx: usize) -> usize {
    fn_data.adjacency.get(face_idx).map_or(0, |v| v.len())
}

/// Return the shared edge between two faces, or None.
#[allow(dead_code)]
pub fn shared_edge(indices: &[u32], face_a: usize, face_b: usize) -> Option<(u32, u32)> {
    let a_verts: Vec<u32> = (0..3).map(|i| indices[face_a * 3 + i]).collect();
    let b_verts: Vec<u32> = (0..3).map(|i| indices[face_b * 3 + i]).collect();
    let mut shared = Vec::new();
    for &v in &a_verts {
        if b_verts.contains(&v) {
            shared.push(v);
        }
    }
    if shared.len() >= 2 {
        Some((shared[0], shared[1]))
    } else {
        None
    }
}

/// Build a face adjacency matrix (flattened bool array).
#[allow(dead_code)]
pub fn face_adjacency_matrix(fn_data: &FaceNeighbor) -> Vec<bool> {
    let n = fn_data.adjacency.len();
    let mut mat = vec![false; n * n];
    for (i, neighbors) in fn_data.adjacency.iter().enumerate() {
        for &j in neighbors {
            mat[i * n + j] = true;
            mat[j * n + i] = true;
        }
    }
    mat
}

/// Convert neighbor data to JSON.
#[allow(dead_code)]
pub fn neighbor_to_json(fn_data: &FaceNeighbor) -> String {
    let entries: Vec<String> = fn_data.adjacency.iter().enumerate().map(|(i, ns)| {
        let ns_str: Vec<String> = ns.iter().map(|n| n.to_string()).collect();
        format!("\"{}\":[{}]", i, ns_str.join(","))
    }).collect();
    format!("{{{}}}", entries.join(","))
}

/// Check if a face has a specific neighbor.
#[allow(dead_code)]
pub fn face_has_neighbor(fn_data: &FaceNeighbor, face_idx: usize, neighbor: usize) -> bool {
    fn_data.adjacency.get(face_idx).is_some_and(|v| v.contains(&neighbor))
}

/// Clear all face neighbor data.
#[allow(dead_code)]
pub fn clear_face_neighbors(fn_data: &mut FaceNeighbor) {
    fn_data.adjacency.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3]
    }

    #[test]
    fn test_build_face_neighbors() {
        let fn_data = build_face_neighbors(&quad_indices());
        assert_eq!(fn_data.adjacency.len(), 2);
    }

    #[test]
    fn test_neighbors_of_face() {
        let fn_data = build_face_neighbors(&quad_indices());
        let ns = neighbors_of_face(&fn_data, 0);
        assert!(ns.contains(&1));
    }

    #[test]
    fn test_face_neighbor_count() {
        let fn_data = build_face_neighbors(&quad_indices());
        assert_eq!(face_neighbor_count(&fn_data, 0), 1);
    }

    #[test]
    fn test_shared_edge() {
        let idx = quad_indices();
        let edge = shared_edge(&idx, 0, 1);
        assert!(edge.is_some());
    }

    #[test]
    fn test_face_adjacency_matrix() {
        let fn_data = build_face_neighbors(&quad_indices());
        let mat = face_adjacency_matrix(&fn_data);
        assert_eq!(mat.len(), 4);
        assert!(mat[1]);
    }

    #[test]
    fn test_neighbor_to_json() {
        let fn_data = build_face_neighbors(&quad_indices());
        let j = neighbor_to_json(&fn_data);
        assert!(j.contains("\"0\""));
    }

    #[test]
    fn test_face_has_neighbor() {
        let fn_data = build_face_neighbors(&quad_indices());
        assert!(face_has_neighbor(&fn_data, 0, 1));
        assert!(!face_has_neighbor(&fn_data, 0, 5));
    }

    #[test]
    fn test_clear_face_neighbors() {
        let mut fn_data = build_face_neighbors(&quad_indices());
        clear_face_neighbors(&mut fn_data);
        assert!(fn_data.adjacency.is_empty());
    }

    #[test]
    fn test_empty_input() {
        let fn_data = build_face_neighbors(&[]);
        assert!(fn_data.adjacency.is_empty());
    }

    #[test]
    fn test_single_triangle() {
        let fn_data = build_face_neighbors(&[0, 1, 2]);
        assert_eq!(fn_data.adjacency.len(), 1);
        assert!(neighbors_of_face(&fn_data, 0).is_empty());
    }
}
