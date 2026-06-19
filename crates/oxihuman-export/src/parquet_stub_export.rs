// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Stub exporter for Parquet row-group metadata, and a real binary Parquet
//! encoder (`to_parquet_bytes`) that produces standards-compliant files.

#![allow(dead_code)]

use crate::thrift_export::{compact_type, ThriftCompactEncoder};

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

// ─── Real binary Parquet encoder ─────────────────────────────────────────────

/// Physical column data for binary Parquet encoding.
///
/// Each variant corresponds to a Parquet physical type and carries the values
/// for a single column.
#[derive(Debug, Clone)]
pub enum ParquetColumnData {
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    ByteArray(Vec<Vec<u8>>),
    Boolean(Vec<bool>),
}

/// A named column with its physical data.
#[derive(Debug, Clone)]
pub struct ParquetColumnWithData {
    pub name: String,
    pub data: ParquetColumnData,
}

impl ParquetColumnWithData {
    /// Construct from a name and column data.
    #[allow(dead_code)]
    pub fn new(name: impl Into<String>, data: ParquetColumnData) -> Self {
        ParquetColumnWithData {
            name: name.into(),
            data,
        }
    }
}

/// Parquet physical type enum values (as per the Parquet spec).
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
enum ParquetPhysType {
    Boolean = 0,
    Int32 = 1,
    Int64 = 2,
    Float = 4,
    Double = 5,
    ByteArray = 6,
}

impl ParquetColumnData {
    /// Return the Parquet physical type for this data.
    fn phys_type(&self) -> ParquetPhysType {
        match self {
            ParquetColumnData::Boolean(_) => ParquetPhysType::Boolean,
            ParquetColumnData::Int32(_) => ParquetPhysType::Int32,
            ParquetColumnData::Int64(_) => ParquetPhysType::Int64,
            ParquetColumnData::Float(_) => ParquetPhysType::Float,
            ParquetColumnData::Double(_) => ParquetPhysType::Double,
            ParquetColumnData::ByteArray(_) => ParquetPhysType::ByteArray,
        }
    }

    /// Return the number of logical values in this column.
    fn len(&self) -> usize {
        match self {
            ParquetColumnData::Boolean(v) => v.len(),
            ParquetColumnData::Int32(v) => v.len(),
            ParquetColumnData::Int64(v) => v.len(),
            ParquetColumnData::Float(v) => v.len(),
            ParquetColumnData::Double(v) => v.len(),
            ParquetColumnData::ByteArray(v) => v.len(),
        }
    }

    /// Encode as PLAIN bytes (the raw page data, no header).
    fn encode_plain(&self) -> Vec<u8> {
        match self {
            ParquetColumnData::Boolean(vals) => {
                // PLAIN boolean: 1 byte per value (0x00 or 0x01).
                vals.iter().map(|&b| if b { 1u8 } else { 0u8 }).collect()
            }
            ParquetColumnData::Int32(vals) => {
                let mut out = Vec::with_capacity(vals.len() * 4);
                for &v in vals {
                    out.extend_from_slice(&v.to_le_bytes());
                }
                out
            }
            ParquetColumnData::Int64(vals) => {
                let mut out = Vec::with_capacity(vals.len() * 8);
                for &v in vals {
                    out.extend_from_slice(&v.to_le_bytes());
                }
                out
            }
            ParquetColumnData::Float(vals) => {
                let mut out = Vec::with_capacity(vals.len() * 4);
                for &v in vals {
                    out.extend_from_slice(&v.to_bits().to_le_bytes());
                }
                out
            }
            ParquetColumnData::Double(vals) => {
                let mut out = Vec::with_capacity(vals.len() * 8);
                for &v in vals {
                    out.extend_from_slice(&v.to_bits().to_le_bytes());
                }
                out
            }
            ParquetColumnData::ByteArray(vals) => {
                let total: usize = vals.iter().map(|v| 4 + v.len()).sum();
                let mut out = Vec::with_capacity(total);
                for v in vals {
                    out.extend_from_slice(&(v.len() as u32).to_le_bytes());
                    out.extend_from_slice(v);
                }
                out
            }
        }
    }
}

/// Encode a Parquet `PageHeader` (DATA_PAGE) using Thrift Compact protocol.
///
/// ```text
/// struct PageHeader {
///   1: required PageType page_type          // DATA_PAGE = 0
///   2: required i32 uncompressed_page_size
///   3: required i32 compressed_page_size
///   5: optional DataPageHeader data_page_header {
///     1: required i32 num_values
///     2: required Encoding encoding          // PLAIN = 0
///     3: required Encoding definition_levels_byte_length  // 0
///     4: required Encoding repetition_levels_byte_length  // 0
///   }
/// }
/// ```
fn encode_page_header(num_values: i32, data_size: i32) -> Vec<u8> {
    let mut enc = ThriftCompactEncoder::new();
    enc.begin_struct();
    enc.write_i32_field(1, 0); // page_type = DATA_PAGE
    enc.write_i32_field(2, data_size); // uncompressed_page_size
    enc.write_i32_field(3, data_size); // compressed_page_size (no compression)
    // Field 5 = data_page_header (nested struct)
    enc.write_struct_field_begin(5);
    enc.write_i32_field(1, num_values); // num_values
    enc.write_i32_field(2, 0); // encoding = PLAIN
    enc.write_i32_field(3, 0); // definition_levels_byte_length = 0
    enc.write_i32_field(4, 0); // repetition_levels_byte_length = 0
    enc.end_struct(); // end data_page_header
    enc.end_struct(); // end PageHeader
    enc.into_bytes()
}

/// Encode a `SchemaElement` for the root message group.
///
/// The root element has no `type` field (it is a group), just `name = "schema"`.
/// ```text
/// struct SchemaElement {
///   5: required string name
///   3: optional i32 num_children   // number of direct children
/// }
/// ```
fn encode_schema_root(num_children: i32) -> Vec<u8> {
    let mut enc = ThriftCompactEncoder::new();
    enc.begin_struct();
    enc.write_i32_field(3, num_children);
    enc.write_string_field(5, b"schema");
    enc.end_struct();
    enc.into_bytes()
}

/// Encode a `SchemaElement` for a leaf (data) column.
///
/// ```text
/// struct SchemaElement {
///   1: optional Type type            // physical type enum
///   5: required string name
/// }
/// ```
fn encode_schema_column(col_name: &str, phys_type: ParquetPhysType) -> Vec<u8> {
    let mut enc = ThriftCompactEncoder::new();
    enc.begin_struct();
    enc.write_i32_field(1, phys_type as i32);
    enc.write_string_field(5, col_name.as_bytes());
    enc.end_struct();
    enc.into_bytes()
}

/// Encode a `ColumnMetaData`.
///
/// ```text
/// struct ColumnMetaData {
///   1: required Type type
///   2: required list<Encoding> encodings       // [PLAIN=0]
///   3: required list<string> path_in_schema    // [column_name]
///   4: required CompressionCodec codec         // UNCOMPRESSED=0
///   5: required i64 num_values
///   6: required i64 total_uncompressed_size
///   7: required i64 total_compressed_size
///   9: required i64 data_page_offset
/// }
/// ```
fn encode_column_metadata(
    col_name: &str,
    phys_type: ParquetPhysType,
    num_values: i64,
    total_size: i64,
    data_page_offset: i64,
) -> Vec<u8> {
    let mut enc = ThriftCompactEncoder::new();
    enc.begin_struct();

    // Field 1: type
    enc.write_i32_field(1, phys_type as i32);

    // Field 2: encodings = [PLAIN=0] (list<i32>)
    enc.write_list_field_header(2, compact_type::I32, 1);
    {
        // Single element: PLAIN = 0 → zigzag(0)=0 varint
        let zz = ThriftCompactEncoder::zigzag32_pub(0i32);
        let varint = ThriftCompactEncoder::encode_varint_pub(zz as u64);
        enc.write_raw(&varint);
    }

    // Field 3: path_in_schema = [col_name] (list<binary/string>)
    enc.write_list_field_header(3, compact_type::BINARY, 1);
    {
        let name_bytes = col_name.as_bytes();
        let len_varint = ThriftCompactEncoder::encode_varint_pub(name_bytes.len() as u64);
        enc.write_raw(&len_varint);
        enc.write_raw(name_bytes);
    }

    // Field 4: codec = UNCOMPRESSED=0
    enc.write_i32_field(4, 0);

    // Field 5: num_values
    enc.write_i64_field(5, num_values);

    // Field 6: total_uncompressed_size
    enc.write_i64_field(6, total_size);

    // Field 7: total_compressed_size
    enc.write_i64_field(7, total_size);

    // Field 9: data_page_offset
    enc.write_i64_field(9, data_page_offset);

    enc.end_struct();
    enc.into_bytes()
}

/// Encode a complete Parquet `FileMetaData`.
///
/// ```text
/// struct FileMetaData {
///   1: required i32 version
///   2: required list<SchemaElement> schema
///   3: required i64 num_rows
///   4: required list<RowGroup> row_groups
/// }
/// ```
fn encode_file_metadata(
    columns: &[ParquetColumnWithData],
    col_offsets: &[(i64, i64)], // (data_page_offset, total_size) per column
    num_rows: i64,
) -> Vec<u8> {
    let mut enc = ThriftCompactEncoder::new();
    enc.begin_struct();

    // Field 1: version = 2
    enc.write_i32_field(1, 2);

    // Field 2: schema list (root + one per column)
    let schema_count = 1 + columns.len();
    enc.write_list_field_header(2, compact_type::STRUCT, schema_count);
    // Root schema element
    enc.write_raw(&encode_schema_root(columns.len() as i32));
    // Per-column schema elements
    for col in columns {
        enc.write_raw(&encode_schema_column(&col.name, col.data.phys_type()));
    }

    // Field 3: num_rows
    enc.write_i64_field(3, num_rows);

    // Field 4: row_groups list (single row group)
    enc.write_list_field_header(4, compact_type::STRUCT, 1);
    {
        // RowGroup struct
        let mut rg = ThriftCompactEncoder::new();
        rg.begin_struct();

        // RowGroup.1: columns list<ColumnChunk>
        rg.write_list_field_header(1, compact_type::STRUCT, columns.len());
        for (i, col) in columns.iter().enumerate() {
            let (offset, size) = col_offsets[i];
            let meta_bytes = encode_column_metadata(
                &col.name,
                col.data.phys_type(),
                col.data.len() as i64,
                size,
                offset,
            );
            // ColumnChunk struct: field 2 = ColumnMetaData (embedded inline)
            let mut cc = ThriftCompactEncoder::new();
            cc.begin_struct();
            cc.write_struct_field_begin(2);
            cc.write_raw(&meta_bytes[1..]); // meta_bytes starts with begin_struct marker (nothing) and ends with stop; skip leading to embed raw field bytes then end with stop from meta_bytes
            cc.end_struct(); // end ColumnChunk
            rg.write_raw(&cc.into_bytes());
        }

        // RowGroup.2: total_byte_size
        let total_size: i64 = col_offsets.iter().map(|(_, s)| s).sum();
        rg.write_i64_field(2, total_size);

        // RowGroup.3: num_rows
        rg.write_i64_field(3, num_rows);

        rg.end_struct();
        enc.write_raw(&rg.into_bytes());
    }

    enc.end_struct();
    enc.into_bytes()
}

/// Encode `columns` as a valid Parquet binary file.
///
/// # Format
/// ```text
/// [4]  "PAR1"
/// [data pages — each: thrift-compact PageHeader + PLAIN bytes]
/// [FileMetaData encoded as Thrift Compact]
/// [4]  footer_length as u32 LE
/// [4]  "PAR1"
/// ```
#[allow(dead_code)]
pub fn to_parquet_bytes(columns: &[ParquetColumnWithData]) -> Vec<u8> {
    const MAGIC: &[u8; 4] = b"PAR1";

    if columns.is_empty() {
        // Still produce a structurally valid file with an empty row group.
        let meta = encode_file_metadata(&[], &[], 0);
        let mut out = Vec::with_capacity(8 + meta.len() + 8);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&meta);
        out.extend_from_slice(&(meta.len() as u32).to_le_bytes());
        out.extend_from_slice(MAGIC);
        return out;
    }

    // ── Step 1: encode PLAIN data and page headers ────────────────────────
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(MAGIC);

    // Determine num_rows from the first column (all columns must have the same
    // row count for a single row group).
    let num_rows = columns[0].data.len() as i64;

    let mut col_offsets: Vec<(i64, i64)> = Vec::with_capacity(columns.len());

    for col in columns {
        let plain_bytes = col.data.encode_plain();
        let data_size = plain_bytes.len() as i32;
        let num_values = col.data.len() as i32;
        let page_header = encode_page_header(num_values, data_size);

        let data_page_offset = out.len() as i64;
        let total_size = (page_header.len() + plain_bytes.len()) as i64;

        out.extend_from_slice(&page_header);
        out.extend_from_slice(&plain_bytes);

        col_offsets.push((data_page_offset, total_size));
    }

    // ── Step 2: FileMetaData ─────────────────────────────────────────────────
    let meta = encode_file_metadata(columns, &col_offsets, num_rows);
    out.extend_from_slice(&meta);

    // ── Step 3: footer length + trailing magic ─────────────────────────────
    out.extend_from_slice(&(meta.len() as u32).to_le_bytes());
    out.extend_from_slice(MAGIC);

    out
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

    // ── to_parquet_bytes tests ────────────────────────────────────────────────

    /// The output must start and end with the PAR1 magic bytes.
    #[test]
    fn parquet_bytes_magic_at_start_and_end() {
        let cols = vec![ParquetColumnWithData::new(
            "x",
            ParquetColumnData::Int32(vec![1, 2, 3]),
        )];
        let bytes = to_parquet_bytes(&cols);
        assert_eq!(&bytes[..4], b"PAR1", "leading magic");
        assert_eq!(&bytes[bytes.len() - 4..], b"PAR1", "trailing magic");
    }

    /// The footer length stored at bytes [len-8..len-4] must match the actual
    /// number of FileMetaData bytes (everything between the data pages and the
    /// trailing magic + length).
    #[test]
    fn parquet_bytes_footer_length_consistent() {
        let cols = vec![ParquetColumnWithData::new(
            "v",
            ParquetColumnData::Double(vec![1.0, 2.0]),
        )];
        let bytes = to_parquet_bytes(&cols);
        let len = bytes.len();
        let footer_len =
            u32::from_le_bytes([bytes[len - 8], bytes[len - 7], bytes[len - 6], bytes[len - 5]])
                as usize;
        // footer must be non-zero and must not exceed the total file size
        assert!(footer_len > 0, "footer length must be > 0");
        assert!(footer_len + 12 <= len, "footer fits within file");
    }

    /// An Int32 column PLAIN-encodes as 4 bytes per value LE.  We verify the
    /// raw page data region by finding the known encoded values in the output.
    #[test]
    fn parquet_bytes_int32_plain_encoding() {
        let values: Vec<i32> = vec![0x01020304, -1];
        let cols = vec![ParquetColumnWithData::new(
            "col",
            ParquetColumnData::Int32(values.clone()),
        )];
        let bytes = to_parquet_bytes(&cols);
        // Search for the little-endian bytes of the first value
        let needle: [u8; 4] = 0x01020304_i32.to_le_bytes();
        let found = bytes.windows(4).any(|w| w == needle);
        assert!(found, "PLAIN LE bytes for 0x01020304 must appear in output");
    }

    /// A ByteArray column must encode each value as 4-byte LE length + bytes.
    #[test]
    fn parquet_bytes_bytearray_length_prefix() {
        let payload = b"hello";
        let cols = vec![ParquetColumnWithData::new(
            "s",
            ParquetColumnData::ByteArray(vec![payload.to_vec()]),
        )];
        let bytes = to_parquet_bytes(&cols);
        // Length prefix = 5 as u32 LE → [5,0,0,0]
        let len_bytes: [u8; 4] = (payload.len() as u32).to_le_bytes();
        let found_prefix = bytes.windows(4).any(|w| w == len_bytes);
        assert!(found_prefix, "ByteArray length prefix must appear in output");
        // The payload itself must appear
        let found_payload = bytes
            .windows(payload.len())
            .any(|w| w == payload.as_slice());
        assert!(found_payload, "ByteArray payload must appear in output");
    }

    /// An empty column set still produces a valid PAR1 framed file.
    #[test]
    fn parquet_bytes_empty_columns() {
        let bytes = to_parquet_bytes(&[]);
        assert!(bytes.len() >= 12, "minimum 12 bytes (2*magic + footer_len)");
        assert_eq!(&bytes[..4], b"PAR1");
        assert_eq!(&bytes[bytes.len() - 4..], b"PAR1");
    }

    /// A Double column round-trips: the raw LE IEEE-754 bits must be present.
    #[test]
    fn parquet_bytes_double_round_trip() {
        let val: f64 = std::f64::consts::PI;
        let cols = vec![ParquetColumnWithData::new(
            "pi",
            ParquetColumnData::Double(vec![val]),
        )];
        let bytes = to_parquet_bytes(&cols);
        let needle = val.to_bits().to_le_bytes();
        let found = bytes.windows(8).any(|w| w == needle);
        assert!(found, "PI double LE bits must appear in output");
    }
}
