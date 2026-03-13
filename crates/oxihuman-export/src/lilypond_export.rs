// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! LilyPond music notation stub export.

/// LilyPond pitch names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LyPitch {
    C,
    D,
    E,
    F,
    G,
    A,
    B,
}

impl LyPitch {
    pub fn lily_name(&self) -> &'static str {
        match self {
            LyPitch::C => "c",
            LyPitch::D => "d",
            LyPitch::E => "e",
            LyPitch::F => "f",
            LyPitch::G => "g",
            LyPitch::A => "a",
            LyPitch::B => "b",
        }
    }
}

/// LilyPond note duration (as integer denominator).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LyDuration {
    Whole,     /* 1 */
    Half,      /* 2 */
    Quarter,   /* 4 */
    Eighth,    /* 8 */
    Sixteenth, /* 16 */
}

impl LyDuration {
    pub fn lily_name(&self) -> &'static str {
        match self {
            LyDuration::Whole => "1",
            LyDuration::Half => "2",
            LyDuration::Quarter => "4",
            LyDuration::Eighth => "8",
            LyDuration::Sixteenth => "16",
        }
    }

    pub fn beats(&self) -> f64 {
        match self {
            LyDuration::Whole => 4.0,
            LyDuration::Half => 2.0,
            LyDuration::Quarter => 1.0,
            LyDuration::Eighth => 0.5,
            LyDuration::Sixteenth => 0.25,
        }
    }
}

/// A LilyPond note.
#[derive(Debug, Clone)]
pub struct LyNote {
    pub pitch: LyPitch,
    pub octave: i32,
    pub duration: LyDuration,
    pub dotted: bool,
}

impl LyNote {
    pub fn new(pitch: LyPitch, octave: i32, duration: LyDuration) -> Self {
        Self {
            pitch,
            octave,
            duration,
            dotted: false,
        }
    }

    pub fn to_lily_token(&self) -> String {
        /* Octave: ' for each above 4, , for each below 4 */
        let base = self.pitch.lily_name();
        let oct_offset = self.octave - 4;
        let oct_str = if oct_offset >= 0 {
            "'".repeat(oct_offset as usize)
        } else {
            ",".repeat((-oct_offset) as usize)
        };
        let dot = if self.dotted { "." } else { "" };
        format!("{}{}{}{}", base, oct_str, self.duration.lily_name(), dot)
    }
}

/// A LilyPond score staff.
#[derive(Debug, Clone, Default)]
pub struct LyStaff {
    pub notes: Vec<LyNote>,
    pub time_sig: (u32, u32),
    pub clef: String,
    pub instrument_name: String,
}

impl LyStaff {
    pub fn new(clef: impl Into<String>, time_sig: (u32, u32)) -> Self {
        Self {
            clef: clef.into(),
            time_sig,
            notes: Vec::new(),
            instrument_name: String::new(),
        }
    }

    pub fn add_note(&mut self, note: LyNote) {
        self.notes.push(note);
    }
}

/// Generate LilyPond source code from a staff.
pub fn generate_lilypond_source(staff: &LyStaff, title: &str, composer: &str) -> String {
    let mut src = String::new();
    src.push_str("\\version \"2.24.0\"\n");
    src.push_str(&format!(
        "\\header {{\n  title = \"{}\"\n  composer = \"{}\"\n}}\n\n",
        title, composer
    ));
    src.push_str("\\score {\n  \\new Staff {\n");
    src.push_str(&format!("    \\clef {}\n", staff.clef));
    src.push_str(&format!(
        "    \\time {}/{}\n    ",
        staff.time_sig.0, staff.time_sig.1
    ));
    let tokens: Vec<String> = staff.notes.iter().map(|n| n.to_lily_token()).collect();
    src.push_str(&tokens.join(" "));
    src.push_str("\n  }\n}\n");
    src
}

/// Count notes in a staff.
pub fn count_lily_notes(staff: &LyStaff) -> usize {
    staff.notes.len()
}

/// Compute total duration of notes in a staff in beats.
pub fn staff_duration_beats(staff: &LyStaff) -> f64 {
    staff
        .notes
        .iter()
        .map(|n| {
            let base = n.duration.beats();
            if n.dotted {
                base * 1.5
            } else {
                base
            }
        })
        .sum()
}

/// Build a simple C major scale staff.
pub fn c_major_scale_staff() -> LyStaff {
    let mut staff = LyStaff::new("treble", (4, 4));
    let pitches = [
        LyPitch::C,
        LyPitch::D,
        LyPitch::E,
        LyPitch::F,
        LyPitch::G,
        LyPitch::A,
        LyPitch::B,
        LyPitch::C,
    ];
    for &p in &pitches {
        staff.add_note(LyNote::new(p, 4, LyDuration::Quarter));
    }
    staff
}

/// Validate that a string looks like a LilyPond source file.
pub fn is_valid_lilypond(src: &str) -> bool {
    src.contains("\\version") && src.contains("\\score")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_name() {
        assert_eq!(LyPitch::C.lily_name(), "c" /* C note */);
        assert_eq!(LyPitch::A.lily_name(), "a" /* A note */);
    }

    #[test]
    fn test_duration_name() {
        assert_eq!(LyDuration::Quarter.lily_name(), "4" /* quarter note */);
        assert_eq!(LyDuration::Whole.lily_name(), "1" /* whole note */);
    }

    #[test]
    fn test_duration_beats() {
        assert_eq!(LyDuration::Half.beats(), 2.0 /* half = 2 beats */);
        assert_eq!(
            LyDuration::Eighth.beats(),
            0.5 /* eighth = 0.5 beats */
        );
    }

    #[test]
    fn test_note_token_middle_c() {
        let note = LyNote::new(LyPitch::C, 4, LyDuration::Quarter);
        let token = note.to_lily_token();
        assert!(token.contains('c') /* C note */);
        assert!(token.contains('4') /* quarter */);
    }

    #[test]
    fn test_note_token_octave_above() {
        let note = LyNote::new(LyPitch::C, 5, LyDuration::Quarter);
        let token = note.to_lily_token();
        assert!(token.contains('\'') /* one octave up */);
    }

    #[test]
    fn test_generate_lilypond_valid() {
        let staff = c_major_scale_staff();
        let src = generate_lilypond_source(&staff, "Test", "Composer");
        assert!(is_valid_lilypond(&src) /* valid LilyPond source */);
    }

    #[test]
    fn test_count_lily_notes() {
        let staff = c_major_scale_staff();
        assert_eq!(
            count_lily_notes(&staff),
            8 /* 8 notes in C major scale */
        );
    }

    #[test]
    fn test_staff_duration_beats() {
        let staff = c_major_scale_staff();
        let dur = staff_duration_beats(&staff);
        assert!((dur - 8.0).abs() < 1e-5 /* 8 quarter notes = 8 beats */);
    }

    #[test]
    fn test_is_valid_lilypond_false() {
        assert!(!is_valid_lilypond("not lilypond") /* invalid */);
    }
}
