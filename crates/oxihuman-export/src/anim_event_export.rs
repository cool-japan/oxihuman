// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Export animation event markers (e.g. footstep, sound trigger) to JSON-like records.

/// A single animation event.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AnimEvent {
    pub name: String,
    pub time: f32,
    pub payload: String,
}

/// A collection of animation events.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct AnimEventTrack {
    pub events: Vec<AnimEvent>,
}

/// Create a new event track.
#[allow(dead_code)]
pub fn new_event_track() -> AnimEventTrack {
    AnimEventTrack::default()
}

/// Add an event to the track.
#[allow(dead_code)]
pub fn add_event(track: &mut AnimEventTrack, name: &str, time: f32, payload: &str) {
    track.events.push(AnimEvent {
        name: name.to_string(),
        time,
        payload: payload.to_string(),
    });
}

/// Sort events by time.
#[allow(dead_code)]
pub fn sort_events(track: &mut AnimEventTrack) {
    track.events.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Count events in the track.
#[allow(dead_code)]
pub fn event_count(track: &AnimEventTrack) -> usize {
    track.events.len()
}

/// Find all events at or after `time`.
#[allow(dead_code)]
pub fn events_from(track: &AnimEventTrack, time: f32) -> Vec<&AnimEvent> {
    track.events.iter().filter(|e| e.time >= time).collect()
}

/// Duration from first to last event (0 if fewer than 2 events).
#[allow(dead_code)]
pub fn track_duration(track: &AnimEventTrack) -> f32 {
    if track.events.len() < 2 {
        return 0.0;
    }
    let first = track.events.iter().map(|e| e.time).fold(f32::MAX, f32::min);
    let last = track.events.iter().map(|e| e.time).fold(f32::MIN, f32::max);
    last - first
}

/// Serialize events to a simple newline-separated string.
#[allow(dead_code)]
pub fn serialize_events(track: &AnimEventTrack) -> String {
    track
        .events
        .iter()
        .map(|e| format!("{},{},{}", e.name, e.time, e.payload))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Check whether the track has any events with the given name.
#[allow(dead_code)]
pub fn has_event_named(track: &AnimEventTrack, name: &str) -> bool {
    track.events.iter().any(|e| e.name == name)
}

/// Remove all events with time less than `cutoff`.
#[allow(dead_code)]
pub fn trim_before(track: &mut AnimEventTrack, cutoff: f32) {
    track.events.retain(|e| e.time >= cutoff);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_event() {
        let mut t = new_event_track();
        add_event(&mut t, "step", 1.0, "left");
        assert_eq!(event_count(&t), 1);
    }

    #[test]
    fn test_sort_events() {
        let mut t = new_event_track();
        add_event(&mut t, "b", 2.0, "");
        add_event(&mut t, "a", 0.5, "");
        sort_events(&mut t);
        assert!((t.events[0].time - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_events_from() {
        let mut t = new_event_track();
        add_event(&mut t, "x", 0.0, "");
        add_event(&mut t, "y", 1.5, "");
        let r = events_from(&t, 1.0);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn test_track_duration_empty() {
        let t = new_event_track();
        assert_eq!(track_duration(&t), 0.0);
    }

    #[test]
    fn test_track_duration() {
        let mut t = new_event_track();
        add_event(&mut t, "a", 1.0, "");
        add_event(&mut t, "b", 3.0, "");
        assert!((track_duration(&t) - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_serialize_events() {
        let mut t = new_event_track();
        add_event(&mut t, "step", 1.0, "L");
        let s = serialize_events(&t);
        assert!(s.contains("step"));
    }

    #[test]
    fn test_has_event_named() {
        let mut t = new_event_track();
        add_event(&mut t, "jump", 0.5, "");
        assert!(has_event_named(&t, "jump"));
        assert!(!has_event_named(&t, "fall"));
    }

    #[test]
    fn test_trim_before() {
        let mut t = new_event_track();
        add_event(&mut t, "a", 0.2, "");
        add_event(&mut t, "b", 1.5, "");
        trim_before(&mut t, 1.0);
        assert_eq!(event_count(&t), 1);
    }

    #[test]
    fn test_empty_serialize() {
        let t = new_event_track();
        assert!(serialize_events(&t).is_empty());
    }

    #[test]
    fn test_event_fields() {
        let e = AnimEvent {
            name: "n".to_string(),
            time: 3.0,
            payload: "p".to_string(),
        };
        assert!((e.time - 3.0).abs() < 1e-6);
    }
}
