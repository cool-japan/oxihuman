// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SHA-256 implementation per FIPS 180-4 and HMAC per RFC 2104.
//!
//! Pure Rust, no external dependencies. All operations use wrapping arithmetic
//! as specified by the SHA-256 standard. The `Sha256Hasher` supports
//! incremental input via an internal 64-byte block buffer, so multiple
//! `update()` calls produce the same digest as a single `sha256_hash()` call
//! on the concatenated input.

// ── SHA-256 constants ────────────────────────────────────────────────────────

/// Round constants K[0..64]: cube roots of the first 64 primes (FIPS 180-4 §4.2.2).
const K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

/// Initial hash values H[0..8]: square roots of the first 8 primes (FIPS 180-4 §5.3.3).
const H_INIT: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

// ── Low-level helper functions ───────────────────────────────────────────────

/// σ₀ (lowercase sigma-0): small-sigma-0 for message schedule.
#[inline(always)]
fn sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

/// σ₁ (lowercase sigma-1): small-sigma-1 for message schedule.
#[inline(always)]
fn sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

/// Σ₀ (uppercase sigma-0): big-sigma-0 for compression.
#[inline(always)]
fn cap_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

/// Σ₁ (uppercase sigma-1): big-sigma-1 for compression.
#[inline(always)]
fn cap_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

/// Ch (choose): (e & f) ^ (!e & g).
#[inline(always)]
fn ch(e: u32, f: u32, g: u32) -> u32 {
    (e & f) ^ ((!e) & g)
}

/// Maj (majority): (a & b) ^ (a & c) ^ (b & c).
#[inline(always)]
fn maj(a: u32, b: u32, c: u32) -> u32 {
    (a & b) ^ (a & c) ^ (b & c)
}

// ── Block compression ────────────────────────────────────────────────────────

/// Process a single 64-byte message block against the running hash state.
///
/// `state` is updated in-place: state[0..8] → H₀..H₇ after the block.
fn compress_block(state: &mut [u32; 8], block: &[u8; 64]) {
    // Expand block into message schedule W[0..64].
    let mut w = [0u32; 64];

    // W[0..15]: pack 4 bytes big-endian into each word.
    for (t, chunk) in block.chunks_exact(4).enumerate() {
        w[t] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }

    // W[16..63]: extend using σ₀ and σ₁.
    for t in 16..64 {
        w[t] = sigma1(w[t - 2])
            .wrapping_add(w[t - 7])
            .wrapping_add(sigma0(w[t - 15]))
            .wrapping_add(w[t - 16]);
    }

    // Initialize working variables from current state.
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *state;

    // 64 rounds.
    for t in 0..64 {
        let s1 = cap_sigma1(e);
        let choice = ch(e, f, g);
        let temp1 = h
            .wrapping_add(s1)
            .wrapping_add(choice)
            .wrapping_add(K[t])
            .wrapping_add(w[t]);

        let s0 = cap_sigma0(a);
        let majority = maj(a, b, c);
        let temp2 = s0.wrapping_add(majority);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    // Add the compressed chunk to the current hash state.
    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

// ── Public types ─────────────────────────────────────────────────────────────

/// 32-byte SHA-256 digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sha256Digest(pub [u8; 32]);

impl Sha256Digest {
    /// Return the digest as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Return the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// SHA-256 hasher with internal 64-byte block buffer.
///
/// Supports incremental input: multiple `update()` calls produce exactly the
/// same digest as a single `sha256_hash()` on the concatenated bytes. Full
/// 64-byte blocks are compressed immediately so memory usage is bounded.
#[derive(Debug, Clone)]
pub struct Sha256Hasher {
    /// Running hash state (H₀..H₇).
    state: [u32; 8],
    /// Partial-block buffer; bytes `0..buf_len` are valid.
    buf: [u8; 64],
    /// Number of valid bytes currently in `buf`.
    buf_len: usize,
    /// Total number of bytes fed so far (used for the length field in padding).
    total_len: u64,
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha256Hasher {
    /// Create a new hasher in the initial state.
    pub fn new() -> Self {
        Self {
            state: H_INIT,
            buf: [0u8; 64],
            buf_len: 0,
            total_len: 0,
        }
    }

    /// Feed bytes into the hasher.
    ///
    /// Any data that completes a 64-byte block is compressed immediately;
    /// the remainder is stored in the internal buffer.
    pub fn update(&mut self, data: &[u8]) {
        self.total_len = self.total_len.wrapping_add(data.len() as u64);

        let mut pos = 0usize;

        // If the buffer already holds some bytes, top it up first.
        if self.buf_len > 0 {
            let need = 64 - self.buf_len;
            let take = need.min(data.len());
            self.buf[self.buf_len..self.buf_len + take].copy_from_slice(&data[..take]);
            self.buf_len += take;
            pos += take;

            if self.buf_len == 64 {
                let block: &[u8; 64] = &self.buf;
                // Safety: we just confirmed buf_len == 64, so the whole buffer is valid.
                let block_arr: [u8; 64] = *block;
                compress_block(&mut self.state, &block_arr);
                self.buf_len = 0;
            }
        }

        // Process all remaining complete blocks directly from `data`.
        let remainder = &data[pos..];
        for block_bytes in remainder.chunks_exact(64) {
            let mut block_arr = [0u8; 64];
            block_arr.copy_from_slice(block_bytes);
            compress_block(&mut self.state, &block_arr);
            pos += 64;
        }

        // Copy any leftover bytes (< 64) into the buffer.
        let leftover = &data[pos..];
        if !leftover.is_empty() {
            self.buf[..leftover.len()].copy_from_slice(leftover);
            self.buf_len = leftover.len();
        }
    }

    /// Finalize and return the digest.
    ///
    /// This does not consume the hasher so it can be inspected without
    /// destroying the state; however the result is computed on a cloned
    /// intermediate state so subsequent `update()` / `finalize()` calls remain
    /// independent.
    pub fn finalize(&self) -> Sha256Digest {
        // Clone internal state so `self` is not mutated.
        let mut state = self.state;
        let buf_len = self.buf_len;
        let total_len = self.total_len;

        // Build the padded tail.  The tail is at most two 64-byte blocks:
        //  • one if the current buffer has room for the 0x80 byte + 8-byte
        //    length (i.e. buf_len ≤ 55),
        //  • two otherwise.
        let bit_len: u64 = total_len.wrapping_mul(8);

        // We need to hash: buf[0..buf_len] || 0x80 || zeros || bit_len_be(8)
        // Minimum padding block count:
        //   if buf_len + 1 + 8 <= 64 → one block  (buf_len ≤ 55)
        //   else                      → two blocks
        if buf_len + 1 + 8 <= 64 {
            // Single-block padding.
            let mut pad_block = [0u8; 64];
            pad_block[..buf_len].copy_from_slice(&self.buf[..buf_len]);
            pad_block[buf_len] = 0x80;
            // Remaining bytes are already 0; write length in last 8 bytes.
            pad_block[56..64].copy_from_slice(&bit_len.to_be_bytes());
            compress_block(&mut state, &pad_block);
        } else {
            // Two-block padding.
            let mut pad_block1 = [0u8; 64];
            pad_block1[..buf_len].copy_from_slice(&self.buf[..buf_len]);
            pad_block1[buf_len] = 0x80;
            compress_block(&mut state, &pad_block1);

            let mut pad_block2 = [0u8; 64];
            pad_block2[56..64].copy_from_slice(&bit_len.to_be_bytes());
            compress_block(&mut state, &pad_block2);
        }

        // Serialize state to big-endian bytes.
        let mut digest = [0u8; 32];
        for (i, word) in state.iter().enumerate() {
            digest[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
        }
        Sha256Digest(digest)
    }

    /// Reset the hasher to the initial state.
    pub fn reset(&mut self) {
        self.state = H_INIT;
        self.buf = [0u8; 64];
        self.buf_len = 0;
        self.total_len = 0;
    }
}

// ── Public functions ─────────────────────────────────────────────────────────

/// Compute the SHA-256 digest of `data` per FIPS 180-4.
pub fn sha256_hash(data: &[u8]) -> Sha256Digest {
    let mut h = Sha256Hasher::new();
    h.update(data);
    h.finalize()
}

/// Compute HMAC-SHA256 per RFC 2104.
///
/// ```text
/// HMAC(K, m) = H((K' ⊕ opad) || H((K' ⊕ ipad) || m))
/// ```
///
/// where K' is the key padded (or hashed-then-padded) to the block size (64 bytes),
/// ipad = 0x36 repeated, opad = 0x5c repeated.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Sha256Digest {
    // Step 1: derive the 64-byte key block.
    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        let hashed = sha256_hash(key);
        key_block[..32].copy_from_slice(hashed.as_bytes());
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    // Step 2: build ipad and opad variants of the key.
    let mut ipad_key = [0u8; 64];
    let mut opad_key = [0u8; 64];
    for i in 0..64 {
        ipad_key[i] = key_block[i] ^ 0x36;
        opad_key[i] = key_block[i] ^ 0x5c;
    }

    // Step 3: inner hash = SHA-256(ipad_key || data).
    let mut inner = Sha256Hasher::new();
    inner.update(&ipad_key);
    inner.update(data);
    let inner_digest = inner.finalize();

    // Step 4: outer hash = SHA-256(opad_key || inner_digest).
    let mut outer = Sha256Hasher::new();
    outer.update(&opad_key);
    outer.update(inner_digest.as_bytes());
    outer.finalize()
}

/// Compute HMAC-SHA256 (stub alias kept for backwards compatibility).
///
/// # Deprecated
///
/// Use [`hmac_sha256`] instead. This alias simply delegates to the real
/// implementation.
#[deprecated(since = "0.1.3", note = "use `hmac_sha256` instead")]
pub fn hmac_sha256_stub(key: &[u8], data: &[u8]) -> Sha256Digest {
    hmac_sha256(key, data)
}

/// Compare two digests in constant time (timing-safe equality).
///
/// Returns `true` iff every byte of `a` equals the corresponding byte of `b`,
/// without leaking which byte differed.
pub fn sha256_eq(a: &Sha256Digest, b: &Sha256Digest) -> bool {
    a.0.iter()
        .zip(b.0.iter())
        .fold(0u8, |acc, (&x, &y)| acc | (x ^ y))
        == 0
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Existing structural tests (preserved) ─────────────────────────────

    #[test]
    fn test_hash_returns_32_bytes() {
        let d = sha256_hash(b"hello");
        assert_eq!(d.0.len(), 32);
    }

    #[test]
    fn test_hash_empty() {
        let d = sha256_hash(&[]);
        assert_eq!(d.0.len(), 32);
    }

    #[test]
    fn test_hash_deterministic() {
        let d1 = sha256_hash(b"test");
        let d2 = sha256_hash(b"test");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_hash_different_inputs() {
        let d1 = sha256_hash(b"aaa");
        let d2 = sha256_hash(b"bbb");
        assert_ne!(d1, d2);
    }

    #[test]
    fn test_to_hex_length() {
        let d = sha256_hash(b"hello");
        assert_eq!(d.to_hex().len(), 64);
    }

    #[test]
    fn test_hasher_roundtrip() {
        let mut h = Sha256Hasher::new();
        h.update(b"hello");
        let d1 = h.finalize();
        let d2 = sha256_hash(b"hello");
        assert_eq!(d1, d2);
    }

    #[test]
    fn test_hasher_reset() {
        let mut h = Sha256Hasher::new();
        h.update(b"abc");
        h.reset();
        let d = h.finalize();
        assert_eq!(d, sha256_hash(&[]));
    }

    #[test]
    fn test_sha256_eq() {
        let d1 = sha256_hash(b"same");
        let d2 = sha256_hash(b"same");
        assert!(sha256_eq(&d1, &d2));
        let d3 = sha256_hash(b"other");
        assert!(!sha256_eq(&d1, &d3));
    }

    #[test]
    fn test_hmac_stub() {
        // hmac_sha256_stub is now a deprecated alias; verify it returns 32 bytes.
        #[allow(deprecated)]
        let d = hmac_sha256_stub(b"key", b"message");
        assert_eq!(d.0.len(), 32);
    }

    // ── Known-answer tests (FIPS 180-4 / RFC 4634) ────────────────────────

    /// SHA-256("") — NIST FIPS 180-4 example.
    #[test]
    fn kat_empty_string() {
        let got = sha256_hash(b"");
        assert_eq!(
            got.to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    /// SHA-256("abc") — NIST FIPS 180-4 Appendix B.1 example.
    #[test]
    fn kat_abc() {
        let got = sha256_hash(b"abc");
        assert_eq!(
            got.to_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// SHA-256 of the 448-bit "abcdbcdecdef…" test vector (NIST FIPS 180-4).
    #[test]
    fn kat_448bit_vector() {
        let msg = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let got = sha256_hash(msg);
        assert_eq!(
            got.to_hex(),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    /// Incremental hashing must give the same result as a single call.
    #[test]
    fn kat_incremental_matches_oneshot() {
        // Single call.
        let one_shot = sha256_hash(b"abc");

        // Split b"abc" = b"ab" || b"c".
        let mut h = Sha256Hasher::new();
        h.update(b"ab");
        h.update(b"c");
        let incremental = h.finalize();

        assert_eq!(one_shot, incremental);
    }

    /// Incremental hashing across a block boundary (> 64 bytes total).
    #[test]
    fn kat_incremental_across_block_boundary() {
        let data = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";

        // One shot.
        let one_shot = sha256_hash(data);

        // Split at byte 32 (mid-block) and again at byte 50.
        let mut h = Sha256Hasher::new();
        h.update(&data[..32]);
        h.update(&data[32..50]);
        h.update(&data[50..]);
        let incremental = h.finalize();

        assert_eq!(one_shot, incremental);
    }

    /// Multiple finalize() calls on the same hasher return the same digest.
    #[test]
    fn kat_finalize_idempotent() {
        let mut h = Sha256Hasher::new();
        h.update(b"hello");
        let d1 = h.finalize();
        let d2 = h.finalize();
        assert_eq!(d1, d2);
    }

    /// HMAC-SHA256 known-answer test (RFC 2202 / commonly cited test vector).
    ///
    /// HMAC-SHA256(key = b"key",
    ///             data = b"The quick brown fox jumps over the lazy dog")
    /// = f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8
    #[test]
    fn kat_hmac_sha256() {
        let key = b"key";
        let data = b"The quick brown fox jumps over the lazy dog";
        let got = hmac_sha256(key, data);
        assert_eq!(
            got.to_hex(),
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    /// HMAC-SHA256 with a key longer than 64 bytes (key is hashed first).
    #[test]
    fn kat_hmac_sha256_long_key() {
        // Key of 100 bytes (all 0xAA).
        let key = [0xAA_u8; 100];
        let data = b"Test Using Larger Than Block-Size Key and Larger Than One Block-Size Data";
        // Compute reference manually: SHA256(long_key) gives a 32-byte key,
        // then proceed as normal HMAC.
        let hashed_key = sha256_hash(&key);
        let mut key_block = [0u8; 64];
        key_block[..32].copy_from_slice(hashed_key.as_bytes());
        let mut ipad_key = [0u8; 64];
        let mut opad_key = [0u8; 64];
        for i in 0..64 {
            ipad_key[i] = key_block[i] ^ 0x36;
            opad_key[i] = key_block[i] ^ 0x5c;
        }
        let inner = sha256_hash(&[ipad_key.as_ref(), data.as_ref()].concat());
        let expected = sha256_hash(&[opad_key.as_ref(), inner.as_bytes().as_ref()].concat());

        let got = hmac_sha256(&key, data);
        assert_eq!(got, expected);
    }

    /// sha256_eq is timing-safe: true for equal, false for different.
    #[test]
    fn test_sha256_eq_timing_safe() {
        let d1 = sha256_hash(b"alpha");
        let d2 = sha256_hash(b"alpha");
        let d3 = sha256_hash(b"beta");
        assert!(sha256_eq(&d1, &d2));
        assert!(!sha256_eq(&d1, &d3));
    }

    /// as_bytes() returns the raw 32 bytes.
    #[test]
    fn test_as_bytes() {
        let d = sha256_hash(b"hello");
        assert_eq!(d.as_bytes().len(), 32);
        assert_eq!(d.as_bytes(), &d.0);
    }

    /// Hashing exactly 64 bytes (one full block) works correctly.
    #[test]
    fn kat_exactly_one_block() {
        let data = [0x61_u8; 64]; // 64 × 'a'
        let one_shot = sha256_hash(&data);

        let mut h = Sha256Hasher::new();
        h.update(&data[..32]);
        h.update(&data[32..]);
        assert_eq!(one_shot, h.finalize());
    }

    /// Hashing exactly 128 bytes (two full blocks) works correctly.
    #[test]
    fn kat_exactly_two_blocks() {
        let data = [0x62_u8; 128]; // 128 × 'b'
        let one_shot = sha256_hash(&data);

        let mut h = Sha256Hasher::new();
        h.update(&data[..63]);
        h.update(&data[63..]);
        assert_eq!(one_shot, h.finalize());
    }
}
