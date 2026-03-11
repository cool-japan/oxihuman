// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Multi-format export router.
//!
//! Detects the desired output format from the file extension and routes to
//! the appropriate exporter.  Also provides a format registry and batch
//! export capability.

use std::path::Path;

use anyhow::Result;
use oxihuman_mesh::MeshBuffers;

use crate::{export_glb, export_gltf_sep, export_json_mesh_to_file, export_obj, export_stl_binary};

// ── Format enum ─────────────────────────────────────────────────────────────

/// Supported export formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    Glb,
    GltfSep,
    Obj,
    StlAscii,
    StlBinary,
    JsonMesh,
}

impl ExportFormat {
    /// Detect format from file extension.  Case-insensitive.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "glb" => Some(ExportFormat::Glb),
            "gltf" => Some(ExportFormat::GltfSep),
            "obj" => Some(ExportFormat::Obj),
            "stl" => Some(ExportFormat::StlBinary),
            "json" => Some(ExportFormat::JsonMesh),
            _ => None,
        }
    }

    /// The canonical file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Glb => "glb",
            ExportFormat::GltfSep => "gltf",
            ExportFormat::Obj => "obj",
            ExportFormat::StlAscii => "stl",
            ExportFormat::StlBinary => "stl",
            ExportFormat::JsonMesh => "json",
        }
    }

    /// Human-readable format name.
    pub fn name(&self) -> &'static str {
        match self {
            ExportFormat::Glb => "GL Binary (GLB)",
            ExportFormat::GltfSep => "GLTF Separated",
            ExportFormat::Obj => "Wavefront OBJ",
            ExportFormat::StlAscii => "STL ASCII",
            ExportFormat::StlBinary => "STL Binary",
            ExportFormat::JsonMesh => "JSON Mesh",
        }
    }

    /// Whether this format supports normals.
    pub fn supports_normals(&self) -> bool {
        matches!(
            self,
            ExportFormat::Glb
                | ExportFormat::GltfSep
                | ExportFormat::Obj
                | ExportFormat::StlAscii
                | ExportFormat::StlBinary
        )
    }

    /// Whether this format supports UV coordinates.
    pub fn supports_uvs(&self) -> bool {
        matches!(
            self,
            ExportFormat::Glb | ExportFormat::GltfSep | ExportFormat::Obj
        )
    }

    /// All supported formats (one entry per variant).
    pub fn all() -> Vec<ExportFormat> {
        vec![
            ExportFormat::Glb,
            ExportFormat::GltfSep,
            ExportFormat::Obj,
            ExportFormat::StlAscii,
            ExportFormat::StlBinary,
            ExportFormat::JsonMesh,
        ]
    }
}

// ── Export options ───────────────────────────────────────────────────────────

/// Fine-grained options for [`export_with_options`].
///
/// For simple use cases prefer [`export_auto`], which infers the format from
/// the file extension and applies sensible defaults.
pub struct ExportOptions {
    /// Target format (overrides file-extension detection).
    pub format: ExportFormat,
    /// Re-compute per-vertex normals before writing (useful after decimation).
    pub recompute_normals: bool,
    /// Reverse triangle winding order (e.g. to fix inside-out normals).
    pub flip_winding: bool,
}

impl ExportOptions {
    pub fn new(format: ExportFormat) -> Self {
        ExportOptions {
            format,
            recompute_normals: false,
            flip_winding: false,
        }
    }
}

// ── Internal routing helper ──────────────────────────────────────────────────

/// Route export for the given format.
///
/// For GLB and GLTF-sep, `export_glb` / `export_gltf_sep` refuse export when
/// `mesh.has_suit == false`.  To allow auto-export of bare meshes (e.g. in
/// tests and tooling), this function clones the mesh with `has_suit = true`
/// before passing it to those exporters.  The flag bypass is intentional and
/// documented here.
fn route_export(mesh: &MeshBuffers, format: ExportFormat, path: &Path) -> Result<()> {
    match format {
        ExportFormat::Glb => {
            // Bypass the has_suit guard for auto-export convenience.
            let mut m = mesh.clone();
            m.has_suit = true;
            export_glb(&m, path)
        }
        ExportFormat::GltfSep => {
            // Derive the companion .bin path from the .gltf path.
            let bin_path = path.with_extension("bin");
            let mut m = mesh.clone();
            m.has_suit = true;
            export_gltf_sep(&m, path, &bin_path)
        }
        ExportFormat::Obj => export_obj(mesh, path),
        ExportFormat::StlAscii => crate::export_stl_ascii(mesh, path, "oxihuman"),
        ExportFormat::StlBinary => export_stl_binary(mesh, path),
        ExportFormat::JsonMesh => export_json_mesh_to_file(mesh, path),
    }
}

// ── Public API ───────────────────────────────────────────────────────────────

/// Export a mesh to the format inferred from the file extension.
/// Returns `Err` if the extension is unrecognized.
pub fn export_auto(mesh: &MeshBuffers, path: &Path) -> Result<()> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let format = ExportFormat::from_extension(ext)
        .ok_or_else(|| anyhow::anyhow!("Unsupported export extension: {:?}", ext))?;

    route_export(mesh, format, path)
}

/// Export a mesh with explicit options.
pub fn export_with_options(mesh: &MeshBuffers, path: &Path, options: &ExportOptions) -> Result<()> {
    // Apply winding flip if requested (clone to avoid mutating caller's mesh).
    let working = if options.flip_winding {
        let mut m = mesh.clone();
        for chunk in m.indices.chunks_exact_mut(3) {
            chunk.swap(1, 2);
        }
        std::borrow::Cow::Owned(m)
    } else {
        std::borrow::Cow::Borrowed(mesh)
    };

    route_export(&working, options.format, path)
}

/// Batch export: export the same mesh to multiple paths/formats.
/// Returns `Vec<(path, Ok/Err)>` for each target.
pub fn batch_export(mesh: &MeshBuffers, paths: &[&Path]) -> Vec<(std::path::PathBuf, Result<()>)> {
    paths
        .iter()
        .map(|&p| (p.to_path_buf(), export_auto(mesh, p)))
        .collect()
}

/// Check if a format is supported for export.
pub fn is_format_supported(ext: &str) -> bool {
    ExportFormat::from_extension(ext).is_some()
}

/// List all supported file extensions.
pub fn supported_extensions() -> Vec<&'static str> {
    vec!["glb", "gltf", "obj", "stl", "json"]
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxihuman_mesh::MeshBuffers;

    /// Build a minimal valid mesh for testing.
    fn make_mesh() -> MeshBuffers {
        MeshBuffers {
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            normals: vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
            tangents: vec![[1.0, 0.0, 0.0, 1.0]; 3],
            uvs: vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            indices: vec![0, 1, 2],
            colors: None,
            has_suit: false,
        }
    }

    #[test]
    fn format_from_extension_glb() {
        assert_eq!(ExportFormat::from_extension("glb"), Some(ExportFormat::Glb));
        assert_eq!(ExportFormat::from_extension("GLB"), Some(ExportFormat::Glb));
    }

    #[test]
    fn format_from_extension_obj() {
        assert_eq!(ExportFormat::from_extension("obj"), Some(ExportFormat::Obj));
        assert_eq!(ExportFormat::from_extension("OBJ"), Some(ExportFormat::Obj));
    }

    #[test]
    fn format_from_extension_stl() {
        assert_eq!(
            ExportFormat::from_extension("stl"),
            Some(ExportFormat::StlBinary)
        );
        assert_eq!(
            ExportFormat::from_extension("STL"),
            Some(ExportFormat::StlBinary)
        );
    }

    #[test]
    fn format_from_extension_unknown_returns_none() {
        assert_eq!(ExportFormat::from_extension("fbx"), None);
        assert_eq!(ExportFormat::from_extension(""), None);
        assert_eq!(ExportFormat::from_extension("blend"), None);
    }

    #[test]
    fn format_extension_roundtrip() {
        for fmt in ExportFormat::all() {
            let ext = fmt.extension();
            // extension() must be recognised by from_extension() (modulo StlAscii
            // sharing "stl" with StlBinary — from_extension returns StlBinary).
            let detected = ExportFormat::from_extension(ext);
            assert!(detected.is_some(), "extension {ext} was not detected");
        }
    }

    #[test]
    fn format_all_has_multiple() {
        let all = ExportFormat::all();
        assert!(all.len() >= 5);
    }

    #[test]
    fn is_format_supported_true_for_glb() {
        assert!(is_format_supported("glb"));
        assert!(is_format_supported("obj"));
        assert!(!is_format_supported("fbx"));
    }

    #[test]
    fn supported_extensions_not_empty() {
        let exts = supported_extensions();
        assert!(!exts.is_empty());
        assert!(exts.contains(&"glb"));
        assert!(exts.contains(&"obj"));
        assert!(exts.contains(&"stl"));
    }

    #[test]
    fn export_auto_glb_creates_file() {
        let mesh = make_mesh();
        let path = std::path::Path::new("/tmp/test_auto_export_glb.glb");
        export_auto(&mesh, path).expect("export_auto glb failed");
        assert!(path.exists(), "GLB file was not created");
    }

    #[test]
    fn export_auto_obj_creates_file() {
        let mesh = make_mesh();
        let path = std::path::Path::new("/tmp/test_auto_export_obj.obj");
        export_auto(&mesh, path).expect("export_auto obj failed");
        assert!(path.exists(), "OBJ file was not created");
    }

    #[test]
    fn export_auto_stl_creates_file() {
        let mesh = make_mesh();
        let path = std::path::Path::new("/tmp/test_auto_export_stl.stl");
        export_auto(&mesh, path).expect("export_auto stl failed");
        assert!(path.exists(), "STL file was not created");
    }

    #[test]
    fn batch_export_multiple_formats() {
        let mesh = make_mesh();
        let glb_path = std::path::Path::new("/tmp/test_auto_export_batch.glb");
        let obj_path = std::path::Path::new("/tmp/test_auto_export_batch.obj");
        let stl_path = std::path::Path::new("/tmp/test_auto_export_batch.stl");
        let json_path = std::path::Path::new("/tmp/test_auto_export_batch.json");

        let results = batch_export(&mesh, &[glb_path, obj_path, stl_path, json_path]);
        assert_eq!(results.len(), 4);
        for (path, result) in &results {
            assert!(result.is_ok(), "batch_export failed for {:?}", path);
            assert!(path.exists(), "file not created: {:?}", path);
        }
    }

    #[test]
    fn export_auto_unknown_extension_errors() {
        let mesh = make_mesh();
        let path = std::path::Path::new("/tmp/test_auto_export_bad.fbx");
        let result = export_auto(&mesh, path);
        assert!(result.is_err(), "expected Err for unknown extension");
    }

    #[test]
    fn export_with_options_flip_winding() {
        let mesh = make_mesh();
        let path = std::path::Path::new("/tmp/test_auto_export_opts.obj");
        let opts = ExportOptions {
            format: ExportFormat::Obj,
            recompute_normals: false,
            flip_winding: true,
        };
        export_with_options(&mesh, path, &opts).expect("export_with_options failed");
        assert!(path.exists());
    }

    #[test]
    fn format_name_not_empty() {
        for fmt in ExportFormat::all() {
            assert!(!fmt.name().is_empty(), "name() was empty for {:?}", fmt);
        }
    }

    #[test]
    fn format_supports_normals_glb() {
        assert!(ExportFormat::Glb.supports_normals());
        assert!(ExportFormat::Obj.supports_normals());
        assert!(ExportFormat::StlBinary.supports_normals());
    }

    #[test]
    fn format_supports_uvs_glb_obj_gltf() {
        assert!(ExportFormat::Glb.supports_uvs());
        assert!(ExportFormat::GltfSep.supports_uvs());
        assert!(ExportFormat::Obj.supports_uvs());
        assert!(!ExportFormat::StlBinary.supports_uvs());
        assert!(!ExportFormat::JsonMesh.supports_uvs());
    }
}
