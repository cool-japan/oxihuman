// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! 3MF (3D Manufacturing Format) export.
//!
//! 3MF is an OPC/ZIP-based package containing XML files describing 3D geometry
//! for additive manufacturing (3D printing). This module produces valid 3MF
//! archives with mesh data, metadata, and build instructions.

use std::fmt;
use std::io::Cursor;

use anyhow::{bail, Context, Result};
use oxiarc_archive::zip::ZipWriter;

// ─── Public types ───────────────────────────────────────────────────────────

/// Unit of measurement for the 3MF model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreeMfUnit {
    Millimeter,
    Centimeter,
    Meter,
    Inch,
    Foot,
}

impl ThreeMfUnit {
    /// Returns the XML attribute value for this unit.
    fn as_str(self) -> &'static str {
        match self {
            Self::Millimeter => "millimeter",
            Self::Centimeter => "centimeter",
            Self::Meter => "meter",
            Self::Inch => "inch",
            Self::Foot => "foot",
        }
    }
}

impl fmt::Display for ThreeMfUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Object type within a 3MF model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreeMfObjectType {
    Model,
    Other,
    Support,
}

impl ThreeMfObjectType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Other => "other",
            Self::Support => "support",
        }
    }
}

impl fmt::Display for ThreeMfObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Triangle mesh data for a 3MF object.
#[derive(Debug, Clone)]
pub struct ThreeMfMesh {
    pub vertices: Vec<[f64; 3]>,
    pub triangles: Vec<[usize; 3]>,
}

/// A single object in the 3MF model (mesh + metadata).
#[derive(Debug, Clone)]
pub struct ThreeMfObject {
    pub id: u32,
    pub name: String,
    pub mesh: ThreeMfMesh,
    pub object_type: ThreeMfObjectType,
}

/// A build item referencing an object, optionally with an affine transform.
#[derive(Debug, Clone)]
pub struct ThreeMfBuildItem {
    pub object_id: u32,
    /// Row-major 3x4 affine matrix (m00 m01 m02 m03 m10 m11 m12 m13 m20 m21 m22 m23).
    pub transform: Option<[f64; 12]>,
}

/// Complete 3MF model representation.
#[derive(Debug, Clone)]
pub struct ThreeMfModel {
    pub unit: ThreeMfUnit,
    pub objects: Vec<ThreeMfObject>,
    pub build_items: Vec<ThreeMfBuildItem>,
    pub metadata: Vec<(String, String)>,
}

// ─── Exporter ───────────────────────────────────────────────────────────────

/// Builds and exports a 3MF (3D Manufacturing Format) archive.
///
/// # Example
///
/// ```rust
/// use oxihuman_export::three_mf_export::{ThreeMfExporter, ThreeMfUnit};
///
/// let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
/// exporter.set_metadata("Title", "Test Cube");
///
/// let vertices = vec![
///     [0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 10.0, 0.0],
///     [0.0, 10.0, 0.0], [0.0, 0.0, 10.0], [10.0, 0.0, 10.0],
///     [10.0, 10.0, 10.0], [0.0, 10.0, 10.0],
/// ];
/// let triangles = vec![
///     [0, 2, 1], [0, 3, 2], [4, 5, 6], [4, 6, 7],
///     [0, 1, 5], [0, 5, 4], [1, 2, 6], [1, 6, 5],
///     [2, 3, 7], [2, 7, 6], [3, 0, 4], [3, 4, 7],
/// ];
///
/// let obj_id = exporter.add_object("Cube", &vertices, &triangles).unwrap();
/// exporter.add_build_item(obj_id, None).unwrap();
/// let _bytes = exporter.export().unwrap();
/// ```
pub struct ThreeMfExporter {
    model: ThreeMfModel,
    next_id: u32,
}

impl ThreeMfExporter {
    /// Create a new exporter with the given measurement unit.
    pub fn new(unit: ThreeMfUnit) -> Self {
        Self {
            model: ThreeMfModel {
                unit,
                objects: Vec::new(),
                build_items: Vec::new(),
                metadata: Vec::new(),
            },
            next_id: 1,
        }
    }

    /// Add a mesh object and return its id.
    ///
    /// Validates that vertex indices in `triangles` are within bounds and that
    /// at least one vertex and one triangle are provided.
    pub fn add_object(
        &mut self,
        name: &str,
        vertices: &[[f64; 3]],
        triangles: &[[usize; 3]],
    ) -> Result<u32> {
        if vertices.is_empty() {
            bail!("3MF object must have at least one vertex");
        }
        if triangles.is_empty() {
            bail!("3MF object must have at least one triangle");
        }

        // Validate vertex coordinates are finite
        for (vi, v) in vertices.iter().enumerate() {
            for (ci, &c) in v.iter().enumerate() {
                if !c.is_finite() {
                    bail!(
                        "vertex {vi} coordinate {ci} is not finite: {c}"
                    );
                }
            }
        }

        // Validate triangle indices
        let n = vertices.len();
        for (ti, tri) in triangles.iter().enumerate() {
            for &idx in tri {
                if idx >= n {
                    bail!(
                        "triangle {ti} references vertex index {idx} but only {n} vertices exist"
                    );
                }
            }
            // Degenerate triangle check (all three indices identical)
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                bail!(
                    "triangle {ti} is degenerate: vertices [{}, {}, {}]",
                    tri[0],
                    tri[1],
                    tri[2]
                );
            }
        }

        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .context("object id overflow")?;

        self.model.objects.push(ThreeMfObject {
            id,
            name: name.to_string(),
            mesh: ThreeMfMesh {
                vertices: vertices.to_vec(),
                triangles: triangles.to_vec(),
            },
            object_type: ThreeMfObjectType::Model,
        });

        Ok(id)
    }

    /// Add an object with a specific type (model, support, other).
    pub fn add_object_with_type(
        &mut self,
        name: &str,
        vertices: &[[f64; 3]],
        triangles: &[[usize; 3]],
        object_type: ThreeMfObjectType,
    ) -> Result<u32> {
        let id = self.add_object(name, vertices, triangles)?;
        // Patch the type on the last-added object
        if let Some(obj) = self.model.objects.last_mut() {
            obj.object_type = object_type;
        }
        Ok(id)
    }

    /// Add a build item referencing an existing object.
    pub fn add_build_item(
        &mut self,
        object_id: u32,
        transform: Option<[f64; 12]>,
    ) -> Result<()> {
        // Validate the referenced object exists
        if !self.model.objects.iter().any(|o| o.id == object_id) {
            bail!("build item references unknown object id {object_id}");
        }

        // Validate transform values are finite
        if let Some(ref t) = transform {
            for (i, &v) in t.iter().enumerate() {
                if !v.is_finite() {
                    bail!("transform element {i} is not finite: {v}");
                }
            }
        }

        self.model.build_items.push(ThreeMfBuildItem {
            object_id,
            transform,
        });
        Ok(())
    }

    /// Set or replace a metadata entry (e.g. "Title", "Designer", "Description").
    pub fn set_metadata(&mut self, key: &str, value: &str) {
        // Replace existing entry with same key, or add new
        for entry in &mut self.model.metadata {
            if entry.0 == key {
                entry.1 = value.to_string();
                return;
            }
        }
        self.model.metadata.push((key.to_string(), value.to_string()));
    }

    /// Return a reference to the internal model.
    pub fn model(&self) -> &ThreeMfModel {
        &self.model
    }

    /// Export the model as a 3MF archive (ZIP bytes).
    pub fn export(&self) -> Result<Vec<u8>> {
        if self.model.objects.is_empty() {
            bail!("3MF model has no objects to export");
        }
        if self.model.build_items.is_empty() {
            bail!("3MF model has no build items");
        }

        let content_types_xml = build_content_types_xml();
        let rels_xml = build_rels_xml();
        let model_xml = build_model_xml(&self.model)?;

        let buf: Vec<u8> = Vec::new();
        let cursor = Cursor::new(buf);
        let mut zip = ZipWriter::new(cursor);

        zip.add_file("[Content_Types].xml", content_types_xml.as_bytes())
            .context("failed to write [Content_Types].xml")?;

        // The _rels directory entry is implicit; we just write the file
        zip.add_file("_rels/.rels", rels_xml.as_bytes())
            .context("failed to write _rels/.rels")?;

        zip.add_file("3D/3dmodel.model", model_xml.as_bytes())
            .context("failed to write 3D/3dmodel.model")?;

        let cursor = zip
            .into_inner()
            .context("failed to finalize ZIP archive")?;

        Ok(cursor.into_inner())
    }
}

// ─── XML generation helpers ─────────────────────────────────────────────────

const NAMESPACE: &str = "http://schemas.microsoft.com/3dmanufacturing/core/2015/02";

/// Build the OPC `[Content_Types].xml` for 3MF.
fn build_content_types_xml() -> String {
    let mut xml = String::with_capacity(512);
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n");
    xml.push_str("  <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\" />\n");
    xml.push_str("  <Default Extension=\"model\" ContentType=\"application/vnd.ms-package.3dmanufacturing-3dmodel+xml\" />\n");
    xml.push_str("</Types>\n");
    xml
}

/// Build the OPC `_rels/.rels` relationships file.
fn build_rels_xml() -> String {
    let mut xml = String::with_capacity(512);
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n");
    xml.push_str("  <Relationship Target=\"/3D/3dmodel.model\" Id=\"rel0\" Type=\"http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel\" />\n");
    xml.push_str("</Relationships>\n");
    xml
}

/// Build the `3D/3dmodel.model` XML from the model data.
fn build_model_xml(model: &ThreeMfModel) -> Result<String> {
    // Rough estimate: 100 bytes per vertex, 80 per triangle, 200 per object header
    let estimated_size: usize = model
        .objects
        .iter()
        .map(|o| o.mesh.vertices.len() * 100 + o.mesh.triangles.len() * 80 + 200)
        .sum::<usize>()
        + 1024;

    let mut xml = String::with_capacity(estimated_size);

    // XML declaration
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

    // Model root element
    xml.push_str("<model unit=\"");
    xml.push_str(model.unit.as_str());
    xml.push_str("\" xml:lang=\"en-US\" xmlns=\"");
    xml.push_str(NAMESPACE);
    xml.push_str("\">\n");

    // Metadata
    for (key, value) in &model.metadata {
        xml.push_str("  <metadata name=\"");
        push_xml_escaped(&mut xml, key);
        xml.push_str("\">");
        push_xml_escaped(&mut xml, value);
        xml.push_str("</metadata>\n");
    }

    // Resources
    xml.push_str("  <resources>\n");
    for obj in &model.objects {
        write_object_xml(&mut xml, obj)?;
    }
    xml.push_str("  </resources>\n");

    // Build
    xml.push_str("  <build>\n");
    for item in &model.build_items {
        write_build_item_xml(&mut xml, item);
    }
    xml.push_str("  </build>\n");

    xml.push_str("</model>\n");

    Ok(xml)
}

/// Append a single `<object>` element with its mesh to the XML buffer.
fn write_object_xml(xml: &mut String, obj: &ThreeMfObject) -> Result<()> {
    xml.push_str("    <object id=\"");
    push_u32(xml, obj.id);
    xml.push_str("\" type=\"");
    xml.push_str(obj.object_type.as_str());
    xml.push_str("\" name=\"");
    push_xml_escaped(xml, &obj.name);
    xml.push_str("\">\n");

    xml.push_str("      <mesh>\n");

    // Vertices
    xml.push_str("        <vertices>\n");
    for v in &obj.mesh.vertices {
        xml.push_str("          <vertex x=\"");
        push_f64(xml, v[0]);
        xml.push_str("\" y=\"");
        push_f64(xml, v[1]);
        xml.push_str("\" z=\"");
        push_f64(xml, v[2]);
        xml.push_str("\" />\n");
    }
    xml.push_str("        </vertices>\n");

    // Triangles
    xml.push_str("        <triangles>\n");
    for tri in &obj.mesh.triangles {
        xml.push_str("          <triangle v1=\"");
        push_usize(xml, tri[0]);
        xml.push_str("\" v2=\"");
        push_usize(xml, tri[1]);
        xml.push_str("\" v3=\"");
        push_usize(xml, tri[2]);
        xml.push_str("\" />\n");
    }
    xml.push_str("        </triangles>\n");

    xml.push_str("      </mesh>\n");
    xml.push_str("    </object>\n");

    Ok(())
}

/// Append a `<item>` element to the XML buffer.
fn write_build_item_xml(xml: &mut String, item: &ThreeMfBuildItem) {
    xml.push_str("    <item objectid=\"");
    push_u32(xml, item.object_id);
    xml.push('"');

    if let Some(ref t) = item.transform {
        xml.push_str(" transform=\"");
        for (i, &v) in t.iter().enumerate() {
            if i > 0 {
                xml.push(' ');
            }
            push_f64(xml, v);
        }
        xml.push('"');
    }

    xml.push_str(" />\n");
}

// ─── Formatting helpers ─────────────────────────────────────────────────────

/// Append an XML-escaped string (handles &, <, >, ", ').
fn push_xml_escaped(buf: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '&' => buf.push_str("&amp;"),
            '<' => buf.push_str("&lt;"),
            '>' => buf.push_str("&gt;"),
            '"' => buf.push_str("&quot;"),
            '\'' => buf.push_str("&apos;"),
            _ => buf.push(ch),
        }
    }
}

/// Append an f64 in a compact representation suitable for 3MF.
///
/// Uses up to 6 decimal places, stripping trailing zeros for cleaner output.
fn push_f64(buf: &mut String, v: f64) {
    use std::fmt::Write;
    // Format with enough precision
    let mut tmp = String::with_capacity(24);
    let _ = write!(tmp, "{:.6}", v);
    // Strip trailing zeros after the decimal point
    if tmp.contains('.') {
        let trimmed = tmp.trim_end_matches('0');
        let trimmed = trimmed.trim_end_matches('.');
        buf.push_str(trimmed);
    } else {
        buf.push_str(&tmp);
    }
}

fn push_u32(buf: &mut String, v: u32) {
    use std::fmt::Write;
    let _ = write!(buf, "{v}");
}

fn push_usize(buf: &mut String, v: usize) {
    use std::fmt::Write;
    let _ = write!(buf, "{v}");
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal cube geometry for testing.
    fn cube_geometry() -> (Vec<[f64; 3]>, Vec<[usize; 3]>) {
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [10.0, 0.0, 0.0],
            [10.0, 10.0, 0.0],
            [0.0, 10.0, 0.0],
            [0.0, 0.0, 10.0],
            [10.0, 0.0, 10.0],
            [10.0, 10.0, 10.0],
            [0.0, 10.0, 10.0],
        ];
        let triangles = vec![
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 1, 5],
            [0, 5, 4],
            [1, 2, 6],
            [1, 6, 5],
            [2, 3, 7],
            [2, 7, 6],
            [3, 0, 4],
            [3, 4, 7],
        ];
        (vertices, triangles)
    }

    #[test]
    fn test_export_basic_cube() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        let (v, t) = cube_geometry();
        let id = exporter.add_object("Cube", &v, &t).expect("should succeed");
        exporter.add_build_item(id, None).expect("should succeed");
        let bytes = exporter.export().expect("should succeed");

        // Should start with PK (ZIP magic)
        assert!(bytes.len() > 4);
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_export_with_metadata() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Inch);
        exporter.set_metadata("Title", "My Model");
        exporter.set_metadata("Designer", "OxiHuman");

        let (v, t) = cube_geometry();
        let id = exporter.add_object("Body", &v, &t).expect("should succeed");
        exporter.add_build_item(id, None).expect("should succeed");

        let bytes = exporter.export().expect("should succeed");
        assert!(!bytes.is_empty());

        // Verify we can read the ZIP and find the model XML
        let cursor = Cursor::new(&bytes);
        let mut reader =
            oxiarc_archive::zip::ZipReader::new(cursor).expect("should succeed");

        let model_entry = reader
            .entry_by_name("3D/3dmodel.model")
            .cloned()
            .expect("missing 3dmodel.model");
        let model_data = reader.extract(&model_entry).expect("should succeed");
        let model_str = std::str::from_utf8(&model_data).expect("should succeed");

        assert!(model_str.contains("unit=\"inch\""));
        assert!(model_str.contains("My Model"));
        assert!(model_str.contains("OxiHuman"));
    }

    #[test]
    fn test_export_with_transform() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Meter);
        let (v, t) = cube_geometry();
        let id = exporter.add_object("Scaled", &v, &t).expect("should succeed");

        // Identity + translation (0,0,5)
        let transform = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 5.0];
        exporter.add_build_item(id, Some(transform)).expect("should succeed");

        let bytes = exporter.export().expect("should succeed");
        let cursor = Cursor::new(&bytes);
        let mut reader =
            oxiarc_archive::zip::ZipReader::new(cursor).expect("should succeed");

        let model_entry = reader
            .entry_by_name("3D/3dmodel.model")
            .cloned()
            .expect("missing 3dmodel.model");
        let model_data = reader.extract(&model_entry).expect("should succeed");
        let model_str = std::str::from_utf8(&model_data).expect("should succeed");

        assert!(model_str.contains("transform=\""));
        assert!(model_str.contains("unit=\"meter\""));
    }

    #[test]
    fn test_export_multiple_objects() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Centimeter);
        let (v, t) = cube_geometry();

        let id1 = exporter.add_object("Object1", &v, &t).expect("should succeed");
        let id2 = exporter.add_object("Object2", &v, &t).expect("should succeed");
        assert_ne!(id1, id2);

        exporter.add_build_item(id1, None).expect("should succeed");
        exporter.add_build_item(id2, None).expect("should succeed");

        let bytes = exporter.export().expect("should succeed");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_empty_model_fails() {
        let exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        assert!(exporter.export().is_err());
    }

    #[test]
    fn test_no_build_items_fails() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        let (v, t) = cube_geometry();
        let _id = exporter.add_object("Thing", &v, &t).expect("should succeed");
        assert!(exporter.export().is_err());
    }

    #[test]
    fn test_invalid_triangle_index() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let triangles = vec![[0, 1, 99]]; // 99 is out of bounds
        let result = exporter.add_object("Bad", &vertices, &triangles);
        assert!(result.is_err());
    }

    #[test]
    fn test_degenerate_triangle() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        let vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let triangles = vec![[0, 0, 1]]; // degenerate
        let result = exporter.add_object("Bad", &vertices, &triangles);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonfinite_vertex() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        let vertices = vec![
            [f64::NAN, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let triangles = vec![[0, 1, 2]];
        let result = exporter.add_object("Bad", &vertices, &triangles);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_item_unknown_object() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        let result = exporter.add_build_item(999, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_metadata_replace() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        exporter.set_metadata("Title", "First");
        exporter.set_metadata("Title", "Second");
        assert_eq!(exporter.model().metadata.len(), 1);
        assert_eq!(exporter.model().metadata[0].1, "Second");
    }

    #[test]
    fn test_xml_escaping() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        exporter.set_metadata("Title", "A & B <C> \"D\"");

        let (v, t) = cube_geometry();
        let id = exporter.add_object("Obj", &v, &t).expect("should succeed");
        exporter.add_build_item(id, None).expect("should succeed");

        let bytes = exporter.export().expect("should succeed");
        let cursor = Cursor::new(&bytes);
        let mut reader =
            oxiarc_archive::zip::ZipReader::new(cursor).expect("should succeed");

        let model_entry = reader
            .entry_by_name("3D/3dmodel.model")
            .cloned()
            .expect("missing 3dmodel.model");
        let model_data = reader.extract(&model_entry).expect("should succeed");
        let model_str = std::str::from_utf8(&model_data).expect("should succeed");

        assert!(model_str.contains("A &amp; B &lt;C&gt; &quot;D&quot;"));
    }

    #[test]
    fn test_content_types_present() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        let (v, t) = cube_geometry();
        let id = exporter.add_object("X", &v, &t).expect("should succeed");
        exporter.add_build_item(id, None).expect("should succeed");

        let bytes = exporter.export().expect("should succeed");
        let cursor = Cursor::new(&bytes);
        let reader =
            oxiarc_archive::zip::ZipReader::new(cursor).expect("should succeed");

        let names: Vec<&str> = reader.entries().iter().map(|e| e.filename.as_str()).collect();
        assert!(names.contains(&"[Content_Types].xml"));
        assert!(names.contains(&"_rels/.rels"));
        assert!(names.contains(&"3D/3dmodel.model"));
    }

    #[test]
    fn test_support_object_type() {
        let mut exporter = ThreeMfExporter::new(ThreeMfUnit::Millimeter);
        let (v, t) = cube_geometry();
        let id = exporter
            .add_object_with_type("SupportBlock", &v, &t, ThreeMfObjectType::Support)
            .expect("should succeed");
        exporter.add_build_item(id, None).expect("should succeed");

        let bytes = exporter.export().expect("should succeed");
        let cursor = Cursor::new(&bytes);
        let mut reader =
            oxiarc_archive::zip::ZipReader::new(cursor).expect("should succeed");

        let model_entry = reader
            .entry_by_name("3D/3dmodel.model")
            .cloned()
            .expect("missing 3dmodel.model");
        let model_data = reader.extract(&model_entry).expect("should succeed");
        let model_str = std::str::from_utf8(&model_data).expect("should succeed");

        assert!(model_str.contains("type=\"support\""));
    }

    #[test]
    fn test_all_units() {
        for unit in [
            ThreeMfUnit::Millimeter,
            ThreeMfUnit::Centimeter,
            ThreeMfUnit::Meter,
            ThreeMfUnit::Inch,
            ThreeMfUnit::Foot,
        ] {
            let mut exporter = ThreeMfExporter::new(unit);
            let (v, t) = cube_geometry();
            let id = exporter.add_object("U", &v, &t).expect("should succeed");
            exporter.add_build_item(id, None).expect("should succeed");
            let bytes = exporter.export().expect("should succeed");
            assert!(!bytes.is_empty(), "failed for unit {unit}");
        }
    }

    #[test]
    fn test_f64_formatting() {
        let mut buf = String::new();
        push_f64(&mut buf, 1.0);
        assert_eq!(buf, "1");

        buf.clear();
        push_f64(&mut buf, 1.5);
        assert_eq!(buf, "1.5");

        buf.clear();
        push_f64(&mut buf, 0.123456);
        assert_eq!(buf, "0.123456");

        buf.clear();
        push_f64(&mut buf, -3.14);
        assert_eq!(buf, "-3.14");
    }
}
