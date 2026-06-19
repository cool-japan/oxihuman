// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Unsigned LEB128 encoding/decoding for AV1 OBU `obu_size` fields.

/// Encode `v` as unsigned LEB128, appending bytes to `out`.
pub fn write_leb128(out: &mut Vec<u8>, v: u64) {
    let mut remaining = v;
    loop {
        let byte = (remaining & 0x7F) as u8;
        remaining >>= 7;
        if remaining != 0 {
            out.push(byte | 0x80);
        } else {
            out.push(byte);
            break;
        }
    }
}

/// Decode unsigned LEB128 from `data` starting at `*pos`.
///
/// On success advances `*pos` past all consumed bytes and returns the decoded value.
/// Returns `None` if data is exhausted before the sequence terminates, or if the
/// decoded value would overflow `u64`.
pub fn read_leb128(data: &[u8], pos: &mut usize) -> Option<u64> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        if *pos >= data.len() {
            return None;
        }
        let byte = data[*pos];
        *pos += 1;
        // Prevent shift overflow: LEB128 for u64 can be at most 10 bytes.
        if shift >= 63 {
            // The 10th byte can only contribute bit 63; any higher bits would overflow.
            let contribution = (byte & 0x7F) as u64;
            // Only bit 0 of the last group is valid for u64 (bit 63 of the value).
            if contribution > 1 {
                return None;
            }
            result |= contribution << shift;
            if byte & 0x80 != 0 {
                return None; // More bytes claimed but we've exhausted u64 space.
            }
            return Some(result);
        }
        let group = (byte & 0x7F) as u64;
        result |= group << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            return Some(result);
        }
    }
}

/// Return the number of bytes that `write_leb128` will produce for `v`.
pub fn leb128_size(v: u64) -> usize {
    if v == 0 {
        return 1;
    }
    let mut n = 0usize;
    let mut remaining = v;
    while remaining > 0 {
        remaining >>= 7;
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(v: u64) -> Vec<u8> {
        let mut out = Vec::new();
        write_leb128(&mut out, v);
        out
    }

    fn decode(data: &[u8]) -> Option<u64> {
        let mut pos = 0usize;
        read_leb128(data, &mut pos)
    }

    #[test]
    fn test_known_values() {
        // 0 → [0x00]
        assert_eq!(encode(0), vec![0x00]);
        // 127 → [0x7F]
        assert_eq!(encode(127), vec![0x7F]);
        // 128 → [0x80, 0x01]
        assert_eq!(encode(128), vec![0x80, 0x01]);
        // 16383 → [0xFF, 0x7F]
        assert_eq!(encode(16383), vec![0xFF, 0x7F]);
        // 16384 → [0x80, 0x80, 0x01]
        assert_eq!(encode(16384), vec![0x80, 0x80, 0x01]);
    }

    #[test]
    fn test_u64_max() {
        let bytes = encode(u64::MAX);
        // u64::MAX in LEB128 is 10 bytes.
        assert_eq!(bytes.len(), 10);
        let decoded = decode(&bytes).expect("should decode u64::MAX");
        assert_eq!(decoded, u64::MAX);
    }

    #[test]
    fn test_round_trips() {
        let values: &[u64] = &[
            0, 1, 63, 64, 127, 128, 255, 256, 16383, 16384, 0xFFFF, 0x1_0000,
            0xFFFF_FFFF, 0x1_0000_0000, u64::MAX / 2, u64::MAX - 1, u64::MAX,
        ];
        for &v in values {
            let bytes = encode(v);
            assert_eq!(
                bytes.len(),
                leb128_size(v),
                "leb128_size mismatch for {v}"
            );
            let mut pos = 0usize;
            let decoded = read_leb128(&bytes, &mut pos).expect("round-trip decode");
            assert_eq!(pos, bytes.len(), "pos not advanced fully for {v}");
            assert_eq!(decoded, v, "round-trip value mismatch for {v}");
        }
    }

    #[test]
    fn test_decode_with_offset() {
        // Encode two values back-to-back and decode sequentially.
        let mut buf = Vec::new();
        write_leb128(&mut buf, 300);
        write_leb128(&mut buf, 12345);
        let mut pos = 0usize;
        let v1 = read_leb128(&buf, &mut pos).unwrap();
        let v2 = read_leb128(&buf, &mut pos).unwrap();
        assert_eq!(v1, 300);
        assert_eq!(v2, 12345);
        assert_eq!(pos, buf.len());
    }

    #[test]
    fn test_decode_truncated_returns_none() {
        // A byte with continuation bit set but no following byte.
        let bad = vec![0x80];
        assert_eq!(decode(&bad), None);
    }

    #[test]
    fn test_leb128_size_known() {
        assert_eq!(leb128_size(0), 1);
        assert_eq!(leb128_size(127), 1);
        assert_eq!(leb128_size(128), 2);
        assert_eq!(leb128_size(16383), 2);
        assert_eq!(leb128_size(16384), 3);
        assert_eq!(leb128_size(u64::MAX), 10);
    }
}
