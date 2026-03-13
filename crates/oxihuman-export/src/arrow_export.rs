// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Export columnar data in Arrow IPC text format (JSON-encoded metadata stub).

#![allow(dead_code)]

/// Arrow data type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowType {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Utf8,
    Binary,
}

impl ArrowType {
    /// Return the Arrow type name.
    #[allow(dead_code)]
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int8 => "int8",
            Self::Int16 => "int16",
            Self::Int32 => "int32",
            Self::Int64 => "int64",
            Self::Float32 => "float32",
            Self::Float64 => "float64",
            Self::Utf8 => "utf8",
            Self::Binary => "binary",
        }
    }
}

/// An Arrow field.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrowField {
    pub name: String,
    pub arrow_type: ArrowType,
    pub nullable: bool,
}

/// An Arrow schema.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ArrowSchema {
    pub fields: Vec<ArrowField>,
}

/// An Arrow batch (column data as string-encoded values).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ArrowBatch {
    pub num_rows: usize,
    pub columns: Vec<Vec<String>>,
}

/// An Arrow export.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ArrowExport {
    pub schema: ArrowSchema,
    pub batches: Vec<ArrowBatch>,
}

/// Create a new Arrow export.
#[allow(dead_code)]
pub fn new_arrow_export() -> ArrowExport {
    ArrowExport {
        schema: ArrowSchema { fields: Vec::new() },
        batches: Vec::new(),
    }
}

/// Add a field to the schema.
#[allow(dead_code)]
pub fn add_arrow_field(doc: &mut ArrowExport, name: &str, arrow_type: ArrowType, nullable: bool) {
    doc.schema.fields.push(ArrowField {
        name: name.to_string(),
        arrow_type,
        nullable,
    });
}

/// Add a batch.
#[allow(dead_code)]
pub fn add_arrow_batch(doc: &mut ArrowExport, num_rows: usize, columns: Vec<Vec<String>>) {
    doc.batches.push(ArrowBatch { num_rows, columns });
}

/// Return field count.
#[allow(dead_code)]
pub fn arrow_field_count(doc: &ArrowExport) -> usize {
    doc.schema.fields.len()
}

/// Return total rows across all batches.
#[allow(dead_code)]
pub fn arrow_total_rows(doc: &ArrowExport) -> usize {
    doc.batches.iter().map(|b| b.num_rows).sum()
}

/// Serialise the schema as JSON.
#[allow(dead_code)]
pub fn arrow_schema_to_json(doc: &ArrowExport) -> String {
    let fields: Vec<String> = doc
        .schema
        .fields
        .iter()
        .map(|f| {
            format!(
                "{{\"name\":\"{}\",\"type\":\"{}\",\"nullable\":{}}}",
                f.name,
                f.arrow_type.type_name(),
                f.nullable
            )
        })
        .collect();
    format!("{{\"fields\":[{}]}}", fields.join(","))
}

/// Serialise a batch as JSON.
#[allow(dead_code)]
pub fn arrow_batch_to_json(batch: &ArrowBatch, schema: &ArrowSchema) -> String {
    let cols: Vec<String> = batch
        .columns
        .iter()
        .enumerate()
        .map(|(i, col)| {
            let name = schema.fields.get(i).map_or("col", |f| f.name.as_str());
            let data = col.join(",");
            format!("{{\"name\":\"{}\",\"data\":[{}]}}", name, data)
        })
        .collect();
    format!(
        "{{\"num_rows\":{},\"columns\":[{}]}}",
        batch.num_rows,
        cols.join(",")
    )
}

/// Export mesh positions as Arrow IPC JSON.
#[allow(dead_code)]
pub fn export_positions_arrow(positions: &[[f32; 3]]) -> String {
    let mut doc = new_arrow_export();
    add_arrow_field(&mut doc, "x", ArrowType::Float32, false);
    add_arrow_field(&mut doc, "y", ArrowType::Float32, false);
    add_arrow_field(&mut doc, "z", ArrowType::Float32, false);
    let xs: Vec<String> = positions.iter().map(|p| format!("{:.6}", p[0])).collect();
    let ys: Vec<String> = positions.iter().map(|p| format!("{:.6}", p[1])).collect();
    let zs: Vec<String> = positions.iter().map(|p| format!("{:.6}", p[2])).collect();
    let n = positions.len();
    add_arrow_batch(&mut doc, n, vec![xs, ys, zs]);
    let schema_json = arrow_schema_to_json(&doc);
    let batch_json = if doc.batches.is_empty() {
        String::from("[]")
    } else {
        format!("[{}]", arrow_batch_to_json(&doc.batches[0], &doc.schema))
    };
    format!("{{\"schema\":{},\"batches\":{}}}", schema_json, batch_json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_arrow_export_empty() {
        let doc = new_arrow_export();
        assert_eq!(arrow_field_count(&doc), 0);
        assert_eq!(arrow_total_rows(&doc), 0);
    }

    #[test]
    fn test_add_field() {
        let mut doc = new_arrow_export();
        add_arrow_field(&mut doc, "x", ArrowType::Float32, false);
        assert_eq!(arrow_field_count(&doc), 1);
    }

    #[test]
    fn test_add_batch() {
        let mut doc = new_arrow_export();
        add_arrow_batch(&mut doc, 5, vec![vec!["1.0".to_string(); 5]]);
        assert_eq!(arrow_total_rows(&doc), 5);
    }

    #[test]
    fn test_type_name_float32() {
        assert_eq!(ArrowType::Float32.type_name(), "float32");
    }

    #[test]
    fn test_type_name_utf8() {
        assert_eq!(ArrowType::Utf8.type_name(), "utf8");
    }

    #[test]
    fn test_schema_to_json_contains_field() {
        let mut doc = new_arrow_export();
        add_arrow_field(&mut doc, "pos_x", ArrowType::Float32, false);
        let s = arrow_schema_to_json(&doc);
        assert!(s.contains("pos_x"));
    }

    #[test]
    fn test_batch_to_json_contains_col() {
        let schema = ArrowSchema {
            fields: vec![ArrowField {
                name: "x".to_string(),
                arrow_type: ArrowType::Float32,
                nullable: false,
            }],
        };
        let batch = ArrowBatch {
            num_rows: 2,
            columns: vec![vec!["1.0".to_string(), "2.0".to_string()]],
        };
        let s = arrow_batch_to_json(&batch, &schema);
        assert!(s.contains("num_rows"));
    }

    #[test]
    fn test_export_positions_arrow() {
        let pts = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let s = export_positions_arrow(&pts);
        assert!(s.contains("schema"));
        assert!(s.contains("float32"));
    }

    #[test]
    fn test_export_positions_empty() {
        let pts: Vec<[f32; 3]> = vec![];
        let s = export_positions_arrow(&pts);
        assert!(s.contains("fields"));
    }

    #[test]
    fn test_total_rows_multi_batch() {
        let mut doc = new_arrow_export();
        add_arrow_batch(&mut doc, 10, vec![]);
        add_arrow_batch(&mut doc, 20, vec![]);
        assert_eq!(arrow_total_rows(&doc), 30);
    }
}
