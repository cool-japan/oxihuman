// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! B-Rep topology export stub.

/// A B-Rep vertex.
#[derive(Debug, Clone)]
pub struct BRepVertex {
    pub id: u32,
    pub position: [f64; 3],
}

/// A B-Rep edge (connects two vertex ids).
#[derive(Debug, Clone)]
pub struct BRepEdge {
    pub id: u32,
    pub v0: u32,
    pub v1: u32,
}

/// A B-Rep face (a list of edge ids forming the boundary loop).
#[derive(Debug, Clone)]
pub struct BRepFace {
    pub id: u32,
    pub edges: Vec<u32>,
    pub normal: [f64; 3],
}

/// B-Rep topology export.
#[derive(Debug, Clone, Default)]
pub struct BRepExport {
    pub vertices: Vec<BRepVertex>,
    pub edges: Vec<BRepEdge>,
    pub faces: Vec<BRepFace>,
}

/// Create an empty B-Rep export.
pub fn new_brep_export() -> BRepExport {
    BRepExport::default()
}

/// Add a vertex; returns its id.
pub fn add_brep_vertex(export: &mut BRepExport, position: [f64; 3]) -> u32 {
    let id = export.vertices.len() as u32;
    export.vertices.push(BRepVertex { id, position });
    id
}

/// Add an edge; returns its id.
pub fn add_brep_edge(export: &mut BRepExport, v0: u32, v1: u32) -> u32 {
    let id = export.edges.len() as u32;
    export.edges.push(BRepEdge { id, v0, v1 });
    id
}

/// Add a face; returns its id.
pub fn add_brep_face(export: &mut BRepExport, edges: Vec<u32>, normal: [f64; 3]) -> u32 {
    let id = export.faces.len() as u32;
    export.faces.push(BRepFace { id, edges, normal });
    id
}

/// Validate that all edge vertex references exist.
pub fn validate_brep(export: &BRepExport) -> bool {
    let nv = export.vertices.len() as u32;
    export.edges.iter().all(|e| e.v0 < nv && e.v1 < nv)
        && export.faces.iter().all(|f| {
            let ne = export.edges.len() as u32;
            f.edges.iter().all(|&eid| eid < ne)
        })
}

/// Compute the Euler characteristic V - E + F.
pub fn euler_characteristic(export: &BRepExport) -> i32 {
    export.vertices.len() as i32 - export.edges.len() as i32 + export.faces.len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_quad() -> BRepExport {
        let mut exp = new_brep_export();
        let v0 = add_brep_vertex(&mut exp, [0.0, 0.0, 0.0]);
        let v1 = add_brep_vertex(&mut exp, [1.0, 0.0, 0.0]);
        let v2 = add_brep_vertex(&mut exp, [1.0, 1.0, 0.0]);
        let v3 = add_brep_vertex(&mut exp, [0.0, 1.0, 0.0]);
        let e0 = add_brep_edge(&mut exp, v0, v1);
        let e1 = add_brep_edge(&mut exp, v1, v2);
        let e2 = add_brep_edge(&mut exp, v2, v3);
        let e3 = add_brep_edge(&mut exp, v3, v0);
        add_brep_face(&mut exp, vec![e0, e1, e2, e3], [0.0, 0.0, 1.0]);
        exp
    }

    #[test]
    fn test_vertex_count() {
        let exp = simple_quad();
        assert_eq!(exp.vertices.len(), 4);
    }

    #[test]
    fn test_edge_count() {
        let exp = simple_quad();
        assert_eq!(exp.edges.len(), 4);
    }

    #[test]
    fn test_face_count() {
        let exp = simple_quad();
        assert_eq!(exp.faces.len(), 1);
    }

    #[test]
    fn test_validate_simple_quad() {
        assert!(validate_brep(&simple_quad()));
    }

    #[test]
    fn test_euler_characteristic_quad() {
        /* V=4, E=4, F=1 → χ = 1 */
        let exp = simple_quad();
        assert_eq!(euler_characteristic(&exp), 1);
    }

    #[test]
    fn test_add_vertex_ids_sequential() {
        let mut exp = new_brep_export();
        let a = add_brep_vertex(&mut exp, [0.0; 3]);
        let b = add_brep_vertex(&mut exp, [1.0; 3]);
        assert_eq!(a, 0);
        assert_eq!(b, 1);
    }

    #[test]
    fn test_validate_empty() {
        assert!(validate_brep(&new_brep_export()));
    }

    #[test]
    fn test_add_edge_ids_sequential() {
        let mut exp = new_brep_export();
        add_brep_vertex(&mut exp, [0.0; 3]);
        add_brep_vertex(&mut exp, [1.0; 3]);
        let e0 = add_brep_edge(&mut exp, 0, 1);
        let e1 = add_brep_edge(&mut exp, 1, 0);
        assert_eq!(e0, 0);
        assert_eq!(e1, 1);
    }

    #[test]
    fn test_face_normal_stored() {
        let mut exp = new_brep_export();
        let v0 = add_brep_vertex(&mut exp, [0.0; 3]);
        let v1 = add_brep_vertex(&mut exp, [1.0; 3]);
        let e0 = add_brep_edge(&mut exp, v0, v1);
        let normal = [0.0, 1.0, 0.0];
        add_brep_face(&mut exp, vec![e0], normal);
        assert_eq!(exp.faces[0].normal, normal);
    }
}
