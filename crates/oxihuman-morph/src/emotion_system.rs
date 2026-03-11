// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Emotion-driven facial expression mapping.
//!
//! Maps emotional states (Ekman's basic emotions + neutral) to blended collections
//! of facial morph target weights, with per-emotion intensity control and
//! valence-arousal space support.

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Emotion
// ---------------------------------------------------------------------------

/// Primary emotions (Ekman's basic emotions + neutral).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Emotion {
    Neutral,
    Happy,
    Sad,
    Angry,
    Surprised,
    Fearful,
    Disgusted,
    Contempt,
}

impl Emotion {
    /// All emotion variants in canonical order.
    pub fn all() -> &'static [Emotion] {
        use Emotion::*;
        &[
            Neutral, Happy, Sad, Angry, Surprised, Fearful, Disgusted, Contempt,
        ]
    }

    /// Human-readable name for the emotion.
    pub fn name(&self) -> &'static str {
        match self {
            Emotion::Neutral => "neutral",
            Emotion::Happy => "happy",
            Emotion::Sad => "sad",
            Emotion::Angry => "angry",
            Emotion::Surprised => "surprised",
            Emotion::Fearful => "fearful",
            Emotion::Disgusted => "disgusted",
            Emotion::Contempt => "contempt",
        }
    }

    /// Valence: positive = pleasant, negative = unpleasant, 0 = neutral.
    pub fn valence(&self) -> f32 {
        match self {
            Emotion::Neutral => 0.0,
            Emotion::Happy => 1.0,
            Emotion::Sad => -1.0,
            Emotion::Angry => -0.8,
            Emotion::Surprised => 0.2,
            Emotion::Fearful => -0.5,
            Emotion::Disgusted => -0.7,
            Emotion::Contempt => -0.6,
        }
    }

    /// Arousal: high = excited/intense, low = calm/passive.
    pub fn arousal(&self) -> f32 {
        match self {
            Emotion::Neutral => 0.0,
            Emotion::Happy => 0.7,
            Emotion::Sad => -0.4,
            Emotion::Angry => 0.9,
            Emotion::Surprised => 0.8,
            Emotion::Fearful => 0.7,
            Emotion::Disgusted => 0.3,
            Emotion::Contempt => 0.2,
        }
    }
}

// ---------------------------------------------------------------------------
// EmotionExpression
// ---------------------------------------------------------------------------

/// A morph weight map for one emotion at a given intensity.
pub struct EmotionExpression {
    /// The emotion this expression represents.
    pub emotion: Emotion,
    /// Global intensity scalar (0..=1).
    pub intensity: f32,
    /// Morph target name → base weight (before intensity scaling).
    pub weights: HashMap<String, f32>,
}

impl EmotionExpression {
    /// Create a new expression with intensity 1.0 and no morph weights.
    pub fn new(emotion: Emotion) -> Self {
        Self {
            emotion,
            intensity: 1.0,
            weights: HashMap::new(),
        }
    }

    /// Builder: add or replace a morph target weight.
    pub fn with_weight(mut self, morph: impl Into<String>, weight: f32) -> Self {
        self.weights.insert(morph.into(), weight);
        self
    }

    /// Builder: set global intensity.
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Return the effective weights: each base weight multiplied by intensity.
    pub fn effective_weights(&self) -> HashMap<String, f32> {
        self.weights
            .iter()
            .map(|(k, &v)| (k.clone(), v * self.intensity))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// EmotionBlend
// ---------------------------------------------------------------------------

/// A blend of multiple emotions, each with its own blend weight.
pub struct EmotionBlend {
    /// Emotion → blend weight (sum should be ≤ 1.0).
    pub components: HashMap<Emotion, f32>,
}

impl EmotionBlend {
    /// Create an empty blend.
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Create a blend containing a single emotion at the given weight.
    pub fn single(emotion: Emotion, weight: f32) -> Self {
        let mut blend = Self::new();
        blend.components.insert(emotion, weight.clamp(0.0, 1.0));
        blend
    }

    /// Add or accumulate a blend weight for an emotion.
    pub fn add(&mut self, emotion: Emotion, weight: f32) {
        let entry = self.components.entry(emotion).or_insert(0.0);
        *entry = (*entry + weight).clamp(0.0, 1.0);
    }

    /// Scale all weights so they sum to 1.0. No-op if sum is zero.
    pub fn normalize(&mut self) {
        let sum: f32 = self.components.values().copied().sum();
        if sum > f32::EPSILON {
            for v in self.components.values_mut() {
                *v /= sum;
            }
        }
    }

    /// Return the emotion with the highest blend weight, if any.
    pub fn dominant(&self) -> Option<&Emotion> {
        self.components
            .iter()
            .filter(|(_, &w)| w > 0.0)
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(e, _)| e)
    }

    /// Returns `true` when all blend weights are below 0.05 (effectively neutral).
    pub fn is_neutral(&self) -> bool {
        self.components.values().all(|&w| w < 0.05)
    }
}

impl Default for EmotionBlend {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// EmotionSystem
// ---------------------------------------------------------------------------

/// Maps emotions to morph target weights and evaluates blended expressions.
pub struct EmotionSystem {
    expressions: HashMap<Emotion, EmotionExpression>,
}

impl EmotionSystem {
    /// Create an empty emotion system.
    pub fn new() -> Self {
        Self {
            expressions: HashMap::new(),
        }
    }

    /// Register or replace an expression for its emotion.
    pub fn add_expression(&mut self, expr: EmotionExpression) {
        self.expressions.insert(expr.emotion.clone(), expr);
    }

    /// Look up the expression for the given emotion.
    pub fn get_expression(&self, emotion: &Emotion) -> Option<&EmotionExpression> {
        self.expressions.get(emotion)
    }

    /// Evaluate blended morph weights for a given emotion blend.
    ///
    /// For each emotion component with weight > 0, the expression weights are
    /// scaled by `component_weight * expression.intensity`, then additively blended
    /// and clamped to [0, 1].
    pub fn evaluate(&self, blend: &EmotionBlend) -> HashMap<String, f32> {
        let mut result: HashMap<String, f32> = HashMap::new();

        for (emotion, &blend_weight) in &blend.components {
            if blend_weight <= 0.0 {
                continue;
            }
            if let Some(expr) = self.expressions.get(emotion) {
                for (morph, &base_w) in &expr.weights {
                    let contribution = base_w * expr.intensity * blend_weight;
                    let entry = result.entry(morph.clone()).or_insert(0.0);
                    *entry += contribution;
                }
            }
        }

        // Clamp all final values to [0, 1].
        for v in result.values_mut() {
            *v = v.clamp(0.0, 1.0);
        }
        result
    }

    /// Evaluate a single emotion at the given intensity (overrides expression intensity).
    pub fn evaluate_single(&self, emotion: &Emotion, intensity: f32) -> HashMap<String, f32> {
        let intensity = intensity.clamp(0.0, 1.0);
        match self.expressions.get(emotion) {
            None => HashMap::new(),
            Some(expr) => expr
                .weights
                .iter()
                .map(|(k, &v)| (k.clone(), (v * intensity).clamp(0.0, 1.0)))
                .collect(),
        }
    }

    /// Convert a valence-arousal coordinate to an emotion blend using inverse-distance
    /// weighting (IDW) over the 3 nearest emotions in V-A space.
    pub fn from_valence_arousal(&self, valence: f32, arousal: f32) -> EmotionBlend {
        let k = 3usize; // number of nearest neighbours

        // Compute squared Euclidean distances to each emotion's V-A position.
        let mut distances: Vec<(Emotion, f32)> = Emotion::all()
            .iter()
            .map(|e| {
                let dv = e.valence() - valence;
                let da = e.arousal() - arousal;
                let dist = (dv * dv + da * da).sqrt();
                (e.clone(), dist)
            })
            .collect();

        // Sort by distance ascending.
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // If the query is exactly on an emotion, return it with weight 1.
        if distances[0].1 < f32::EPSILON {
            return EmotionBlend::single(distances[0].0.clone(), 1.0);
        }

        // Take the k nearest and compute IDW weights.
        let nearest = &distances[..k.min(distances.len())];
        let inv_dist_sum: f32 = nearest.iter().map(|(_, d)| 1.0 / d).sum();

        let mut blend = EmotionBlend::new();
        for (emotion, dist) in nearest {
            let w = (1.0 / dist) / inv_dist_sum;
            blend.components.insert(emotion.clone(), w);
        }
        blend
    }
}

impl Default for EmotionSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// default_emotion_system
// ---------------------------------------------------------------------------

/// Build a default emotion system with typical MakeHuman-style morph names.
pub fn default_emotion_system() -> EmotionSystem {
    let mut sys = EmotionSystem::new();

    // Neutral — baseline, no morph contribution.
    sys.add_expression(EmotionExpression::new(Emotion::Neutral));

    // Happy
    sys.add_expression(
        EmotionExpression::new(Emotion::Happy)
            .with_weight("smile_mouth", 0.8)
            .with_weight("cheeks_raise", 0.6)
            .with_weight("eyes_squint", 0.3),
    );

    // Sad
    sys.add_expression(
        EmotionExpression::new(Emotion::Sad)
            .with_weight("mouth_frown", 0.7)
            .with_weight("brow_inner_up", 0.5)
            .with_weight("eyes_widen", 0.2),
    );

    // Angry
    sys.add_expression(
        EmotionExpression::new(Emotion::Angry)
            .with_weight("brow_down", 0.8)
            .with_weight("nose_scrunch", 0.5)
            .with_weight("lip_press", 0.6),
    );

    // Surprised
    sys.add_expression(
        EmotionExpression::new(Emotion::Surprised)
            .with_weight("eyes_widen", 0.9)
            .with_weight("brow_raise", 0.8)
            .with_weight("jaw_drop", 0.6),
    );

    // Fearful
    sys.add_expression(
        EmotionExpression::new(Emotion::Fearful)
            .with_weight("eyes_widen", 0.8)
            .with_weight("brow_raise", 0.5)
            .with_weight("mouth_open", 0.4)
            .with_weight("lip_stretch", 0.5),
    );

    // Disgusted
    sys.add_expression(
        EmotionExpression::new(Emotion::Disgusted)
            .with_weight("nose_scrunch", 0.8)
            .with_weight("upper_lip_raise", 0.7)
            .with_weight("brow_down", 0.3),
    );

    // Contempt
    sys.add_expression(
        EmotionExpression::new(Emotion::Contempt)
            .with_weight("lip_corner_pull_r", 0.6)
            .with_weight("cheek_raise_r", 0.3)
            .with_weight("brow_down", 0.2),
    );

    sys
}

// ---------------------------------------------------------------------------
// lerp_emotion_blend
// ---------------------------------------------------------------------------

/// Linearly interpolate between two emotion blends.
///
/// The union of keys from both blends is used; missing keys are treated as 0.
/// `t = 0` returns a clone of `a`; `t = 1` returns a clone of `b`.
pub fn lerp_emotion_blend(a: &EmotionBlend, b: &EmotionBlend, t: f32) -> EmotionBlend {
    let t = t.clamp(0.0, 1.0);
    let mut result = EmotionBlend::new();

    // Collect all emotions from both blends.
    let mut all_emotions: Vec<Emotion> = a.components.keys().cloned().collect();
    for e in b.components.keys() {
        if !all_emotions.contains(e) {
            all_emotions.push(e.clone());
        }
    }

    for emotion in all_emotions {
        let wa = a.components.get(&emotion).copied().unwrap_or(0.0);
        let wb = b.components.get(&emotion).copied().unwrap_or(0.0);
        let w = wa + (wb - wa) * t;
        if w > 0.0 {
            result.components.insert(emotion, w);
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotion_all() {
        let all = Emotion::all();
        assert_eq!(all.len(), 8);
        assert!(all.contains(&Emotion::Neutral));
        assert!(all.contains(&Emotion::Happy));
        assert!(all.contains(&Emotion::Sad));
        assert!(all.contains(&Emotion::Angry));
        assert!(all.contains(&Emotion::Surprised));
        assert!(all.contains(&Emotion::Fearful));
        assert!(all.contains(&Emotion::Disgusted));
        assert!(all.contains(&Emotion::Contempt));
    }

    #[test]
    fn test_emotion_names() {
        assert_eq!(Emotion::Neutral.name(), "neutral");
        assert_eq!(Emotion::Happy.name(), "happy");
        assert_eq!(Emotion::Sad.name(), "sad");
        assert_eq!(Emotion::Angry.name(), "angry");
        assert_eq!(Emotion::Surprised.name(), "surprised");
        assert_eq!(Emotion::Fearful.name(), "fearful");
        assert_eq!(Emotion::Disgusted.name(), "disgusted");
        assert_eq!(Emotion::Contempt.name(), "contempt");
    }

    #[test]
    fn test_emotion_valence_arousal() {
        assert!((Emotion::Neutral.valence() - 0.0).abs() < f32::EPSILON);
        assert!((Emotion::Neutral.arousal() - 0.0).abs() < f32::EPSILON);
        assert!((Emotion::Happy.valence() - 1.0).abs() < f32::EPSILON);
        assert!((Emotion::Happy.arousal() - 0.7).abs() < f32::EPSILON);
        assert!((Emotion::Sad.valence() - (-1.0)).abs() < f32::EPSILON);
        assert!((Emotion::Sad.arousal() - (-0.4)).abs() < f32::EPSILON);
        assert!((Emotion::Angry.valence() - (-0.8)).abs() < f32::EPSILON);
        assert!((Emotion::Angry.arousal() - 0.9).abs() < f32::EPSILON);
        assert!((Emotion::Contempt.valence() - (-0.6)).abs() < f32::EPSILON);
        assert!((Emotion::Contempt.arousal() - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_expression_effective_weights() {
        let expr = EmotionExpression::new(Emotion::Happy)
            .with_weight("smile_mouth", 0.8)
            .with_weight("cheeks_raise", 0.6)
            .with_intensity(0.5);

        let eff = expr.effective_weights();
        let smile = eff["smile_mouth"];
        let cheeks = eff["cheeks_raise"];
        assert!((smile - 0.4).abs() < 1e-5, "smile: {smile}");
        assert!((cheeks - 0.3).abs() < 1e-5, "cheeks: {cheeks}");
    }

    #[test]
    fn test_blend_single() {
        let blend = EmotionBlend::single(Emotion::Happy, 0.7);
        assert_eq!(blend.components.len(), 1);
        assert!((blend.components[&Emotion::Happy] - 0.7).abs() < 1e-5);
    }

    #[test]
    fn test_blend_add() {
        let mut blend = EmotionBlend::new();
        blend.add(Emotion::Happy, 0.4);
        blend.add(Emotion::Sad, 0.3);
        blend.add(Emotion::Happy, 0.2); // accumulates
        assert!((blend.components[&Emotion::Happy] - 0.6).abs() < 1e-5);
        assert!((blend.components[&Emotion::Sad] - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_blend_normalize() {
        let mut blend = EmotionBlend::new();
        blend.components.insert(Emotion::Happy, 0.4);
        blend.components.insert(Emotion::Sad, 0.6);
        blend.normalize();
        let sum: f32 = blend.components.values().sum();
        assert!((sum - 1.0).abs() < 1e-5, "sum after normalize: {sum}");
    }

    #[test]
    fn test_blend_dominant() {
        let mut blend = EmotionBlend::new();
        blend.components.insert(Emotion::Happy, 0.3);
        blend.components.insert(Emotion::Angry, 0.7);
        blend.components.insert(Emotion::Sad, 0.1);
        let dom = blend.dominant().expect("should have dominant");
        assert_eq!(*dom, Emotion::Angry);
    }

    #[test]
    fn test_blend_is_neutral() {
        let mut blend = EmotionBlend::new();
        assert!(blend.is_neutral(), "empty blend is neutral");

        blend.components.insert(Emotion::Happy, 0.04);
        assert!(blend.is_neutral(), "all weights < 0.05 → neutral");

        blend.components.insert(Emotion::Sad, 0.06);
        assert!(!blend.is_neutral(), "weight 0.06 → not neutral");
    }

    #[test]
    fn test_system_evaluate_single() {
        let sys = default_emotion_system();
        let weights = sys.evaluate_single(&Emotion::Happy, 1.0);
        assert!(
            weights.contains_key("smile_mouth"),
            "should contain smile_mouth"
        );
        let smile = weights["smile_mouth"];
        assert!(
            (smile - 0.8).abs() < 1e-5,
            "smile_mouth at full intensity: {smile}"
        );

        let half = sys.evaluate_single(&Emotion::Happy, 0.5);
        let smile_half = half["smile_mouth"];
        assert!(
            (smile_half - 0.4).abs() < 1e-5,
            "smile_mouth at half intensity: {smile_half}"
        );
    }

    #[test]
    fn test_system_evaluate_blend() {
        let sys = default_emotion_system();
        let mut blend = EmotionBlend::new();
        blend.components.insert(Emotion::Happy, 1.0);
        let weights = sys.evaluate(&blend);
        assert!(weights.contains_key("smile_mouth"));
        let smile = weights["smile_mouth"];
        assert!((smile - 0.8).abs() < 1e-5, "blended smile_mouth: {smile}");
    }

    #[test]
    fn test_from_valence_arousal() {
        let sys = default_emotion_system();

        // Query exactly at Happy's V-A position (1.0, 0.7).
        let blend = sys.from_valence_arousal(1.0, 0.7);
        let dom = blend.dominant().expect("should have dominant emotion");
        assert_eq!(*dom, Emotion::Happy, "nearest to (1,0.7) should be Happy");

        // Query at Angry's V-A position (-0.8, 0.9).
        let blend_angry = sys.from_valence_arousal(-0.8, 0.9);
        let dom_angry = blend_angry.dominant().expect("should have dominant");
        assert_eq!(*dom_angry, Emotion::Angry);
    }

    #[test]
    fn test_default_emotion_system() {
        let sys = default_emotion_system();
        for emotion in Emotion::all() {
            assert!(
                sys.get_expression(emotion).is_some(),
                "default system should have expression for {}",
                emotion.name()
            );
        }
        // Happy should have at least 3 morph targets.
        let happy_expr = sys.get_expression(&Emotion::Happy).unwrap();
        assert!(
            happy_expr.weights.len() >= 3,
            "Happy should have at least 3 morph weights"
        );
    }

    #[test]
    fn test_lerp_emotion_blend() {
        let a = EmotionBlend::single(Emotion::Happy, 1.0);
        let b = EmotionBlend::single(Emotion::Sad, 1.0);

        let mid = lerp_emotion_blend(&a, &b, 0.5);
        let happy_w = mid.components.get(&Emotion::Happy).copied().unwrap_or(0.0);
        let sad_w = mid.components.get(&Emotion::Sad).copied().unwrap_or(0.0);
        assert!((happy_w - 0.5).abs() < 1e-5, "happy at t=0.5: {happy_w}");
        assert!((sad_w - 0.5).abs() < 1e-5, "sad at t=0.5: {sad_w}");

        // t=0 should be identical to a.
        let at_zero = lerp_emotion_blend(&a, &b, 0.0);
        assert!((at_zero.components[&Emotion::Happy] - 1.0).abs() < 1e-5);
        assert!(!at_zero.components.contains_key(&Emotion::Sad));

        // t=1 should be identical to b.
        let at_one = lerp_emotion_blend(&a, &b, 1.0);
        assert!(!at_one.components.contains_key(&Emotion::Happy));
        assert!((at_one.components[&Emotion::Sad] - 1.0).abs() < 1e-5);
    }
}
