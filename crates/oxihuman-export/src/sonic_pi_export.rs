// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Sonic Pi Ruby stub export.

/// A Sonic Pi synthesizer preset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SonicPiSynth {
    Beep,
    Saw,
    Tri,
    Pulse,
    Piano,
    Other(String),
}

impl SonicPiSynth {
    pub fn name(&self) -> String {
        match self {
            SonicPiSynth::Beep => ":beep".to_string(),
            SonicPiSynth::Saw => ":saw".to_string(),
            SonicPiSynth::Tri => ":tri".to_string(),
            SonicPiSynth::Pulse => ":pulse".to_string(),
            SonicPiSynth::Piano => ":piano".to_string(),
            SonicPiSynth::Other(s) => format!(":{}", s),
        }
    }
}

/// A single note in a Sonic Pi program.
#[derive(Debug, Clone)]
pub struct SonicPiNote {
    pub synth: SonicPiSynth,
    pub note: i32,
    pub duration: f64,
    pub amplitude: f64,
}

impl SonicPiNote {
    pub fn new(synth: SonicPiSynth, note: i32, duration: f64, amplitude: f64) -> Self {
        Self {
            synth,
            note,
            duration,
            amplitude,
        }
    }

    pub fn to_ruby_line(&self) -> String {
        format!(
            "use_synth {}\nplay {}, amp: {}, release: {}",
            self.synth.name(),
            self.note,
            self.amplitude,
            self.duration
        )
    }
}

/// A Sonic Pi program (sequence of live_loop blocks and note sequences).
#[derive(Debug, Clone, Default)]
pub struct SonicPiProgram {
    pub notes: Vec<SonicPiNote>,
    pub bpm: f64,
    pub loop_name: String,
}

impl SonicPiProgram {
    pub fn new(bpm: f64, loop_name: impl Into<String>) -> Self {
        Self {
            notes: Vec::new(),
            bpm,
            loop_name: loop_name.into(),
        }
    }

    pub fn add_note(&mut self, note: SonicPiNote) {
        self.notes.push(note);
    }
}

/// Generate Sonic Pi Ruby source code from a program.
pub fn generate_sonic_pi_source(prog: &SonicPiProgram) -> String {
    let mut src = String::new();
    src.push_str("# Auto-generated Sonic Pi program\n");
    src.push_str(&format!("use_bpm {}\n\n", prog.bpm));
    src.push_str(&format!("live_loop :{} do\n", prog.loop_name));
    for note in &prog.notes {
        for line in note.to_ruby_line().lines() {
            src.push_str(&format!("  {}\n", line));
        }
        src.push_str(&format!("  sleep {}\n", note.duration));
    }
    src.push_str("end\n");
    src
}

/// Count notes in a Sonic Pi program.
pub fn count_sonic_pi_notes(prog: &SonicPiProgram) -> usize {
    prog.notes.len()
}

/// Check if a source string is a valid Sonic Pi program stub.
pub fn is_valid_sonic_pi(src: &str) -> bool {
    src.contains("live_loop") || src.contains("play ")
}

/// Build a simple scale melody for Sonic Pi.
pub fn scale_melody(
    synth: SonicPiSynth,
    base_note: i32,
    length: usize,
    bpm: f64,
) -> SonicPiProgram {
    let mut prog = SonicPiProgram::new(bpm, "melody");
    for i in 0..length {
        prog.add_note(SonicPiNote::new(
            synth.clone(),
            base_note + i as i32,
            0.5,
            0.8,
        ));
    }
    prog
}

/// Compute total duration of a program in beats.
pub fn program_duration_beats(prog: &SonicPiProgram) -> f64 {
    prog.notes.iter().map(|n| n.duration).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synth_name() {
        assert_eq!(SonicPiSynth::Beep.name(), ":beep" /* beep synth */);
        assert_eq!(SonicPiSynth::Piano.name(), ":piano" /* piano synth */);
    }

    #[test]
    fn test_other_synth() {
        let s = SonicPiSynth::Other("chiplead".to_string());
        assert_eq!(s.name(), ":chiplead" /* custom synth name */);
    }

    #[test]
    fn test_note_to_ruby_line() {
        let note = SonicPiNote::new(SonicPiSynth::Beep, 60, 0.5, 1.0);
        let line = note.to_ruby_line();
        assert!(line.contains("play 60") /* correct MIDI note */);
        assert!(line.contains(":beep") /* synth name */);
    }

    #[test]
    fn test_generate_source_has_live_loop() {
        let prog = SonicPiProgram::new(120.0, "main");
        let src = generate_sonic_pi_source(&prog);
        assert!(is_valid_sonic_pi(&src) /* has live_loop */);
    }

    #[test]
    fn test_generate_source_bpm() {
        let prog = SonicPiProgram::new(90.0, "beat");
        let src = generate_sonic_pi_source(&prog);
        assert!(src.contains("use_bpm 90") /* BPM in source */);
    }

    #[test]
    fn test_count_notes_empty() {
        let prog = SonicPiProgram::new(120.0, "x");
        assert_eq!(count_sonic_pi_notes(&prog), 0 /* empty program */);
    }

    #[test]
    fn test_scale_melody_length() {
        let prog = scale_melody(SonicPiSynth::Beep, 60, 8, 120.0);
        assert_eq!(count_sonic_pi_notes(&prog), 8 /* 8 notes in scale */);
    }

    #[test]
    fn test_program_duration_beats() {
        let prog = scale_melody(SonicPiSynth::Saw, 48, 4, 120.0);
        let dur = program_duration_beats(&prog);
        assert!((dur - 2.0).abs() < 1e-5 /* 4 × 0.5 beats = 2.0 */);
    }

    #[test]
    fn test_is_valid_sonic_pi_false() {
        assert!(!is_valid_sonic_pi("# empty file") /* not a valid program */);
    }
}
