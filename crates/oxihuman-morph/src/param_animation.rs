// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// InterpMode
// ---------------------------------------------------------------------------

/// Interpolation mode between keyframes.
#[derive(Debug, Clone, PartialEq)]
pub enum InterpMode {
    /// Hold value until next keyframe.
    Step,
    /// Linear interpolation (lerp).
    Linear,
    /// Smoothstep: 3t^2 - 2t^3.
    Smooth,
    /// Cubic bezier with in/out tangents.
    Bezier,
    /// Sinusoidal ease.
    Sine,
}

// ---------------------------------------------------------------------------
// Keyframe
// ---------------------------------------------------------------------------

/// A single keyframe on an animation track.
pub struct Keyframe {
    /// Time in seconds.
    pub time: f32,
    /// Parameter value in [0..1].
    pub value: f32,
    /// Interpolation mode to use from this keyframe to the next.
    pub interp: InterpMode,
    /// Bezier in-tangent (used when `interp == InterpMode::Bezier`).
    pub tan_in: f32,
    /// Bezier out-tangent (used when `interp == InterpMode::Bezier`).
    pub tan_out: f32,
}

impl Keyframe {
    /// Create a keyframe with `Linear` interpolation.
    pub fn new(time: f32, value: f32) -> Self {
        Self {
            time,
            value,
            interp: InterpMode::Linear,
            tan_in: 0.0,
            tan_out: 0.0,
        }
    }

    /// Create a keyframe with `Step` interpolation.
    pub fn step(time: f32, value: f32) -> Self {
        Self {
            time,
            value,
            interp: InterpMode::Step,
            tan_in: 0.0,
            tan_out: 0.0,
        }
    }

    /// Create a keyframe with `Smooth` interpolation.
    pub fn smooth(time: f32, value: f32) -> Self {
        Self {
            time,
            value,
            interp: InterpMode::Smooth,
            tan_in: 0.0,
            tan_out: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// LoopMode
// ---------------------------------------------------------------------------

/// How a track behaves when time is outside [first_kf, last_kf].
#[derive(Debug, Clone, PartialEq)]
pub enum LoopMode {
    /// Hold the first or last value.
    Clamp,
    /// Wrap time back to the start.
    Loop,
    /// Alternate forward/backward.
    PingPong,
}

// ---------------------------------------------------------------------------
// ParamTrack
// ---------------------------------------------------------------------------

/// Animation track for one named parameter.
pub struct ParamTrack {
    /// Name of the parameter this track controls.
    pub param_name: String,
    /// Keyframes sorted by ascending time.
    pub keyframes: Vec<Keyframe>,
    /// Loop behaviour.
    pub loop_mode: LoopMode,
    /// Value returned before the first keyframe (when `loop_mode == Clamp`).
    pub pre_infinity: f32,
    /// Value returned after the last keyframe (when `loop_mode == Clamp`).
    pub post_infinity: f32,
}

impl ParamTrack {
    /// Create a new empty track.
    pub fn new(param_name: &str) -> Self {
        Self {
            param_name: param_name.to_owned(),
            keyframes: Vec::new(),
            loop_mode: LoopMode::Clamp,
            pre_infinity: 0.0,
            post_infinity: 0.0,
        }
    }

    /// Append a keyframe (will be sorted when `sort_keyframes` is called, or
    /// evaluate will sort implicitly on the first call).
    pub fn add_keyframe(&mut self, kf: Keyframe) {
        self.keyframes.push(kf);
        // Keep sorted so evaluate is always correct.
        self.sort_keyframes();
    }

    /// Sort keyframes by ascending time.
    pub fn sort_keyframes(&mut self) {
        self.keyframes.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Time of the last keyframe (or `0.0` if no keyframes).
    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
    }

    /// Number of keyframes.
    pub fn frame_count(&self) -> usize {
        self.keyframes.len()
    }

    /// Apply `loop_mode` to a raw time value and return the normalised local
    /// time in [first_kf.time, last_kf.time].
    fn apply_loop(&self, t: f32) -> Option<f32> {
        if self.keyframes.is_empty() {
            return None;
        }
        let first = self.keyframes.first().map_or(0.0, |k| k.time);
        let last = self.keyframes.last().map_or(0.0, |k| k.time);
        let span = last - first;

        if span <= 0.0 {
            return Some(first);
        }

        let local = match self.loop_mode {
            LoopMode::Clamp => t.clamp(first, last),
            LoopMode::Loop => {
                let offset = t - first;
                let wrapped = offset.rem_euclid(span);
                first + wrapped
            }
            LoopMode::PingPong => {
                let offset = t - first;
                let cycle = span * 2.0;
                let wrapped = offset.rem_euclid(cycle);
                let ping = if wrapped <= span {
                    wrapped
                } else {
                    cycle - wrapped
                };
                first + ping
            }
        };
        Some(local)
    }

    /// Evaluate the track at time `t`.
    pub fn evaluate(&self, t: f32) -> f32 {
        if self.keyframes.is_empty() {
            return 0.0;
        }
        if self.keyframes.len() == 1 {
            return self.keyframes[0].value;
        }

        let first = &self.keyframes[0];
        let last = &self.keyframes[self.keyframes.len() - 1];

        // Handle out-of-range for Clamp mode
        if self.loop_mode == LoopMode::Clamp {
            if t < first.time {
                return self.pre_infinity;
            }
            if t > last.time {
                return self.post_infinity;
            }
        }

        let local_t = match self.apply_loop(t) {
            Some(v) => v,
            None => return 0.0,
        };

        // Binary-search for the bracket [idx_a, idx_b].
        let idx_b = self.keyframes.partition_point(|k| k.time <= local_t);

        // Clamp to valid range
        if idx_b == 0 {
            return self.keyframes[0].value;
        }
        if idx_b >= self.keyframes.len() {
            return self.keyframes[self.keyframes.len() - 1].value;
        }

        let idx_a = idx_b - 1;
        let ka = &self.keyframes[idx_a];
        let kb = &self.keyframes[idx_b];

        let span = kb.time - ka.time;
        let frac = if span > 0.0 {
            (local_t - ka.time) / span
        } else {
            0.0
        };

        interpolate(ka.value, kb.value, frac, &ka.interp, ka.tan_out, kb.tan_in)
    }

    /// Sample `sample_count` evenly-spaced values across `[0, duration]`.
    /// Returns `(time, value)` pairs.
    pub fn bake(&self, sample_count: usize) -> Vec<(f32, f32)> {
        if sample_count == 0 || self.keyframes.is_empty() {
            return Vec::new();
        }
        let end = self.duration();
        (0..sample_count)
            .map(|i| {
                let t = if sample_count == 1 {
                    0.0
                } else {
                    end * (i as f32 / (sample_count - 1) as f32)
                };
                (t, self.evaluate(t))
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// ParamClip
// ---------------------------------------------------------------------------

/// A full animation clip: multiple tracks for multiple parameters.
pub struct ParamClip {
    /// Human-readable clip name.
    pub name: String,
    /// All tracks in this clip.
    pub tracks: Vec<ParamTrack>,
    /// Intended frames per second.
    pub fps: f32,
}

impl ParamClip {
    /// Create a new empty clip with default 30 fps.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            tracks: Vec::new(),
            fps: 30.0,
        }
    }

    /// Add a track to the clip.
    pub fn add_track(&mut self, track: ParamTrack) {
        self.tracks.push(track);
    }

    /// Find the first track whose `param_name` matches `param`.
    pub fn find_track(&self, param: &str) -> Option<&ParamTrack> {
        self.tracks.iter().find(|t| t.param_name == param)
    }

    /// Number of tracks.
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Maximum duration across all tracks.
    pub fn duration(&self) -> f32 {
        self.tracks
            .iter()
            .map(|t| t.duration())
            .fold(0.0_f32, f32::max)
    }

    /// Evaluate all tracks at time `t`, returning a `HashMap` of param_name → value.
    pub fn evaluate_all(&self, t: f32) -> HashMap<String, f32> {
        self.tracks
            .iter()
            .map(|tr| (tr.param_name.clone(), tr.evaluate(t)))
            .collect()
    }

    /// Bake all tracks to a list of frames at the given `fps`.
    /// Each frame is a `HashMap<param_name, value>`.
    pub fn bake_all(&self, fps: f32) -> Vec<HashMap<String, f32>> {
        let dur = self.duration();
        if dur <= 0.0 || fps <= 0.0 {
            return Vec::new();
        }
        let dt = 1.0 / fps;
        let frame_count = (dur * fps).ceil() as usize + 1;
        (0..frame_count)
            .map(|i| {
                let t = (i as f32 * dt).min(dur);
                self.evaluate_all(t)
            })
            .collect()
    }

    /// Scale all keyframe times by `factor`.
    pub fn scale_time(&mut self, factor: f32) {
        for track in &mut self.tracks {
            for kf in &mut track.keyframes {
                kf.time *= factor;
            }
        }
    }

    /// Shift all keyframe times by `offset`.
    pub fn shift_time(&mut self, offset: f32) {
        for track in &mut self.tracks {
            for kf in &mut track.keyframes {
                kf.time += offset;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Interpolation functions
// ---------------------------------------------------------------------------

/// Smoothstep: 3t^2 - 2t^3.
#[inline]
pub fn smoothstep_interp(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Cubic Hermite spline: interpolates from `p0` to `p1` with tangents `m0` and `m1`.
pub fn cubic_hermite(p0: f32, p1: f32, m0: f32, m1: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    (2.0 * t3 - 3.0 * t2 + 1.0) * p0
        + (t3 - 2.0 * t2 + t) * m0
        + (-2.0 * t3 + 3.0 * t2) * p1
        + (t3 - t2) * m1
}

/// Interpolate between `a` and `b` at parameter `t` (in `[0,1]`) using the
/// given `InterpMode`. `tan_out` is the outgoing tangent from `a`, `tan_in`
/// is the incoming tangent into `b` (used for `Bezier`).
#[allow(clippy::too_many_arguments)]
pub fn interpolate(a: f32, b: f32, t: f32, mode: &InterpMode, tan_out: f32, tan_in: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match mode {
        InterpMode::Step => a,
        InterpMode::Linear => a + (b - a) * t,
        InterpMode::Smooth => {
            let s = smoothstep_interp(t);
            a + (b - a) * s
        }
        InterpMode::Bezier => cubic_hermite(a, b, tan_out, tan_in, t),
        InterpMode::Sine => {
            // Ease in/out using cosine
            let s = 0.5 - 0.5 * (std::f32::consts::PI * t).cos();
            a + (b - a) * s
        }
    }
}

// ---------------------------------------------------------------------------
// Factory functions
// ---------------------------------------------------------------------------

/// Create a breathing animation clip.
///
/// Produces a looping clip with:
/// - `chest_expand`: chest expansion driven by a sine-like curve.
/// - `belly_push`: slight belly movement.
/// - `weight_shift`: subtle lateral weight shift.
pub fn breathing_clip(breath_rate_hz: f32) -> ParamClip {
    let mut clip = ParamClip::new("breathing");
    let period = if breath_rate_hz > 0.0 {
        1.0 / breath_rate_hz
    } else {
        4.0
    };
    let half = period * 0.5;

    // --- chest_expand ---
    let mut chest = ParamTrack::new("chest_expand");
    chest.loop_mode = LoopMode::Loop;
    chest.add_keyframe(Keyframe::smooth(0.0, 0.0));
    chest.add_keyframe(Keyframe::smooth(half, 1.0));
    chest.add_keyframe(Keyframe::smooth(period, 0.0));
    clip.add_track(chest);

    // --- belly_push ---
    let mut belly = ParamTrack::new("belly_push");
    belly.loop_mode = LoopMode::Loop;
    belly.add_keyframe(Keyframe::smooth(0.0, 0.0));
    belly.add_keyframe(Keyframe::smooth(half * 0.4, 0.6));
    belly.add_keyframe(Keyframe::smooth(half, 0.8));
    belly.add_keyframe(Keyframe::smooth(period, 0.0));
    clip.add_track(belly);

    // --- weight_shift (subtle lateral) ---
    let mut shift = ParamTrack::new("weight_shift");
    shift.loop_mode = LoopMode::Loop;
    shift.add_keyframe(Keyframe {
        time: 0.0,
        value: 0.5,
        interp: InterpMode::Sine,
        tan_in: 0.0,
        tan_out: 0.0,
    });
    shift.add_keyframe(Keyframe {
        time: half,
        value: 0.55,
        interp: InterpMode::Sine,
        tan_in: 0.0,
        tan_out: 0.0,
    });
    shift.add_keyframe(Keyframe {
        time: period,
        value: 0.5,
        interp: InterpMode::Sine,
        tan_in: 0.0,
        tan_out: 0.0,
    });
    clip.add_track(shift);

    clip
}

/// Create a morph blend animation that fades from expression `from` to
/// expression `to` over `duration` seconds.
pub fn blend_clip(from: &str, to: &str, duration: f32) -> ParamClip {
    let name = format!("blend_{}_to_{}", from, to);
    let mut clip = ParamClip::new(&name);

    // Track for the "from" expression weight: 1.0 → 0.0
    let mut from_track = ParamTrack::new(from);
    from_track.add_keyframe(Keyframe::smooth(0.0, 1.0));
    from_track.add_keyframe(Keyframe::smooth(duration, 0.0));
    clip.add_track(from_track);

    // Track for the "to" expression weight: 0.0 → 1.0
    let mut to_track = ParamTrack::new(to);
    to_track.add_keyframe(Keyframe::smooth(0.0, 0.0));
    to_track.add_keyframe(Keyframe::smooth(duration, 1.0));
    clip.add_track(to_track);

    clip
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // ---- smoothstep_interp ----

    #[test]
    fn smoothstep_boundaries() {
        assert!((smoothstep_interp(0.0) - 0.0).abs() < 1e-6);
        assert!((smoothstep_interp(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn smoothstep_midpoint() {
        // smoothstep(0.5) = 0.25 * (3 - 1) = 0.5
        assert!((smoothstep_interp(0.5) - 0.5).abs() < 1e-6);
    }

    // ---- cubic_hermite ----

    #[test]
    fn cubic_hermite_endpoints() {
        // At t=0, result should be p0; at t=1, result should be p1.
        let p0 = 0.2_f32;
        let p1 = 0.8_f32;
        assert!((cubic_hermite(p0, p1, 0.0, 0.0, 0.0) - p0).abs() < 1e-6);
        assert!((cubic_hermite(p0, p1, 0.0, 0.0, 1.0) - p1).abs() < 1e-6);
    }

    // ---- interpolate ----

    #[test]
    fn interpolate_step_holds_a() {
        let v = interpolate(0.3, 0.9, 0.8, &InterpMode::Step, 0.0, 0.0);
        assert!((v - 0.3).abs() < 1e-6);
    }

    #[test]
    fn interpolate_linear_midpoint() {
        let v = interpolate(0.0, 1.0, 0.5, &InterpMode::Linear, 0.0, 0.0);
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn interpolate_smooth_midpoint() {
        // smoothstep(0.5) = 0.5 → lerp(0,1,0.5) = 0.5
        let v = interpolate(0.0, 1.0, 0.5, &InterpMode::Smooth, 0.0, 0.0);
        assert!((v - 0.5).abs() < 1e-6);
    }

    #[test]
    fn interpolate_sine_boundaries() {
        let a = interpolate(0.0, 1.0, 0.0, &InterpMode::Sine, 0.0, 0.0);
        let b = interpolate(0.0, 1.0, 1.0, &InterpMode::Sine, 0.0, 0.0);
        assert!(a.abs() < 1e-6);
        assert!((b - 1.0).abs() < 1e-6);
    }

    // ---- Keyframe constructors ----

    #[test]
    fn keyframe_new_is_linear() {
        let kf = Keyframe::new(1.0, 0.5);
        assert_eq!(kf.interp, InterpMode::Linear);
        assert!((kf.time - 1.0).abs() < 1e-6);
        assert!((kf.value - 0.5).abs() < 1e-6);
    }

    #[test]
    fn keyframe_step_constructor() {
        let kf = Keyframe::step(2.0, 0.7);
        assert_eq!(kf.interp, InterpMode::Step);
    }

    #[test]
    fn keyframe_smooth_constructor() {
        let kf = Keyframe::smooth(3.0, 0.3);
        assert_eq!(kf.interp, InterpMode::Smooth);
    }

    // ---- ParamTrack ----

    #[test]
    fn param_track_evaluate_linear() {
        let mut track = ParamTrack::new("test");
        track.add_keyframe(Keyframe::new(0.0, 0.0));
        track.add_keyframe(Keyframe::new(1.0, 1.0));
        let v = track.evaluate(0.5);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn param_track_evaluate_step() {
        let mut track = ParamTrack::new("step_track");
        track.add_keyframe(Keyframe::step(0.0, 0.0));
        track.add_keyframe(Keyframe::step(1.0, 1.0));
        // Before second keyframe: holds 0.0
        let v = track.evaluate(0.5);
        assert!((v - 0.0).abs() < 1e-5);
    }

    #[test]
    fn param_track_clamp_pre_post() {
        let mut track = ParamTrack::new("clamp");
        track.loop_mode = LoopMode::Clamp;
        track.pre_infinity = 0.1;
        track.post_infinity = 0.9;
        track.add_keyframe(Keyframe::new(1.0, 0.2));
        track.add_keyframe(Keyframe::new(2.0, 0.8));
        // Before first keyframe → pre_infinity
        let pre = track.evaluate(0.0);
        assert!((pre - 0.1).abs() < 1e-5, "pre={pre}");
        // After last keyframe → post_infinity
        let post = track.evaluate(5.0);
        assert!((post - 0.9).abs() < 1e-5, "post={post}");
    }

    #[test]
    fn param_track_loop_mode() {
        let mut track = ParamTrack::new("loop_track");
        track.loop_mode = LoopMode::Loop;
        track.add_keyframe(Keyframe::new(0.0, 0.0));
        track.add_keyframe(Keyframe::new(1.0, 1.0));
        // At t=1.5 (0.5 into second cycle) should equal t=0.5
        let v_original = track.evaluate(0.5);
        let v_looped = track.evaluate(1.5);
        assert!((v_original - v_looped).abs() < 1e-4);
    }

    #[test]
    fn param_track_pingpong_mode() {
        let mut track = ParamTrack::new("pp");
        track.loop_mode = LoopMode::PingPong;
        track.add_keyframe(Keyframe::new(0.0, 0.0));
        track.add_keyframe(Keyframe::new(1.0, 1.0));
        // At t=1.5 in pingpong should equal t=0.5 going backward
        let v_fwd = track.evaluate(0.5); // 0.5
        let v_rev = track.evaluate(1.5); // should also be ~0.5
        assert!((v_fwd - v_rev).abs() < 1e-4);
    }

    #[test]
    fn param_track_duration_and_frame_count() {
        let mut track = ParamTrack::new("dur");
        track.add_keyframe(Keyframe::new(0.0, 0.0));
        track.add_keyframe(Keyframe::new(3.0, 1.0));
        assert!((track.duration() - 3.0).abs() < 1e-5);
        assert_eq!(track.frame_count(), 2);
    }

    #[test]
    fn param_track_bake_count() {
        let mut track = ParamTrack::new("bake");
        track.add_keyframe(Keyframe::new(0.0, 0.0));
        track.add_keyframe(Keyframe::new(1.0, 1.0));
        let samples = track.bake(11);
        assert_eq!(samples.len(), 11);
        // First sample at t=0, value=0.0
        assert!((samples[0].1 - 0.0).abs() < 1e-5);
        // Last sample at t=1, value=1.0
        assert!((samples[10].1 - 1.0).abs() < 1e-5);
    }

    // ---- ParamClip ----

    #[test]
    fn param_clip_find_track() {
        let mut clip = ParamClip::new("test_clip");
        let mut track = ParamTrack::new("jaw_open");
        track.add_keyframe(Keyframe::new(0.0, 0.0));
        clip.add_track(track);
        assert!(clip.find_track("jaw_open").is_some());
        assert!(clip.find_track("missing").is_none());
    }

    #[test]
    fn param_clip_duration_max() {
        let mut clip = ParamClip::new("multi");
        let mut t1 = ParamTrack::new("a");
        t1.add_keyframe(Keyframe::new(0.0, 0.0));
        t1.add_keyframe(Keyframe::new(2.0, 1.0));
        let mut t2 = ParamTrack::new("b");
        t2.add_keyframe(Keyframe::new(0.0, 0.0));
        t2.add_keyframe(Keyframe::new(5.0, 1.0));
        clip.add_track(t1);
        clip.add_track(t2);
        assert!((clip.duration() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn param_clip_evaluate_all() {
        let mut clip = ParamClip::new("eval");
        let mut tr = ParamTrack::new("smile");
        tr.add_keyframe(Keyframe::new(0.0, 0.0));
        tr.add_keyframe(Keyframe::new(1.0, 1.0));
        clip.add_track(tr);
        let map = clip.evaluate_all(0.5);
        let v = map["smile"];
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn param_clip_scale_shift_time() {
        let mut clip = ParamClip::new("scale_shift");
        let mut tr = ParamTrack::new("p");
        tr.add_keyframe(Keyframe::new(0.0, 0.0));
        tr.add_keyframe(Keyframe::new(2.0, 1.0));
        clip.add_track(tr);
        clip.scale_time(2.0);
        assert!((clip.duration() - 4.0).abs() < 1e-5);
        clip.shift_time(1.0);
        assert!((clip.duration() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn param_clip_bake_all_frames() {
        let mut clip = ParamClip::new("bake_all");
        let mut tr = ParamTrack::new("eye");
        tr.add_keyframe(Keyframe::new(0.0, 0.0));
        tr.add_keyframe(Keyframe::new(1.0, 1.0));
        clip.add_track(tr);
        let frames = clip.bake_all(10.0);
        // 10fps over 1s → ~11 frames
        assert!(!frames.is_empty());
        assert!(frames[0].contains_key("eye"));
    }

    // ---- Factory functions ----

    #[test]
    fn breathing_clip_has_three_tracks() {
        let clip = breathing_clip(0.25); // 4-second period
        assert_eq!(clip.track_count(), 3);
        assert!(clip.find_track("chest_expand").is_some());
        assert!(clip.find_track("belly_push").is_some());
        assert!(clip.find_track("weight_shift").is_some());
    }

    #[test]
    fn breathing_clip_loops() {
        let clip = breathing_clip(1.0); // 1Hz = 1s period
        let track = clip.find_track("chest_expand").expect("should succeed");
        assert_eq!(track.loop_mode, LoopMode::Loop);
        // At t=0 and t=1 (one full loop), values should be equal
        let v0 = track.evaluate(0.0);
        let v1 = track.evaluate(1.0);
        assert!((v0 - v1).abs() < 1e-4, "v0={v0}, v1={v1}");
    }

    #[test]
    fn blend_clip_two_tracks() {
        let clip = blend_clip("neutral", "happy", 2.0);
        assert_eq!(clip.track_count(), 2);
        assert!(clip.find_track("neutral").is_some());
        assert!(clip.find_track("happy").is_some());
        // At t=0: neutral=1.0, happy=0.0
        let map0 = clip.evaluate_all(0.0);
        assert!((map0["neutral"] - 1.0).abs() < 1e-5);
        assert!((map0["happy"] - 0.0).abs() < 1e-5);
        // At t=2: neutral=0.0, happy=1.0
        let map2 = clip.evaluate_all(2.0);
        assert!((map2["neutral"] - 0.0).abs() < 1e-5);
        assert!((map2["happy"] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn blend_clip_name_contains_expressions() {
        let clip = blend_clip("sad", "surprised", 1.0);
        assert!(clip.name.contains("sad"));
        assert!(clip.name.contains("surprised"));
    }

    // ---- File output test ----

    #[test]
    fn bake_writes_to_tmp() {
        let mut clip = ParamClip::new("write_test");
        let mut tr = ParamTrack::new("brow");
        tr.add_keyframe(Keyframe::new(0.0, 0.0));
        tr.add_keyframe(Keyframe::new(1.0, 1.0));
        clip.add_track(tr);
        let frames = clip.bake_all(5.0);
        let mut lines = Vec::new();
        for (i, frame) in frames.iter().enumerate() {
            let v = frame.get("brow").copied().unwrap_or(0.0);
            lines.push(format!("frame {i}: brow={v:.4}"));
        }
        let output = lines.join("\n");
        let tmp = std::env::temp_dir().join("param_animation_bake_test.txt");
        fs::write(&tmp, &output).expect("should succeed");
        let read_back = fs::read_to_string(&tmp).expect("should succeed");
        assert!(read_back.contains("brow="));
    }

    #[test]
    fn bezier_interpolate_boundaries() {
        let a = interpolate(0.0, 1.0, 0.0, &InterpMode::Bezier, 0.0, 0.0);
        let b = interpolate(0.0, 1.0, 1.0, &InterpMode::Bezier, 0.0, 0.0);
        assert!(a.abs() < 1e-6);
        assert!((b - 1.0).abs() < 1e-6);
    }

    #[test]
    fn param_clip_track_count() {
        let mut clip = ParamClip::new("count_test");
        assert_eq!(clip.track_count(), 0);
        clip.add_track(ParamTrack::new("a"));
        clip.add_track(ParamTrack::new("b"));
        assert_eq!(clip.track_count(), 2);
    }
}
