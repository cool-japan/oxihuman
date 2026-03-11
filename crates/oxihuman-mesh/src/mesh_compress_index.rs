// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Index buffer compression: delta encoding and 16-bit packing.

/// Result of index compression.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CompressedIndices {
    /// Delta-encoded indices (each value = current - previous, or absolute for i==0).
    pub deltas: Vec<i32>,
    /// Original count.
    pub original_count: usize,
}

/// Encode a u32 index buffer as delta values.
#[allow(dead_code)]
pub fn encode_delta_indices(indices: &[u32]) -> CompressedIndices {
    let mut deltas = Vec::with_capacity(indices.len());
    let mut prev = 0u32;
    for &idx in indices {
        deltas.push(idx as i32 - prev as i32);
        prev = idx;
    }
    CompressedIndices {
        deltas,
        original_count: indices.len(),
    }
}

/// Decode delta-encoded indices back to u32.
#[allow(dead_code)]
pub fn decode_delta_indices(ci: &CompressedIndices) -> Vec<u32> {
    let mut out = Vec::with_capacity(ci.deltas.len());
    let mut running = 0i32;
    for &d in &ci.deltas {
        running += d;
        out.push(running.max(0) as u32);
    }
    out
}

/// Pack u32 indices as u16, clamping values exceeding 65535.
#[allow(dead_code)]
pub fn pack_u16(indices: &[u32]) -> Vec<u16> {
    indices.iter().map(|&i| i.min(65535) as u16).collect()
}

/// Unpack u16 indices to u32.
#[allow(dead_code)]
pub fn unpack_u16(indices: &[u16]) -> Vec<u32> {
    indices.iter().map(|&i| i as u32).collect()
}

/// Check if an index buffer fits in u16.
#[allow(dead_code)]
pub fn fits_u16(indices: &[u32]) -> bool {
    indices.iter().all(|&i| i <= 65535)
}

/// Compute the maximum index value.
#[allow(dead_code)]
pub fn max_index_ci(indices: &[u32]) -> u32 {
    indices.iter().copied().fold(0, u32::max)
}

/// Estimate compressed size in bytes for delta-encoded output (variable-length encoding stub).
#[allow(dead_code)]
pub fn estimate_delta_size_bytes(ci: &CompressedIndices) -> usize {
    // Each delta: 1 byte if fits in i8, 2 if i16, 4 otherwise
    ci.deltas
        .iter()
        .map(|&d| {
            if (-128..=127).contains(&d) {
                1
            } else if (-32768..=32767).contains(&d) {
                2
            } else {
                4
            }
        })
        .sum()
}

/// Serialize compression stats to JSON.
#[allow(dead_code)]
pub fn compress_index_to_json(ci: &CompressedIndices) -> String {
    let est = estimate_delta_size_bytes(ci);
    format!(
        "{{\"original_count\":{},\"estimated_bytes\":{}}}",
        ci.original_count, est
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_indices() -> Vec<u32> {
        vec![0, 1, 2, 0, 2, 3, 3, 4, 5]
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let idx = sample_indices();
        let ci = encode_delta_indices(&idx);
        let decoded = decode_delta_indices(&ci);
        assert_eq!(decoded, idx);
    }

    #[test]
    fn test_delta_count_matches() {
        let idx = sample_indices();
        let ci = encode_delta_indices(&idx);
        assert_eq!(ci.deltas.len(), idx.len());
    }

    #[test]
    fn test_pack_u16_basic() {
        let idx = vec![0u32, 100, 65535];
        let packed = pack_u16(&idx);
        assert_eq!(packed[2], 65535u16);
    }

    #[test]
    fn test_unpack_u16_roundtrip() {
        let idx = vec![0u16, 255, 1000];
        let unpacked = unpack_u16(&idx);
        assert_eq!(unpacked[2], 1000u32);
    }

    #[test]
    fn test_fits_u16_true() {
        let idx = vec![0u32, 100, 65535];
        assert!(fits_u16(&idx));
    }

    #[test]
    fn test_fits_u16_false() {
        let idx = vec![0u32, 65536];
        assert!(!fits_u16(&idx));
    }

    #[test]
    fn test_max_index_ci() {
        let idx = sample_indices();
        assert_eq!(max_index_ci(&idx), 5);
    }

    #[test]
    fn test_estimate_delta_size_bytes_positive() {
        let idx = sample_indices();
        let ci = encode_delta_indices(&idx);
        assert!(estimate_delta_size_bytes(&ci) > 0);
    }

    #[test]
    fn test_compress_index_to_json() {
        let idx = sample_indices();
        let ci = encode_delta_indices(&idx);
        let j = compress_index_to_json(&ci);
        assert!(j.contains("original_count"));
    }

    #[test]
    fn test_empty_indices() {
        let ci = encode_delta_indices(&[]);
        let decoded = decode_delta_indices(&ci);
        assert!(decoded.is_empty());
    }
}
