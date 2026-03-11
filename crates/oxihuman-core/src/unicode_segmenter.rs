// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Unicode grapheme cluster segmenter stub.
//!
//! Splits a string into grapheme clusters (user-perceived characters).
//! For this stub, an ASCII character or a non-ASCII UTF-8 sequence is each
//! treated as one cluster. Full UAX#29 rules require external data tables.

/// A single grapheme cluster with its byte span.
#[derive(Debug, Clone, PartialEq)]
pub struct Grapheme {
    pub text: String,
    pub byte_start: usize,
    pub byte_end: usize,
}

impl Grapheme {
    pub fn len_bytes(&self) -> usize {
        self.byte_end.saturating_sub(self.byte_start)
    }

    pub fn is_ascii(&self) -> bool {
        self.text.is_ascii()
    }
}

/// Segment a UTF-8 string into grapheme clusters.
///
/// This stub splits on `char` boundaries, which is correct for BMP characters
/// but does not handle combining marks or emoji sequences.
pub fn segment_graphemes(text: &str) -> Vec<Grapheme> {
    let mut clusters = Vec::new();
    let mut byte_offset = 0usize;

    for ch in text.chars() {
        let ch_str = ch.to_string();
        let len = ch_str.len();
        clusters.push(Grapheme {
            text: ch_str,
            byte_start: byte_offset,
            byte_end: byte_offset + len,
        });
        byte_offset += len;
    }

    clusters
}

/// Count the number of grapheme clusters in a string.
pub fn grapheme_count(text: &str) -> usize {
    text.chars().count()
}

/// Return the Nth grapheme cluster (0-based), or `None` if out of range.
pub fn nth_grapheme(text: &str, n: usize) -> Option<Grapheme> {
    segment_graphemes(text).into_iter().nth(n)
}

/// Reverse the grapheme cluster order of a string.
pub fn reverse_graphemes(text: &str) -> String {
    segment_graphemes(text)
        .iter()
        .rev()
        .map(|g| g.text.as_str())
        .collect()
}

/// Truncate `text` to at most `max_clusters` grapheme clusters.
pub fn truncate_graphemes(text: &str, max_clusters: usize) -> &str {
    let graphemes = segment_graphemes(text);
    if graphemes.len() <= max_clusters {
        return text;
    }
    let end = graphemes[max_clusters].byte_start;
    &text[..end]
}

/// Check whether a string contains any non-ASCII grapheme clusters.
pub fn has_multibyte_graphemes(text: &str) -> bool {
    !text.is_ascii()
}

/// Split text into lines of at most `width` grapheme clusters.
pub fn word_wrap_graphemes(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }
    let graphemes = segment_graphemes(text);
    graphemes
        .chunks(width)
        .map(|chunk| chunk.iter().map(|g| g.text.as_str()).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_segment_count() {
        let g = segment_graphemes("hello");
        assert_eq!(g.len(), 5);
    }

    #[test]
    fn test_grapheme_byte_spans_ascii() {
        let g = segment_graphemes("ab");
        assert_eq!(g[0].byte_start, 0);
        assert_eq!(g[0].byte_end, 1);
        assert_eq!(g[1].byte_start, 1);
    }

    #[test]
    fn test_multibyte_grapheme() {
        /* 'é' is 2 bytes in UTF-8 */
        let g = segment_graphemes("é");
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].len_bytes(), 2);
    }

    #[test]
    fn test_grapheme_count() {
        assert_eq!(grapheme_count("hello"), 5);
        assert_eq!(grapheme_count(""), 0);
    }

    #[test]
    fn test_nth_grapheme() {
        let g = nth_grapheme("abc", 1);
        assert_eq!(g.unwrap().text, "b");
    }

    #[test]
    fn test_reverse_graphemes() {
        assert_eq!(reverse_graphemes("abc"), "cba");
    }

    #[test]
    fn test_truncate_graphemes() {
        let t = truncate_graphemes("hello", 3);
        assert_eq!(t, "hel");
    }

    #[test]
    fn test_has_multibyte_graphemes_false() {
        assert!(!has_multibyte_graphemes("hello"));
    }

    #[test]
    fn test_has_multibyte_graphemes_true() {
        assert!(has_multibyte_graphemes("héllo"));
    }

    #[test]
    fn test_word_wrap_graphemes() {
        let lines = word_wrap_graphemes("abcdef", 2);
        assert_eq!(lines.len(), 3);
    }
}
