// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Buffer serialization and parsing utilities for mesh data transfer.

use oxihuman_export::mesh_quantize::QuantizedMesh;

/// Serialise a [`QuantizedMesh`] to bytes using the same layout as
/// `write_quantized_bin`, but entirely in memory.
pub fn serialize_quantized_to_bytes(q: &QuantizedMesh) -> Vec<u8> {
    let vc = q.positions.len() as u32;
    let ic = q.indices.len() as u32;

    let mut buf: Vec<u8> = Vec::new();

    // Header: magic + version + vertex_count + index_count = 16 bytes.
    buf.extend_from_slice(b"QMSH");
    buf.extend_from_slice(&1u32.to_le_bytes()); // version = 1
    buf.extend_from_slice(&vc.to_le_bytes());
    buf.extend_from_slice(&ic.to_le_bytes());

    // Ranges: 3 x (min f32, max f32) = 24 bytes.
    for r in &q.pos_range {
        buf.extend_from_slice(&r.min.to_le_bytes());
        buf.extend_from_slice(&r.max.to_le_bytes());
    }

    // Positions: vertex_count x u16 x 3.
    for p in &q.positions {
        for &v in p {
            buf.extend_from_slice(&v.to_le_bytes());
        }
    }

    // Normals: vertex_count x i8 x 3 (stored as raw bytes).
    for n in &q.normals {
        for &b in n {
            buf.push(b as u8);
        }
    }

    // UVs: vertex_count x u16 x 2.
    for uv in &q.uvs {
        for &v in uv {
            buf.extend_from_slice(&v.to_le_bytes());
        }
    }

    // Indices: index_count x u32.
    for &idx in &q.indices {
        buf.extend_from_slice(&idx.to_le_bytes());
    }

    // has_suit flag.
    buf.push(q.has_suit as u8);

    buf
}

/// Parse the header from bytes returned by `build_mesh_bytes()`.
pub fn parse_mesh_bytes_header(bytes: &[u8]) -> Option<(u32, u32, u32)> {
    if bytes.len() < 12 {
        return None;
    }
    let version = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
    let n_verts = u32::from_le_bytes(bytes[4..8].try_into().ok()?);
    let n_idx = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    Some((version, n_verts, n_idx))
}
