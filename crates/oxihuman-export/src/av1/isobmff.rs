// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ISOBMFF (ISO Base Media File Format) box construction helpers.
//!
//! Provides a minimal set of primitives needed to build an AVIF container:
//! box framing, FullBox headers, and assorted write helpers.  All sizes are
//! big-endian per the ISO 14496-12 specification.

// ─────────────────────────────────────────────────────────────────────────────
// Box framing
// ─────────────────────────────────────────────────────────────────────────────

/// Begin writing a box with the given 4-byte type code.
///
/// Reserves space for the 4-byte size field (initially zero) and writes the
/// type.  Returns the byte offset of the size placeholder so it can be
/// back-patched with [`end_box`] after the payload is written.
pub fn begin_box(out: &mut Vec<u8>, box_type: &[u8; 4]) -> usize {
    let start = out.len();
    out.extend_from_slice(&[0u8; 4]); // size placeholder
    out.extend_from_slice(box_type);
    start
}

/// Finalise a box by back-patching the 4-byte size field at `start`.
///
/// The recorded size includes the 4-byte size field and the 4-byte type field.
/// Panics in debug mode if `out.len() - start > u32::MAX`.
pub fn end_box(out: &mut [u8], start: usize) {
    let size = out.len() - start;
    debug_assert!(size <= u32::MAX as usize, "box size exceeds u32 range");
    let size_bytes = (size as u32).to_be_bytes();
    out[start..start + 4].copy_from_slice(&size_bytes);
}

// ─────────────────────────────────────────────────────────────────────────────
// FullBox helper
// ─────────────────────────────────────────────────────────────────────────────

/// Write a FullBox header: 1 byte version followed by 3 bytes of flags.
pub fn write_full_box_header(out: &mut Vec<u8>, version: u8, flags: u32) {
    out.push(version);
    out.push(((flags >> 16) & 0xFF) as u8);
    out.push(((flags >> 8) & 0xFF) as u8);
    out.push((flags & 0xFF) as u8);
}

// ─────────────────────────────────────────────────────────────────────────────
// Primitive write helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Write a 32-bit unsigned integer in big-endian byte order.
pub fn write_u32_be(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Write a 16-bit unsigned integer in big-endian byte order.
pub fn write_u16_be(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Write a 64-bit unsigned integer in big-endian byte order.
pub fn write_u64_be(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_be_bytes());
}

/// Write a null-terminated ASCII string (C-string).
pub fn write_cstring(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(s.as_bytes());
    out.push(0u8); // NUL terminator
}

/// Write a 4-byte ASCII fourcc code.
pub fn write_fourcc(out: &mut Vec<u8>, s: &[u8; 4]) {
    out.extend_from_slice(s);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_end_box_size() {
        let mut out = Vec::new();
        let start = begin_box(&mut out, b"test");
        out.extend_from_slice(b"payload_here");
        end_box(&mut out, start);

        // Total: 4 (size) + 4 (type) + 12 (payload) = 20 bytes.
        assert_eq!(out.len(), 20);
        let size = u32::from_be_bytes([out[0], out[1], out[2], out[3]]);
        assert_eq!(size, 20, "size field must equal total box length");
        assert_eq!(&out[4..8], b"test", "type field must match");
    }

    #[test]
    fn test_nested_boxes() {
        let mut out = Vec::new();
        let outer_start = begin_box(&mut out, b"moov");
        {
            let inner_start = begin_box(&mut out, b"trak");
            out.extend_from_slice(b"inner_data");
            end_box(&mut out, inner_start);
        }
        end_box(&mut out, outer_start);

        // Outer: 8 (header) + [8 (inner header) + 10 (inner data)] = 26 bytes.
        let outer_size = u32::from_be_bytes([out[0], out[1], out[2], out[3]]);
        assert_eq!(outer_size as usize, out.len());
    }

    #[test]
    fn test_full_box_header() {
        let mut out = Vec::new();
        write_full_box_header(&mut out, 1, 0x000003);
        assert_eq!(out, &[1, 0x00, 0x00, 0x03]);
    }

    #[test]
    fn test_write_u32_be() {
        let mut out = Vec::new();
        write_u32_be(&mut out, 0xDEADBEEF);
        assert_eq!(out, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_write_u16_be() {
        let mut out = Vec::new();
        write_u16_be(&mut out, 0x1234);
        assert_eq!(out, &[0x12, 0x34]);
    }

    #[test]
    fn test_write_cstring() {
        let mut out = Vec::new();
        write_cstring(&mut out, "hello");
        assert_eq!(out, b"hello\0");
    }

    #[test]
    fn test_write_fourcc() {
        let mut out = Vec::new();
        write_fourcc(&mut out, b"av01");
        assert_eq!(out, b"av01");
    }

    #[test]
    fn test_empty_box() {
        let mut out = Vec::new();
        let start = begin_box(&mut out, b"mdat");
        end_box(&mut out, start);
        // 4 (size) + 4 (type) = 8 bytes.
        assert_eq!(out.len(), 8);
        let size = u32::from_be_bytes([out[0], out[1], out[2], out[3]]);
        assert_eq!(size, 8);
    }
}
