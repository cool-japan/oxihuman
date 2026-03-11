// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Normalize whitespace and line endings in text buffers.
//!
//! Handles CRLF → LF, trailing whitespace trimming, collapsing of
//! redundant blank lines, and EOF newline normalization.

/// The desired line-ending style.
#[derive(Debug, Clone, PartialEq)]
pub enum LineEnding {
    Lf,
    CrLf,
    Cr,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }
}

/// Configuration for whitespace normalization.
#[derive(Debug, Clone)]
pub struct NormalizerConfig {
    pub target_ending: LineEnding,
    pub trim_trailing: bool,
    pub max_blank_lines: usize,
    pub ensure_final_newline: bool,
}

impl Default for NormalizerConfig {
    fn default() -> Self {
        Self {
            target_ending: LineEnding::Lf,
            trim_trailing: true,
            max_blank_lines: 2,
            ensure_final_newline: true,
        }
    }
}

/// Statistics about whitespace issues found in text.
#[derive(Debug, Clone, Default)]
pub struct WhitespaceStats {
    pub crlf_count: usize,
    pub trailing_whitespace_lines: usize,
    pub excess_blank_lines: usize,
    pub missing_final_newline: bool,
}

impl WhitespaceStats {
    pub fn has_issues(&self) -> bool {
        self.crlf_count > 0
            || self.trailing_whitespace_lines > 0
            || self.excess_blank_lines > 0
            || self.missing_final_newline
    }
}

/// Detect whitespace issues in `text` relative to `cfg`.
pub fn detect_issues(text: &str, cfg: &NormalizerConfig) -> WhitespaceStats {
    let mut stats = WhitespaceStats::default();
    let mut blank_run = 0usize;

    stats.crlf_count = text.matches("\r\n").count();

    for line in text.lines() {
        if line.is_empty() {
            blank_run += 1;
            if blank_run > cfg.max_blank_lines {
                stats.excess_blank_lines += 1;
            }
        } else {
            blank_run = 0;
            if line != line.trim_end() {
                stats.trailing_whitespace_lines += 1;
            }
        }
    }

    stats.missing_final_newline =
        cfg.ensure_final_newline && !text.ends_with('\n') && !text.ends_with("\r\n");
    stats
}

/// Normalize whitespace in `text` according to `cfg`.
pub fn normalize(text: &str, cfg: &NormalizerConfig) -> String {
    /* Convert all endings to LF first */
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");
    let ending = cfg.target_ending.as_str();

    let mut out = String::with_capacity(unified.len());
    let mut blank_run = 0usize;

    for line in unified.lines() {
        let processed = if cfg.trim_trailing {
            line.trim_end()
        } else {
            line
        };
        if processed.is_empty() {
            blank_run += 1;
            if blank_run <= cfg.max_blank_lines {
                out.push_str(ending);
            }
        } else {
            blank_run = 0;
            out.push_str(processed);
            out.push_str(ending);
        }
    }

    /* Handle ensure_final_newline */
    if cfg.ensure_final_newline && !out.ends_with(ending) {
        out.push_str(ending);
    }

    out
}

/// Count trailing whitespace characters on a single line.
pub fn trailing_whitespace_count(line: &str) -> usize {
    line.len().saturating_sub(line.trim_end().len())
}

/// Strip all trailing whitespace from every line, joining with `\n`.
pub fn strip_trailing(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

/// Collapse runs of blank lines to a maximum of `max` consecutive.
pub fn collapse_blank_lines(text: &str, max: usize) -> String {
    let mut out = String::new();
    let mut blank = 0usize;
    for line in text.lines() {
        if line.trim().is_empty() {
            blank += 1;
            if blank <= max {
                out.push('\n');
            }
        } else {
            blank = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_crlf() {
        let cfg = NormalizerConfig::default();
        let stats = detect_issues("line1\r\nline2\r\n", &cfg);
        assert_eq!(stats.crlf_count, 2);
    }

    #[test]
    fn test_detect_trailing_whitespace() {
        let cfg = NormalizerConfig::default();
        let stats = detect_issues("line   \nclean\n", &cfg);
        assert_eq!(stats.trailing_whitespace_lines, 1);
    }

    #[test]
    fn test_normalize_crlf_to_lf() {
        let cfg = NormalizerConfig::default();
        let result = normalize("a\r\nb\r\n", &cfg);
        assert!(!result.contains('\r'));
    }

    #[test]
    fn test_normalize_trim_trailing() {
        let cfg = NormalizerConfig::default();
        let result = normalize("hello   \n", &cfg);
        assert_eq!(result, "hello\n");
    }

    #[test]
    fn test_collapse_blank_lines() {
        let text = "a\n\n\n\nb\n";
        let result = collapse_blank_lines(text, 1);
        assert!(result.matches('\n').count() < text.matches('\n').count());
    }

    #[test]
    fn test_strip_trailing() {
        let result = strip_trailing("  hi   \n");
        assert_eq!(result, "  hi\n");
    }

    #[test]
    fn test_trailing_whitespace_count() {
        assert_eq!(trailing_whitespace_count("abc   "), 3);
        assert_eq!(trailing_whitespace_count("abc"), 0);
    }

    #[test]
    fn test_has_issues_false_for_clean() {
        let cfg = NormalizerConfig::default();
        let stats = detect_issues("clean line\n", &cfg);
        assert!(!stats.has_issues());
    }

    #[test]
    fn test_line_ending_as_str() {
        assert_eq!(LineEnding::Lf.as_str(), "\n");
        assert_eq!(LineEnding::CrLf.as_str(), "\r\n");
    }
}
