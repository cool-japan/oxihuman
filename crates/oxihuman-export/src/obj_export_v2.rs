// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]
//! Wavefront OBJ export v2 with MTL support.

#[allow(dead_code)]
pub struct ObjExportV2 {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub faces: Vec<Vec<[u32; 3]>>,
    pub material_name: Option<String>,
}

#[allow(dead_code)]
pub fn new_obj_export_v2() -> ObjExportV2 {
    ObjExportV2 {
        vertices: Vec::new(),
        normals: Vec::new(),
        uvs: Vec::new(),
        faces: Vec::new(),
        material_name: None,
    }
}

#[allow(dead_code)]
pub fn obj2_add_vertex(e: &mut ObjExportV2, pos: [f32; 3]) {
    e.vertices.push(pos);
}

#[allow(dead_code)]
pub fn obj2_add_normal(e: &mut ObjExportV2, n: [f32; 3]) {
    e.normals.push(n);
}

#[allow(dead_code)]
pub fn obj2_add_uv(e: &mut ObjExportV2, uv: [f32; 2]) {
    e.uvs.push(uv);
}

#[allow(dead_code)]
pub fn obj2_add_face(e: &mut ObjExportV2, verts: Vec<[u32; 3]>) {
    e.faces.push(verts);
}

#[allow(dead_code)]
pub fn obj2_vertex_count(e: &ObjExportV2) -> usize {
    e.vertices.len()
}

#[allow(dead_code)]
pub fn obj2_to_string(e: &ObjExportV2) -> String {
    let mut out = String::from("# OBJ exported by oxihuman\n");
    if let Some(ref mtl) = e.material_name {
        out.push_str(&format!("mtllib {}.mtl\n", mtl));
    }
    for v in &e.vertices {
        out.push_str(&format!("v {} {} {}\n", v[0], v[1], v[2]));
    }
    for uv in &e.uvs {
        out.push_str(&format!("vt {} {}\n", uv[0], uv[1]));
    }
    for n in &e.normals {
        out.push_str(&format!("vn {} {} {}\n", n[0], n[1], n[2]));
    }
    for face in &e.faces {
        let face_str: Vec<String> = face.iter().map(|vtx| {
            format!("{}/{}/{}", vtx[0] + 1, vtx[1] + 1, vtx[2] + 1)
        }).collect();
        out.push_str(&format!("f {}\n", face_str.join(" ")));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let e = new_obj_export_v2();
        assert_eq!(obj2_vertex_count(&e), 0);
    }

    #[test]
    fn test_add_vertex() {
        let mut e = new_obj_export_v2();
        obj2_add_vertex(&mut e, [1.0, 2.0, 3.0]);
        assert_eq!(obj2_vertex_count(&e), 1);
    }

    #[test]
    fn test_add_normal() {
        let mut e = new_obj_export_v2();
        obj2_add_normal(&mut e, [0.0, 1.0, 0.0]);
        assert_eq!(e.normals.len(), 1);
    }

    #[test]
    fn test_add_uv() {
        let mut e = new_obj_export_v2();
        obj2_add_uv(&mut e, [0.5, 0.5]);
        assert_eq!(e.uvs.len(), 1);
    }

    #[test]
    fn test_add_face() {
        let mut e = new_obj_export_v2();
        obj2_add_face(&mut e, vec![[0u32, 0, 0], [1, 0, 0], [2, 0, 0]]);
        assert_eq!(e.faces.len(), 1);
    }

    #[test]
    fn test_to_string_contains_v() {
        let mut e = new_obj_export_v2();
        obj2_add_vertex(&mut e, [1.0, 2.0, 3.0]);
        let s = obj2_to_string(&e);
        assert!(s.contains("v "));
    }

    #[test]
    fn test_vertex_count_multiple() {
        let mut e = new_obj_export_v2();
        for _ in 0..5 {
            obj2_add_vertex(&mut e, [0.0, 0.0, 0.0]);
        }
        assert_eq!(obj2_vertex_count(&e), 5);
    }

    #[test]
    fn test_to_string_face_line() {
        let mut e = new_obj_export_v2();
        obj2_add_face(&mut e, vec![[0u32, 0, 0], [1, 0, 0], [2, 0, 0]]);
        let s = obj2_to_string(&e);
        assert!(s.contains("f "));
    }
}
