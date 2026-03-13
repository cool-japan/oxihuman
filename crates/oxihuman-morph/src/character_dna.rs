// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

//! DNA-like compact binary encoding of morph parameters for sharing and
//! seeding character variants.
//!
//! A short base64 or hex string encodes the full character morph state,
//! enabling easy sharing, mutation, and crossover of character designs.

use crate::params::ParamState;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Base64 alphabet (standard, with padding)
// ---------------------------------------------------------------------------

const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A compact DNA representation of a character's parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterDna {
    /// Raw bytes encoding the parameters.
    pub bytes: Vec<u8>,
    /// Version tag for format compatibility.
    pub version: u8,
}

/// Extended DNA that includes named extra params beyond the core 4.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendedDna {
    pub core: CharacterDna,
    pub extra_keys: Vec<String>,
}

// ---------------------------------------------------------------------------
// LCG random number generator (no external crates)
// ---------------------------------------------------------------------------

fn lcg(state: &mut u64) -> u8 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((*state >> 33) & 0xFF) as u8
}

// ---------------------------------------------------------------------------
// Encode / decode
// ---------------------------------------------------------------------------

/// Encode a [`ParamState`] into a compact [`CharacterDna`].
///
/// Format:
/// - byte 0: version (1)
/// - bytes 1–4: height, weight, muscle, age each as u8 (0=0.0, 255=1.0)
/// - remaining: for each (key, value) in `params.extra`:
///   key_len(u8) + key_bytes + value(u8)
pub fn encode_dna(params: &ParamState) -> CharacterDna {
    let mut bytes = vec![
        1u8,
        f32_to_u8(params.height),
        f32_to_u8(params.weight),
        f32_to_u8(params.muscle),
        f32_to_u8(params.age),
    ];

    // extra params — sort for deterministic encoding
    let mut extras: Vec<(&String, &f32)> = params.extra.iter().collect();
    extras.sort_by_key(|(k, _)| k.as_str());

    for (key, val) in extras {
        let key_bytes = key.as_bytes();
        // Clamp key length to u8::MAX
        let key_len = key_bytes.len().min(255) as u8;
        bytes.push(key_len);
        bytes.extend_from_slice(&key_bytes[..key_len as usize]);
        bytes.push(f32_to_u8(*val));
    }

    CharacterDna { bytes, version: 1 }
}

/// Decode a [`CharacterDna`] back into a [`ParamState`].
///
/// This is lossy (~0.004 precision due to u8 quantisation).
pub fn decode_dna(dna: &CharacterDna) -> ParamState {
    let bytes = &dna.bytes;

    if bytes.is_empty() {
        return ParamState::default();
    }

    // byte 0 is version (currently only 1 supported)
    let mut cursor = 1usize;

    let height = read_u8_f32(bytes, &mut cursor);
    let weight = read_u8_f32(bytes, &mut cursor);
    let muscle = read_u8_f32(bytes, &mut cursor);
    let age = read_u8_f32(bytes, &mut cursor);

    let mut extra = HashMap::new();

    while cursor < bytes.len() {
        // Read key length
        let key_len = bytes[cursor] as usize;
        cursor += 1;

        if cursor + key_len + 1 > bytes.len() {
            break; // truncated
        }

        let key_bytes = &bytes[cursor..cursor + key_len];
        cursor += key_len;

        let key = String::from_utf8_lossy(key_bytes).into_owned();
        let val = u8_to_f32(bytes[cursor]);
        cursor += 1;

        extra.insert(key, val);
    }

    ParamState {
        height,
        weight,
        muscle,
        age,
        extra,
    }
}

// ---------------------------------------------------------------------------
// Hex serialisation
// ---------------------------------------------------------------------------

/// Encode a [`CharacterDna`] as a lowercase hex string.
pub fn dna_to_hex(dna: &CharacterDna) -> String {
    dna.bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

/// Parse a hex string into a [`CharacterDna`].
///
/// Returns an error if the string contains non-hex characters or has odd length.
pub fn dna_from_hex(hex: &str) -> anyhow::Result<CharacterDna> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        anyhow::bail!("hex string has odd length: {}", hex.len());
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let chars: Vec<char> = hex.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let hi = hex_char(chars[i])?;
        let lo = hex_char(chars[i + 1])?;
        bytes.push((hi << 4) | lo);
        i += 2;
    }

    let version = bytes.first().copied().unwrap_or(1);
    Ok(CharacterDna { bytes, version })
}

// ---------------------------------------------------------------------------
// Base64 serialisation (hand-written, no external crates)
// ---------------------------------------------------------------------------

/// Encode a [`CharacterDna`] as a standard base64 string (with `=` padding).
pub fn dna_to_base64(dna: &CharacterDna) -> String {
    let input = &dna.bytes;
    let mut out = Vec::with_capacity(input.len().div_ceil(3) * 4);

    let mut i = 0;
    while i + 2 < input.len() {
        let b0 = input[i] as u32;
        let b1 = input[i + 1] as u32;
        let b2 = input[i + 2] as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_CHARS[((n >> 18) & 0x3F) as usize]);
        out.push(B64_CHARS[((n >> 12) & 0x3F) as usize]);
        out.push(B64_CHARS[((n >> 6) & 0x3F) as usize]);
        out.push(B64_CHARS[(n & 0x3F) as usize]);
        i += 3;
    }

    let rem = input.len() - i;
    if rem == 1 {
        let b0 = input[i] as u32;
        let n = b0 << 16;
        out.push(B64_CHARS[((n >> 18) & 0x3F) as usize]);
        out.push(B64_CHARS[((n >> 12) & 0x3F) as usize]);
        out.push(b'=');
        out.push(b'=');
    } else if rem == 2 {
        let b0 = input[i] as u32;
        let b1 = input[i + 1] as u32;
        let n = (b0 << 16) | (b1 << 8);
        out.push(B64_CHARS[((n >> 18) & 0x3F) as usize]);
        out.push(B64_CHARS[((n >> 12) & 0x3F) as usize]);
        out.push(B64_CHARS[((n >> 6) & 0x3F) as usize]);
        out.push(b'=');
    }

    // SAFETY: all bytes come from B64_CHARS which is valid ASCII
    unsafe { String::from_utf8_unchecked(out) }
}

/// Decode a standard base64 string into a [`CharacterDna`].
pub fn dna_from_base64(s: &str) -> anyhow::Result<CharacterDna> {
    let s = s.trim();
    if !s.len().is_multiple_of(4) {
        anyhow::bail!(
            "base64 string length must be a multiple of 4, got {}",
            s.len()
        );
    }

    let chars: Vec<u8> = s.bytes().collect();
    let mut bytes: Vec<u8> = Vec::with_capacity(s.len() / 4 * 3);
    let mut i = 0;

    while i < chars.len() {
        let c0 = b64_val(chars[i])?;
        let c1 = b64_val(chars[i + 1])?;
        let c2_raw = chars[i + 2];
        let c3_raw = chars[i + 3];

        bytes.push((c0 << 2) | (c1 >> 4));

        if c2_raw != b'=' {
            let c2 = b64_val(c2_raw)?;
            bytes.push(((c1 & 0x0F) << 4) | (c2 >> 2));
            if c3_raw != b'=' {
                let c3 = b64_val(c3_raw)?;
                bytes.push(((c2 & 0x03) << 6) | c3);
            }
        }

        i += 4;
    }

    let version = bytes.first().copied().unwrap_or(1);
    Ok(CharacterDna { bytes, version })
}

// ---------------------------------------------------------------------------
// Distance / mutation / crossover
// ---------------------------------------------------------------------------

/// Hamming-like distance: sum of absolute differences over the minimum length
/// of the two byte slices (ignoring the version byte).
pub fn dna_distance(a: &CharacterDna, b: &CharacterDna) -> f32 {
    let min_len = a.bytes.len().min(b.bytes.len());
    let sum: u32 = a.bytes[..min_len]
        .iter()
        .zip(b.bytes[..min_len].iter())
        .map(|(x, y)| (*x as i32 - *y as i32).unsigned_abs())
        .sum();
    sum as f32
}

/// Randomly mutate bytes in a [`CharacterDna`] based on `rate` (0.0–1.0).
///
/// Uses a simple LCG seeded by `seed`. The version byte (index 0) is never
/// mutated.
pub fn mutate_dna(dna: &CharacterDna, rate: f32, seed: u64) -> CharacterDna {
    let rate = rate.clamp(0.0, 1.0);
    let threshold = (rate * 255.0) as u8;
    let mut state = seed;
    let mut bytes = dna.bytes.clone();

    for (i, byte) in bytes.iter_mut().enumerate() {
        if i == 0 {
            continue; // preserve version byte
        }
        let roll = lcg(&mut state);
        if roll < threshold {
            let delta = lcg(&mut state);
            *byte = byte.wrapping_add(delta).wrapping_sub(128);
        }
    }

    CharacterDna {
        bytes,
        version: dna.version,
    }
}

/// Uniform byte-level crossover between two [`CharacterDna`] values.
///
/// For each byte position (starting after version), the LCG selects whether
/// to take the byte from `a` or `b`. The version byte is taken from `a`.
pub fn crossover_dna(a: &CharacterDna, b: &CharacterDna, seed: u64) -> CharacterDna {
    let len = a.bytes.len().max(b.bytes.len());
    let mut state = seed;
    let mut bytes = Vec::with_capacity(len);

    for i in 0..len {
        if i == 0 {
            bytes.push(a.version);
            continue;
        }
        let pick_a = (lcg(&mut state) & 1) == 0;
        let byte = if pick_a {
            a.bytes.get(i).copied().unwrap_or(0)
        } else {
            b.bytes.get(i).copied().unwrap_or(0)
        };
        bytes.push(byte);
    }

    CharacterDna {
        bytes,
        version: a.version,
    }
}

/// Decode a [`CharacterDna`] into a flat `HashMap<String, f32>` that always
/// contains `"height"`, `"weight"`, `"muscle"`, and `"age"`.
pub fn dna_to_params_map(dna: &CharacterDna) -> HashMap<String, f32> {
    let params = decode_dna(dna);
    let mut map = HashMap::new();
    map.insert("height".to_string(), params.height);
    map.insert("weight".to_string(), params.weight);
    map.insert("muscle".to_string(), params.muscle);
    map.insert("age".to_string(), params.age);
    for (k, v) in params.extra {
        map.insert(k, v);
    }
    map
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

#[inline]
fn f32_to_u8(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[inline]
fn u8_to_f32(b: u8) -> f32 {
    b as f32 / 255.0
}

#[inline]
fn read_u8_f32(bytes: &[u8], cursor: &mut usize) -> f32 {
    if *cursor < bytes.len() {
        let v = u8_to_f32(bytes[*cursor]);
        *cursor += 1;
        v
    } else {
        0.0
    }
}

fn hex_char(c: char) -> anyhow::Result<u8> {
    match c {
        '0'..='9' => Ok(c as u8 - b'0'),
        'a'..='f' => Ok(c as u8 - b'a' + 10),
        'A'..='F' => Ok(c as u8 - b'A' + 10),
        _ => anyhow::bail!("invalid hex character: {:?}", c),
    }
}

fn b64_val(c: u8) -> anyhow::Result<u8> {
    match c {
        b'A'..=b'Z' => Ok(c - b'A'),
        b'a'..=b'z' => Ok(c - b'a' + 26),
        b'0'..=b'9' => Ok(c - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        _ => anyhow::bail!("invalid base64 character: {:?}", c as char),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_params(height: f32, weight: f32, muscle: f32, age: f32) -> ParamState {
        ParamState::new(height, weight, muscle, age)
    }

    fn make_params_with_extra(
        height: f32,
        weight: f32,
        muscle: f32,
        age: f32,
        extra: &[(&str, f32)],
    ) -> ParamState {
        let mut p = make_params(height, weight, muscle, age);
        for (k, v) in extra {
            p.extra.insert(k.to_string(), *v);
        }
        p
    }

    // ------------------------------------------------------------------ 1
    #[test]
    fn test_encode_decode_roundtrip_core() {
        let params = make_params(0.7, 0.4, 0.6, 0.3);
        let dna = encode_dna(&params);
        let decoded = decode_dna(&dna);
        // Precision is ~1/255 ≈ 0.004
        assert!(
            (decoded.height - params.height).abs() < 0.005,
            "height mismatch"
        );
        assert!(
            (decoded.weight - params.weight).abs() < 0.005,
            "weight mismatch"
        );
        assert!(
            (decoded.muscle - params.muscle).abs() < 0.005,
            "muscle mismatch"
        );
        assert!((decoded.age - params.age).abs() < 0.005, "age mismatch");
    }

    // ------------------------------------------------------------------ 2
    #[test]
    fn test_encode_decode_roundtrip_with_extra() {
        let params = make_params_with_extra(
            0.5,
            0.5,
            0.5,
            0.5,
            &[("nose_width", 0.8), ("jaw_size", 0.2)],
        );
        let dna = encode_dna(&params);
        let decoded = decode_dna(&dna);
        assert!((decoded.extra["nose_width"] - 0.8).abs() < 0.005);
        assert!((decoded.extra["jaw_size"] - 0.2).abs() < 0.005);
    }

    // ------------------------------------------------------------------ 3
    #[test]
    fn test_version_field_is_one() {
        let params = make_params(0.5, 0.5, 0.5, 0.5);
        let dna = encode_dna(&params);
        assert_eq!(dna.version, 1);
        assert_eq!(dna.bytes[0], 1);
    }

    // ------------------------------------------------------------------ 4
    #[test]
    fn test_dna_to_hex_roundtrip() {
        let params = make_params(0.2, 0.8, 0.4, 0.9);
        let dna = encode_dna(&params);
        let hex = dna_to_hex(&dna);
        let dna2 = dna_from_hex(&hex).expect("should succeed");
        assert_eq!(dna.bytes, dna2.bytes);
    }

    // ------------------------------------------------------------------ 5
    #[test]
    fn test_dna_from_hex_invalid_char() {
        let result = dna_from_hex("01ZZ");
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------ 6
    #[test]
    fn test_dna_from_hex_odd_length() {
        let result = dna_from_hex("01a");
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------ 7
    #[test]
    fn test_dna_to_base64_roundtrip() {
        let params = make_params(0.3, 0.6, 0.9, 0.1);
        let dna = encode_dna(&params);
        let b64 = dna_to_base64(&dna);
        let dna2 = dna_from_base64(&b64).expect("should succeed");
        assert_eq!(dna.bytes, dna2.bytes);
    }

    // ------------------------------------------------------------------ 8
    #[test]
    fn test_dna_to_base64_padding() {
        // Ensure padded output length is multiple of 4
        let params = make_params(0.5, 0.5, 0.5, 0.5);
        let dna = encode_dna(&params);
        let b64 = dna_to_base64(&dna);
        assert_eq!(b64.len() % 4, 0);
    }

    // ------------------------------------------------------------------ 9
    #[test]
    fn test_dna_from_base64_invalid_length() {
        let result = dna_from_base64("ABC"); // not multiple of 4
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------ 10
    #[test]
    fn test_dna_distance_identical() {
        let params = make_params(0.5, 0.5, 0.5, 0.5);
        let dna = encode_dna(&params);
        assert_eq!(dna_distance(&dna, &dna), 0.0);
    }

    // ------------------------------------------------------------------ 11
    #[test]
    fn test_dna_distance_different() {
        let dna_a = encode_dna(&make_params(0.0, 0.0, 0.0, 0.0));
        let dna_b = encode_dna(&make_params(1.0, 1.0, 1.0, 1.0));
        let dist = dna_distance(&dna_a, &dna_b);
        // Each of the 4 core bytes differs by 255, version byte is the same
        assert!(dist > 0.0, "distance should be positive");
        assert!(
            (dist - 255.0 * 4.0).abs() < 1.0,
            "expected ~1020, got {}",
            dist
        );
    }

    // ------------------------------------------------------------------ 12
    #[test]
    fn test_mutate_dna_deterministic() {
        let params = make_params(0.5, 0.5, 0.5, 0.5);
        let dna = encode_dna(&params);
        let m1 = mutate_dna(&dna, 0.5, 42);
        let m2 = mutate_dna(&dna, 0.5, 42);
        assert_eq!(m1.bytes, m2.bytes);
    }

    // ------------------------------------------------------------------ 13
    #[test]
    fn test_mutate_dna_zero_rate_unchanged() {
        let params = make_params(0.5, 0.5, 0.5, 0.5);
        let dna = encode_dna(&params);
        let mutated = mutate_dna(&dna, 0.0, 99);
        assert_eq!(dna.bytes, mutated.bytes);
    }

    // ------------------------------------------------------------------ 14
    #[test]
    fn test_mutate_dna_preserves_version() {
        let params = make_params(0.4, 0.6, 0.3, 0.7);
        let dna = encode_dna(&params);
        let mutated = mutate_dna(&dna, 1.0, 12345);
        assert_eq!(mutated.version, 1);
        assert_eq!(mutated.bytes[0], 1);
    }

    // ------------------------------------------------------------------ 15
    #[test]
    fn test_crossover_dna_deterministic() {
        let dna_a = encode_dna(&make_params(0.1, 0.2, 0.3, 0.4));
        let dna_b = encode_dna(&make_params(0.9, 0.8, 0.7, 0.6));
        let c1 = crossover_dna(&dna_a, &dna_b, 7);
        let c2 = crossover_dna(&dna_a, &dna_b, 7);
        assert_eq!(c1.bytes, c2.bytes);
    }

    // ------------------------------------------------------------------ 16
    #[test]
    fn test_crossover_dna_bytes_come_from_parents() {
        let dna_a = encode_dna(&make_params(0.0, 0.0, 0.0, 0.0));
        let dna_b = encode_dna(&make_params(1.0, 1.0, 1.0, 1.0));
        let child = crossover_dna(&dna_a, &dna_b, 99);
        for (i, &byte) in child.bytes.iter().enumerate() {
            if i == 0 {
                assert_eq!(byte, 1, "version byte must be 1");
                continue;
            }
            let a_byte = dna_a.bytes.get(i).copied().unwrap_or(0);
            let b_byte = dna_b.bytes.get(i).copied().unwrap_or(0);
            assert!(
                byte == a_byte || byte == b_byte,
                "byte {} = {} must come from a ({}) or b ({})",
                i,
                byte,
                a_byte,
                b_byte
            );
        }
    }

    // ------------------------------------------------------------------ 17
    #[test]
    fn test_dna_to_params_map_core_keys_present() {
        let dna = encode_dna(&make_params(0.25, 0.75, 0.5, 0.0));
        let map = dna_to_params_map(&dna);
        assert!(map.contains_key("height"));
        assert!(map.contains_key("weight"));
        assert!(map.contains_key("muscle"));
        assert!(map.contains_key("age"));
    }

    // ------------------------------------------------------------------ 18
    #[test]
    fn test_dna_to_params_map_values_accurate() {
        let dna = encode_dna(&make_params(0.25, 0.75, 0.5, 1.0));
        let map = dna_to_params_map(&dna);
        assert!((map["height"] - 0.25).abs() < 0.005);
        assert!((map["weight"] - 0.75).abs() < 0.005);
        assert!((map["muscle"] - 0.5).abs() < 0.005);
        assert!((map["age"] - 1.0).abs() < 0.005);
    }

    // ------------------------------------------------------------------ 19
    #[test]
    fn test_empty_extra_params() {
        let params = make_params(0.5, 0.5, 0.5, 0.5);
        let dna = encode_dna(&params);
        // version(1) + 4 core bytes = 5 bytes total
        assert_eq!(dna.bytes.len(), 5);
        let decoded = decode_dna(&dna);
        assert!(decoded.extra.is_empty());
    }

    // ------------------------------------------------------------------ 20
    #[test]
    fn test_hex_output_is_lowercase() {
        let params = make_params(0.5, 0.5, 0.5, 0.5);
        let dna = encode_dna(&params);
        let hex = dna_to_hex(&dna);
        assert_eq!(hex, hex.to_lowercase());
    }

    // ------------------------------------------------------------------ 21
    #[test]
    fn test_hex_length_matches_bytes() {
        let params = make_params_with_extra(0.1, 0.2, 0.3, 0.4, &[("x", 0.5), ("y", 0.6)]);
        let dna = encode_dna(&params);
        let hex = dna_to_hex(&dna);
        assert_eq!(hex.len(), dna.bytes.len() * 2);
    }

    // ------------------------------------------------------------------ 22
    #[test]
    fn test_dna_boundary_values_zero() {
        let params = make_params(0.0, 0.0, 0.0, 0.0);
        let dna = encode_dna(&params);
        let decoded = decode_dna(&dna);
        assert!(decoded.height.abs() < 0.005);
        assert!(decoded.weight.abs() < 0.005);
        assert!(decoded.muscle.abs() < 0.005);
        assert!(decoded.age.abs() < 0.005);
    }

    // ------------------------------------------------------------------ 23
    #[test]
    fn test_dna_boundary_values_one() {
        let params = make_params(1.0, 1.0, 1.0, 1.0);
        let dna = encode_dna(&params);
        let decoded = decode_dna(&dna);
        assert!((decoded.height - 1.0).abs() < 0.005);
        assert!((decoded.weight - 1.0).abs() < 0.005);
        assert!((decoded.muscle - 1.0).abs() < 0.005);
        assert!((decoded.age - 1.0).abs() < 0.005);
    }

    // ------------------------------------------------------------------ 24
    #[test]
    fn test_extended_dna_struct() {
        let dna = encode_dna(&make_params(0.5, 0.5, 0.5, 0.5));
        let ext = ExtendedDna {
            core: dna.clone(),
            extra_keys: vec!["nose_width".to_string(), "jaw_size".to_string()],
        };
        assert_eq!(ext.core, dna);
        assert_eq!(ext.extra_keys.len(), 2);
    }

    // ------------------------------------------------------------------ 25
    #[test]
    fn test_lcg_produces_deterministic_sequence() {
        let mut s1 = 12345u64;
        let mut s2 = 12345u64;
        let seq1: Vec<u8> = (0..10).map(|_| lcg(&mut s1)).collect();
        let seq2: Vec<u8> = (0..10).map(|_| lcg(&mut s2)).collect();
        assert_eq!(seq1, seq2);
    }

    // ------------------------------------------------------------------ 26 (bonus)
    #[test]
    fn test_extra_keys_sorted_deterministically() {
        // Same extra params in different insertion orders should encode identically
        let mut p1 = make_params(0.5, 0.5, 0.5, 0.5);
        p1.extra.insert("z_key".to_string(), 0.3);
        p1.extra.insert("a_key".to_string(), 0.7);

        let mut p2 = make_params(0.5, 0.5, 0.5, 0.5);
        p2.extra.insert("a_key".to_string(), 0.7);
        p2.extra.insert("z_key".to_string(), 0.3);

        let d1 = encode_dna(&p1);
        let d2 = encode_dna(&p2);
        assert_eq!(d1.bytes, d2.bytes);
    }
}
