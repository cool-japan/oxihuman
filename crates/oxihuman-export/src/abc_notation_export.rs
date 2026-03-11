// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! ABC music notation stub export.

/// ABC notation tune fields.
#[derive(Debug, Clone, Default)]
pub struct AbcTuneHeader {
    pub index: u32,
    pub title: String,
    pub composer: String,
    pub meter: String,
    pub default_note_length: String,
    pub tempo: String,
    pub key: String,
}

impl AbcTuneHeader {
    pub fn new(
        index: u32,
        title: impl Into<String>,
        composer: impl Into<String>,
        meter: impl Into<String>,
        key: impl Into<String>,
    ) -> Self {
        Self {
            index,
            title: title.into(),
            composer: composer.into(),
            meter: meter.into(),
            default_note_length: "1/4".to_string(),
            tempo: "120".to_string(),
            key: key.into(),
        }
    }

    pub fn to_abc_header(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("X:{}\n", self.index));
        s.push_str(&format!("T:{}\n", self.title));
        if !self.composer.is_empty() {
            s.push_str(&format!("C:{}\n", self.composer));
        }
        s.push_str(&format!("M:{}\n", self.meter));
        s.push_str(&format!("L:{}\n", self.default_note_length));
        s.push_str(&format!("Q:{}\n", self.tempo));
        s.push_str(&format!("K:{}\n", self.key));
        s
    }
}

/// A single ABC note token.
#[derive(Debug, Clone)]
pub struct AbcNote {
    pub pitch: String,
    pub duration_modifier: String,
}

impl AbcNote {
    pub fn new(pitch: impl Into<String>) -> Self {
        Self {
            pitch: pitch.into(),
            duration_modifier: String::new(),
        }
    }

    pub fn with_duration(mut self, modifier: impl Into<String>) -> Self {
        self.duration_modifier = modifier.into();
        self
    }

    pub fn to_abc_token(&self) -> String {
        format!("{}{}", self.pitch, self.duration_modifier)
    }
}

/// An ABC tune body (sequence of bars).
#[derive(Debug, Clone, Default)]
pub struct AbcTuneBody {
    pub bars: Vec<Vec<AbcNote>>,
}

impl AbcTuneBody {
    pub fn new() -> Self {
        Self { bars: Vec::new() }
    }

    pub fn add_bar(&mut self, notes: Vec<AbcNote>) {
        self.bars.push(notes);
    }

    pub fn to_abc_body(&self) -> String {
        self.bars
            .iter()
            .map(|bar| {
                let tokens: String = bar
                    .iter()
                    .map(|n| n.to_abc_token())
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{} |", tokens)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// A complete ABC tune (header + body).
#[derive(Debug, Clone, Default)]
pub struct AbcTune {
    pub header: AbcTuneHeader,
    pub body: AbcTuneBody,
}

impl AbcTune {
    pub fn new(header: AbcTuneHeader) -> Self {
        Self {
            header,
            body: AbcTuneBody::new(),
        }
    }
}

/// Generate ABC notation source from a tune.
pub fn generate_abc_notation(tune: &AbcTune) -> String {
    format!(
        "{}\n{}\n",
        tune.header.to_abc_header(),
        tune.body.to_abc_body()
    )
}

/// Validate that a string looks like ABC notation.
pub fn is_valid_abc(src: &str) -> bool {
    src.contains("X:") && src.contains("T:") && src.contains("K:")
}

/// Count total notes across all bars.
pub fn count_abc_notes(body: &AbcTuneBody) -> usize {
    body.bars.iter().map(|b| b.len()).sum()
}

/// Build a simple C major scale in ABC notation.
pub fn c_major_scale_abc() -> AbcTune {
    let header = AbcTuneHeader::new(1, "C Major Scale", "", "4/4", "C");
    let mut tune = AbcTune::new(header);
    let notes = ["C", "D", "E", "F", "G", "A", "B", "c"];
    let bar1: Vec<AbcNote> = notes[0..4].iter().map(|&n| AbcNote::new(n)).collect();
    let bar2: Vec<AbcNote> = notes[4..8].iter().map(|&n| AbcNote::new(n)).collect();
    tune.body.add_bar(bar1);
    tune.body.add_bar(bar2);
    tune
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_x_field() {
        let h = AbcTuneHeader::new(1, "Test", "", "4/4", "G");
        let abc = h.to_abc_header();
        assert!(abc.contains("X:1") /* tune index */);
    }

    #[test]
    fn test_header_key_field() {
        let h = AbcTuneHeader::new(1, "Test", "", "4/4", "D");
        let abc = h.to_abc_header();
        assert!(abc.contains("K:D") /* key of D */);
    }

    #[test]
    fn test_abc_note_token() {
        let note = AbcNote::new("C").with_duration("2");
        assert_eq!(note.to_abc_token(), "C2" /* half note C */);
    }

    #[test]
    fn test_abc_note_no_modifier() {
        let note = AbcNote::new("G");
        assert_eq!(note.to_abc_token(), "G" /* quarter note G */);
    }

    #[test]
    fn test_generate_abc_valid() {
        let tune = c_major_scale_abc();
        let abc = generate_abc_notation(&tune);
        assert!(is_valid_abc(&abc) /* valid ABC notation */);
    }

    #[test]
    fn test_count_abc_notes() {
        let tune = c_major_scale_abc();
        assert_eq!(count_abc_notes(&tune.body), 8 /* 8 notes in C major */);
    }

    #[test]
    fn test_is_valid_abc_false() {
        assert!(!is_valid_abc("not abc notation") /* invalid */);
    }

    #[test]
    fn test_body_contains_bar_separator() {
        let tune = c_major_scale_abc();
        let body = tune.body.to_abc_body();
        assert!(body.contains('|') /* bar lines present */);
    }

    #[test]
    fn test_composer_in_header() {
        let h = AbcTuneHeader::new(1, "Test", "J.S. Bach", "3/4", "Am");
        let abc = h.to_abc_header();
        assert!(abc.contains("C:J.S. Bach") /* composer field */);
    }
}
