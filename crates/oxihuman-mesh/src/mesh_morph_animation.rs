// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Morph target animation sequence — keyframed blend weights over time.

/// A keyframe mapping morph target weights to a time.
#[derive(Debug, Clone)]
pub struct MorphKeyframe {
    pub time: f32,
    pub weights: Vec<f32>,
}

/// A named morph target.
#[derive(Debug, Clone)]
pub struct MorphTarget {
    pub name: String,
    pub deltas: Vec<[f32; 3]>,
}

/// A sequence of morph keyframes.
#[derive(Debug, Default, Clone)]
pub struct MorphAnimSequence {
    pub targets: Vec<MorphTarget>,
    pub keyframes: Vec<MorphKeyframe>,
    pub frame_rate: f32,
}

impl MorphAnimSequence {
    /// Creates a new morph animation sequence.
    pub fn new(frame_rate: f32) -> Self {
        Self {
            frame_rate,
            ..Default::default()
        }
    }

    /// Adds a morph target.
    pub fn add_target(&mut self, target: MorphTarget) {
        self.targets.push(target);
    }

    /// Adds a keyframe.
    pub fn add_keyframe(&mut self, kf: MorphKeyframe) {
        self.keyframes.push(kf);
        self.keyframes.sort_by(|a, b| {
            a.time
                .partial_cmp(&b.time)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Returns the duration of the sequence.
    pub fn duration(&self) -> f32 {
        self.keyframes.last().map(|k| k.time).unwrap_or(0.0)
    }

    /// Evaluates weights at a given time by linear interpolation.
    pub fn evaluate_weights(&self, time: f32) -> Vec<f32> {
        if self.keyframes.is_empty() {
            return vec![];
        }
        let n = self.targets.len();
        if self.keyframes.len() == 1 {
            return self.keyframes[0].weights.clone();
        }
        let idx = self.keyframes.partition_point(|k| k.time <= time);
        if idx == 0 {
            return self.keyframes[0].weights.clone();
        }
        if idx >= self.keyframes.len() {
            return self.keyframes[self.keyframes.len() - 1].weights.clone();
        }
        let a = &self.keyframes[idx - 1];
        let b = &self.keyframes[idx];
        let span = (b.time - a.time).max(f32::EPSILON);
        let t = ((time - a.time) / span).clamp(0.0, 1.0);
        (0..n)
            .map(|i| {
                let wa = a.weights.get(i).copied().unwrap_or(0.0);
                let wb = b.weights.get(i).copied().unwrap_or(0.0);
                wa + (wb - wa) * t
            })
            .collect()
    }
}

/// Applies morph weights to a base mesh returning blended positions.
pub fn apply_morph_weights(
    base: &[[f32; 3]],
    targets: &[MorphTarget],
    weights: &[f32],
) -> Vec<[f32; 3]> {
    let mut result: Vec<[f32; 3]> = base.to_vec();
    for (target, &w) in targets.iter().zip(weights.iter()) {
        for (i, delta) in target.deltas.iter().enumerate() {
            if i < result.len() {
                result[i][0] += delta[0] * w;
                result[i][1] += delta[1] * w;
                result[i][2] += delta[2] * w;
            }
        }
    }
    result
}

/// Validates that all keyframe weight vectors have the correct length.
pub fn validate_morph_keyframes(seq: &MorphAnimSequence) -> bool {
    let n = seq.targets.len();
    seq.keyframes.iter().all(|k| k.weights.len() == n)
}

/// Returns the target with the given name.
pub fn find_target_by_name<'a>(seq: &'a MorphAnimSequence, name: &str) -> Option<&'a MorphTarget> {
    seq.targets.iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_target(name: &str) -> MorphTarget {
        MorphTarget {
            name: name.to_string(),
            deltas: vec![[0.1, 0.0, 0.0]],
        }
    }

    fn make_kf(t: f32, n: usize) -> MorphKeyframe {
        MorphKeyframe {
            time: t,
            weights: vec![0.0; n],
        }
    }

    #[test]
    fn test_new_sequence_empty() {
        /* New sequence should have no keyframes */
        assert_eq!(MorphAnimSequence::new(24.0).keyframes.len(), 0);
    }

    #[test]
    fn test_add_target() {
        /* Adding a target should increase target count */
        let mut seq = MorphAnimSequence::new(24.0);
        seq.add_target(make_target("blink"));
        assert_eq!(seq.targets.len(), 1);
    }

    #[test]
    fn test_add_keyframe_sorts() {
        /* Keyframes should be sorted by time after adding */
        let mut seq = MorphAnimSequence::new(24.0);
        seq.add_target(make_target("x"));
        seq.add_keyframe(make_kf(2.0, 1));
        seq.add_keyframe(make_kf(0.0, 1));
        assert!(seq.keyframes[0].time <= seq.keyframes[1].time);
    }

    #[test]
    fn test_duration_empty() {
        /* Empty sequence should have zero duration */
        assert_eq!(MorphAnimSequence::new(24.0).duration(), 0.0);
    }

    #[test]
    fn test_evaluate_weights_empty_returns_empty() {
        /* No keyframes → empty weights */
        let seq = MorphAnimSequence::new(24.0);
        assert!(seq.evaluate_weights(0.5).is_empty());
    }

    #[test]
    fn test_apply_morph_weights_no_targets() {
        /* No targets → base mesh unchanged */
        let base = vec![[1.0f32, 0.0, 0.0]];
        let result = apply_morph_weights(&base, &[], &[]);
        assert_eq!(result[0][0], 1.0);
    }

    #[test]
    fn test_apply_morph_weights_with_target() {
        /* Delta of 0.1 with weight 1 should shift x by 0.1 */
        let base = vec![[0.0f32; 3]];
        let targets = vec![make_target("test")];
        let result = apply_morph_weights(&base, &targets, &[1.0]);
        assert!((result[0][0] - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validate_morph_keyframes_valid() {
        /* Correct length keyframes should validate */
        let mut seq = MorphAnimSequence::new(24.0);
        seq.add_target(make_target("a"));
        seq.add_keyframe(make_kf(0.0, 1));
        assert!(validate_morph_keyframes(&seq));
    }

    #[test]
    fn test_find_target_by_name_found() {
        /* Should find target by name */
        let mut seq = MorphAnimSequence::new(24.0);
        seq.add_target(make_target("smile"));
        assert!(find_target_by_name(&seq, "smile").is_some());
    }

    #[test]
    fn test_find_target_by_name_not_found() {
        /* Should return None for unknown target */
        let seq = MorphAnimSequence::new(24.0);
        assert!(find_target_by_name(&seq, "missing").is_none());
    }
}
