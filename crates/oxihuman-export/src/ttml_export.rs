// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! TTML (Timed Text Markup Language) subtitle export.

/// A TTML span (inline styled text).
#[derive(Debug, Clone)]
pub struct TtmlSpan {
    pub text: String,
    pub style_id: Option<String>,
}

/// A TTML paragraph (block-level subtitle).
#[derive(Debug, Clone)]
pub struct TtmlParagraph {
    /// Start time in milliseconds.
    pub begin_ms: u64,
    /// End time in milliseconds.
    pub end_ms: u64,
    pub spans: Vec<TtmlSpan>,
    pub region: Option<String>,
}

impl TtmlParagraph {
    /// Plain text content of this paragraph.
    pub fn plain_text(&self) -> String {
        self.spans
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A TTML document.
#[derive(Debug, Clone, Default)]
pub struct TtmlDocument {
    pub paragraphs: Vec<TtmlParagraph>,
    pub lang: String,
}

impl TtmlDocument {
    /// Add a simple paragraph.
    pub fn add_paragraph(&mut self, begin_ms: u64, end_ms: u64, text: impl Into<String>) {
        self.paragraphs.push(TtmlParagraph {
            begin_ms,
            end_ms,
            spans: vec![TtmlSpan {
                text: text.into(),
                style_id: None,
            }],
            region: None,
        });
    }

    /// Paragraph count.
    pub fn paragraph_count(&self) -> usize {
        self.paragraphs.len()
    }
}

/// Format milliseconds as TTML time expression `HH:MM:SS.mmm`.
pub fn ms_to_ttml_time(ms: u64) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1_000;
    let ms = ms % 1_000;
    format!("{h:02}:{m:02}:{s:02}.{ms:03}")
}

/// Build a minimal TTML XML string.
pub fn render_ttml(doc: &TtmlDocument) -> String {
    let lang = if doc.lang.is_empty() { "en" } else { &doc.lang };
    let mut out = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xml:lang="{lang}" xmlns="http://www.w3.org/ns/ttml">
  <body>
    <div>
"#
    );
    for p in &doc.paragraphs {
        out.push_str(&format!(
            "      <p begin=\"{}\" end=\"{}\">{}</p>\n",
            ms_to_ttml_time(p.begin_ms),
            ms_to_ttml_time(p.end_ms),
            p.plain_text()
        ));
    }
    out.push_str("    </div>\n  </body>\n</tt>\n");
    out
}

/// Validate that all paragraphs have non-zero duration and text.
pub fn validate_ttml(doc: &TtmlDocument) -> bool {
    doc.paragraphs
        .iter()
        .all(|p| p.begin_ms < p.end_ms && !p.plain_text().is_empty())
}

/// Total subtitle duration.
pub fn total_duration_ms(doc: &TtmlDocument) -> u64 {
    doc.paragraphs.iter().map(|p| p.end_ms).max().unwrap_or(0)
}

/// Escape XML special characters in a string.
pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> TtmlDocument {
        let mut d = TtmlDocument {
            lang: "en".into(),
            ..Default::default()
        };
        d.add_paragraph(0, 2000, "Hello TTML");
        d.add_paragraph(3000, 6000, "Second");
        d
    }

    #[test]
    fn paragraph_count() {
        assert_eq!(sample_doc().paragraph_count(), 2);
    }

    #[test]
    fn ms_to_ttml_format() {
        assert_eq!(ms_to_ttml_time(3_723_456), "01:02:03.456");
    }

    #[test]
    fn render_ttml_starts_with_xml() {
        let s = render_ttml(&sample_doc());
        assert!(s.starts_with("<?xml"));
    }

    #[test]
    fn render_ttml_contains_tt_tag() {
        assert!(render_ttml(&sample_doc()).contains("<tt"));
    }

    #[test]
    fn render_ttml_contains_text() {
        assert!(render_ttml(&sample_doc()).contains("Hello TTML"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_ttml(&sample_doc()));
    }

    #[test]
    fn validate_bad_timing() {
        let mut d = TtmlDocument::default();
        d.add_paragraph(5000, 1000, "bad");
        assert!(!validate_ttml(&d));
    }

    #[test]
    fn total_duration() {
        assert_eq!(total_duration_ms(&sample_doc()), 6000);
    }

    #[test]
    fn xml_escape_ampersand() {
        assert_eq!(xml_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn xml_escape_lt() {
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
    }
}
