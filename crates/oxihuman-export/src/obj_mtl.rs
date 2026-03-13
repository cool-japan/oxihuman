// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! OBJ + MTL material file export with PBR material properties.
//!
//! This module extends the basic `obj.rs` exporter with:
//! - Full MTL material definitions (Phong + PBR extensions)
//! - Preset materials (skin, cloth, metal)
//! - Richer OBJ export options (normals, UVs, object name, precision, V-flip)
//! - Validation and round-trip utilities

#![allow(dead_code)]

use anyhow::Result;
use oxihuman_mesh::MeshBuffers;
use std::fmt::Write as FmtWrite;
use std::path::Path;

// ---------------------------------------------------------------------------
// MtlMaterial
// ---------------------------------------------------------------------------

/// Material definition supporting both Phong shading and PBR extensions.
#[derive(Debug, Clone)]
pub struct MtlMaterial {
    /// Material name (used in `newmtl` and `usemtl` directives).
    pub name: String,
    /// Ambient colour `Ka`.
    pub ambient: [f32; 3],
    /// Diffuse colour `Kd`.
    pub diffuse: [f32; 3],
    /// Specular colour `Ks`.
    pub specular: [f32; 3],
    /// Emissive colour `Ke`.
    pub emissive: [f32; 3],
    /// Specular exponent `Ns`.
    pub shininess: f32,
    /// Opacity `d` (1.0 = fully opaque).
    pub opacity: f32,
    /// Index of refraction `Ni`.
    pub ior: f32,
    /// Illumination model.
    pub illum: u32,
    /// Diffuse texture map `map_Kd`.
    pub diffuse_map: Option<String>,
    /// Normal / bump map `map_Bump`.
    pub normal_map: Option<String>,
    /// Roughness map `map_Pr` (PBR extension).
    pub roughness_map: Option<String>,
    /// PBR roughness `Pr`.
    pub roughness: f32,
    /// PBR metallic `Pm`.
    pub metallic: f32,
}

impl Default for MtlMaterial {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            ambient: [0.1, 0.1, 0.1],
            diffuse: [0.8, 0.8, 0.8],
            specular: [0.0, 0.0, 0.0],
            emissive: [0.0, 0.0, 0.0],
            shininess: 10.0,
            opacity: 1.0,
            ior: 1.45,
            illum: 2,
            diffuse_map: None,
            normal_map: None,
            roughness_map: None,
            roughness: 0.5,
            metallic: 0.0,
        }
    }
}

impl MtlMaterial {
    /// Flesh-toned skin preset.
    pub fn skin() -> Self {
        Self {
            name: "skin".to_string(),
            ambient: [0.12, 0.08, 0.06],
            diffuse: [0.80, 0.55, 0.42],
            specular: [0.10, 0.07, 0.05],
            emissive: [0.0, 0.0, 0.0],
            shininess: 5.0,
            opacity: 1.0,
            ior: 1.4,
            illum: 2,
            diffuse_map: None,
            normal_map: None,
            roughness_map: None,
            roughness: 0.75,
            metallic: 0.0,
        }
    }

    /// Fabric / cloth preset.
    pub fn cloth() -> Self {
        Self {
            name: "cloth".to_string(),
            ambient: [0.05, 0.05, 0.08],
            diffuse: [0.30, 0.30, 0.45],
            specular: [0.02, 0.02, 0.02],
            emissive: [0.0, 0.0, 0.0],
            shininess: 2.0,
            opacity: 1.0,
            ior: 1.5,
            illum: 1,
            diffuse_map: None,
            normal_map: None,
            roughness_map: None,
            roughness: 0.90,
            metallic: 0.0,
        }
    }

    /// Metallic preset.
    pub fn metal() -> Self {
        Self {
            name: "metal".to_string(),
            ambient: [0.15, 0.15, 0.15],
            diffuse: [0.60, 0.60, 0.65],
            specular: [0.80, 0.80, 0.80],
            emissive: [0.0, 0.0, 0.0],
            shininess: 120.0,
            opacity: 1.0,
            ior: 2.5,
            illum: 3,
            diffuse_map: None,
            normal_map: None,
            roughness_map: None,
            roughness: 0.10,
            metallic: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// build_mtl / export_mtl
// ---------------------------------------------------------------------------

/// Build MTL file content as a UTF-8 string from a slice of materials.
pub fn build_mtl(materials: &[MtlMaterial]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# OxiHuman exported material library");
    let _ = writeln!(out, "# Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)");
    let _ = writeln!(out, "# Materials: {}", materials.len());
    let _ = writeln!(out);

    for mat in materials {
        let _ = writeln!(out, "newmtl {}", mat.name);
        let _ = writeln!(
            out,
            "Ka {:.6} {:.6} {:.6}",
            mat.ambient[0], mat.ambient[1], mat.ambient[2]
        );
        let _ = writeln!(
            out,
            "Kd {:.6} {:.6} {:.6}",
            mat.diffuse[0], mat.diffuse[1], mat.diffuse[2]
        );
        let _ = writeln!(
            out,
            "Ks {:.6} {:.6} {:.6}",
            mat.specular[0], mat.specular[1], mat.specular[2]
        );
        let _ = writeln!(
            out,
            "Ke {:.6} {:.6} {:.6}",
            mat.emissive[0], mat.emissive[1], mat.emissive[2]
        );
        let _ = writeln!(out, "Ns {:.6}", mat.shininess);
        let _ = writeln!(out, "d {:.6}", mat.opacity);
        let _ = writeln!(out, "Ni {:.6}", mat.ior);
        let _ = writeln!(out, "illum {}", mat.illum);
        // PBR extensions
        let _ = writeln!(out, "Pr {:.6}", mat.roughness);
        let _ = writeln!(out, "Pm {:.6}", mat.metallic);

        if let Some(ref path) = mat.diffuse_map {
            let _ = writeln!(out, "map_Kd {}", path);
        }
        if let Some(ref path) = mat.normal_map {
            let _ = writeln!(out, "map_Bump {}", path);
            let _ = writeln!(out, "norm {}", path);
        }
        if let Some(ref path) = mat.roughness_map {
            let _ = writeln!(out, "map_Pr {}", path);
        }
        let _ = writeln!(out);
    }

    out
}

/// Write MTL content to `path`.
pub fn export_mtl(materials: &[MtlMaterial], path: &Path) -> Result<()> {
    let content = build_mtl(materials);
    std::fs::write(path, content)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// ObjMtlOptions
// ---------------------------------------------------------------------------

/// Options for the richer OBJ exporter that includes an MTL reference.
#[derive(Debug, Clone)]
pub struct ObjMtlOptions {
    /// Filename of the accompanying MTL file (e.g. `"mesh.mtl"`).
    pub mtl_filename: String,
    /// Name of the material to activate (`usemtl`).
    pub material_name: String,
    /// Emit `vn` lines and include normal indices in face records.
    pub include_normals: bool,
    /// Emit `vt` lines and include UV indices in face records.
    pub include_uvs: bool,
    /// Value for the `o` (object name) line.
    pub object_name: String,
    /// Flip the V coordinate of UVs (some engines need 1-v).
    pub flip_v_uv: bool,
    /// Number of decimal places for floating-point values.
    pub precision: usize,
}

impl Default for ObjMtlOptions {
    fn default() -> Self {
        Self {
            mtl_filename: "mesh.mtl".to_string(),
            material_name: "default".to_string(),
            include_normals: true,
            include_uvs: true,
            object_name: "OxiHumanMesh".to_string(),
            flip_v_uv: false,
            precision: 6,
        }
    }
}

// ---------------------------------------------------------------------------
// ObjMtlStats
// ---------------------------------------------------------------------------

/// Statistics returned by `export_obj_mtl`.
#[derive(Debug, Clone)]
pub struct ObjMtlStats {
    pub vertex_count: usize,
    pub normal_count: usize,
    pub uv_count: usize,
    pub face_count: usize,
    pub obj_bytes: usize,
    pub mtl_bytes: usize,
}

// ---------------------------------------------------------------------------
// build_obj_with_mtl
// ---------------------------------------------------------------------------

/// Build an OBJ string that references an MTL file.
pub fn build_obj_with_mtl(mesh: &MeshBuffers, options: &ObjMtlOptions) -> String {
    let prec = options.precision;
    let mut out = String::new();

    let _ = writeln!(out, "# OxiHuman exported mesh (OBJ+MTL)");
    let _ = writeln!(out, "# Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)");
    let _ = writeln!(out, "# Vertices: {}", mesh.positions.len());
    let _ = writeln!(out, "# Faces: {}", mesh.indices.len() / 3);
    let _ = writeln!(out);
    let _ = writeln!(out, "mtllib {}", options.mtl_filename);
    let _ = writeln!(out, "o {}", options.object_name);
    let _ = writeln!(out);

    // Vertex positions
    for p in &mesh.positions {
        let _ = writeln!(
            out,
            "v {:.prec$} {:.prec$} {:.prec$}",
            p[0],
            p[1],
            p[2],
            prec = prec
        );
    }
    let _ = writeln!(out);

    // UV coordinates
    let emit_uvs = options.include_uvs && !mesh.uvs.is_empty();
    if emit_uvs {
        for uv in &mesh.uvs {
            let v = if options.flip_v_uv {
                1.0 - uv[1]
            } else {
                uv[1]
            };
            let _ = writeln!(out, "vt {:.prec$} {:.prec$}", uv[0], v, prec = prec);
        }
        let _ = writeln!(out);
    }

    // Vertex normals
    let emit_norms = options.include_normals && !mesh.normals.is_empty();
    if emit_norms {
        for n in &mesh.normals {
            let _ = writeln!(
                out,
                "vn {:.prec$} {:.prec$} {:.prec$}",
                n[0],
                n[1],
                n[2],
                prec = prec
            );
        }
        let _ = writeln!(out);
    }

    // usemtl + faces
    let _ = writeln!(out, "usemtl {}", options.material_name);
    let _ = writeln!(out, "s 1");

    for tri in mesh.indices.chunks_exact(3) {
        let (i0, i1, i2) = (tri[0] + 1, tri[1] + 1, tri[2] + 1); // 1-indexed
        let face = match (emit_uvs, emit_norms) {
            (true, true) => format!("f {0}/{0}/{0} {1}/{1}/{1} {2}/{2}/{2}", i0, i1, i2),
            (true, false) => format!("f {0}/{0} {1}/{1} {2}/{2}", i0, i1, i2),
            (false, true) => format!("f {0}//{0} {1}//{1} {2}//{2}", i0, i1, i2),
            (false, false) => format!("f {} {} {}", i0, i1, i2),
        };
        let _ = writeln!(out, "{}", face);
    }

    out
}

// ---------------------------------------------------------------------------
// export_obj_mtl
// ---------------------------------------------------------------------------

/// Export OBJ + MTL as a pair of files to the same directory.
///
/// The MTL file is written alongside the OBJ file using `options.mtl_filename`
/// as the file name (the directory component is taken from `obj_path`).
#[allow(clippy::too_many_arguments)]
pub fn export_obj_mtl(
    mesh: &MeshBuffers,
    obj_path: &Path,
    materials: &[MtlMaterial],
    options: &ObjMtlOptions,
) -> Result<ObjMtlStats> {
    let obj_content = build_obj_with_mtl(mesh, options);
    let mtl_content = build_mtl(materials);

    // Derive MTL path: same directory as OBJ, with options.mtl_filename
    let mtl_path = obj_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&options.mtl_filename);

    std::fs::write(obj_path, &obj_content)?;
    std::fs::write(&mtl_path, &mtl_content)?;

    let stats = ObjMtlStats {
        vertex_count: mesh.positions.len(),
        normal_count: if options.include_normals {
            mesh.normals.len()
        } else {
            0
        },
        uv_count: if options.include_uvs {
            mesh.uvs.len()
        } else {
            0
        },
        face_count: mesh.indices.len() / 3,
        obj_bytes: obj_content.len(),
        mtl_bytes: mtl_content.len(),
    };

    Ok(stats)
}

// ---------------------------------------------------------------------------
// validate_obj
// ---------------------------------------------------------------------------

/// Validate an OBJ string: checks that face indices do not exceed the declared
/// vertex / UV / normal counts, and that every face token is parseable.
pub fn validate_obj(content: &str) -> Result<(), String> {
    let mut v_count: usize = 0;
    let mut vt_count: usize = 0;
    let mut vn_count: usize = 0;

    for (lineno, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut tokens = line.splitn(2, ' ');
        match tokens.next().unwrap_or("") {
            "v" => v_count += 1,
            "vt" => vt_count += 1,
            "vn" => vn_count += 1,
            "f" => {
                let rest = tokens.next().unwrap_or("").trim();
                for vtx in rest.split_whitespace() {
                    // vtx can be: v, v/vt, v//vn, v/vt/vn
                    let parts: Vec<&str> = vtx.split('/').collect();
                    if parts.is_empty() {
                        return Err(format!("line {}: empty face vertex", lineno + 1));
                    }
                    // Parse position index
                    let vi: isize = parts[0].parse().map_err(|_| {
                        format!("line {}: invalid vertex index '{}'", lineno + 1, parts[0])
                    })?;
                    let vi_abs = if vi < 0 {
                        // relative index
                        (v_count as isize + vi) as usize
                    } else {
                        (vi - 1) as usize
                    };
                    if vi_abs >= v_count {
                        return Err(format!(
                            "line {}: vertex index {} out of range (have {})",
                            lineno + 1,
                            vi,
                            v_count
                        ));
                    }
                    // Parse optional UV index
                    if parts.len() >= 2 && !parts[1].is_empty() {
                        let vti: isize = parts[1].parse().map_err(|_| {
                            format!("line {}: invalid UV index '{}'", lineno + 1, parts[1])
                        })?;
                        let vti_abs = if vti < 0 {
                            (vt_count as isize + vti) as usize
                        } else {
                            (vti - 1) as usize
                        };
                        if vti_abs >= vt_count {
                            return Err(format!(
                                "line {}: UV index {} out of range (have {})",
                                lineno + 1,
                                vti,
                                vt_count
                            ));
                        }
                    }
                    // Parse optional normal index
                    if parts.len() >= 3 && !parts[2].is_empty() {
                        let vni: isize = parts[2].parse().map_err(|_| {
                            format!("line {}: invalid normal index '{}'", lineno + 1, parts[2])
                        })?;
                        let vni_abs = if vni < 0 {
                            (vn_count as isize + vni) as usize
                        } else {
                            (vni - 1) as usize
                        };
                        if vni_abs >= vn_count {
                            return Err(format!(
                                "line {}: normal index {} out of range (have {})",
                                lineno + 1,
                                vni,
                                vn_count
                            ));
                        }
                    }
                }
            }
            _ => {} // ignore unknown directives
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// parse_mtl_names
// ---------------------------------------------------------------------------

/// Parse material names from MTL content (lines beginning with `newmtl`).
pub fn parse_mtl_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("newmtl") {
                let name = rest.trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
            None
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;
    use oxihuman_morph::engine::MeshBuffers as MB;

    // ---- helpers -----------------------------------------------------------

    fn triangle_mesh() -> MeshBuffers {
        MeshBuffers::from_morph(MB {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            has_suit: false,
        })
    }

    fn quad_mesh() -> MeshBuffers {
        // Two triangles forming a unit square
        MeshBuffers::from_morph(MB {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![[0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            has_suit: false,
        })
    }

    // ---- MtlMaterial defaults ----------------------------------------------

    #[test]
    fn mtl_material_default_values() {
        let m = MtlMaterial::default();
        assert_eq!(m.name, "default");
        assert!((m.opacity - 1.0).abs() < 1e-6);
        assert!((m.shininess - 10.0).abs() < 1e-6);
        assert!((m.roughness - 0.5).abs() < 1e-6);
        assert!((m.metallic - 0.0).abs() < 1e-6);
        assert!((m.ior - 1.45).abs() < 1e-5);
        assert_eq!(m.illum, 2);
    }

    // ---- Preset materials --------------------------------------------------

    #[test]
    fn skin_preset_name_and_values() {
        let s = MtlMaterial::skin();
        assert_eq!(s.name, "skin");
        assert!((s.metallic - 0.0).abs() < 1e-6);
        assert!(s.roughness > 0.5, "skin should be rough");
    }

    #[test]
    fn cloth_preset_low_metallic() {
        let c = MtlMaterial::cloth();
        assert_eq!(c.name, "cloth");
        assert!((c.metallic - 0.0).abs() < 1e-6);
        assert!(c.roughness > 0.8, "cloth should be very rough");
        assert_eq!(c.illum, 1);
    }

    #[test]
    fn metal_preset_high_metallic() {
        let m = MtlMaterial::metal();
        assert_eq!(m.name, "metal");
        assert!((m.metallic - 1.0).abs() < 1e-6);
        assert!(m.roughness < 0.2, "metal should be shiny");
        assert_eq!(m.illum, 3);
    }

    // ---- build_mtl ---------------------------------------------------------

    #[test]
    fn build_mtl_contains_newmtl() {
        let mats = vec![MtlMaterial::skin(), MtlMaterial::cloth()];
        let s = build_mtl(&mats);
        assert!(s.contains("newmtl skin"), "should contain 'newmtl skin'");
        assert!(s.contains("newmtl cloth"), "should contain 'newmtl cloth'");
    }

    #[test]
    fn build_mtl_contains_pbr_lines() {
        let mats = vec![MtlMaterial::metal()];
        let s = build_mtl(&mats);
        assert!(s.contains("Pr "), "should contain Pr (roughness)");
        assert!(s.contains("Pm "), "should contain Pm (metallic)");
    }

    #[test]
    fn build_mtl_with_texture_maps() {
        let mut mat = MtlMaterial::skin();
        mat.diffuse_map = Some("skin_color.png".to_string());
        mat.normal_map = Some("skin_normal.png".to_string());
        mat.roughness_map = Some("skin_roughness.png".to_string());
        let s = build_mtl(&[mat]);
        assert!(s.contains("map_Kd skin_color.png"));
        assert!(s.contains("map_Bump skin_normal.png"));
        assert!(s.contains("norm skin_normal.png"));
        assert!(s.contains("map_Pr skin_roughness.png"));
    }

    // ---- export_mtl --------------------------------------------------------

    #[test]
    fn export_mtl_creates_file() {
        let mats = vec![MtlMaterial::default(), MtlMaterial::skin()];
        let path = std::path::PathBuf::from("/tmp/test_oxihuman.mtl");
        export_mtl(&mats, &path).expect("should succeed");
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).expect("should succeed");
        assert!(content.contains("newmtl default"));
        assert!(content.contains("newmtl skin"));
        std::fs::remove_file(&path).ok();
    }

    // ---- build_obj_with_mtl ------------------------------------------------

    #[test]
    fn obj_with_mtl_has_mtllib_line() {
        let mesh = triangle_mesh();
        let opts = ObjMtlOptions {
            mtl_filename: "human.mtl".to_string(),
            ..Default::default()
        };
        let s = build_obj_with_mtl(&mesh, &opts);
        assert!(
            s.contains("mtllib human.mtl"),
            "should have mtllib directive"
        );
    }

    #[test]
    fn obj_with_mtl_has_usemtl_line() {
        let mesh = triangle_mesh();
        let opts = ObjMtlOptions {
            material_name: "skin".to_string(),
            ..Default::default()
        };
        let s = build_obj_with_mtl(&mesh, &opts);
        assert!(s.contains("usemtl skin"), "should have usemtl directive");
    }

    #[test]
    fn obj_with_mtl_object_name() {
        let mesh = triangle_mesh();
        let opts = ObjMtlOptions {
            object_name: "MyHuman".to_string(),
            ..Default::default()
        };
        let s = build_obj_with_mtl(&mesh, &opts);
        assert!(s.contains("o MyHuman"));
    }

    #[test]
    fn obj_with_mtl_flip_v_uv() {
        let mesh = triangle_mesh();
        // mesh has uv [0,0], [1,0], [0,1]
        let opts_flip = ObjMtlOptions {
            flip_v_uv: true,
            ..Default::default()
        };
        let s_flip = build_obj_with_mtl(&mesh, &opts_flip);
        // V coord of [0,1] should become 1 - 1 = 0
        // V coord of [1,0] should become 1 - 0 = 1
        assert!(
            s_flip.contains("vt 1.000000 1.000000"),
            "flipped V of (1,0) => (1,1)"
        );

        let opts_no_flip = ObjMtlOptions {
            flip_v_uv: false,
            ..Default::default()
        };
        let s_no = build_obj_with_mtl(&mesh, &opts_no_flip);
        assert!(
            s_no.contains("vt 1.000000 0.000000"),
            "no-flip V of (1,0) stays 0"
        );
    }

    #[test]
    fn obj_with_mtl_precision() {
        let mesh = triangle_mesh();
        let opts = ObjMtlOptions {
            precision: 3,
            ..Default::default()
        };
        let s = build_obj_with_mtl(&mesh, &opts);
        assert!(s.contains("v 0.000 0.000 0.000"));
        assert!(s.contains("v 1.000 0.000 0.000"));
    }

    #[test]
    fn obj_with_mtl_no_normals_no_uvs() {
        let mesh = triangle_mesh();
        let opts = ObjMtlOptions {
            include_normals: false,
            include_uvs: false,
            ..Default::default()
        };
        let s = build_obj_with_mtl(&mesh, &opts);
        assert!(!s.contains("vn "), "should not have normal lines");
        assert!(!s.contains("vt "), "should not have UV lines");
        // face should be plain index
        assert!(s.contains("f 1 2 3"), "face should be plain indices");
    }

    // ---- export_obj_mtl ----------------------------------------------------

    #[test]
    fn export_obj_mtl_creates_both_files() {
        let mesh = quad_mesh();
        let mats = vec![MtlMaterial::skin()];
        let opts = ObjMtlOptions {
            mtl_filename: "test_quad.mtl".to_string(),
            material_name: "skin".to_string(),
            ..Default::default()
        };
        let obj_path = std::path::PathBuf::from("/tmp/test_quad.obj");
        let stats = export_obj_mtl(&mesh, &obj_path, &mats, &opts).expect("should succeed");

        assert!(obj_path.exists());
        let mtl_path = std::path::PathBuf::from("/tmp/test_quad.mtl");
        assert!(mtl_path.exists());

        assert_eq!(stats.vertex_count, 4);
        assert_eq!(stats.face_count, 2);
        assert!(stats.obj_bytes > 0);
        assert!(stats.mtl_bytes > 0);

        std::fs::remove_file(&obj_path).ok();
        std::fs::remove_file(&mtl_path).ok();
    }

    #[test]
    fn export_obj_mtl_stats_counts() {
        let mesh = triangle_mesh();
        let mats = vec![MtlMaterial::default()];
        let opts = ObjMtlOptions::default();
        let obj_path = std::path::PathBuf::from("/tmp/test_stats.obj");
        let stats = export_obj_mtl(&mesh, &obj_path, &mats, &opts).expect("should succeed");

        assert_eq!(stats.vertex_count, 3);
        assert_eq!(stats.normal_count, 3);
        assert_eq!(stats.uv_count, 3);
        assert_eq!(stats.face_count, 1);

        std::fs::remove_file(&obj_path).ok();
        std::fs::remove_file(std::path::Path::new("/tmp/mesh.mtl")).ok();
    }

    // ---- validate_obj ------------------------------------------------------

    #[test]
    fn validate_obj_valid_triangle() {
        let mesh = triangle_mesh();
        let opts = ObjMtlOptions::default();
        let s = build_obj_with_mtl(&mesh, &opts);
        assert!(
            validate_obj(&s).is_ok(),
            "valid triangle OBJ should pass validation"
        );
    }

    #[test]
    fn validate_obj_bad_index() {
        // Manually construct OBJ with an out-of-range face index
        let bad = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 99\n";
        let result = validate_obj(bad);
        assert!(result.is_err(), "out-of-range index should fail validation");
    }

    #[test]
    fn validate_obj_no_faces() {
        let ok = "v 0 0 0\nv 1 0 0\nv 0 1 0\n";
        assert!(
            validate_obj(ok).is_ok(),
            "OBJ with no faces should still be valid"
        );
    }

    // ---- parse_mtl_names ---------------------------------------------------

    #[test]
    fn parse_mtl_names_round_trip() {
        let mats = vec![
            MtlMaterial::skin(),
            MtlMaterial::cloth(),
            MtlMaterial::metal(),
        ];
        let content = build_mtl(&mats);
        let names = parse_mtl_names(&content);
        assert_eq!(names, vec!["skin", "cloth", "metal"]);
    }

    #[test]
    fn parse_mtl_names_empty() {
        let names = parse_mtl_names("# just a comment\n");
        assert!(names.is_empty());
    }

    // ---- ObjMtlOptions default ---------------------------------------------

    #[test]
    fn obj_mtl_options_default() {
        let opts = ObjMtlOptions::default();
        assert_eq!(opts.mtl_filename, "mesh.mtl");
        assert_eq!(opts.material_name, "default");
        assert!(opts.include_normals);
        assert!(opts.include_uvs);
        assert_eq!(opts.object_name, "OxiHumanMesh");
        assert!(!opts.flip_v_uv);
        assert_eq!(opts.precision, 6);
    }
}
