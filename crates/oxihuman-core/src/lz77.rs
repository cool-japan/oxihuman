// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! LZ77 compression (sliding window encode/decode).

#![allow(dead_code)]

/// A single LZ77 token.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(u8),
    Match { offset: usize, length: usize },
}

/// Encode input bytes using LZ77 with given window and lookahead sizes.
#[allow(dead_code)]
pub fn encode(data: &[u8], window: usize, lookahead: usize) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    while pos < data.len() {
        let (best_offset, best_len) = find_best_match(data, pos, window, lookahead);
        if best_len >= 3 {
            tokens.push(Token::Match { offset: best_offset, length: best_len });
            pos += best_len;
        } else {
            tokens.push(Token::Literal(data[pos]));
            pos += 1;
        }
    }
    tokens
}

fn find_best_match(data: &[u8], pos: usize, window: usize, lookahead: usize) -> (usize, usize) {
    let start = pos.saturating_sub(window);
    let max_len = lookahead.min(data.len() - pos);
    let mut best_offset = 0;
    let mut best_len = 0;
    for back in start..pos {
        let mut length = 0;
        while length < max_len && data[back + length] == data[pos + length] {
            length += 1;
            if back + length >= pos {
                break;
            }
        }
        if length > best_len {
            best_len = length;
            best_offset = pos - back;
        }
    }
    (best_offset, best_len)
}

/// Decode LZ77 tokens back to bytes.
#[allow(dead_code)]
pub fn decode(tokens: &[Token]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::new();
    for token in tokens {
        match token {
            Token::Literal(b) => output.push(*b),
            Token::Match { offset, length } => {
                let start = output.len().saturating_sub(*offset);
                for i in 0..*length {
                    let byte = output[start + i];
                    output.push(byte);
                }
            }
        }
    }
    output
}

/// Compression ratio: compressed_tokens / input_bytes.
#[allow(dead_code)]
pub fn compression_ratio(original_len: usize, token_count: usize) -> f64 {
    if original_len == 0 {
        return 1.0;
    }
    token_count as f64 / original_len as f64
}

/// Count literal tokens.
#[allow(dead_code)]
pub fn literal_count(tokens: &[Token]) -> usize {
    tokens.iter().filter(|t| matches!(t, Token::Literal(_))).count()
}

/// Count match tokens.
#[allow(dead_code)]
pub fn match_count(tokens: &[Token]) -> usize {
    tokens.iter().filter(|t| matches!(t, Token::Match { .. })).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let input = b"abcabcabcabc";
        let tokens = encode(input, 255, 255);
        let decoded = decode(&tokens);
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_no_repetition() {
        let input = b"abcdefgh";
        let tokens = encode(input, 255, 255);
        assert_eq!(literal_count(&tokens), 8);
    }

    #[test]
    fn test_all_same() {
        let input = b"aaaaaaaaaaaa";
        let tokens = encode(input, 255, 255);
        let decoded = decode(&tokens);
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_empty_input() {
        let tokens = encode(b"", 255, 255);
        assert!(tokens.is_empty());
        assert_eq!(decode(&tokens), b"");
    }

    #[test]
    fn test_single_byte() {
        let input = b"x";
        let tokens = encode(input, 255, 255);
        assert_eq!(tokens, vec![Token::Literal(b'x')]);
        assert_eq!(decode(&tokens), input);
    }

    #[test]
    fn test_match_count_increases_with_repetition() {
        let input: Vec<u8> = b"abcabc".to_vec();
        let tokens = encode(&input, 255, 255);
        assert!(match_count(&tokens) >= 1);
    }

    #[test]
    fn test_compression_ratio() {
        let r = compression_ratio(100, 50);
        assert!((r - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_compression_ratio_empty() {
        assert!((compression_ratio(0, 0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_long_repetitive() {
        let input: Vec<u8> = b"xyxyxyxyxyxy".to_vec();
        let tokens = encode(&input, 255, 255);
        let decoded = decode(&tokens);
        assert_eq!(decoded, input);
        assert!(tokens.len() < input.len());
    }

    #[test]
    fn test_literal_count() {
        let tokens = vec![
            Token::Literal(1),
            Token::Match { offset: 1, length: 3 },
            Token::Literal(2),
        ];
        assert_eq!(literal_count(&tokens), 2);
        assert_eq!(match_count(&tokens), 1);
    }
}
