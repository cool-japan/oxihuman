// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Keyframe-based emotion animation with interpolation and blending.

use std::collections::HashMap;

// ── Types ────────────────────────────────────────────────────────────────────

/// A single keyframe in an emotion timeline.
#[allow(dead_code)]
pub struct EmotionKeyframe {
    /// Time in seconds.
    pub time: f32,
    /// Map of emotion name to weight in 0..=1.
    pub emotions: HashMap<String, f32>,
    /// Easing function applied to interpolation leaving this keyframe.
    pub easing: TimelineEasing,
}

/// Easing curve applied between two keyframes.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum TimelineEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// Jump to the *start* keyframe value; no interpolation.
    Step,
}

/// Loop behaviour once the timeline reaches its end.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum TimelineLoop {
    /// Play once then hold the last frame.
    Once,
    /// Wrap back to the beginning.
    Loop,
    /// Bounce back and forth.
    PingPong,
}

/// A keyframe-driven timeline of emotion weights.
#[allow(dead_code)]
pub struct EmotionTimeline {
    /// Keyframes, always kept sorted by `time`.
    pub keyframes: Vec<EmotionKeyframe>,
    /// Total duration in seconds.
    pub duration: f32,
    /// How time is mapped past `duration`.
    pub loop_mode: TimelineLoop,
}

// ── Implementations ──────────────────────────────────────────────────────────

impl EmotionTimeline {
    /// Create an empty timeline.
    #[allow(dead_code)]
    pub fn new(duration: f32, loop_mode: TimelineLoop) -> Self {
        Self {
            keyframes: Vec::new(),
            duration,
            loop_mode,
        }
    }

    /// Insert a keyframe, keeping `keyframes` sorted by time.
    #[allow(dead_code)]
    pub fn add_keyframe(&mut self, kf: EmotionKeyframe) {
        let pos = self
            .keyframes
            .partition_point(|existing| existing.time <= kf.time);
        self.keyframes.insert(pos, kf);
    }

    /// Sample the emotion weights at an arbitrary time `t`.
    #[allow(dead_code)]
    pub fn sample(&self, t: f32) -> HashMap<String, f32> {
        let t = normalize_emotion_time(t, self.duration, &self.loop_mode);

        if self.keyframes.is_empty() {
            return HashMap::new();
        }

        // Find surrounding keyframes.
        let idx = self.keyframes.partition_point(|kf| kf.time <= t);

        if idx == 0 {
            // Before or at first keyframe – hold first.
            return self.keyframes[0].emotions.clone();
        }
        if idx >= self.keyframes.len() {
            // After last keyframe – hold last.
            return self.keyframes[self.keyframes.len() - 1].emotions.clone();
        }

        let kf_a = &self.keyframes[idx - 1];
        let kf_b = &self.keyframes[idx];

        let span = kf_b.time - kf_a.time;
        let raw_t = if span.abs() < f32::EPSILON {
            1.0_f32
        } else {
            (t - kf_a.time) / span
        };

        let eased_t = apply_easing_fn(raw_t, &kf_a.easing);
        interpolate_emotions(&kf_a.emotions, &kf_b.emotions, eased_t)
    }

    /// Bake the timeline to a uniform frame sequence at `fps` frames per second.
    /// The returned vec has `ceil(duration * fps) + 1` entries (inclusive of t=0 and t=duration).
    #[allow(dead_code)]
    pub fn bake(&self, fps: f32) -> Vec<HashMap<String, f32>> {
        let frame_count = (self.duration * fps).ceil() as usize + 1;
        (0..frame_count)
            .map(|i| {
                let t = (i as f32) / fps;
                self.sample(t)
            })
            .collect()
    }
}

// ── Free functions ────────────────────────────────────────────────────────────

/// Linear-interpolate two emotion weight maps, taking the union of their keys.
/// Missing keys are treated as weight 0.0.
#[allow(dead_code)]
pub fn interpolate_emotions(
    a: &HashMap<String, f32>,
    b: &HashMap<String, f32>,
    t: f32,
) -> HashMap<String, f32> {
    let mut result = HashMap::new();

    for (k, &va) in a {
        let vb = b.get(k).copied().unwrap_or(0.0);
        result.insert(k.clone(), va + (vb - va) * t);
    }
    for (k, &vb) in b {
        if !result.contains_key(k) {
            // Key only in b: a-side is 0.
            result.insert(k.clone(), vb * t);
        }
    }
    result
}

/// Apply an easing curve to a normalised `t` in 0..=1.
#[allow(dead_code)]
pub fn apply_easing_fn(t: f32, easing: &TimelineEasing) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match easing {
        TimelineEasing::Linear => t,
        TimelineEasing::EaseIn => t * t,
        TimelineEasing::EaseOut => t * (2.0 - t),
        TimelineEasing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
        // Step: return 0 until the end of the interval, then jump.
        TimelineEasing::Step => {
            if t < 1.0 {
                0.0
            } else {
                1.0
            }
        }
    }
}

/// Map an arbitrary time `t` into `0..=duration` respecting the loop mode.
#[allow(dead_code)]
pub fn normalize_emotion_time(t: f32, duration: f32, loop_mode: &TimelineLoop) -> f32 {
    if duration <= 0.0 {
        return 0.0;
    }
    match loop_mode {
        TimelineLoop::Once => t.clamp(0.0, duration),
        TimelineLoop::Loop => {
            let wrapped = t % duration;
            if wrapped < 0.0 {
                wrapped + duration
            } else {
                wrapped
            }
        }
        TimelineLoop::PingPong => {
            let period = 2.0 * duration;
            let wrapped = ((t % period) + period) % period;
            if wrapped <= duration {
                wrapped
            } else {
                period - wrapped
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_kf(time: f32, emotions: &[(&str, f32)]) -> EmotionKeyframe {
        EmotionKeyframe {
            time,
            emotions: emotions.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            easing: TimelineEasing::Linear,
        }
    }

    fn make_kf_easing(
        time: f32,
        emotions: &[(&str, f32)],
        easing: TimelineEasing,
    ) -> EmotionKeyframe {
        EmotionKeyframe {
            time,
            emotions: emotions.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            easing,
        }
    }

    // 1. add_keyframe keeps list sorted
    #[test]
    fn test_add_keyframe_sorted() {
        let mut tl = EmotionTimeline::new(2.0, TimelineLoop::Once);
        tl.add_keyframe(make_kf(1.5, &[]));
        tl.add_keyframe(make_kf(0.5, &[]));
        tl.add_keyframe(make_kf(1.0, &[]));
        let times: Vec<f32> = tl.keyframes.iter().map(|k| k.time).collect();
        assert_eq!(times, vec![0.5, 1.0, 1.5]);
    }

    // 2. sample at exact keyframe time returns that keyframe's values
    #[test]
    fn test_sample_exact_keyframe() {
        let mut tl = EmotionTimeline::new(2.0, TimelineLoop::Once);
        tl.add_keyframe(make_kf(0.0, &[("happy", 0.0)]));
        tl.add_keyframe(make_kf(1.0, &[("happy", 1.0)]));
        let result = tl.sample(0.0);
        assert!((result["happy"] - 0.0).abs() < 1e-5);
        let result2 = tl.sample(1.0);
        assert!((result2["happy"] - 1.0).abs() < 1e-5);
    }

    // 3. sample between two keyframes interpolates correctly
    #[test]
    fn test_sample_between_keyframes() {
        let mut tl = EmotionTimeline::new(2.0, TimelineLoop::Once);
        tl.add_keyframe(make_kf(0.0, &[("happy", 0.0)]));
        tl.add_keyframe(make_kf(2.0, &[("happy", 1.0)]));
        let result = tl.sample(1.0);
        assert!((result["happy"] - 0.5).abs() < 1e-5);
    }

    // 4. sample past end clamps to last keyframe (Once mode)
    #[test]
    fn test_sample_past_end_clamps() {
        let mut tl = EmotionTimeline::new(1.0, TimelineLoop::Once);
        tl.add_keyframe(make_kf(0.0, &[("sad", 0.0)]));
        tl.add_keyframe(make_kf(1.0, &[("sad", 0.8)]));
        let result = tl.sample(99.0);
        assert!((result["sad"] - 0.8).abs() < 1e-5);
    }

    // 5. Loop mode wraps time back to start
    #[test]
    fn test_loop_wraps() {
        let mut tl = EmotionTimeline::new(1.0, TimelineLoop::Loop);
        tl.add_keyframe(make_kf(0.0, &[("anger", 1.0)]));
        tl.add_keyframe(make_kf(1.0, &[("anger", 0.0)]));
        // t=1.25 should wrap to t=0.25 => anger=0.75
        let result = tl.sample(1.25);
        assert!((result["anger"] - 0.75).abs() < 1e-4);
    }

    // 6. PingPong mode bounces
    #[test]
    fn test_ping_pong() {
        let mut tl = EmotionTimeline::new(1.0, TimelineLoop::PingPong);
        tl.add_keyframe(make_kf(0.0, &[("joy", 0.0)]));
        tl.add_keyframe(make_kf(1.0, &[("joy", 1.0)]));
        // t=1.5 → ping-pong → t=0.5 → joy=0.5
        let result = tl.sample(1.5);
        assert!((result["joy"] - 0.5).abs() < 1e-4);
    }

    // 7. bake frame count = ceil(duration*fps)+1
    #[test]
    fn test_bake_frame_count() {
        let mut tl = EmotionTimeline::new(2.0, TimelineLoop::Once);
        tl.add_keyframe(make_kf(0.0, &[("e", 0.0)]));
        tl.add_keyframe(make_kf(2.0, &[("e", 1.0)]));
        let frames = tl.bake(30.0);
        let expected = (2.0_f32 * 30.0).ceil() as usize + 1; // 61
        assert_eq!(frames.len(), expected);
    }

    // 8. interpolate_emotions merges keys from both maps
    #[test]
    fn test_interpolate_emotions_merges_keys() {
        let a: HashMap<String, f32> = [("happy".to_string(), 1.0)].into();
        let b: HashMap<String, f32> = [("sad".to_string(), 1.0)].into();
        let result = interpolate_emotions(&a, &b, 0.5);
        assert!((result["happy"] - 0.5).abs() < 1e-5);
        assert!((result["sad"] - 0.5).abs() < 1e-5);
    }

    // 9. apply_easing_fn Linear is identity
    #[test]
    fn test_easing_linear_identity() {
        for &v in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            assert!((apply_easing_fn(v, &TimelineEasing::Linear) - v).abs() < 1e-6);
        }
    }

    // 10. apply_easing_fn EaseInOut midpoint ≈ 0.5
    #[test]
    fn test_easing_ease_in_out_midpoint() {
        let v = apply_easing_fn(0.5, &TimelineEasing::EaseInOut);
        assert!((v - 0.5).abs() < 1e-5);
    }

    // 11. Step easing returns 0 for t<1 and 1 at t=1
    #[test]
    fn test_easing_step_returns_prior() {
        assert!((apply_easing_fn(0.0, &TimelineEasing::Step) - 0.0).abs() < 1e-6);
        assert!((apply_easing_fn(0.5, &TimelineEasing::Step) - 0.0).abs() < 1e-6);
        assert!((apply_easing_fn(0.999, &TimelineEasing::Step) - 0.0).abs() < 1e-6);
        assert!((apply_easing_fn(1.0, &TimelineEasing::Step) - 1.0).abs() < 1e-6);
    }

    // 12. sample with Step easing holds prior value throughout interval
    #[test]
    fn test_step_easing_holds_prior_value() {
        let mut tl = EmotionTimeline::new(2.0, TimelineLoop::Once);
        tl.add_keyframe(make_kf_easing(0.0, &[("fear", 0.2)], TimelineEasing::Step));
        tl.add_keyframe(make_kf_easing(1.0, &[("fear", 0.8)], TimelineEasing::Step));
        tl.add_keyframe(make_kf_easing(2.0, &[("fear", 0.4)], TimelineEasing::Step));
        // Between 0.0 and 1.0, step easing should hold 0.2 (start value).
        let mid = tl.sample(0.5);
        assert!((mid["fear"] - 0.2).abs() < 1e-5);
    }
}
