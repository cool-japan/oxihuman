// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Advanced SubStation Alpha (ASS/SSA) subtitle export.

/// An ASS dialogue line.
#[derive(Debug, Clone)]
pub struct AssDialogue {
    /// Layer number.
    pub layer: u32,
    /// Start time in centiseconds.
    pub start_cs: u64,
    /// End time in centiseconds.
    pub end_cs: u64,
    /// Style name.
    pub style: String,
    /// Speaker name (optional).
    pub name: String,
    /// Subtitle text (may include ASS override tags).
    pub text: String,
}

/// An ASS style definition.
#[derive(Debug, Clone)]
pub struct AssStyle {
    pub name: String,
    pub fontname: String,
    pub fontsize: u32,
    pub primary_colour: u32,
}

/// A complete ASS document.
#[derive(Debug, Clone, Default)]
pub struct AssDocument {
    pub styles: Vec<AssStyle>,
    pub dialogues: Vec<AssDialogue>,
    pub title: String,
}

impl AssDocument {
    /// Add a dialogue entry.
    pub fn add_dialogue(&mut self, start_cs: u64, end_cs: u64, text: impl Into<String>) {
        self.dialogues.push(AssDialogue {
            layer: 0,
            start_cs,
            end_cs,
            style: "Default".to_string(),
            name: String::new(),
            text: text.into(),
        });
    }

    /// Number of dialogue lines.
    pub fn dialogue_count(&self) -> usize {
        self.dialogues.len()
    }
}

/// Format centiseconds as ASS timestamp `H:MM:SS.cc`.
pub fn cs_to_ass_time(cs: u64) -> String {
    let h = cs / 360_000;
    let m = (cs % 360_000) / 6_000;
    let s = (cs % 6_000) / 100;
    let c = cs % 100;
    format!("{h}:{m:02}:{s:02}.{c:02}")
}

/// Render the ASS document header.
pub fn render_script_info(doc: &AssDocument) -> String {
    format!(
        "[Script Info]\nTitle: {}\nScriptType: v4.00+\n\n",
        doc.title
    )
}

/// Render all dialogue lines.
pub fn render_dialogues(doc: &AssDocument) -> String {
    let mut out = String::from("[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
    for d in &doc.dialogues {
        out.push_str(&format!(
            "Dialogue: {},{},{},{},{},0000,0000,0000,,{}\n",
            d.layer,
            cs_to_ass_time(d.start_cs),
            cs_to_ass_time(d.end_cs),
            d.style,
            d.name,
            d.text,
        ));
    }
    out
}

/// Render the full ASS document.
pub fn render_ass(doc: &AssDocument) -> String {
    render_script_info(doc) + &render_dialogues(doc)
}

/// Validate that all dialogues have valid timings.
pub fn validate_ass(doc: &AssDocument) -> bool {
    doc.dialogues
        .iter()
        .all(|d| d.start_cs < d.end_cs && !d.text.is_empty())
}

/// Maximum dialogue end time.
pub fn total_duration_cs(doc: &AssDocument) -> u64 {
    doc.dialogues.iter().map(|d| d.end_cs).max().unwrap_or(0)
}

/// Default ASS style definition.
pub fn default_style() -> AssStyle {
    AssStyle {
        name: "Default".to_string(),
        fontname: "Arial".to_string(),
        fontsize: 20,
        primary_colour: 0x00FFFFFF,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_doc() -> AssDocument {
        let mut d = AssDocument {
            title: "Test".into(),
            ..Default::default()
        };
        d.add_dialogue(0, 200, "Hello ASS");
        d.add_dialogue(300, 500, "Second line");
        d
    }

    #[test]
    fn dialogue_count() {
        assert_eq!(sample_doc().dialogue_count(), 2);
    }

    #[test]
    fn cs_to_ass_time_format() {
        /* 0 cs → 0:00:00.00 */
        assert_eq!(cs_to_ass_time(0), "0:00:00.00");
    }

    #[test]
    fn cs_to_ass_time_nonzero() {
        /* 360000 cs = 1 hour → 1:00:00.00 */
        assert_eq!(cs_to_ass_time(360_000), "1:00:00.00");
    }

    #[test]
    fn render_script_info_header() {
        let s = render_script_info(&sample_doc());
        assert!(s.contains("[Script Info]"));
    }

    #[test]
    fn render_dialogues_contains_event() {
        let s = render_dialogues(&sample_doc());
        assert!(s.contains("Dialogue:"));
    }

    #[test]
    fn render_ass_complete() {
        let s = render_ass(&sample_doc());
        assert!(s.contains("[Script Info]"));
        assert!(s.contains("[Events]"));
    }

    #[test]
    fn validate_ok() {
        assert!(validate_ass(&sample_doc()));
    }

    #[test]
    fn validate_bad() {
        let mut d = AssDocument::default();
        d.dialogues.push(AssDialogue {
            layer: 0,
            start_cs: 500,
            end_cs: 100,
            style: "Default".into(),
            name: String::new(),
            text: "bad".into(),
        });
        assert!(!validate_ass(&d));
    }

    #[test]
    fn total_duration_correct() {
        assert_eq!(total_duration_cs(&sample_doc()), 500);
    }

    #[test]
    fn default_style_name() {
        assert_eq!(default_style().name, "Default");
    }
}
