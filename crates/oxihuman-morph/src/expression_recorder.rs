// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Record and playback expression state over time.

#[allow(dead_code)]
#[derive(Clone)]
pub struct ExpressionSnapshot {
    pub time: f32,
    pub weights: Vec<(String, f32)>,
    pub metadata: String,
}

#[allow(dead_code)]
pub struct ExpressionRecording {
    pub name: String,
    pub snapshots: Vec<ExpressionSnapshot>,
    pub fps: f32,
    pub looping: bool,
}

#[allow(dead_code)]
pub struct RecorderState {
    pub recording: bool,
    pub playing: bool,
    pub current_time: f32,
    pub playback_speed: f32,
}

#[allow(dead_code)]
pub fn new_recording(name: &str, fps: f32) -> ExpressionRecording {
    ExpressionRecording {
        name: name.to_string(),
        snapshots: Vec::new(),
        fps,
        looping: false,
    }
}

#[allow(dead_code)]
pub fn record_snapshot(rec: &mut ExpressionRecording, time: f32, weights: Vec<(String, f32)>) {
    rec.snapshots.push(ExpressionSnapshot {
        time,
        weights,
        metadata: String::new(),
    });
    rec.snapshots.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[allow(dead_code)]
pub fn playback_at(rec: &ExpressionRecording, time: f32) -> Vec<(String, f32)> {
    if rec.snapshots.is_empty() {
        return Vec::new();
    }
    if rec.snapshots.len() == 1 {
        return rec.snapshots[0].weights.clone();
    }
    // Find surrounding snapshots
    let before = rec.snapshots.iter().rfind(|s| s.time <= time);
    let after = rec.snapshots.iter().find(|s| s.time > time);
    match (before, after) {
        (Some(b), Some(a)) => {
            let span = a.time - b.time;
            let t = if span > 1e-8 {
                (time - b.time) / span
            } else {
                0.0
            };
            interpolate_weights(&b.weights, &a.weights, t)
        }
        (Some(b), None) => b.weights.clone(),
        (None, Some(a)) => a.weights.clone(),
        (None, None) => Vec::new(),
    }
}

fn interpolate_weights(a: &[(String, f32)], b: &[(String, f32)], t: f32) -> Vec<(String, f32)> {
    let mut result: Vec<(String, f32)> = Vec::new();
    for (name, wa) in a {
        let wb = b
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, w)| *w)
            .unwrap_or(0.0);
        result.push((name.clone(), wa + (wb - wa) * t));
    }
    for (name, wb) in b {
        if !result.iter().any(|(n, _)| n == name) {
            result.push((name.clone(), *wb * t));
        }
    }
    result
}

#[allow(dead_code)]
pub fn recording_duration(rec: &ExpressionRecording) -> f32 {
    if rec.snapshots.is_empty() {
        return 0.0;
    }
    let first = rec.snapshots.first().map_or(0.0, |s| s.time);
    let last = rec.snapshots.last().map_or(0.0, |s| s.time);
    last - first
}

#[allow(dead_code)]
pub fn snapshot_count(rec: &ExpressionRecording) -> usize {
    rec.snapshots.len()
}

#[allow(dead_code)]
pub fn trim_recording(rec: &mut ExpressionRecording, start: f32, end: f32) {
    rec.snapshots.retain(|s| s.time >= start && s.time <= end);
}

#[allow(dead_code)]
pub fn reverse_recording(rec: &mut ExpressionRecording) {
    if rec.snapshots.is_empty() {
        return;
    }
    let max_t = rec.snapshots[rec.snapshots.len() - 1].time;
    for snap in &mut rec.snapshots {
        snap.time = max_t - snap.time;
    }
    rec.snapshots.reverse();
}

#[allow(dead_code)]
pub fn scale_recording_time(rec: &mut ExpressionRecording, factor: f32) {
    for snap in &mut rec.snapshots {
        snap.time *= factor;
    }
}

#[allow(dead_code)]
pub fn new_recorder_state() -> RecorderState {
    RecorderState {
        recording: false,
        playing: false,
        current_time: 0.0,
        playback_speed: 1.0,
    }
}

#[allow(dead_code)]
pub fn start_recording(state: &mut RecorderState) {
    state.recording = true;
    state.playing = false;
}

#[allow(dead_code)]
pub fn stop_recording(state: &mut RecorderState) {
    state.recording = false;
}

#[allow(dead_code)]
pub fn start_playback(state: &mut RecorderState) {
    state.playing = true;
    state.recording = false;
}

#[allow(dead_code)]
pub fn stop_playback(state: &mut RecorderState) {
    state.playing = false;
}

#[allow(dead_code)]
pub fn advance_playback(
    state: &mut RecorderState,
    rec: &ExpressionRecording,
    dt: f32,
) -> Vec<(String, f32)> {
    if !state.playing {
        return Vec::new();
    }
    state.current_time += dt * state.playback_speed;
    let duration = recording_duration(rec);
    if rec.looping && duration > 0.0 {
        state.current_time %= duration;
    } else if state.current_time > duration {
        state.current_time = duration;
        state.playing = false;
    }
    playback_at(rec, state.current_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_recording() {
        let rec = new_recording("test", 30.0);
        assert_eq!(rec.name, "test");
        assert_eq!(rec.fps, 30.0);
        assert!(!rec.looping);
        assert!(rec.snapshots.is_empty());
    }

    #[test]
    fn test_record_snapshot() {
        let mut rec = new_recording("r", 24.0);
        record_snapshot(&mut rec, 0.0, vec![("smile".to_string(), 0.5)]);
        record_snapshot(&mut rec, 1.0, vec![("smile".to_string(), 1.0)]);
        assert_eq!(snapshot_count(&rec), 2);
    }

    #[test]
    fn test_snapshot_count() {
        let mut rec = new_recording("r", 24.0);
        assert_eq!(snapshot_count(&rec), 0);
        record_snapshot(&mut rec, 0.0, vec![]);
        assert_eq!(snapshot_count(&rec), 1);
    }

    #[test]
    fn test_recording_duration() {
        let mut rec = new_recording("r", 24.0);
        assert!((recording_duration(&rec)).abs() < 1e-5);
        record_snapshot(&mut rec, 0.0, vec![]);
        record_snapshot(&mut rec, 2.0, vec![]);
        assert!((recording_duration(&rec) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_playback_at_empty() {
        let rec = new_recording("r", 24.0);
        let result = playback_at(&rec, 0.5);
        assert!(result.is_empty());
    }

    #[test]
    fn test_playback_at_interpolates() {
        let mut rec = new_recording("r", 24.0);
        record_snapshot(&mut rec, 0.0, vec![("brow".to_string(), 0.0)]);
        record_snapshot(&mut rec, 1.0, vec![("brow".to_string(), 1.0)]);
        let weights = playback_at(&rec, 0.5);
        let brow = weights
            .iter()
            .find(|(n, _)| n == "brow")
            .expect("should succeed");
        assert!((brow.1 - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_playback_at_before_start() {
        let mut rec = new_recording("r", 24.0);
        record_snapshot(&mut rec, 1.0, vec![("x".to_string(), 0.8)]);
        let weights = playback_at(&rec, 0.0);
        assert_eq!(weights.len(), 1);
    }

    #[test]
    fn test_trim_recording() {
        let mut rec = new_recording("r", 24.0);
        for i in 0..5 {
            record_snapshot(&mut rec, i as f32, vec![]);
        }
        trim_recording(&mut rec, 1.0, 3.0);
        assert_eq!(snapshot_count(&rec), 3);
    }

    #[test]
    fn test_reverse_recording() {
        let mut rec = new_recording("r", 24.0);
        record_snapshot(&mut rec, 0.0, vec![("a".to_string(), 0.0)]);
        record_snapshot(&mut rec, 1.0, vec![("a".to_string(), 1.0)]);
        reverse_recording(&mut rec);
        assert!((rec.snapshots[0].time).abs() < 1e-5);
        assert!((rec.snapshots[1].time - 1.0).abs() < 1e-5);
        assert!((rec.snapshots[0].weights[0].1 - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_scale_recording_time() {
        let mut rec = new_recording("r", 24.0);
        record_snapshot(&mut rec, 1.0, vec![]);
        record_snapshot(&mut rec, 2.0, vec![]);
        scale_recording_time(&mut rec, 2.0);
        assert!((rec.snapshots[0].time - 2.0).abs() < 1e-5);
        assert!((rec.snapshots[1].time - 4.0).abs() < 1e-5);
    }

    #[test]
    fn test_new_recorder_state() {
        let state = new_recorder_state();
        assert!(!state.recording);
        assert!(!state.playing);
        assert!((state.current_time).abs() < 1e-5);
        assert!((state.playback_speed - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_recorder_state_transitions() {
        let mut state = new_recorder_state();
        start_recording(&mut state);
        assert!(state.recording);
        stop_recording(&mut state);
        assert!(!state.recording);
        start_playback(&mut state);
        assert!(state.playing);
        stop_playback(&mut state);
        assert!(!state.playing);
    }

    #[test]
    fn test_start_recording_stops_playback() {
        let mut state = new_recorder_state();
        start_playback(&mut state);
        start_recording(&mut state);
        assert!(!state.playing);
        assert!(state.recording);
    }

    #[test]
    fn test_advance_playback() {
        let mut rec = new_recording("r", 24.0);
        record_snapshot(&mut rec, 0.0, vec![("x".to_string(), 0.0)]);
        record_snapshot(&mut rec, 2.0, vec![("x".to_string(), 1.0)]);
        let mut state = new_recorder_state();
        start_playback(&mut state);
        let weights = advance_playback(&mut state, &rec, 1.0);
        assert!(!weights.is_empty());
        assert!((state.current_time - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_advance_playback_not_playing() {
        let rec = new_recording("r", 24.0);
        let mut state = new_recorder_state();
        let weights = advance_playback(&mut state, &rec, 1.0);
        assert!(weights.is_empty());
    }
}
