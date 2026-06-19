// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Frame header OBU for the simplified self-consistent AV1-like codec.
//!
//! Payload layout:
//! ```text
//! Byte 0:  base_q_idx (u8; 0 = lossless)
//! Byte 1:  lossless_flag (u8; 1 when base_q_idx == 0, else 0)
//! ```

use super::obu::{wrap_obu, ObuType};

/// Frame header encoding parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    /// Quantizer index (0 = lossless).
    pub base_q_idx: u8,
}

/// Encode the frame-header OBU payload.
pub fn encode_frame_header(hdr: &FrameHeader) -> Vec<u8> {
    let lossless_flag: u8 = if hdr.base_q_idx == 0 { 1 } else { 0 };
    vec![hdr.base_q_idx, lossless_flag]
}

/// Build the complete frame-header OBU.
pub fn frame_header_obu(hdr: &FrameHeader) -> Vec<u8> {
    wrap_obu(ObuType::FrameHeader, &encode_frame_header(hdr))
}

/// Decode a frame-header OBU payload.
///
/// Returns `None` if the payload is too short.
pub fn decode_frame_header(data: &[u8]) -> Option<FrameHeader> {
    if data.len() < 2 {
        return None;
    }
    Some(FrameHeader {
        base_q_idx: data[0],
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av1::obu::parse_obu_stream;

    #[test]
    fn test_lossless_round_trip() {
        let hdr = FrameHeader { base_q_idx: 0 };
        let payload = encode_frame_header(&hdr);
        assert_eq!(payload[0], 0, "base_q_idx must be 0");
        assert_eq!(payload[1], 1, "lossless flag must be 1 when q=0");
        let decoded = decode_frame_header(&payload).expect("decode should succeed");
        assert_eq!(decoded.base_q_idx, 0);
    }

    #[test]
    fn test_lossy_round_trip() {
        let hdr = FrameHeader { base_q_idx: 24 };
        let payload = encode_frame_header(&hdr);
        assert_eq!(payload[0], 24);
        assert_eq!(payload[1], 0, "lossless flag must be 0 when q>0");
        let decoded = decode_frame_header(&payload).expect("decode should succeed");
        assert_eq!(decoded.base_q_idx, 24);
    }

    #[test]
    fn test_obu_type_is_frame_header() {
        let hdr = FrameHeader { base_q_idx: 0 };
        let obu = frame_header_obu(&hdr);
        let obus = parse_obu_stream(&obu);
        assert_eq!(obus.len(), 1);
        assert_eq!(obus[0].0, ObuType::FrameHeader as u8);
    }

    #[test]
    fn test_decode_short_payload_returns_none() {
        assert!(decode_frame_header(&[]).is_none());
        assert!(decode_frame_header(&[0]).is_none());
    }

    #[test]
    fn test_payload_length_is_two() {
        let hdr = FrameHeader { base_q_idx: 10 };
        let payload = encode_frame_header(&hdr);
        assert_eq!(payload.len(), 2, "frame header payload must be 2 bytes");
    }
}
