// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! SMIL (Synchronized Multimedia Integration Language) stub export.

/// A SMIL media element (audio/video/image/text reference).
#[derive(Debug, Clone)]
pub struct SmilMedia {
    pub tag: String,
    pub src: String,
    pub begin_ms: u64,
    pub dur_ms: u64,
    pub region: Option<String>,
}

/// A SMIL region definition.
#[derive(Debug, Clone)]
pub struct SmilRegion {
    pub id: String,
    pub top: String,
    pub left: String,
    pub width: String,
    pub height: String,
}

/// A SMIL document.
#[derive(Debug, Clone, Default)]
pub struct SmilDocument {
    pub title: String,
    pub regions: Vec<SmilRegion>,
    pub media: Vec<SmilMedia>,
}

impl SmilDocument {
    /// Add a video media element.
    pub fn add_video(&mut self, src: impl Into<String>, begin_ms: u64, dur_ms: u64) {
        self.media.push(SmilMedia {
            tag: "video".into(),
            src: src.into(),
            begin_ms,
            dur_ms,
            region: None,
        });
    }

    /// Add an audio media element.
    pub fn add_audio(&mut self, src: impl Into<String>, begin_ms: u64, dur_ms: u64) {
        self.media.push(SmilMedia {
            tag: "audio".into(),
            src: src.into(),
            begin_ms,
            dur_ms,
            region: None,
        });
    }

    /// Number of media elements.
    pub fn media_count(&self) -> usize {
        self.media.len()
    }

    /// Total presentation duration.
    pub fn total_duration_ms(&self) -> u64 {
        self.media
            .iter()
            .map(|m| m.begin_ms + m.dur_ms)
            .max()
            .unwrap_or(0)
    }
}

/// Render a SMIL document as XML string.
pub fn render_smil(doc: &SmilDocument) -> String {
    let mut out = format!(
        "<smil xmlns=\"http://www.w3.org/2001/SMIL20/Language\">\n  <head>\n    <meta name=\"title\" content=\"{}\"/>\n  </head>\n  <body>\n    <par>\n",
        doc.title
    );
    for m in &doc.media {
        let region_attr = m
            .region
            .as_deref()
            .map(|r| format!(" region=\"{r}\""))
            .unwrap_or_default();
        out.push_str(&format!(
            "      <{} src=\"{}\" begin=\"{}ms\" dur=\"{}ms\"{}/>\n",
            m.tag, m.src, m.begin_ms, m.dur_ms, region_attr
        ));
    }
    out.push_str("    </par>\n  </body>\n</smil>\n");
    out
}

/// Validate that all media have non-zero duration.
pub fn validate_smil(doc: &SmilDocument) -> bool {
    doc.media.iter().all(|m| m.dur_ms > 0 && !m.src.is_empty())
}

/// Format milliseconds as a SMIL clock value string.
pub fn ms_to_smil_clock(ms: u64) -> String {
    format!("{}ms", ms)
}

/// Add a default fullscreen region.
pub fn add_fullscreen_region(doc: &mut SmilDocument) {
    doc.regions.push(SmilRegion {
        id: "fullscreen".into(),
        top: "0%".into(),
        left: "0%".into(),
        width: "100%".into(),
        height: "100%".into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> SmilDocument {
        let mut d = SmilDocument {
            title: "Test SMIL".into(),
            ..Default::default()
        };
        d.add_video("clip.mp4", 0, 5000);
        d.add_audio("track.mp3", 0, 5000);
        d
    }

    #[test]
    fn media_count() {
        assert_eq!(sample_doc().media_count(), 2);
    }

    #[test]
    fn total_duration() {
        assert_eq!(sample_doc().total_duration_ms(), 5000);
    }

    #[test]
    fn render_contains_smil_tag() {
        assert!(render_smil(&sample_doc()).contains("<smil"));
    }

    #[test]
    fn render_contains_video() {
        assert!(render_smil(&sample_doc()).contains("<video"));
    }

    #[test]
    fn render_contains_audio() {
        assert!(render_smil(&sample_doc()).contains("<audio"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_smil(&sample_doc()));
    }

    #[test]
    fn validate_zero_duration() {
        let mut d = SmilDocument::default();
        d.add_video("x.mp4", 0, 0);
        assert!(!validate_smil(&d));
    }

    #[test]
    fn ms_to_smil_clock_format() {
        assert_eq!(ms_to_smil_clock(1500), "1500ms");
    }

    #[test]
    fn add_fullscreen_region_adds_one() {
        let mut d = SmilDocument::default();
        add_fullscreen_region(&mut d);
        assert_eq!(d.regions.len(), 1);
    }

    #[test]
    fn fullscreen_region_id() {
        let mut d = SmilDocument::default();
        add_fullscreen_region(&mut d);
        assert_eq!(d.regions[0].id, "fullscreen");
    }
}
