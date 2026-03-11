#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Blend shapes driven by speech / phoneme data.

/// A single phoneme → blend-shape mapping.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SpeechBlendShape {
    pub phoneme: String,
    pub morph_index: usize,
    pub weight: f32,
}

/// A collection of `SpeechBlendShape` entries.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SpeechBlendSet {
    pub shapes: Vec<SpeechBlendShape>,
}

/// Create an empty `SpeechBlendSet`.
#[allow(dead_code)]
pub fn new_speech_blend_set() -> SpeechBlendSet {
    SpeechBlendSet::default()
}

/// Register a phoneme → morph index mapping.
#[allow(dead_code)]
pub fn add_phoneme_shape(sbs: &mut SpeechBlendSet, phoneme: &str, idx: usize, weight: f32) {
    sbs.shapes.push(SpeechBlendShape {
        phoneme: phoneme.to_string(),
        morph_index: idx,
        weight: weight.clamp(0.0, 1.0),
    });
}

/// Apply all shapes matching `phoneme` to `out`, accumulating their weights.
#[allow(dead_code)]
pub fn apply_phoneme(sbs: &SpeechBlendSet, phoneme: &str, out: &mut [f32]) {
    for shape in sbs.shapes.iter().filter(|s| s.phoneme == phoneme) {
        if shape.morph_index < out.len() {
            out[shape.morph_index] += shape.weight;
        }
    }
}

/// Return the number of registered shapes.
#[allow(dead_code)]
pub fn phoneme_count(sbs: &SpeechBlendSet) -> usize {
    sbs.shapes.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_is_empty() {
        let sbs = new_speech_blend_set();
        assert_eq!(phoneme_count(&sbs), 0);
    }

    #[test]
    fn add_phoneme_increments_count() {
        let mut sbs = new_speech_blend_set();
        add_phoneme_shape(&mut sbs, "AA", 0, 0.8);
        assert_eq!(phoneme_count(&sbs), 1);
    }

    #[test]
    fn apply_phoneme_sets_weight() {
        let mut sbs = new_speech_blend_set();
        add_phoneme_shape(&mut sbs, "AA", 2, 0.7);
        let mut out = vec![0.0_f32; 5];
        apply_phoneme(&sbs, "AA", &mut out);
        assert!((out[2] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn apply_phoneme_no_match_no_change() {
        let mut sbs = new_speech_blend_set();
        add_phoneme_shape(&mut sbs, "AA", 0, 0.9);
        let mut out = vec![0.0_f32; 5];
        apply_phoneme(&sbs, "IY", &mut out);
        assert!(out.iter().all(|&v| v.abs() < 1e-6));
    }

    #[test]
    fn apply_phoneme_accumulates_multiple_shapes() {
        let mut sbs = new_speech_blend_set();
        add_phoneme_shape(&mut sbs, "M", 0, 0.4);
        add_phoneme_shape(&mut sbs, "M", 0, 0.3);
        let mut out = vec![0.0_f32; 2];
        apply_phoneme(&sbs, "M", &mut out);
        assert!((out[0] - 0.7).abs() < 1e-6);
    }

    #[test]
    fn apply_phoneme_out_of_range_no_panic() {
        let mut sbs = new_speech_blend_set();
        add_phoneme_shape(&mut sbs, "AE", 99, 1.0);
        let mut out = vec![0.0_f32; 4];
        apply_phoneme(&sbs, "AE", &mut out); // must not panic
    }

    #[test]
    fn weight_clamped_to_one() {
        let mut sbs = new_speech_blend_set();
        add_phoneme_shape(&mut sbs, "X", 0, 5.0);
        assert!((sbs.shapes[0].weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn weight_clamped_below_zero() {
        let mut sbs = new_speech_blend_set();
        add_phoneme_shape(&mut sbs, "X", 0, -1.0);
        assert!((sbs.shapes[0].weight - 0.0).abs() < 1e-6);
    }

    #[test]
    fn phoneme_count_multiple() {
        let mut sbs = new_speech_blend_set();
        for i in 0..6usize {
            add_phoneme_shape(&mut sbs, "P", i, 0.5);
        }
        assert_eq!(phoneme_count(&sbs), 6);
    }

    #[test]
    fn apply_phoneme_empty_set_no_panic() {
        let sbs = new_speech_blend_set();
        let mut out = vec![0.0_f32; 4];
        apply_phoneme(&sbs, "AA", &mut out);
    }
}
