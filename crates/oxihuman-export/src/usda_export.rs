// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! USDA (Universal Scene Description ASCII) text format writer.
//!
//! Produces valid `.usda` files with support for meshes, materials,
//! skeletons, skin bindings, blend shapes, and xform transforms.

use std::fmt::Write as FmtWrite;

// ── Enums ────────────────────────────────────────────────────────────────────

/// Subdivision scheme for a USD mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsdSubdivScheme {
    /// No subdivision.
    None,
    /// Catmull-Clark subdivision.
    CatmullClark,
    /// Loop subdivision.
    Loop,
    /// Bilinear subdivision.
    Bilinear,
}

impl UsdSubdivScheme {
    /// Return the USDA token string for this scheme.
    fn as_token(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::CatmullClark => "catmullClark",
            Self::Loop => "loop",
            Self::Bilinear => "bilinear",
        }
    }
}

// ── Data structures ──────────────────────────────────────────────────────────

/// A USD Mesh primitive with geometry data.
pub struct UsdMesh {
    /// Name of this mesh prim.
    pub name: String,
    /// Vertex positions (point3f).
    pub positions: Vec<[f64; 3]>,
    /// Per-vertex or per-face-vertex normals (normal3f).
    pub normals: Vec<[f64; 3]>,
    /// Texture coordinates (texCoord2f).
    pub uvs: Vec<[f64; 2]>,
    /// Number of vertices per face (e.g. `[3, 3, 4]`).
    pub face_vertex_counts: Vec<i32>,
    /// Indices into the points array.
    pub face_vertex_indices: Vec<i32>,
    /// Subdivision scheme.
    pub subdivision_scheme: UsdSubdivScheme,
}

/// A USD material using UsdPreviewSurface.
pub struct UsdMaterial {
    /// Name of this material prim.
    pub name: String,
    /// Diffuse base colour (linear).
    pub diffuse_color: [f64; 3],
    /// Metallic factor (0..1).
    pub metallic: f64,
    /// Roughness factor (0..1).
    pub roughness: f64,
    /// Opacity (0..1).
    pub opacity: f64,
    /// Normal map scale.
    pub normal_scale: f64,
}

/// A USD skeleton definition.
pub struct UsdSkeleton {
    /// Short joint names (e.g. `["Hips", "Spine"]`).
    pub joint_names: Vec<String>,
    /// Full joint paths (e.g. `["Hips", "Hips/Spine"]`).
    pub joint_paths: Vec<String>,
    /// Bind transforms as column-major 4x4 matrices (flattened to 16 elements).
    pub bind_transforms: Vec<[f64; 16]>,
    /// Rest transforms as column-major 4x4 matrices (flattened to 16 elements).
    pub rest_transforms: Vec<[f64; 16]>,
}

/// Skin binding data that associates a mesh with a skeleton.
pub struct UsdSkinBinding {
    /// Per-vertex joint indices (variable number of influences per vertex).
    pub joint_indices: Vec<Vec<i32>>,
    /// Per-vertex joint weights (same shape as `joint_indices`).
    pub joint_weights: Vec<Vec<f64>>,
    /// Scene path to the skeleton prim.
    pub skeleton_path: String,
}

/// A single blend shape (morph target).
pub struct UsdBlendShape {
    /// Name of the blend shape.
    pub name: String,
    /// Position offsets for affected vertices.
    pub offsets: Vec<[f64; 3]>,
    /// Indices of vertices that are affected.
    pub point_indices: Vec<i32>,
}

// ── Writer ───────────────────────────────────────────────────────────────────

/// USDA text format writer.
///
/// Builds a `.usda` file incrementally. Call methods in the order dictated
/// by the USD hierarchy you wish to produce, then call [`finish`](Self::finish)
/// to obtain the final string.
pub struct UsdaWriter {
    output: String,
    indent_level: usize,
}

impl Default for UsdaWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl UsdaWriter {
    // ── Construction ─────────────────────────────────────────────────────

    /// Create a new, empty writer.
    pub fn new() -> Self {
        Self {
            output: String::with_capacity(4096),
            indent_level: 0,
        }
    }

    // ── Header ───────────────────────────────────────────────────────────

    /// Write the USDA file header with layer metadata.
    ///
    /// `up_axis` should be `"Y"` or `"Z"`.
    pub fn write_header(&mut self, up_axis: &str, meters_per_unit: f64) {
        self.output.push_str("#usda 1.0\n(\n");
        let _ = writeln!(self.output, "    upAxis = \"{}\"", up_axis);
        let _ = writeln!(
            self.output,
            "    metersPerUnit = {:.6}",
            meters_per_unit
        );
        self.output.push_str(")\n\n");
    }

    // ── Scope / Def helpers ──────────────────────────────────────────────

    /// Open a `def <kind> "<name>" { ... }` block.
    pub fn begin_def(&mut self, kind: &str, name: &str) {
        self.write_indent();
        let _ = writeln!(self.output, "def {} \"{}\" {{", kind, name);
        self.indent_level += 1;
    }

    /// Close the most recent `def` block.
    pub fn end_def(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
        self.write_indent();
        self.output.push_str("}\n");
    }

    // ── Mesh ─────────────────────────────────────────────────────────────

    /// Write a complete Mesh prim.
    pub fn write_mesh(&mut self, mesh: &UsdMesh) -> anyhow::Result<()> {
        self.begin_def("Mesh", &mesh.name);

        // subdivision scheme
        self.write_indent();
        let _ = writeln!(
            self.output,
            "uniform token subdivisionScheme = \"{}\"",
            mesh.subdivision_scheme.as_token()
        );

        // points
        self.write_indent();
        self.output.push_str("point3f[] points = ");
        self.write_f64x3_array(&mesh.positions);
        self.output.push('\n');

        // normals
        if !mesh.normals.is_empty() {
            self.write_indent();
            self.output.push_str("normal3f[] normals = ");
            self.write_f64x3_array(&mesh.normals);
            self.output.push_str(" (\n");
            self.write_indent();
            self.output.push_str("    interpolation = \"faceVarying\"\n");
            self.write_indent();
            self.output.push_str(")\n");
        }

        // UVs
        if !mesh.uvs.is_empty() {
            self.write_indent();
            self.output.push_str("texCoord2f[] primvars:st = ");
            self.write_f64x2_array(&mesh.uvs);
            self.output.push_str(" (\n");
            self.write_indent();
            self.output.push_str("    interpolation = \"faceVarying\"\n");
            self.write_indent();
            self.output.push_str(")\n");
        }

        // face vertex counts
        self.write_indent();
        self.output.push_str("int[] faceVertexCounts = ");
        self.write_i32_array(&mesh.face_vertex_counts);
        self.output.push('\n');

        // face vertex indices
        self.write_indent();
        self.output.push_str("int[] faceVertexIndices = ");
        self.write_i32_array(&mesh.face_vertex_indices);
        self.output.push('\n');

        self.end_def();
        Ok(())
    }

    // ── Material ─────────────────────────────────────────────────────────

    /// Write a Material prim with a UsdPreviewSurface shader.
    pub fn write_material(&mut self, mat: &UsdMaterial) -> anyhow::Result<()> {
        self.begin_def("Material", &mat.name);

        // surface output
        self.write_indent();
        let _ = writeln!(
            self.output,
            "token outputs:surface.connect = </{}/PBRShader.outputs:surface>",
            mat.name
        );

        // PBR shader
        self.begin_def("Shader", "PBRShader");

        self.write_indent();
        self.output
            .push_str("uniform token info:id = \"UsdPreviewSurface\"\n");

        self.write_indent();
        let _ = writeln!(
            self.output,
            "color3f inputs:diffuseColor = ({:.6}, {:.6}, {:.6})",
            mat.diffuse_color[0], mat.diffuse_color[1], mat.diffuse_color[2]
        );

        self.write_indent();
        let _ = writeln!(
            self.output,
            "float inputs:metallic = {:.6}",
            mat.metallic
        );

        self.write_indent();
        let _ = writeln!(
            self.output,
            "float inputs:roughness = {:.6}",
            mat.roughness
        );

        self.write_indent();
        let _ = writeln!(self.output, "float inputs:opacity = {:.6}", mat.opacity);

        self.write_indent();
        let _ = writeln!(
            self.output,
            "float inputs:normal = {:.6}",
            mat.normal_scale
        );

        self.write_indent();
        self.output
            .push_str("token outputs:surface\n");

        self.end_def(); // Shader

        self.end_def(); // Material
        Ok(())
    }

    // ── Skeleton ─────────────────────────────────────────────────────────

    /// Write a Skeleton prim.
    pub fn write_skeleton(&mut self, skel: &UsdSkeleton) -> anyhow::Result<()> {
        self.begin_def("Skeleton", "Skeleton");

        // joint names
        self.write_indent();
        self.output.push_str("uniform token[] joints = ");
        self.write_string_array(&skel.joint_paths);
        self.output.push('\n');

        // joint names (display)
        self.write_indent();
        self.output.push_str("uniform token[] jointNames = ");
        self.write_string_array(&skel.joint_names);
        self.output.push('\n');

        // bind transforms
        self.write_indent();
        self.output
            .push_str("matrix4d[] bindTransforms = ");
        self.write_matrix4d_array(&skel.bind_transforms);
        self.output.push('\n');

        // rest transforms
        self.write_indent();
        self.output
            .push_str("matrix4d[] restTransforms = ");
        self.write_matrix4d_array(&skel.rest_transforms);
        self.output.push('\n');

        self.end_def();
        Ok(())
    }

    // ── Skin binding ─────────────────────────────────────────────────────

    /// Write skin binding properties on an existing mesh prim.
    ///
    /// This writes a `SkelBindingAPI` block that should appear inside or
    /// adjacent to the mesh prim identified by `mesh_path`.
    pub fn write_skin_binding(
        &mut self,
        mesh_path: &str,
        binding: &UsdSkinBinding,
    ) -> anyhow::Result<()> {
        self.begin_def("SkelBindingAPI", &format!("{}_SkelBinding", sanitise_name(mesh_path)));

        // skeleton path
        self.write_indent();
        let _ = writeln!(
            self.output,
            "uniform token primvars:skel:skeleton = \"{}\"",
            binding.skeleton_path
        );

        // Flatten joint indices and weights to a fixed element size.
        // USD requires uniform element size, so we find the maximum number
        // of influences and pad shorter entries with 0.
        let element_size = binding
            .joint_indices
            .iter()
            .map(|v| v.len())
            .max()
            .unwrap_or(0);

        if element_size > 0 {
            // joint indices (flattened)
            self.write_indent();
            self.output
                .push_str("int[] primvars:skel:jointIndices = ");
            self.write_flat_joint_indices(&binding.joint_indices, element_size);
            self.output.push_str(" (\n");
            self.write_indent();
            let _ = writeln!(self.output, "    elementSize = {}", element_size);
            self.write_indent();
            self.output.push_str("    interpolation = \"vertex\"\n");
            self.write_indent();
            self.output.push_str(")\n");

            // joint weights (flattened)
            self.write_indent();
            self.output
                .push_str("float[] primvars:skel:jointWeights = ");
            self.write_flat_joint_weights(&binding.joint_weights, element_size);
            self.output.push_str(" (\n");
            self.write_indent();
            let _ = writeln!(self.output, "    elementSize = {}", element_size);
            self.write_indent();
            self.output.push_str("    interpolation = \"vertex\"\n");
            self.write_indent();
            self.output.push_str(")\n");
        }

        self.end_def();
        Ok(())
    }

    // ── Blend shapes ─────────────────────────────────────────────────────

    /// Write blend shape prims under the given mesh path scope.
    pub fn write_blend_shapes(
        &mut self,
        mesh_path: &str,
        shapes: &[UsdBlendShape],
    ) -> anyhow::Result<()> {
        if shapes.is_empty() {
            return Ok(());
        }

        self.begin_def(
            "Scope",
            &format!("{}_BlendShapes", sanitise_name(mesh_path)),
        );

        // Collect blend shape names for the targets relationship.
        let target_names: Vec<String> = shapes
            .iter()
            .map(|s| format!("<./{}>", s.name))
            .collect();

        self.write_indent();
        let _ = write!(self.output, "uniform token[] blendShapes = [");
        for (i, shape) in shapes.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            let _ = write!(self.output, "\"{}\"", shape.name);
        }
        self.output.push_str("]\n");

        self.write_indent();
        let _ = write!(self.output, "uniform rel blendShapeTargets = [");
        for (i, target) in target_names.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str(target);
        }
        self.output.push_str("]\n");

        // Write each blend shape
        for shape in shapes {
            self.begin_def("BlendShape", &shape.name);

            // offsets
            self.write_indent();
            self.output.push_str("vector3f[] offsets = ");
            self.write_f64x3_array(&shape.offsets);
            self.output.push('\n');

            // point indices
            self.write_indent();
            self.output.push_str("int[] pointIndices = ");
            self.write_i32_array(&shape.point_indices);
            self.output.push('\n');

            self.end_def();
        }

        self.end_def(); // Scope
        Ok(())
    }

    // ── Xform ────────────────────────────────────────────────────────────

    /// Write an Xform prim with a 4x4 transform matrix.
    pub fn write_xform(&mut self, name: &str, matrix: &[f64; 16]) -> anyhow::Result<()> {
        self.begin_def("Xform", name);

        self.write_indent();
        self.output
            .push_str("matrix4d xformOp:transform = ");
        self.write_matrix4d(matrix);
        self.output.push('\n');

        self.write_indent();
        self.output
            .push_str("uniform token[] xformOpOrder = [\"xformOp:transform\"]\n");

        self.end_def();
        Ok(())
    }

    // ── Finalise ─────────────────────────────────────────────────────────

    /// Consume the writer and return the complete USDA string.
    ///
    /// Any unclosed `def` blocks are closed automatically.
    pub fn finish(mut self) -> String {
        // Close any remaining open scopes.
        while self.indent_level > 0 {
            self.end_def();
        }
        self.output
    }

    // ── Internal helpers ─────────────────────────────────────────────────

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn write_f64x3_array(&mut self, data: &[[f64; 3]]) {
        self.output.push('[');
        for (i, v) in data.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            let _ = write!(
                self.output,
                "({:.6}, {:.6}, {:.6})",
                v[0], v[1], v[2]
            );
        }
        self.output.push(']');
    }

    fn write_f64x2_array(&mut self, data: &[[f64; 2]]) {
        self.output.push('[');
        for (i, v) in data.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            let _ = write!(self.output, "({:.6}, {:.6})", v[0], v[1]);
        }
        self.output.push(']');
    }

    fn write_i32_array(&mut self, data: &[i32]) {
        self.output.push('[');
        for (i, v) in data.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            let _ = write!(self.output, "{}", v);
        }
        self.output.push(']');
    }

    fn write_string_array(&mut self, data: &[String]) {
        self.output.push('[');
        for (i, s) in data.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            let _ = write!(self.output, "\"{}\"", s);
        }
        self.output.push(']');
    }

    /// Write a single 4x4 matrix in USDA row-major display format.
    ///
    /// USD stores matrices as `( (r00,r01,r02,r03), ... )`.
    /// Input is a flat 16-element array in row-major order.
    fn write_matrix4d(&mut self, m: &[f64; 16]) {
        self.output.push_str("( ");
        for row in 0..4 {
            if row > 0 {
                self.output.push_str(", ");
            }
            let base = row * 4;
            let _ = write!(
                self.output,
                "({:.6}, {:.6}, {:.6}, {:.6})",
                m[base],
                m[base + 1],
                m[base + 2],
                m[base + 3]
            );
        }
        self.output.push_str(" )");
    }

    fn write_matrix4d_array(&mut self, matrices: &[[f64; 16]]) {
        self.output.push('[');
        for (i, m) in matrices.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.write_matrix4d(m);
        }
        self.output.push(']');
    }

    /// Flatten variable-length joint indices into a fixed-width array padded with 0.
    fn write_flat_joint_indices(&mut self, data: &[Vec<i32>], element_size: usize) {
        self.output.push('[');
        let mut first = true;
        for indices in data {
            for j in 0..element_size {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                let val = if j < indices.len() { indices[j] } else { 0 };
                let _ = write!(self.output, "{}", val);
            }
        }
        self.output.push(']');
    }

    /// Flatten variable-length joint weights into a fixed-width array padded with 0.0.
    fn write_flat_joint_weights(&mut self, data: &[Vec<f64>], element_size: usize) {
        self.output.push('[');
        let mut first = true;
        for weights in data {
            for j in 0..element_size {
                if !first {
                    self.output.push_str(", ");
                }
                first = false;
                let val = if j < weights.len() {
                    weights[j]
                } else {
                    0.0
                };
                let _ = write!(self.output, "{:.6}", val);
            }
        }
        self.output.push(']');
    }
}

// ── Utility ──────────────────────────────────────────────────────────────────

/// Sanitise a scene path into a valid prim name (replace `/` and spaces).
fn sanitise_name(path: &str) -> String {
    path.chars()
        .map(|c| match c {
            '/' | ' ' | '.' => '_',
            _ => c,
        })
        .collect()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_matrix() -> [f64; 16] {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]
    }

    fn sample_mesh() -> UsdMesh {
        UsdMesh {
            name: "Body".to_string(),
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
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
            uvs: vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
            ],
            face_vertex_counts: vec![3, 3],
            face_vertex_indices: vec![0, 1, 2, 0, 2, 3],
            subdivision_scheme: UsdSubdivScheme::None,
        }
    }

    fn sample_material() -> UsdMaterial {
        UsdMaterial {
            name: "Skin".to_string(),
            diffuse_color: [0.8, 0.6, 0.5],
            metallic: 0.0,
            roughness: 0.7,
            opacity: 1.0,
            normal_scale: 1.0,
        }
    }

    fn sample_skeleton() -> UsdSkeleton {
        UsdSkeleton {
            joint_names: vec!["Hips".to_string(), "Spine".to_string()],
            joint_paths: vec!["Hips".to_string(), "Hips/Spine".to_string()],
            bind_transforms: vec![identity_matrix(), identity_matrix()],
            rest_transforms: vec![identity_matrix(), identity_matrix()],
        }
    }

    // ── Header tests ─────────────────────────────────────────────────────

    #[test]
    fn test_header_contains_magic() {
        let mut w = UsdaWriter::new();
        w.write_header("Y", 1.0);
        let out = w.finish();
        assert!(out.starts_with("#usda 1.0"), "must start with #usda 1.0");
    }

    #[test]
    fn test_header_up_axis_y() {
        let mut w = UsdaWriter::new();
        w.write_header("Y", 1.0);
        let out = w.finish();
        assert!(out.contains("upAxis = \"Y\""));
    }

    #[test]
    fn test_header_up_axis_z() {
        let mut w = UsdaWriter::new();
        w.write_header("Z", 0.01);
        let out = w.finish();
        assert!(out.contains("upAxis = \"Z\""));
        assert!(out.contains("metersPerUnit = 0.010000"));
    }

    // ── Def block tests ──────────────────────────────────────────────────

    #[test]
    fn test_begin_end_def() {
        let mut w = UsdaWriter::new();
        w.begin_def("Xform", "Root");
        w.end_def();
        let out = w.finish();
        assert!(out.contains("def Xform \"Root\" {"));
        assert!(out.contains('}'));
    }

    #[test]
    fn test_nested_def() {
        let mut w = UsdaWriter::new();
        w.begin_def("Xform", "Root");
        w.begin_def("Xform", "Child");
        w.end_def();
        w.end_def();
        let out = w.finish();
        assert!(out.contains("def Xform \"Root\""));
        assert!(out.contains("    def Xform \"Child\""));
    }

    #[test]
    fn test_finish_closes_unclosed_defs() {
        let mut w = UsdaWriter::new();
        w.begin_def("Xform", "A");
        w.begin_def("Xform", "B");
        // deliberately not closing
        let out = w.finish();
        let close_count = out.matches('}').count();
        assert!(close_count >= 2, "finish must auto-close open defs");
    }

    // ── Mesh tests ───────────────────────────────────────────────────────

    #[test]
    fn test_write_mesh_contains_points() {
        let mut w = UsdaWriter::new();
        w.write_header("Y", 1.0);
        w.begin_def("Xform", "Root");
        w.write_mesh(&sample_mesh()).expect("write_mesh");
        w.end_def();
        let out = w.finish();
        assert!(out.contains("point3f[] points = "));
        assert!(out.contains("(0.000000, 0.000000, 0.000000)"));
        assert!(out.contains("(1.000000, 0.000000, 0.000000)"));
    }

    #[test]
    fn test_write_mesh_contains_normals() {
        let mut w = UsdaWriter::new();
        w.write_mesh(&sample_mesh()).expect("write_mesh");
        let out = w.finish();
        assert!(out.contains("normal3f[] normals = "));
        assert!(out.contains("interpolation = \"faceVarying\""));
    }

    #[test]
    fn test_write_mesh_contains_uvs() {
        let mut w = UsdaWriter::new();
        w.write_mesh(&sample_mesh()).expect("write_mesh");
        let out = w.finish();
        assert!(out.contains("texCoord2f[] primvars:st = "));
    }

    #[test]
    fn test_write_mesh_contains_face_data() {
        let mut w = UsdaWriter::new();
        w.write_mesh(&sample_mesh()).expect("write_mesh");
        let out = w.finish();
        assert!(out.contains("int[] faceVertexCounts = [3, 3]"));
        assert!(out.contains("int[] faceVertexIndices = [0, 1, 2, 0, 2, 3]"));
    }

    #[test]
    fn test_write_mesh_subdivision_scheme() {
        let mut mesh = sample_mesh();
        mesh.subdivision_scheme = UsdSubdivScheme::CatmullClark;
        let mut w = UsdaWriter::new();
        w.write_mesh(&mesh).expect("write_mesh");
        let out = w.finish();
        assert!(out.contains("subdivisionScheme = \"catmullClark\""));
    }

    #[test]
    fn test_write_mesh_no_normals() {
        let mut mesh = sample_mesh();
        mesh.normals.clear();
        let mut w = UsdaWriter::new();
        w.write_mesh(&mesh).expect("write_mesh");
        let out = w.finish();
        assert!(
            !out.contains("normal3f[]"),
            "no normals section if empty"
        );
    }

    #[test]
    fn test_write_mesh_no_uvs() {
        let mut mesh = sample_mesh();
        mesh.uvs.clear();
        let mut w = UsdaWriter::new();
        w.write_mesh(&mesh).expect("write_mesh");
        let out = w.finish();
        assert!(
            !out.contains("texCoord2f[]"),
            "no UVs section if empty"
        );
    }

    // ── Material tests ───────────────────────────────────────────────────

    #[test]
    fn test_write_material_basic() {
        let mut w = UsdaWriter::new();
        w.write_material(&sample_material()).expect("write_material");
        let out = w.finish();
        assert!(out.contains("def Material \"Skin\""));
        assert!(out.contains("UsdPreviewSurface"));
        assert!(out.contains("diffuseColor"));
        assert!(out.contains("metallic"));
        assert!(out.contains("roughness"));
        assert!(out.contains("opacity"));
    }

    #[test]
    fn test_write_material_diffuse_values() {
        let mut w = UsdaWriter::new();
        w.write_material(&sample_material()).expect("write_material");
        let out = w.finish();
        assert!(out.contains("0.800000"));
        assert!(out.contains("0.600000"));
        assert!(out.contains("0.500000"));
    }

    #[test]
    fn test_write_material_surface_output() {
        let mut w = UsdaWriter::new();
        w.write_material(&sample_material()).expect("write_material");
        let out = w.finish();
        assert!(out.contains("outputs:surface"));
    }

    // ── Skeleton tests ───────────────────────────────────────────────────

    #[test]
    fn test_write_skeleton_basic() {
        let mut w = UsdaWriter::new();
        w.write_skeleton(&sample_skeleton()).expect("write_skeleton");
        let out = w.finish();
        assert!(out.contains("def Skeleton \"Skeleton\""));
        assert!(out.contains("uniform token[] joints"));
        assert!(out.contains("\"Hips\""));
        assert!(out.contains("\"Hips/Spine\""));
    }

    #[test]
    fn test_write_skeleton_transforms() {
        let mut w = UsdaWriter::new();
        w.write_skeleton(&sample_skeleton()).expect("write_skeleton");
        let out = w.finish();
        assert!(out.contains("matrix4d[] bindTransforms"));
        assert!(out.contains("matrix4d[] restTransforms"));
        // Check identity matrix values
        assert!(out.contains("1.000000"));
    }

    #[test]
    fn test_write_skeleton_joint_names() {
        let mut w = UsdaWriter::new();
        w.write_skeleton(&sample_skeleton()).expect("write_skeleton");
        let out = w.finish();
        assert!(out.contains("jointNames"));
        assert!(out.contains("\"Hips\""));
        assert!(out.contains("\"Spine\""));
    }

    // ── Skin binding tests ───────────────────────────────────────────────

    #[test]
    fn test_write_skin_binding_basic() {
        let binding = UsdSkinBinding {
            joint_indices: vec![vec![0, 1], vec![0], vec![1, 0]],
            joint_weights: vec![vec![0.7, 0.3], vec![1.0], vec![0.5, 0.5]],
            skeleton_path: "/Root/Skeleton".to_string(),
        };
        let mut w = UsdaWriter::new();
        w.write_skin_binding("/Root/Body", &binding)
            .expect("write_skin_binding");
        let out = w.finish();
        assert!(out.contains("primvars:skel:skeleton"));
        assert!(out.contains("/Root/Skeleton"));
    }

    #[test]
    fn test_write_skin_binding_joint_indices_flattened() {
        let binding = UsdSkinBinding {
            joint_indices: vec![vec![0, 1], vec![0]],
            joint_weights: vec![vec![0.7, 0.3], vec![1.0]],
            skeleton_path: "/Root/Skeleton".to_string(),
        };
        let mut w = UsdaWriter::new();
        w.write_skin_binding("/Root/Body", &binding)
            .expect("write_skin_binding");
        let out = w.finish();
        // Second vertex has only one index, should be padded with 0
        assert!(out.contains("primvars:skel:jointIndices"));
        assert!(out.contains("elementSize = 2"));
    }

    #[test]
    fn test_write_skin_binding_weights_flattened() {
        let binding = UsdSkinBinding {
            joint_indices: vec![vec![0, 1], vec![0]],
            joint_weights: vec![vec![0.7, 0.3], vec![1.0]],
            skeleton_path: "/Skel".to_string(),
        };
        let mut w = UsdaWriter::new();
        w.write_skin_binding("/Mesh", &binding)
            .expect("write_skin_binding");
        let out = w.finish();
        assert!(out.contains("primvars:skel:jointWeights"));
        assert!(out.contains("0.700000"));
        assert!(out.contains("0.300000"));
        assert!(out.contains("1.000000"));
    }

    #[test]
    fn test_write_skin_binding_empty() {
        let binding = UsdSkinBinding {
            joint_indices: vec![],
            joint_weights: vec![],
            skeleton_path: "/Skel".to_string(),
        };
        let mut w = UsdaWriter::new();
        w.write_skin_binding("/Mesh", &binding)
            .expect("write_skin_binding");
        let out = w.finish();
        // No jointIndices or jointWeights if empty
        assert!(!out.contains("primvars:skel:jointIndices"));
    }

    // ── Blend shape tests ────────────────────────────────────────────────

    #[test]
    fn test_write_blend_shapes_basic() {
        let shapes = vec![
            UsdBlendShape {
                name: "Smile".to_string(),
                offsets: vec![[0.1, 0.2, 0.0], [0.05, 0.1, 0.0]],
                point_indices: vec![10, 11],
            },
            UsdBlendShape {
                name: "Frown".to_string(),
                offsets: vec![[-0.1, -0.2, 0.0]],
                point_indices: vec![10],
            },
        ];
        let mut w = UsdaWriter::new();
        w.write_blend_shapes("/Root/Body", &shapes)
            .expect("write_blend_shapes");
        let out = w.finish();
        assert!(out.contains("def BlendShape \"Smile\""));
        assert!(out.contains("def BlendShape \"Frown\""));
        assert!(out.contains("vector3f[] offsets"));
        assert!(out.contains("int[] pointIndices"));
    }

    #[test]
    fn test_write_blend_shapes_names_array() {
        let shapes = vec![UsdBlendShape {
            name: "Open".to_string(),
            offsets: vec![[0.0, 0.1, 0.0]],
            point_indices: vec![5],
        }];
        let mut w = UsdaWriter::new();
        w.write_blend_shapes("/Mesh", &shapes)
            .expect("write_blend_shapes");
        let out = w.finish();
        assert!(out.contains("blendShapes = [\"Open\"]"));
    }

    #[test]
    fn test_write_blend_shapes_targets_rel() {
        let shapes = vec![
            UsdBlendShape {
                name: "A".to_string(),
                offsets: vec![[1.0, 0.0, 0.0]],
                point_indices: vec![0],
            },
            UsdBlendShape {
                name: "B".to_string(),
                offsets: vec![[0.0, 1.0, 0.0]],
                point_indices: vec![1],
            },
        ];
        let mut w = UsdaWriter::new();
        w.write_blend_shapes("/M", &shapes)
            .expect("write_blend_shapes");
        let out = w.finish();
        assert!(out.contains("blendShapeTargets = [<./A>, <./B>]"));
    }

    #[test]
    fn test_write_blend_shapes_empty() {
        let mut w = UsdaWriter::new();
        w.write_blend_shapes("/Mesh", &[])
            .expect("write_blend_shapes");
        let out = w.finish();
        assert!(
            !out.contains("BlendShape"),
            "empty shapes should produce no output"
        );
    }

    // ── Xform tests ──────────────────────────────────────────────────────

    #[test]
    fn test_write_xform_basic() {
        let mut w = UsdaWriter::new();
        w.write_xform("Root", &identity_matrix())
            .expect("write_xform");
        let out = w.finish();
        assert!(out.contains("def Xform \"Root\""));
        assert!(out.contains("matrix4d xformOp:transform"));
        assert!(out.contains("xformOpOrder"));
    }

    #[test]
    fn test_write_xform_matrix_values() {
        let mat = [
            2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 1.0, 2.0, 3.0, 1.0,
        ];
        let mut w = UsdaWriter::new();
        w.write_xform("Scaled", &mat).expect("write_xform");
        let out = w.finish();
        assert!(out.contains("2.000000"));
        assert!(out.contains("3.000000"));
    }

    // ── Integration / round-trip tests ───────────────────────────────────

    #[test]
    fn test_full_scene() {
        let mut w = UsdaWriter::new();
        w.write_header("Y", 1.0);
        w.begin_def("Xform", "Root");
        w.write_mesh(&sample_mesh()).expect("write_mesh");
        w.write_material(&sample_material()).expect("write_material");
        w.end_def();
        let out = w.finish();

        assert!(out.starts_with("#usda 1.0"));
        assert!(out.contains("def Xform \"Root\""));
        assert!(out.contains("def Mesh \"Body\""));
        assert!(out.contains("def Material \"Skin\""));
    }

    #[test]
    fn test_full_scene_with_skeleton_and_skin() {
        let mut w = UsdaWriter::new();
        w.write_header("Y", 1.0);
        w.begin_def("Xform", "Root");
        w.write_mesh(&sample_mesh()).expect("write_mesh");
        w.write_skeleton(&sample_skeleton()).expect("write_skeleton");
        let binding = UsdSkinBinding {
            joint_indices: vec![vec![0], vec![0, 1], vec![1], vec![0, 1]],
            joint_weights: vec![vec![1.0], vec![0.6, 0.4], vec![1.0], vec![0.5, 0.5]],
            skeleton_path: "/Root/Skeleton".to_string(),
        };
        w.write_skin_binding("/Root/Body", &binding)
            .expect("write_skin_binding");
        w.end_def();
        let out = w.finish();

        assert!(out.contains("def Skeleton"));
        assert!(out.contains("primvars:skel:skeleton"));
        assert!(out.contains("primvars:skel:jointIndices"));
        assert!(out.contains("primvars:skel:jointWeights"));
    }

    #[test]
    fn test_full_scene_with_blend_shapes() {
        let shapes = vec![
            UsdBlendShape {
                name: "Smile".to_string(),
                offsets: vec![[0.1, 0.2, 0.0]],
                point_indices: vec![0],
            },
            UsdBlendShape {
                name: "Blink".to_string(),
                offsets: vec![[0.0, -0.1, 0.0]],
                point_indices: vec![2],
            },
        ];
        let mut w = UsdaWriter::new();
        w.write_header("Y", 1.0);
        w.begin_def("Xform", "Root");
        w.write_mesh(&sample_mesh()).expect("write_mesh");
        w.write_blend_shapes("/Root/Body", &shapes)
            .expect("write_blend_shapes");
        w.end_def();
        let out = w.finish();

        assert!(out.contains("def BlendShape \"Smile\""));
        assert!(out.contains("def BlendShape \"Blink\""));
    }

    #[test]
    fn test_subdiv_scheme_tokens() {
        assert_eq!(UsdSubdivScheme::None.as_token(), "none");
        assert_eq!(UsdSubdivScheme::CatmullClark.as_token(), "catmullClark");
        assert_eq!(UsdSubdivScheme::Loop.as_token(), "loop");
        assert_eq!(UsdSubdivScheme::Bilinear.as_token(), "bilinear");
    }

    #[test]
    fn test_sanitise_name() {
        assert_eq!(sanitise_name("/Root/Body"), "_Root_Body");
        assert_eq!(sanitise_name("hello world"), "hello_world");
        assert_eq!(sanitise_name("a.b.c"), "a_b_c");
        assert_eq!(sanitise_name("NoChange"), "NoChange");
    }

    #[test]
    fn test_writer_default() {
        let w = UsdaWriter::default();
        let out = w.finish();
        assert!(out.is_empty(), "default writer should produce empty output");
    }

    #[test]
    fn test_end_def_at_zero_indent() {
        let mut w = UsdaWriter::new();
        // Should not panic even if indent_level is already 0
        w.end_def();
        let out = w.finish();
        assert!(out.contains('}'));
    }

    #[test]
    fn test_multiple_meshes() {
        let mut w = UsdaWriter::new();
        w.write_header("Y", 1.0);
        w.begin_def("Xform", "Root");

        let mut mesh1 = sample_mesh();
        mesh1.name = "Head".to_string();
        w.write_mesh(&mesh1).expect("write_mesh head");

        let mut mesh2 = sample_mesh();
        mesh2.name = "Hand".to_string();
        w.write_mesh(&mesh2).expect("write_mesh hand");

        w.end_def();
        let out = w.finish();

        assert!(out.contains("def Mesh \"Head\""));
        assert!(out.contains("def Mesh \"Hand\""));
    }

    #[test]
    fn test_xform_with_translation() {
        let mut mat = identity_matrix();
        mat[12] = 5.0;
        mat[13] = 10.0;
        mat[14] = -3.0;
        let mut w = UsdaWriter::new();
        w.write_xform("Offset", &mat).expect("write_xform");
        let out = w.finish();
        assert!(out.contains("5.000000"));
        assert!(out.contains("10.000000"));
        assert!(out.contains("-3.000000"));
    }

    #[test]
    fn test_write_to_file() {
        let mut w = UsdaWriter::new();
        w.write_header("Y", 1.0);
        w.begin_def("Xform", "Root");
        w.write_mesh(&sample_mesh()).expect("write_mesh");
        w.end_def();
        let out = w.finish();

        let path = std::env::temp_dir().join("test_usda_export_writer.usda");
        std::fs::write(&path, &out).expect("write file");

        let read_back = std::fs::read_to_string(&path).expect("read file");
        assert_eq!(out, read_back);
        assert!(read_back.starts_with("#usda 1.0"));

        let _ = std::fs::remove_file(&path);
    }
}
