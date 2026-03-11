// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Run-length encoding/decoding for byte sequences.

#![allow(dead_code)]

/// A single run: (value, count).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Run {
    pub value: u8,
    pub count: u32,
}

/// Run-length encode a byte slice.
#[allow(dead_code)]
pub fn rle_encode(data: &[u8]) -> Vec<Run> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut current = data[0];
    let mut count = 1u32;
    for &b in &data[1..] {
        if b == current {
            count += 1;
        } else {
            out.push(Run {
                value: current,
                count,
            });
            current = b;
            count = 1;
        }
    }
    out.push(Run {
        value: current,
        count,
    });
    out
}

/// Run-length decode back to bytes.
#[allow(dead_code)]
pub fn rle_decode(runs: &[Run]) -> Vec<u8> {
    let total: u32 = runs.iter().map(|r| r.count).sum();
    let mut out = Vec::with_capacity(total as usize);
    for r in runs {
        for _ in 0..r.count {
            out.push(r.value);
        }
    }
    out
}

/// Return the number of runs.
#[allow(dead_code)]
pub fn rle_run_count(runs: &[Run]) -> usize {
    runs.len()
}

/// Return the total number of decoded bytes.
#[allow(dead_code)]
pub fn rle_decoded_len(runs: &[Run]) -> usize {
    runs.iter().map(|r| r.count as usize).sum()
}

/// Compression ratio: decoded_len / encoded_len. Returns 1.0 for empty.
#[allow(dead_code)]
pub fn rle_compression_ratio_v2(runs: &[Run]) -> f32 {
    if runs.is_empty() {
        return 1.0;
    }
    rle_decoded_len(runs) as f32 / runs.len() as f32
}

/// Find the most frequent byte value in the original sequence.
#[allow(dead_code)]
pub fn rle_most_frequent(runs: &[Run]) -> Option<u8> {
    if runs.is_empty() {
        return None;
    }
    runs.iter().max_by_key(|r| r.count).map(|r| r.value)
}

/// Check if the encoded sequence is uniform (single run).
#[allow(dead_code)]
pub fn rle_is_uniform(runs: &[Run]) -> bool {
    runs.len() == 1
}

/// Merge adjacent runs with the same value (e.g., after concatenation).
#[allow(dead_code)]
pub fn rle_merge(runs: &[Run]) -> Vec<Run> {
    if runs.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut cur = runs[0];
    for &r in &runs[1..] {
        if r.value == cur.value {
            cur.count += r.count;
        } else {
            out.push(cur);
            cur = r;
        }
    }
    out.push(cur);
    out
}

/// Encode then decode to verify roundtrip (returns true if data matches).
#[allow(dead_code)]
pub fn rle_verify_roundtrip(data: &[u8]) -> bool {
    let runs = rle_encode(data);
    let decoded = rle_decode(&runs);
    decoded == data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_empty() {
        assert!(rle_encode(&[]).is_empty());
    }

    #[test]
    fn decode_empty() {
        assert!(rle_decode(&[]).is_empty());
    }

    #[test]
    fn encode_uniform() {
        let runs = rle_encode(&[5u8; 10]);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].value, 5);
        assert_eq!(runs[0].count, 10);
    }

    #[test]
    fn roundtrip_basic() {
        let data = vec![1u8, 1, 2, 3, 3, 3, 4];
        assert!(rle_verify_roundtrip(&data));
    }

    #[test]
    fn roundtrip_alternating() {
        let data: Vec<u8> = (0..10).map(|i| i % 2).collect();
        assert!(rle_verify_roundtrip(&data));
    }

    #[test]
    fn decoded_len_correct() {
        let data = vec![0u8, 0, 0, 1, 1];
        let runs = rle_encode(&data);
        assert_eq!(rle_decoded_len(&runs), 5);
    }

    #[test]
    fn most_frequent() {
        let data = vec![1u8, 2, 2, 2, 3];
        let runs = rle_encode(&data);
        assert_eq!(rle_most_frequent(&runs), Some(2));
    }

    #[test]
    fn is_uniform_true() {
        let runs = rle_encode(&[7u8; 5]);
        assert!(rle_is_uniform(&runs));
    }

    #[test]
    fn merge_adjacent() {
        let runs = vec![
            Run { value: 1, count: 3 },
            Run { value: 1, count: 2 },
            Run { value: 2, count: 1 },
        ];
        let merged = rle_merge(&runs);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].count, 5);
    }

    #[test]
    fn compression_ratio_uniform() {
        let data = vec![9u8; 100];
        let runs = rle_encode(&data);
        let ratio = rle_compression_ratio_v2(&runs);
        assert!(ratio >= 100.0);
    }
}
