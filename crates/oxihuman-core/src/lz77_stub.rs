// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Real LZ77 compression with sliding window, hash chains, and lazy matching.

/// Default sliding window size (32 KB).
const DEFAULT_WINDOW_SIZE: usize = 32768;
/// Maximum match length (DEFLATE-compatible).
const DEFAULT_MAX_MATCH: usize = 258;
/// Minimum match length.
const MIN_MATCH_LEN: usize = 3;
/// Hash table size (must be power of two).
const HASH_TABLE_SIZE: usize = 1 << 15;
/// Hash mask for 3-byte sequences.
const HASH_MASK: u32 = HASH_TABLE_SIZE as u32 - 1;
/// Maximum chain length to search before giving up.
const MAX_CHAIN_LEN: usize = 256;
/// Hash shift for rolling hash computation.
const HASH_SHIFT: u32 = 5;

#[derive(Debug, Clone)]
pub struct Lz77Config {
    pub window_size: usize,
    pub min_match: usize,
}

#[derive(Debug, Clone)]
pub struct Lz77Token {
    pub is_literal: bool,
    pub literal: u8,
    pub offset: u16,
    pub length: u16,
}

#[derive(Debug, Clone)]
pub struct Lz77Result {
    pub tokens: Vec<Lz77Token>,
    pub compressed_size: usize,
    pub ratio: f32,
}

/// An individual token in the compressed stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// A literal byte that could not be compressed.
    Literal(u8),
    /// A back-reference: copy `length` bytes from `distance` bytes back.
    Match { distance: u16, length: u16 },
}

pub fn default_lz77_config() -> Lz77Config {
    Lz77Config {
        window_size: DEFAULT_WINDOW_SIZE,
        min_match: MIN_MATCH_LEN,
    }
}

// ---------------------------------------------------------------------------
// Hash-chain helpers
// ---------------------------------------------------------------------------

/// Compute a hash value for a 3-byte sequence starting at `data[pos]`.
/// Returns `None` if fewer than 3 bytes remain.
fn hash3(data: &[u8], pos: usize) -> Option<u32> {
    if pos + 2 >= data.len() {
        return None;
    }
    let h = ((u32::from(data[pos]) << HASH_SHIFT) ^ u32::from(data[pos + 1])) << HASH_SHIFT
        ^ u32::from(data[pos + 2]);
    Some(h & HASH_MASK)
}

/// Internal structure for hash-chain based matching.
struct HashChain {
    /// head[hash] = most recent position with that hash (or sentinel).
    head: Vec<u32>,
    /// prev[pos % window] = previous position with same hash.
    prev: Vec<u32>,
    window_size: usize,
}

const NIL: u32 = u32::MAX;

impl HashChain {
    fn new(window_size: usize) -> Self {
        Self {
            head: vec![NIL; HASH_TABLE_SIZE],
            prev: vec![NIL; window_size],
            window_size,
        }
    }

    /// Insert position `pos` into the chain for the given hash value.
    fn insert(&mut self, hash: u32, pos: usize) {
        let idx = pos % self.window_size;
        let h = hash as usize;
        self.prev[idx] = self.head[h];
        self.head[h] = pos as u32;
    }

    /// Find the longest match for `data[pos..]` in the sliding window.
    /// Returns `(distance, length)` or `(0, 0)` if no match >= min_match.
    fn find_longest_match(
        &self,
        data: &[u8],
        pos: usize,
        hash: u32,
        window_size: usize,
        max_match: usize,
    ) -> (u16, u16) {
        let data_len = data.len();
        let max_len = max_match.min(data_len.saturating_sub(pos));
        if max_len < MIN_MATCH_LEN {
            return (0, 0);
        }

        let min_pos = pos.saturating_sub(window_size);
        let mut best_len: usize = MIN_MATCH_LEN - 1;
        let mut best_dist: usize = 0;

        let mut chain_pos = self.head[hash as usize];
        let mut chain_count = 0;

        while chain_pos != NIL && chain_count < MAX_CHAIN_LEN {
            let candidate = chain_pos as usize;
            if candidate < min_pos {
                break;
            }
            if candidate >= pos {
                // Walk the chain further; this entry is at or after our position.
                chain_pos = self.prev[candidate % self.window_size];
                chain_count += 1;
                continue;
            }

            let dist = pos - candidate;

            // Quick reject: check byte at best_len position first.
            if best_len < max_len
                && data.get(candidate + best_len).copied() == data.get(pos + best_len).copied()
            {
                // Full comparison
                let mut len = 0;
                while len < max_len
                    && data.get(candidate + len).copied() == data.get(pos + len).copied()
                {
                    len += 1;
                }

                if len > best_len {
                    best_len = len;
                    best_dist = dist;
                    if best_len == max_len {
                        break;
                    }
                }
            }

            chain_pos = self.prev[candidate % self.window_size];
            chain_count += 1;
        }

        if best_len >= MIN_MATCH_LEN && best_dist > 0 {
            (best_dist as u16, best_len as u16)
        } else {
            (0, 0)
        }
    }
}

// ---------------------------------------------------------------------------
// Token-based API (new, clean enum)
// ---------------------------------------------------------------------------

/// Compress `data` into a sequence of `Token`s using LZ77 with hash chains
/// and lazy matching.
#[allow(dead_code)]
pub fn compress(data: &[u8]) -> Vec<Token> {
    compress_with_params(data, DEFAULT_WINDOW_SIZE, DEFAULT_MAX_MATCH)
}

/// Compress with explicit window size and max match length.
#[allow(dead_code)]
pub fn compress_with_params(data: &[u8], window_size: usize, max_match: usize) -> Vec<Token> {
    let n = data.len();
    if n == 0 {
        return Vec::new();
    }

    let window_size = window_size.min(DEFAULT_WINDOW_SIZE);
    let max_match = max_match.min(DEFAULT_MAX_MATCH);

    let mut tokens = Vec::with_capacity(n / 2);
    let mut chain = HashChain::new(window_size);
    let mut pos: usize = 0;

    // Pending match from lazy evaluation
    let mut pending: Option<(u16, u16, usize)> = None; // (dist, len, position)

    while pos < n {
        let hash = hash3(data, pos);

        let (dist, len) = if let Some(h) = hash {
            chain.find_longest_match(data, pos, h, window_size, max_match)
        } else {
            (0, 0)
        };

        if let Some((p_dist, p_len, p_pos)) = pending.take() {
            // Lazy matching: we had a pending match from previous position.
            // If current match is strictly longer, emit the previous position
            // as a literal and keep the current match as the new candidate.
            if len > p_len + 1 {
                // Emit the pending position as a literal
                tokens.push(Token::Literal(data[p_pos]));
                // Current match becomes new pending
                pending = Some((dist, len, pos));
                if let Some(h) = hash {
                    chain.insert(h, pos);
                }
                pos += 1;
                continue;
            }
            // Otherwise emit the pending match (it was good enough)
            tokens.push(Token::Match {
                distance: p_dist,
                length: p_len,
            });
            // Insert hashes for all positions covered by the pending match
            // (p_pos was already inserted; insert p_pos+1 .. p_pos+p_len-1)
            let match_end = (p_pos + p_len as usize).min(n);
            for i in (p_pos + 1)..match_end {
                if let Some(h) = hash3(data, i) {
                    chain.insert(h, i);
                }
            }
            // Advance pos past the pending match
            pos = match_end;
            continue;
        }

        if len >= MIN_MATCH_LEN as u16 {
            // We have a match candidate. Use lazy matching: defer emission and
            // check if position+1 gives a better match.
            pending = Some((dist, len, pos));
            if let Some(h) = hash {
                chain.insert(h, pos);
            }
            pos += 1;
        } else {
            // No match; emit literal
            tokens.push(Token::Literal(data[pos]));
            if let Some(h) = hash {
                chain.insert(h, pos);
            }
            pos += 1;
        }
    }

    // Flush any remaining pending match
    if let Some((p_dist, p_len, p_pos)) = pending {
        let actual_len = p_len.min((n - p_pos) as u16);
        if actual_len >= MIN_MATCH_LEN as u16 {
            tokens.push(Token::Match {
                distance: p_dist,
                length: actual_len,
            });
        } else {
            for item in data[p_pos..(p_pos + actual_len as usize).min(n)].iter() {
                tokens.push(Token::Literal(*item));
            }
        }
    }

    tokens
}

/// Decompress a sequence of `Token`s back to the original data.
#[allow(dead_code)]
pub fn decompress(tokens: &[Token]) -> Vec<u8> {
    let mut out = Vec::new();
    for tok in tokens {
        match tok {
            Token::Literal(b) => out.push(*b),
            Token::Match { distance, length } => {
                let dist = *distance as usize;
                let len = *length as usize;
                if dist == 0 || dist > out.len() {
                    // Invalid back-reference; skip gracefully.
                    continue;
                }
                let start = out.len() - dist;
                for i in 0..len {
                    // Must index one at a time because the match can overlap
                    // (run-length style, e.g. distance=1, length=100).
                    let b = out[start + (i % dist)];
                    out.push(b);
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Legacy public API (preserved for backward compatibility)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub fn lz77_compress(data: &[u8], config: &Lz77Config) -> Lz77Result {
    let window_size = config.window_size.min(DEFAULT_WINDOW_SIZE);
    let new_tokens = compress_with_params(data, window_size, DEFAULT_MAX_MATCH);

    // Convert Token -> Lz77Token
    let tokens: Vec<Lz77Token> = new_tokens
        .iter()
        .map(|t| match t {
            Token::Literal(b) => Lz77Token {
                is_literal: true,
                literal: *b,
                offset: 0,
                length: 0,
            },
            Token::Match { distance, length } => Lz77Token {
                is_literal: false,
                literal: 0,
                offset: *distance,
                length: *length,
            },
        })
        .collect();

    let compressed_size = tokens.len();
    let original_size = data.len();
    let ratio = if original_size == 0 {
        1.0
    } else {
        compressed_size as f32 / original_size as f32
    };

    Lz77Result {
        tokens,
        compressed_size,
        ratio,
    }
}

#[allow(dead_code)]
pub fn lz77_decompress(tokens: &[Lz77Token]) -> Vec<u8> {
    let mut out = Vec::with_capacity(tokens.len());
    for tok in tokens {
        if tok.is_literal {
            out.push(tok.literal);
        } else {
            let dist = tok.offset as usize;
            let len = tok.length as usize;
            if dist == 0 || dist > out.len() {
                continue;
            }
            let start = out.len() - dist;
            for i in 0..len {
                let b = out[start + (i % dist)];
                out.push(b);
            }
        }
    }
    out
}

#[allow(dead_code)]
pub fn lz77_compression_ratio(result: &Lz77Result) -> f32 {
    result.ratio
}

#[allow(dead_code)]
pub fn lz77_token_count(result: &Lz77Result) -> usize {
    result.tokens.len()
}

#[allow(dead_code)]
pub fn lz77_is_literal(token: &Lz77Token) -> bool {
    token.is_literal
}

#[allow(dead_code)]
pub fn lz77_to_json(result: &Lz77Result) -> String {
    format!(
        "{{\"token_count\":{},\"compressed_size\":{},\"ratio\":{}}}",
        result.tokens.len(),
        result.compressed_size,
        result.ratio
    )
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Original tests (preserved) ----

    #[test]
    fn test_default_config() {
        let cfg = default_lz77_config();
        assert_eq!(cfg.window_size, 32768);
        assert_eq!(cfg.min_match, 3);
    }

    #[test]
    fn test_compress_empty() {
        let cfg = default_lz77_config();
        let result = lz77_compress(b"", &cfg);
        assert_eq!(lz77_token_count(&result), 0);
    }

    #[test]
    fn test_compress_produces_tokens() {
        let cfg = default_lz77_config();
        let result = lz77_compress(b"hello", &cfg);
        // "hello" has no repeated 3-byte sequence, so all 5 tokens are literals
        assert_eq!(lz77_token_count(&result), 5);
    }

    #[test]
    fn test_roundtrip() {
        let cfg = default_lz77_config();
        let data = b"hello world";
        let result = lz77_compress(data, &cfg);
        let decoded = lz77_decompress(&result.tokens);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_all_literals() {
        let cfg = default_lz77_config();
        let data = b"abc";
        let result = lz77_compress(data, &cfg);
        for tok in &result.tokens {
            assert!(lz77_is_literal(tok));
        }
    }

    #[test]
    fn test_compression_ratio_stub() {
        let cfg = default_lz77_config();
        let result = lz77_compress(b"test", &cfg);
        // "test" has no repeats >= 3 bytes, so ratio should be 1.0 (all literals).
        assert!((lz77_compression_ratio(&result) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let cfg = default_lz77_config();
        let result = lz77_compress(b"hi", &cfg);
        let j = lz77_to_json(&result);
        assert!(j.contains("token_count"));
    }

    #[test]
    fn test_decompress_back_reference() {
        let tokens = vec![
            Lz77Token {
                is_literal: true,
                literal: b'a',
                offset: 0,
                length: 0,
            },
            Lz77Token {
                is_literal: true,
                literal: b'b',
                offset: 0,
                length: 0,
            },
            Lz77Token {
                is_literal: false,
                literal: 0,
                offset: 2,
                length: 2,
            },
        ];
        let decoded = lz77_decompress(&tokens);
        assert_eq!(decoded, b"abab");
    }

    // ---- New tests for real compression ----

    #[test]
    fn test_new_api_empty() {
        let tokens = compress(b"");
        assert!(tokens.is_empty());
        let out = decompress(&tokens);
        assert!(out.is_empty());
    }

    #[test]
    fn test_new_api_single_byte() {
        let tokens = compress(b"x");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Literal(b'x'));
        assert_eq!(decompress(&tokens), b"x");
    }

    #[test]
    fn test_new_api_no_repeats() {
        let data = b"abcdefghij";
        let tokens = compress(data);
        // All unique 3-byte windows, so should be all literals
        for tok in &tokens {
            assert!(matches!(tok, Token::Literal(_)));
        }
        assert_eq!(decompress(&tokens), data);
    }

    #[test]
    fn test_new_api_roundtrip_repetitive() {
        let data = b"abcabcabcabcabcabcabcabcabc";
        let tokens = compress(data);
        let out = decompress(&tokens);
        assert_eq!(out, data);
        // Should have fewer tokens than data length (real compression)
        assert!(
            tokens.len() < data.len(),
            "Expected compression: {} tokens for {} bytes",
            tokens.len(),
            data.len()
        );
    }

    #[test]
    fn test_new_api_all_same_byte() {
        // Highly compressible
        let data = vec![b'A'; 1000];
        let tokens = compress(&data);
        let out = decompress(&tokens);
        assert_eq!(out, data);
        // Should compress very well
        assert!(
            tokens.len() < 20,
            "Expected high compression for 1000 identical bytes, got {} tokens",
            tokens.len()
        );
    }

    #[test]
    fn test_new_api_compression_ratio_repetitive() {
        // Build repetitive data
        let pattern = b"the quick brown fox jumps over the lazy dog. ";
        let mut data = Vec::new();
        for _ in 0..50 {
            data.extend_from_slice(pattern);
        }
        let tokens = compress(&data);
        let out = decompress(&tokens);
        assert_eq!(out, data);

        let ratio = tokens.len() as f64 / data.len() as f64;
        assert!(
            ratio < 0.5,
            "Expected ratio < 0.5 on repetitive text, got {:.3}",
            ratio
        );
    }

    #[test]
    fn test_new_api_run_length_match() {
        // distance=1 overlap: "aaa...a"
        let data = vec![b'z'; 300];
        let tokens = compress(&data);
        let out = decompress(&tokens);
        assert_eq!(out, data);

        // Should have very few tokens
        let has_match = tokens.iter().any(|t| matches!(t, Token::Match { .. }));
        assert!(has_match, "Expected at least one Match token");
    }

    #[test]
    fn test_new_api_binary_data_roundtrip() {
        let data: Vec<u8> = (0..=255).cycle().take(2048).collect();
        let tokens = compress(&data);
        let out = decompress(&tokens);
        assert_eq!(out, data);
    }

    #[test]
    fn test_new_api_lazy_matching_benefit() {
        // Construct data where lazy matching helps
        let data = b"xyzabcxyzabcdefabcdef";
        let tokens = compress(data);
        let out = decompress(&tokens);
        assert_eq!(out.as_slice(), data.as_slice());
    }

    #[test]
    fn test_new_api_short_data() {
        // Data shorter than MIN_MATCH
        let data = b"ab";
        let tokens = compress(data);
        assert_eq!(tokens.len(), 2);
        assert_eq!(decompress(&tokens), data);
    }

    #[test]
    fn test_decompress_invalid_backref() {
        // Invalid distance (larger than output so far) should be skipped
        let tokens = vec![
            Token::Literal(b'a'),
            Token::Match {
                distance: 100,
                length: 5,
            },
            Token::Literal(b'b'),
        ];
        let out = decompress(&tokens);
        assert_eq!(out, b"ab");
    }

    #[test]
    fn test_legacy_api_roundtrip_repetitive() {
        let cfg = default_lz77_config();
        let data = b"abcdefabcdefabcdefabcdefabcdef";
        let result = lz77_compress(data, &cfg);
        let decoded = lz77_decompress(&result.tokens);
        assert_eq!(decoded, data);
        assert!(
            lz77_compression_ratio(&result) < 1.0,
            "Expected ratio < 1.0, got {}",
            result.ratio
        );
    }

    #[test]
    fn test_compress_with_params_small_window() {
        let data = b"abcabcabcabcabc";
        let tokens = compress_with_params(data, 8, 10);
        let out = decompress(&tokens);
        assert_eq!(out.as_slice(), data.as_slice());
    }

    #[test]
    fn test_large_match() {
        // Ensure matches up to DEFAULT_MAX_MATCH work
        let chunk: Vec<u8> = (0..200).collect();
        let mut data = chunk.clone();
        data.extend_from_slice(&chunk);
        let tokens = compress(&data);
        let out = decompress(&tokens);
        assert_eq!(out, data);

        let has_long_match = tokens.iter().any(|t| match t {
            Token::Match { length, .. } => *length >= 100,
            _ => false,
        });
        assert!(has_long_match, "Expected a long match (>=100 bytes)");
    }
}
