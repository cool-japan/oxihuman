// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Conflict marker parser and renderer.
//!
//! Parses `<<<<<<< / ======= / >>>>>>>` conflict markers in text and can
//! re-render them in various formats (unified, labelled, etc.).

/// Default conflict separator strings.
pub const MARKER_OURS: &str = "<<<<<<<";
pub const MARKER_SEP: &str = "=======";
pub const MARKER_THEIRS: &str = ">>>>>>>";

/// A parsed conflict block.
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictBlock {
    pub ours_label: Option<String>,
    pub theirs_label: Option<String>,
    pub ours_lines: Vec<String>,
    pub theirs_lines: Vec<String>,
}

impl ConflictBlock {
    pub fn new(ours: Vec<String>, theirs: Vec<String>) -> Self {
        Self {
            ours_label: None,
            theirs_label: None,
            ours_lines: ours,
            theirs_lines: theirs,
        }
    }

    pub fn ours_len(&self) -> usize {
        self.ours_lines.len()
    }
    pub fn theirs_len(&self) -> usize {
        self.theirs_lines.len()
    }
}

/// Result of parsing a conflicted text.
#[derive(Debug, Clone)]
pub struct ParsedConflicts {
    pub preamble: Vec<String>,
    pub blocks: Vec<ConflictBlock>,
    pub postamble: Vec<String>,
}

impl ParsedConflicts {
    pub fn new() -> Self {
        Self {
            preamble: Vec::new(),
            blocks: Vec::new(),
            postamble: Vec::new(),
        }
    }

    pub fn conflict_count(&self) -> usize {
        self.blocks.len()
    }
    pub fn has_conflicts(&self) -> bool {
        !self.blocks.is_empty()
    }
}

impl Default for ParsedConflicts {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse conflict markers in a block of text (lines).
pub fn parse_conflict_markers(text: &str) -> ParsedConflicts {
    let mut result = ParsedConflicts::new();
    let mut in_ours = false;
    let mut in_theirs = false;
    let mut current_ours: Vec<String> = Vec::new();
    let mut current_theirs: Vec<String> = Vec::new();
    let mut ours_label: Option<String> = None;

    for raw_line in text.lines() {
        if raw_line.starts_with(MARKER_OURS) {
            in_ours = true;
            in_theirs = false;
            ours_label = if raw_line.len() > 7 {
                Some(raw_line[7..].trim().to_string())
            } else {
                None
            };
            current_ours.clear();
            current_theirs.clear();
        } else if raw_line.starts_with(MARKER_SEP) && in_ours {
            in_ours = false;
            in_theirs = true;
        } else if raw_line.starts_with(MARKER_THEIRS) && in_theirs {
            in_theirs = false;
            let theirs_label: Option<String> = if raw_line.len() > 7 {
                Some(raw_line[7..].trim().to_string())
            } else {
                None
            };
            let mut block = ConflictBlock::new(current_ours.clone(), current_theirs.clone());
            block.ours_label = ours_label.take();
            block.theirs_label = theirs_label;
            result.blocks.push(block);
        } else if in_ours {
            current_ours.push(raw_line.to_string());
        } else if in_theirs {
            current_theirs.push(raw_line.to_string());
        } else if result.blocks.is_empty() {
            result.preamble.push(raw_line.to_string());
        } else {
            result.postamble.push(raw_line.to_string());
        }
    }
    result
}

/// Render a conflict block back to text with standard markers.
pub fn render_conflict_block(block: &ConflictBlock) -> String {
    let mut out = String::new();
    let ours_lbl = block.ours_label.as_deref().unwrap_or("OURS");
    let theirs_lbl = block.theirs_label.as_deref().unwrap_or("THEIRS");
    out.push_str(&format!("{} {}\n", MARKER_OURS, ours_lbl));
    for l in &block.ours_lines {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str(MARKER_SEP);
    out.push('\n');
    for l in &block.theirs_lines {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str(&format!("{} {}\n", MARKER_THEIRS, theirs_lbl));
    out
}

/// Resolve all conflicts by choosing one side.
pub fn resolve_all(parsed: &ParsedConflicts, prefer_ours: bool) -> Vec<String> {
    let mut lines: Vec<String> = parsed.preamble.clone();
    for block in &parsed.blocks {
        if prefer_ours {
            lines.extend(block.ours_lines.iter().cloned());
        } else {
            lines.extend(block.theirs_lines.iter().cloned());
        }
    }
    lines.extend(parsed.postamble.iter().cloned());
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str =
        "before\n<<<<<<< HEAD\nours line\n=======\ntheirs line\n>>>>>>> branch\nafter\n";

    #[test]
    fn test_parse_finds_one_conflict() {
        let p = parse_conflict_markers(SAMPLE);
        assert_eq!(p.conflict_count(), 1);
    }

    #[test]
    fn test_parse_preamble() {
        let p = parse_conflict_markers(SAMPLE);
        assert_eq!(p.preamble, vec!["before"]);
    }

    #[test]
    fn test_parse_ours_lines() {
        let p = parse_conflict_markers(SAMPLE);
        assert_eq!(p.blocks[0].ours_lines, vec!["ours line"]);
    }

    #[test]
    fn test_parse_theirs_lines() {
        let p = parse_conflict_markers(SAMPLE);
        assert_eq!(p.blocks[0].theirs_lines, vec!["theirs line"]);
    }

    #[test]
    fn test_parse_postamble() {
        let p = parse_conflict_markers(SAMPLE);
        assert_eq!(p.postamble, vec!["after"]);
    }

    #[test]
    fn test_render_contains_markers() {
        let block = ConflictBlock::new(vec!["a".into()], vec!["b".into()]);
        let rendered = render_conflict_block(&block);
        assert!(rendered.contains(MARKER_OURS));
        assert!(rendered.contains(MARKER_SEP));
        assert!(rendered.contains(MARKER_THEIRS));
    }

    #[test]
    fn test_resolve_ours() {
        let p = parse_conflict_markers(SAMPLE);
        let lines = resolve_all(&p, true);
        assert!(lines.contains(&"ours line".to_string()));
        assert!(!lines.contains(&"theirs line".to_string()));
    }

    #[test]
    fn test_resolve_theirs() {
        let p = parse_conflict_markers(SAMPLE);
        let lines = resolve_all(&p, false);
        assert!(lines.contains(&"theirs line".to_string()));
    }

    #[test]
    fn test_no_conflict_text() {
        let p = parse_conflict_markers("no conflicts here\n");
        assert!(!p.has_conflicts());
    }
}
