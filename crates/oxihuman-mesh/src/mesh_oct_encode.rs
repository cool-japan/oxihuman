// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

// Octahedral normal encoding/decoding.
// Encodes unit normals into 2D float coordinates in [-1, 1].

fn sign_not_zero(v: f32) -> f32 {
    if v >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// Encode a unit normal to octahedral float2.
#[allow(dead_code)]
pub fn oct_encode(n: [f32; 3]) -> [f32; 2] {
    let sum = n[0].abs() + n[1].abs() + n[2].abs();
    let p = if sum < 1e-10 {
        [0.0f32, 0.0]
    } else {
        [n[0] / sum, n[1] / sum]
    };
    if n[2] < 0.0 {
        [
            (1.0 - p[1].abs()) * sign_not_zero(p[0]),
            (1.0 - p[0].abs()) * sign_not_zero(p[1]),
        ]
    } else {
        p
    }
}

/// Decode octahedral float2 back to unit normal.
#[allow(dead_code)]
pub fn oct_decode(p: [f32; 2]) -> [f32; 3] {
    let z = 1.0 - p[0].abs() - p[1].abs();
    let n = if z < 0.0 {
        [
            (1.0 - p[1].abs()) * sign_not_zero(p[0]),
            (1.0 - p[0].abs()) * sign_not_zero(p[1]),
            z,
        ]
    } else {
        [p[0], p[1], z]
    };
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 1.0];
    }
    [n[0] / len, n[1] / len, n[2] / len]
}

/// Encode to u8 snorm in [0, 255].
#[allow(dead_code)]
pub fn oct_encode_u8(n: [f32; 3]) -> [u8; 2] {
    let e = oct_encode(n);
    let to_u8 = |v: f32| -> u8 {
        let clamped = v.clamp(-1.0, 1.0);
        ((clamped * 0.5 + 0.5) * 255.0).round() as u8
    };
    [to_u8(e[0]), to_u8(e[1])]
}

/// Decode from u8 snorm.
#[allow(dead_code)]
pub fn oct_decode_u8(b: [u8; 2]) -> [f32; 3] {
    let to_f = |v: u8| -> f32 { (v as f32 / 255.0) * 2.0 - 1.0 };
    oct_decode([to_f(b[0]), to_f(b[1])])
}

/// Encode a batch of normals.
#[allow(dead_code)]
pub fn oct_encode_batch(normals: &[[f32; 3]]) -> Vec<[f32; 2]> {
    normals.iter().map(|&n| oct_encode(n)).collect()
}

/// Decode a batch.
#[allow(dead_code)]
pub fn oct_decode_batch(encoded: &[[f32; 2]]) -> Vec<[f32; 3]> {
    encoded.iter().map(|&e| oct_decode(e)).collect()
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Max reconstruction error across a batch.
#[allow(dead_code)]
pub fn oct_encode_decode_error(normals: &[[f32; 3]]) -> f32 {
    if normals.is_empty() {
        return 0.0;
    }
    let encoded = oct_encode_batch(normals);
    let decoded = oct_decode_batch(&encoded);
    normals
        .iter()
        .zip(decoded.iter())
        .map(|(&a, &b)| {
            let d = dot3(a, b).clamp(-1.0, 1.0);
            (1.0 - d).abs()
        })
        .fold(0.0f32, f32::max)
}

/// Compute memory savings ratio vs 3 * f32.
#[allow(dead_code)]
pub fn oct_compression_ratio_f32() -> f32 {
    2.0 / 3.0
}

#[allow(dead_code)]
pub fn oct_encode_to_json(encoded: &[[f32; 2]]) -> String {
    format!("{{\"count\":{}}}", encoded.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_normals() -> Vec<[f32; 3]> {
        vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
        ]
    }

    #[test]
    fn test_encode_decode_round_trip() {
        for n in unit_normals() {
            let e = oct_encode(n);
            let d = oct_decode(e);
            let err = (d[0] - n[0]).abs() + (d[1] - n[1]).abs() + (d[2] - n[2]).abs();
            assert!(err < 0.01, "Round-trip error too large: {:?} -> {:?}", n, d);
        }
    }

    #[test]
    fn test_encode_u8_decode_u8_round_trip() {
        let n = [0.0f32, 0.0, 1.0];
        let enc = oct_encode_u8(n);
        let dec = oct_decode_u8(enc);
        let err = (dec[2] - 1.0).abs();
        assert!(err < 0.02);
    }

    #[test]
    fn test_batch_encode_count() {
        let ns = unit_normals();
        let enc = oct_encode_batch(&ns);
        assert_eq!(enc.len(), ns.len());
    }

    #[test]
    fn test_batch_decode_count() {
        let enc = vec![[0.0f32, 0.0]; 5];
        let dec = oct_decode_batch(&enc);
        assert_eq!(dec.len(), 5);
    }

    #[test]
    fn test_encode_decode_error_small() {
        let ns = unit_normals();
        let err = oct_encode_decode_error(&ns);
        assert!(err < 0.02);
    }

    #[test]
    fn test_empty_encode() {
        let enc = oct_encode_batch(&[]);
        assert_eq!(enc.len(), 0);
    }

    #[test]
    fn test_empty_error() {
        assert_eq!(oct_encode_decode_error(&[]), 0.0);
    }

    #[test]
    fn test_compression_ratio() {
        let r = oct_compression_ratio_f32();
        assert!((r - 2.0 / 3.0).abs() < 1e-6);
    }

    #[test]
    fn test_encode_to_json() {
        let ns = unit_normals();
        let enc = oct_encode_batch(&ns);
        let j = oct_encode_to_json(&enc);
        assert!(j.contains("count"));
    }

    #[test]
    fn test_decoded_normals_unit_length() {
        let ns = unit_normals();
        let enc = oct_encode_batch(&ns);
        let dec = oct_decode_batch(&enc);
        for d in dec {
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            assert!((len - 1.0).abs() < 0.01);
        }
    }
}
