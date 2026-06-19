// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Cap'n Proto schema text generator **and** real binary encoder.
//!
//! Text output follows the `.capnp` schema syntax.
//! Binary output follows the Cap'n Proto framing wire format:
//! <https://capnproto.org/encoding.html>
//!
//! All multi-byte integers are little-endian.  The segment table header is
//! emitted by [`write_frame`] from `oxihuman_core::capnproto`.

#![allow(dead_code)]

use oxihuman_core::capnproto::{
    encode_list_ptr, encode_struct_ptr, write_frame, CapnMessage, CapnSegment, ElementSize,
    ListPointer, Message, StructPointer,
};

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

// ============================================================================
// Binary encoder — Cap'n Proto wire format
// ============================================================================

/// Encode a [`CapnpExport`] document as a Cap'n Proto binary message.
///
/// ## Wire layout (single segment)
///
/// ```text
/// [framing header: u32 LE (0) — 1 segment; u32 LE — segment word count]
/// [word 0] root struct pointer  — offset=0, data_words=2, pointer_words=0
/// [word 1] doc.file_id          (UInt64 — first data word)
/// [word 2] doc.structs.len()    (UInt64 — struct count in low 32 bits)
/// ```
///
/// The framing header is emitted by `write_frame` so the output is a fully
/// self-contained, parseable Cap'n Proto message.
#[allow(dead_code)]
pub fn to_capnp_bytes(doc: &CapnpExport) -> Vec<u8> {
    // segment capacity: 1 (root ptr) + 2 (data words) = 3 words
    let mut seg = CapnSegment::with_capacity(3);

    // Root struct pointer: the struct body starts immediately after the
    // pointer word (offset_words = 0), contains 2 data words and no pointer
    // section.
    let root_ptr = StructPointer {
        offset_words: 0,
        data_words: 2,
        pointer_words: 0,
    };
    seg.push_word(encode_struct_ptr(&root_ptr));

    // Data word 0: file_id (UInt64).
    seg.push_word(doc.file_id);

    // Data word 1: struct_count packed into the low 32 bits of a 64-bit word.
    // The upper 32 bits are reserved / zero.
    seg.push_word(doc.structs.len() as u64);

    // Build the legacy CapnMessage so we can convert to raw bytes.
    let mut capn_msg = CapnMessage::new();
    capn_msg.add_segment(seg);

    // Convert CapnSegment words → raw bytes for each segment.
    let mut raw_bytes: Vec<u8> = Vec::with_capacity(capn_msg.total_bytes());
    for seg_idx in 0..capn_msg.segment_count() {
        if let Some(s) = capn_msg.segment(seg_idx) {
            for &word in s.words() {
                raw_bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
    }

    // Wrap in the proper Cap'n Proto framing envelope (segment count + word
    // counts header) using the canonical `write_frame` path.
    let framed_msg = Message {
        segments: vec![raw_bytes],
    };
    write_frame(&framed_msg)
}

/// Encode mesh geometry as a Cap'n Proto binary message.
///
/// ## Wire layout (single segment)
///
/// ```text
/// [framing header]
/// [word 0]  root struct pointer — data_words=0, pointer_words=2
/// [word 1]  list pointer → positions data (f32 × positions.len()*3)
/// [word 2]  list pointer → indices data  (u32 × indices.len())
/// [words 3..3+pos_words-1]  positions: f32 values packed 2 per word (LE)
/// [words 3+pos_words..]     indices:   u32 values packed 2 per word (LE)
/// ```
///
/// Positions are encoded as a `List(Float32)` with
/// `element_count = positions.len() * 3` (x, y, z interleaved).
/// Indices are encoded as a `List(UInt32)`.
///
/// All pointer offsets are computed so the output is a parseable
/// Cap'n Proto message.
#[allow(dead_code)]
pub fn export_mesh_capnp_binary(positions: &[[f32; 3]], indices: &[u32]) -> Vec<u8> {
    // ---- Compute layout parameters -----------------------------------------

    // Positions: 3 × f32 per vertex — each f32 is 4 bytes → 2 f32 per word.
    let pos_element_count = positions.len() * 3;
    // Words needed for positions list body: ceil(pos_element_count / 2).
    let pos_words = pos_element_count.div_ceil(2);

    // Indices: 1 × u32 per index — 2 u32 per word.
    let idx_element_count = indices.len();
    let idx_words = idx_element_count.div_ceil(2);

    // ---- Build segment words ------------------------------------------------
    // Capacity: 1 (root ptr) + 2 (ptr section) + pos_words + idx_words
    let capacity = 3 + pos_words + idx_words;
    let mut seg = CapnSegment::with_capacity(capacity);

    // Word 0 — root struct pointer.
    // The struct has data_words=0 and pointer_words=2.  It starts at the word
    // immediately after the root pointer (offset_words=0).
    let root_ptr = StructPointer {
        offset_words: 0,
        data_words: 0,
        pointer_words: 2,
    };
    seg.push_word(encode_struct_ptr(&root_ptr));

    // Words 1..2 — pointer section (two list pointers).
    //
    // The struct pointer section occupies words 1 and 2.  The list data
    // starts at word 3.
    //
    // Pointer at word 1: offset_words = number of words between (word 1 + 1 =
    // word 2) and the first word of the positions list (word 3).
    //   offset = word 3 - word 2 = 1
    let pos_list_ptr = ListPointer {
        offset_words: 1,
        element_size: ElementSize::FourBytes,
        element_count: pos_element_count as u32,
    };
    seg.push_word(encode_list_ptr(&pos_list_ptr));

    // Pointer at word 2: offset_words = number of words between (word 2 + 1 =
    // word 3) and the first word of the indices list (word 3 + pos_words).
    //   offset = (3 + pos_words) - 3 = pos_words
    let idx_list_ptr = ListPointer {
        offset_words: pos_words as i32,
        element_size: ElementSize::FourBytes,
        element_count: idx_element_count as u32,
    };
    seg.push_word(encode_list_ptr(&idx_list_ptr));

    // Words 3..3+pos_words — positions data (f32 values, 2 per word, LE).
    {
        let flat: Vec<f32> = positions
            .iter()
            .flat_map(|&[x, y, z]| [x, y, z])
            .collect();
        let mut i = 0usize;
        while i < flat.len() {
            let lo = flat[i].to_bits() as u64;
            let hi = if i + 1 < flat.len() {
                flat[i + 1].to_bits() as u64
            } else {
                0u64
            };
            seg.push_word(lo | (hi << 32));
            i += 2;
        }
        // Pad to pos_words if flat.len() was odd (already handled by div_ceil).
        // When flat is empty pos_words == 0, no padding needed.
    }

    // Words 3+pos_words.. — indices data (u32 values, 2 per word, LE).
    {
        let mut i = 0usize;
        while i < indices.len() {
            let lo = indices[i] as u64;
            let hi = if i + 1 < indices.len() {
                indices[i + 1] as u64
            } else {
                0u64
            };
            seg.push_word(lo | (hi << 32));
            i += 2;
        }
    }

    // ---- Assemble & frame ---------------------------------------------------
    let mut capn_msg = CapnMessage::new();
    capn_msg.add_segment(seg);

    let mut raw_bytes: Vec<u8> = Vec::with_capacity(capn_msg.total_bytes());
    for seg_idx in 0..capn_msg.segment_count() {
        if let Some(s) = capn_msg.segment(seg_idx) {
            for &word in s.words() {
                raw_bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
    }

    let framed_msg = Message {
        segments: vec![raw_bytes],
    };
    write_frame(&framed_msg)
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
        let st = find_capnp_struct(&doc, "M").expect("should succeed");
        assert_eq!(st.fields[1].index, 1);
    }

    // =========================================================================
    // Binary encoder tests: to_capnp_bytes
    // =========================================================================

    /// The framing header first u32 (LE) encodes `segment_count - 1`.
    /// For a 1-segment message that value is 0.
    #[test]
    fn test_to_capnp_bytes_segment_count_word_is_zero() {
        let doc = new_capnp_export(0xdeadbeef_cafebabe);
        let bytes = to_capnp_bytes(&doc);
        assert!(bytes.len() >= 4, "output must have at least 4 bytes");
        let seg_count_minus_one =
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(seg_count_minus_one, 0, "single-segment framing: count-1 == 0");
    }

    /// Output must be non-empty for any document, including the default empty one.
    #[test]
    fn test_to_capnp_bytes_non_empty_for_empty_doc() {
        let doc = CapnpExport::default();
        let bytes = to_capnp_bytes(&doc);
        assert!(!bytes.is_empty(), "to_capnp_bytes must produce at least some bytes");
    }

    /// Encoding is deterministic: same document → same bytes.
    #[test]
    fn test_to_capnp_bytes_deterministic() {
        let mut doc = new_capnp_export(0x1111_2222_3333_4444);
        add_capnp_struct(&mut doc, "Bone");
        add_capnp_field(&mut doc, "length", "Float32");
        let bytes_a = to_capnp_bytes(&doc);
        let bytes_b = to_capnp_bytes(&doc);
        assert_eq!(bytes_a, bytes_b, "encoding must be deterministic");
    }

    /// The segment body starts with the root struct pointer word.
    /// For data_words=2, pointer_words=0, offset_words=0 the encoded pointer
    /// must have bits [1:0] = 0b00 (struct kind) and bits [47:32] = 2.
    #[test]
    fn test_to_capnp_bytes_root_struct_ptr_kind_bits() {
        let doc = new_capnp_export(1);
        let bytes = to_capnp_bytes(&doc);
        // 1-segment message: header = u32(0) + u32(word_count) = 8 bytes.
        // Segment body starts at byte 8.
        assert!(bytes.len() >= 16, "must have header (8 B) + at least 1 word");
        let ptr_word = u64::from_le_bytes(
            bytes[8..16].try_into().expect("slice must be exactly 8 bytes"),
        );
        // Bits [1:0] must be 0b00 for a struct pointer.
        assert_eq!(ptr_word & 0b11, 0b00, "root pointer kind must be struct (0b00)");
        // Bits [47:32]: data_words = 2.
        let data_words = ((ptr_word >> 32) & 0xFFFF) as u16;
        assert_eq!(data_words, 2, "root struct must have data_words=2");
        // Bits [63:48]: pointer_words = 0.
        let pointer_words = (ptr_word >> 48) as u16;
        assert_eq!(pointer_words, 0, "root struct must have pointer_words=0");
    }

    /// The file_id is placed at the first data word (word 1 inside the
    /// segment, which is byte 8 after the root pointer).
    #[test]
    fn test_to_capnp_bytes_file_id_word() {
        let file_id: u64 = 0xfeed_face_dead_beef;
        let doc = new_capnp_export(file_id);
        let bytes = to_capnp_bytes(&doc);
        // Header: 8 bytes. Segment word 0 (root ptr): 8 bytes. Word 1 = byte 16.
        assert!(bytes.len() >= 24, "must have at least 3 words + header");
        let word1 = u64::from_le_bytes(
            bytes[16..24].try_into().expect("slice must be exactly 8 bytes"),
        );
        assert_eq!(word1, file_id, "data word 0 must equal file_id");
    }

    /// The struct count is encoded in the low 32 bits of the second data word.
    #[test]
    fn test_to_capnp_bytes_struct_count_word() {
        let mut doc = new_capnp_export(1);
        add_capnp_struct(&mut doc, "A");
        add_capnp_struct(&mut doc, "B");
        add_capnp_struct(&mut doc, "C");
        let bytes = to_capnp_bytes(&doc);
        // Header: 8 bytes. Word 0 (root ptr): bytes 8-15. Word 1 (file_id): 16-23.
        // Word 2 (struct count): bytes 24-31.
        assert!(bytes.len() >= 32, "must have at least 4 words + header");
        let word2 = u64::from_le_bytes(
            bytes[24..32].try_into().expect("slice must be exactly 8 bytes"),
        );
        // Struct count = 3 in low 32 bits.
        assert_eq!(word2 & 0xFFFF_FFFF, 3, "low 32 bits of word 2 must be struct count");
    }

    /// A document with file_id=0 and no structs must still produce a valid
    /// single-segment framing.
    #[test]
    fn test_to_capnp_bytes_zero_file_id_no_structs() {
        let doc = CapnpExport::default();
        let bytes = to_capnp_bytes(&doc);
        // Minimum valid output: 8-byte header + 3 words (24 bytes) = 32 bytes.
        assert!(bytes.len() >= 32, "minimum valid output is 32 bytes");
        let seg_count_minus_one =
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(seg_count_minus_one, 0);
        // Segment word count declared in header must match actual segment length.
        let declared_words =
            u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
        let actual_words = (bytes.len() - 8) / 8;
        assert_eq!(declared_words, actual_words, "declared word count must match body size");
    }

    // =========================================================================
    // Binary encoder tests: export_mesh_capnp_binary
    // =========================================================================

    /// Output is non-empty for any mesh (even empty ones).
    #[test]
    fn test_mesh_binary_non_empty() {
        let positions: &[[f32; 3]] = &[];
        let indices: &[u32] = &[];
        let bytes = export_mesh_capnp_binary(positions, indices);
        assert!(!bytes.is_empty());
    }

    /// Single-segment framing: first u32 LE must be 0.
    #[test]
    fn test_mesh_binary_single_segment_framing() {
        let positions = [[0.0f32, 1.0, 2.0], [3.0, 4.0, 5.0]];
        let indices = [0u32, 1, 0];
        let bytes = export_mesh_capnp_binary(&positions, &indices);
        assert!(bytes.len() >= 4);
        let seg_count_minus_one =
            u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(seg_count_minus_one, 0);
    }

    /// Root struct pointer must have data_words=0, pointer_words=2, kind=struct.
    #[test]
    fn test_mesh_binary_root_ptr_shape() {
        let positions = [[1.0f32, 0.0, 0.0]];
        let indices = [0u32];
        let bytes = export_mesh_capnp_binary(&positions, &indices);
        // Header = 8 bytes. Root ptr at byte 8.
        assert!(bytes.len() >= 16);
        let ptr_word = u64::from_le_bytes(
            bytes[8..16].try_into().expect("slice 8 bytes"),
        );
        assert_eq!(ptr_word & 0b11, 0b00, "root ptr must be struct kind");
        let data_words = ((ptr_word >> 32) & 0xFFFF) as u16;
        assert_eq!(data_words, 0, "no data section in mesh root struct");
        let pointer_words = (ptr_word >> 48) as u16;
        assert_eq!(pointer_words, 2, "mesh root struct has 2 pointer words");
    }

    /// Position list pointer at word 1 of segment must have kind=list (bits[1:0]=0b01).
    #[test]
    fn test_mesh_binary_pos_list_ptr_kind() {
        let positions = [[0.5f32, 0.5, 0.5]];
        let indices = [0u32];
        let bytes = export_mesh_capnp_binary(&positions, &indices);
        // Header 8 bytes. Segment word 1 (pos list ptr) at byte 16.
        assert!(bytes.len() >= 24);
        let ptr_word = u64::from_le_bytes(
            bytes[16..24].try_into().expect("slice 8 bytes"),
        );
        assert_eq!(ptr_word & 0b11, 0b01, "positions list pointer kind must be 0b01");
    }

    /// Index list pointer at word 2 of segment must have kind=list.
    #[test]
    fn test_mesh_binary_idx_list_ptr_kind() {
        let positions = [[0.0f32, 0.0, 0.0]];
        let indices = [0u32, 1, 2];
        let bytes = export_mesh_capnp_binary(&positions, &indices);
        // Header 8 bytes. Segment word 2 (idx list ptr) at byte 24.
        assert!(bytes.len() >= 32);
        let ptr_word = u64::from_le_bytes(
            bytes[24..32].try_into().expect("slice 8 bytes"),
        );
        assert_eq!(ptr_word & 0b11, 0b01, "indices list pointer kind must be 0b01");
    }

    /// Empty positions and indices → positions list element_count = 0
    /// and indices list element_count = 0.
    #[test]
    fn test_mesh_binary_empty_mesh_element_counts() {
        let bytes = export_mesh_capnp_binary(&[], &[]);
        // Header 8 B. Words 0,1,2 of segment. Byte offset of pos list ptr = 8+8 = 16.
        assert!(bytes.len() >= 32);
        // Positions list ptr at byte 16.
        let pos_ptr = u64::from_le_bytes(bytes[16..24].try_into().expect("8 bytes"));
        // element_count in bits [63:35] → high32 >> 3.
        let pos_count = ((pos_ptr >> 32) >> 3) & 0x1FFF_FFFF;
        assert_eq!(pos_count, 0, "empty positions list must have element_count=0");
        // Indices list ptr at byte 24.
        let idx_ptr = u64::from_le_bytes(bytes[24..32].try_into().expect("8 bytes"));
        let idx_count = ((idx_ptr >> 32) >> 3) & 0x1FFF_FFFF;
        assert_eq!(idx_count, 0, "empty indices list must have element_count=0");
    }

    /// Positions element_count must equal positions.len() * 3.
    #[test]
    fn test_mesh_binary_positions_element_count() {
        let positions = [[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
        let indices: &[u32] = &[];
        let bytes = export_mesh_capnp_binary(&positions, indices);
        assert!(bytes.len() >= 24);
        let pos_ptr = u64::from_le_bytes(bytes[16..24].try_into().expect("8 bytes"));
        let pos_count = ((pos_ptr >> 32) >> 3) & 0x1FFF_FFFF;
        assert_eq!(
            pos_count as usize,
            positions.len() * 3,
            "element_count must be vertex_count * 3"
        );
    }

    /// Indices element_count must equal indices.len().
    #[test]
    fn test_mesh_binary_indices_element_count() {
        let positions: &[[f32; 3]] = &[];
        let indices = [10u32, 20, 30, 40, 50, 60];
        let bytes = export_mesh_capnp_binary(positions, &indices);
        assert!(bytes.len() >= 32);
        let idx_ptr = u64::from_le_bytes(bytes[24..32].try_into().expect("8 bytes"));
        let idx_count = ((idx_ptr >> 32) >> 3) & 0x1FFF_FFFF;
        assert_eq!(idx_count as usize, indices.len(), "element_count must be indices.len()");
    }

    /// Encoding is deterministic.
    #[test]
    fn test_mesh_binary_deterministic() {
        let positions = [[0.1f32, 0.2, 0.3], [1.0, 2.0, 3.0]];
        let indices = [0u32, 1, 2];
        let a = export_mesh_capnp_binary(&positions, &indices);
        let b = export_mesh_capnp_binary(&positions, &indices);
        assert_eq!(a, b, "encoding must be deterministic");
    }

    /// Declared segment word count in header must match the actual body length.
    #[test]
    fn test_mesh_binary_declared_word_count_matches_body() {
        let positions = [[0.0f32, 1.0, 2.0], [3.0, 4.0, 5.0], [6.0, 7.0, 8.0]];
        let indices = [0u32, 1, 2, 0, 2, 3];
        let bytes = export_mesh_capnp_binary(&positions, &indices);
        // 1-segment message: header = u32(0) + u32(word_count) = 8 bytes.
        assert!(bytes.len() >= 8);
        let declared_words =
            u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
        let body_bytes = bytes.len() - 8;
        assert_eq!(
            body_bytes % 8,
            0,
            "body must be word-aligned"
        );
        assert_eq!(
            declared_words,
            body_bytes / 8,
            "declared word count must match actual segment body"
        );
    }
}
