// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Sequence header OBU for the simplified self-consistent AV1-like codec.
//!
//! The payload layout is intentionally simple (not bitstream-compliant with the
//! full AV1 spec) because our in-tree decoder must agree with our encoder on
//! byte layout:
//!
//! ```text
//! Byte 0:   seq_profile  (u8, value = 1 for 4:4:4)
//! Byte 1:   still_picture flag (u8, value = 1)
//! Bytes 2–5: width  (u32 little-endian)
//! Bytes 6–9: height (u32 little-endian)
//! ```

use super::obu::{wrap_obu, ObuType};

/// Sequence header data for encoding/decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqHeader {
    pub width: u32,
    pub height: u32,
}

/// Encode the sequence-header OBU payload bytes.
///
/// Layout: `[profile=1][still=1][width LE u32][height LE u32]`
pub fn encode_sequence_header(hdr: &SeqHeader) -> Vec<u8> {
    let mut v = Vec::with_capacity(10);
    v.push(1u8); // seq_profile = 1 (4:4:4)
    v.push(1u8); // still_picture = 1
    v.extend_from_slice(&hdr.width.to_le_bytes());
    v.extend_from_slice(&hdr.height.to_le_bytes());
    v
}

/// Build the complete sequence-header OBU (OBU header + LEB128 size + payload).
pub fn sequence_header_obu(hdr: &SeqHeader) -> Vec<u8> {
    wrap_obu(ObuType::SequenceHeader, &encode_sequence_header(hdr))
}

/// Decode a sequence-header OBU payload.
///
/// Returns `None` if the payload is too short or malformed.
pub fn decode_sequence_header(data: &[u8]) -> Option<SeqHeader> {
    if data.len() < 10 {
        return None;
    }
    let _profile = data[0];
    let _still = data[1];
    let width = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
    let height = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
    Some(SeqHeader { width, height })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av1::obu::parse_obu_stream;

    #[test]
    fn test_encode_decode_round_trip_small() {
        let hdr = SeqHeader {
            width: 4,
            height: 4,
        };
        let payload = encode_sequence_header(&hdr);
        let decoded = decode_sequence_header(&payload).expect("decode should succeed");
        assert_eq!(decoded.width, hdr.width);
        assert_eq!(decoded.height, hdr.height);
    }

    #[test]
    fn test_encode_decode_round_trip_large() {
        let hdr = SeqHeader {
            width: 3840,
            height: 2160,
        };
        let payload = encode_sequence_header(&hdr);
        let decoded = decode_sequence_header(&payload).expect("decode should succeed");
        assert_eq!(decoded.width, hdr.width);
        assert_eq!(decoded.height, hdr.height);
    }

    #[test]
    fn test_obu_wrapping_produces_correct_type() {
        let hdr = SeqHeader {
            width: 8,
            height: 8,
        };
        let obu = sequence_header_obu(&hdr);
        assert!(!obu.is_empty(), "OBU bytes must not be empty");

        let obus = parse_obu_stream(&obu);
        assert_eq!(obus.len(), 1, "should parse exactly one OBU");
        assert_eq!(
            obus[0].0,
            ObuType::SequenceHeader as u8,
            "OBU type must be SequenceHeader (1)"
        );
    }

    #[test]
    fn test_payload_length_is_ten() {
        let hdr = SeqHeader {
            width: 1920,
            height: 1080,
        };
        let payload = encode_sequence_header(&hdr);
        assert_eq!(payload.len(), 10, "payload must be exactly 10 bytes");
    }

    #[test]
    fn test_decode_short_payload_returns_none() {
        // A payload shorter than 10 bytes must return None gracefully.
        let short = [1u8, 1, 0, 0];
        assert!(decode_sequence_header(&short).is_none());
    }

    #[test]
    fn test_profile_and_still_bytes() {
        let hdr = SeqHeader {
            width: 256,
            height: 128,
        };
        let payload = encode_sequence_header(&hdr);
        assert_eq!(payload[0], 1, "seq_profile must be 1");
        assert_eq!(payload[1], 1, "still_picture must be 1");
    }
}
