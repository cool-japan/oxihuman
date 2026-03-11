// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! WebVTT subtitle export.

/// A single WebVTT cue.
#[derive(Debug, Clone)]
pub struct VttCue {
    /// Optional cue identifier.
    pub id: Option<String>,
    /// Start time in milliseconds.
    pub start_ms: u64,
    /// End time in milliseconds.
    pub end_ms: u64,
    /// Cue text.
    pub text: String,
    /// Optional positioning settings string.
    pub settings: Option<String>,
}

/// A WebVTT document.
#[derive(Debug, Clone, Default)]
pub struct VttDocument {
    pub cues: Vec<VttCue>,
    /// Optional file metadata/notes.
    pub header_note: Option<String>,
}

impl VttDocument {
    /// Add a cue.
    pub fn add_cue(&mut self, start_ms: u64, end_ms: u64, text: impl Into<String>) {
        self.cues.push(VttCue {
            id: None,
            start_ms,
            end_ms,
            text: text.into(),
            settings: None,
        });
    }

    /// Number of cues.
    pub fn cue_count(&self) -> usize {
        self.cues.len()
    }
}

/// Format ms as WebVTT timestamp `HH:MM:SS.mmm`.
pub fn ms_to_vtt_time(ms: u64) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1_000;
    let ms = ms % 1_000;
    format!("{h:02}:{m:02}:{s:02}.{ms:03}")
}

/// Render a VttDocument to a String.
pub fn render_vtt(doc: &VttDocument) -> String {
    let mut out = String::from("WEBVTT\n");
    if let Some(note) = &doc.header_note {
        out.push('\n');
        out.push_str("NOTE ");
        out.push_str(note);
        out.push('\n');
    }
    out.push('\n');
    for cue in &doc.cues {
        if let Some(id) = &cue.id {
            out.push_str(id);
            out.push('\n');
        }
        let mut timing = format!(
            "{} --> {}",
            ms_to_vtt_time(cue.start_ms),
            ms_to_vtt_time(cue.end_ms)
        );
        if let Some(settings) = &cue.settings {
            timing.push(' ');
            timing.push_str(settings);
        }
        out.push_str(&timing);
        out.push('\n');
        out.push_str(&cue.text);
        out.push_str("\n\n");
    }
    out
}

/// Validate that all cues have start < end.
pub fn validate_vtt(doc: &VttDocument) -> bool {
    doc.cues
        .iter()
        .all(|c| c.start_ms < c.end_ms && !c.text.is_empty())
}

/// Longest cue text length.
pub fn max_cue_length(doc: &VttDocument) -> usize {
    doc.cues.iter().map(|c| c.text.len()).max().unwrap_or(0)
}

/// Total duration (end of last cue).
pub fn total_duration_ms(doc: &VttDocument) -> u64 {
    doc.cues.iter().map(|c| c.end_ms).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> VttDocument {
        let mut d = VttDocument::default();
        d.add_cue(0, 2000, "Hello");
        d.add_cue(3000, 5000, "World");
        d
    }

    #[test]
    fn cue_count() {
        assert_eq!(sample_doc().cue_count(), 2);
    }

    #[test]
    fn webvtt_header_present() {
        /* rendered output starts with WEBVTT */
        let s = render_vtt(&sample_doc());
        assert!(s.starts_with("WEBVTT"));
    }

    #[test]
    fn ms_to_vtt_time_format() {
        /* 3723456 ms → 01:02:03.456 */
        assert_eq!(ms_to_vtt_time(3_723_456), "01:02:03.456");
    }

    #[test]
    fn arrow_in_output() {
        assert!(render_vtt(&sample_doc()).contains("-->"));
    }

    #[test]
    fn text_in_output() {
        assert!(render_vtt(&sample_doc()).contains("Hello"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_vtt(&sample_doc()));
    }

    #[test]
    fn validate_bad_timing() {
        let mut d = VttDocument::default();
        d.cues.push(VttCue {
            id: None,
            start_ms: 5000,
            end_ms: 3000,
            text: "bad".into(),
            settings: None,
        });
        assert!(!validate_vtt(&d));
    }

    #[test]
    fn total_duration() {
        assert_eq!(total_duration_ms(&sample_doc()), 5000);
    }

    #[test]
    fn max_cue_length_nonzero() {
        assert!(max_cue_length(&sample_doc()) > 0);
    }

    #[test]
    fn empty_max_length() {
        assert_eq!(max_cue_length(&VttDocument::default()), 0);
    }
}
