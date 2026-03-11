// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! MIDI clip sequence stub export.

/// A single MIDI note event.
#[derive(Debug, Clone, Copy)]
pub struct MidiNote {
    pub channel: u8,
    pub pitch: u8,
    pub velocity: u8,
    pub start_tick: u32,
    pub duration_ticks: u32,
}

impl MidiNote {
    pub fn new(channel: u8, pitch: u8, velocity: u8, start_tick: u32, duration_ticks: u32) -> Self {
        Self {
            channel: channel & 0x0F,
            pitch: pitch & 0x7F,
            velocity: velocity & 0x7F,
            start_tick,
            duration_ticks,
        }
    }
}

/// A MIDI clip containing multiple notes.
#[derive(Debug, Clone, Default)]
pub struct MidiClip {
    pub notes: Vec<MidiNote>,
    pub tempo_bpm: f64,
    pub ticks_per_beat: u16,
}

impl MidiClip {
    pub fn new(tempo_bpm: f64, ticks_per_beat: u16) -> Self {
        Self {
            notes: Vec::new(),
            tempo_bpm,
            ticks_per_beat,
        }
    }

    pub fn add_note(&mut self, note: MidiNote) {
        self.notes.push(note);
    }

    pub fn duration_ticks(&self) -> u32 {
        self.notes
            .iter()
            .map(|n| n.start_tick + n.duration_ticks)
            .max()
            .unwrap_or(0)
    }
}

/// MIDI clip export config.
#[derive(Debug, Clone)]
pub struct MidiClipExportConfig {
    pub format: u16,
    pub track_count: u16,
}

impl Default for MidiClipExportConfig {
    fn default() -> Self {
        Self {
            format: 0,
            track_count: 1,
        }
    }
}

/// Write a MIDI variable-length quantity (VLQ) encoding.
pub fn encode_vlq(value: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut v = value;
    bytes.push((v & 0x7F) as u8);
    v >>= 7;
    while v > 0 {
        bytes.push(((v & 0x7F) | 0x80) as u8);
        v >>= 7;
    }
    bytes.reverse();
    bytes
}

/// Build a MIDI note-on event bytes.
pub fn midi_note_on(channel: u8, pitch: u8, velocity: u8) -> [u8; 3] {
    [0x90 | (channel & 0x0F), pitch & 0x7F, velocity & 0x7F]
}

/// Build a MIDI note-off event bytes.
pub fn midi_note_off(channel: u8, pitch: u8) -> [u8; 3] {
    [0x80 | (channel & 0x0F), pitch & 0x7F, 0x00]
}

/// Export a MIDI clip to a minimal standard MIDI file byte stream.
pub fn export_midi_clip(clip: &MidiClip, _cfg: &MidiClipExportConfig) -> Vec<u8> {
    /* Stub: build a minimal MIDI file with note-on/off events */
    let mut buf = Vec::new();
    /* MThd header */
    buf.extend_from_slice(b"MThd");
    buf.extend_from_slice(&6u32.to_be_bytes()); /* chunk length */
    buf.extend_from_slice(&0u16.to_be_bytes()); /* format 0 */
    buf.extend_from_slice(&1u16.to_be_bytes()); /* 1 track */
    buf.extend_from_slice(&clip.ticks_per_beat.to_be_bytes());

    /* MTrk track chunk (stub: empty track body + end-of-track) */
    let track_body: Vec<u8> = {
        let mut t = Vec::new();
        let mut last_tick = 0u32;
        let mut events: Vec<(u32, Vec<u8>)> = Vec::new();

        for note in &clip.notes {
            let on_bytes = midi_note_on(note.channel, note.pitch, note.velocity).to_vec();
            events.push((note.start_tick, on_bytes));
            let off_bytes = midi_note_off(note.channel, note.pitch).to_vec();
            events.push((note.start_tick + note.duration_ticks, off_bytes));
        }
        events.sort_by_key(|(tick, _)| *tick);

        for (tick, ev) in events {
            let delta = tick.saturating_sub(last_tick);
            t.extend_from_slice(&encode_vlq(delta));
            t.extend_from_slice(&ev);
            last_tick = tick;
        }
        /* End-of-track meta event */
        t.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);
        t
    };

    buf.extend_from_slice(b"MTrk");
    buf.extend_from_slice(&(track_body.len() as u32).to_be_bytes());
    buf.extend_from_slice(&track_body);
    buf
}

/// Validate that exported bytes start with the MIDI header magic.
pub fn validate_midi_header(data: &[u8]) -> bool {
    data.len() >= 14 && &data[0..4] == b"MThd"
}

/// Compute total duration in seconds for a MIDI clip.
pub fn clip_duration_secs(clip: &MidiClip) -> f64 {
    if clip.tempo_bpm <= 0.0 || clip.ticks_per_beat == 0 {
        return 0.0;
    }
    let total_ticks = clip.duration_ticks() as f64;
    let secs_per_beat = 60.0 / clip.tempo_bpm;
    total_ticks / clip.ticks_per_beat as f64 * secs_per_beat
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_note_on_byte() {
        let on = midi_note_on(0, 60, 100);
        assert_eq!(on[0], 0x90 /* note-on channel 0 */);
        assert_eq!(on[1], 60 /* middle C */);
    }

    #[test]
    fn test_midi_note_off_byte() {
        let off = midi_note_off(0, 60);
        assert_eq!(off[0], 0x80 /* note-off channel 0 */);
        assert_eq!(off[2], 0 /* velocity 0 */);
    }

    #[test]
    fn test_encode_vlq_single_byte() {
        assert_eq!(encode_vlq(0x7F), vec![0x7F] /* fits in one byte */);
    }

    #[test]
    fn test_encode_vlq_two_bytes() {
        let vlq = encode_vlq(128);
        assert_eq!(vlq.len(), 2 /* 128 requires 2 VLQ bytes */);
        assert_eq!(vlq[0], 0x81 /* high byte */);
        assert_eq!(vlq[1], 0x00 /* low byte */);
    }

    #[test]
    fn test_clip_empty_duration() {
        let clip = MidiClip::new(120.0, 480);
        assert_eq!(clip.duration_ticks(), 0 /* empty clip */);
    }

    #[test]
    fn test_clip_add_note() {
        let mut clip = MidiClip::new(120.0, 480);
        clip.add_note(MidiNote::new(0, 60, 100, 0, 480));
        assert_eq!(clip.notes.len(), 1 /* one note added */);
    }

    #[test]
    fn test_export_midi_clip_header() {
        let clip = MidiClip::new(120.0, 480);
        let cfg = MidiClipExportConfig::default();
        let bytes = export_midi_clip(&clip, &cfg);
        assert!(validate_midi_header(&bytes) /* valid MIDI header */);
    }

    #[test]
    fn test_clip_duration_secs() {
        let mut clip = MidiClip::new(120.0, 480);
        clip.add_note(MidiNote::new(0, 60, 100, 0, 480)); /* one beat at 120 BPM */
        let dur = clip_duration_secs(&clip);
        assert!((dur - 0.5).abs() < 0.01 /* 1 beat at 120 BPM = 0.5 s */);
    }

    #[test]
    fn test_midi_note_channel_masked() {
        let note = MidiNote::new(15, 64, 80, 0, 100);
        assert_eq!(note.channel, 15 /* channel 15 valid */);
        let note2 = MidiNote::new(20, 64, 80, 0, 100); /* channel 20 → masked to 4 */
        assert!(note2.channel <= 15 /* channel clamped to 4 bits */);
    }
}
