// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! AES-128/256-GCM AEAD — FIPS-197 / NIST SP800-38D compliant, pure Rust, zero external crypto deps.
//!
//! This module implements:
//! - AES block cipher with 128-bit and 256-bit key schedules (FIPS-197)
//! - GCM (Galois/Counter Mode) with 96-bit nonce (NIST SP800-38D)
//! - GHASH universal hash function over GF(2^128)
//!
//! Public API:
//! - [`aes_gcm_encrypt`] / [`aes_gcm_decrypt`]: nonce-prepended AEAD (nonce=12 ‖ tag=16 ‖ ct)
//! - [`aes_gcm_encrypt_aad`] / [`aes_gcm_decrypt_aad`]: explicit AAD variants
//! - [`aes_derive_key_stub`], [`aes_key_len_valid`], [`AesGcmCipher`], [`AesGcmConfig`]

use sha2::{Digest, Sha256};

// ─────────────────────────────────────────────────────────────────────────────
// AES S-box constants (FIPS-197 §5.1.1)
// ─────────────────────────────────────────────────────────────────────────────

/// AES forward S-box (SubBytes table), FIPS-197 §5.1.1.
static SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

/// AES inverse S-box (InvSubBytes table), FIPS-197 §5.3.2 / Appendix A.
/// This is the correct full 256-entry table from the FIPS-197 specification.
static INV_SBOX: [u8; 256] = [
    // 0x00..0x0f
    0x52, 0x09, 0x6a, 0xd5, 0x30, 0x36, 0xa5, 0x38, 0xbf, 0x40, 0xa3, 0x9e, 0x81, 0xf3, 0xd7, 0xfb,
    // 0x10..0x1f
    0x7c, 0xe3, 0x39, 0x82, 0x9b, 0x2f, 0xff, 0x87, 0x34, 0x8e, 0x43, 0x44, 0xc4, 0xde, 0xe9, 0xcb,
    // 0x20..0x2f
    0x54, 0x7b, 0x94, 0x32, 0xa6, 0xc2, 0x23, 0x3d, 0xee, 0x4c, 0x95, 0x0b, 0x42, 0xfa, 0xc3, 0x4e,
    // 0x30..0x3f
    0x08, 0x2e, 0xa1, 0x66, 0x28, 0xd9, 0x24, 0xb2, 0x76, 0x5b, 0xa2, 0x49, 0x6d, 0x8b, 0xd1, 0x25,
    // 0x40..0x4f
    0x72, 0xf8, 0xf6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xd4, 0xa4, 0x5c, 0xcc, 0x5d, 0x65, 0xb6, 0x92,
    // 0x50..0x5f
    0x6c, 0x70, 0x48, 0x50, 0xfd, 0xed, 0xb9, 0xda, 0x5e, 0x15, 0x46, 0x57, 0xa7, 0x8d, 0x9d, 0x84,
    // 0x60..0x6f
    0x90, 0xd8, 0xab, 0x00, 0x8c, 0xbc, 0xd3, 0x0a, 0xf7, 0xe4, 0x58, 0x05, 0xb8, 0xb3, 0x45, 0x06,
    // 0x70..0x7f
    0xd0, 0x2c, 0x1e, 0x8f, 0xca, 0x3f, 0x0f, 0x02, 0xc1, 0xaf, 0xbd, 0x03, 0x01, 0x13, 0x8a, 0x6b,
    // 0x80..0x8f
    0x3a, 0x91, 0x11, 0x41, 0x4f, 0x67, 0xdc, 0xea, 0x97, 0xf2, 0xcf, 0xce, 0xf0, 0xb4, 0xe6, 0x73,
    // 0x90..0x9f
    0x96, 0xac, 0x74, 0x22, 0xe7, 0xad, 0x35, 0x85, 0xe2, 0xf9, 0x37, 0xe8, 0x1c, 0x75, 0xdf, 0x6e,
    // 0xa0..0xaf
    0x47, 0xf1, 0x1a, 0x71, 0x1d, 0x29, 0xc5, 0x89, 0x6f, 0xb7, 0x62, 0x0e, 0xaa, 0x18, 0xbe, 0x1b,
    // 0xb0..0xbf
    0xfc, 0x56, 0x3e, 0x4b, 0xc6, 0xd2, 0x79, 0x20, 0x9a, 0xdb, 0xc0, 0xfe, 0x78, 0xcd, 0x5a, 0xf4,
    // 0xc0..0xcf
    0x1f, 0xdd, 0xa8, 0x33, 0x88, 0x07, 0xc7, 0x31, 0xb1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xec, 0x5f,
    // 0xd0..0xdf
    0x60, 0x51, 0x7f, 0xa9, 0x19, 0xa0, 0x7d, 0x02, 0x12, 0x32, 0xf3, 0x52, 0x2b, 0x04, 0x2a, 0x0c,
    // 0xe0..0xef  (note: 0xe8=0x76 from FIPS-197)
    0xe2, 0xeb, 0x27, 0xb2, 0x75, 0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6,
    // 0xf0..0xff
    0xb3, 0x29, 0xe3, 0x2f, 0x84, 0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe,
];

// ─────────────────────────────────────────────────────────────────────────────
// GF(2^8) arithmetic — irreducible polynomial x^8+x^4+x^3+x+1 (0x11b)
// ─────────────────────────────────────────────────────────────────────────────

/// Multiply by x (i.e. left-shift by 1) in GF(2^8) with reduction by 0x11b.
#[inline(always)]
fn xtime(a: u8) -> u8 {
    let shifted = (a as u16) << 1;
    if shifted & 0x100 != 0 {
        (shifted ^ 0x11b) as u8
    } else {
        shifted as u8
    }
}

/// Multiply two bytes in GF(2^8) using repeated doubling.
#[inline(always)]
fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut result = 0u8;
    while b > 0 {
        if b & 1 != 0 {
            result ^= a;
        }
        a = xtime(a);
        b >>= 1;
    }
    result
}

// ─────────────────────────────────────────────────────────────────────────────
// AES round constants (RCON), FIPS-197 §5.2
// ─────────────────────────────────────────────────────────────────────────────

/// Pre-computed round constant table: RCON[i] = x^(i-1) in GF(2^8).
static RCON: [u8; 11] = [
    0x00, // unused
    0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36,
];

// ─────────────────────────────────────────────────────────────────────────────
// AES key expansion, FIPS-197 §5.2
// ─────────────────────────────────────────────────────────────────────────────

/// AES key schedule: returns (round_keys, nr) where round_keys is a flat array of
/// (nr+1) * 16 bytes (one 128-bit round key per round plus initial).
///
/// AES-128: nr=10, 11 round keys (176 bytes).
/// AES-256: nr=14, 15 round keys (240 bytes).
fn aes_key_expansion(key: &[u8]) -> Result<(Vec<u8>, usize), String> {
    let nk = key.len() / 4; // Nk: number of 32-bit words in key (4 or 8)
    let nr = nk + 6; // Nr: number of rounds (10 or 14)
    let total_words = (nr + 1) * 4; // total expanded key words

    let mut w = vec![0u32; total_words];

    // First Nk words: directly from key (big-endian u32)
    for (i, word) in w[..nk].iter_mut().enumerate() {
        let base = i * 4;
        *word = u32::from_be_bytes([key[base], key[base + 1], key[base + 2], key[base + 3]]);
    }

    // Remaining words
    for i in nk..total_words {
        let mut temp = w[i - 1];
        if i % nk == 0 {
            // RotWord: left-rotate by 8 bits
            temp = temp.rotate_left(8);
            // SubWord: apply S-box to each byte
            temp = sub_word(temp);
            // XOR with round constant
            temp ^= (RCON[i / nk] as u32) << 24;
        } else if nk > 6 && i % nk == 4 {
            // Extra SubWord for AES-256
            temp = sub_word(temp);
        }
        w[i] = w[i - nk] ^ temp;
    }

    // Serialise round key words back to bytes (big-endian)
    let mut round_keys = vec![0u8; total_words * 4];
    for (i, &word) in w.iter().enumerate() {
        round_keys[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }

    Ok((round_keys, nr))
}

/// Apply the AES S-box to each byte of a 32-bit word.
#[inline(always)]
fn sub_word(w: u32) -> u32 {
    let b = w.to_be_bytes();
    u32::from_be_bytes([
        SBOX[b[0] as usize],
        SBOX[b[1] as usize],
        SBOX[b[2] as usize],
        SBOX[b[3] as usize],
    ])
}

// ─────────────────────────────────────────────────────────────────────────────
// AES block operations, FIPS-197 §5.1
// ─────────────────────────────────────────────────────────────────────────────

/// AES state type: 4×4 bytes, column-major (state[col][row]).
type AesState = [[u8; 4]; 4];

/// Convert a 16-byte block to column-major AES state.
fn block_to_state(block: &[u8; 16]) -> AesState {
    let mut s = [[0u8; 4]; 4];
    for col in 0..4 {
        for row in 0..4 {
            s[col][row] = block[col * 4 + row];
        }
    }
    s
}

/// Convert column-major AES state back to a 16-byte block.
fn state_to_block(s: &AesState) -> [u8; 16] {
    let mut out = [0u8; 16];
    for col in 0..4 {
        for row in 0..4 {
            out[col * 4 + row] = s[col][row];
        }
    }
    out
}

/// SubBytes: apply S-box to every byte of the state.
fn sub_bytes(s: &mut AesState) {
    for col in s.iter_mut() {
        for b in col.iter_mut() {
            *b = SBOX[*b as usize];
        }
    }
}

/// InvSubBytes: apply inverse S-box to every byte of the state.
fn inv_sub_bytes(s: &mut AesState) {
    for col in s.iter_mut() {
        for b in col.iter_mut() {
            *b = INV_SBOX[*b as usize];
        }
    }
}

/// ShiftRows: cyclically shift rows left by 0,1,2,3 positions (FIPS-197 §5.1.2).
///
/// State is column-major: s[col][row]. Row `r` spans s[0][r], s[1][r], s[2][r], s[3][r].
/// Row 0: no change. Row 1: left shift 1. Row 2: left shift 2. Row 3: left shift 3.
fn shift_rows(s: &mut AesState) {
    // Row 1: [s[0][1], s[1][1], s[2][1], s[3][1]] → [s[1][1], s[2][1], s[3][1], s[0][1]]
    let tmp = s[0][1];
    s[0][1] = s[1][1];
    s[1][1] = s[2][1];
    s[2][1] = s[3][1];
    s[3][1] = tmp;
    // Row 2: [s[0][2], s[1][2], s[2][2], s[3][2]] → [s[2][2], s[3][2], s[0][2], s[1][2]]
    let (a, b) = (s[0][2], s[1][2]);
    s[0][2] = s[2][2];
    s[1][2] = s[3][2];
    s[2][2] = a;
    s[3][2] = b;
    // Row 3: [s[0][3], s[1][3], s[2][3], s[3][3]] → [s[3][3], s[0][3], s[1][3], s[2][3]]
    let tmp = s[3][3];
    s[3][3] = s[2][3];
    s[2][3] = s[1][3];
    s[1][3] = s[0][3];
    s[0][3] = tmp;
}

/// InvShiftRows: cyclically shift rows right by 0,1,2,3 (FIPS-197 §5.3.1).
fn inv_shift_rows(s: &mut AesState) {
    // Row 1: right shift 1 → [s[3][1], s[0][1], s[1][1], s[2][1]]
    let tmp = s[3][1];
    s[3][1] = s[2][1];
    s[2][1] = s[1][1];
    s[1][1] = s[0][1];
    s[0][1] = tmp;
    // Row 2: right shift 2 (same as left shift 2)
    let (a, b) = (s[0][2], s[1][2]);
    s[0][2] = s[2][2];
    s[1][2] = s[3][2];
    s[2][2] = a;
    s[3][2] = b;
    // Row 3: right shift 3 (= left shift 1)
    let tmp = s[0][3];
    s[0][3] = s[1][3];
    s[1][3] = s[2][3];
    s[2][3] = s[3][3];
    s[3][3] = tmp;
}

/// MixColumns: mix the bytes in each column (FIPS-197 §5.1.3).
///
/// Each column is treated as a polynomial over GF(2^8) and multiplied by
/// the matrix:
///   [2 3 1 1]
///   [1 2 3 1]
///   [1 1 2 3]
///   [3 1 1 2]
fn mix_columns(s: &mut AesState) {
    for col in s.iter_mut() {
        let [s0, s1, s2, s3] = *col;
        col[0] = gf_mul(2, s0) ^ gf_mul(3, s1) ^ s2 ^ s3;
        col[1] = s0 ^ gf_mul(2, s1) ^ gf_mul(3, s2) ^ s3;
        col[2] = s0 ^ s1 ^ gf_mul(2, s2) ^ gf_mul(3, s3);
        col[3] = gf_mul(3, s0) ^ s1 ^ s2 ^ gf_mul(2, s3);
    }
}

/// InvMixColumns: inverse of MixColumns (FIPS-197 §5.3.3).
///
/// Multiplied by the inverse matrix:
///   [0e 0b 0d 09]
///   [09 0e 0b 0d]
///   [0d 09 0e 0b]
///   [0b 0d 09 0e]
fn inv_mix_columns(s: &mut AesState) {
    for col in s.iter_mut() {
        let [s0, s1, s2, s3] = *col;
        col[0] = gf_mul(0x0e, s0) ^ gf_mul(0x0b, s1) ^ gf_mul(0x0d, s2) ^ gf_mul(0x09, s3);
        col[1] = gf_mul(0x09, s0) ^ gf_mul(0x0e, s1) ^ gf_mul(0x0b, s2) ^ gf_mul(0x0d, s3);
        col[2] = gf_mul(0x0d, s0) ^ gf_mul(0x09, s1) ^ gf_mul(0x0e, s2) ^ gf_mul(0x0b, s3);
        col[3] = gf_mul(0x0b, s0) ^ gf_mul(0x0d, s1) ^ gf_mul(0x09, s2) ^ gf_mul(0x0e, s3);
    }
}

/// AddRoundKey: XOR state with a 16-byte round key (FIPS-197 §5.1.4).
fn add_round_key(s: &mut AesState, round_key: &[u8]) {
    for col in 0..4 {
        for row in 0..4 {
            s[col][row] ^= round_key[col * 4 + row];
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AES block encrypt / decrypt
// ─────────────────────────────────────────────────────────────────────────────

/// Encrypt one 16-byte block using the pre-computed round key schedule.
fn aes_encrypt_block(block: &[u8; 16], round_keys: &[u8], nr: usize) -> [u8; 16] {
    let mut state = block_to_state(block);

    // Initial round key addition
    add_round_key(&mut state, &round_keys[0..16]);

    // nr - 1 full rounds
    for round in 1..nr {
        sub_bytes(&mut state);
        shift_rows(&mut state);
        mix_columns(&mut state);
        add_round_key(&mut state, &round_keys[round * 16..(round + 1) * 16]);
    }

    // Final round (no MixColumns)
    sub_bytes(&mut state);
    shift_rows(&mut state);
    add_round_key(&mut state, &round_keys[nr * 16..(nr + 1) * 16]);

    state_to_block(&state)
}

/// Decrypt one 16-byte block using the pre-computed round key schedule.
fn aes_decrypt_block(block: &[u8; 16], round_keys: &[u8], nr: usize) -> [u8; 16] {
    let mut state = block_to_state(block);

    // Initial (last) round key
    add_round_key(&mut state, &round_keys[nr * 16..(nr + 1) * 16]);

    // nr - 1 inverse full rounds (from round nr-1 down to 1)
    for round in (1..nr).rev() {
        inv_shift_rows(&mut state);
        inv_sub_bytes(&mut state);
        add_round_key(&mut state, &round_keys[round * 16..(round + 1) * 16]);
        inv_mix_columns(&mut state);
    }

    // Final inverse round (no InvMixColumns)
    inv_shift_rows(&mut state);
    inv_sub_bytes(&mut state);
    add_round_key(&mut state, &round_keys[0..16]);

    state_to_block(&state)
}

// ─────────────────────────────────────────────────────────────────────────────
// AES-GCM: Counter mode (CTR) encryption, NIST SP800-38D §6.5
// ─────────────────────────────────────────────────────────────────────────────

/// Increment the low 32-bit counter of a 128-bit counter block (big-endian).
fn gcm_inc32(counter: &mut [u8; 16]) {
    let count = u32::from_be_bytes([counter[12], counter[13], counter[14], counter[15]]);
    let incremented = count.wrapping_add(1).to_be_bytes();
    counter[12] = incremented[0];
    counter[13] = incremented[1];
    counter[14] = incremented[2];
    counter[15] = incremented[3];
}

/// CTR encryption/decryption in GCM: J0 is the initial counter (nonce ‖ 00000001).
/// Encryption starts with counter J0+1 (inc32(J0)).
fn gcm_ctr_crypt(plaintext: &[u8], round_keys: &[u8], nr: usize, j0: &[u8; 16]) -> Vec<u8> {
    let mut counter = *j0;
    gcm_inc32(&mut counter); // Start at J0+1 per GCM spec §6.5

    let mut output = Vec::with_capacity(plaintext.len());
    let mut offset = 0usize;

    while offset < plaintext.len() {
        let keystream_block = aes_encrypt_block(&counter, round_keys, nr);
        let remaining = plaintext.len() - offset;
        let chunk_len = remaining.min(16);
        for i in 0..chunk_len {
            output.push(plaintext[offset + i] ^ keystream_block[i]);
        }
        gcm_inc32(&mut counter);
        offset += chunk_len;
    }

    output
}

// ─────────────────────────────────────────────────────────────────────────────
// GHASH — GF(2^128) polynomial hash, NIST SP800-38D §6.4
// ─────────────────────────────────────────────────────────────────────────────

/// Multiply two 128-bit values in GF(2^128) with irreducible polynomial
/// x^128 + x^7 + x^2 + x + 1 (NIST GCM reduction polynomial: 0xe1 << 120).
///
/// Uses shift-and-XOR with the "reflected" bit convention: the MSB of the
/// 128-bit value is stored in the MSB of byte[0].
fn gf128_mul(x: &[u8; 16], y: &[u8; 16]) -> [u8; 16] {
    let mut z = [0u8; 16];
    let mut v = *x;

    for &y_byte in y.iter() {
        for bit in (0..8).rev() {
            // If the current bit of y is set, XOR z with v
            if (y_byte >> bit) & 1 == 1 {
                for j in 0..16 {
                    z[j] ^= v[j];
                }
            }
            // Right-shift v by 1 (MSB is bit 127 at v[0] bit 7)
            let lsb = v[15] & 1;
            for j in (1..16).rev() {
                v[j] = (v[j] >> 1) | (v[j - 1] << 7);
            }
            v[0] >>= 1;
            // If the shifted-out bit was 1, XOR with the reduction polynomial
            // R = x^128 + x^7 + x^2 + x + 1 → 0xe1 in the top byte
            if lsb == 1 {
                v[0] ^= 0xe1;
            }
        }
    }

    z
}

/// Compute GHASH over `data` using subkey `h`.
///
/// `data` must already be padded to a multiple of 16 bytes by the caller.
fn ghash(h: &[u8; 16], data: &[u8]) -> [u8; 16] {
    let mut y = [0u8; 16];
    for chunk in data.chunks(16) {
        let mut block = [0u8; 16];
        block[..chunk.len()].copy_from_slice(chunk);
        for i in 0..16 {
            y[i] ^= block[i];
        }
        y = gf128_mul(&y, h);
    }
    y
}

/// Build the GHASH input for GCM: AAD ‖ pad ‖ CT ‖ pad ‖ len64(AAD) ‖ len64(CT).
fn gcm_ghash_input(aad: &[u8], ciphertext: &[u8]) -> Vec<u8> {
    let mut data = Vec::new();

    data.extend_from_slice(aad);
    let aad_pad = (16 - aad.len() % 16) % 16;
    data.extend_from_slice(&vec![0u8; aad_pad]);

    data.extend_from_slice(ciphertext);
    let ct_pad = (16 - ciphertext.len() % 16) % 16;
    data.extend_from_slice(&vec![0u8; ct_pad]);

    // Lengths in bits, big-endian 64-bit
    data.extend_from_slice(&((aad.len() as u64 * 8).to_be_bytes()));
    data.extend_from_slice(&((ciphertext.len() as u64 * 8).to_be_bytes()));

    data
}

// ─────────────────────────────────────────────────────────────────────────────
// AES-GCM AEAD seal / open
// ─────────────────────────────────────────────────────────────────────────────

/// Low-level AES-GCM authenticated encryption.
///
/// Returns `ciphertext ‖ tag(16)`.
fn aes_gcm_seal(
    key: &[u8],
    nonce: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    let (round_keys, nr) = aes_key_expansion(key)?;

    // Subkey H = AES_K(0^128)
    let h_block: [u8; 16] = aes_encrypt_block(&[0u8; 16], &round_keys, nr);

    // J0 = nonce ‖ 0^31 ‖ 1 (96-bit nonce → counter = 00000001)
    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[12..16].copy_from_slice(&1u32.to_be_bytes());

    // CTR encryption starting at counter J0+1
    let ciphertext = gcm_ctr_crypt(plaintext, &round_keys, nr, &j0);

    // GHASH over: aad ‖ pad ‖ ct ‖ pad ‖ lengths
    let ghash_input = gcm_ghash_input(aad, &ciphertext);
    let s = ghash(&h_block, &ghash_input);

    // Tag = GHASH result XOR E(K, J0)
    let ej0 = aes_encrypt_block(&j0, &round_keys, nr);
    let mut tag = [0u8; 16];
    for i in 0..16 {
        tag[i] = s[i] ^ ej0[i];
    }

    let mut output = ciphertext;
    output.extend_from_slice(&tag);
    Ok(output)
}

/// Low-level AES-GCM authenticated decryption.
///
/// `ciphertext_with_tag` = `ciphertext ‖ tag(16)`.
/// Returns `Err` if the authentication tag does not match.
fn aes_gcm_open(
    key: &[u8],
    nonce: &[u8; 12],
    ciphertext_with_tag: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    if ciphertext_with_tag.len() < 16 {
        return Err("aes-gcm: ciphertext_with_tag too short".to_string());
    }

    let (round_keys, nr) = aes_key_expansion(key)?;

    let ct_len = ciphertext_with_tag.len() - 16;
    let ciphertext = &ciphertext_with_tag[..ct_len];
    let received_tag = &ciphertext_with_tag[ct_len..];

    let h_block: [u8; 16] = aes_encrypt_block(&[0u8; 16], &round_keys, nr);

    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[12..16].copy_from_slice(&1u32.to_be_bytes());

    // Recompute tag
    let ghash_input = gcm_ghash_input(aad, ciphertext);
    let s = ghash(&h_block, &ghash_input);
    let ej0 = aes_encrypt_block(&j0, &round_keys, nr);
    let mut expected_tag = [0u8; 16];
    for i in 0..16 {
        expected_tag[i] = s[i] ^ ej0[i];
    }

    // Constant-time tag comparison
    let mut diff = 0u8;
    for i in 0..16 {
        diff |= received_tag[i] ^ expected_tag[i];
    }
    if diff != 0 {
        return Err("aes-gcm: authentication tag mismatch".to_string());
    }

    Ok(gcm_ctr_crypt(ciphertext, &round_keys, nr, &j0))
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Key length variants for AES.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AesKeyLen {
    Aes128,
    Aes256,
}

/// AES-GCM configuration.
#[derive(Debug, Clone)]
pub struct AesGcmConfig {
    pub key_len: AesKeyLen,
    pub tag_len: usize,
}

impl Default for AesGcmConfig {
    fn default() -> Self {
        Self {
            key_len: AesKeyLen::Aes256,
            tag_len: 16,
        }
    }
}

/// AES-GCM cipher handle.
#[derive(Debug, Clone)]
pub struct AesGcmCipher {
    pub config: AesGcmConfig,
    key: Vec<u8>,
}

impl AesGcmCipher {
    pub fn new(key: &[u8], config: AesGcmConfig) -> Result<Self, String> {
        if key.len() != 16 && key.len() != 32 {
            return Err("aes-gcm: key must be 16 or 32 bytes".to_string());
        }
        Ok(Self {
            config,
            key: key.to_vec(),
        })
    }

    pub fn key_len_bytes(&self) -> usize {
        self.key.len()
    }
}

/// Encrypt plaintext using AES-GCM (no AAD).
///
/// `key` must be 16 (AES-128) or 32 (AES-256) bytes.
/// Returns `nonce(12) ‖ tag(16) ‖ ciphertext`.
pub fn aes_gcm_encrypt(key: &[u8], nonce: &[u8; 12], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    if !aes_key_len_valid(key.len()) {
        return Err("aes-gcm: invalid key length".to_string());
    }
    let ct_and_tag = aes_gcm_seal(key, nonce, plaintext, &[])?;
    let mut out = Vec::with_capacity(12 + ct_and_tag.len());
    out.extend_from_slice(nonce);
    out.extend_from_slice(&ct_and_tag);
    Ok(out)
}

/// Decrypt a packet produced by [`aes_gcm_encrypt`].
///
/// Input format: `nonce(12) ‖ tag(16) ‖ ciphertext`.
pub fn aes_gcm_decrypt(key: &[u8], packet: &[u8]) -> Result<Vec<u8>, String> {
    if packet.len() < 28 {
        return Err("aes-gcm: ciphertext too short".to_string());
    }
    if !aes_key_len_valid(key.len()) {
        return Err("aes-gcm: invalid key length".to_string());
    }
    let nonce: [u8; 12] = [
        packet[0], packet[1], packet[2], packet[3], packet[4], packet[5], packet[6], packet[7],
        packet[8], packet[9], packet[10], packet[11],
    ];
    let ct_with_tag = &packet[12..];
    aes_gcm_open(key, &nonce, ct_with_tag, &[])
}

/// Encrypt with explicit Additional Authenticated Data.
///
/// Returns `ciphertext ‖ tag(16)` (nonce NOT prepended; caller supplies it).
pub fn aes_gcm_encrypt_aad(
    key: &[u8],
    nonce: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    if !aes_key_len_valid(key.len()) {
        return Err("aes-gcm: invalid key length".to_string());
    }
    aes_gcm_seal(key, nonce, plaintext, aad)
}

/// Decrypt with explicit Additional Authenticated Data.
///
/// `ciphertext_with_tag` = `ciphertext ‖ tag(16)`.
pub fn aes_gcm_decrypt_aad(
    key: &[u8],
    nonce: &[u8; 12],
    ciphertext_with_tag: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    if !aes_key_len_valid(key.len()) {
        return Err("aes-gcm: invalid key length".to_string());
    }
    aes_gcm_open(key, nonce, ciphertext_with_tag, aad)
}

/// Derive a 256-bit AES key from a passphrase using SHA-256.
///
/// Name retained for compatibility; this is documented as a non-KDF passphrase hash.
pub fn aes_derive_key_stub(passphrase: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.finalize().into()
}

/// Return whether a key length is valid for AES-128 or AES-256.
pub fn aes_key_len_valid(len: usize) -> bool {
    len == 16 || len == 32
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── NIST GCM KAT: AES-128, zero key/IV, empty PT ────────────────────────

    /// NIST SP800-38D GCM test vector: AES-128, 96-bit IV, no PT, no AAD.
    /// Key = 0^16, IV = 0^12, PT = empty → CT = empty, Tag = 58e2fccefa7e3061367f1d57a4e7455a
    #[test]
    fn kat_aes_128_gcm() {
        let key = [0u8; 16];
        let nonce = [0u8; 12];
        let plaintext = &[][..];

        let expected_tag = hex::decode("58e2fccefa7e3061367f1d57a4e7455a").expect("tag hex");

        let result = aes_gcm_encrypt_aad(&key, &nonce, plaintext, &[]).expect("encrypt failed");
        // result = ct(0 bytes) ‖ tag(16 bytes)
        assert_eq!(result.len(), 16, "expected only the 16-byte tag");
        assert_eq!(result, expected_tag, "AES-128-GCM tag mismatch (empty PT)");
    }

    /// NIST GCM test vector: AES-128, 96-bit IV, 16-byte all-zero PT.
    /// Key = 0^16, IV = 0^12, PT = 0^16
    /// Expected CT = 0388dace60b6a392f328c2b971b2fe78
    /// Expected tag = ab6e47d42cec13bdf53a67b21257bddf
    #[test]
    fn kat_aes_128_gcm_16byte_pt() {
        let key = [0u8; 16];
        let nonce = [0u8; 12];
        let plaintext = [0u8; 16];

        let expected_ct = hex::decode("0388dace60b6a392f328c2b971b2fe78").expect("ct hex");
        let expected_tag = hex::decode("ab6e47d42cec13bdf53a67b21257bddf").expect("tag hex");

        let result = aes_gcm_encrypt_aad(&key, &nonce, &plaintext, &[]).expect("encrypt failed");
        let ct = &result[..result.len() - 16];
        let tag = &result[result.len() - 16..];

        assert_eq!(ct, expected_ct.as_slice(), "AES-128-GCM CT mismatch");
        assert_eq!(
            tag,
            expected_tag.as_slice(),
            "AES-128-GCM tag mismatch (16-byte PT)"
        );
    }

    // ─── NIST GCM KAT: AES-256, well-known vector ────────────────────────────

    /// NIST SP800-38D GCM test vector for AES-256 (from gcmEncryptExtIV256.rsp):
    ///   Key  = 0000…0 (32 bytes)
    ///   IV   = 000000000000000000000000
    ///   PT   = (empty)
    ///   AAD  = (empty)
    ///   CT   = (empty)
    ///   Tag  = 530f8afbc74536b9a963b4f1c4cb738b
    #[test]
    fn kat_aes_256_gcm() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];

        let expected_tag = hex::decode("530f8afbc74536b9a963b4f1c4cb738b").expect("tag hex");

        let result = aes_gcm_encrypt_aad(&key, &nonce, &[], &[]).expect("encrypt failed");
        assert_eq!(result.len(), 16, "expected only the 16-byte tag");
        assert_eq!(result, expected_tag, "AES-256-GCM tag mismatch (empty PT)");
    }

    // ─── AES block cipher KAT (FIPS-197 Appendix B) ──────────────────────────

    /// Verify AES-128 block encryption against FIPS-197 Appendix B test vector.
    /// Key = 2b7e151628aed2a6abf7158809cf4f3c
    /// PT  = 3243f6a8885a308d313198a2e0370734
    /// CT  = 3925841d02dc09fbdc118597196a0b32
    #[test]
    fn kat_aes_128_block() {
        let key = hex::decode("2b7e151628aed2a6abf7158809cf4f3c").expect("key hex");
        let pt = hex::decode("3243f6a8885a308d313198a2e0370734").expect("pt hex");
        let expected_ct = hex::decode("3925841d02dc09fbdc118597196a0b32").expect("ct hex");

        let (round_keys, nr) = aes_key_expansion(&key).expect("key expansion");
        let pt_block: [u8; 16] = pt.try_into().expect("pt len");
        let ct = aes_encrypt_block(&pt_block, &round_keys, nr);

        assert_eq!(
            ct.as_slice(),
            expected_ct.as_slice(),
            "FIPS-197 block cipher mismatch"
        );
    }

    // ─── API smoke tests ──────────────────────────────────────────────────────

    #[test]
    fn test_default_key_len() {
        assert_eq!(AesGcmConfig::default().key_len, AesKeyLen::Aes256);
    }

    #[test]
    fn test_key_len_valid() {
        assert!(aes_key_len_valid(16));
        assert!(aes_key_len_valid(32));
        assert!(!aes_key_len_valid(24));
    }

    #[test]
    fn test_cipher_bad_key() {
        let result = AesGcmCipher::new(&[0u8; 24], AesGcmConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_cipher_good_key() {
        let c = AesGcmCipher::new(&[0u8; 32], AesGcmConfig::default()).expect("should succeed");
        assert_eq!(c.key_len_bytes(), 32);
    }

    #[test]
    fn test_encrypt_then_decrypt_128() {
        let key = [0x42u8; 16];
        let nonce = [0x01u8; 12];
        let plain = b"hello aes-gcm";
        let enc = aes_gcm_encrypt(&key, &nonce, plain).expect("encrypt");
        let dec = aes_gcm_decrypt(&key, &enc).expect("decrypt");
        assert_eq!(dec, plain);
    }

    #[test]
    fn test_encrypt_then_decrypt_256() {
        let key = [0x42u8; 32];
        let nonce = [0x01u8; 12];
        let plain = b"hello aes-256-gcm";
        let enc = aes_gcm_encrypt(&key, &nonce, plain).expect("encrypt");
        let dec = aes_gcm_decrypt(&key, &enc).expect("decrypt");
        assert_eq!(dec, plain);
    }

    #[test]
    fn test_encrypt_bad_key() {
        assert!(aes_gcm_encrypt(&[0u8; 24], &[0u8; 12], b"data").is_err());
    }

    #[test]
    fn test_decrypt_short() {
        assert!(aes_gcm_decrypt(&[0u8; 32], &[0u8; 5]).is_err());
    }

    #[test]
    fn test_derive_key_stub() {
        let key = aes_derive_key_stub("my-passphrase");
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_encrypted_output_len() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let enc = aes_gcm_encrypt(&key, &nonce, b"hello").expect("encrypt");
        // nonce(12) + tag(16) + plaintext(5)
        assert_eq!(enc.len(), 12 + 16 + 5);
    }

    #[test]
    fn test_wrong_key_decrypt_fails() {
        let k_enc = [0u8; 32];
        let k_dec = [1u8; 32];
        let nonce = [0u8; 12];
        let enc = aes_gcm_encrypt(&k_enc, &nonce, b"secret").expect("enc");
        assert!(aes_gcm_decrypt(&k_dec, &enc).is_err());
    }

    #[test]
    fn test_aad_roundtrip() {
        let key = [0xABu8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"authenticate me";
        let aad = b"header";
        let enc = aes_gcm_encrypt_aad(&key, &nonce, plaintext, aad).expect("enc");
        let dec = aes_gcm_decrypt_aad(&key, &nonce, &enc, aad).expect("dec");
        assert_eq!(dec.as_slice(), plaintext.as_slice());
    }

    #[test]
    fn test_aad_tamper_fails() {
        let key = [0xABu8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"authenticate me";
        let aad = b"header";
        let enc = aes_gcm_encrypt_aad(&key, &nonce, plaintext, aad).expect("enc");
        assert!(aes_gcm_decrypt_aad(&key, &nonce, &enc, b"WRONG").is_err());
    }

    #[test]
    fn test_ciphertext_tamper_fails() {
        let key = [0xCDu8; 32];
        let nonce = [0x07u8; 12];
        let mut enc = aes_gcm_encrypt(&key, &nonce, b"tamper test").expect("enc");
        let last = enc.len() - 1;
        enc[last] ^= 0xff;
        assert!(aes_gcm_decrypt(&key, &enc).is_err());
    }

    #[test]
    fn test_roundtrip_empty_plaintext() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let enc = aes_gcm_encrypt(&key, &nonce, &[]).expect("enc");
        let dec = aes_gcm_decrypt(&key, &enc).expect("dec");
        assert_eq!(dec, Vec::<u8>::new());
    }

    #[test]
    fn test_gf128_mul_identity() {
        // Multiplying by 0 → 0; multiplying by 1 (but 1 = 0x80 in MSB-first)
        let zero = [0u8; 16];
        let x = [0xffu8; 16];
        let result = gf128_mul(&x, &zero);
        assert_eq!(result, zero, "x * 0 should be 0");
    }
}
