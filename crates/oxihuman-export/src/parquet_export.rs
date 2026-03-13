// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Parquet format export stub — columnar binary data layout description.

/// Parquet column type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParquetColType {
    Int32,
    Int64,
    Float,
    Double,
    ByteArray,
    Boolean,
}

/// A Parquet column definition.
#[derive(Debug, Clone)]
pub struct ParquetColDef {
    pub name: String,
    pub col_type: ParquetColType,
    pub nullable: bool,
    pub row_count: usize,
}

/// A Parquet file export stub (metadata only, no actual binary encoding).
#[derive(Debug, Clone, Default)]
pub struct ParquetExportDef {
    pub columns: Vec<ParquetColDef>,
    pub row_group_size: usize,
    pub compression: String,
}

/// Create a new Parquet export definition.
pub fn new_parquet_export(row_group_size: usize, compression: &str) -> ParquetExportDef {
    ParquetExportDef {
        columns: vec![],
        row_group_size: if row_group_size == 0 { 128 } else { row_group_size },
        compression: compression.into(),
    }
}

/// Add a column definition.
pub fn add_column(export: &mut ParquetExportDef, name: &str, col_type: ParquetColType, nullable: bool, row_count: usize) {
    export.columns.push(ParquetColDef { name: name.into(), col_type, nullable, row_count });
}

/// Export mesh positions as a Parquet schema stub.
pub fn export_mesh_positions_parquet(positions: &[[f32; 3]]) -> ParquetExportDef {
    let n = positions.len();
    let mut def = new_parquet_export(512, "snappy");
    add_column(&mut def, "x", ParquetColType::Float, false, n);
    add_column(&mut def, "y", ParquetColType::Float, false, n);
    add_column(&mut def, "z", ParquetColType::Float, false, n);
    def
}

/// Serialize export metadata to JSON string.
pub fn parquet_metadata_to_json(export: &ParquetExportDef) -> String {
    let cols: Vec<String> = export.columns.iter().map(|c| {
        format!("{{\"name\":\"{}\",\"rows\":{},\"nullable\":{}}}", c.name, c.row_count, c.nullable)
    }).collect();
    format!("{{\"compression\":\"{}\",\"row_group_size\":{},\"columns\":[{}]}}",
        export.compression, export.row_group_size, cols.join(","))
}

/// Number of columns defined.
pub fn column_count(export: &ParquetExportDef) -> usize {
    export.columns.len()
}

/// Total rows across all columns (assumes same row count per column).
pub fn total_rows(export: &ParquetExportDef) -> usize {
    export.columns.first().map_or(0, |c| c.row_count)
}

/// Estimated row group count.
pub fn row_group_count(export: &ParquetExportDef) -> usize {
    let rows = total_rows(export);
    if export.row_group_size == 0 { return 0; }
    rows.div_ceil(export.row_group_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_parquet_export_defaults() {
        /* default row group size applies when 0 */
        let e = new_parquet_export(0, "gzip");
        assert_eq!(e.row_group_size, 128);
    }

    #[test]
    fn test_add_column_increases_count() {
        /* adding column increases count */
        let mut e = new_parquet_export(100, "none");
        add_column(&mut e, "x", ParquetColType::Float, false, 50);
        assert_eq!(column_count(&e), 1);
    }

    #[test]
    fn test_export_mesh_positions_three_columns() {
        /* mesh export has x, y, z columns */
        let p = vec![[1.0f32,2.0,3.0]; 10];
        let e = export_mesh_positions_parquet(&p);
        assert_eq!(column_count(&e), 3);
    }

    #[test]
    fn test_total_rows_matches_input() {
        /* total rows equals input count */
        let p = vec![[0.0f32;3]; 42];
        let e = export_mesh_positions_parquet(&p);
        assert_eq!(total_rows(&e), 42);
    }

    #[test]
    fn test_row_group_count() {
        /* row group count computed correctly */
        let mut e = new_parquet_export(10, "none");
        add_column(&mut e, "x", ParquetColType::Float, false, 25);
        assert_eq!(row_group_count(&e), 3);
    }

    #[test]
    fn test_parquet_metadata_to_json_contains_compression() {
        /* JSON contains compression field */
        let e = new_parquet_export(128, "snappy");
        let j = parquet_metadata_to_json(&e);
        assert!(j.contains("snappy"));
    }

    #[test]
    fn test_column_nullable_flag() {
        /* nullable flag is stored */
        let mut e = new_parquet_export(100, "none");
        add_column(&mut e, "id", ParquetColType::Int64, true, 10);
        assert!(e.columns[0].nullable);
    }

    #[test]
    fn test_column_type_stored() {
        /* column type is stored correctly */
        let mut e = new_parquet_export(100, "none");
        add_column(&mut e, "flag", ParquetColType::Boolean, false, 5);
        assert_eq!(e.columns[0].col_type, ParquetColType::Boolean);
    }

    #[test]
    fn test_empty_export_zero_rows() {
        /* no columns → zero rows */
        let e = new_parquet_export(100, "none");
        assert_eq!(total_rows(&e), 0);
    }

    #[test]
    fn test_compression_stored() {
        /* compression string is stored */
        let e = new_parquet_export(128, "zstd");
        assert_eq!(e.compression, "zstd");
    }
}
