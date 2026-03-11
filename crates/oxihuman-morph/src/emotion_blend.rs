// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! EmotionBlend — named emotion curves and blending.

#![allow(dead_code)]

/// A single-emotion intensity curve sampled at uniform time steps.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmotionCurve {
    pub name: String,
    /// Intensity samples in [0.0, 1.0], uniformly spaced over [0, duration].
    pub samples: Vec<f32>,
    pub duration: f32,
}

/// A collection of named emotion curves.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct EmotionBlendSet {
    pub curves: Vec<EmotionCurve>,
}

/// Create an empty `EmotionBlendSet`.
#[allow(dead_code)]
pub fn new_emotion_blend_set() -> EmotionBlendSet {
    EmotionBlendSet { curves: Vec::new() }
}

/// Add a named emotion curve with given samples and duration.
#[allow(dead_code)]
pub fn add_emotion_curve(set: &mut EmotionBlendSet, name: &str, samples: Vec<f32>, duration: f32) {
    set.curves.push(EmotionCurve { name: name.to_owned(), samples, duration });
}

/// Evaluate the named emotion curve at time `t` (linear interpolation).
#[allow(dead_code)]
pub fn evaluate_emotion(set: &EmotionBlendSet, name: &str, t: f32) -> f32 {
    for c in &set.curves {
        if c.name == name {
            return sample_curve(c, t);
        }
    }
    0.0
}

fn sample_curve(c: &EmotionCurve, t: f32) -> f32 {
    if c.samples.is_empty() {
        return 0.0;
    }
    let n = c.samples.len();
    if n == 1 {
        return c.samples[0];
    }
    let t_clamped = t.clamp(0.0, c.duration);
    let frac = t_clamped / c.duration.max(f32::EPSILON) * (n - 1) as f32;
    let lo = frac.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let alpha = frac - lo as f32;
    c.samples[lo] * (1.0 - alpha) + c.samples[hi] * alpha
}

/// Linearly blend two named emotions at `t`.
#[allow(dead_code)]
pub fn blend_two_emotions(set: &EmotionBlendSet, name_a: &str, name_b: &str, t: f32, at: f32) -> f32 {
    let a = evaluate_emotion(set, name_a, at);
    let b = evaluate_emotion(set, name_b, at);
    a * (1.0 - t) + b * t
}

/// Return the number of emotion curves in the set.
#[allow(dead_code)]
pub fn emotion_count(set: &EmotionBlendSet) -> usize {
    set.curves.len()
}

/// Return the name of the emotion curve at `index`.
#[allow(dead_code)]
pub fn emotion_name(set: &EmotionBlendSet, index: usize) -> Option<&str> {
    set.curves.get(index).map(|c| c.name.as_str())
}

/// Return the intensity of the named emotion at time `t`.
#[allow(dead_code)]
pub fn emotion_at_time(set: &EmotionBlendSet, name: &str, t: f32) -> f32 {
    evaluate_emotion(set, name, t)
}

/// Return the time at which the named emotion reaches its peak intensity.
#[allow(dead_code)]
pub fn emotion_peak_time(set: &EmotionBlendSet, name: &str) -> f32 {
    for c in &set.curves {
        if c.name == name && !c.samples.is_empty() {
            let (idx, _) = c
                .samples
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, &0.0));
            let n = c.samples.len();
            return idx as f32 / (n - 1).max(1) as f32 * c.duration;
        }
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty_set() {
        let s = new_emotion_blend_set();
        assert_eq!(emotion_count(&s), 0);
    }

    #[test]
    fn test_add_emotion_curve() {
        let mut s = new_emotion_blend_set();
        add_emotion_curve(&mut s, "happy", vec![0.0, 1.0], 1.0);
        assert_eq!(emotion_count(&s), 1);
    }

    #[test]
    fn test_emotion_name() {
        let mut s = new_emotion_blend_set();
        add_emotion_curve(&mut s, "sad", vec![0.5], 1.0);
        assert_eq!(emotion_name(&s, 0), Some("sad"));
        assert!(emotion_name(&s, 1).is_none());
    }

    #[test]
    fn test_evaluate_emotion_start() {
        let mut s = new_emotion_blend_set();
        add_emotion_curve(&mut s, "joy", vec![0.0, 1.0], 1.0);
        let v = evaluate_emotion(&s, "joy", 0.0);
        assert!((v).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_emotion_end() {
        let mut s = new_emotion_blend_set();
        add_emotion_curve(&mut s, "joy", vec![0.0, 1.0], 1.0);
        let v = evaluate_emotion(&s, "joy", 1.0);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_unknown_emotion() {
        let s = new_emotion_blend_set();
        assert_eq!(evaluate_emotion(&s, "unknown", 0.5), 0.0);
    }

    #[test]
    fn test_blend_two_emotions_midpoint() {
        let mut s = new_emotion_blend_set();
        add_emotion_curve(&mut s, "a", vec![0.0, 0.0], 1.0);
        add_emotion_curve(&mut s, "b", vec![1.0, 1.0], 1.0);
        let v = blend_two_emotions(&s, "a", "b", 0.5, 0.5);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_emotion_peak_time() {
        let mut s = new_emotion_blend_set();
        add_emotion_curve(&mut s, "anger", vec![0.0, 0.5, 1.0, 0.5], 3.0);
        let t = emotion_peak_time(&s, "anger");
        assert!((t - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_emotion_at_time_midpoint() {
        let mut s = new_emotion_blend_set();
        add_emotion_curve(&mut s, "x", vec![0.0, 1.0], 2.0);
        let v = emotion_at_time(&s, "x", 1.0);
        assert!((v - 0.5).abs() < 1e-4);
    }
}
