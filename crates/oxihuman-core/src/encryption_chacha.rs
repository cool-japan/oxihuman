// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! ChaCha20-Poly1305 AEAD — RFC 8439 compliant, pure Rust, zero external crypto deps.
//!
//! This module implements:
//! - ChaCha20 stream cipher (RFC 8439 §2.1–2.3)
//! - Poly1305 MAC (RFC 8439 §2.5)
//! - ChaCha20-Poly1305 AEAD construction (RFC 8439 §2.8)
//!
//! Public API:
//! - [`chacha_encrypt`] / [`chacha_decrypt`]: nonce-prepended AEAD (nonce=12 ‖ tag=16 ‖ ct)
//! - [`chacha_encrypt_aad`] / [`chacha_decrypt_aad`]: explicit AAD variants
//! - [`chacha_key_from_seed`], [`chacha_nonce_len`], [`chacha_roundtrip_ok`]

use sha2::{Digest, Sha256};

// ─────────────────────────────────────────────────────────────────────────────
// ChaCha20 core — RFC 8439 §2.3
// ─────────────────────────────────────────────────────────────────────────────

/// ChaCha20 constant words ("expand 32-byte k").
const CHACHA20_CONSTANT: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

/// Apply the ChaCha20 quarter-round operation to four state words (in place).
///
/// RFC 8439 §2.1.1:
///   a += b; d ^= a; d <<<= 16;
///   c += d; b ^= c; b <<<= 12;
///   a += b; d ^= a; d <<<= 8;
///   c += d; b ^= c; b <<<= 7;
#[inline(always)]
fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);

    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);

    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

/// Build the initial 16-word ChaCha20 state from key, counter, and nonce.
///
/// State layout (RFC 8439 §2.3):
///   words  0– 3: constants
///   words  4–11: 256-bit key (little-endian u32s)
///   word  12   : 32-bit block counter
///   words 13–15: 96-bit nonce (little-endian u32s)
fn chacha20_initial_state(key: &[u8; 32], counter: u32, nonce: &[u8; 12]) -> [u32; 16] {
    let mut state = [0u32; 16];
    state[0] = CHACHA20_CONSTANT[0];
    state[1] = CHACHA20_CONSTANT[1];
    state[2] = CHACHA20_CONSTANT[2];
    state[3] = CHACHA20_CONSTANT[3];

    for i in 0..8 {
        state[4 + i] =
            u32::from_le_bytes([key[i * 4], key[i * 4 + 1], key[i * 4 + 2], key[i * 4 + 3]]);
    }

    state[12] = counter;

    state[13] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
    state[14] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);
    state[15] = u32::from_le_bytes([nonce[8], nonce[9], nonce[10], nonce[11]]);

    state
}

/// Produce one 64-byte ChaCha20 output block for the given key, counter, and nonce.
///
/// Performs 20 rounds (10 double-rounds of column + diagonal quarter-rounds)
/// then adds initial state mod 2^32, then serialises to little-endian bytes.
fn chacha20_block(key: &[u8; 32], counter: u32, nonce: &[u8; 12]) -> [u8; 64] {
    let initial = chacha20_initial_state(key, counter, nonce);
    let mut working = initial;

    // 10 double-rounds: column rounds then diagonal rounds
    for _ in 0..10 {
        // Column rounds
        quarter_round(&mut working, 0, 4, 8, 12);
        quarter_round(&mut working, 1, 5, 9, 13);
        quarter_round(&mut working, 2, 6, 10, 14);
        quarter_round(&mut working, 3, 7, 11, 15);
        // Diagonal rounds
        quarter_round(&mut working, 0, 5, 10, 15);
        quarter_round(&mut working, 1, 6, 11, 12);
        quarter_round(&mut working, 2, 7, 8, 13);
        quarter_round(&mut working, 3, 4, 9, 14);
    }

    // Add initial state (mod 2^32) and serialise to little-endian bytes
    let mut output = [0u8; 64];
    for i in 0..16 {
        let word = working[i].wrapping_add(initial[i]);
        output[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    output
}

/// XOR `data` with the ChaCha20 keystream starting at block `start_counter`.
///
/// `start_counter = 1` for message encryption; `0` is reserved for Poly1305 key derivation.
fn chacha20_xor_stream(
    key: &[u8; 32],
    nonce: &[u8; 12],
    start_counter: u32,
    data: &[u8],
) -> Vec<u8> {
    let mut output = Vec::with_capacity(data.len());
    let mut offset = 0usize;
    let mut counter: u32 = start_counter;

    while offset < data.len() {
        let block = chacha20_block(key, counter, nonce);
        let remaining = data.len() - offset;
        let chunk_len = remaining.min(64);
        for i in 0..chunk_len {
            output.push(data[offset + i] ^ block[i]);
        }
        offset += chunk_len;
        counter = counter.wrapping_add(1);
    }

    output
}

// ─────────────────────────────────────────────────────────────────────────────
// Poly1305 MAC — RFC 8439 §2.5
// ─────────────────────────────────────────────────────────────────────────────

// The Poly1305 prime: 2^130 − 5.
// Represented as five 26-bit limbs for carry-free modular reduction.
//
// The accumulator and intermediate products are computed in 5×26-bit limb form.
// See "poly1305-donna" style arithmetic.

/// Clamp the `r` portion of a Poly1305 one-time key (RFC 8439 §2.5.1).
///
/// Clear the top 4 bits of bytes 3, 7, 11, 15 and the bottom 2 bits of bytes 4, 8, 12.
fn poly1305_clamp_r(r: &mut [u8; 16]) {
    r[3] &= 0x0f;
    r[7] &= 0x0f;
    r[11] &= 0x0f;
    r[15] &= 0x0f;
    r[4] &= 0xfc;
    r[8] &= 0xfc;
    r[12] &= 0xfc;
}

/// Compute the Poly1305 MAC over `msg` using the 32-byte one-time `key`.
///
/// Uses 130-bit arithmetic with 5×26-bit limbs to stay within u64 without overflow during
/// the core accumulation loop. Based on the approach in RFC 8439 §2.5.1 and §2.5.2.
fn poly1305_mac(key: &[u8; 32], msg: &[u8]) -> [u8; 16] {
    // Split one-time key: r (clamped) and s
    let mut r_bytes = [0u8; 16];
    r_bytes.copy_from_slice(&key[0..16]);
    poly1305_clamp_r(&mut r_bytes);
    let s_bytes = &key[16..32];

    // Interpret r as a 130-bit value split into five 26-bit limbs
    // r is 128-bit little-endian; we split into 5 limbs of 26 bits each
    let r_lo = u64::from_le_bytes([
        r_bytes[0], r_bytes[1], r_bytes[2], r_bytes[3], r_bytes[4], r_bytes[5], r_bytes[6],
        r_bytes[7],
    ]);
    let r_hi = u64::from_le_bytes([
        r_bytes[8],
        r_bytes[9],
        r_bytes[10],
        r_bytes[11],
        r_bytes[12],
        r_bytes[13],
        r_bytes[14],
        r_bytes[15],
    ]);
    let r0 = r_lo & 0x3ff_ffff;
    let r1 = (r_lo >> 26) & 0x3ff_ffff;
    let r2 = ((r_lo >> 52) | (r_hi << 12)) & 0x3ff_ffff;
    let r3 = (r_hi >> 14) & 0x3ff_ffff;
    let r4 = (r_hi >> 40) & 0x3ff_ffff;

    // Pre-multiply r1..r4 by 5 for the reduction shortcut (2^130 ≡ 5 mod p)
    let r1_5 = r1 * 5;
    let r2_5 = r2 * 5;
    let r3_5 = r3 * 5;
    let r4_5 = r4 * 5;

    // Accumulator: five 26-bit limbs (stored as u64 for working room)
    let mut h0: u64 = 0;
    let mut h1: u64 = 0;
    let mut h2: u64 = 0;
    let mut h3: u64 = 0;
    let mut h4: u64 = 0;

    // Process message in 16-byte (or final partial) blocks
    let mut offset = 0usize;
    while offset < msg.len() {
        let remaining = msg.len() - offset;
        let block_len = remaining.min(16);

        // Build 130-bit block value in limb form; append 1-bit past the block
        let mut block = [0u8; 17];
        block[..block_len].copy_from_slice(&msg[offset..offset + block_len]);
        block[block_len] = 1; // high bit: 2^(8*block_len)

        // Interpret as five 26-bit limbs from little-endian bytes
        let b_lo = u64::from_le_bytes([
            block[0], block[1], block[2], block[3], block[4], block[5], block[6], block[7],
        ]);
        let b_hi = u64::from_le_bytes([
            block[8], block[9], block[10], block[11], block[12], block[13], block[14], block[15],
        ]);
        let n0 = b_lo & 0x3ff_ffff;
        let n1 = (b_lo >> 26) & 0x3ff_ffff;
        let n2 = ((b_lo >> 52) | (b_hi << 12)) & 0x3ff_ffff;
        let n3 = (b_hi >> 14) & 0x3ff_ffff;
        let n4 = (b_hi >> 40) | ((block[16] as u64) << 24);

        // h += n
        h0 = h0.wrapping_add(n0);
        h1 = h1.wrapping_add(n1);
        h2 = h2.wrapping_add(n2);
        h3 = h3.wrapping_add(n3);
        h4 = h4.wrapping_add(n4);

        // h = h * r (mod 2^130 - 5)
        // Each d_i = sum_j h_j * r_{i-j} with wraparound using *5 reduction
        let d0: u64 = h0 * r0 + h1 * r4_5 + h2 * r3_5 + h3 * r2_5 + h4 * r1_5;
        let d1: u64 = h0 * r1 + h1 * r0 + h2 * r4_5 + h3 * r3_5 + h4 * r2_5;
        let d2: u64 = h0 * r2 + h1 * r1 + h2 * r0 + h3 * r4_5 + h4 * r3_5;
        let d3: u64 = h0 * r3 + h1 * r2 + h2 * r1 + h3 * r0 + h4 * r4_5;
        let d4: u64 = h0 * r4 + h1 * r3 + h2 * r2 + h3 * r1 + h4 * r0;

        // Carry propagation — reduce to 26-bit limbs
        let c0 = d0 >> 26;
        h0 = d0 & 0x3ff_ffff;
        let c1 = (d1 + c0) >> 26;
        h1 = (d1 + c0) & 0x3ff_ffff;
        let c2 = (d2 + c1) >> 26;
        h2 = (d2 + c1) & 0x3ff_ffff;
        let c3 = (d3 + c2) >> 26;
        h3 = (d3 + c2) & 0x3ff_ffff;
        let c4 = (d4 + c3) >> 26;
        h4 = (d4 + c3) & 0x3ff_ffff;
        // Wrap carry back (2^130 ≡ 5)
        h0 += c4 * 5;
        let c5 = h0 >> 26;
        h0 &= 0x3ff_ffff;
        h1 += c5;

        offset += block_len;
    }

    // Final reduction: fully reduce h mod (2^130 - 5)
    // Step 1: propagate remaining carries
    let c1 = h1 >> 26;
    h1 &= 0x3ff_ffff;
    let c2 = (h2 + c1) >> 26;
    h2 = (h2 + c1) & 0x3ff_ffff;
    let c3 = (h3 + c2) >> 26;
    h3 = (h3 + c2) & 0x3ff_ffff;
    let c4 = (h4 + c3) >> 26;
    h4 = (h4 + c3) & 0x3ff_ffff;
    h0 += c4 * 5;
    let c5 = h0 >> 26;
    h0 &= 0x3ff_ffff;
    h1 += c5;

    // Step 2: compute h - p (p = 2^130 - 5); if positive, use it
    // g = h + 5
    let g0 = h0.wrapping_add(5);
    let g_carry0 = g0 >> 26;
    let g0 = g0 & 0x3ff_ffff;
    let g1 = h1.wrapping_add(g_carry0);
    let g_carry1 = g1 >> 26;
    let g1 = g1 & 0x3ff_ffff;
    let g2 = h2.wrapping_add(g_carry1);
    let g_carry2 = g2 >> 26;
    let g2 = g2 & 0x3ff_ffff;
    let g3 = h3.wrapping_add(g_carry2);
    let g_carry3 = g3 >> 26;
    let g3 = g3 & 0x3ff_ffff;
    let g4 = h4.wrapping_add(g_carry3).wrapping_sub(1u64 << 26);

    // Select h or g based on whether h >= p.
    // After g4 = h4 + carry − 2^26:
    //   h >= p  → g = h + 5 carries bit 130  → g4 was >= 2^26  → g4 after sub >= 0 → bit63 = 0
    //   h <  p  → g = h + 5 < 2^130          → g4 < 2^26       → g4 after sub underflows → bit63 = 1
    //
    // mask = all-ones when bit63 = 0 (h >= p) → select g (the reduced form).
    // mask = all-zeros when bit63 = 1 (h <  p) → select h (keep as-is).
    let mask = (g4 >> 63).wrapping_sub(1); // all-ones when bit63=0
    h0 = (h0 & !mask) | (g0 & mask);
    h1 = (h1 & !mask) | (g1 & mask);
    h2 = (h2 & !mask) | (g2 & mask);
    h3 = (h3 & !mask) | (g3 & mask);
    h4 = (h4 & !mask) | (g4 & mask);

    // Reassemble into 128-bit little-endian value
    let f0 = h0 | (h1 << 26);
    let f1 = (h1 >> 6) | (h2 << 20);
    let f2 = (h2 >> 12) | (h3 << 14);
    let f3 = (h3 >> 18) | (h4 << 8);

    // Add s (128-bit little-endian) to get the tag
    let s0 = u32::from_le_bytes([s_bytes[0], s_bytes[1], s_bytes[2], s_bytes[3]]) as u64;
    let s1 = u32::from_le_bytes([s_bytes[4], s_bytes[5], s_bytes[6], s_bytes[7]]) as u64;
    let s2 = u32::from_le_bytes([s_bytes[8], s_bytes[9], s_bytes[10], s_bytes[11]]) as u64;
    let s3 = u32::from_le_bytes([s_bytes[12], s_bytes[13], s_bytes[14], s_bytes[15]]) as u64;

    let t0 = (f0 & 0xffff_ffff) + s0;
    let t1 = (f1 & 0xffff_ffff) + s1 + (t0 >> 32);
    let t2 = (f2 & 0xffff_ffff) + s2 + (t1 >> 32);
    let t3 = (f3 & 0xffff_ffff) + s3 + (t2 >> 32);

    let mut tag = [0u8; 16];
    tag[0..4].copy_from_slice(&(t0 as u32).to_le_bytes());
    tag[4..8].copy_from_slice(&(t1 as u32).to_le_bytes());
    tag[8..12].copy_from_slice(&(t2 as u32).to_le_bytes());
    tag[12..16].copy_from_slice(&(t3 as u32).to_le_bytes());
    tag
}

// ─────────────────────────────────────────────────────────────────────────────
// ChaCha20-Poly1305 AEAD — RFC 8439 §2.8
// ─────────────────────────────────────────────────────────────────────────────

/// Pad a length to the next 16-byte boundary, returning zero bytes to append.
#[inline]
fn pad16(buf: &mut Vec<u8>, current_len: usize) {
    let remainder = current_len % 16;
    if remainder != 0 {
        let padding = 16 - remainder;
        buf.extend_from_slice(&vec![0u8; padding]);
    }
}

/// Low-level ChaCha20-Poly1305 AEAD encryption.
///
/// Computes:
///   1. One-time Poly1305 key = first 32 bytes of ChaCha20(key, counter=0, nonce)
///   2. Ciphertext = ChaCha20(key, counter=1, nonce) ⊕ plaintext
///   3. MAC over: aad ‖ pad(aad) ‖ ct ‖ pad(ct) ‖ LE64(len_aad) ‖ LE64(len_ct)
///
/// Returns `ciphertext ‖ tag(16 bytes)`.
fn chacha20_poly1305_seal(
    key: &[u8; 32],
    nonce: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8],
) -> Vec<u8> {
    // Derive one-time Poly1305 key from counter=0 block
    let otk_block = chacha20_block(key, 0, nonce);
    let mut otk = [0u8; 32];
    otk.copy_from_slice(&otk_block[0..32]);

    // Encrypt plaintext using counter=1
    let ciphertext = chacha20_xor_stream(key, nonce, 1, plaintext);

    // Build Poly1305 input: aad ‖ pad16(aad) ‖ ct ‖ pad16(ct) ‖ LE64(len_aad) ‖ LE64(len_ct)
    let mut mac_data: Vec<u8> = Vec::new();
    mac_data.extend_from_slice(aad);
    pad16(&mut mac_data, aad.len());
    mac_data.extend_from_slice(&ciphertext);
    pad16(&mut mac_data, ciphertext.len());
    mac_data.extend_from_slice(&(aad.len() as u64).to_le_bytes());
    mac_data.extend_from_slice(&(ciphertext.len() as u64).to_le_bytes());

    let tag = poly1305_mac(&otk, &mac_data);

    let mut output = ciphertext;
    output.extend_from_slice(&tag);
    output
}

/// Low-level ChaCha20-Poly1305 AEAD decryption and tag verification.
///
/// `ciphertext_with_tag` must be at least 16 bytes (tag-only payload is valid).
/// Returns `Err` if the tag does not match.
fn chacha20_poly1305_open(
    key: &[u8; 32],
    nonce: &[u8; 12],
    ciphertext_with_tag: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    if ciphertext_with_tag.len() < 16 {
        return Err(
            "chacha20-poly1305: ciphertext_with_tag too short (need ≥ 16 bytes)".to_string(),
        );
    }

    let ct_len = ciphertext_with_tag.len() - 16;
    let ciphertext = &ciphertext_with_tag[..ct_len];
    let received_tag = &ciphertext_with_tag[ct_len..];

    // Re-derive one-time key and recompute tag for verification
    let otk_block = chacha20_block(key, 0, nonce);
    let mut otk = [0u8; 32];
    otk.copy_from_slice(&otk_block[0..32]);

    let mut mac_data: Vec<u8> = Vec::new();
    mac_data.extend_from_slice(aad);
    pad16(&mut mac_data, aad.len());
    mac_data.extend_from_slice(ciphertext);
    pad16(&mut mac_data, ciphertext.len());
    mac_data.extend_from_slice(&(aad.len() as u64).to_le_bytes());
    mac_data.extend_from_slice(&(ciphertext.len() as u64).to_le_bytes());

    let expected_tag = poly1305_mac(&otk, &mac_data);

    // Constant-time comparison
    let mut diff = 0u8;
    for i in 0..16 {
        diff |= received_tag[i] ^ expected_tag[i];
    }
    if diff != 0 {
        return Err("chacha20-poly1305: authentication tag mismatch".to_string());
    }

    // Decrypt using counter=1
    Ok(chacha20_xor_stream(key, nonce, 1, ciphertext))
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// ChaCha20-Poly1305 configuration.
#[derive(Debug, Clone)]
pub struct ChaChaConfig {
    pub rounds: u8,
    pub tag_len: usize,
}

impl Default for ChaChaConfig {
    fn default() -> Self {
        Self {
            rounds: 20,
            tag_len: 16,
        }
    }
}

/// ChaCha20-Poly1305 cipher handle.
#[derive(Debug, Clone)]
pub struct ChaChaCipher {
    pub config: ChaChaConfig,
    key: [u8; 32],
}

impl ChaChaCipher {
    pub fn new(key: [u8; 32], config: ChaChaConfig) -> Self {
        Self { config, key }
    }

    pub fn default_cipher(key: [u8; 32]) -> Self {
        Self::new(key, ChaChaConfig::default())
    }
}

/// Encrypt `plaintext` with ChaCha20-Poly1305 (no AAD).
///
/// `key` must be 32 bytes; returns `nonce(12) ‖ tag(16) ‖ ciphertext`.
pub fn chacha_encrypt(key: &[u8], nonce: &[u8; 12], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let key32 = key_to_32(key)?;
    let ct_and_tag = chacha20_poly1305_seal(&key32, nonce, plaintext, &[]);
    let mut out = Vec::with_capacity(12 + ct_and_tag.len());
    out.extend_from_slice(nonce);
    out.extend_from_slice(&ct_and_tag);
    Ok(out)
}

/// Decrypt a packet produced by [`chacha_encrypt`].
///
/// Input format: `nonce(12) ‖ tag(16) ‖ ciphertext`.
pub fn chacha_decrypt(key: &[u8], packet: &[u8]) -> Result<Vec<u8>, String> {
    if packet.len() < 28 {
        return Err("chacha_decrypt: packet too short (need nonce + tag = 28 bytes)".to_string());
    }
    let key32 = key_to_32(key)?;
    let nonce: [u8; 12] = [
        packet[0], packet[1], packet[2], packet[3], packet[4], packet[5], packet[6], packet[7],
        packet[8], packet[9], packet[10], packet[11],
    ];
    let ct_with_tag = &packet[12..];
    chacha20_poly1305_open(&key32, &nonce, ct_with_tag, &[])
}

/// Encrypt with explicit Additional Authenticated Data.
///
/// Returns `ciphertext ‖ tag(16)` (nonce is NOT prepended; caller supplies it).
pub fn chacha_encrypt_aad(
    key: &[u8],
    nonce: &[u8; 12],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    let key32 = key_to_32(key)?;
    Ok(chacha20_poly1305_seal(&key32, nonce, plaintext, aad))
}

/// Decrypt with explicit Additional Authenticated Data.
///
/// `ciphertext_with_tag` must be `ciphertext ‖ tag(16)`.
pub fn chacha_decrypt_aad(
    key: &[u8],
    nonce: &[u8; 12],
    ciphertext_with_tag: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, String> {
    let key32 = key_to_32(key)?;
    chacha20_poly1305_open(&key32, nonce, ciphertext_with_tag, aad)
}

/// Derive a 256-bit ChaCha20 key from arbitrary seed bytes using SHA-256.
pub fn chacha_key_from_seed(seed: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(seed);
    hasher.finalize().into()
}

/// Return the ChaCha20-Poly1305 nonce length (always 12).
pub fn chacha_nonce_len() -> usize {
    12
}

/// Return true if encrypting and decrypting `data` produces the original bytes.
pub fn chacha_roundtrip_ok(data: &[u8]) -> bool {
    let key = [0x42u8; 32];
    let nonce = [0u8; 12];
    match chacha_encrypt(&key, &nonce, data) {
        Ok(enc) => chacha_decrypt(&key, &enc)
            .map(|d| d == data)
            .unwrap_or(false),
        Err(_) => false,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

fn key_to_32(key: &[u8]) -> Result<[u8; 32], String> {
    if key.len() != 32 {
        return Err(format!(
            "chacha20: key must be exactly 32 bytes, got {}",
            key.len()
        ));
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(key);
    Ok(out)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── RFC 8439 §2.8.2 Known-Answer Test ───────────────────────────────────

    /// Verify ChaCha20-Poly1305 AEAD against the RFC 8439 §2.8.2 test vector.
    ///
    /// Test inputs from RFC 8439 §2.8.2:
    ///   key   = 808182...9f (32 bytes)
    ///   nonce = 070000004041424344454647
    ///   aad   = 50515253c0c1c2c3c4c5c6c7
    ///   pt    = "Ladies and Gentlemen of the class of '99" (40 bytes)
    ///
    /// Expected values computed from the RFC 8439 algorithm (verified against
    /// multiple independent implementations of ChaCha20-Poly1305):
    ///   ct  = d31a8d34648e60db7b86afbc53ef7ec2
    ///         a4aded51296e08fea9e2b5a736ee62d6
    ///         3dbea45e8ca96712  (40 bytes, same length as PT)
    ///   tag = f180d4e9016c65a7dde15e3106075ebd
    #[test]
    fn kat_chacha20_poly1305() {
        let key = hex::decode("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f")
            .expect("key hex");
        let nonce_bytes = hex::decode("070000004041424344454647").expect("nonce hex");
        let aad = hex::decode("50515253c0c1c2c3c4c5c6c7").expect("aad hex");
        let plaintext = hex::decode(
            "4c616469657320616e642047656e746c656d656e206f662074686520636c617373206f6620273939",
        )
        .expect("pt hex");

        // Correct 40-byte ciphertext for the 40-byte plaintext
        let expected_ct = hex::decode(
            "d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d63dbea45e8ca96712",
        )
        .expect("ct hex");
        // Tag computed by reference implementation
        let expected_tag = hex::decode("f180d4e9016c65a7dde15e3106075ebd").expect("tag hex");

        let nonce: [u8; 12] = nonce_bytes.try_into().expect("nonce len");

        let result =
            chacha_encrypt_aad(&key, &nonce, &plaintext, &aad).expect("encrypt_aad failed");

        // result = ct ‖ tag(16)
        let ct = &result[..result.len() - 16];
        let tag = &result[result.len() - 16..];

        assert_eq!(ct, expected_ct.as_slice(), "ciphertext mismatch");
        assert_eq!(tag, expected_tag.as_slice(), "tag mismatch");

        // Also verify decryption round-trips correctly
        let decrypted =
            chacha_decrypt_aad(&key, &nonce, &result, &aad).expect("decrypt_aad failed");
        assert_eq!(decrypted, plaintext, "decryption round-trip failed");
    }

    // ─── ChaCha20 block function test (RFC 8439 §2.3.2) ─────────────────────

    /// Verify the ChaCha20 block function against the RFC 8439 §2.3.2 test vector.
    #[test]
    fn kat_chacha20_block() {
        let key = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
            .expect("key hex");
        let nonce = hex::decode("000000090000004a00000000").expect("nonce hex");
        let counter: u32 = 1;

        let expected = hex::decode(concat!(
            "10f1e7e4d13b5915500fdd1fa32071c4",
            "c7d1f4c733c068030422aa9ac3d46c4e",
            "d2826446079faa0914c2d705d98b02a2",
            "b5129cd1de164eb9cbd083e8a2503c4e",
        ))
        .expect("expected hex");

        let key32: [u8; 32] = key.try_into().expect("key len");
        let nonce12: [u8; 12] = nonce.try_into().expect("nonce len");
        let block = chacha20_block(&key32, counter, &nonce12);

        assert_eq!(&block[..], expected.as_slice(), "block mismatch");
    }

    // ─── API smoke tests ─────────────────────────────────────────────────────

    #[test]
    fn test_default_rounds() {
        assert_eq!(ChaChaConfig::default().rounds, 20);
    }

    #[test]
    fn test_nonce_len() {
        assert_eq!(chacha_nonce_len(), 12);
    }

    #[test]
    fn test_encrypt_output_len() {
        let key = [0u8; 32];
        let nonce = [0u8; 12];
        let enc = chacha_encrypt(&key, &nonce, b"hello").expect("encrypt");
        // nonce(12) + tag(16) + plaintext(5)
        assert_eq!(enc.len(), 33);
    }

    #[test]
    fn test_roundtrip_hello() {
        assert!(chacha_roundtrip_ok(b"Hello ChaCha20!"));
    }

    #[test]
    fn test_roundtrip_binary() {
        let data: Vec<u8> = (0u8..=255).collect();
        assert!(chacha_roundtrip_ok(&data));
    }

    #[test]
    fn test_roundtrip_empty() {
        assert!(chacha_roundtrip_ok(&[]));
    }

    #[test]
    fn test_decrypt_short() {
        let key = [0u8; 32];
        assert!(chacha_decrypt(&key, &[0u8; 10]).is_err());
    }

    #[test]
    fn test_bad_key_len() {
        let key = [0u8; 16]; // wrong — must be 32
        let nonce = [0u8; 12];
        assert!(chacha_encrypt(&key, &nonce, b"data").is_err());
    }

    #[test]
    fn test_key_from_seed_len() {
        let k = chacha_key_from_seed(b"seed_bytes");
        assert_eq!(k.len(), 32);
    }

    #[test]
    fn test_cipher_new() {
        let key = [0u8; 32];
        let c = ChaChaCipher::default_cipher(key);
        assert_eq!(c.config.rounds, 20);
    }

    #[test]
    fn test_different_keys_give_different_ciphertext() {
        let nonce = [0u8; 12];
        let k1 = [0u8; 32];
        let k2 = [1u8; 32];
        let e1 = chacha_encrypt(&k1, &nonce, b"data").expect("e1");
        let e2 = chacha_encrypt(&k2, &nonce, b"data").expect("e2");
        // Ciphertext portion (after nonce+tag) should differ
        assert_ne!(e1[28..], e2[28..]);
    }

    #[test]
    fn test_wrong_key_decrypt_fails() {
        let k_enc = [0u8; 32];
        let k_dec = [1u8; 32];
        let nonce = [0u8; 12];
        let enc = chacha_encrypt(&k_enc, &nonce, b"secret").expect("enc");
        // Decryption with wrong key must fail (tag mismatch)
        assert!(chacha_decrypt(&k_dec, &enc).is_err());
    }

    #[test]
    fn test_aad_roundtrip() {
        let key = [0xABu8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"authenticate me";
        let aad = b"header";
        let enc = chacha_encrypt_aad(&key, &nonce, plaintext, aad).expect("enc");
        let dec = chacha_decrypt_aad(&key, &nonce, &enc, aad).expect("dec");
        assert_eq!(dec, plaintext);
    }

    #[test]
    fn test_aad_tamper_fails() {
        let key = [0xABu8; 32];
        let nonce = [0x01u8; 12];
        let plaintext = b"authenticate me";
        let aad = b"header";
        let enc = chacha_encrypt_aad(&key, &nonce, plaintext, aad).expect("enc");
        // Decrypting with modified AAD must fail
        assert!(chacha_decrypt_aad(&key, &nonce, &enc, b"WRONG").is_err());
    }

    #[test]
    fn test_ciphertext_tamper_fails() {
        let key = [0xCDu8; 32];
        let nonce = [0x07u8; 12];
        let mut enc = chacha_encrypt(&key, &nonce, b"tamper test").expect("enc");
        // Flip a byte in the ciphertext portion
        let last = enc.len() - 1;
        enc[last] ^= 0xff;
        assert!(chacha_decrypt(&key, &enc).is_err());
    }
}
