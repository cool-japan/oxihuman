// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Stub exporter that generates FlatBuffers schema (.fbs) text.

#![allow(dead_code)]

/// A FlatBuffers field.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbsField {
    pub name: String,
    pub field_type: String,
    pub default_value: Option<String>,
}

/// A FlatBuffers table descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FbsTable {
    pub name: String,
    pub fields: Vec<FbsField>,
    pub is_struct: bool,
}

/// A FlatBuffers schema export.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct FlatbufExport {
    pub namespace: String,
    pub tables: Vec<FbsTable>,
    pub root_type: String,
}

/// Create a new FlatBuffers export.
#[allow(dead_code)]
pub fn new_flatbuf_export(namespace: &str) -> FlatbufExport {
    FlatbufExport {
        namespace: namespace.to_string(),
        tables: Vec::new(),
        root_type: String::new(),
    }
}

/// Add a table (or struct).
#[allow(dead_code)]
pub fn add_fbs_table(doc: &mut FlatbufExport, name: &str, is_struct: bool) {
    doc.tables.push(FbsTable {
        name: name.to_string(),
        fields: Vec::new(),
        is_struct,
    });
}

/// Add a field to the last table.
#[allow(dead_code)]
pub fn add_fbs_field(
    doc: &mut FlatbufExport,
    name: &str,
    field_type: &str,
    default_value: Option<&str>,
) {
    if let Some(table) = doc.tables.last_mut() {
        table.fields.push(FbsField {
            name: name.to_string(),
            field_type: field_type.to_string(),
            default_value: default_value.map(|s| s.to_string()),
        });
    }
}

/// Set the root type.
#[allow(dead_code)]
pub fn set_fbs_root_type(doc: &mut FlatbufExport, name: &str) {
    doc.root_type = name.to_string();
}

/// Return number of tables.
#[allow(dead_code)]
pub fn fbs_table_count(doc: &FlatbufExport) -> usize {
    doc.tables.len()
}

/// Return number of fields in last table.
#[allow(dead_code)]
pub fn fbs_last_table_field_count(doc: &FlatbufExport) -> usize {
    doc.tables.last().map_or(0, |t| t.fields.len())
}

/// Serialise as FlatBuffers .fbs schema.
#[allow(dead_code)]
pub fn to_fbs_schema(doc: &FlatbufExport) -> String {
    let mut out = if doc.namespace.is_empty() {
        String::new()
    } else {
        format!("namespace {};\n\n", doc.namespace)
    };
    for table in &doc.tables {
        let kw = if table.is_struct { "struct" } else { "table" };
        out.push_str(&format!("{} {} {{\n", kw, table.name));
        for f in &table.fields {
            if let Some(ref dv) = f.default_value {
                out.push_str(&format!("  {}:{} = {};\n", f.name, f.field_type, dv));
            } else {
                out.push_str(&format!("  {}:{};\n", f.name, f.field_type));
            }
        }
        out.push_str("}\n\n");
    }
    if !doc.root_type.is_empty() {
        out.push_str(&format!("root_type {};\n", doc.root_type));
    }
    out
}

/// Find a table by name.
#[allow(dead_code)]
pub fn find_fbs_table<'a>(doc: &'a FlatbufExport, name: &str) -> Option<&'a FbsTable> {
    doc.tables.iter().find(|t| t.name == name)
}

/// Generate a stub FBS schema for a mesh.
#[allow(dead_code)]
pub fn export_mesh_fbs_schema(vertex_count: usize) -> String {
    let mut doc = new_flatbuf_export("oxihuman");
    add_fbs_table(&mut doc, "Mesh", false);
    add_fbs_field(&mut doc, "name", "string", None);
    add_fbs_field(&mut doc, "vertex_count", "uint32", Some("0"));
    add_fbs_field(&mut doc, "positions", "[float]", None);
    set_fbs_root_type(&mut doc, "Mesh");
    let mut out = format!("// vertex_count hint: {}\n", vertex_count);
    out.push_str(&to_fbs_schema(&doc));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flatbuf_export_empty() {
        let doc = new_flatbuf_export("ns");
        assert_eq!(fbs_table_count(&doc), 0);
    }

    #[test]
    fn test_add_table() {
        let mut doc = new_flatbuf_export("ns");
        add_fbs_table(&mut doc, "Mesh", false);
        assert_eq!(fbs_table_count(&doc), 1);
    }

    #[test]
    fn test_add_field() {
        let mut doc = new_flatbuf_export("ns");
        add_fbs_table(&mut doc, "M", false);
        add_fbs_field(&mut doc, "count", "uint32", None);
        assert_eq!(fbs_last_table_field_count(&doc), 1);
    }

    #[test]
    fn test_to_fbs_contains_namespace() {
        let doc = new_flatbuf_export("myns");
        let s = to_fbs_schema(&doc);
        assert!(s.contains("namespace myns"));
    }

    #[test]
    fn test_to_fbs_contains_table() {
        let mut doc = new_flatbuf_export("ns");
        add_fbs_table(&mut doc, "Vertex", false);
        let s = to_fbs_schema(&doc);
        assert!(s.contains("table Vertex"));
    }

    #[test]
    fn test_to_fbs_contains_struct() {
        let mut doc = new_flatbuf_export("ns");
        add_fbs_table(&mut doc, "Vec3", true);
        let s = to_fbs_schema(&doc);
        assert!(s.contains("struct Vec3"));
    }

    #[test]
    fn test_to_fbs_contains_field() {
        let mut doc = new_flatbuf_export("ns");
        add_fbs_table(&mut doc, "M", false);
        add_fbs_field(&mut doc, "vertices", "[float]", None);
        let s = to_fbs_schema(&doc);
        assert!(s.contains("vertices:[float]"));
    }

    #[test]
    fn test_root_type() {
        let mut doc = new_flatbuf_export("ns");
        set_fbs_root_type(&mut doc, "Mesh");
        let s = to_fbs_schema(&doc);
        assert!(s.contains("root_type Mesh"));
    }

    #[test]
    fn test_export_mesh_fbs_schema() {
        let s = export_mesh_fbs_schema(512);
        assert!(s.contains("Mesh"));
        assert!(s.contains("vertex_count hint: 512"));
    }

    #[test]
    fn test_find_fbs_table() {
        let mut doc = new_flatbuf_export("ns");
        add_fbs_table(&mut doc, "Bone", false);
        assert!(find_fbs_table(&doc, "Bone").is_some());
        assert!(find_fbs_table(&doc, "Ghost").is_none());
    }
}
