// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! OBU (Open Bitstream Unit) header framing for AV1 streams.
//!
//! Each OBU consists of:
//!   - 1-byte header (extension_flag=0, has_size_field=1 for AVIF)
//!   - LEB128 obu_size field (payload length in bytes)
//!   - Payload bytes

use super::leb128::{read_leb128, write_leb128};

/// AV1 OBU type identifiers (Section 6.2.2 of the AV1 specification).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObuType {
    SequenceHeader = 1,
    TemporalDelimiter = 2,
    FrameHeader = 3,
    TileGroup = 4,
    Metadata = 5,
    Frame = 6,
    RedundantFrameHeader = 7,
    TileList = 8,
    Padding = 15,
}

impl ObuType {
    /// Construct from raw u8 nibble.  Returns `None` for reserved/unknown values.
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(ObuType::SequenceHeader),
            2 => Some(ObuType::TemporalDelimiter),
            3 => Some(ObuType::FrameHeader),
            4 => Some(ObuType::TileGroup),
            5 => Some(ObuType::Metadata),
            6 => Some(ObuType::Frame),
            7 => Some(ObuType::RedundantFrameHeader),
            8 => Some(ObuType::TileList),
            15 => Some(ObuType::Padding),
            _ => None,
        }
    }
}

/// OBU header byte layout (no extension):
///
/// ```text
/// [7:4] obu_type      (4 bits)
/// [3]   extension_flag = 0
/// [2]   has_size_field = 1 (required for AVIF)
/// [1]   obu_reserved_1bit = 0
/// [0]   (implicit trailing 0)
/// ```
///
/// Build a complete OBU unit: header byte + LEB128(payload_len) + payload.
pub fn wrap_obu(obu_type: ObuType, payload: &[u8]) -> Vec<u8> {
    // Header byte: obu_type in bits [7:4], extension_flag=0 (bit 3), has_size_field=1 (bit 2).
    // Bits [1:0] are reserved zeros.
    let type_nibble = (obu_type as u8) & 0x0F;
    let header_byte: u8 = (type_nibble << 4) | 0x04; // bit[2] = has_size_field

    let mut out = Vec::with_capacity(1 + 4 + payload.len());
    out.push(header_byte);
    write_leb128(&mut out, payload.len() as u64);
    out.extend_from_slice(payload);
    out
}

/// Parse an OBU header from `data[*pos..]`.
///
/// Returns `(obu_type_raw, payload_start, payload_len)` on success.
/// `payload_start` is an absolute index into `data`.
/// Advances `*pos` past the header and size field, but NOT past the payload.
/// Returns `None` on malformed input or truncated data.
pub fn parse_obu_header(data: &[u8], pos: &mut usize) -> Option<(u8, usize, usize)> {
    if *pos >= data.len() {
        return None;
    }
    let header_byte = data[*pos];
    *pos += 1;

    let obu_type_raw = (header_byte >> 4) & 0x0F;
    let extension_flag = (header_byte >> 3) & 1;
    let has_size_field = (header_byte >> 2) & 1;

    // Skip extension byte if present.
    if extension_flag != 0 {
        if *pos >= data.len() {
            return None;
        }
        *pos += 1;
    }

    let payload_len: usize = if has_size_field != 0 {
        read_leb128(data, pos)? as usize
    } else {
        // Without a size field the payload extends to the end of the temporal unit.
        data.len().saturating_sub(*pos)
    };

    let payload_start = *pos;
    // Verify the payload is present in the slice.
    if payload_start + payload_len > data.len() {
        return None;
    }

    Some((obu_type_raw, payload_start, payload_len))
}

/// Parse a series of OBUs from `data`, returning a list of `(obu_type_raw, payload_bytes)`.
///
/// Unknown / reserved OBU types are included verbatim (type as u8).
/// Stops when the data is exhausted or a parse error occurs.
pub fn parse_obu_stream(data: &[u8]) -> Vec<(u8, Vec<u8>)> {
    let mut result = Vec::new();
    let mut pos = 0usize;
    while pos < data.len() {
        let pos_before = pos;
        match parse_obu_header(data, &mut pos) {
            Some((obu_type, payload_start, payload_len)) => {
                let payload = data[payload_start..payload_start + payload_len].to_vec();
                result.push((obu_type, payload));
                // Advance past payload.
                pos = payload_start + payload_len;
            }
            None => {
                // Malformed stream — stop parsing.
                let _ = pos_before;
                break;
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_parse_sequence_header_round_trip() {
        let payload = b"fake_seq_header_data";
        let obu = wrap_obu(ObuType::SequenceHeader, payload);

        // Verify header byte.
        let expected_header = ((ObuType::SequenceHeader as u8) << 4) | 0x04;
        assert_eq!(obu[0], expected_header);

        let mut pos = 0usize;
        let (typ, pstart, plen) = parse_obu_header(&obu, &mut pos)
            .expect("parse should succeed");
        assert_eq!(typ, ObuType::SequenceHeader as u8);
        assert_eq!(plen, payload.len());
        assert_eq!(&obu[pstart..pstart + plen], payload);
    }

    #[test]
    fn test_wrap_parse_tile_group_round_trip() {
        let payload: Vec<u8> = (0u8..=127).collect();
        let obu = wrap_obu(ObuType::TileGroup, &payload);

        let mut pos = 0usize;
        let (typ, pstart, plen) = parse_obu_header(&obu, &mut pos).unwrap();
        assert_eq!(typ, ObuType::TileGroup as u8);
        assert_eq!(plen, payload.len());
        assert_eq!(&obu[pstart..pstart + plen], payload.as_slice());
    }

    #[test]
    fn test_parse_obu_stream_multiple() {
        let payloads: &[(&[u8], ObuType)] = &[
            (b"seq", ObuType::SequenceHeader),
            (b"frame_hdr", ObuType::FrameHeader),
            (b"tile_data_here", ObuType::TileGroup),
        ];

        let mut stream = Vec::new();
        for (payload, obu_type) in payloads {
            stream.extend_from_slice(&wrap_obu(*obu_type, payload));
        }

        let parsed = parse_obu_stream(&stream);
        assert_eq!(parsed.len(), payloads.len());
        for (i, (payload, obu_type)) in payloads.iter().enumerate() {
            assert_eq!(parsed[i].0, *obu_type as u8, "type mismatch at index {i}");
            assert_eq!(parsed[i].1.as_slice(), *payload, "payload mismatch at index {i}");
        }
    }

    #[test]
    fn test_empty_payload() {
        let obu = wrap_obu(ObuType::TemporalDelimiter, &[]);
        let mut pos = 0usize;
        let (typ, _pstart, plen) = parse_obu_header(&obu, &mut pos).unwrap();
        assert_eq!(typ, ObuType::TemporalDelimiter as u8);
        assert_eq!(plen, 0);
    }

    #[test]
    fn test_large_payload_leb128_encoding() {
        // Payload larger than 127 bytes to exercise multi-byte LEB128.
        let payload: Vec<u8> = (0u8..200).collect();
        let obu = wrap_obu(ObuType::Frame, &payload);
        let mut pos = 0usize;
        let (typ, pstart, plen) = parse_obu_header(&obu, &mut pos).unwrap();
        assert_eq!(typ, ObuType::Frame as u8);
        assert_eq!(plen, 200);
        assert_eq!(&obu[pstart..pstart + plen], payload.as_slice());
    }

    #[test]
    fn test_parse_truncated_returns_none() {
        let mut pos = 0usize;
        // Only header byte, no size field data.
        let data = [((ObuType::SequenceHeader as u8) << 4) | 0x04];
        let result = parse_obu_header(&data, &mut pos);
        // LEB128 decode of 0 bytes returns None.
        assert!(result.is_none());
    }

    #[test]
    fn test_obu_type_from_u8() {
        assert_eq!(ObuType::from_u8(1), Some(ObuType::SequenceHeader));
        assert_eq!(ObuType::from_u8(4), Some(ObuType::TileGroup));
        assert_eq!(ObuType::from_u8(15), Some(ObuType::Padding));
        assert_eq!(ObuType::from_u8(0), None);
        assert_eq!(ObuType::from_u8(9), None);
    }
}
