// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Extended Wavefront OBJ export with material library (.mtl) support.

/// Material definition for .mtl export.
#[derive(Clone, Debug)]
pub struct DotObjMaterial {
    pub name: String,
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub shininess: f32,
    pub opacity: f32,
    pub diffuse_map: Option<String>,
}

impl Default for DotObjMaterial {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            diffuse: [0.8, 0.8, 0.8],
            specular: [0.5, 0.5, 0.5],
            shininess: 32.0,
            opacity: 1.0,
            diffuse_map: None,
        }
    }
}

/// Extended OBJ export container.
#[derive(Clone, Debug, Default)]
pub struct DotObjExport {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub materials: Vec<DotObjMaterial>,
    pub object_name: String,
    pub mtl_lib_name: String,
}

/// Create a new DotObj export.
pub fn new_dotobj_export(name: &str) -> DotObjExport {
    DotObjExport {
        object_name: name.to_string(),
        mtl_lib_name: format!("{}.mtl", name),
        ..Default::default()
    }
}

/// Set mesh geometry.
pub fn dotobj_set_mesh(
    doc: &mut DotObjExport,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
) {
    doc.positions = positions;
    doc.normals = normals;
    doc.uvs = uvs;
    doc.indices = indices;
}

/// Add a material.
pub fn dotobj_add_material(doc: &mut DotObjExport, mat: DotObjMaterial) {
    doc.materials.push(mat);
}

/// Return the material count.
pub fn dotobj_material_count(doc: &DotObjExport) -> usize {
    doc.materials.len()
}

/// Return the face count.
pub fn dotobj_face_count(doc: &DotObjExport) -> usize {
    doc.indices.len() / 3
}

/// Render the .obj text content.
pub fn render_dotobj(doc: &DotObjExport) -> String {
    let mut out = String::from("# oxihuman dotobj export\n");
    if !doc.mtl_lib_name.is_empty() {
        out.push_str(&format!("mtllib {}\n", doc.mtl_lib_name));
    }
    if !doc.object_name.is_empty() {
        out.push_str(&format!("o {}\n", doc.object_name));
    }
    for p in &doc.positions {
        out.push_str(&format!("v {:.6} {:.6} {:.6}\n", p[0], p[1], p[2]));
    }
    for uv in &doc.uvs {
        out.push_str(&format!("vt {:.6} {:.6}\n", uv[0], uv[1]));
    }
    for n in &doc.normals {
        out.push_str(&format!("vn {:.6} {:.6} {:.6}\n", n[0], n[1], n[2]));
    }
    if !doc.materials.is_empty() {
        out.push_str(&format!("usemtl {}\n", doc.materials[0].name));
    }
    let tri_count = doc.indices.len() / 3;
    let has_uv = !doc.uvs.is_empty();
    let has_n = !doc.normals.is_empty();
    for t in 0..tri_count {
        let mut face = String::from("f");
        for k in 0..3 {
            let vi = doc.indices[t * 3 + k] + 1; // OBJ is 1-indexed
            face.push(' ');
            if has_uv && has_n {
                face.push_str(&format!("{}/{}/{}", vi, vi, vi));
            } else if has_uv {
                face.push_str(&format!("{}/{}", vi, vi));
            } else if has_n {
                face.push_str(&format!("{}//{}", vi, vi));
            } else {
                face.push_str(&vi.to_string());
            }
        }
        out.push_str(&face);
        out.push('\n');
    }
    out
}

/// Render the .mtl text content.
pub fn render_dotmtl(doc: &DotObjExport) -> String {
    let mut out = String::from("# oxihuman mtl export\n");
    for mat in &doc.materials {
        out.push_str(&format!("newmtl {}\n", mat.name));
        out.push_str(&format!(
            "Kd {:.4} {:.4} {:.4}\n",
            mat.diffuse[0], mat.diffuse[1], mat.diffuse[2]
        ));
        out.push_str(&format!(
            "Ks {:.4} {:.4} {:.4}\n",
            mat.specular[0], mat.specular[1], mat.specular[2]
        ));
        out.push_str(&format!("Ns {:.4}\n", mat.shininess));
        out.push_str(&format!("d {:.4}\n", mat.opacity));
        if let Some(ref map) = mat.diffuse_map {
            out.push_str(&format!("map_Kd {}\n", map));
        }
    }
    out
}

/// Validate the export (positions non-empty, indices valid).
pub fn validate_dotobj(doc: &DotObjExport) -> bool {
    if doc.positions.is_empty() {
        return false;
    }
    let n = doc.positions.len() as u32;
    doc.indices.iter().all(|&i| i < n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_doc() -> DotObjExport {
        let mut d = new_dotobj_export("test");
        dotobj_set_mesh(
            &mut d,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![],
            vec![],
            vec![0, 1, 2],
        );
        d
    }

    #[test]
    fn new_export_empty() {
        let d = new_dotobj_export("test");
        assert_eq!(dotobj_face_count(&d), 0);
    }

    #[test]
    fn face_count_after_set() {
        let d = simple_doc();
        assert_eq!(dotobj_face_count(&d), 1);
    }

    #[test]
    fn add_material_increments() {
        let mut d = simple_doc();
        dotobj_add_material(&mut d, DotObjMaterial::default());
        assert_eq!(dotobj_material_count(&d), 1);
    }

    #[test]
    fn render_obj_contains_v() {
        let d = simple_doc();
        let s = render_dotobj(&d);
        assert!(s.contains("\nv "));
    }

    #[test]
    fn render_obj_contains_face() {
        let d = simple_doc();
        let s = render_dotobj(&d);
        assert!(s.contains("\nf "));
    }

    #[test]
    fn render_mtl_contains_newmtl() {
        let mut d = simple_doc();
        dotobj_add_material(
            &mut d,
            DotObjMaterial {
                name: "mat1".to_string(),
                ..Default::default()
            },
        );
        let s = render_dotmtl(&d);
        assert!(s.contains("newmtl mat1"));
    }

    #[test]
    fn validate_valid_doc() {
        let d = simple_doc();
        assert!(validate_dotobj(&d));
    }

    #[test]
    fn validate_empty_fails() {
        let d = new_dotobj_export("test");
        assert!(!validate_dotobj(&d));
    }
}
