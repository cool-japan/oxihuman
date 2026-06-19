// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Pure-Rust BLAKE3 implementation per the BLAKE3 specification.
//!
//! References:
//!  - <https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf>
//!  - BLAKE3 is built on a Merkle tree of 1024-byte chunks compressed using a
//!    7-round BLAKE2s-derived compression function with domain-separation flags.

// ─── Constants ────────────────────────────────────────────────────────────────

/// Initial value words (same as SHA-256: first 32 bits of √prime₁..√prime₈).
const IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

/// Maximum bytes consumed per compression block.
const BLOCK_LEN: u32 = 64;
/// Maximum bytes per chunk (16 blocks of 64 bytes each).
#[allow(dead_code)]
const CHUNK_LEN: usize = 1024;

// Domain-separation flags.
const CHUNK_START: u32 = 1;
const CHUNK_END: u32 = 2;
const PARENT: u32 = 4;
const ROOT: u32 = 8;
const KEYED_HASH: u32 = 16;

/// Message schedule permutations for each of the 7 rounds.
const MSG_SCHEDULE: [[usize; 16]; 7] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8],
    [3, 4, 10, 12, 13, 2, 7, 14, 6, 5, 9, 0, 11, 15, 8, 1],
    [10, 7, 12, 9, 14, 3, 13, 15, 4, 0, 11, 2, 5, 8, 1, 6],
    [12, 13, 9, 11, 15, 10, 14, 8, 7, 2, 5, 3, 0, 1, 6, 4],
    [9, 14, 11, 5, 8, 12, 15, 1, 13, 3, 0, 10, 2, 6, 4, 7],
    [11, 15, 5, 0, 1, 9, 8, 6, 14, 10, 2, 12, 3, 4, 7, 13],
];

// ─── Public API types ─────────────────────────────────────────────────────────

/// 32-byte BLAKE3 digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blake3Digest(pub [u8; 32]);

impl Blake3Digest {
    /// Return the digest as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Return a reference to the raw 32-byte array.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Incremental BLAKE3 hasher.
///
/// Supports both unkeyed and keyed modes.  Multiple calls to [`update`] are
/// equivalent to a single call with the concatenated data.
///
/// [`update`]: Blake3Hasher::update
#[derive(Debug, Clone)]
pub struct Blake3Hasher {
    /// Initialisation chaining-value (IV or derived from key).
    init_cv: [u32; 8],
    /// Domain flags added to every compression call (0 or KEYED_HASH).
    base_flags: u32,

    // --- current chunk accumulation ---
    /// CV at the start of the current chunk (reset per chunk).
    chunk_cv: [u32; 8],
    /// Index of the current chunk.
    chunk_counter: u64,
    /// Bytes buffered within the current 64-byte block.
    block_buf: [u8; 64],
    /// How many bytes of `block_buf` are valid.
    block_len: usize,
    /// How many *full* blocks have already been compressed in the current chunk.
    blocks_compressed: usize,

    /// Stack of completed chaining values for the Merkle tree.
    cv_stack: Vec<[u32; 8]>,
}

impl Default for Blake3Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Blake3Hasher {
    /// Create a new unkeyed hasher.
    pub fn new() -> Self {
        Self::init(IV, 0)
    }

    /// Create a keyed hasher.  `key` must be exactly 32 bytes.
    pub fn new_keyed(key: [u8; 32]) -> Self {
        let key_words = bytes_to_words_32(&key);
        Self::init(key_words, KEYED_HASH)
    }

    fn init(init_cv: [u32; 8], base_flags: u32) -> Self {
        Self {
            init_cv,
            base_flags,
            chunk_cv: init_cv,
            chunk_counter: 0,
            block_buf: [0u8; 64],
            block_len: 0,
            blocks_compressed: 0,
            cv_stack: Vec::new(),
        }
    }

    /// Feed more data into the hasher.
    ///
    /// # Invariant
    ///
    /// We never eagerly compress the last (potentially final) block of a chunk
    /// because we cannot emit CHUNK_END until we know more data follows.
    /// Specifically, blocks 0-14 of a chunk are compressed eagerly when full
    /// and more data is available; block 15 sits in the buffer until either
    /// `finalize` is called or data for the next chunk arrives.
    pub fn update(&mut self, data: &[u8]) {
        let mut remaining = data;
        while !remaining.is_empty() {
            // ------------------------------------------------------------------
            // If the current block buffer is full AND we are not on the last
            // block position of the chunk, compress it eagerly (non-terminal).
            // We leave block 15 (blocks_compressed == 15, block_len == 64) in
            // the buffer so that CHUNK_END can be applied correctly in finalize
            // or when the next chunk's data arrives below.
            // ------------------------------------------------------------------
            if self.block_len == 64 && self.blocks_compressed < 15 {
                self.compress_mid_block();
            }

            // ------------------------------------------------------------------
            // If block 15 is full and more data is available, it is definitely
            // a non-terminal block within a full chunk.  Compress it with
            // CHUNK_END and start a fresh chunk.
            // ------------------------------------------------------------------
            if self.block_len == 64 && self.blocks_compressed == 15 {
                // This is the 16th (last) block of the current chunk and there
                // is still more data to process — emit CHUNK_END.
                self.compress_chunk_final_block();
                self.rotate_chunk();
                continue; // restart the loop to consume `remaining`
            }

            // Fill the current block buffer up to 64 bytes or chunk boundary.
            let take = remaining.len().min(64 - self.block_len);
            self.block_buf[self.block_len..self.block_len + take]
                .copy_from_slice(&remaining[..take]);
            self.block_len += take;
            remaining = &remaining[take..];
        }
    }

    /// Compress blocks 0-14 of the current chunk (non-terminal, no CHUNK_END).
    fn compress_mid_block(&mut self) {
        debug_assert_eq!(self.block_len, 64);
        debug_assert!(self.blocks_compressed < 15);

        let mut flags = self.base_flags;
        if self.blocks_compressed == 0 {
            flags |= CHUNK_START;
        }

        let block_words = bytes_to_words_64(&self.block_buf);
        self.chunk_cv = compress_cv(
            &self.chunk_cv,
            &block_words,
            self.chunk_counter,
            BLOCK_LEN,
            flags,
        );
        self.blocks_compressed += 1;
        self.block_len = 0;
        self.block_buf = [0u8; 64];
    }

    /// Compress block 15 of the current chunk with CHUNK_END.
    /// This finalises the chunk's CV (stored back into `self.chunk_cv`).
    fn compress_chunk_final_block(&mut self) {
        debug_assert_eq!(self.block_len, 64);
        debug_assert_eq!(self.blocks_compressed, 15);

        let mut flags = self.base_flags | CHUNK_END;
        if self.blocks_compressed == 0 {
            // Defensive: single-block chunk would also need CHUNK_START.
            flags |= CHUNK_START;
        }

        let block_words = bytes_to_words_64(&self.block_buf);
        self.chunk_cv = compress_cv(
            &self.chunk_cv,
            &block_words,
            self.chunk_counter,
            BLOCK_LEN,
            flags,
        );
        self.blocks_compressed += 1; // now 16 — chunk is done
        self.block_len = 0;
        self.block_buf = [0u8; 64];
    }

    /// Push the just-completed chunk's CV onto the Merkle stack and reset
    /// chunk state for the next chunk.
    fn rotate_chunk(&mut self) {
        debug_assert_eq!(self.blocks_compressed, 16);

        let chunk_cv = self.chunk_cv;
        self.chunk_counter += 1;
        self.chunk_cv = self.init_cv;
        self.blocks_compressed = 0;
        self.block_len = 0;
        self.block_buf = [0u8; 64];

        // Merge subtrees: while the new chunk_counter has a trailing zero, the
        // top two entries on the stack represent equal-depth subtrees that can
        // be combined into a parent node.
        let mut cv = chunk_cv;
        let mut counter = self.chunk_counter;
        while counter & 1 == 0 {
            let left = self.cv_stack.pop().expect("cv_stack invariant violated");
            cv = parent_cv(&left, &cv, self.init_cv, self.base_flags);
            counter >>= 1;
        }
        self.cv_stack.push(cv);
    }

    /// Reset the hasher to its initial state (same mode/key).
    pub fn reset(&mut self) {
        let init_cv = self.init_cv;
        let base_flags = self.base_flags;
        *self = Self::init(init_cv, base_flags);
    }

    /// Consume the hasher and return the final digest.
    pub fn finalize(&self) -> Blake3Digest {
        // We work on a clone so `finalize` takes `&self` (matching the stub).
        let mut s = self.clone();
        s.finalize_inner()
    }

    fn finalize_inner(&mut self) -> Blake3Digest {
        // Compress the last (possibly partial) block of the current chunk with
        // CHUNK_END — this is always the terminal block.
        let mut last_block = [0u8; 64];
        last_block[..self.block_len].copy_from_slice(&self.block_buf[..self.block_len]);

        let last_block_len = self.block_len as u32;

        let mut flags = self.base_flags | CHUNK_END;
        if self.blocks_compressed == 0 {
            flags |= CHUNK_START;
        }

        let block_words = bytes_to_words_64(&last_block);

        // Is this the ONLY chunk (and therefore the root)?
        let is_root = self.cv_stack.is_empty();

        if is_root {
            // Single chunk: produce root output directly.
            flags |= ROOT;
            let state = compress_block(
                &self.chunk_cv,
                &block_words,
                self.chunk_counter,
                last_block_len,
                flags,
            );
            return words_to_digest(&state);
        }

        // Multi-chunk: compute this chunk's CV then fold the stack.
        let mut cv = compress_cv(
            &self.chunk_cv,
            &block_words,
            self.chunk_counter,
            last_block_len,
            flags,
        );

        // Drain the stack from top to bottom — each entry is a left sibling.
        while self.cv_stack.len() > 1 {
            let right = cv;
            let left = self.cv_stack.pop().expect("stack non-empty");
            cv = parent_cv(&left, &right, self.init_cv, self.base_flags);
        }

        // The last remaining entry is the leftmost subtree; combine with ROOT.
        let left = self.cv_stack.pop().expect("root entry on stack");
        let root_cv = parent_cv_root(&left, &cv, self.init_cv, self.base_flags);
        Blake3Digest(root_cv)
    }
}

// ─── Public free functions ────────────────────────────────────────────────────

/// Compute the BLAKE3 hash of `data`.
pub fn blake3_hash(data: &[u8]) -> Blake3Digest {
    let mut h = Blake3Hasher::new();
    h.update(data);
    h.finalize()
}

/// Compute a keyed BLAKE3 hash of `data`.
pub fn blake3_keyed_hash(key: &[u8; 32], data: &[u8]) -> Blake3Digest {
    let mut h = Blake3Hasher::new_keyed(*key);
    h.update(data);
    h.finalize()
}

/// Return the output length of BLAKE3 in bytes (default output: 32).
pub fn blake3_output_len() -> usize {
    32
}

/// Verify that hashing the same data twice produces the same digest.
pub fn blake3_stable(data: &[u8]) -> bool {
    blake3_hash(data) == blake3_hash(data)
}

// ─── Core compression primitives ─────────────────────────────────────────────

/// G mixing function as specified in the BLAKE3 paper.
#[inline(always)]
fn g(v: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, x: u32, y: u32) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(12);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(8);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(7);
}

/// Full 16-word compression function output (used for root output extraction).
fn compress_block(
    cv: &[u32; 8],
    block: &[u32; 16],
    counter: u64,
    block_len: u32,
    flags: u32,
) -> [u32; 16] {
    let mut v = [0u32; 16];
    v[0..8].copy_from_slice(cv);
    v[8] = IV[0];
    v[9] = IV[1];
    v[10] = IV[2];
    v[11] = IV[3];
    v[12] = counter as u32;
    v[13] = (counter >> 32) as u32;
    v[14] = block_len;
    v[15] = flags;

    for s in &MSG_SCHEDULE {
        // column step
        g(&mut v, 0, 4, 8, 12, block[s[0]], block[s[1]]);
        g(&mut v, 1, 5, 9, 13, block[s[2]], block[s[3]]);
        g(&mut v, 2, 6, 10, 14, block[s[4]], block[s[5]]);
        g(&mut v, 3, 7, 11, 15, block[s[6]], block[s[7]]);
        // diagonal step
        g(&mut v, 0, 5, 10, 15, block[s[8]], block[s[9]]);
        g(&mut v, 1, 6, 11, 12, block[s[10]], block[s[11]]);
        g(&mut v, 2, 7, 8, 13, block[s[12]], block[s[13]]);
        g(&mut v, 3, 4, 9, 14, block[s[14]], block[s[15]]);
    }

    // XOR lower half with upper half to produce the new chaining-value words.
    for i in 0..8 {
        v[i] ^= v[i + 8];
        v[i + 8] ^= cv[i]; // second half: v[i+8] ^ cv[i] gives the keyed output words
    }
    v
}

/// Compression function returning only the 8-word chaining value.
#[inline]
fn compress_cv(
    cv: &[u32; 8],
    block: &[u32; 16],
    counter: u64,
    block_len: u32,
    flags: u32,
) -> [u32; 8] {
    let full = compress_block(cv, block, counter, block_len, flags);
    let mut out = [0u32; 8];
    out.copy_from_slice(&full[0..8]);
    out
}

/// Compute the parent chaining value of two child CVs (non-root).
fn parent_cv(left: &[u32; 8], right: &[u32; 8], key: [u32; 8], base_flags: u32) -> [u32; 8] {
    let block = cv_pair_to_block(left, right);
    compress_cv(&key, &block, 0, BLOCK_LEN, base_flags | PARENT)
}

/// Compute the root 32-byte digest from two child CVs.
fn parent_cv_root(left: &[u32; 8], right: &[u32; 8], key: [u32; 8], base_flags: u32) -> [u8; 32] {
    let block = cv_pair_to_block(left, right);
    let state = compress_block(&key, &block, 0, BLOCK_LEN, base_flags | PARENT | ROOT);
    words_to_32_bytes(&state)
}

// ─── Helper functions ─────────────────────────────────────────────────────────

/// Interpret a 32-byte key as 8 little-endian u32 words.
fn bytes_to_words_32(bytes: &[u8; 32]) -> [u32; 8] {
    let mut words = [0u32; 8];
    for (i, w) in words.iter_mut().enumerate() {
        *w = u32::from_le_bytes(bytes[i * 4..(i + 1) * 4].try_into().unwrap_or([0; 4]));
    }
    words
}

/// Interpret a 64-byte block as 16 little-endian u32 words.
fn bytes_to_words_64(bytes: &[u8; 64]) -> [u32; 16] {
    let mut words = [0u32; 16];
    for (i, w) in words.iter_mut().enumerate() {
        *w = u32::from_le_bytes(bytes[i * 4..(i + 1) * 4].try_into().unwrap_or([0; 4]));
    }
    words
}

/// Pack two 8-word chaining values into a 16-word block for parent compression.
fn cv_pair_to_block(left: &[u32; 8], right: &[u32; 8]) -> [u32; 16] {
    let mut block = [0u32; 16];
    block[0..8].copy_from_slice(left);
    block[8..16].copy_from_slice(right);
    block
}

/// Extract the first 32 bytes of the 16-word root state (little-endian).
fn words_to_32_bytes(state: &[u32; 16]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, &w) in state[0..8].iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&w.to_le_bytes());
    }
    out
}

/// Wrap a 16-word root state into a [`Blake3Digest`].
fn words_to_digest(state: &[u32; 16]) -> Blake3Digest {
    Blake3Digest(words_to_32_bytes(state))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Known-answer test ────────────────────────────────────────────────────

    /// Official BLAKE3 test vector: BLAKE3(b"") must equal this exact hex string.
    #[test]
    fn test_kat_empty_input() {
        let expected = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
        let got = blake3_hash(b"");
        assert_eq!(got.to_hex(), expected, "KAT failure: BLAKE3(\"\") mismatch");
    }

    // ── Behavioural tests (preserved from stub) ──────────────────────────────

    #[test]
    fn test_hash_len() {
        let d = blake3_hash(b"test");
        assert_eq!(d.0.len(), 32);
    }

    #[test]
    fn test_hash_empty() {
        let d = blake3_hash(&[]);
        assert_eq!(d.0.len(), 32);
    }

    #[test]
    fn test_hash_deterministic() {
        assert_eq!(blake3_hash(b"abc"), blake3_hash(b"abc"));
    }

    #[test]
    fn test_hash_distinct_inputs() {
        assert_ne!(blake3_hash(b"x"), blake3_hash(b"y"));
    }

    #[test]
    fn test_hex_length() {
        assert_eq!(blake3_hash(b"hi").to_hex().len(), 64);
    }

    #[test]
    fn test_keyed_differs_from_plain() {
        let key = [0xFFu8; 32];
        assert_ne!(blake3_hash(b"msg"), blake3_keyed_hash(&key, b"msg"));
    }

    #[test]
    fn test_hasher_incremental() {
        let mut h = Blake3Hasher::new();
        h.update(b"hello");
        assert_eq!(h.finalize(), blake3_hash(b"hello"));
    }

    #[test]
    fn test_hasher_incremental_split() {
        // Splitting the input at any boundary must give the same result.
        let data = b"hello, world! this is a test of incremental hashing.";
        let expected = blake3_hash(data);

        for split in 0..=data.len() {
            let mut h = Blake3Hasher::new();
            h.update(&data[..split]);
            h.update(&data[split..]);
            assert_eq!(h.finalize(), expected, "split at byte {} failed", split);
        }
    }

    #[test]
    fn test_hasher_incremental_large() {
        // Exercise the multi-chunk path (> 1024 bytes).
        let data: Vec<u8> = (0u16..4096).map(|i| (i & 0xFF) as u8).collect();
        let all_at_once = blake3_hash(&data);

        let mut h = Blake3Hasher::new();
        for chunk in data.chunks(97) {
            h.update(chunk);
        }
        assert_eq!(h.finalize(), all_at_once, "large incremental mismatch");
    }

    #[test]
    fn test_hasher_reset() {
        let mut h = Blake3Hasher::new();
        h.update(b"something");
        h.reset();
        assert_eq!(h.finalize(), blake3_hash(&[]));
    }

    #[test]
    fn test_output_len() {
        assert_eq!(blake3_output_len(), 32);
    }

    #[test]
    fn test_stable() {
        assert!(blake3_stable(b"stability check"));
    }

    #[test]
    fn test_multi_chunk_boundary() {
        // Exactly one chunk (1024 bytes) and one chunk + 1 byte.
        let exactly_one_chunk = vec![0x42u8; 1024];
        let just_over = vec![0x42u8; 1025];
        let d1 = blake3_hash(&exactly_one_chunk);
        let d2 = blake3_hash(&just_over);
        assert_ne!(d1, d2, "1024 vs 1025 bytes must differ");

        // Deterministic at 1025 bytes via incremental path.
        let mut h = Blake3Hasher::new();
        for b in &just_over {
            h.update(std::slice::from_ref(b));
        }
        assert_eq!(h.finalize(), d2, "byte-by-byte 1025 mismatch");
    }
}
