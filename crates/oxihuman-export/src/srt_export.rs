// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! SRT subtitle export.

/// A single SRT subtitle entry.
#[derive(Debug, Clone)]
pub struct SrtEntry {
    /// 1-based sequence number.
    pub index: u32,
    /// Start time in milliseconds.
    pub start_ms: u64,
    /// End time in milliseconds.
    pub end_ms: u64,
    /// Subtitle text lines.
    pub text: Vec<String>,
}

/// An SRT document.
#[derive(Debug, Clone, Default)]
pub struct SrtDocument {
    pub entries: Vec<SrtEntry>,
}

impl SrtDocument {
    /// Add a subtitle entry.
    pub fn add_entry(&mut self, start_ms: u64, end_ms: u64, text: impl Into<String>) {
        let index = self.entries.len() as u32 + 1;
        self.entries.push(SrtEntry {
            index,
            start_ms,
            end_ms,
            text: vec![text.into()],
        });
    }

    /// Number of entries.
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

/// Format milliseconds as SRT timestamp `HH:MM:SS,mmm`.
pub fn ms_to_srt_time(ms: u64) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1_000;
    let ms = ms % 1_000;
    format!("{h:02}:{m:02}:{s:02},{ms:03}")
}

/// Render an SrtDocument to a String in SRT format.
pub fn render_srt(doc: &SrtDocument) -> String {
    let mut out = String::new();
    for entry in &doc.entries {
        out.push_str(&format!("{}\n", entry.index));
        out.push_str(&format!(
            "{} --> {}\n",
            ms_to_srt_time(entry.start_ms),
            ms_to_srt_time(entry.end_ms)
        ));
        for line in &entry.text {
            out.push_str(line);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

/// Validate that all entries have increasing timestamps and non-empty text.
pub fn validate_srt(doc: &SrtDocument) -> bool {
    doc.entries
        .iter()
        .all(|e| e.start_ms < e.end_ms && !e.text.is_empty())
}

/// Total duration of the subtitle file in milliseconds.
pub fn total_duration_ms(doc: &SrtDocument) -> u64 {
    doc.entries.iter().map(|e| e.end_ms).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> SrtDocument {
        let mut doc = SrtDocument::default();
        doc.add_entry(0, 2000, "Hello world");
        doc.add_entry(3000, 5000, "Second line");
        doc
    }

    #[test]
    fn entry_count_correct() {
        /* two entries */
        assert_eq!(sample_doc().entry_count(), 2);
    }

    #[test]
    fn indices_start_at_one() {
        /* first entry index = 1 */
        assert_eq!(sample_doc().entries[0].index, 1);
    }

    #[test]
    fn ms_to_srt_time_format() {
        /* 3723456 ms → 01:02:03,456 */
        assert_eq!(ms_to_srt_time(3_723_456), "01:02:03,456");
    }

    #[test]
    fn ms_to_srt_zero() {
        assert_eq!(ms_to_srt_time(0), "00:00:00,000");
    }

    #[test]
    fn render_contains_arrow() {
        /* rendered output contains --> */
        let s = render_srt(&sample_doc());
        assert!(s.contains("-->"));
    }

    #[test]
    fn render_contains_text() {
        let s = render_srt(&sample_doc());
        assert!(s.contains("Hello world"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_srt(&sample_doc()));
    }

    #[test]
    fn validate_bad_timestamps() {
        /* start >= end is invalid */
        let mut doc = SrtDocument::default();
        doc.entries.push(SrtEntry {
            index: 1,
            start_ms: 5000,
            end_ms: 3000,
            text: vec!["bad".to_string()],
        });
        assert!(!validate_srt(&doc));
    }

    #[test]
    fn total_duration_correct() {
        assert_eq!(total_duration_ms(&sample_doc()), 5000);
    }

    #[test]
    fn empty_doc_duration_zero() {
        assert_eq!(total_duration_ms(&SrtDocument::default()), 0);
    }
}
