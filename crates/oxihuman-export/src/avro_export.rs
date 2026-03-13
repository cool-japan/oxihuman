// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export records as Apache Avro JSON encoding (schema + records).

#![allow(dead_code)]

/// An Avro field definition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AvroField {
    pub name: String,
    pub field_type: String,
}

/// An Avro schema.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AvroSchema {
    pub name: String,
    pub namespace: String,
    pub fields: Vec<AvroField>,
}

/// An Avro record (name -> value pairs as strings).
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AvroRecord {
    pub values: Vec<(String, String)>,
}

/// An Avro export containing schema + records.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AvroExport {
    pub schema: AvroSchema,
    pub records: Vec<AvroRecord>,
}

/// Create a new Avro schema.
#[allow(dead_code)]
pub fn new_avro_schema(name: &str, namespace: &str) -> AvroSchema {
    AvroSchema {
        name: name.to_string(),
        namespace: namespace.to_string(),
        fields: Vec::new(),
    }
}

/// Add a field to the schema.
#[allow(dead_code)]
pub fn add_avro_field(schema: &mut AvroSchema, name: &str, field_type: &str) {
    schema.fields.push(AvroField {
        name: name.to_string(),
        field_type: field_type.to_string(),
    });
}

/// Create a new Avro export.
#[allow(dead_code)]
pub fn new_avro_export(schema: AvroSchema) -> AvroExport {
    AvroExport {
        schema,
        records: Vec::new(),
    }
}

/// Add a record.
#[allow(dead_code)]
pub fn add_avro_record(doc: &mut AvroExport, values: &[(&str, &str)]) {
    doc.records.push(AvroRecord {
        values: values
            .iter()
            .map(|&(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    });
}

/// Return number of records.
#[allow(dead_code)]
pub fn avro_record_count(doc: &AvroExport) -> usize {
    doc.records.len()
}

/// Return number of fields in schema.
#[allow(dead_code)]
pub fn avro_field_count(doc: &AvroExport) -> usize {
    doc.schema.fields.len()
}

/// Serialise the schema as JSON.
#[allow(dead_code)]
pub fn schema_to_json(schema: &AvroSchema) -> String {
    let fields: Vec<String> = schema
        .fields
        .iter()
        .map(|f| format!("{{\"name\":\"{}\",\"type\":\"{}\"}}", f.name, f.field_type))
        .collect();
    format!(
        "{{\"type\":\"record\",\"name\":\"{}\",\"namespace\":\"{}\",\"fields\":[{}]}}",
        schema.name,
        schema.namespace,
        fields.join(",")
    )
}

/// Serialise a single record as JSON.
#[allow(dead_code)]
pub fn record_to_json(record: &AvroRecord) -> String {
    let pairs: Vec<String> = record
        .values
        .iter()
        .map(|(k, v)| format!("\"{}\":{}", k, v))
        .collect();
    format!("{{{}}}", pairs.join(","))
}

/// Serialise all records as a JSON array.
#[allow(dead_code)]
pub fn records_to_json(doc: &AvroExport) -> String {
    let recs: Vec<String> = doc.records.iter().map(record_to_json).collect();
    format!("[{}]", recs.join(","))
}

/// Export mesh stats as Avro JSON.
#[allow(dead_code)]
pub fn export_mesh_avro(vertex_count: usize, index_count: usize, name: &str) -> String {
    let mut schema = new_avro_schema("Mesh", "oxihuman");
    add_avro_field(&mut schema, "name", "string");
    add_avro_field(&mut schema, "vertex_count", "int");
    add_avro_field(&mut schema, "index_count", "int");
    let schema_json = schema_to_json(&schema);
    let mut doc = new_avro_export(schema);
    add_avro_record(
        &mut doc,
        &[
            ("name", &format!("\"{}\"", name)),
            ("vertex_count", &vertex_count.to_string()),
            ("index_count", &index_count.to_string()),
        ],
    );
    format!(
        "{{\"schema\":{},\"records\":{}}}",
        schema_json,
        records_to_json(&doc)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_avro_schema() {
        let schema = new_avro_schema("Mesh", "ns");
        assert_eq!(schema.name, "Mesh");
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn test_add_avro_field() {
        let mut schema = new_avro_schema("M", "ns");
        add_avro_field(&mut schema, "count", "int");
        assert_eq!(schema.fields.len(), 1);
    }

    #[test]
    fn test_add_avro_record() {
        let schema = new_avro_schema("M", "ns");
        let mut doc = new_avro_export(schema);
        add_avro_record(&mut doc, &[("count", "42")]);
        assert_eq!(avro_record_count(&doc), 1);
    }

    #[test]
    fn test_schema_to_json_contains_name() {
        let schema = new_avro_schema("Vertex", "ns");
        let s = schema_to_json(&schema);
        assert!(s.contains("Vertex"));
    }

    #[test]
    fn test_schema_to_json_contains_field() {
        let mut schema = new_avro_schema("M", "ns");
        add_avro_field(&mut schema, "x", "float");
        let s = schema_to_json(&schema);
        assert!(s.contains("\"name\":\"x\""));
    }

    #[test]
    fn test_record_to_json() {
        let rec = AvroRecord {
            values: vec![("k".to_string(), "1".to_string())],
        };
        let s = record_to_json(&rec);
        assert!(s.contains("\"k\":1"));
    }

    #[test]
    fn test_records_to_json_empty() {
        let schema = new_avro_schema("M", "ns");
        let doc = new_avro_export(schema);
        let s = records_to_json(&doc);
        assert_eq!(s, "[]");
    }

    #[test]
    fn test_avro_field_count() {
        let mut schema = new_avro_schema("M", "ns");
        add_avro_field(&mut schema, "a", "int");
        add_avro_field(&mut schema, "b", "string");
        let doc = new_avro_export(schema);
        assert_eq!(avro_field_count(&doc), 2);
    }

    #[test]
    fn test_export_mesh_avro() {
        let s = export_mesh_avro(100, 300, "head");
        assert!(s.contains("Mesh"));
        assert!(s.contains("head"));
    }

    #[test]
    fn test_records_to_json_single() {
        let schema = new_avro_schema("M", "ns");
        let mut doc = new_avro_export(schema);
        add_avro_record(&mut doc, &[("n", "5")]);
        let s = records_to_json(&doc);
        assert!(s.starts_with('['));
    }
}
