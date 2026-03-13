// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! GLB binary container stub for glTF 2.0 export.
//!
//! Provides a pure-Rust implementation of the GLB container format
//! (magic header, JSON chunk, BIN chunk) without external dependencies.

// ── Constants ─────────────────────────────────────────────────────────────────

/// GLB magic bytes: ASCII "glTF" as little-endian u32 (0x46546C67).
pub const GLB_MAGIC: u32 = 0x46546C67;
/// GLB version supported by this module.
pub const GLB_VERSION: u32 = 2;
/// Chunk type identifier for JSON chunks.
pub const CHUNK_TYPE_JSON: u32 = 0x4E4F534A;
/// Chunk type identifier for binary (BIN\0) chunks.
pub const CHUNK_TYPE_BIN: u32 = 0x004E4942;
/// Minimum GLB header size in bytes (magic + version + total length).
pub const GLB_HEADER_BYTES: usize = 12;

// ── Types ─────────────────────────────────────────────────────────────────────

/// Identifies the type of a GLB chunk.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlbChunkType {
    /// UTF-8 JSON payload (chunk type 0x4E4F534A).
    Json,
    /// Raw binary buffer (chunk type 0x004E4942).
    Binary,
}

/// A single chunk within a GLB container.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlbChunk {
    /// Chunk type tag.
    pub chunk_type: GlbChunkType,
    /// Raw payload bytes (padded to 4-byte alignment when serialised).
    pub data: Vec<u8>,
}

/// In-memory GLB container holding an ordered list of chunks.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GlbContainer {
    /// Ordered chunks; first must be JSON, second (optional) BIN.
    pub chunks: Vec<GlbChunk>,
}

/// Configuration for GLB export behaviour.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GlbExportConfig {
    /// Embed binary buffer in the GLB (true) or write a separate .bin (false).
    pub embed_binary: bool,
    /// Align chunk payloads to 4 bytes (required by spec).
    pub pad_chunks: bool,
    /// Include a GLB generator string in the JSON asset metadata.
    pub include_generator: bool,
    /// Maximum binary chunk size in bytes (0 = unlimited).
    pub max_bin_bytes: usize,
}

// ── Constructor helpers ────────────────────────────────────────────────────────

/// Returns a default [`GlbExportConfig`] suitable for most use-cases.
#[allow(dead_code)]
pub fn default_glb_config() -> GlbExportConfig {
    GlbExportConfig {
        embed_binary: true,
        pad_chunks: true,
        include_generator: true,
        max_bin_bytes: 0,
    }
}

/// Creates an empty [`GlbContainer`].
#[allow(dead_code)]
pub fn new_glb_container() -> GlbContainer {
    GlbContainer::default()
}

// ── Chunk manipulation ─────────────────────────────────────────────────────────

/// Appends a JSON chunk with the given UTF-8 payload.
///
/// Replaces the existing JSON chunk if one already exists.
#[allow(dead_code)]
pub fn add_json_chunk(container: &mut GlbContainer, json_bytes: Vec<u8>) {
    let chunk = GlbChunk {
        chunk_type: GlbChunkType::Json,
        data: json_bytes,
    };
    if let Some(pos) = container
        .chunks
        .iter()
        .position(|c| c.chunk_type == GlbChunkType::Json)
    {
        container.chunks[pos] = chunk;
    } else {
        container.chunks.insert(0, chunk);
    }
}

/// Appends a BIN chunk with the given raw bytes.
///
/// Replaces the existing BIN chunk if one already exists.
#[allow(dead_code)]
pub fn add_binary_chunk(container: &mut GlbContainer, bin_bytes: Vec<u8>) {
    let chunk = GlbChunk {
        chunk_type: GlbChunkType::Binary,
        data: bin_bytes,
    };
    if let Some(pos) = container
        .chunks
        .iter()
        .position(|c| c.chunk_type == GlbChunkType::Binary)
    {
        container.chunks[pos] = chunk;
    } else {
        container.chunks.push(chunk);
    }
}

/// Returns the total number of chunks in the container.
#[allow(dead_code)]
pub fn chunk_count(container: &GlbContainer) -> usize {
    container.chunks.len()
}

// ── Size computation ───────────────────────────────────────────────────────────

/// Computes the total serialised byte size of the GLB file.
///
/// Each chunk is padded to a 4-byte boundary (8 bytes header + padded payload).
#[allow(dead_code)]
pub fn glb_total_size(container: &GlbContainer) -> usize {
    let chunks_size: usize = container
        .chunks
        .iter()
        .map(|c| 8 + pad4(c.data.len()))
        .sum();
    GLB_HEADER_BYTES + chunks_size
}

// ── Serialisation ─────────────────────────────────────────────────────────────

/// Serialises the container to a complete GLB byte vector.
#[allow(dead_code)]
pub fn encode_glb(container: &GlbContainer) -> Vec<u8> {
    let total = glb_total_size(container);
    let mut out = Vec::with_capacity(total);

    // 12-byte GLB header
    out.extend_from_slice(&GLB_MAGIC.to_le_bytes());
    out.extend_from_slice(&GLB_VERSION.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());

    for chunk in &container.chunks {
        let padded_len = pad4(chunk.data.len());
        out.extend_from_slice(&(padded_len as u32).to_le_bytes());
        let type_id: u32 = match chunk.chunk_type {
            GlbChunkType::Json => CHUNK_TYPE_JSON,
            GlbChunkType::Binary => CHUNK_TYPE_BIN,
        };
        out.extend_from_slice(&type_id.to_le_bytes());
        out.extend_from_slice(&chunk.data);
        // Padding bytes: JSON pads with 0x20 (space), BIN pads with 0x00
        let pad_byte = match chunk.chunk_type {
            GlbChunkType::Json => 0x20u8,
            GlbChunkType::Binary => 0x00u8,
        };
        let pad_count = padded_len - chunk.data.len();
        out.extend(std::iter::repeat_n(pad_byte, pad_count));
    }
    out
}

// ── Deserialisation ────────────────────────────────────────────────────────────

/// Parses the 12-byte GLB header and returns `(magic, version, total_length)`.
///
/// Returns `None` if the buffer is shorter than 12 bytes.
#[allow(dead_code)]
pub fn decode_glb_header(bytes: &[u8]) -> Option<(u32, u32, u32)> {
    if bytes.len() < GLB_HEADER_BYTES {
        return None;
    }
    let magic = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
    let version = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
    let length = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    Some((magic, version, length))
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Returns `true` if the container is a well-formed GLB (has a JSON chunk first).
#[allow(dead_code)]
pub fn validate_glb(container: &GlbContainer) -> bool {
    if container.chunks.is_empty() {
        return false;
    }
    container.chunks[0].chunk_type == GlbChunkType::Json
}

// ── Accessors ─────────────────────────────────────────────────────────────────

/// Returns a reference to the first JSON chunk payload, if any.
#[allow(dead_code)]
pub fn glb_json_chunk(container: &GlbContainer) -> Option<&[u8]> {
    container
        .chunks
        .iter()
        .find(|c| c.chunk_type == GlbChunkType::Json)
        .map(|c| c.data.as_slice())
}

/// Returns a reference to the first binary chunk payload, if any.
#[allow(dead_code)]
pub fn glb_binary_chunk(container: &GlbContainer) -> Option<&[u8]> {
    container
        .chunks
        .iter()
        .find(|c| c.chunk_type == GlbChunkType::Binary)
        .map(|c| c.data.as_slice())
}

/// Returns a hex preview of the first `n` bytes of the encoded GLB.
#[allow(dead_code)]
pub fn glb_to_hex_preview(container: &GlbContainer, n: usize) -> String {
    let bytes = encode_glb(container);
    bytes
        .iter()
        .take(n)
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Returns the 4-byte GLB magic as a byte array (`b"glTF"`).
#[allow(dead_code)]
pub fn glb_magic_bytes() -> [u8; 4] {
    GLB_MAGIC.to_le_bytes()
}

// ── Private helpers ────────────────────────────────────────────────────────────

/// Rounds `n` up to the nearest multiple of 4.
#[inline]
fn pad4(n: usize) -> usize {
    (n + 3) & !3
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_embed_binary() {
        let cfg = default_glb_config();
        assert!(cfg.embed_binary);
    }

    #[test]
    fn test_default_config_pad_chunks() {
        let cfg = default_glb_config();
        assert!(cfg.pad_chunks);
    }

    #[test]
    fn test_new_container_empty() {
        let c = new_glb_container();
        assert_eq!(chunk_count(&c), 0);
    }

    #[test]
    fn test_add_json_chunk_inserts_at_front() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        assert_eq!(chunk_count(&c), 1);
        assert_eq!(c.chunks[0].chunk_type, GlbChunkType::Json);
    }

    #[test]
    fn test_add_binary_chunk() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        add_binary_chunk(&mut c, vec![0u8; 16]);
        assert_eq!(chunk_count(&c), 2);
        assert_eq!(c.chunks[1].chunk_type, GlbChunkType::Binary);
    }

    #[test]
    fn test_replace_json_chunk() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        add_json_chunk(&mut c, b"{\"x\":1}".to_vec());
        assert_eq!(chunk_count(&c), 1);
        assert_eq!(c.chunks[0].data, b"{\"x\":1}");
    }

    #[test]
    fn test_replace_binary_chunk() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        add_binary_chunk(&mut c, vec![1u8; 4]);
        add_binary_chunk(&mut c, vec![2u8; 8]);
        assert_eq!(chunk_count(&c), 2);
        assert_eq!(c.chunks[1].data, vec![2u8; 8]);
    }

    #[test]
    fn test_glb_total_size_header_only_empty() {
        let c = new_glb_container();
        assert_eq!(glb_total_size(&c), GLB_HEADER_BYTES);
    }

    #[test]
    fn test_glb_total_size_with_chunks() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec()); // 2 bytes → padded to 4
        let expected = GLB_HEADER_BYTES + 8 + 4;
        assert_eq!(glb_total_size(&c), expected);
    }

    #[test]
    fn test_encode_glb_magic() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        let bytes = encode_glb(&c);
        let magic = u32::from_le_bytes(bytes[0..4].try_into().expect("should succeed"));
        assert_eq!(magic, GLB_MAGIC);
    }

    #[test]
    fn test_encode_glb_version() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        let bytes = encode_glb(&c);
        let version = u32::from_le_bytes(bytes[4..8].try_into().expect("should succeed"));
        assert_eq!(version, GLB_VERSION);
    }

    #[test]
    fn test_encode_glb_length_matches_total_size() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{\"a\":1}".to_vec());
        add_binary_chunk(&mut c, vec![0u8; 12]);
        let bytes = encode_glb(&c);
        let length = u32::from_le_bytes(bytes[8..12].try_into().expect("should succeed")) as usize;
        assert_eq!(length, bytes.len());
    }

    #[test]
    fn test_decode_glb_header_valid() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        let bytes = encode_glb(&c);
        let (magic, version, _len) = decode_glb_header(&bytes).expect("should succeed");
        assert_eq!(magic, GLB_MAGIC);
        assert_eq!(version, GLB_VERSION);
    }

    #[test]
    fn test_decode_glb_header_too_short() {
        assert!(decode_glb_header(&[0u8; 4]).is_none());
    }

    #[test]
    fn test_validate_glb_valid() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        assert!(validate_glb(&c));
    }

    #[test]
    fn test_validate_glb_empty_fails() {
        let c = new_glb_container();
        assert!(!validate_glb(&c));
    }

    #[test]
    fn test_glb_json_chunk_accessor() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{\"v\":42}".to_vec());
        let data = glb_json_chunk(&c).expect("should succeed");
        assert_eq!(data, b"{\"v\":42}");
    }

    #[test]
    fn test_glb_binary_chunk_none_when_absent() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        assert!(glb_binary_chunk(&c).is_none());
    }

    #[test]
    fn test_glb_binary_chunk_present() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        add_binary_chunk(&mut c, vec![0xAB, 0xCD]);
        let bin = glb_binary_chunk(&c).expect("should succeed");
        assert_eq!(bin, &[0xAB, 0xCD]);
    }

    #[test]
    fn test_glb_to_hex_preview_length() {
        let mut c = new_glb_container();
        add_json_chunk(&mut c, b"{}".to_vec());
        let hex = glb_to_hex_preview(&c, 4);
        // 4 bytes → 4 two-char groups + 3 spaces = 11 chars
        assert_eq!(hex.len(), 11);
    }

    #[test]
    fn test_glb_magic_bytes() {
        let mb = glb_magic_bytes();
        // "glTF" little-endian
        assert_eq!(&mb, b"glTF");
    }

    #[test]
    fn test_pad4_alignment() {
        assert_eq!(pad4(0), 0);
        assert_eq!(pad4(1), 4);
        assert_eq!(pad4(4), 4);
        assert_eq!(pad4(5), 8);
    }
}
