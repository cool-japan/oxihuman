//! AMF (Additive Manufacturing Format) XML export stub.
//!
//! Provides structures and functions for building AMF documents describing 3D objects
//! suitable for additive manufacturing workflows, with basic XML serialization.

/// Configuration for AMF export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AmfExportConfig {
    /// Document name embedded in the AMF header.
    pub document_name: String,
    /// Unit system, e.g. `"millimeter"`, `"inch"`.
    pub unit: String,
    /// AMF format version string.
    pub version: String,
}

/// A single AMF object (mesh).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AmfObject {
    /// Unique object identifier.
    pub id: u32,
    /// Human-readable object name.
    pub name: String,
    /// Number of vertices in the mesh.
    pub vertex_count: u32,
    /// Number of triangular faces in the mesh.
    pub face_count: u32,
    /// Optional material name assigned to this object.
    pub material_name: Option<String>,
}

/// An AMF document containing metadata and a list of objects.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AmfDocument {
    /// Export configuration.
    pub config: AmfExportConfig,
    /// Ordered list of objects in the document.
    pub objects: Vec<AmfObject>,
    /// Running counter used to assign unique object IDs.
    next_id: u32,
}

/// Returns a sensible default [`AmfExportConfig`].
#[allow(dead_code)]
pub fn default_amf_config() -> AmfExportConfig {
    AmfExportConfig {
        document_name: "AmfDocument".to_string(),
        unit: "millimeter".to_string(),
        version: "1.1".to_string(),
    }
}

/// Creates a new, empty [`AmfDocument`] using the provided configuration.
#[allow(dead_code)]
pub fn new_amf_document(cfg: &AmfExportConfig) -> AmfDocument {
    AmfDocument {
        config: cfg.clone(),
        objects: Vec::new(),
        next_id: 1,
    }
}

/// Adds a new object to the document and returns its assigned ID.
#[allow(dead_code)]
pub fn amf_add_object(
    doc: &mut AmfDocument,
    name: &str,
    vertex_count: u32,
    face_count: u32,
) -> u32 {
    let id = doc.next_id;
    doc.next_id += 1;
    doc.objects.push(AmfObject {
        id,
        name: name.to_string(),
        vertex_count,
        face_count,
        material_name: None,
    });
    id
}

/// Assigns a material name to the object with the given ID.
///
/// Does nothing if no object with that ID exists.
#[allow(dead_code)]
pub fn amf_set_material(doc: &mut AmfDocument, object_id: u32, material_name: &str) {
    for obj in &mut doc.objects {
        if obj.id == object_id {
            obj.material_name = Some(material_name.to_string());
            return;
        }
    }
}

/// Serializes the document to an AMF-style XML string.
#[allow(dead_code)]
pub fn amf_to_xml_string(doc: &AmfDocument) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str(&format!(
        "<amf unit=\"{}\" version=\"{}\">\n",
        doc.config.unit, doc.config.version
    ));
    out.push_str(&format!(
        "  <metadata type=\"name\">{}</metadata>\n",
        doc.config.document_name
    ));

    for obj in &doc.objects {
        out.push_str(&format!("  <object id=\"{}\">\n", obj.id));
        out.push_str(&format!("    <!-- name: {} -->\n", obj.name));
        if let Some(mat) = &obj.material_name {
            out.push_str(&format!("    <metadata type=\"material\">{mat}</metadata>\n"));
        }
        out.push_str("    <mesh>\n");
        out.push_str(&format!(
            "      <!-- vertices: {} faces: {} -->\n",
            obj.vertex_count, obj.face_count
        ));
        out.push_str("      <vertices/>\n");
        out.push_str("      <volume/>\n");
        out.push_str("    </mesh>\n");
        out.push_str("  </object>\n");
    }

    out.push_str("</amf>\n");
    out
}

/// Writes the AMF document to a file at the given path.
///
/// Returns `Err` with a description if the path is empty or the write fails.
#[allow(dead_code)]
pub fn amf_write_to_file(doc: &AmfDocument, path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path must not be empty".to_string());
    }
    let content = amf_to_xml_string(doc);
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// Returns the number of objects currently in the document.
#[allow(dead_code)]
pub fn amf_object_count(doc: &AmfDocument) -> usize {
    doc.objects.len()
}

/// Sets the unit system for the document (e.g. `"millimeter"`, `"inch"`).
#[allow(dead_code)]
pub fn amf_set_unit(doc: &mut AmfDocument, unit: &str) {
    doc.config.unit = unit.to_string();
}

/// Returns the document name from the configuration.
#[allow(dead_code)]
pub fn amf_document_name(doc: &AmfDocument) -> &str {
    &doc.config.document_name
}

/// Removes all objects from the document and resets the ID counter.
#[allow(dead_code)]
pub fn amf_document_clear(doc: &mut AmfDocument) {
    doc.objects.clear();
    doc.next_id = 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_amf_config();
        assert_eq!(cfg.unit, "millimeter");
        assert_eq!(cfg.version, "1.1");
    }

    #[test]
    fn test_new_document_empty() {
        let cfg = default_amf_config();
        let doc = new_amf_document(&cfg);
        assert_eq!(amf_object_count(&doc), 0);
    }

    #[test]
    fn test_add_object_returns_id() {
        let cfg = default_amf_config();
        let mut doc = new_amf_document(&cfg);
        let id = amf_add_object(&mut doc, "Cube", 8, 12);
        assert_eq!(id, 1);
        assert_eq!(amf_object_count(&doc), 1);
    }

    #[test]
    fn test_add_multiple_objects_unique_ids() {
        let cfg = default_amf_config();
        let mut doc = new_amf_document(&cfg);
        let id1 = amf_add_object(&mut doc, "A", 4, 4);
        let id2 = amf_add_object(&mut doc, "B", 6, 8);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_set_material() {
        let cfg = default_amf_config();
        let mut doc = new_amf_document(&cfg);
        let id = amf_add_object(&mut doc, "Part", 10, 6);
        amf_set_material(&mut doc, id, "PLA");
        assert_eq!(doc.objects[0].material_name.as_deref(), Some("PLA"));
    }

    #[test]
    fn test_set_material_unknown_id_noop() {
        let cfg = default_amf_config();
        let mut doc = new_amf_document(&cfg);
        amf_set_material(&mut doc, 999, "ABS");
        assert!(doc.objects.is_empty());
    }

    #[test]
    fn test_xml_contains_amf_tag() {
        let cfg = default_amf_config();
        let doc = new_amf_document(&cfg);
        let xml = amf_to_xml_string(&doc);
        assert!(xml.contains("<amf"));
        assert!(xml.contains("</amf>"));
    }

    #[test]
    fn test_xml_contains_object() {
        let cfg = default_amf_config();
        let mut doc = new_amf_document(&cfg);
        amf_add_object(&mut doc, "Sphere", 100, 200);
        let xml = amf_to_xml_string(&doc);
        assert!(xml.contains("<object"));
        assert!(xml.contains("Sphere"));
    }

    #[test]
    fn test_set_unit() {
        let cfg = default_amf_config();
        let mut doc = new_amf_document(&cfg);
        amf_set_unit(&mut doc, "inch");
        assert_eq!(doc.config.unit, "inch");
    }

    #[test]
    fn test_document_name() {
        let cfg = default_amf_config();
        let doc = new_amf_document(&cfg);
        assert_eq!(amf_document_name(&doc), "AmfDocument");
    }

    #[test]
    fn test_document_clear() {
        let cfg = default_amf_config();
        let mut doc = new_amf_document(&cfg);
        amf_add_object(&mut doc, "X", 3, 1);
        amf_document_clear(&mut doc);
        assert_eq!(amf_object_count(&doc), 0);
        // After clear, IDs restart from 1
        let id = amf_add_object(&mut doc, "Y", 3, 1);
        assert_eq!(id, 1);
    }

    #[test]
    fn test_write_to_file_empty_path() {
        let cfg = default_amf_config();
        let doc = new_amf_document(&cfg);
        assert!(amf_write_to_file(&doc, "").is_err());
    }
}
