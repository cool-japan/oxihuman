// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

//! Named expression keyframe sequence: compose multi-expression animations
//! with hold/ease timing.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Map of expression name → weight in \[0, 1\].
pub type ExprWeights = HashMap<String, f32>;

/// Easing function applied when transitioning to the next keyframe.
#[derive(Debug, Clone, PartialEq)]
pub enum EaseType {
    /// Constant velocity – no acceleration.
    Linear,
    /// Accelerate from zero.
    EaseIn,
    /// Decelerate to zero.
    EaseOut,
    /// Smooth S-curve acceleration and deceleration.
    EaseInOut,
    /// Jump instantly to the next keyframe value (hold until next).
    Step,
}

/// A single snapshot in an expression animation track.
pub struct ExprKeyframe {
    /// Time in seconds at which this keyframe is reached.
    pub time: f32,
    /// Expression weights active at this instant.
    pub weights: ExprWeights,
    /// Easing applied when blending toward the following keyframe.
    pub ease_to_next: EaseType,
    /// Seconds to hold at this keyframe before starting the transition.
    pub hold_duration: f32,
}

/// How the track behaves after the last keyframe has been reached.
#[derive(Debug, Clone, PartialEq)]
pub enum SeqLoopMode {
    /// Play once and freeze on the last keyframe.
    Once,
    /// Wrap time back to the beginning and repeat.
    Loop,
    /// Bounce back and forth between first and last keyframe.
    PingPong,
}

/// A named sequence of [`ExprKeyframe`]s.
pub struct ExprTrack {
    /// Human-readable name for this track (e.g. `"happy_cycle"`).
    pub name: String,
    /// Keyframes sorted ascending by `time`.
    pub keyframes: Vec<ExprKeyframe>,
    /// Looping behaviour.
    pub loop_mode: SeqLoopMode,
}

/// Combines multiple [`ExprTrack`]s and evaluates them additively.
pub struct ExprSequencer {
    tracks: Vec<ExprTrack>,
}

// ---------------------------------------------------------------------------
// Standalone functions
// ---------------------------------------------------------------------------

/// Apply an easing function to `t` ∈ \[0, 1\].
///
/// The result is also ∈ \[0, 1\].
pub fn ease_value(t: f32, ease: &EaseType) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match ease {
        EaseType::Linear => t,
        EaseType::EaseIn => t * t,
        EaseType::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        EaseType::EaseInOut => t * t * (3.0 - 2.0 * t),
        EaseType::Step => {
            if t < 1.0 {
                0.0
            } else {
                1.0
            }
        }
    }
}

/// Linearly interpolate between two [`ExprWeights`] maps.
///
/// Keys present in only one map are treated as having weight `0` in the other.
pub fn lerp_weights(a: &ExprWeights, b: &ExprWeights, t: f32) -> ExprWeights {
    let t = t.clamp(0.0, 1.0);
    let mut out: ExprWeights = HashMap::new();

    for (k, va) in a {
        let vb = b.get(k).copied().unwrap_or(0.0);
        out.insert(k.clone(), va + (vb - va) * t);
    }
    for (k, vb) in b {
        if !out.contains_key(k) {
            out.insert(k.clone(), vb * t);
        }
    }
    out
}

/// Generate a repeating blink animation track.
///
/// * `interval` – seconds between blink starts.
/// * `duration` – total duration of one blink (open → close → open).
pub fn blink_track(interval: f32, duration: f32) -> ExprTrack {
    let interval = interval.max(duration + 0.01);
    let close_t = duration * 0.3;
    let open_t = duration;

    let closed: ExprWeights = [("blink".to_string(), 1.0)].into();
    let open: ExprWeights = [("blink".to_string(), 0.0)].into();

    let mut track = ExprTrack::new("blink");
    track.loop_mode = SeqLoopMode::Loop;

    // Start open, close quickly, hold shut briefly, reopen.
    track.add_keyframe(ExprKeyframe {
        time: 0.0,
        weights: open.clone(),
        ease_to_next: EaseType::EaseIn,
        hold_duration: interval - duration,
    });
    track.add_keyframe(ExprKeyframe {
        time: interval - duration,
        weights: open.clone(),
        ease_to_next: EaseType::EaseIn,
        hold_duration: 0.0,
    });
    track.add_keyframe(ExprKeyframe {
        time: interval - duration + close_t,
        weights: closed,
        ease_to_next: EaseType::EaseOut,
        hold_duration: 0.0,
    });
    track.add_keyframe(ExprKeyframe {
        time: interval - duration + open_t,
        weights: open,
        ease_to_next: EaseType::Linear,
        hold_duration: 0.0,
    });

    track
}

/// Generate a subtle breathing expression track.
///
/// `rate` is breaths per minute.  The track loops indefinitely.
pub fn breathing_expr_track(rate: f32) -> ExprTrack {
    let period = 60.0 / rate.max(1.0);
    let inhale_end = period * 0.4;
    let exhale_end = period;

    let inhale: ExprWeights = [
        ("nostrils_flare".to_string(), 0.3),
        ("chest_expand".to_string(), 0.2),
    ]
    .into();
    let exhale: ExprWeights = [
        ("nostrils_flare".to_string(), 0.0),
        ("chest_expand".to_string(), 0.0),
    ]
    .into();

    let mut track = ExprTrack::new("breathing");
    track.loop_mode = SeqLoopMode::Loop;

    track.add_keyframe(ExprKeyframe {
        time: 0.0,
        weights: exhale.clone(),
        ease_to_next: EaseType::EaseInOut,
        hold_duration: 0.0,
    });
    track.add_keyframe(ExprKeyframe {
        time: inhale_end,
        weights: inhale,
        ease_to_next: EaseType::EaseInOut,
        hold_duration: 0.0,
    });
    track.add_keyframe(ExprKeyframe {
        time: exhale_end,
        weights: exhale,
        ease_to_next: EaseType::EaseInOut,
        hold_duration: 0.0,
    });

    track
}

// ---------------------------------------------------------------------------
// ExprTrack impl
// ---------------------------------------------------------------------------

impl ExprTrack {
    /// Create a new, empty track with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            keyframes: Vec::new(),
            loop_mode: SeqLoopMode::Once,
        }
    }

    /// Insert a keyframe, keeping the list sorted by `time`.
    pub fn add_keyframe(&mut self, kf: ExprKeyframe) {
        let pos = self.keyframes.partition_point(|k| k.time <= kf.time);
        self.keyframes.insert(pos, kf);
    }

    /// Remove the keyframe at `idx` (no-op if out of bounds).
    pub fn remove_keyframe(&mut self, idx: usize) {
        if idx < self.keyframes.len() {
            self.keyframes.remove(idx);
        }
    }

    /// Total track duration: time of the last keyframe plus its hold.
    pub fn duration(&self) -> f32 {
        self.keyframes
            .last()
            .map(|kf| kf.time + kf.hold_duration)
            .unwrap_or(0.0)
    }

    /// Evaluate expression weights at time `t_in`, respecting loop mode and easing.
    pub fn evaluate(&self, t_in: f32) -> ExprWeights {
        if self.keyframes.is_empty() {
            return HashMap::new();
        }
        if self.keyframes.len() == 1 {
            return self.keyframes[0].weights.clone();
        }

        let dur = self.duration();
        let t = self.remap_time(t_in, dur);

        // Find the pair of keyframes that straddle `t`.
        let next_idx = self.keyframes.partition_point(|kf| kf.time <= t);

        if next_idx == 0 {
            return self.keyframes[0].weights.clone();
        }
        if next_idx >= self.keyframes.len() {
            return self.keyframes[self.keyframes.len() - 1].weights.clone();
        }

        let prev = &self.keyframes[next_idx - 1];
        let next = &self.keyframes[next_idx];

        // Time after the hold period ends.
        let transition_start = prev.time + prev.hold_duration;

        // Still within the hold window.
        if t <= transition_start {
            return prev.weights.clone();
        }

        let span = next.time - transition_start;
        let local_t = if span > f32::EPSILON {
            ((t - transition_start) / span).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let eased_t = ease_value(local_t, &prev.ease_to_next);
        lerp_weights(&prev.weights, &next.weights, eased_t)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn remap_time(&self, t: f32, dur: f32) -> f32 {
        if dur <= 0.0 {
            return 0.0;
        }
        match self.loop_mode {
            SeqLoopMode::Once => t.clamp(0.0, dur),
            SeqLoopMode::Loop => {
                let wrapped = t % dur;
                if wrapped < 0.0 {
                    wrapped + dur
                } else {
                    wrapped
                }
            }
            SeqLoopMode::PingPong => {
                let cycle = dur * 2.0;
                let pos = t % cycle;
                let pos = if pos < 0.0 { pos + cycle } else { pos };
                if pos <= dur {
                    pos
                } else {
                    cycle - pos
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ExprSequencer impl
// ---------------------------------------------------------------------------

impl ExprSequencer {
    /// Create an empty sequencer.
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    /// Add a track to the sequencer.
    pub fn add_track(&mut self, track: ExprTrack) {
        self.tracks.push(track);
    }

    /// Number of tracks currently registered.
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Evaluate all tracks at time `t` and blend additively.
    ///
    /// When the same expression key appears in multiple tracks the weights
    /// are summed and then clamped to \[0, 1\].
    pub fn evaluate_all(&self, t: f32) -> ExprWeights {
        let mut out: ExprWeights = HashMap::new();
        for track in &self.tracks {
            let w = track.evaluate(t);
            for (k, v) in w {
                *out.entry(k).or_insert(0.0) += v;
            }
        }
        // Clamp additive sum.
        for v in out.values_mut() {
            *v = v.clamp(0.0, 1.0);
        }
        out
    }
}

impl Default for ExprSequencer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    fn single_weight(name: &str, v: f32) -> ExprWeights {
        [(name.to_string(), v)].into()
    }

    // --- ease_value ----------------------------------------------------------

    #[test]
    fn test_ease_linear() {
        assert!(approx(ease_value(0.0, &EaseType::Linear), 0.0));
        assert!(approx(ease_value(0.5, &EaseType::Linear), 0.5));
        assert!(approx(ease_value(1.0, &EaseType::Linear), 1.0));
    }

    #[test]
    fn test_ease_in() {
        assert!(approx(ease_value(0.0, &EaseType::EaseIn), 0.0));
        assert!(approx(ease_value(0.5, &EaseType::EaseIn), 0.25));
        assert!(approx(ease_value(1.0, &EaseType::EaseIn), 1.0));
    }

    #[test]
    fn test_ease_out() {
        assert!(approx(ease_value(0.0, &EaseType::EaseOut), 0.0));
        assert!(approx(ease_value(0.5, &EaseType::EaseOut), 0.75));
        assert!(approx(ease_value(1.0, &EaseType::EaseOut), 1.0));
    }

    #[test]
    fn test_ease_in_out_midpoint() {
        // smoothstep(0.5) = 0.5
        assert!(approx(ease_value(0.5, &EaseType::EaseInOut), 0.5));
    }

    #[test]
    fn test_ease_step() {
        assert!(approx(ease_value(0.0, &EaseType::Step), 0.0));
        assert!(approx(ease_value(0.5, &EaseType::Step), 0.0));
        assert!(approx(ease_value(1.0, &EaseType::Step), 1.0));
    }

    #[test]
    fn test_ease_clamps_input() {
        assert!(approx(ease_value(-1.0, &EaseType::Linear), 0.0));
        assert!(approx(ease_value(2.0, &EaseType::Linear), 1.0));
    }

    // --- lerp_weights --------------------------------------------------------

    #[test]
    fn test_lerp_weights_midpoint() {
        let a = single_weight("smile", 0.0);
        let b = single_weight("smile", 1.0);
        let mid = lerp_weights(&a, &b, 0.5);
        assert!(approx(*mid.get("smile").unwrap(), 0.5));
    }

    #[test]
    fn test_lerp_weights_missing_key_in_b() {
        let a = single_weight("anger", 0.8);
        let b: ExprWeights = HashMap::new();
        let result = lerp_weights(&a, &b, 0.5);
        assert!(approx(*result.get("anger").unwrap(), 0.4));
    }

    #[test]
    fn test_lerp_weights_missing_key_in_a() {
        let a: ExprWeights = HashMap::new();
        let b = single_weight("joy", 1.0);
        let result = lerp_weights(&a, &b, 0.5);
        assert!(approx(*result.get("joy").unwrap(), 0.5));
    }

    #[test]
    fn test_lerp_weights_t0_equals_a() {
        let a = single_weight("fear", 0.6);
        let b = single_weight("fear", 0.0);
        let result = lerp_weights(&a, &b, 0.0);
        assert!(approx(*result.get("fear").unwrap(), 0.6));
    }

    // --- ExprTrack -----------------------------------------------------------

    #[test]
    fn test_track_add_sorted() {
        let mut track = ExprTrack::new("test");
        track.add_keyframe(ExprKeyframe {
            time: 2.0,
            weights: single_weight("a", 1.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        track.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: single_weight("a", 0.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        assert!(approx(track.keyframes[0].time, 0.0));
        assert!(approx(track.keyframes[1].time, 2.0));
    }

    #[test]
    fn test_track_duration() {
        let mut track = ExprTrack::new("dur_test");
        track.add_keyframe(ExprKeyframe {
            time: 3.0,
            weights: HashMap::new(),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.5,
        });
        assert!(approx(track.duration(), 3.5));
    }

    #[test]
    fn test_track_remove_keyframe() {
        let mut track = ExprTrack::new("rm");
        track.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: HashMap::new(),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        track.add_keyframe(ExprKeyframe {
            time: 1.0,
            weights: HashMap::new(),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        track.remove_keyframe(0);
        assert_eq!(track.keyframes.len(), 1);
        assert!(approx(track.keyframes[0].time, 1.0));
    }

    #[test]
    fn test_track_evaluate_at_start() {
        let mut track = ExprTrack::new("eval");
        track.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: single_weight("smile", 0.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        track.add_keyframe(ExprKeyframe {
            time: 1.0,
            weights: single_weight("smile", 1.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        let w = track.evaluate(0.0);
        assert!(approx(*w.get("smile").unwrap(), 0.0));
    }

    #[test]
    fn test_track_evaluate_at_end() {
        let mut track = ExprTrack::new("eval_end");
        track.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: single_weight("smile", 0.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        track.add_keyframe(ExprKeyframe {
            time: 1.0,
            weights: single_weight("smile", 1.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        let w = track.evaluate(1.0);
        assert!(approx(*w.get("smile").unwrap(), 1.0));
    }

    #[test]
    fn test_track_evaluate_midpoint_linear() {
        let mut track = ExprTrack::new("mid_linear");
        track.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: single_weight("brow", 0.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        track.add_keyframe(ExprKeyframe {
            time: 2.0,
            weights: single_weight("brow", 1.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        let w = track.evaluate(1.0);
        assert!(approx(*w.get("brow").unwrap(), 0.5));
    }

    #[test]
    fn test_track_loop_mode() {
        let mut track = ExprTrack::new("loop");
        track.loop_mode = SeqLoopMode::Loop;
        track.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: single_weight("x", 0.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        track.add_keyframe(ExprKeyframe {
            time: 1.0,
            weights: single_weight("x", 1.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        // At t=1.5 with loop (dur=1), maps to t=0.5 → 0.5
        let w = track.evaluate(1.5);
        assert!(approx(*w.get("x").unwrap(), 0.5));
    }

    #[test]
    fn test_track_pingpong_mode() {
        let mut track = ExprTrack::new("pp");
        track.loop_mode = SeqLoopMode::PingPong;
        track.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: single_weight("y", 0.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        track.add_keyframe(ExprKeyframe {
            time: 1.0,
            weights: single_weight("y", 1.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        // PingPong: dur=1, cycle=2. At t=1.5, pos=1.5 > 1 → cycle-pos=0.5 → y=0.5
        let w = track.evaluate(1.5);
        assert!(approx(*w.get("y").unwrap(), 0.5));
    }

    #[test]
    fn test_track_hold_duration() {
        let mut track = ExprTrack::new("hold");
        track.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: single_weight("sad", 1.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 1.0, // hold for 1 second before transitioning
        });
        track.add_keyframe(ExprKeyframe {
            time: 2.0,
            weights: single_weight("sad", 0.0),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });
        // At t=0.5 we should still be in the hold window → weight = 1.0
        let w = track.evaluate(0.5);
        assert!(approx(*w.get("sad").unwrap(), 1.0));
        // At t=1.5 transition is half-way (from t=1..t=2) → weight ≈ 0.5
        let w2 = track.evaluate(1.5);
        assert!(approx(*w2.get("sad").unwrap(), 0.5));
    }

    // --- ExprSequencer -------------------------------------------------------

    #[test]
    fn test_sequencer_track_count() {
        let mut seq = ExprSequencer::new();
        seq.add_track(ExprTrack::new("a"));
        seq.add_track(ExprTrack::new("b"));
        assert_eq!(seq.track_count(), 2);
    }

    #[test]
    fn test_sequencer_evaluate_all_additive_clamp() {
        let mut seq = ExprSequencer::new();

        let mut t1 = ExprTrack::new("t1");
        t1.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: single_weight("smile", 0.8),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });

        let mut t2 = ExprTrack::new("t2");
        t2.add_keyframe(ExprKeyframe {
            time: 0.0,
            weights: single_weight("smile", 0.6),
            ease_to_next: EaseType::Linear,
            hold_duration: 0.0,
        });

        seq.add_track(t1);
        seq.add_track(t2);

        let w = seq.evaluate_all(0.0);
        // 0.8 + 0.6 = 1.4 → clamped to 1.0
        assert!(approx(*w.get("smile").unwrap(), 1.0));
    }

    // --- blink_track ---------------------------------------------------------

    #[test]
    fn test_blink_track_loops() {
        let track = blink_track(4.0, 0.2);
        assert_eq!(track.loop_mode, SeqLoopMode::Loop);
    }

    #[test]
    fn test_blink_track_has_keyframes() {
        let track = blink_track(4.0, 0.2);
        assert!(track.keyframes.len() >= 3);
    }

    // --- breathing_expr_track ------------------------------------------------

    #[test]
    fn test_breathing_track_loops() {
        let track = breathing_expr_track(15.0);
        assert_eq!(track.loop_mode, SeqLoopMode::Loop);
    }

    #[test]
    fn test_breathing_track_has_keyframes() {
        let track = breathing_expr_track(15.0);
        assert!(track.keyframes.len() >= 2);
    }

    #[test]
    fn test_breathing_track_period() {
        let rate = 12.0_f32;
        let expected_period = 60.0 / rate;
        let track = breathing_expr_track(rate);
        assert!(approx(track.duration(), expected_period));
    }
}
