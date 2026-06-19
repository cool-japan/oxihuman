// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Tile group OBU for the simplified self-consistent AV1-like codec.
//!
//! The tile group carries the entropy-coded coefficient stream for the entire
//! image (single tile).  Payload layout:
//!
//! ```text
//! Bytes 0–3: data length as u32 little-endian
//! Bytes 4…:  raw arithmetic-coder bytes
//! ```

use super::obu::{wrap_obu, ObuType};

/// Wrap entropy-coded coefficient bytes in a tile-group OBU.
///
/// The payload is: `[4 bytes LE length][ec_bytes...]`
pub fn tile_group_obu(ec_bytes: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(4 + ec_bytes.len());
    let len = ec_bytes.len() as u32;
    payload.extend_from_slice(&len.to_le_bytes());
    payload.extend_from_slice(ec_bytes);
    wrap_obu(ObuType::TileGroup, &payload)
}

/// Extract the entropy-coder bytes from a tile-group OBU payload.
///
/// Returns `None` if the payload is too short or the embedded length field
/// is inconsistent with the slice length.
pub fn parse_tile_group(payload: &[u8]) -> Option<&[u8]> {
    if payload.len() < 4 {
        return None;
    }
    let len = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
    if payload.len() < 4 + len {
        return None;
    }
    Some(&payload[4..4 + len])
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::av1::obu::parse_obu_stream;

    #[test]
    fn test_tile_group_round_trip_empty() {
        let obu = tile_group_obu(&[]);
        let obus = parse_obu_stream(&obu);
        assert_eq!(obus.len(), 1);
        assert_eq!(obus[0].0, ObuType::TileGroup as u8);
        let ec = parse_tile_group(&obus[0].1).expect("parse must succeed");
        assert!(ec.is_empty());
    }

    #[test]
    fn test_tile_group_round_trip_data() {
        let data: Vec<u8> = (0u8..200).collect();
        let obu = tile_group_obu(&data);
        let obus = parse_obu_stream(&obu);
        assert_eq!(obus.len(), 1);
        let ec = parse_tile_group(&obus[0].1).expect("parse must succeed");
        assert_eq!(ec, data.as_slice());
    }

    #[test]
    fn test_parse_short_payload_returns_none() {
        assert!(parse_tile_group(&[]).is_none());
        assert!(parse_tile_group(&[0, 0, 0]).is_none());
    }

    #[test]
    fn test_parse_inconsistent_length_returns_none() {
        // Length field claims 100 bytes but only 4 + 5 = 9 bytes total.
        let mut payload = vec![100u8, 0, 0, 0]; // len = 100
        payload.extend_from_slice(&[0u8; 5]);
        assert!(parse_tile_group(&payload).is_none());
    }

    #[test]
    fn test_obu_type_is_tile_group() {
        let obu = tile_group_obu(&[1, 2, 3]);
        let obus = parse_obu_stream(&obu);
        assert_eq!(obus[0].0, ObuType::TileGroup as u8);
    }
}
