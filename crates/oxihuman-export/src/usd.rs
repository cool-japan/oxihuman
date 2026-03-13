// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(non_snake_case)]

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::Context;
use oxihuman_mesh::MeshBuffers;

/// Options controlling USDA export output.
pub struct UsdExportOptions {
    pub prim_name: String,
    pub root_name: String,
    pub up_axis: String,
    pub meters_per_unit: f32,
    pub include_normals: bool,
    pub include_uvs: bool,
    pub include_displayColor: bool,
}

impl Default for UsdExportOptions {
    fn default() -> Self {
        Self {
            prim_name: "Body".to_string(),
            root_name: "Root".to_string(),
            up_axis: "Y".to_string(),
            meters_per_unit: 1.0,
            include_normals: true,
            include_uvs: true,
            include_displayColor: false,
        }
    }
}

/// Statistics returned after a successful USDA export.
pub struct UsdExportStats {
    pub vertex_count: usize,
    pub face_count: usize,
    pub file_size_bytes: usize,
    pub has_normals: bool,
    pub has_uvs: bool,
}

// ── formatting helpers ────────────────────────────────────────────────────────

/// Format a float3 slice as a USD tuple list: `[(x, y, z), ...]`
pub fn format_float3_array(data: &[[f32; 3]]) -> String {
    let inner: Vec<String> = data
        .iter()
        .map(|v| format!("({:.6}, {:.6}, {:.6})", v[0], v[1], v[2]))
        .collect();
    format!("[{}]", inner.join(", "))
}

/// Format a float2 slice as a USD tuple list: `[(u, v), ...]`
pub fn format_float2_array(data: &[[f32; 2]]) -> String {
    let inner: Vec<String> = data
        .iter()
        .map(|v| format!("({:.6}, {:.6})", v[0], v[1]))
        .collect();
    format!("[{}]", inner.join(", "))
}

/// Format a u32 slice as a USD array: `[i0, i1, i2, ...]`
pub fn format_int_array(data: &[u32]) -> String {
    let inner: Vec<String> = data.iter().map(|i| i.to_string()).collect();
    format!("[{}]", inner.join(", "))
}

// ── core builder ─────────────────────────────────────────────────────────────

/// Build a USDA string from a mesh and options.
pub fn build_usda(mesh: &MeshBuffers, opts: &UsdExportOptions) -> String {
    let face_count = mesh.indices.len() / 3;

    // 1. Header
    let mut out = format!(
        "#usda 1.0\n(\n    defaultPrim = \"{root}\"\n    upAxis = \"{up}\"\n    metersPerUnit = {mpu}\n)\n\n",
        root = opts.root_name,
        up = opts.up_axis,
        mpu = opts.meters_per_unit,
    );

    // 2. Root xform + mesh prim
    out.push_str(&format!(
        "def Xform \"{root}\"\n{{\n    def Mesh \"{prim}\"\n    {{\n",
        root = opts.root_name,
        prim = opts.prim_name,
    ));

    // 3. Points
    out.push_str(&format!(
        "        float3[] points = {}\n",
        format_float3_array(&mesh.positions)
    ));

    // 4. faceVertexCounts — one 3 per triangle
    let counts: Vec<u32> = vec![3u32; face_count];
    out.push_str(&format!(
        "        int[] faceVertexCounts = {}\n",
        format_int_array(&counts)
    ));

    // 5. faceVertexIndices
    out.push_str(&format!(
        "        int[] faceVertexIndices = {}\n",
        format_int_array(&mesh.indices)
    ));

    // 6. Optional normals
    if opts.include_normals && !mesh.normals.is_empty() {
        out.push_str(&format!(
            "        normal3f[] normals = {}\n",
            format_float3_array(&mesh.normals)
        ));
    }

    // 7. Optional UVs
    if opts.include_uvs && !mesh.uvs.is_empty() {
        out.push_str(&format!(
            "        texCoord2f[] primvars:st = {}\n",
            format_float2_array(&mesh.uvs)
        ));
        out.push_str("        uniform token[] primvars:st:indices = None\n");
    }

    // 8. Transform
    out.push_str("        double3 xformOp:translate = (0, 0, 0)\n");
    out.push_str("        uniform token[] xformOpOrder = [\"xformOp:translate\"]\n");

    // 9. Close braces
    out.push_str("    }\n}\n");

    out
}

// ── file-level exports ────────────────────────────────────────────────────────

/// Export a single mesh to a `.usda` file.
pub fn export_usda(
    mesh: &MeshBuffers,
    path: &Path,
    opts: &UsdExportOptions,
) -> anyhow::Result<UsdExportStats> {
    let content = build_usda(mesh, opts);
    let bytes = content.as_bytes();

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| format!("create dir {:?}", parent))?;
        }
    }

    fs::write(path, bytes).with_context(|| format!("write {:?}", path))?;

    Ok(UsdExportStats {
        vertex_count: mesh.positions.len(),
        face_count: mesh.indices.len() / 3,
        file_size_bytes: bytes.len(),
        has_normals: opts.include_normals && !mesh.normals.is_empty(),
        has_uvs: opts.include_uvs && !mesh.uvs.is_empty(),
    })
}

/// Export multiple meshes as a USDA scene (each as a separate Mesh prim).
pub fn export_usda_scene(
    meshes: &[(&MeshBuffers, &str)],
    path: &Path,
    opts: &UsdExportOptions,
) -> anyhow::Result<()> {
    let mut out = format!(
        "#usda 1.0\n(\n    defaultPrim = \"{root}\"\n    upAxis = \"{up}\"\n    metersPerUnit = {mpu}\n)\n\n",
        root = opts.root_name,
        up = opts.up_axis,
        mpu = opts.meters_per_unit,
    );

    out.push_str(&format!(
        "def Xform \"{root}\"\n{{\n",
        root = opts.root_name
    ));

    for (mesh, name) in meshes {
        let face_count = mesh.indices.len() / 3;
        out.push_str(&format!("    def Mesh \"{name}\"\n    {{\n"));

        out.push_str(&format!(
            "        float3[] points = {}\n",
            format_float3_array(&mesh.positions)
        ));

        let counts: Vec<u32> = vec![3u32; face_count];
        out.push_str(&format!(
            "        int[] faceVertexCounts = {}\n",
            format_int_array(&counts)
        ));

        out.push_str(&format!(
            "        int[] faceVertexIndices = {}\n",
            format_int_array(&mesh.indices)
        ));

        if opts.include_normals && !mesh.normals.is_empty() {
            out.push_str(&format!(
                "        normal3f[] normals = {}\n",
                format_float3_array(&mesh.normals)
            ));
        }

        if opts.include_uvs && !mesh.uvs.is_empty() {
            out.push_str(&format!(
                "        texCoord2f[] primvars:st = {}\n",
                format_float2_array(&mesh.uvs)
            ));
            out.push_str("        uniform token[] primvars:st:indices = None\n");
        }

        out.push_str("        double3 xformOp:translate = (0, 0, 0)\n");
        out.push_str("        uniform token[] xformOpOrder = [\"xformOp:translate\"]\n");
        out.push_str("    }\n");
    }

    out.push_str("}\n");

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| format!("create dir {:?}", parent))?;
        }
    }

    fs::write(path, out.as_bytes()).with_context(|| format!("write {:?}", path))?;
    Ok(())
}

/// Validate that a `.usda` file has the correct `#usda 1.0` header.
pub fn validate_usda(path: &Path) -> anyhow::Result<bool> {
    let file = fs::File::open(path).with_context(|| format!("open {:?}", path))?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader
        .read_line(&mut first_line)
        .with_context(|| "read first line")?;
    Ok(first_line.trim() == "#usda 1.0")
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;

    /// Build a minimal 2-triangle (quad) test mesh.
    fn two_tri_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            normals: vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 4],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            colors: None,
            has_suit: true,
        }
    }

    #[test]
    fn test_build_usda_header() {
        let mesh = two_tri_mesh();
        let opts = UsdExportOptions::default();
        let usda = build_usda(&mesh, &opts);
        assert!(usda.starts_with("#usda 1.0"), "must start with #usda 1.0");
        assert!(usda.contains("defaultPrim = \"Root\""));
        assert!(usda.contains("upAxis = \"Y\""));
        assert!(usda.contains("metersPerUnit = 1"));
    }

    #[test]
    fn test_build_usda_has_points() {
        let mesh = two_tri_mesh();
        let opts = UsdExportOptions::default();
        let usda = build_usda(&mesh, &opts);
        assert!(
            usda.contains("float3[] points"),
            "must contain points array"
        );
        assert!(usda.contains("(0.000000, 0.000000, 0.000000)"));
        assert!(usda.contains("(1.000000, 0.000000, 0.000000)"));
    }

    #[test]
    fn test_build_usda_has_face_counts() {
        let mesh = two_tri_mesh();
        let opts = UsdExportOptions::default();
        let usda = build_usda(&mesh, &opts);
        assert!(
            usda.contains("int[] faceVertexCounts"),
            "must contain faceVertexCounts"
        );
        // 2 triangles → [3, 3]
        assert!(usda.contains("[3, 3]"));
    }

    #[test]
    fn test_build_usda_has_indices() {
        let mesh = two_tri_mesh();
        let opts = UsdExportOptions::default();
        let usda = build_usda(&mesh, &opts);
        assert!(
            usda.contains("int[] faceVertexIndices"),
            "must contain faceVertexIndices"
        );
        assert!(usda.contains("[0, 1, 2, 0, 2, 3]"));
    }

    #[test]
    fn test_build_usda_with_normals() {
        let mesh = two_tri_mesh();
        let opts = UsdExportOptions {
            include_normals: true,
            ..Default::default()
        };
        let usda = build_usda(&mesh, &opts);
        assert!(usda.contains("normal3f[] normals"), "must contain normals");
        assert!(usda.contains("(0.000000, 0.000000, 1.000000)"));
    }

    #[test]
    fn test_build_usda_with_uvs() {
        let mesh = two_tri_mesh();
        let opts = UsdExportOptions {
            include_uvs: true,
            ..Default::default()
        };
        let usda = build_usda(&mesh, &opts);
        assert!(
            usda.contains("texCoord2f[] primvars:st"),
            "must contain UVs"
        );
        assert!(usda.contains("primvars:st:indices"));
    }

    #[test]
    fn test_format_float3_array() {
        let data: Vec<[f32; 3]> = vec![[1.0, 2.0, 3.0], [4.5, 5.5, 6.5]];
        let result = format_float3_array(&data);
        assert_eq!(
            result,
            "[(1.000000, 2.000000, 3.000000), (4.500000, 5.500000, 6.500000)]"
        );
    }

    #[test]
    fn test_format_float2_array() {
        let data: Vec<[f32; 2]> = vec![[0.0, 1.0], [0.5, 0.5]];
        let result = format_float2_array(&data);
        assert_eq!(result, "[(0.000000, 1.000000), (0.500000, 0.500000)]");
    }

    #[test]
    fn test_format_int_array() {
        let data: Vec<u32> = vec![0, 1, 2, 3];
        let result = format_int_array(&data);
        assert_eq!(result, "[0, 1, 2, 3]");
    }

    #[test]
    fn test_export_usda_to_file() {
        let mesh = two_tri_mesh();
        let opts = UsdExportOptions::default();
        let path = std::path::PathBuf::from("/tmp/test_export.usda");
        let stats = export_usda(&mesh, &path, &opts).expect("export_usda should succeed");
        assert_eq!(stats.vertex_count, 4);
        assert_eq!(stats.face_count, 2);
        assert!(stats.file_size_bytes > 0);
        assert!(stats.has_normals);
        assert!(stats.has_uvs);
        assert!(path.exists());
    }

    #[test]
    fn test_validate_usda_valid() {
        let mesh = two_tri_mesh();
        let opts = UsdExportOptions::default();
        let path = std::path::PathBuf::from("/tmp/test_validate_valid.usda");
        export_usda(&mesh, &path, &opts).expect("export_usda should succeed");
        let valid = validate_usda(&path).expect("validate_usda should succeed");
        assert!(valid, "exported file should be valid USDA");
    }

    #[test]
    fn test_validate_usda_invalid() {
        let path = std::path::PathBuf::from("/tmp/test_validate_invalid.usda");
        fs::write(&path, b"not a usda file\nsome content\n").expect("write temp file");
        let valid = validate_usda(&path).expect("validate_usda should succeed");
        assert!(!valid, "file without #usda 1.0 header should be invalid");
    }

    #[test]
    fn test_export_usda_scene() {
        let mesh1 = two_tri_mesh();
        let mesh2 = two_tri_mesh();
        let opts = UsdExportOptions::default();
        let path = std::path::PathBuf::from("/tmp/test_scene.usda");
        let meshes: Vec<(&MeshBuffers, &str)> = vec![(&mesh1, "Body"), (&mesh2, "Hair")];
        export_usda_scene(&meshes, &path, &opts).expect("export_usda_scene should succeed");
        let content = fs::read_to_string(&path).expect("read scene file");
        assert!(content.starts_with("#usda 1.0"));
        assert!(content.contains("def Mesh \"Body\""));
        assert!(content.contains("def Mesh \"Hair\""));
        assert!(path.exists());
    }
}
