// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export scene/mesh data as TOML text.

#![allow(dead_code)]

/// A simple key-value record for TOML export.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TomlRecord {
    pub key: String,
    pub value: String,
}

/// A TOML export document.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TomlExport {
    pub section: String,
    pub records: Vec<TomlRecord>,
}

/// Create a new empty TOML export with the given section name.
#[allow(dead_code)]
pub fn new_toml_export(section: &str) -> TomlExport {
    TomlExport {
        section: section.to_string(),
        records: Vec::new(),
    }
}

/// Add a string entry.
#[allow(dead_code)]
pub fn add_string(doc: &mut TomlExport, key: &str, value: &str) {
    doc.records.push(TomlRecord {
        key: key.to_string(),
        value: format!("\"{}\"", value),
    });
}

/// Add an integer entry.
#[allow(dead_code)]
pub fn add_integer(doc: &mut TomlExport, key: &str, value: i64) {
    doc.records.push(TomlRecord {
        key: key.to_string(),
        value: value.to_string(),
    });
}

/// Add a float entry.
#[allow(dead_code)]
pub fn add_float(doc: &mut TomlExport, key: &str, value: f64) {
    doc.records.push(TomlRecord {
        key: key.to_string(),
        value: format!("{:.6}", value),
    });
}

/// Add a boolean entry.
#[allow(dead_code)]
pub fn add_bool(doc: &mut TomlExport, key: &str, value: bool) {
    doc.records.push(TomlRecord {
        key: key.to_string(),
        value: if value {
            "true".to_string()
        } else {
            "false".to_string()
        },
    });
}

/// Return the number of records in the document.
#[allow(dead_code)]
pub fn record_count(doc: &TomlExport) -> usize {
    doc.records.len()
}

/// Find a record by key, returning its value.
#[allow(dead_code)]
pub fn find_record<'a>(doc: &'a TomlExport, key: &str) -> Option<&'a str> {
    doc.records
        .iter()
        .find(|r| r.key == key)
        .map(|r| r.value.as_str())
}

/// Serialise the document to a TOML string.
#[allow(dead_code)]
pub fn to_toml_string(doc: &TomlExport) -> String {
    let mut out = format!("[{}]\n", doc.section);
    for r in &doc.records {
        out.push_str(&format!("{} = {}\n", r.key, r.value));
    }
    out
}

/// Export mesh stats as a TOML document.
#[allow(dead_code)]
pub fn export_mesh_stats_toml(vertex_count: usize, index_count: usize, name: &str) -> String {
    let mut doc = new_toml_export("mesh");
    add_string(&mut doc, "name", name);
    add_integer(&mut doc, "vertices", vertex_count as i64);
    add_integer(&mut doc, "indices", index_count as i64);
    add_integer(&mut doc, "triangles", (index_count / 3) as i64);
    to_toml_string(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_toml_export() {
        let doc = new_toml_export("scene");
        assert_eq!(doc.section, "scene");
        assert!(doc.records.is_empty());
    }

    #[test]
    fn test_add_string() {
        let mut doc = new_toml_export("mesh");
        add_string(&mut doc, "name", "body");
        assert_eq!(record_count(&doc), 1);
    }

    #[test]
    fn test_add_integer() {
        let mut doc = new_toml_export("mesh");
        add_integer(&mut doc, "vertices", 512);
        let val = find_record(&doc, "vertices").expect("should succeed");
        assert_eq!(val, "512");
    }

    #[test]
    fn test_add_float() {
        let mut doc = new_toml_export("mesh");
        add_float(&mut doc, "scale", 1.5);
        assert!(find_record(&doc, "scale").is_some());
    }

    #[test]
    fn test_add_bool() {
        let mut doc = new_toml_export("mesh");
        add_bool(&mut doc, "visible", true);
        assert_eq!(find_record(&doc, "visible"), Some("true"));
    }

    #[test]
    fn test_to_toml_string_contains_section() {
        let doc = new_toml_export("scene");
        let s = to_toml_string(&doc);
        assert!(s.contains("[scene]"));
    }

    #[test]
    fn test_to_toml_string_contains_key() {
        let mut doc = new_toml_export("mesh");
        add_integer(&mut doc, "count", 10);
        let s = to_toml_string(&doc);
        assert!(s.contains("count = 10"));
    }

    #[test]
    fn test_export_mesh_stats_toml() {
        let s = export_mesh_stats_toml(100, 300, "test_mesh");
        assert!(s.contains("vertices"));
        assert!(s.contains("test_mesh"));
    }

    #[test]
    fn test_find_record_missing() {
        let doc = new_toml_export("x");
        assert!(find_record(&doc, "missing").is_none());
    }

    #[test]
    fn test_record_count() {
        let mut doc = new_toml_export("x");
        add_string(&mut doc, "a", "1");
        add_string(&mut doc, "b", "2");
        assert_eq!(record_count(&doc), 2);
    }
}
