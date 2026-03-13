// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Guitar Pro tab stub export.

/// Tuning for a single guitar string (MIDI note number for open string).
#[derive(Debug, Clone, Copy)]
pub struct StringTuning(pub u8);

impl StringTuning {
    pub fn standard_guitar() -> [StringTuning; 6] {
        /* E2 A2 D3 G3 B3 E4 */
        [
            StringTuning(40),
            StringTuning(45),
            StringTuning(50),
            StringTuning(55),
            StringTuning(59),
            StringTuning(64),
        ]
    }
}

/// A Guitar Pro note on a specific string and fret.
#[derive(Debug, Clone, Copy)]
pub struct GpNote {
    pub string_idx: u8,
    pub fret: u8,
    pub duration_ticks: u32,
    pub tied: bool,
}

impl GpNote {
    pub fn new(string_idx: u8, fret: u8, duration_ticks: u32) -> Self {
        Self {
            string_idx,
            fret,
            duration_ticks,
            tied: false,
        }
    }

    pub fn midi_pitch(&self, tuning: &[StringTuning]) -> Option<u8> {
        /* MIDI pitch = open string MIDI + fret */
        tuning
            .get(self.string_idx as usize)
            .map(|t| t.0.saturating_add(self.fret))
    }
}

/// A Guitar Pro beat (chord or single note).
#[derive(Debug, Clone, Default)]
pub struct GpBeat {
    pub notes: Vec<GpNote>,
    pub duration_ticks: u32,
}

impl GpBeat {
    pub fn new(duration_ticks: u32) -> Self {
        Self {
            duration_ticks,
            notes: Vec::new(),
        }
    }

    pub fn add_note(&mut self, note: GpNote) {
        self.notes.push(note);
    }
}

/// A Guitar Pro measure.
#[derive(Debug, Clone, Default)]
pub struct GpMeasure {
    pub beats: Vec<GpBeat>,
    pub time_sig_num: u8,
    pub time_sig_den: u8,
}

impl GpMeasure {
    pub fn new(time_sig_num: u8, time_sig_den: u8) -> Self {
        Self {
            time_sig_num,
            time_sig_den,
            beats: Vec::new(),
        }
    }

    pub fn add_beat(&mut self, beat: GpBeat) {
        self.beats.push(beat);
    }
}

/// A Guitar Pro track.
#[derive(Debug, Clone, Default)]
pub struct GpTrack {
    pub name: String,
    pub tuning: Vec<StringTuning>,
    pub measures: Vec<GpMeasure>,
    pub capo_fret: u8,
}

impl GpTrack {
    pub fn new(name: impl Into<String>, tuning: Vec<StringTuning>) -> Self {
        Self {
            name: name.into(),
            tuning,
            measures: Vec::new(),
            capo_fret: 0,
        }
    }

    pub fn add_measure(&mut self, measure: GpMeasure) {
        self.measures.push(measure);
    }
}

/// A Guitar Pro song document.
#[derive(Debug, Clone, Default)]
pub struct GpSong {
    pub title: String,
    pub artist: String,
    pub tempo: u32,
    pub tracks: Vec<GpTrack>,
}

impl GpSong {
    pub fn new(title: impl Into<String>, artist: impl Into<String>, tempo: u32) -> Self {
        Self {
            title: title.into(),
            artist: artist.into(),
            tempo,
            tracks: Vec::new(),
        }
    }

    pub fn add_track(&mut self, track: GpTrack) {
        self.tracks.push(track);
    }
}

/// Export a Guitar Pro song to a stub text representation.
pub fn export_gp_stub(song: &GpSong) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "GP_STUB title=\"{}\" artist=\"{}\" tempo={}\n",
        song.title, song.artist, song.tempo
    ));
    for (ti, track) in song.tracks.iter().enumerate() {
        out.push_str(&format!("  TRACK {} name=\"{}\"\n", ti, track.name));
        for (mi, measure) in track.measures.iter().enumerate() {
            out.push_str(&format!(
                "    MEASURE {} sig={}/{}\n",
                mi, measure.time_sig_num, measure.time_sig_den
            ));
            for beat in &measure.beats {
                for note in &beat.notes {
                    out.push_str(&format!(
                        "      NOTE s={} f={} dur={}\n",
                        note.string_idx, note.fret, note.duration_ticks
                    ));
                }
            }
        }
    }
    out
}

/// Validate that a string looks like a GP stub export.
pub fn is_gp_stub(src: &str) -> bool {
    src.starts_with("GP_STUB")
}

/// Count total notes across all tracks.
pub fn count_gp_notes(song: &GpSong) -> usize {
    song.tracks
        .iter()
        .flat_map(|t| t.measures.iter())
        .flat_map(|m| m.beats.iter())
        .flat_map(|b| b.notes.iter())
        .count()
}

/// Build a simple single-note riff.
pub fn single_note_riff(fret: u8, count: usize) -> GpTrack {
    let tuning = StringTuning::standard_guitar().to_vec();
    let mut track = GpTrack::new("Guitar", tuning);
    let mut measure = GpMeasure::new(4, 4);
    for _ in 0..count {
        let mut beat = GpBeat::new(480);
        beat.add_note(GpNote::new(0, fret, 480));
        measure.add_beat(beat);
    }
    track.add_measure(measure);
    track
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_tuning_count() {
        let tuning = StringTuning::standard_guitar();
        assert_eq!(tuning.len(), 6 /* 6-string guitar */);
    }

    #[test]
    fn test_midi_pitch_open_string() {
        let tuning = StringTuning::standard_guitar().to_vec();
        let note = GpNote::new(0, 0, 480); /* open low E */
        assert_eq!(note.midi_pitch(&tuning), Some(40) /* E2 = MIDI 40 */);
    }

    #[test]
    fn test_midi_pitch_fret() {
        let tuning = StringTuning::standard_guitar().to_vec();
        let note = GpNote::new(0, 5, 480); /* 5th fret = A2 */
        assert_eq!(note.midi_pitch(&tuning), Some(45) /* A2 = MIDI 45 */);
    }

    #[test]
    fn test_export_gp_stub_header() {
        let song = GpSong::new("Test", "Artist", 120);
        let out = export_gp_stub(&song);
        assert!(is_gp_stub(&out) /* GP stub header */);
    }

    #[test]
    fn test_export_gp_stub_title() {
        let song = GpSong::new("My Song", "Me", 100);
        let out = export_gp_stub(&song);
        assert!(out.contains("My Song") /* title in output */);
    }

    #[test]
    fn test_count_gp_notes_empty() {
        let song = GpSong::new("x", "y", 120);
        assert_eq!(count_gp_notes(&song), 0 /* no notes */);
    }

    #[test]
    fn test_single_note_riff() {
        let track = single_note_riff(0, 4);
        let mut song = GpSong::new("Riff", "Me", 120);
        song.add_track(track);
        assert_eq!(count_gp_notes(&song), 4 /* 4 notes */);
    }

    #[test]
    fn test_is_gp_stub_false() {
        assert!(!is_gp_stub("not a guitar pro file") /* invalid */);
    }

    #[test]
    fn test_gp_song_tempo() {
        let song = GpSong::new("Test", "A", 180);
        assert_eq!(song.tempo, 180 /* correct tempo */);
    }
}
