// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

/// Interpolation mode between keyframes.
#[derive(Clone, Debug, PartialEq)]
pub enum TrackInterp {
    /// Hold value until next keyframe.
    Step,
    /// Linear interpolation between keyframes.
    Linear,
    /// Smoothstep interpolation between keyframes.
    SmoothStep,
    /// Cubic Catmull-Rom interpolation between keyframes.
    Cubic,
}

/// A single keyframe: time (seconds) and scalar value.
#[derive(Clone, Debug)]
pub struct Keyframe {
    /// Time in seconds.
    pub time: f32,
    /// Scalar value at this keyframe.
    pub value: f32,
}

/// A named animation track holding keyframes for one parameter.
pub struct AnimTrack {
    /// Track name (identifies the morph parameter).
    pub name: String,
    /// Interpolation mode used when evaluating between keyframes.
    pub interp: TrackInterp,
    /// Keyframes sorted by ascending time.
    keyframes: Vec<Keyframe>,
}

impl AnimTrack {
    /// Create a new empty track with the given name and interpolation mode.
    pub fn new(name: impl Into<String>, interp: TrackInterp) -> Self {
        Self {
            name: name.into(),
            interp,
            keyframes: Vec::new(),
        }
    }

    /// Add a keyframe at `time` with `value`.
    /// Keyframes are kept sorted by time; if a keyframe at the exact time already
    /// exists its value is replaced.
    pub fn add_keyframe(&mut self, time: f32, value: f32) {
        // Check for exact match and replace.
        if let Some(kf) = self
            .keyframes
            .iter_mut()
            .find(|k| (k.time - time).abs() < f32::EPSILON)
        {
            kf.value = value;
            return;
        }
        let pos = self.keyframes.partition_point(|k| k.time < time);
        self.keyframes.insert(pos, Keyframe { time, value });
    }

    /// Remove the keyframe whose time exactly matches `time`.
    /// Returns `true` if a keyframe was removed.
    pub fn remove_keyframe(&mut self, time: f32) -> bool {
        if let Some(pos) = self
            .keyframes
            .iter()
            .position(|k| (k.time - time).abs() < f32::EPSILON)
        {
            self.keyframes.remove(pos);
            true
        } else {
            false
        }
    }

    /// Number of keyframes in this track.
    pub fn keyframe_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Slice of all keyframes in time order.
    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }

    /// Time of the last keyframe, or `0.0` if the track is empty.
    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
    }

    /// Evaluate the track at `time` using the configured interpolation mode.
    pub fn evaluate(&self, time: f32) -> f32 {
        if self.keyframes.is_empty() {
            return 0.0;
        }
        let first = &self.keyframes[0];
        let last = &self.keyframes[self.keyframes.len() - 1];

        if time <= first.time {
            return first.value;
        }
        if time >= last.time {
            return last.value;
        }

        // Find bracket [i, i+1] such that keyframes[i].time <= time < keyframes[i+1].time.
        let idx_b = self.keyframes.partition_point(|k| k.time <= time);
        let idx_a = idx_b - 1;

        let ka = &self.keyframes[idx_a];
        let kb = &self.keyframes[idx_b];

        let span = kb.time - ka.time;
        let t = if span > 0.0 {
            (time - ka.time) / span
        } else {
            0.0
        };

        match self.interp {
            TrackInterp::Step => ka.value,
            TrackInterp::Linear => lerp(ka.value, kb.value, t),
            TrackInterp::SmoothStep => {
                let s = t * t * (3.0 - 2.0 * t);
                lerp(ka.value, kb.value, s)
            }
            TrackInterp::Cubic => {
                // Catmull-Rom: use neighbours as tangent guides, clamped at boundaries.
                let n = self.keyframes.len();
                let p0 = if idx_a == 0 {
                    ka.value
                } else {
                    self.keyframes[idx_a - 1].value
                };
                let p3 = if idx_b + 1 >= n {
                    kb.value
                } else {
                    self.keyframes[idx_b + 1].value
                };
                catmull_rom_scalar(p0, ka.value, kb.value, p3, t)
            }
        }
    }

    /// Change the interpolation mode for this track.
    pub fn set_interp(&mut self, interp: TrackInterp) {
        self.interp = interp;
    }
}

// ---------------------------------------------------------------------------
// Helper math
// ---------------------------------------------------------------------------

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Catmull-Rom spline interpolation between p1 and p2 with neighbours p0, p3.
#[inline]
fn catmull_rom_scalar(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

// ---------------------------------------------------------------------------
// Timeline
// ---------------------------------------------------------------------------

/// Multi-track animation timeline.
pub struct Timeline {
    /// Human-readable name for this timeline.
    pub name: String,
    /// Frames per second (used by `frame_count`).
    pub fps: f32,
    tracks: HashMap<String, AnimTrack>,
}

impl Timeline {
    /// Create a new timeline with default fps of 24.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fps: 24.0,
            tracks: HashMap::new(),
        }
    }

    /// Create a new timeline with an explicit fps value.
    pub fn with_fps(name: impl Into<String>, fps: f32) -> Self {
        Self {
            name: name.into(),
            fps,
            tracks: HashMap::new(),
        }
    }

    /// Add a track to the timeline.
    pub fn add_track(&mut self, track: AnimTrack) {
        self.tracks.insert(track.name.clone(), track);
    }

    /// Remove the track named `name`. Returns `true` if a track was removed.
    pub fn remove_track(&mut self, name: &str) -> bool {
        self.tracks.remove(name).is_some()
    }

    /// Get an immutable reference to the named track.
    pub fn get_track(&self, name: &str) -> Option<&AnimTrack> {
        self.tracks.get(name)
    }

    /// Get a mutable reference to the named track.
    pub fn get_track_mut(&mut self, name: &str) -> Option<&mut AnimTrack> {
        self.tracks.get_mut(name)
    }

    /// Sorted list of all track names.
    pub fn track_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.tracks.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// Number of tracks in this timeline.
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Maximum duration across all tracks.
    pub fn duration(&self) -> f32 {
        self.tracks
            .values()
            .map(|t| t.duration())
            .fold(0.0_f32, f32::max)
    }

    /// Number of frames: `ceil(duration * fps)`.
    pub fn frame_count(&self) -> usize {
        (self.duration() * self.fps).ceil() as usize
    }

    /// Evaluate all tracks at `time`, returning a map of track name to value.
    pub fn evaluate(&self, time: f32) -> HashMap<String, f32> {
        self.tracks
            .iter()
            .map(|(name, track)| (name.clone(), track.evaluate(time)))
            .collect()
    }

    /// Sample the timeline at uniform intervals of `1.0 / sample_fps` from `t = 0`
    /// to the timeline duration, returning a `Vec` of name→value snapshots.
    pub fn bake(&self, sample_fps: f32) -> Vec<HashMap<String, f32>> {
        let duration = self.duration();
        if duration <= 0.0 || sample_fps <= 0.0 {
            return Vec::new();
        }
        let dt = 1.0 / sample_fps;
        let mut frames = Vec::new();
        let mut t = 0.0_f32;
        while t <= duration + f32::EPSILON {
            frames.push(self.evaluate(t));
            t += dt;
        }
        frames
    }

    /// Add a keyframe to the named track, creating it (with `Linear` interp) if absent.
    pub fn set_key(&mut self, track_name: &str, time: f32, value: f32) {
        let track = self
            .tracks
            .entry(track_name.to_owned())
            .or_insert_with(|| AnimTrack::new(track_name, TrackInterp::Linear));
        track.add_keyframe(time, value);
    }

    /// Shift all keyframes in every track by `dt` seconds.
    pub fn time_offset(&mut self, dt: f32) {
        for track in self.tracks.values_mut() {
            for kf in track.keyframes.iter_mut() {
                kf.time += dt;
            }
        }
    }

    /// Scale all keyframe times by `factor` (speed up or slow down the timeline).
    pub fn time_scale(&mut self, factor: f32) {
        for track in self.tracks.values_mut() {
            for kf in track.keyframes.iter_mut() {
                kf.time *= factor;
            }
        }
    }
}

impl Default for Timeline {
    fn default() -> Self {
        Self::new("untitled")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- AnimTrack tests ----

    #[test]
    fn test_track_new() {
        let track = AnimTrack::new("jaw_open", TrackInterp::Linear);
        assert_eq!(track.name, "jaw_open");
        assert_eq!(track.interp, TrackInterp::Linear);
        assert_eq!(track.keyframe_count(), 0);
    }

    #[test]
    fn test_track_add_keyframe_sorted() {
        let mut track = AnimTrack::new("t", TrackInterp::Linear);
        track.add_keyframe(2.0, 0.8);
        track.add_keyframe(0.0, 0.0);
        track.add_keyframe(1.0, 0.5);
        let kfs = track.keyframes();
        assert_eq!(kfs.len(), 3);
        assert!((kfs[0].time - 0.0).abs() < f32::EPSILON);
        assert!((kfs[1].time - 1.0).abs() < f32::EPSILON);
        assert!((kfs[2].time - 2.0).abs() < f32::EPSILON);
        // Replace existing key at t=1.0
        track.add_keyframe(1.0, 0.9);
        assert_eq!(track.keyframe_count(), 3);
        assert!((track.keyframes()[1].value - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn test_track_evaluate_empty() {
        let track = AnimTrack::new("empty", TrackInterp::Linear);
        assert!((track.evaluate(0.5) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_track_evaluate_single() {
        let mut track = AnimTrack::new("single", TrackInterp::Linear);
        track.add_keyframe(1.0, 0.42);
        assert!((track.evaluate(0.0) - 0.42).abs() < 1e-6);
        assert!((track.evaluate(1.0) - 0.42).abs() < 1e-6);
        assert!((track.evaluate(5.0) - 0.42).abs() < 1e-6);
    }

    #[test]
    fn test_track_evaluate_linear() {
        let mut track = AnimTrack::new("lin", TrackInterp::Linear);
        track.add_keyframe(0.0, 0.0);
        track.add_keyframe(1.0, 1.0);
        assert!((track.evaluate(0.5) - 0.5).abs() < 1e-6);
        assert!((track.evaluate(0.25) - 0.25).abs() < 1e-6);
        assert!((track.evaluate(0.75) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_track_evaluate_step() {
        let mut track = AnimTrack::new("step", TrackInterp::Step);
        track.add_keyframe(0.0, 0.0);
        track.add_keyframe(1.0, 1.0);
        // Before the second key, should return first key's value.
        assert!((track.evaluate(0.5) - 0.0).abs() < 1e-6);
        // At or after last key, returns last value.
        assert!((track.evaluate(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_track_evaluate_smoothstep() {
        let mut track = AnimTrack::new("smooth", TrackInterp::SmoothStep);
        track.add_keyframe(0.0, 0.0);
        track.add_keyframe(1.0, 1.0);
        // smoothstep(0.5) = 0.5^2 * (3 - 2*0.5) = 0.25 * 2 = 0.5
        let v = track.evaluate(0.5);
        assert!((v - 0.5).abs() < 1e-5);
        // smoothstep is slower at edges; value at 0.25 < 0.25 (linear)
        let v_low = track.evaluate(0.25);
        assert!(v_low < 0.25 + 1e-5);
    }

    #[test]
    fn test_track_duration() {
        let track = AnimTrack::new("empty", TrackInterp::Linear);
        assert!((track.duration() - 0.0).abs() < f32::EPSILON);

        let mut track2 = AnimTrack::new("t2", TrackInterp::Linear);
        track2.add_keyframe(0.0, 0.0);
        track2.add_keyframe(3.5, 1.0);
        assert!((track2.duration() - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_track_remove_keyframe() {
        let mut track = AnimTrack::new("rem", TrackInterp::Linear);
        track.add_keyframe(0.0, 0.0);
        track.add_keyframe(1.0, 1.0);
        assert!(track.remove_keyframe(0.0));
        assert_eq!(track.keyframe_count(), 1);
        // Removing non-existent key returns false.
        assert!(!track.remove_keyframe(99.0));
    }

    // ---- Timeline tests ----

    #[test]
    fn test_timeline_new() {
        let tl = Timeline::new("main");
        assert_eq!(tl.name, "main");
        assert!((tl.fps - 24.0).abs() < f32::EPSILON);
        assert_eq!(tl.track_count(), 0);
    }

    #[test]
    fn test_timeline_add_track() {
        let mut tl = Timeline::new("main");
        let track = AnimTrack::new("smile", TrackInterp::Linear);
        tl.add_track(track);
        assert_eq!(tl.track_count(), 1);
        assert!(tl.get_track("smile").is_some());
        assert!(tl.remove_track("smile"));
        assert_eq!(tl.track_count(), 0);
        assert!(!tl.remove_track("smile"));
    }

    #[test]
    fn test_timeline_evaluate() {
        let mut tl = Timeline::new("main");
        let mut track = AnimTrack::new("brow_raise", TrackInterp::Linear);
        track.add_keyframe(0.0, 0.0);
        track.add_keyframe(1.0, 1.0);
        tl.add_track(track);
        let snapshot = tl.evaluate(0.5);
        let v = snapshot["brow_raise"];
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_timeline_bake() {
        let mut tl = Timeline::new("bake_test");
        let mut track = AnimTrack::new("p", TrackInterp::Linear);
        track.add_keyframe(0.0, 0.0);
        track.add_keyframe(1.0, 1.0);
        tl.add_track(track);
        // At 10 fps → frames at t=0, 0.1, 0.2, …, 1.0 = 11 frames.
        let frames = tl.bake(10.0);
        assert_eq!(frames.len(), 11);
        // First frame value should be 0.0.
        assert!((frames[0]["p"] - 0.0).abs() < 1e-5);
        // Last frame value should be 1.0.
        assert!((frames[10]["p"] - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_timeline_set_key() {
        let mut tl = Timeline::new("sk");
        tl.set_key("eye_blink", 0.0, 0.0);
        tl.set_key("eye_blink", 0.5, 1.0);
        let track = tl.get_track("eye_blink").unwrap();
        assert_eq!(track.keyframe_count(), 2);
        assert_eq!(track.interp, TrackInterp::Linear);
    }

    #[test]
    fn test_timeline_time_scale() {
        let mut tl = Timeline::new("scale");
        tl.set_key("p", 0.0, 0.0);
        tl.set_key("p", 2.0, 1.0);
        assert!((tl.duration() - 2.0).abs() < 1e-5);
        tl.time_scale(0.5);
        assert!((tl.duration() - 1.0).abs() < 1e-5);
    }
}
