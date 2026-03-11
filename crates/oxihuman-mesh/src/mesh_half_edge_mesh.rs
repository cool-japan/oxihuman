// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Half-edge mesh data structure for efficient traversal.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HalfEdgeMeshData {
    pub vertex: u32,
    pub face: Option<u32>,
    pub next: u32,
    pub twin: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct HalfEdgeMeshStructure {
    pub half_edges: Vec<HalfEdgeMeshData>,
    pub vertex_count: usize,
    pub face_count: usize,
}

#[allow(dead_code)]
pub fn build_half_edge_mesh(indices: &[[u32;3]], vertex_count: usize) -> HalfEdgeMeshStructure {
    let mut half_edges = Vec::new();
    let mut edge_map: HashMap<(u32,u32), u32> = HashMap::new();
    for (fi, tri) in indices.iter().enumerate() {
        let base = half_edges.len() as u32;
        for k in 0..3u32 {
            half_edges.push(HalfEdgeMeshData {
                vertex: tri[k as usize],
                face: Some(fi as u32),
                next: base + (k + 1) % 3,
                twin: None,
            });
            let a = tri[k as usize]; let b = tri[((k+1)%3) as usize];
            edge_map.insert((a,b), base + k);
        }
    }
    // Link twins
    for i in 0..half_edges.len() {
        let v = half_edges[i].vertex;
        let nv = half_edges[half_edges[i].next as usize].vertex;
        if let Some(&twin_idx) = edge_map.get(&(nv, v)) {
            half_edges[i].twin = Some(twin_idx);
        }
    }
    HalfEdgeMeshStructure { half_edges, vertex_count, face_count: indices.len() }
}

#[allow(dead_code)]
pub fn half_edge_count_hem(mesh: &HalfEdgeMeshStructure) -> usize { mesh.half_edges.len() }
#[allow(dead_code)]
pub fn is_boundary_half_edge(mesh: &HalfEdgeMeshStructure, he: usize) -> bool { mesh.half_edges.get(he).is_some_and(|e| e.twin.is_none()) }
#[allow(dead_code)]
pub fn vertex_of_half_edge(mesh: &HalfEdgeMeshStructure, he: usize) -> u32 { mesh.half_edges.get(he).map_or(u32::MAX, |e| e.vertex) }
#[allow(dead_code)]
pub fn next_half_edge(mesh: &HalfEdgeMeshStructure, he: usize) -> u32 { mesh.half_edges.get(he).map_or(u32::MAX, |e| e.next) }
#[allow(dead_code)]
pub fn twin_half_edge(mesh: &HalfEdgeMeshStructure, he: usize) -> Option<u32> { mesh.half_edges.get(he).and_then(|e| e.twin) }
#[allow(dead_code)]
pub fn boundary_edge_count_hem(mesh: &HalfEdgeMeshStructure) -> usize { mesh.half_edges.iter().filter(|e| e.twin.is_none()).count() }
#[allow(dead_code)]
pub fn half_edge_mesh_to_json(mesh: &HalfEdgeMeshStructure) -> String {
    format!("{{\"half_edges\":{},\"vertices\":{},\"faces\":{}}}", mesh.half_edges.len(), mesh.vertex_count, mesh.face_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tri_mesh() -> (Vec<[u32;3]>, usize) { (vec![[0,1,2]], 3) }
    fn quad_mesh() -> (Vec<[u32;3]>, usize) { (vec![[0,1,2],[0,2,3]], 4) }
    #[test] fn test_build() { let(i,n)=tri_mesh(); let m=build_half_edge_mesh(&i,n); assert_eq!(m.half_edges.len(),3); }
    #[test] fn test_half_edge_count() { let(i,n)=tri_mesh(); let m=build_half_edge_mesh(&i,n); assert_eq!(half_edge_count_hem(&m),3); }
    #[test] fn test_boundary() { let(i,n)=tri_mesh(); let m=build_half_edge_mesh(&i,n); assert!(boundary_edge_count_hem(&m)>0); }
    #[test] fn test_is_boundary() { let(i,n)=tri_mesh(); let m=build_half_edge_mesh(&i,n); assert!(is_boundary_half_edge(&m,0)); }
    #[test] fn test_vertex_of() { let(i,n)=tri_mesh(); let m=build_half_edge_mesh(&i,n); assert_eq!(vertex_of_half_edge(&m,0),0); }
    #[test] fn test_next() { let(i,n)=tri_mesh(); let m=build_half_edge_mesh(&i,n); assert_eq!(next_half_edge(&m,0),1); }
    #[test] fn test_twin_shared_edge() { let(i,n)=quad_mesh(); let m=build_half_edge_mesh(&i,n); let has_twin=m.half_edges.iter().any(|e| e.twin.is_some()); assert!(has_twin); }
    #[test] fn test_to_json() { let(i,n)=tri_mesh(); let m=build_half_edge_mesh(&i,n); assert!(half_edge_mesh_to_json(&m).contains("half_edges")); }
    #[test] fn test_empty() { let m=build_half_edge_mesh(&[],0); assert!(m.half_edges.is_empty()); }
    #[test] fn test_face_count() { let(i,n)=quad_mesh(); let m=build_half_edge_mesh(&i,n); assert_eq!(m.face_count,2); }
}
