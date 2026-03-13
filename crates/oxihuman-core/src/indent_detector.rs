// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Auto-detect indentation style from source text.
//!
//! Analyses leading whitespace on each line and infers whether the file
//! uses spaces or tabs, and what the indent width is.

/// The detected indentation style.
#[derive(Debug, Clone, PartialEq)]
pub enum IndentStyle {
    Spaces(usize),
    Tabs,
    Mixed,
    Unknown,
}

/// Result of an indentation detection run.
#[derive(Debug, Clone)]
pub struct IndentResult {
    pub style: IndentStyle,
    /// Number of lines with leading spaces.
    pub space_lines: usize,
    /// Number of lines with leading tabs.
    pub tab_lines: usize,
    /// Total indented lines analysed.
    pub indented_lines: usize,
}

impl IndentResult {
    pub fn is_spaces(&self) -> bool {
        matches!(self.style, IndentStyle::Spaces(_))
    }

    pub fn is_tabs(&self) -> bool {
        self.style == IndentStyle::Tabs
    }

    pub fn indent_width(&self) -> Option<usize> {
        if let IndentStyle::Spaces(w) = self.style {
            Some(w)
        } else {
            None
        }
    }
}

/// Detect the indentation style used in a block of text.
pub fn detect_indent(text: &str) -> IndentResult {
    let mut space_lines = 0usize;
    let mut tab_lines = 0usize;
    let mut space_widths: Vec<usize> = Vec::new();

    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let leading_spaces = line.bytes().take_while(|&b| b == b' ').count();
        let leading_tabs = line.bytes().take_while(|&b| b == b'\t').count();

        if leading_spaces > 0 && leading_tabs == 0 {
            space_lines += 1;
            space_widths.push(leading_spaces);
        } else if leading_tabs > 0 && leading_spaces == 0 {
            tab_lines += 1;
        }
    }

    let indented_lines = space_lines + tab_lines;
    let style = if space_lines > 0 && tab_lines > 0 {
        IndentStyle::Mixed
    } else if tab_lines > 0 {
        IndentStyle::Tabs
    } else if space_lines > 0 {
        let width = infer_indent_width(&space_widths);
        IndentStyle::Spaces(width)
    } else {
        IndentStyle::Unknown
    };

    IndentResult {
        style,
        space_lines,
        tab_lines,
        indented_lines,
    }
}

/// Infer the most likely indent width from a list of leading-space counts.
fn infer_indent_width(widths: &[usize]) -> usize {
    /* Find the GCD of all widths as a heuristic */
    if widths.is_empty() {
        return 4;
    }
    let mut g = widths[0];
    for &w in &widths[1..] {
        if w > 0 {
            g = gcd(g, w);
        }
    }
    g.max(1)
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// Normalize the indentation of text to use spaces of a given width.
pub fn normalize_to_spaces(text: &str, width: usize) -> String {
    let mut out = String::new();
    for line in text.lines() {
        let tabs = line.bytes().take_while(|&b| b == b'\t').count();
        let spaces = " ".repeat(tabs * width);
        out.push_str(&spaces);
        out.push_str(&line[tabs..]);
        out.push('\n');
    }
    out
}

/// Normalize indentation to tabs.
pub fn normalize_to_tabs(text: &str, space_width: usize) -> String {
    if space_width == 0 {
        return text.to_string();
    }
    let mut out = String::new();
    for line in text.lines() {
        let spaces = line.bytes().take_while(|&b| b == b' ').count();
        let tabs = spaces / space_width;
        let remainder = spaces % space_width;
        out.push_str(&"\t".repeat(tabs));
        out.push_str(&" ".repeat(remainder));
        out.push_str(&line[spaces..]);
        out.push('\n');
    }
    out
}

/// Count how many lines have inconsistent indentation (mix within same file).
pub fn count_mixed_indent_lines(text: &str) -> usize {
    text.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            let leading_len = line.len() - trimmed.len();
            let leading = &line[..leading_len];
            leading.contains(' ') && leading.contains('\t')
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_spaces_4() {
        let text = "code\n    indented\n    also\n        deep\n";
        let r = detect_indent(text);
        assert!(r.is_spaces());
        assert_eq!(r.indent_width(), Some(4));
    }

    #[test]
    fn test_detect_tabs() {
        let text = "code\n\tindented\n\t\tdeeper\n";
        let r = detect_indent(text);
        assert!(r.is_tabs());
    }

    #[test]
    fn test_detect_unknown_no_indent() {
        let text = "no\nindent\nhere\n";
        let r = detect_indent(text);
        assert_eq!(r.style, IndentStyle::Unknown);
    }

    #[test]
    fn test_space_lines_count() {
        let text = "    a\n    b\n\tc\n";
        let r = detect_indent(text);
        assert_eq!(r.space_lines, 2);
        assert_eq!(r.tab_lines, 1);
    }

    #[test]
    fn test_normalize_tabs_to_spaces() {
        let text = "\thello\n";
        let norm = normalize_to_spaces(text, 2);
        assert_eq!(norm, "  hello\n");
    }

    #[test]
    fn test_normalize_spaces_to_tabs() {
        let text = "    hello\n";
        let norm = normalize_to_tabs(text, 4);
        assert_eq!(norm, "\thello\n");
    }

    #[test]
    fn test_count_mixed_lines() {
        let text = " \tmixed\nnormal\n";
        assert_eq!(count_mixed_indent_lines(text), 1);
    }

    #[test]
    fn test_detect_spaces_2() {
        let text = "  a\n  b\n    c\n";
        let r = detect_indent(text);
        assert_eq!(r.indent_width(), Some(2));
    }

    #[test]
    fn test_indented_lines_total() {
        let text = "  a\n  b\n\tc\n";
        let r = detect_indent(text);
        assert_eq!(r.indented_lines, 3);
    }
}
