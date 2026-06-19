// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! FlatBuffers schema (.fbs) text generator **and** real binary encoder.
//!
//! Binary layout follows the FlatBuffers wire-format spec:
//! <https://flatbuffers.dev/internals/>
//!
//! All integers are little-endian.  The root object is located by the u32 at
//! bytes \[0..4\] of every finished buffer.
//!
//! # Buffer layout (append order used by `FbBinaryBuilder`)
//!
//! ```text
//! [root-offset placeholder: u32] [data: strings / vectors / ...] [object body: soffset(i32) | inline fields...] [vtable: vtable_size(u16) | object_size(u16) | field_off_0(u16) | ...]
//! ```
//!
//! * The **root offset** (bytes 0–3) is patched last to point at the object.
//! * String / vector payloads are written **before** the table object; the
//!   relative offsets stored in object fields are therefore negative (pointing
//!   backwards).
//! * The **soffset** (first four bytes of the object) is a signed i32 whose
//!   value is `vtable_address - object_address` (positive because we write the
//!   vtable immediately after the object body).

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

// ── FlatBuffers binary encoder ────────────────────────────────────────────────

/// Low-level byte buffer used to build a FlatBuffers binary front-to-back.
///
/// The layout is:
/// ```text
/// [root-offset(u32)] [prelude data: strings / vectors]
/// [object: soffset(i32) | inline fields...] [vtable]
/// ```
/// After calling [`FbBinaryBuilder::finish`] the root-offset at offset 0 is
/// patched to point at the object.
#[allow(dead_code)]
struct FbBinaryBuilder {
    buf: Vec<u8>,
}

#[allow(dead_code)]
impl FbBinaryBuilder {
    /// Create a new builder.  Reserves 4 bytes for the root-offset placeholder.
    fn new() -> Self {
        let mut buf = Vec::with_capacity(256);
        // Placeholder for root offset; will be back-patched in `finish`.
        buf.extend_from_slice(&[0u8; 4]);
        FbBinaryBuilder { buf }
    }

    /// Current write position (number of bytes accumulated so far).
    fn cursor(&self) -> usize {
        self.buf.len()
    }

    /// Append a single byte.
    fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    /// Append a u16 little-endian.
    fn write_u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Append a u32 little-endian.
    fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Append a signed i32 little-endian.
    fn write_i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    /// Append a raw byte slice.
    fn write_bytes(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }

    /// Pad with zero bytes until `self.cursor()` is a multiple of `align`.
    fn align_to(&mut self, align: usize) {
        while !self.buf.len().is_multiple_of(align) {
            self.buf.push(0);
        }
    }

    /// Back-patch a u32 little-endian at an existing absolute offset.
    fn patch_u32(&mut self, offset: usize, v: u32) {
        let bytes = v.to_le_bytes();
        self.buf[offset..offset + 4].copy_from_slice(&bytes);
    }

    /// Back-patch an i32 little-endian at an existing absolute offset.
    fn patch_i32(&mut self, offset: usize, v: i32) {
        let bytes = v.to_le_bytes();
        self.buf[offset..offset + 4].copy_from_slice(&bytes);
    }

    /// Write a FlatBuffers string: u32 length + UTF-8 bytes + NUL terminator,
    /// then pad to 4-byte alignment.  Returns the absolute offset of the length
    /// prefix (i.e. the "pointer target" for references to this string).
    fn write_fbs_string(&mut self, s: &str) -> usize {
        // Align to 4 before writing so length prefix is aligned.
        self.align_to(4);
        let pos = self.cursor();
        self.write_u32(s.len() as u32);
        self.write_bytes(s.as_bytes());
        self.write_u8(0); // NUL terminator
        self.align_to(4);
        pos
    }

    /// Write a vector of u32: u32 length prefix + element words.
    /// Returns the absolute offset of the length prefix.
    fn write_u32_vec(&mut self, data: &[u32]) -> usize {
        self.align_to(4);
        let pos = self.cursor();
        self.write_u32(data.len() as u32);
        for v in data {
            self.write_u32(*v);
        }
        self.align_to(4);
        pos
    }

    /// Write a vector of f32: u32 length prefix + element words.
    /// Returns the absolute offset of the length prefix.
    fn write_f32_vec(&mut self, data: &[f32]) -> usize {
        self.align_to(4);
        let pos = self.cursor();
        self.write_u32(data.len() as u32);
        for v in data {
            self.write_bytes(&v.to_le_bytes());
        }
        self.align_to(4);
        pos
    }

    /// Finish the buffer.
    ///
    /// Writes the object (soffset placeholder + `fields`), the vtable, and
    /// back-patches the root offset and the soffset.
    ///
    /// `fields` is a list of `(slot_index, Option<data_offset>)` pairs.  Each
    /// entry represents one vtable slot:
    ///
    /// * `None`  — field absent; vtable entry is 0.
    /// * `Some(absolute_target)` — the field value is the i32 relative offset
    ///   `absolute_target - field_position`; vtable entry records the
    ///   field's byte position within the object.
    ///
    /// For **inline u32** fields (not an offset to a prior blob) pass the
    /// absolute address of their own inline storage location; those are filled
    /// in via `inline_u32_fields`.
    ///
    /// Returns the finished buffer.
    fn finish(
        mut self,
        offset_fields: &[(usize, Option<usize>)], // (slot, Some(target_abs))
        inline_u32_fields: &[(usize, u32)],        // (slot, value)
    ) -> Vec<u8> {
        // ------------------------------------------------------------------
        // Phase 1: write the object body.
        // Object layout: [soffset(i32)] [field_0(4 bytes)] ... [field_N(4 bytes)]
        // All fields are 4 bytes: either an i32 relative offset or a u32 value.
        // ------------------------------------------------------------------

        // Total number of vtable slots = max(slot_index) + 1 across both lists.
        let num_slots = {
            let max_off = offset_fields.iter().map(|(s, _)| *s).max().unwrap_or(0);
            let max_inline = inline_u32_fields.iter().map(|(s, _)| *s).max().unwrap_or(0);
            if offset_fields.is_empty() && inline_u32_fields.is_empty() {
                0
            } else {
                max_off.max(max_inline) + 1
            }
        };

        // Determine which slots are present and sort them by slot index so we
        // can emit field data in slot order.
        // Slot → absolute target offset (None = absent, Some(usize) = ref target or inline addr).
        let mut slot_target: Vec<Option<SlotKind>> = vec![None; num_slots];
        for &(slot, target_opt) in offset_fields {
            slot_target[slot] = target_opt.map(SlotKind::Offset);
        }
        for &(slot, value) in inline_u32_fields {
            slot_target[slot] = Some(SlotKind::Inline(value));
        }

        // Align the buffer before the object (FlatBuffers objects are 4-byte aligned).
        self.align_to(4);
        let object_start = self.cursor();

        // Write soffset placeholder (will be back-patched after writing vtable).
        self.write_i32(0i32);

        // Track per-slot field offsets relative to object_start (for vtable).
        let mut vtable_field_offsets: Vec<u16> = vec![0u16; num_slots];

        for (slot, kind_opt) in slot_target.iter().enumerate() {
            match kind_opt {
                None => {
                    // Absent field: no bytes written; vtable entry stays 0.
                }
                Some(SlotKind::Offset(target_abs)) => {
                    // The field stores the relative offset from its own position
                    // to the target blob (string or vector).
                    let field_abs = self.cursor();
                    let field_off_in_object = (field_abs - object_start) as u16;
                    vtable_field_offsets[slot] = field_off_in_object;
                    let rel: i32 = *target_abs as i32 - field_abs as i32;
                    self.write_i32(rel);
                }
                Some(SlotKind::Inline(value)) => {
                    let field_abs = self.cursor();
                    let field_off_in_object = (field_abs - object_start) as u16;
                    vtable_field_offsets[slot] = field_off_in_object;
                    self.write_u32(*value);
                }
            }
        }

        let object_end = self.cursor();
        let object_size = (object_end - object_start) as u16;

        // ------------------------------------------------------------------
        // Phase 2: write the vtable immediately after the object.
        // vtable layout: [vtable_size(u16)] [object_size(u16)] [field_off_0(u16)] ...
        // ------------------------------------------------------------------
        self.align_to(2);
        let vtable_start = self.cursor();

        let vtable_size = (4 + 2 * num_slots) as u16;
        self.write_u16(vtable_size);
        self.write_u16(object_size);
        for &fo in &vtable_field_offsets {
            self.write_u16(fo);
        }

        // ------------------------------------------------------------------
        // Phase 3: back-patch soffset and root offset.
        // ------------------------------------------------------------------

        // soffset at object_start = vtable_start - object_start (positive, vtable after object).
        let soffset: i32 = vtable_start as i32 - object_start as i32;
        self.patch_i32(object_start, soffset);

        // Root offset at byte 0 = absolute position of the object start.
        self.patch_u32(0, object_start as u32);

        self.buf
    }
}

/// Discriminates between the two kinds of values a vtable slot can carry.
#[allow(dead_code)]
#[derive(Clone)]
enum SlotKind {
    /// An i32 relative offset pointing to a blob (string / vector) that was
    /// written before the object.
    Offset(usize),
    /// An inline u32 value stored directly in the object body.
    Inline(u32),
}

// ── Public binary encoding API ────────────────────────────────────────────────

/// Encode a [`FlatbufExport`] descriptor as a FlatBuffers binary buffer.
///
/// The produced binary contains a single root table with three fields:
///
/// | slot | name          | type           |
/// |------|---------------|----------------|
/// |  0   | `namespace`   | string (offset)|
/// |  1   | `table_count` | u32 (inline)   |
/// |  2   | `root_type`   | string (offset)|
///
/// Strings are encoded as `u32` length + UTF-8 bytes + NUL + 4-byte padding.
#[allow(dead_code)]
pub fn to_flatbuf_bytes(doc: &FlatbufExport) -> Vec<u8> {
    let mut builder = FbBinaryBuilder::new();

    // Write strings first (they will be at lower addresses than the object,
    // so all relative offsets from the object fields to the strings are negative).
    let ns_abs = builder.write_fbs_string(&doc.namespace);
    let rt_abs = builder.write_fbs_string(&doc.root_type);

    // Inline u32 field value.
    let table_count = doc.tables.len() as u32;

    // offset_fields: (slot, Some(absolute_target))
    let offset_fields: &[(usize, Option<usize>)] =
        &[(0, Some(ns_abs)), (2, Some(rt_abs))];

    // inline_u32_fields: (slot, value)
    let inline_fields: &[(usize, u32)] = &[(1, table_count)];

    builder.finish(offset_fields, inline_fields)
}

/// Encode mesh geometry as a FlatBuffers binary buffer.
///
/// The produced binary contains a single root table with two fields:
///
/// | slot | name        | type                  |
/// |------|-------------|-----------------------|
/// |  0   | `positions` | \[float32\] (vector offset)|
/// |  1   | `indices`   | \[uint32\] (vector offset) |
///
/// `positions` is the row-major flattening of `[[f32; 3]]` into a `[f32]`
/// vector.  `indices` is stored as-is.
#[allow(dead_code)]
pub fn flatbuf_encode_mesh(positions: &[[f32; 3]], indices: &[u32]) -> Vec<u8> {
    let mut builder = FbBinaryBuilder::new();

    // Flatten positions: [[f32; 3]] → Vec<f32>.
    let flat_pos: Vec<f32> = positions
        .iter()
        .flat_map(|p| p.iter().copied())
        .collect();

    // Write vectors before the table so their absolute offsets are known.
    let pos_abs = builder.write_f32_vec(&flat_pos);
    let idx_abs = builder.write_u32_vec(indices);

    let offset_fields: &[(usize, Option<usize>)] =
        &[(0, Some(pos_abs)), (1, Some(idx_abs))];

    builder.finish(offset_fields, &[])
}

// ── Read-back helpers (used only in tests) ────────────────────────────────────

/// Read a u32 little-endian from `buf` at `offset`.  Returns `None` if the
/// slice is too short.
#[allow(dead_code)]
fn read_u32_at(buf: &[u8], offset: usize) -> Option<u32> {
    let bytes: [u8; 4] = buf.get(offset..offset + 4)?.try_into().ok()?;
    Some(u32::from_le_bytes(bytes))
}

/// Follow the FlatBuffers root offset and soffset to locate the vtable, then
/// read back the absolute address of the target blob for a given vtable slot.
///
/// Works for offset-type fields only.  Returns `None` if the slot is absent or
/// the buffer is malformed.
#[allow(dead_code)]
fn resolve_offset_field(buf: &[u8], slot: usize) -> Option<usize> {
    // Root offset: absolute address of the object.
    let root_off = read_u32_at(buf, 0)? as usize;

    // soffset at object_start: i32 LE.
    let soffset_bytes: [u8; 4] = buf.get(root_off..root_off + 4)?.try_into().ok()?;
    let soffset = i32::from_le_bytes(soffset_bytes);
    let vtable_off = (root_off as i64 + soffset as i64) as usize;

    // vtable header.
    let vtable_size = u16::from_le_bytes(buf.get(vtable_off..vtable_off + 2)?.try_into().ok()?) as usize;
    let slot_entry_off = vtable_off + 4 + slot * 2;
    if slot_entry_off + 2 > vtable_off + vtable_size {
        return None;
    }
    let field_off_in_obj =
        u16::from_le_bytes(buf.get(slot_entry_off..slot_entry_off + 2)?.try_into().ok()?) as usize;
    if field_off_in_obj == 0 {
        return None; // absent field
    }

    // Absolute position of the field itself.
    let field_abs = root_off + field_off_in_obj;

    // The stored i32 is `target - field_abs`.
    let rel_bytes: [u8; 4] = buf.get(field_abs..field_abs + 4)?.try_into().ok()?;
    let rel = i32::from_le_bytes(rel_bytes);
    Some((field_abs as i64 + rel as i64) as usize)
}

/// Read a FlatBuffers string at absolute offset `str_off`.
/// Layout: u32 length + UTF-8 bytes (no NUL counted in length).
#[allow(dead_code)]
fn read_fbs_string(buf: &[u8], str_off: usize) -> Option<String> {
    let len = read_u32_at(buf, str_off)? as usize;
    let bytes = buf.get(str_off + 4..str_off + 4 + len)?;
    String::from_utf8(bytes.to_vec()).ok()
}

/// Read the inline u32 for a vtable slot (for `table_count`, slot 1).
#[allow(dead_code)]
fn read_inline_u32_field(buf: &[u8], slot: usize) -> Option<u32> {
    let root_off = read_u32_at(buf, 0)? as usize;
    let soffset_bytes: [u8; 4] = buf.get(root_off..root_off + 4)?.try_into().ok()?;
    let soffset = i32::from_le_bytes(soffset_bytes);
    let vtable_off = (root_off as i64 + soffset as i64) as usize;
    let vtable_size = u16::from_le_bytes(buf.get(vtable_off..vtable_off + 2)?.try_into().ok()?) as usize;
    let slot_entry_off = vtable_off + 4 + slot * 2;
    if slot_entry_off + 2 > vtable_off + vtable_size {
        return None;
    }
    let field_off_in_obj =
        u16::from_le_bytes(buf.get(slot_entry_off..slot_entry_off + 2)?.try_into().ok()?) as usize;
    if field_off_in_obj == 0 {
        return None;
    }
    read_u32_at(buf, root_off + field_off_in_obj)
}

/// Return the u32 element count stored at the head of a FlatBuffers vector.
/// `vec_abs` is the absolute byte offset of the length prefix.
#[allow(dead_code)]
fn read_vec_count(buf: &[u8], vec_abs: usize) -> Option<u32> {
    read_u32_at(buf, vec_abs)
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

    // ── Binary encoder tests ──────────────────────────────────────────────────

    /// `flatbuf_encode_mesh` must produce a non-empty buffer for non-trivial
    /// input.
    #[test]
    fn test_flatbuf_encode_mesh_non_empty() {
        let positions: &[[f32; 3]] = &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let indices: &[u32] = &[0, 1, 2];
        let buf = flatbuf_encode_mesh(positions, indices);
        assert!(!buf.is_empty(), "encoded buffer must not be empty");
    }

    /// The first 4 bytes of the encoded mesh buffer must be a valid root offset
    /// (i.e. strictly less than the buffer length and at least 4 to skip the
    /// offset word itself).
    #[test]
    fn test_flatbuf_encode_mesh_root_offset() {
        let positions: &[[f32; 3]] = &[
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
        ];
        let indices: &[u32] = &[0, 1];
        let buf = flatbuf_encode_mesh(positions, indices);

        let root_off = read_u32_at(&buf, 0).expect("buffer has at least 4 bytes") as usize;
        assert!(
            root_off >= 4,
            "root offset {root_off} must be >= 4 (past the offset word)",
        );
        assert!(
            root_off < buf.len(),
            "root offset {root_off} must be inside the buffer (len={})",
            buf.len(),
        );
    }

    /// `to_flatbuf_bytes` for a doc whose namespace is `"oxihuman"` must encode
    /// that string so that it can be read back via the vtable.
    #[test]
    fn test_to_flatbuf_bytes_namespace() {
        let mut doc = new_flatbuf_export("oxihuman");
        add_fbs_table(&mut doc, "Mesh", false);
        set_fbs_root_type(&mut doc, "Mesh");
        let buf = to_flatbuf_bytes(&doc);

        assert!(!buf.is_empty(), "encoded buffer must not be empty");

        // Slot 0 is the namespace string — resolve and decode it.
        let str_abs = resolve_offset_field(&buf, 0)
            .expect("namespace field (slot 0) must be present and point to a valid target");
        let ns = read_fbs_string(&buf, str_abs)
            .expect("namespace string must be valid UTF-8");
        assert_eq!(ns, "oxihuman", "decoded namespace must match the original");
    }

    /// `flatbuf_encode_mesh` for 5 indices must encode a vector whose length
    /// prefix equals 5 somewhere in the buffer.
    #[test]
    fn test_flatbuf_encode_mesh_indices() {
        let positions: &[[f32; 3]] = &[
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        let indices: &[u32] = &[0, 1, 2, 2, 3];
        let buf = flatbuf_encode_mesh(positions, indices);

        // Follow the vtable to find the indices vector (slot 1).
        let vec_abs = resolve_offset_field(&buf, 1)
            .expect("indices field (slot 1) must be present");
        let count = read_vec_count(&buf, vec_abs)
            .expect("vector length prefix must be readable");
        assert_eq!(count, 5, "index count encoded in the vector must be 5");
    }

    /// Round-trip: `to_flatbuf_bytes` must encode `table_count` (slot 1)
    /// correctly for a doc with 3 tables.
    #[test]
    fn test_to_flatbuf_bytes_table_count() {
        let mut doc = new_flatbuf_export("ns");
        add_fbs_table(&mut doc, "A", false);
        add_fbs_table(&mut doc, "B", false);
        add_fbs_table(&mut doc, "C", false);
        let buf = to_flatbuf_bytes(&doc);

        let count = read_inline_u32_field(&buf, 1)
            .expect("table_count field (slot 1) must be present");
        assert_eq!(count, 3, "table_count must equal the number of tables added");
    }

    /// `flatbuf_encode_mesh` encodes positions correctly: the positions vector
    /// length must equal 3 × number of input vertices.
    #[test]
    fn test_flatbuf_encode_mesh_positions_length() {
        let positions: &[[f32; 3]] = &[
            [1.0, 2.0, 3.0],
            [4.0, 5.0, 6.0],
            [7.0, 8.0, 9.0],
            [10.0, 11.0, 12.0],
        ];
        let indices: &[u32] = &[0, 1, 2, 3];
        let buf = flatbuf_encode_mesh(positions, indices);

        let vec_abs = resolve_offset_field(&buf, 0)
            .expect("positions field (slot 0) must be present");
        let count = read_vec_count(&buf, vec_abs)
            .expect("positions vector length prefix must be readable");
        assert_eq!(
            count as usize,
            positions.len() * 3,
            "positions vector length must be 3 × vertex count",
        );
    }
}
