// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! MIDI file export (Type 0, single track, note events).

/// A MIDI note event.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MidiNote {
    pub tick_on: u32,
    pub tick_off: u32,
    pub channel: u8,
    pub pitch: u8,
    pub velocity: u8,
}

/// A MIDI export container.
#[allow(dead_code)]
pub struct MidiExport {
    pub tempo_bpm: u32,
    pub ticks_per_beat: u16,
    pub notes: Vec<MidiNote>,
}

impl MidiExport {
    #[allow(dead_code)]
    pub fn new(tempo_bpm: u32, ticks_per_beat: u16) -> Self {
        Self {
            tempo_bpm,
            ticks_per_beat,
            notes: Vec::new(),
        }
    }
}

/// Add a note to the MIDI export.
#[allow(dead_code)]
pub fn add_note(midi: &mut MidiExport, note: MidiNote) {
    midi.notes.push(note);
}

/// Encode a variable-length quantity (VLQ).
#[allow(dead_code)]
pub fn encode_vlq(value: u32) -> Vec<u8> {
    let mut out = Vec::new();
    let mut v = value;
    let first = true;
    let mut bytes = Vec::new();
    loop {
        bytes.push((v & 0x7F) as u8);
        v >>= 7;
        if v == 0 {
            break;
        }
    }
    bytes.reverse();
    for (i, &b) in bytes.iter().enumerate() {
        if i < bytes.len() - 1 {
            out.push(b | 0x80);
        } else {
            out.push(b);
        }
    }
    if out.is_empty() && first {
        out.push(0);
    }
    let _ = first;
    out
}

/// Build MIDI file header chunk.
#[allow(dead_code)]
pub fn build_midi_header(midi: &MidiExport) -> Vec<u8> {
    let mut hdr = Vec::new();
    hdr.extend_from_slice(b"MThd");
    hdr.extend_from_slice(&6u32.to_be_bytes());
    hdr.extend_from_slice(&0u16.to_be_bytes());
    hdr.extend_from_slice(&1u16.to_be_bytes());
    hdr.extend_from_slice(&midi.ticks_per_beat.to_be_bytes());
    hdr
}

/// Build MIDI track chunk from notes.
#[allow(dead_code)]
pub fn build_midi_track(midi: &MidiExport) -> Vec<u8> {
    let mut events: Vec<(u32, Vec<u8>)> = Vec::new();
    let us_per_beat = 60_000_000 / midi.tempo_bpm.max(1);
    let tempo_event: Vec<u8> = {
        let mut e = Vec::new();
        e.extend_from_slice(&encode_vlq(0));
        e.push(0xFF);
        e.push(0x51);
        e.push(0x03);
        e.extend_from_slice(&us_per_beat.to_be_bytes()[1..]);
        e
    };
    events.push((0, tempo_event));
    let mut sorted = midi.notes.clone();
    sorted.sort_by_key(|n| n.tick_on);
    for note in &sorted {
        let on_ev = vec![
            0x90 | (note.channel & 0x0F),
            note.pitch & 0x7F,
            note.velocity & 0x7F,
        ];
        let off_ev = vec![0x80 | (note.channel & 0x0F), note.pitch & 0x7F, 0x40];
        events.push((note.tick_on, on_ev));
        events.push((note.tick_off, off_ev));
    }
    events.sort_by_key(|e| e.0);
    let mut track_data = Vec::new();
    let mut prev_tick = 0u32;
    for (tick, ev) in &events {
        let delta = tick.saturating_sub(prev_tick);
        track_data.extend_from_slice(&encode_vlq(delta));
        track_data.extend_from_slice(ev);
        prev_tick = *tick;
    }
    track_data.extend_from_slice(&encode_vlq(0));
    track_data.extend_from_slice(&[0xFF, 0x2F, 0x00]);
    let mut chunk = Vec::new();
    chunk.extend_from_slice(b"MTrk");
    chunk.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    chunk.extend_from_slice(&track_data);
    chunk
}

/// Export MIDI to bytes.
#[allow(dead_code)]
pub fn export_midi(midi: &MidiExport) -> Vec<u8> {
    let mut out = build_midi_header(midi);
    out.extend_from_slice(&build_midi_track(midi));
    out
}

/// Duration in ticks.
#[allow(dead_code)]
pub fn midi_duration_ticks(midi: &MidiExport) -> u32 {
    midi.notes.iter().map(|n| n.tick_off).max().unwrap_or(0)
}

/// Duration in seconds.
#[allow(dead_code)]
pub fn midi_duration_secs(midi: &MidiExport) -> f32 {
    let ticks = midi_duration_ticks(midi) as f32;
    let beats_per_sec = midi.tempo_bpm as f32 / 60.0;
    ticks / (midi.ticks_per_beat as f32 * beats_per_sec)
}

/// Note count.
#[allow(dead_code)]
pub fn midi_note_count(midi: &MidiExport) -> usize {
    midi.notes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_midi() -> MidiExport {
        let mut m = MidiExport::new(120, 480);
        add_note(
            &mut m,
            MidiNote {
                tick_on: 0,
                tick_off: 480,
                channel: 0,
                pitch: 60,
                velocity: 100,
            },
        );
        add_note(
            &mut m,
            MidiNote {
                tick_on: 480,
                tick_off: 960,
                channel: 0,
                pitch: 64,
                velocity: 80,
            },
        );
        m
    }

    #[test]
    fn header_starts_with_mthd() {
        let m = simple_midi();
        let hdr = build_midi_header(&m);
        assert_eq!(&hdr[0..4], b"MThd");
    }

    #[test]
    fn header_length_14() {
        let m = simple_midi();
        let hdr = build_midi_header(&m);
        // MThd(4) + chunk_len(4) + format(2) + num_tracks(2) + ticks_per_beat(2) = 14
        assert_eq!(hdr.len(), 14);
    }

    #[test]
    fn track_starts_with_mtrk() {
        let m = simple_midi();
        let trk = build_midi_track(&m);
        assert_eq!(&trk[0..4], b"MTrk");
    }

    #[test]
    fn export_midi_nonempty() {
        let m = simple_midi();
        let out = export_midi(&m);
        assert!(out.len() > 10);
    }

    #[test]
    fn midi_note_count_correct() {
        let m = simple_midi();
        assert_eq!(midi_note_count(&m), 2);
    }

    #[test]
    fn midi_duration_ticks_correct() {
        let m = simple_midi();
        assert_eq!(midi_duration_ticks(&m), 960);
    }

    #[test]
    fn midi_duration_secs_positive() {
        let m = simple_midi();
        let d = midi_duration_secs(&m);
        assert!(d > 0.0);
    }

    #[test]
    fn vlq_zero() {
        let v = encode_vlq(0);
        assert_eq!(v, vec![0]);
    }

    #[test]
    fn vlq_128() {
        let v = encode_vlq(128);
        assert_eq!(v, vec![0x81, 0x00]);
    }

    #[test]
    fn empty_midi_exports() {
        let m = MidiExport::new(120, 480);
        let out = export_midi(&m);
        assert!(out.len() >= 10);
    }
}
