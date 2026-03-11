// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! AC3D 3D model format export.

/// An AC3D surface.
#[derive(Clone, Debug)]
pub struct Ac3dSurface {
    pub flags: u32,
    pub vertices: Vec<(u32, f32, f32)>, // (index, u, v)
}

/// An AC3D material.
#[derive(Clone, Debug)]
pub struct Ac3dMaterial {
    pub name: String,
    pub rgb: [f32; 3],
    pub amb: [f32; 3],
    pub emis: [f32; 3],
    pub spec: [f32; 3],
    pub shi: u32,
    pub trans: f32,
}

impl Default for Ac3dMaterial {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            rgb: [0.8, 0.8, 0.8],
            amb: [0.2, 0.2, 0.2],
            emis: [0.0, 0.0, 0.0],
            spec: [0.5, 0.5, 0.5],
            shi: 10,
            trans: 0.0,
        }
    }
}

/// An AC3D object.
#[derive(Clone, Debug)]
pub struct Ac3dObject {
    pub name: String,
    pub positions: Vec<[f32; 3]>,
    pub surfaces: Vec<Ac3dSurface>,
    pub mat_index: u32,
}

/// An AC3D document.
#[derive(Clone, Debug, Default)]
pub struct Ac3dExport {
    pub materials: Vec<Ac3dMaterial>,
    pub objects: Vec<Ac3dObject>,
}

/// Create a new AC3D export document.
pub fn new_ac3d_export() -> Ac3dExport {
    Ac3dExport::default()
}

/// Add a material and return its index.
pub fn ac3d_add_material(doc: &mut Ac3dExport, mat: Ac3dMaterial) -> u32 {
    let idx = doc.materials.len() as u32;
    doc.materials.push(mat);
    idx
}

/// Add an object from mesh geometry.
pub fn ac3d_add_mesh(
    doc: &mut Ac3dExport,
    name: &str,
    positions: Vec<[f32; 3]>,
    indices: &[u32],
    mat_index: u32,
) {
    let surfaces: Vec<Ac3dSurface> = indices
        .chunks(3)
        .map(|t| Ac3dSurface {
            flags: 0,
            vertices: t.iter().map(|&i| (i, 0.0, 0.0)).collect(),
        })
        .collect();
    doc.objects.push(Ac3dObject {
        name: name.to_string(),
        positions,
        surfaces,
        mat_index,
    });
}

/// Return the material count.
pub fn ac3d_material_count(doc: &Ac3dExport) -> usize {
    doc.materials.len()
}

/// Return the object count.
pub fn ac3d_object_count(doc: &Ac3dExport) -> usize {
    doc.objects.len()
}

/// Render the AC3D file.
pub fn render_ac3d(doc: &Ac3dExport) -> String {
    let mut out = String::from("AC3Db\n");
    for mat in &doc.materials {
        out.push_str(&format!(
            "MATERIAL \"{}\" rgb {:.4} {:.4} {:.4}  amb {:.4} {:.4} {:.4}  emis {:.4} {:.4} {:.4}  spec {:.4} {:.4} {:.4}  shi {}  trans {:.4}\n",
            mat.name,
            mat.rgb[0], mat.rgb[1], mat.rgb[2],
            mat.amb[0], mat.amb[1], mat.amb[2],
            mat.emis[0], mat.emis[1], mat.emis[2],
            mat.spec[0], mat.spec[1], mat.spec[2],
            mat.shi, mat.trans
        ));
    }
    out.push_str("OBJECT world\nkids ");
    out.push_str(&format!("{}\n", doc.objects.len()));
    for obj in &doc.objects {
        out.push_str("OBJECT poly\n");
        out.push_str(&format!("name \"{}\"\n", obj.name));
        out.push_str(&format!("numvert {}\n", obj.positions.len()));
        for p in &obj.positions {
            out.push_str(&format!("{:.6} {:.6} {:.6}\n", p[0], p[1], p[2]));
        }
        out.push_str(&format!("numsurf {}\n", obj.surfaces.len()));
        for surf in &obj.surfaces {
            out.push_str(&format!("SURF 0x{:02X}\n", surf.flags));
            out.push_str(&format!("mat {}\n", obj.mat_index));
            out.push_str(&format!("refs {}\n", surf.vertices.len()));
            for &(idx, u, v) in &surf.vertices {
                out.push_str(&format!("{} {:.4} {:.4}\n", idx, u, v));
            }
        }
        out.push_str("kids 0\n");
    }
    out
}

/// Estimate the file size.
pub fn ac3d_size_estimate(doc: &Ac3dExport) -> usize {
    render_ac3d(doc).len()
}

/// Validate the document.
pub fn validate_ac3d(doc: &Ac3dExport) -> bool {
    for obj in &doc.objects {
        let n = obj.positions.len() as u32;
        for surf in &obj.surfaces {
            for &(idx, _, _) in &surf.vertices {
                if idx >= n {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_doc() -> Ac3dExport {
        let mut d = new_ac3d_export();
        let mat_idx = ac3d_add_material(&mut d, Ac3dMaterial::default());
        ac3d_add_mesh(
            &mut d,
            "tri",
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[0, 1, 2],
            mat_idx,
        );
        d
    }

    #[test]
    fn material_count() {
        let d = simple_doc();
        assert_eq!(ac3d_material_count(&d), 1);
    }

    #[test]
    fn object_count() {
        let d = simple_doc();
        assert_eq!(ac3d_object_count(&d), 1);
    }

    #[test]
    fn render_starts_with_ac3d() {
        let d = simple_doc();
        assert!(render_ac3d(&d).starts_with("AC3Db\n"));
    }

    #[test]
    fn render_contains_material() {
        let d = simple_doc();
        assert!(render_ac3d(&d).contains("MATERIAL"));
    }

    #[test]
    fn render_contains_numvert() {
        let d = simple_doc();
        assert!(render_ac3d(&d).contains("numvert 3"));
    }

    #[test]
    fn validate_valid() {
        let d = simple_doc();
        assert!(validate_ac3d(&d));
    }

    #[test]
    fn size_estimate_positive() {
        let d = simple_doc();
        assert!(ac3d_size_estimate(&d) > 0);
    }

    #[test]
    fn empty_doc_valid() {
        let d = new_ac3d_export();
        assert!(validate_ac3d(&d));
    }
}
