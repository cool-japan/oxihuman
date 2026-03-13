// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Paragraph boundary detector stub.
//!
//! Splits text into paragraphs based on blank-line separators, with
//! configurable minimum blank-line count and optional list/heading detection.

/// A detected paragraph with its text and byte span.
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub text: String,
    pub start_line: usize,
    pub end_line: usize,
    pub kind: ParagraphKind,
}

impl Paragraph {
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line)
    }

    pub fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }

    pub fn is_empty_paragraph(&self) -> bool {
        self.text.trim().is_empty()
    }
}

/// The inferred type of a paragraph.
#[derive(Debug, Clone, PartialEq)]
pub enum ParagraphKind {
    Normal,
    Heading,
    ListItem,
    CodeBlock,
    Empty,
}

/// Configuration for paragraph detection.
#[derive(Debug, Clone)]
pub struct ParagraphConfig {
    /// Number of consecutive blank lines required to start a new paragraph.
    pub min_blank_lines: usize,
    /// Enable detection of Markdown-like headings (`#`, `##`, ...).
    pub detect_headings: bool,
    /// Enable detection of list items (`-`, `*`, `1.`).
    pub detect_lists: bool,
}

impl Default for ParagraphConfig {
    fn default() -> Self {
        Self {
            min_blank_lines: 1,
            detect_headings: true,
            detect_lists: true,
        }
    }
}

/// Infer the kind of paragraph from its first non-empty line.
fn infer_kind(line: &str, cfg: &ParagraphConfig) -> ParagraphKind {
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return ParagraphKind::Empty;
    }
    if cfg.detect_headings && trimmed.starts_with('#') {
        return ParagraphKind::Heading;
    }
    if cfg.detect_lists && (trimmed.starts_with("- ") || trimmed.starts_with("* ")) {
        return ParagraphKind::ListItem;
    }
    if cfg.detect_lists
        && trimmed
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        && trimmed.contains(". ")
    {
        return ParagraphKind::ListItem;
    }
    if trimmed.starts_with("```") {
        return ParagraphKind::CodeBlock;
    }
    ParagraphKind::Normal
}

/// Split text into paragraphs.
pub fn detect_paragraphs(text: &str, cfg: &ParagraphConfig) -> Vec<Paragraph> {
    let mut paragraphs: Vec<Paragraph> = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    let n = lines.len();

    let mut start = 0usize;
    let mut blank_run = 0usize;
    let mut current_lines: Vec<&str> = Vec::new();

    #[allow(clippy::ptr_arg)]
    let flush = |lines: &Vec<&str>,
                 s: usize,
                 e: usize,
                 paragraphs: &mut Vec<Paragraph>,
                 cfg: &ParagraphConfig| {
        let text = lines.join("\n");
        if text.trim().is_empty() {
            return;
        }
        let kind = lines
            .first()
            .map(|l| infer_kind(l, cfg))
            .unwrap_or(ParagraphKind::Normal);
        paragraphs.push(Paragraph {
            text,
            start_line: s,
            end_line: e,
            kind,
        });
    };

    let mut i = 0;
    while i < n {
        let line = lines[i];
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run >= cfg.min_blank_lines && !current_lines.is_empty() {
                flush(&current_lines, start, i, &mut paragraphs, cfg);
                current_lines.clear();
                start = i + 1;
            }
        } else {
            blank_run = 0;
            if current_lines.is_empty() {
                start = i;
            }
            current_lines.push(line);
        }
        i += 1;
    }

    if !current_lines.is_empty() {
        flush(&current_lines, start, n, &mut paragraphs, cfg);
    }

    paragraphs
}

/// Count paragraphs in text.
pub fn paragraph_count(text: &str) -> usize {
    let cfg = ParagraphConfig::default();
    detect_paragraphs(text, &cfg).len()
}

/// Return paragraph kinds summary.
pub fn kind_summary(paragraphs: &[Paragraph]) -> (usize, usize, usize, usize) {
    let normal = paragraphs
        .iter()
        .filter(|p| p.kind == ParagraphKind::Normal)
        .count();
    let heading = paragraphs
        .iter()
        .filter(|p| p.kind == ParagraphKind::Heading)
        .count();
    let list = paragraphs
        .iter()
        .filter(|p| p.kind == ParagraphKind::ListItem)
        .count();
    let code = paragraphs
        .iter()
        .filter(|p| p.kind == ParagraphKind::CodeBlock)
        .count();
    (normal, heading, list, code)
}

/// Find the longest paragraph by word count.
pub fn longest_paragraph(paragraphs: &[Paragraph]) -> Option<&Paragraph> {
    paragraphs.iter().max_by_key(|p| p.word_count())
}

/// Filter paragraphs with fewer than `min_words` words.
pub fn filter_by_min_words(paragraphs: Vec<Paragraph>, min_words: usize) -> Vec<Paragraph> {
    paragraphs
        .into_iter()
        .filter(|p| p.word_count() >= min_words)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str =
        "First paragraph.\nStill first.\n\nSecond paragraph.\n\n# A heading\n\n- list item\n";

    #[test]
    fn test_paragraph_count() {
        assert!(paragraph_count(SAMPLE) >= 3);
    }

    #[test]
    fn test_heading_detected() {
        let cfg = ParagraphConfig::default();
        let paras = detect_paragraphs(SAMPLE, &cfg);
        assert!(paras.iter().any(|p| p.kind == ParagraphKind::Heading));
    }

    #[test]
    fn test_list_item_detected() {
        let cfg = ParagraphConfig::default();
        let paras = detect_paragraphs(SAMPLE, &cfg);
        assert!(paras.iter().any(|p| p.kind == ParagraphKind::ListItem));
    }

    #[test]
    fn test_normal_paragraph() {
        let cfg = ParagraphConfig::default();
        let paras = detect_paragraphs(SAMPLE, &cfg);
        assert!(paras.iter().any(|p| p.kind == ParagraphKind::Normal));
    }

    #[test]
    fn test_paragraph_word_count() {
        let p = Paragraph {
            text: "one two three".into(),
            start_line: 0,
            end_line: 1,
            kind: ParagraphKind::Normal,
        };
        assert_eq!(p.word_count(), 3);
    }

    #[test]
    fn test_line_count() {
        let p = Paragraph {
            text: "x".into(),
            start_line: 2,
            end_line: 5,
            kind: ParagraphKind::Normal,
        };
        assert_eq!(p.line_count(), 3);
    }

    #[test]
    fn test_kind_summary() {
        let cfg = ParagraphConfig::default();
        let paras = detect_paragraphs(SAMPLE, &cfg);
        let (_, headings, lists, _) = kind_summary(&paras);
        assert!(headings >= 1);
        assert!(lists >= 1);
    }

    #[test]
    fn test_filter_by_min_words() {
        let cfg = ParagraphConfig::default();
        let paras = detect_paragraphs(SAMPLE, &cfg);
        let filtered = filter_by_min_words(paras, 5);
        assert!(filtered.iter().all(|p| p.word_count() >= 5));
    }

    #[test]
    fn test_empty_text() {
        assert_eq!(paragraph_count(""), 0);
    }
}
