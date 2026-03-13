// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FBX ASCII format writer (FBX 7.4 compatible).
//!
//! Produces human-readable FBX files with geometry, normals, UVs, and
//! optional skeletal (skin/bone) data. The output follows the Autodesk
//! ASCII FBX conventions closely enough to be loaded by most DCC tools
//! that accept FBX 7.4 ASCII.

use std::fmt::Write as FmtWrite;

/// Counter-based unique ID generator for FBX objects.
struct IdGen {
    next: i64,
}

impl IdGen {
    fn new(start: i64) -> Self {
        Self { next: start }
    }

    fn take(&mut self) -> i64 {
        let id = self.next;
        self.next += 1;
        id
    }
}

// ── Writer ───────────────────────────────────────────────────────────────────

/// Writes FBX data in the human-readable ASCII format (version 7.4).
pub struct FbxAsciiWriter {
    output: Vec<u8>,
    indent_level: usize,
}

impl Default for FbxAsciiWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl FbxAsciiWriter {
    /// Creates a new writer with the standard FBX 7.4 ASCII header already
    /// emitted.
    pub fn new() -> Self {
        let mut w = Self {
            output: Vec::with_capacity(64 * 1024),
            indent_level: 0,
        };
        // Write the file-level header comments and FBXHeaderExtension.
        w.push_line("; FBX 7.4.0 project file");
        w.push_line("; Creator: OxiHuman FBX ASCII Exporter");
        w.push_line("; -------------------------------------------");
        w.push_line("");
        w.open_section("FBXHeaderExtension");
        w.push_property_i32("FBXHeaderVersion", 1003);
        w.push_property_i32("FBXVersion", 7400);
        w.push_property_i32("EncryptionType", 0);
        w.open_section("CreationTimeStamp");
        w.push_property_i32("Version", 1000);
        w.push_property_i32("Year", 2026);
        w.push_property_i32("Month", 3);
        w.push_property_i32("Day", 11);
        w.close_section();
        w.push_kv_string("Creator", "OxiHuman FBX ASCII Exporter");
        w.close_section();
        w.push_line("");
        // GlobalSettings
        w.open_section("GlobalSettings");
        w.push_property_i32("Version", 1000);
        w.open_section("Properties70");
        w.push_p70_int("UpAxis", 1);
        w.push_p70_int("UpAxisSign", 1);
        w.push_p70_int("FrontAxis", 2);
        w.push_p70_int("FrontAxisSign", 1);
        w.push_p70_int("CoordAxis", 0);
        w.push_p70_int("CoordAxisSign", 1);
        w.push_p70_double("UnitScaleFactor", 1.0);
        w.close_section();
        w.close_section();
        w.push_line("");
        w
    }

    // ── High-level API ──────────────────────────────────────────────────

    /// Writes a complete mesh object block including geometry, model, and
    /// connections. Optionally includes skin deformer and bone data when
    /// `bone_weights` / `bone_names` are provided.
    pub fn write_mesh(
        &mut self,
        name: &str,
        positions: &[[f64; 3]],
        normals: &[[f64; 3]],
        uvs: &[[f64; 2]],
        triangles: &[[usize; 3]],
        bone_weights: Option<&[Vec<(usize, f64)>]>,
        bone_names: Option<&[String]>,
    ) -> anyhow::Result<()> {
        let mut ids = IdGen::new(100_000_000);

        let geometry_id = ids.take();
        let model_id = ids.take();

        // ── Objects section ─────────────────────────────────────────────
        self.open_section("Objects");

        // Geometry node
        self.write_geometry_block(
            geometry_id,
            name,
            positions,
            normals,
            uvs,
            triangles,
        )?;

        // Model node
        self.write_model_block(model_id, name)?;

        // Skeleton / skin deformer (optional)
        let mut connection_pairs: Vec<(i64, i64)> = Vec::new();
        connection_pairs.push((geometry_id, model_id));
        connection_pairs.push((model_id, 0)); // model -> root

        if let (Some(weights), Some(names)) = (bone_weights, bone_names) {
            self.write_skin_section(
                &mut ids,
                geometry_id,
                positions,
                weights,
                names,
                &mut connection_pairs,
            )?;
        }

        self.close_section(); // Objects
        self.push_line("");

        // ── Connections section ─────────────────────────────────────────
        self.open_section("Connections");
        for (child, parent) in &connection_pairs {
            self.push_connection(*child, *parent);
        }
        self.close_section();
        self.push_line("");

        Ok(())
    }

    /// Consumes the writer and returns the finished FBX ASCII bytes.
    pub fn finish(self) -> Vec<u8> {
        self.output
    }

    // ── Geometry block ──────────────────────────────────────────────────

    fn write_geometry_block(
        &mut self,
        geometry_id: i64,
        name: &str,
        positions: &[[f64; 3]],
        normals: &[[f64; 3]],
        uvs: &[[f64; 2]],
        triangles: &[[usize; 3]],
    ) -> anyhow::Result<()> {
        self.push_line(&format!(
            "Geometry: {geometry_id}, \"Geometry::{name}\", \"Mesh\" {{"
        ));
        self.indent_level += 1;

        // Vertices
        let vert_count = positions.len() * 3;
        self.push_line(&format!("Vertices: *{vert_count} {{"));
        self.indent_level += 1;
        self.push_line(&format!("a: {}", format_f64_triples(positions)));
        self.indent_level -= 1;
        self.push_line("}");

        // PolygonVertexIndex  (FBX encodes last index of each polygon as -(idx+1))
        let idx_count = triangles.len() * 3;
        self.push_line(&format!("PolygonVertexIndex: *{idx_count} {{"));
        self.indent_level += 1;
        self.push_line(&format!("a: {}", format_triangle_indices(triangles)));
        self.indent_level -= 1;
        self.push_line("}");

        // LayerElementNormal
        if !normals.is_empty() {
            self.write_layer_element_normal(normals)?;
        }

        // LayerElementUV
        if !uvs.is_empty() {
            self.write_layer_element_uv(uvs)?;
        }

        // Layer block
        self.write_layer_block(!normals.is_empty(), !uvs.is_empty());

        self.indent_level -= 1;
        self.push_line("}");
        Ok(())
    }

    fn write_layer_element_normal(&mut self, normals: &[[f64; 3]]) -> anyhow::Result<()> {
        self.open_section("LayerElementNormal: 0");
        self.push_property_i32("Version", 101);
        self.push_kv_string("Name", "Normals");
        self.push_kv_string("MappingInformationType", "ByVertice");
        self.push_kv_string("ReferenceInformationType", "Direct");
        let count = normals.len() * 3;
        self.push_line(&format!("Normals: *{count} {{"));
        self.indent_level += 1;
        self.push_line(&format!("a: {}", format_f64_triples(normals)));
        self.indent_level -= 1;
        self.push_line("}");
        self.close_section();
        Ok(())
    }

    fn write_layer_element_uv(&mut self, uvs: &[[f64; 2]]) -> anyhow::Result<()> {
        self.open_section("LayerElementUV: 0");
        self.push_property_i32("Version", 101);
        self.push_kv_string("Name", "UVMap");
        self.push_kv_string("MappingInformationType", "ByVertice");
        self.push_kv_string("ReferenceInformationType", "Direct");
        let count = uvs.len() * 2;
        self.push_line(&format!("UV: *{count} {{"));
        self.indent_level += 1;
        self.push_line(&format!("a: {}", format_f64_pairs(uvs)));
        self.indent_level -= 1;
        self.push_line("}");
        self.close_section();
        Ok(())
    }

    fn write_layer_block(&mut self, has_normals: bool, has_uvs: bool) {
        self.open_section("Layer: 0");
        self.push_property_i32("Version", 100);
        if has_normals {
            self.open_section("LayerElement");
            self.push_kv_string("Type", "LayerElementNormal");
            self.push_property_i32("TypedIndex", 0);
            self.close_section();
        }
        if has_uvs {
            self.open_section("LayerElement");
            self.push_kv_string("Type", "LayerElementUV");
            self.push_property_i32("TypedIndex", 0);
            self.close_section();
        }
        self.close_section();
    }

    // ── Model block ─────────────────────────────────────────────────────

    fn write_model_block(&mut self, model_id: i64, name: &str) -> anyhow::Result<()> {
        self.push_line(&format!(
            "Model: {model_id}, \"Model::{name}\", \"Mesh\" {{"
        ));
        self.indent_level += 1;
        self.push_property_i32("Version", 232);
        self.open_section("Properties70");
        self.push_p70_vec3d("Lcl Translation", 0.0, 0.0, 0.0);
        self.push_p70_vec3d("Lcl Rotation", 0.0, 0.0, 0.0);
        self.push_p70_vec3d("Lcl Scaling", 1.0, 1.0, 1.0);
        self.close_section();
        self.indent_level -= 1;
        self.push_line("}");
        Ok(())
    }

    // ── Skin / bones section ────────────────────────────────────────────

    fn write_skin_section(
        &mut self,
        ids: &mut IdGen,
        geometry_id: i64,
        positions: &[[f64; 3]],
        weights: &[Vec<(usize, f64)>],
        bone_names: &[String],
        connections: &mut Vec<(i64, i64)>,
    ) -> anyhow::Result<()> {
        let deformer_id = ids.take();

        // Deformer (Skin)
        self.push_line(&format!(
            "Deformer: {deformer_id}, \"Deformer::Skin\", \"Skin\" {{"
        ));
        self.indent_level += 1;
        self.push_property_i32("Version", 101);
        self.push_property_f64("Link_DeformAcuracy", 50.0);
        self.indent_level -= 1;
        self.push_line("}");
        connections.push((deformer_id, geometry_id));

        // One SubDeformer (Cluster) per bone
        for (bone_idx, bone_name) in bone_names.iter().enumerate() {
            let cluster_id = ids.take();
            let limb_id = ids.take();

            // Collect vertex indices and weight values for this bone
            let mut indices_buf: Vec<i32> = Vec::new();
            let mut weights_buf: Vec<f64> = Vec::new();
            for (vi, vertex_weights) in weights.iter().enumerate() {
                for &(bi, w) in vertex_weights {
                    if bi == bone_idx && w.abs() > 1e-12 {
                        if let Ok(vi32) = i32::try_from(vi) {
                            indices_buf.push(vi32);
                            weights_buf.push(w);
                        }
                    }
                }
            }

            // SubDeformer (Cluster)
            self.push_line(&format!(
                "Deformer: {cluster_id}, \"SubDeformer::{bone_name}\", \"Cluster\" {{"
            ));
            self.indent_level += 1;
            self.push_property_i32("Version", 100);

            if !indices_buf.is_empty() {
                let n = indices_buf.len();
                self.push_line(&format!("Indexes: *{n} {{"));
                self.indent_level += 1;
                self.push_line(&format!("a: {}", format_i32_slice(&indices_buf)));
                self.indent_level -= 1;
                self.push_line("}");

                self.push_line(&format!("Weights: *{n} {{"));
                self.indent_level += 1;
                self.push_line(&format!("a: {}", format_f64_slice(&weights_buf)));
                self.indent_level -= 1;
                self.push_line("}");
            }

            // Transform (identity for bind pose — callers can adjust externally)
            self.write_identity_transform("Transform", positions.len())?;
            self.write_identity_transform("TransformLink", positions.len())?;

            self.indent_level -= 1;
            self.push_line("}");

            // NodeAttribute (LimbNode) for bone
            self.push_line(&format!(
                "NodeAttribute: {limb_id}, \"NodeAttribute::{bone_name}\", \"LimbNode\" {{"
            ));
            self.indent_level += 1;
            self.push_kv_string("TypeFlags", "Skeleton");
            self.indent_level -= 1;
            self.push_line("}");

            connections.push((cluster_id, deformer_id));
            connections.push((limb_id, cluster_id));
        }

        Ok(())
    }

    fn write_identity_transform(&mut self, label: &str, _n: usize) -> anyhow::Result<()> {
        self.push_line(&format!("{label}: *16 {{"));
        self.indent_level += 1;
        self.push_line("a: 1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1");
        self.indent_level -= 1;
        self.push_line("}");
        Ok(())
    }

    // ── Low-level helpers ───────────────────────────────────────────────

    fn indent(&self) -> String {
        "\t".repeat(self.indent_level)
    }

    fn push_line(&mut self, text: &str) {
        let ind = self.indent();
        let _ = write!(
            StringAdapter(&mut self.output),
            "{ind}{text}\n"
        );
    }

    fn open_section(&mut self, name: &str) {
        self.push_line(&format!("{name}: {{"));
        self.indent_level += 1;
    }

    fn close_section(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
        self.push_line("}");
    }

    fn push_property_i32(&mut self, key: &str, val: i32) {
        self.push_line(&format!("{key}: {val}"));
    }

    fn push_property_f64(&mut self, key: &str, val: f64) {
        self.push_line(&format!("{key}: {val}"));
    }

    fn push_kv_string(&mut self, key: &str, val: &str) {
        self.push_line(&format!("{key}: \"{val}\""));
    }

    fn push_p70_int(&mut self, name: &str, val: i32) {
        self.push_line(&format!(
            "P: \"{name}\", \"int\", \"Integer\", \"\", {val}"
        ));
    }

    fn push_p70_double(&mut self, name: &str, val: f64) {
        self.push_line(&format!(
            "P: \"{name}\", \"double\", \"Number\", \"\", {val}"
        ));
    }

    fn push_p70_vec3d(&mut self, name: &str, x: f64, y: f64, z: f64) {
        self.push_line(&format!(
            "P: \"{name}\", \"Vector3D\", \"Vector\", \"\", {x},{y},{z}"
        ));
    }

    fn push_connection(&mut self, child: i64, parent: i64) {
        self.push_line(&format!("C: \"OO\", {child}, {parent}"));
    }
}

// ── Adapter for `write!` into Vec<u8> ───────────────────────────────────────

struct StringAdapter<'a>(&'a mut Vec<u8>);

impl<'a> std::fmt::Write for StringAdapter<'a> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

// ── Formatting helpers ──────────────────────────────────────────────────────

fn format_f64_triples(data: &[[f64; 3]]) -> String {
    let mut buf = String::with_capacity(data.len() * 30);
    for (i, v) in data.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{},{},{}", v[0], v[1], v[2]);
    }
    buf
}

fn format_f64_pairs(data: &[[f64; 2]]) -> String {
    let mut buf = String::with_capacity(data.len() * 20);
    for (i, v) in data.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{},{}", v[0], v[1]);
    }
    buf
}

fn format_triangle_indices(triangles: &[[usize; 3]]) -> String {
    let mut buf = String::with_capacity(triangles.len() * 15);
    for (i, tri) in triangles.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        // FBX convention: last index of each polygon is bitwise-negated (-(idx+1))
        let last = -(tri[2] as i64) - 1;
        let _ = write!(buf, "{},{},{last}", tri[0], tri[1]);
    }
    buf
}

fn format_i32_slice(data: &[i32]) -> String {
    let mut buf = String::with_capacity(data.len() * 6);
    for (i, v) in data.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{v}");
    }
    buf
}

fn format_f64_slice(data: &[f64]) -> String {
    let mut buf = String::with_capacity(data.len() * 12);
    for (i, v) in data.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        let _ = write!(buf, "{v}");
    }
    buf
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_writer_has_header() {
        let w = FbxAsciiWriter::new();
        let bytes = w.finish();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("FBX 7.4.0"));
        assert!(text.contains("FBXHeaderVersion: 1003"));
        assert!(text.contains("FBXVersion: 7400"));
    }

    #[test]
    fn test_write_mesh_basic() {
        let mut w = FbxAsciiWriter::new();
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let normals = vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ];
        let uvs = vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [0.0, 1.0],
        ];
        let triangles = vec![[0, 1, 2]];

        let result = w.write_mesh("TestMesh", &positions, &normals, &uvs, &triangles, None, None);
        assert!(result.is_ok());

        let text = String::from_utf8_lossy(&w.finish());
        assert!(text.contains("Geometry::TestMesh"));
        assert!(text.contains("Vertices: *9"));
        assert!(text.contains("PolygonVertexIndex: *3"));
        assert!(text.contains("LayerElementNormal"));
        assert!(text.contains("LayerElementUV"));
        assert!(text.contains("Objects:"));
        assert!(text.contains("Connections:"));
    }

    #[test]
    fn test_write_mesh_no_normals_no_uvs() {
        let mut w = FbxAsciiWriter::new();
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let triangles = vec![[0, 1, 2]];

        let result = w.write_mesh("Simple", &positions, &[], &[], &triangles, None, None);
        assert!(result.is_ok());

        let text = String::from_utf8_lossy(&w.finish());
        assert!(!text.contains("LayerElementNormal"));
        assert!(!text.contains("LayerElementUV"));
    }

    #[test]
    fn test_write_mesh_with_bones() {
        let mut w = FbxAsciiWriter::new();
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let normals = vec![[0.0, 0.0, 1.0]; 4];
        let uvs: Vec<[f64; 2]> = vec![];
        let triangles = vec![[0, 1, 2], [1, 3, 2]];

        let bone_weights = vec![
            vec![(0, 1.0)],
            vec![(0, 0.5), (1, 0.5)],
            vec![(1, 1.0)],
            vec![(1, 1.0)],
        ];
        let bone_names = vec!["Hips".to_string(), "Spine".to_string()];

        let result = w.write_mesh(
            "Skinned",
            &positions,
            &normals,
            &uvs,
            &triangles,
            Some(&bone_weights),
            Some(&bone_names),
        );
        assert!(result.is_ok());

        let text = String::from_utf8_lossy(&w.finish());
        assert!(text.contains("Deformer::Skin"));
        assert!(text.contains("SubDeformer::Hips"));
        assert!(text.contains("SubDeformer::Spine"));
        assert!(text.contains("LimbNode"));
    }

    #[test]
    fn test_format_triangle_indices_negative() {
        let tris = vec![[0, 1, 2], [3, 4, 5]];
        let s = format_triangle_indices(&tris);
        // Last index per triangle: -(2+1) = -3, -(5+1) = -6
        assert!(s.contains("-3"));
        assert!(s.contains("-6"));
    }

    #[test]
    fn test_default_trait() {
        let w = FbxAsciiWriter::default();
        let bytes = w.finish();
        assert!(!bytes.is_empty());
    }
}
