// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stub exporter that generates Cap'n Proto schema text from mesh/scene data.

#![allow(dead_code)]

/// A Cap'n Proto field descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CapnpField {
    pub name: String,
    pub field_type: String,
    pub index: usize,
}

/// A Cap'n Proto struct descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CapnpStruct {
    pub name: String,
    pub fields: Vec<CapnpField>,
}

/// A Cap'n Proto schema export.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CapnpExport {
    pub file_id: u64,
    pub structs: Vec<CapnpStruct>,
}

/// Create a new Cap'n Proto export with a given file ID.
#[allow(dead_code)]
pub fn new_capnp_export(file_id: u64) -> CapnpExport {
    CapnpExport {
        file_id,
        structs: Vec::new(),
    }
}

/// Add a struct.
#[allow(dead_code)]
pub fn add_capnp_struct(doc: &mut CapnpExport, name: &str) {
    doc.structs.push(CapnpStruct {
        name: name.to_string(),
        fields: Vec::new(),
    });
}

/// Add a field to the last struct.
#[allow(dead_code)]
pub fn add_capnp_field(doc: &mut CapnpExport, name: &str, field_type: &str) {
    if let Some(s) = doc.structs.last_mut() {
        let idx = s.fields.len();
        s.fields.push(CapnpField {
            name: name.to_string(),
            field_type: field_type.to_string(),
            index: idx,
        });
    }
}

/// Return the number of structs.
#[allow(dead_code)]
pub fn capnp_struct_count(doc: &CapnpExport) -> usize {
    doc.structs.len()
}

/// Return the number of fields in the last struct.
#[allow(dead_code)]
pub fn capnp_last_struct_field_count(doc: &CapnpExport) -> usize {
    doc.structs.last().map_or(0, |s| s.fields.len())
}

/// Serialise as Cap'n Proto schema text (stub).
#[allow(dead_code)]
pub fn to_capnp_schema(doc: &CapnpExport) -> String {
    let mut out = format!("@0x{:016x};\n\n", doc.file_id);
    for st in &doc.structs {
        out.push_str(&format!("struct {} {{\n", st.name));
        for f in &st.fields {
            out.push_str(&format!("  {} @{} :{},\n", f.name, f.index, f.field_type));
        }
        out.push_str("}\n\n");
    }
    out
}

/// Generate a stub schema for a mesh.
#[allow(dead_code)]
pub fn export_mesh_capnp_schema(vertex_count: usize, index_count: usize) -> String {
    let mut doc = new_capnp_export(0xdeadbeefcafe0001);
    add_capnp_struct(&mut doc, "Mesh");
    add_capnp_field(&mut doc, "vertexCount", "UInt32");
    add_capnp_field(&mut doc, "indexCount", "UInt32");
    add_capnp_field(&mut doc, "positions", "List(Float32)");

    let mut comment = format!(
        "# vertexCount={}, indexCount={}\n",
        vertex_count, index_count
    );
    comment.push_str(&to_capnp_schema(&doc));
    comment
}

/// Find a struct by name.
#[allow(dead_code)]
pub fn find_capnp_struct<'a>(doc: &'a CapnpExport, name: &str) -> Option<&'a CapnpStruct> {
    doc.structs.iter().find(|s| s.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_capnp_export_empty() {
        let doc = new_capnp_export(1);
        assert_eq!(capnp_struct_count(&doc), 0);
    }

    #[test]
    fn test_add_struct() {
        let mut doc = new_capnp_export(1);
        add_capnp_struct(&mut doc, "Vertex");
        assert_eq!(capnp_struct_count(&doc), 1);
    }

    #[test]
    fn test_add_field() {
        let mut doc = new_capnp_export(1);
        add_capnp_struct(&mut doc, "Mesh");
        add_capnp_field(&mut doc, "count", "UInt32");
        assert_eq!(capnp_last_struct_field_count(&doc), 1);
    }

    #[test]
    fn test_to_capnp_schema_contains_struct() {
        let mut doc = new_capnp_export(1);
        add_capnp_struct(&mut doc, "MyStruct");
        let s = to_capnp_schema(&doc);
        assert!(s.contains("struct MyStruct"));
    }

    #[test]
    fn test_to_capnp_schema_contains_field() {
        let mut doc = new_capnp_export(1);
        add_capnp_struct(&mut doc, "M");
        add_capnp_field(&mut doc, "vertices", "UInt32");
        let s = to_capnp_schema(&doc);
        assert!(s.contains("vertices"));
    }

    #[test]
    fn test_to_capnp_schema_has_file_id() {
        let doc = new_capnp_export(0x1234);
        let s = to_capnp_schema(&doc);
        assert!(s.contains("@0x"));
    }

    #[test]
    fn test_export_mesh_capnp_schema() {
        let s = export_mesh_capnp_schema(100, 300);
        assert!(s.contains("Mesh"));
        assert!(s.contains("vertexCount=100"));
    }

    #[test]
    fn test_find_capnp_struct() {
        let mut doc = new_capnp_export(1);
        add_capnp_struct(&mut doc, "Bone");
        assert!(find_capnp_struct(&doc, "Bone").is_some());
    }

    #[test]
    fn test_find_missing_struct() {
        let doc = new_capnp_export(1);
        assert!(find_capnp_struct(&doc, "Ghost").is_none());
    }

    #[test]
    fn test_field_index_increments() {
        let mut doc = new_capnp_export(1);
        add_capnp_struct(&mut doc, "M");
        add_capnp_field(&mut doc, "a", "Int32");
        add_capnp_field(&mut doc, "b", "Int32");
        let st = find_capnp_struct(&doc, "M").unwrap();
        assert_eq!(st.fields[1].index, 1);
    }
}
