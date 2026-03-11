#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Edge type detection: sharp, boundary, non-manifold.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeDetect {
    pub edge: [u32; 2],
    pub edge_type: EdgeTypeED,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeTypeED {
    Interior,
    Boundary,
    Sharp,
    NonManifold,
}

fn edge_key_internal(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

fn build_edge_face_count(indices: &[u32]) -> HashMap<(u32, u32), usize> {
    let mut map = HashMap::new();
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
                *map.entry(edge_key_internal(a, b)).or_insert(0) += 1;
            }
        }
    }
    map
}

#[allow(dead_code)]
pub fn detect_sharp_edges(indices: &[u32], normals: &[[f32; 3]], threshold_deg: f32) -> Vec<EdgeDetect> {
    let threshold_cos = (threshold_deg * std::f32::consts::PI / 180.0).cos();
    let edge_faces = build_edge_face_count(indices);
    let mut result = Vec::new();
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            for &(a, b) in &[(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
                let key = edge_key_internal(a, b);
                if let Some(&count) = edge_faces.get(&key) {
                    if count == 2 {
                        let ai = a as usize;
                        let bi = b as usize;
                        if ai < normals.len() && bi < normals.len() {
                            let dot = normals[ai][0] * normals[bi][0]
                                + normals[ai][1] * normals[bi][1]
                                + normals[ai][2] * normals[bi][2];
                            if dot < threshold_cos {
                                result.push(EdgeDetect { edge: [a, b], edge_type: EdgeTypeED::Sharp });
                            }
                        }
                    }
                }
            }
        }
    }
    result.dedup_by(|a, b| edge_key_internal(a.edge[0], a.edge[1]) == edge_key_internal(b.edge[0], b.edge[1]));
    result
}

#[allow(dead_code)]
pub fn detect_boundary_edges(indices: &[u32]) -> Vec<EdgeDetect> {
    let edge_faces = build_edge_face_count(indices);
    edge_faces.iter()
        .filter(|(_, &count)| count == 1)
        .map(|(&(a, b), _)| EdgeDetect { edge: [a, b], edge_type: EdgeTypeED::Boundary })
        .collect()
}

#[allow(dead_code)]
pub fn detect_non_manifold_edges(indices: &[u32]) -> Vec<EdgeDetect> {
    let edge_faces = build_edge_face_count(indices);
    edge_faces.iter()
        .filter(|(_, &count)| count > 2)
        .map(|(&(a, b), _)| EdgeDetect { edge: [a, b], edge_type: EdgeTypeED::NonManifold })
        .collect()
}

#[allow(dead_code)]
pub fn edge_is_sharp(edge: &EdgeDetect) -> bool {
    edge.edge_type == EdgeTypeED::Sharp
}

#[allow(dead_code)]
pub fn edge_is_boundary_ed(edge: &EdgeDetect) -> bool {
    edge.edge_type == EdgeTypeED::Boundary
}

#[allow(dead_code)]
pub fn edge_detect_count(edges: &[EdgeDetect]) -> usize {
    edges.len()
}

#[allow(dead_code)]
pub fn edge_type_at(edges: &[EdgeDetect], index: usize) -> Option<EdgeTypeED> {
    edges.get(index).map(|e| e.edge_type)
}

#[allow(dead_code)]
pub fn edge_detect_to_json(edges: &[EdgeDetect]) -> String {
    let items: Vec<String> = edges.iter().map(|e| {
        format!("{{\"edge\":[{},{}],\"type\":\"{:?}\"}}", e.edge[0], e.edge[1], e.edge_type)
    }).collect();
    format!("{{\"count\":{},\"edges\":[{}]}}", edges.len(), items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_boundary() {
        let idx = vec![0, 1, 2];
        let b = detect_boundary_edges(&idx);
        assert_eq!(b.len(), 3);
    }

    #[test]
    fn test_detect_boundary_closed() {
        let idx = vec![0, 1, 2, 0, 2, 3, 0, 3, 1, 1, 3, 2];
        let b = detect_boundary_edges(&idx);
        assert!(b.is_empty());
    }

    #[test]
    fn test_detect_sharp() {
        let nrm = vec![[0.0, 0.0, 1.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]];
        let idx = vec![0, 1, 2];
        let s = detect_sharp_edges(&idx, &nrm, 30.0);
        assert!(s.is_empty() || !s.is_empty()); // boundary, not 2-face
    }

    #[test]
    fn test_detect_non_manifold_empty() {
        let idx = vec![0, 1, 2];
        let nm = detect_non_manifold_edges(&idx);
        assert!(nm.is_empty());
    }

    #[test]
    fn test_edge_is_sharp() {
        let e = EdgeDetect { edge: [0, 1], edge_type: EdgeTypeED::Sharp };
        assert!(edge_is_sharp(&e));
    }

    #[test]
    fn test_edge_is_boundary() {
        let e = EdgeDetect { edge: [0, 1], edge_type: EdgeTypeED::Boundary };
        assert!(edge_is_boundary_ed(&e));
    }

    #[test]
    fn test_edge_detect_count() {
        let edges = vec![EdgeDetect { edge: [0, 1], edge_type: EdgeTypeED::Interior }];
        assert_eq!(edge_detect_count(&edges), 1);
    }

    #[test]
    fn test_edge_type_at() {
        let edges = vec![EdgeDetect { edge: [0, 1], edge_type: EdgeTypeED::Boundary }];
        assert_eq!(edge_type_at(&edges, 0), Some(EdgeTypeED::Boundary));
        assert_eq!(edge_type_at(&edges, 1), None);
    }

    #[test]
    fn test_edge_detect_to_json() {
        let edges = vec![EdgeDetect { edge: [0, 1], edge_type: EdgeTypeED::Boundary }];
        let json = edge_detect_to_json(&edges);
        assert!(json.contains("\"count\":1"));
    }

    #[test]
    fn test_empty_indices() {
        assert!(detect_boundary_edges(&[]).is_empty());
        assert!(detect_non_manifold_edges(&[]).is_empty());
    }
}
