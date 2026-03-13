// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Stub exporter for Parquet row-group metadata.

#![allow(dead_code)]

/// Parquet physical type.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParquetType {
    Boolean,
    Int32,
    Int64,
    Float,
    Double,
    ByteArray,
}

impl ParquetType {
    /// Return the type name string.
    #[allow(dead_code)]
    pub fn type_name(self) -> &'static str {
        match self {
            Self::Boolean => "BOOLEAN",
            Self::Int32 => "INT32",
            Self::Int64 => "INT64",
            Self::Float => "FLOAT",
            Self::Double => "DOUBLE",
            Self::ByteArray => "BYTE_ARRAY",
        }
    }
}

/// A Parquet column descriptor.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ParquetColumn {
    pub name: String,
    pub ptype: ParquetType,
    pub num_values: usize,
    pub compressed_size: usize,
    pub uncompressed_size: usize,
}

/// A Parquet row group metadata.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ParquetRowGroup {
    pub num_rows: usize,
    pub columns: Vec<ParquetColumn>,
}

/// A Parquet file metadata stub.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ParquetExport {
    pub schema_name: String,
    pub row_groups: Vec<ParquetRowGroup>,
    pub compression: String,
}

/// Create a new Parquet export.
#[allow(dead_code)]
pub fn new_parquet_export(schema_name: &str, compression: &str) -> ParquetExport {
    ParquetExport {
        schema_name: schema_name.to_string(),
        row_groups: Vec::new(),
        compression: compression.to_string(),
    }
}

/// Add a row group.
#[allow(dead_code)]
pub fn add_row_group(doc: &mut ParquetExport, num_rows: usize) {
    doc.row_groups.push(ParquetRowGroup {
        num_rows,
        columns: Vec::new(),
    });
}

/// Add a column to the last row group.
#[allow(dead_code)]
pub fn add_parquet_column(
    doc: &mut ParquetExport,
    name: &str,
    ptype: ParquetType,
    num_values: usize,
    compressed_size: usize,
    uncompressed_size: usize,
) {
    if let Some(rg) = doc.row_groups.last_mut() {
        rg.columns.push(ParquetColumn {
            name: name.to_string(),
            ptype,
            num_values,
            compressed_size,
            uncompressed_size,
        });
    }
}

/// Return the number of row groups.
#[allow(dead_code)]
pub fn parquet_row_group_count(doc: &ParquetExport) -> usize {
    doc.row_groups.len()
}

/// Return the total number of rows across all row groups.
#[allow(dead_code)]
pub fn parquet_total_rows(doc: &ParquetExport) -> usize {
    doc.row_groups.iter().map(|rg| rg.num_rows).sum()
}

/// Serialise as a JSON metadata string (stub).
#[allow(dead_code)]
pub fn to_parquet_metadata_json(doc: &ParquetExport) -> String {
    let rgs: Vec<String> = doc
        .row_groups
        .iter()
        .map(|rg| {
            let cols: Vec<String> = rg
                .columns
                .iter()
                .map(|c| {
                    format!(
                        "{{\"name\":\"{}\",\"type\":\"{}\",\"num_values\":{},\"compressed\":{},\"uncompressed\":{}}}",
                        c.name, c.ptype.type_name(), c.num_values, c.compressed_size, c.uncompressed_size
                    )
                })
                .collect();
            format!("{{\"num_rows\":{},\"columns\":[{}]}}", rg.num_rows, cols.join(","))
        })
        .collect();
    format!(
        "{{\"schema\":\"{}\",\"compression\":\"{}\",\"row_groups\":[{}]}}",
        doc.schema_name,
        doc.compression,
        rgs.join(",")
    )
}

/// Export a mesh as Parquet metadata stub.
#[allow(dead_code)]
pub fn export_mesh_parquet_meta(vertex_count: usize, index_count: usize) -> String {
    let mut doc = new_parquet_export("Mesh", "SNAPPY");
    add_row_group(&mut doc, vertex_count);
    add_parquet_column(
        &mut doc,
        "x",
        ParquetType::Float,
        vertex_count,
        vertex_count * 3,
        vertex_count * 4,
    );
    add_parquet_column(
        &mut doc,
        "y",
        ParquetType::Float,
        vertex_count,
        vertex_count * 3,
        vertex_count * 4,
    );
    add_parquet_column(
        &mut doc,
        "z",
        ParquetType::Float,
        vertex_count,
        vertex_count * 3,
        vertex_count * 4,
    );
    add_row_group(&mut doc, index_count);
    add_parquet_column(
        &mut doc,
        "index",
        ParquetType::Int32,
        index_count,
        index_count * 3,
        index_count * 4,
    );
    to_parquet_metadata_json(&doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_parquet_export_empty() {
        let doc = new_parquet_export("Schema", "NONE");
        assert_eq!(parquet_row_group_count(&doc), 0);
    }

    #[test]
    fn test_add_row_group() {
        let mut doc = new_parquet_export("S", "NONE");
        add_row_group(&mut doc, 100);
        assert_eq!(parquet_row_group_count(&doc), 1);
    }

    #[test]
    fn test_total_rows() {
        let mut doc = new_parquet_export("S", "NONE");
        add_row_group(&mut doc, 50);
        add_row_group(&mut doc, 75);
        assert_eq!(parquet_total_rows(&doc), 125);
    }

    #[test]
    fn test_add_column() {
        let mut doc = new_parquet_export("S", "NONE");
        add_row_group(&mut doc, 10);
        add_parquet_column(&mut doc, "x", ParquetType::Float, 10, 30, 40);
        assert_eq!(doc.row_groups[0].columns.len(), 1);
    }

    #[test]
    fn test_type_name_float() {
        assert_eq!(ParquetType::Float.type_name(), "FLOAT");
    }

    #[test]
    fn test_type_name_int32() {
        assert_eq!(ParquetType::Int32.type_name(), "INT32");
    }

    #[test]
    fn test_to_json_contains_schema() {
        let doc = new_parquet_export("MySchema", "GZIP");
        let s = to_parquet_metadata_json(&doc);
        assert!(s.contains("MySchema"));
    }

    #[test]
    fn test_to_json_contains_compression() {
        let doc = new_parquet_export("S", "SNAPPY");
        let s = to_parquet_metadata_json(&doc);
        assert!(s.contains("SNAPPY"));
    }

    #[test]
    fn test_export_mesh_parquet_meta() {
        let s = export_mesh_parquet_meta(100, 300);
        assert!(s.contains("Mesh"));
        assert!(s.contains("SNAPPY"));
    }

    #[test]
    fn test_export_mesh_parquet_two_row_groups() {
        let s = export_mesh_parquet_meta(10, 30);
        // Two row groups (vertices and indices)
        assert_eq!(s.matches("num_rows").count(), 2);
    }
}
