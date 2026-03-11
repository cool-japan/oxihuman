// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sentence boundary detector stub.
//!
//! Splits text into sentences using a simplified heuristic approach based on
//! terminal punctuation (`.`, `!`, `?`) followed by whitespace and an uppercase letter.

/// A detected sentence with its byte span.
#[derive(Debug, Clone, PartialEq)]
pub struct Sentence {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

impl Sentence {
    pub fn byte_len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn word_count_est(&self) -> usize {
        self.text.split_whitespace().count()
    }
}

/// Configuration for sentence splitting.
#[derive(Debug, Clone)]
pub struct SentenceSplitterConfig {
    /// Terminal punctuation characters.
    pub terminals: Vec<char>,
    /// If true, try to handle abbreviations (Mr., Dr., etc.) by not splitting.
    pub abbreviation_guard: bool,
}

impl Default for SentenceSplitterConfig {
    fn default() -> Self {
        Self {
            terminals: vec!['.', '!', '?'],
            abbreviation_guard: true,
        }
    }
}

static ABBREVIATIONS: &[&str] = &[
    "Mr", "Mrs", "Ms", "Dr", "Prof", "Sr", "Jr", "vs", "etc", "St", "Ave", "Blvd", "Dept", "est",
];

/// Split text into sentences.
pub fn split_sentences(text: &str, cfg: &SentenceSplitterConfig) -> Vec<Sentence> {
    let mut sentences: Vec<Sentence> = Vec::new();
    let mut start = 0usize;
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        let (byte_pos, ch) = chars[i];
        if cfg.terminals.contains(&ch) {
            /* Check for abbreviation guard */
            let is_abbrev = if cfg.abbreviation_guard {
                /* Look back at the word before the dot */
                let before = &text[start..byte_pos];
                let last_word = before.split_whitespace().last().unwrap_or("");
                ABBREVIATIONS
                    .iter()
                    .any(|a| last_word.eq_ignore_ascii_case(a))
            } else {
                false
            };

            /* Look ahead for whitespace + uppercase to confirm sentence end */
            let next_upper = (i + 1..n)
                .find(|&j| {
                    let (_, nc) = chars[j];
                    !nc.is_whitespace()
                })
                .map(|j| chars[j].1.is_uppercase())
                .unwrap_or(false);

            if !is_abbrev && (next_upper || i + 1 == n) {
                let end = byte_pos + ch.len_utf8();
                let sentence_text = text[start..end].trim().to_string();
                if !sentence_text.is_empty() {
                    sentences.push(Sentence {
                        text: sentence_text,
                        start,
                        end,
                    });
                }
                /* Skip whitespace */
                let mut j = i + 1;
                while j < n && chars[j].1.is_whitespace() {
                    j += 1;
                }
                start = if j < n { chars[j].0 } else { text.len() };
                i = j;
                continue;
            }
        }
        i += 1;
    }

    /* Remaining text */
    if start < text.len() {
        let remainder = text[start..].trim().to_string();
        if !remainder.is_empty() {
            sentences.push(Sentence {
                text: remainder,
                start,
                end: text.len(),
            });
        }
    }

    sentences
}

/// Count the number of sentences in text.
pub fn sentence_count(text: &str) -> usize {
    let cfg = SentenceSplitterConfig::default();
    split_sentences(text, &cfg).len()
}

/// Return the average word count per sentence.
pub fn avg_words_per_sentence(text: &str) -> f64 {
    let cfg = SentenceSplitterConfig::default();
    let sents = split_sentences(text, &cfg);
    if sents.is_empty() {
        return 0.0;
    }
    let total: usize = sents.iter().map(|s| s.word_count_est()).sum();
    total as f64 / sents.len() as f64
}

/// Find the longest sentence by character count.
pub fn longest_sentence(sentences: &[Sentence]) -> Option<&Sentence> {
    sentences.iter().max_by_key(|s| s.text.len())
}

/// Filter sentences shorter than `min_words` words.
pub fn filter_short_sentences(sentences: Vec<Sentence>, min_words: usize) -> Vec<Sentence> {
    sentences
        .into_iter()
        .filter(|s| s.word_count_est() >= min_words)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_split() {
        let text = "Hello world. How are you? I am fine!";
        let sents = split_sentences(text, &SentenceSplitterConfig::default());
        assert!(sents.len() >= 2);
    }

    #[test]
    fn test_sentence_count() {
        let text = "First. Second. Third.";
        assert!(sentence_count(text) >= 1);
    }

    #[test]
    fn test_byte_len() {
        let s = Sentence {
            text: "Hi.".into(),
            start: 0,
            end: 3,
        };
        assert_eq!(s.byte_len(), 3);
    }

    #[test]
    fn test_word_count_est() {
        let s = Sentence {
            text: "One two three.".into(),
            start: 0,
            end: 14,
        };
        assert_eq!(s.word_count_est(), 3);
    }

    #[test]
    fn test_avg_words_per_sentence() {
        let text = "One two. Three four five.";
        let avg = avg_words_per_sentence(text);
        assert!(avg > 0.0);
    }

    #[test]
    fn test_longest_sentence() {
        let sents = vec![
            Sentence {
                text: "Hi.".into(),
                start: 0,
                end: 3,
            },
            Sentence {
                text: "Hello world friend.".into(),
                start: 4,
                end: 23,
            },
        ];
        let longest = longest_sentence(&sents).unwrap();
        assert_eq!(longest.text, "Hello world friend.");
    }

    #[test]
    fn test_filter_short() {
        let sents = vec![
            Sentence {
                text: "Hi.".into(),
                start: 0,
                end: 3,
            },
            Sentence {
                text: "Hello there world.".into(),
                start: 4,
                end: 22,
            },
        ];
        let filtered = filter_short_sentences(sents, 2);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_empty_text() {
        assert_eq!(sentence_count(""), 0);
    }

    #[test]
    fn test_no_terminal_is_one_sentence() {
        let text = "this has no terminal punctuation";
        let sents = split_sentences(text, &SentenceSplitterConfig::default());
        assert_eq!(sents.len(), 1);
    }
}
