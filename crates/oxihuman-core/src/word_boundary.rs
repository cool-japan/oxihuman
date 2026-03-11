// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Unicode word boundary detector stub.
//!
//! Identifies word boundaries in text according to a simplified version of
//! the Unicode Word Boundary algorithm (UAX#29).

/// A word span in the source string.
#[derive(Debug, Clone, PartialEq)]
pub struct WordSpan {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub is_word: bool,
}

impl WordSpan {
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty_span(&self) -> bool {
        self.start == self.end
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Configuration for word boundary detection.
#[derive(Debug, Clone)]
pub struct WordBoundaryConfig {
    /// If true, hyphenated compounds ("well-known") are treated as one word.
    pub merge_hyphenated: bool,
    /// If true, apostrophes in contractions ("don't") are kept inside the word.
    pub keep_apostrophe_words: bool,
}

impl Default for WordBoundaryConfig {
    fn default() -> Self {
        Self {
            merge_hyphenated: false,
            keep_apostrophe_words: true,
        }
    }
}

/// Determine whether a character is a word character.
pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Split text into word spans (alternating word / non-word segments).
pub fn find_word_spans(text: &str, cfg: &WordBoundaryConfig) -> Vec<WordSpan> {
    let mut spans: Vec<WordSpan> = Vec::new();
    let start = 0usize;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let n = chars.len();

    let mut i = 0;
    while i < n {
        let (byte_start, ch) = chars[i];
        let word = if is_word_char(ch) {
            /* Accumulate word characters */
            let mut j = i + 1;
            while j < n {
                let (_, nc) = chars[j];
                if is_word_char(nc) {
                    j += 1;
                } else if cfg.keep_apostrophe_words
                    && nc == '\''
                    && j + 1 < n
                    && is_word_char(chars[j + 1].1)
                {
                    j += 1; /* keep apostrophe */
                } else if cfg.merge_hyphenated
                    && nc == '-'
                    && j + 1 < n
                    && is_word_char(chars[j + 1].1)
                {
                    j += 1;
                } else {
                    break;
                }
            }
            let byte_end = if j < n { chars[j].0 } else { text.len() };
            let span = WordSpan {
                text: text[byte_start..byte_end].to_string(),
                start: byte_start,
                end: byte_end,
                is_word: true,
            };
            i = j;
            span
        } else {
            /* Non-word segment */
            let j = i + 1;
            let byte_end = if j < n { chars[j].0 } else { text.len() };
            let span = WordSpan {
                text: text[byte_start..byte_end].to_string(),
                start: byte_start,
                end: byte_end,
                is_word: false,
            };
            i = j;
            span
        };
        spans.push(word);
    }
    let _ = start;
    spans
}

/// Extract only the word spans (filtering out punctuation / whitespace).
pub fn extract_words(text: &str, cfg: &WordBoundaryConfig) -> Vec<String> {
    find_word_spans(text, cfg)
        .into_iter()
        .filter(|s| s.is_word)
        .map(|s| s.text)
        .collect()
}

/// Count words in the text.
pub fn word_count(text: &str) -> usize {
    let cfg = WordBoundaryConfig::default();
    extract_words(text, &cfg).len()
}

/// Check whether `pos` (byte offset) is a word boundary.
pub fn is_boundary_at(text: &str, pos: usize) -> bool {
    if pos == 0 || pos == text.len() {
        return true;
    }
    let before = text[..pos]
        .chars()
        .last()
        .map(is_word_char)
        .unwrap_or(false);
    let after = text[pos..]
        .chars()
        .next()
        .map(is_word_char)
        .unwrap_or(false);
    before != after
}

/// Find all word boundary byte positions in `text`.
pub fn word_boundary_positions(text: &str) -> Vec<usize> {
    let mut positions = vec![0];
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    for win in chars.windows(2) {
        let (_, a) = win[0];
        let (pos_b, b) = win[1];
        if is_word_char(a) != is_word_char(b) {
            positions.push(pos_b);
        }
    }
    if !text.is_empty() {
        positions.push(text.len());
    }
    positions.dedup();
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_cfg() -> WordBoundaryConfig {
        WordBoundaryConfig::default()
    }

    #[test]
    fn test_extract_words_simple() {
        let words = extract_words("hello world", &default_cfg());
        assert_eq!(words, vec!["hello", "world"]);
    }

    #[test]
    fn test_word_count() {
        assert_eq!(word_count("one two three"), 3);
    }

    #[test]
    fn test_punctuation_not_word() {
        let spans = find_word_spans("hi, there", &default_cfg());
        assert!(spans.iter().any(|s| !s.is_word));
    }

    #[test]
    fn test_is_boundary_at_space() {
        assert!(is_boundary_at("hello world", 5));
    }

    #[test]
    fn test_is_boundary_at_middle_not_boundary() {
        assert!(!is_boundary_at("hello", 2));
    }

    #[test]
    fn test_word_boundary_positions_count() {
        let pos = word_boundary_positions("hi there");
        assert!(pos.len() >= 2);
    }

    #[test]
    fn test_contraction_kept_together() {
        let words = extract_words("don't stop", &default_cfg());
        assert_eq!(words[0], "don't");
    }

    #[test]
    fn test_empty_text() {
        assert_eq!(word_count(""), 0);
    }

    #[test]
    fn test_is_word_char_underscore() {
        assert!(is_word_char('_'));
    }
}
