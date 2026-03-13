// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan) / SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Cell complex representation for volumetric meshes.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CellComplex {
    pub vertices: Vec<[f32; 3]>,
    pub edges: Vec<[u32; 2]>,
    pub faces: Vec<Vec<u32>>,
    pub cells: Vec<Vec<usize>>,
}

#[allow(dead_code)]
pub fn new_cell_complex() -> CellComplex {
    CellComplex { vertices: Vec::new(), edges: Vec::new(), faces: Vec::new(), cells: Vec::new() }
}

#[allow(dead_code)]
pub fn cell_add_vertex(cx: &mut CellComplex, v: [f32; 3]) -> u32 {
    cx.vertices.push(v);
    (cx.vertices.len() - 1) as u32
}

#[allow(dead_code)]
pub fn cell_add_edge(cx: &mut CellComplex, a: u32, b: u32) -> usize {
    cx.edges.push([a, b]);
    cx.edges.len() - 1
}

#[allow(dead_code)]
pub fn cell_add_face(cx: &mut CellComplex, verts: Vec<u32>) -> usize {
    cx.faces.push(verts);
    cx.faces.len() - 1
}

#[allow(dead_code)]
pub fn cell_add_cell(cx: &mut CellComplex, face_indices: Vec<usize>) -> usize {
    cx.cells.push(face_indices);
    cx.cells.len() - 1
}

#[allow(dead_code)]
pub fn cell_vertex_count(cx: &CellComplex) -> usize { cx.vertices.len() }

#[allow(dead_code)]
pub fn cell_edge_count(cx: &CellComplex) -> usize { cx.edges.len() }

#[allow(dead_code)]
pub fn cell_face_count(cx: &CellComplex) -> usize { cx.faces.len() }

#[allow(dead_code)]
pub fn cell_count(cx: &CellComplex) -> usize { cx.cells.len() }

#[allow(dead_code)]
pub fn cell_euler_characteristic(cx: &CellComplex) -> i64 {
    cx.vertices.len() as i64 - cx.edges.len() as i64 + cx.faces.len() as i64 - cx.cells.len() as i64
}

#[allow(dead_code)]
pub fn cell_complex_to_json(cx: &CellComplex) -> String {
    format!("{{\"V\":{},\"E\":{},\"F\":{},\"C\":{},\"euler\":{}}}", cx.vertices.len(), cx.edges.len(), cx.faces.len(), cx.cells.len(), cell_euler_characteristic(cx))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tetra() -> CellComplex {
        let mut cx = new_cell_complex();
        cell_add_vertex(&mut cx, [0.0, 0.0, 0.0]);
        cell_add_vertex(&mut cx, [1.0, 0.0, 0.0]);
        cell_add_vertex(&mut cx, [0.0, 1.0, 0.0]);
        cell_add_vertex(&mut cx, [0.0, 0.0, 1.0]);
        cell_add_edge(&mut cx, 0, 1);
        cell_add_edge(&mut cx, 0, 2);
        cell_add_edge(&mut cx, 0, 3);
        cell_add_edge(&mut cx, 1, 2);
        cell_add_edge(&mut cx, 1, 3);
        cell_add_edge(&mut cx, 2, 3);
        cell_add_face(&mut cx, vec![0, 1, 2]);
        cell_add_face(&mut cx, vec![0, 1, 3]);
        cell_add_face(&mut cx, vec![0, 2, 3]);
        cell_add_face(&mut cx, vec![1, 2, 3]);
        cell_add_cell(&mut cx, vec![0, 1, 2, 3]);
        cx
    }

    #[test] fn test_new() { let cx = new_cell_complex(); assert_eq!(cell_vertex_count(&cx), 0); }
    #[test] fn test_add_vertex() { let mut cx = new_cell_complex(); cell_add_vertex(&mut cx, [1.0,2.0,3.0]); assert_eq!(cx.vertices.len(), 1); }
    #[test] fn test_vertex_count() { let cx = tetra(); assert_eq!(cell_vertex_count(&cx), 4); }
    #[test] fn test_edge_count() { let cx = tetra(); assert_eq!(cell_edge_count(&cx), 6); }
    #[test] fn test_face_count() { let cx = tetra(); assert_eq!(cell_face_count(&cx), 4); }
    #[test] fn test_cell_count() { let cx = tetra(); assert_eq!(cell_count(&cx), 1); }
    #[test] fn test_euler() { let cx = tetra(); assert_eq!(cell_euler_characteristic(&cx), 1); }
    #[test] fn test_to_json() { let cx = tetra(); assert!(cell_complex_to_json(&cx).contains("euler")); }
    #[test] fn test_add_edge() { let mut cx = new_cell_complex(); cell_add_vertex(&mut cx,[0.0,0.0,0.0]); cell_add_vertex(&mut cx,[1.0,0.0,0.0]); assert_eq!(cell_add_edge(&mut cx, 0, 1), 0); }
    #[test] fn test_add_face() { let mut cx = new_cell_complex(); assert_eq!(cell_add_face(&mut cx, vec![0,1,2]), 0); }
}
