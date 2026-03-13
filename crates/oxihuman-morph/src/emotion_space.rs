// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! PAD (Pleasure-Arousal-Dominance) emotion space mapping to facial expression weights.
//!
//! This module provides a three-dimensional emotion space following the
//! Mehrabian & Russell PAD model, with interpolation methods (IDW and RBF/Gaussian)
//! to blend morph target weights across the space.

#![allow(dead_code)]

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// PadPoint
// ---------------------------------------------------------------------------

/// A point in PAD (Pleasure-Arousal-Dominance) space.
///
/// All dimensions are in the range `[-1.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PadPoint {
    /// Pleasure/Valence: -1 = very negative affect, +1 = very positive affect.
    pub pleasure: f32,
    /// Arousal: -1 = completely calm/sleepy, +1 = highly excited/stimulated.
    pub arousal: f32,
    /// Dominance: -1 = submissive/controlled, +1 = dominant/in-control.
    pub dominance: f32,
}

impl PadPoint {
    /// Construct a new PAD point.
    pub fn new(p: f32, a: f32, d: f32) -> Self {
        Self {
            pleasure: p,
            arousal: a,
            dominance: d,
        }
    }

    /// The neutral origin `(0, 0, 0)`.
    pub fn neutral() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// Euclidean distance to another PAD point.
    pub fn distance(&self, other: &PadPoint) -> f32 {
        let dp = self.pleasure - other.pleasure;
        let da = self.arousal - other.arousal;
        let dd = self.dominance - other.dominance;
        (dp * dp + da * da + dd * dd).sqrt()
    }

    /// Linear interpolation between `self` and `other` by factor `t` (0 = self, 1 = other).
    pub fn lerp(&self, other: &PadPoint, t: f32) -> PadPoint {
        PadPoint {
            pleasure: self.pleasure + (other.pleasure - self.pleasure) * t,
            arousal: self.arousal + (other.arousal - self.arousal) * t,
            dominance: self.dominance + (other.dominance - self.dominance) * t,
        }
    }

    /// Clamp all dimensions to `[-1.0, 1.0]`.
    pub fn clamp(&self) -> PadPoint {
        PadPoint {
            pleasure: self.pleasure.clamp(-1.0, 1.0),
            arousal: self.arousal.clamp(-1.0, 1.0),
            dominance: self.dominance.clamp(-1.0, 1.0),
        }
    }

    // -----------------------------------------------------------------------
    // Named emotion anchors (Mehrabian PAD coordinates)
    // -----------------------------------------------------------------------

    /// Happy: high pleasure, moderate arousal, slightly dominant.
    pub fn happy() -> Self {
        Self::new(0.8, 0.5, 0.3)
    }

    /// Sad: negative pleasure, low arousal, slightly submissive.
    pub fn sad() -> Self {
        Self::new(-0.6, -0.4, -0.4)
    }

    /// Angry: negative pleasure, high arousal, dominant.
    pub fn angry() -> Self {
        Self::new(-0.5, 0.8, 0.7)
    }

    /// Fearful: negative pleasure, high arousal, submissive.
    pub fn fearful() -> Self {
        Self::new(-0.6, 0.7, -0.6)
    }

    /// Surprised: mildly positive pleasure, high arousal, slightly submissive.
    pub fn surprised() -> Self {
        Self::new(0.1, 0.8, -0.3)
    }

    /// Disgusted: negative pleasure, moderate arousal, slightly dominant.
    pub fn disgusted() -> Self {
        Self::new(-0.7, 0.3, 0.4)
    }

    /// Contemptuous: mildly negative pleasure, low arousal, dominant.
    pub fn contemptuous() -> Self {
        Self::new(-0.1, 0.2, 0.6)
    }
}

// ---------------------------------------------------------------------------
// EmotionAnchor
// ---------------------------------------------------------------------------

/// A named anchor point in PAD space with associated facial morph target weights.
pub struct EmotionAnchor {
    /// Human-readable emotion name (e.g. `"happy"`, `"sad"`).
    pub name: String,
    /// Position in PAD space.
    pub pad: PadPoint,
    /// Map from morph target name → weight `[0.0, 1.0]`.
    pub morph_weights: HashMap<String, f32>,
}

impl EmotionAnchor {
    /// Create a new anchor.
    pub fn new(name: &str, pad: PadPoint, morph_weights: HashMap<String, f32>) -> Self {
        Self {
            name: name.to_string(),
            pad,
            morph_weights,
        }
    }
}

// ---------------------------------------------------------------------------
// EmotionSpace
// ---------------------------------------------------------------------------

/// A collection of [`EmotionAnchor`]s that defines an interpolatable emotion space.
pub struct EmotionSpace {
    anchors: Vec<EmotionAnchor>,
}

impl EmotionSpace {
    /// Create an empty emotion space.
    pub fn new() -> Self {
        Self {
            anchors: Vec::new(),
        }
    }

    /// Add an anchor to the space.
    pub fn add_anchor(&mut self, anchor: EmotionAnchor) {
        self.anchors.push(anchor);
    }

    /// Return the number of anchors.
    pub fn anchor_count(&self) -> usize {
        self.anchors.len()
    }

    /// Find an anchor by name (case-sensitive).
    pub fn find_anchor(&self, name: &str) -> Option<&EmotionAnchor> {
        self.anchors.iter().find(|a| a.name == name)
    }

    /// Build the default space with Ekman's seven basic emotions mapped to PAD space.
    ///
    /// Each emotion has a canonical set of facial morph target weights drawn from
    /// common MakeHuman-style expression unit names.
    pub fn default_space() -> Self {
        let mut space = Self::new();

        // Neutral
        space.add_anchor(EmotionAnchor::new(
            "neutral",
            PadPoint::neutral(),
            HashMap::new(),
        ));

        // Happy
        space.add_anchor(EmotionAnchor::new(
            "happy",
            PadPoint::happy(),
            [
                ("mouth-corner-puller", 0.85),
                ("mouth-elevation", 0.6),
                ("cheek-raiser", 0.5),
                ("eye-squint", 0.3),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        ));

        // Sad
        space.add_anchor(EmotionAnchor::new(
            "sad",
            PadPoint::sad(),
            [
                ("brow-lowerer", 0.4),
                ("inner-brow-raiser", 0.7),
                ("lip-corner-depressor", 0.8),
                ("lower-lip-depressor", 0.4),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        ));

        // Angry
        space.add_anchor(EmotionAnchor::new(
            "angry",
            PadPoint::angry(),
            [
                ("brow-lowerer", 0.9),
                ("nose-wrinkler", 0.5),
                ("upper-lip-raiser", 0.6),
                ("lip-tightener", 0.7),
                ("jaw-drop", 0.2),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        ));

        // Fearful
        space.add_anchor(EmotionAnchor::new(
            "fearful",
            PadPoint::fearful(),
            [
                ("inner-brow-raiser", 0.8),
                ("brow-raiser", 0.6),
                ("eye-widener", 0.7),
                ("lip-corner-puller", 0.4),
                ("jaw-drop", 0.5),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        ));

        // Surprised
        space.add_anchor(EmotionAnchor::new(
            "surprised",
            PadPoint::surprised(),
            [
                ("brow-raiser", 0.9),
                ("eye-widener", 0.8),
                ("jaw-drop", 0.7),
                ("upper-lip-raiser", 0.3),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        ));

        // Disgusted
        space.add_anchor(EmotionAnchor::new(
            "disgusted",
            PadPoint::disgusted(),
            [
                ("nose-wrinkler", 0.9),
                ("upper-lip-raiser", 0.8),
                ("brow-lowerer", 0.4),
                ("lower-lip-depressor", 0.3),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        ));

        // Contemptuous
        space.add_anchor(EmotionAnchor::new(
            "contemptuous",
            PadPoint::contemptuous(),
            [
                ("lip-corner-puller", 0.5), // unilateral
                ("brow-lowerer", 0.3),
                ("eye-narrower", 0.4),
            ]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        ));

        space
    }

    // -----------------------------------------------------------------------
    // Evaluation
    // -----------------------------------------------------------------------

    /// Evaluate expression weights at `pad` using inverse-distance weighting (IDW).
    ///
    /// Uses `power = 2.0` (standard Shepard's method).
    pub fn evaluate(&self, pad: &PadPoint) -> HashMap<String, f32> {
        self.evaluate_idw(pad, 2.0)
    }

    /// Evaluate with configurable IDW power.
    fn evaluate_idw(&self, pad: &PadPoint, power: f32) -> HashMap<String, f32> {
        if self.anchors.is_empty() {
            return HashMap::new();
        }

        // Collect (weight, morph_weights) pairs
        let mut weighted: Vec<(f32, &HashMap<String, f32>)> =
            Vec::with_capacity(self.anchors.len());
        let mut weight_sum = 0.0_f32;
        let mut exact_match: Option<&HashMap<String, f32>> = None;

        for anchor in &self.anchors {
            let d = pad.distance(&anchor.pad);
            if d < 1e-7 {
                exact_match = Some(&anchor.morph_weights);
                break;
            }
            let w = idw_weight(d, power);
            weight_sum += w;
            weighted.push((w, &anchor.morph_weights));
        }

        if let Some(exact) = exact_match {
            return exact.clone();
        }

        if weight_sum < 1e-12 {
            return HashMap::new();
        }

        let mut result: HashMap<String, f32> = HashMap::new();
        for (w, mw) in &weighted {
            for (key, val) in *mw {
                *result.entry(key.clone()).or_insert(0.0) += w * val;
            }
        }
        for v in result.values_mut() {
            *v /= weight_sum;
            *v = v.clamp(0.0, 1.0);
        }
        result
    }

    /// Evaluate expression weights using Gaussian RBF interpolation.
    ///
    /// `sigma` controls the width of influence: smaller = sharper, larger = broader.
    pub fn evaluate_rbf(&self, pad: &PadPoint, sigma: f32) -> HashMap<String, f32> {
        if self.anchors.is_empty() {
            return HashMap::new();
        }

        let sigma2 = sigma * sigma;
        let mut weights: Vec<f32> = Vec::with_capacity(self.anchors.len());
        let mut weight_sum = 0.0_f32;

        for anchor in &self.anchors {
            let d2 = {
                let dp = pad.pleasure - anchor.pad.pleasure;
                let da = pad.arousal - anchor.pad.arousal;
                let dd = pad.dominance - anchor.pad.dominance;
                dp * dp + da * da + dd * dd
            };
            let w = (-d2 / (2.0 * sigma2)).exp();
            weights.push(w);
            weight_sum += w;
        }

        if weight_sum < 1e-12 {
            return HashMap::new();
        }

        let mut result: HashMap<String, f32> = HashMap::new();
        for (anchor, w) in self.anchors.iter().zip(weights.iter()) {
            for (key, val) in &anchor.morph_weights {
                *result.entry(key.clone()).or_insert(0.0) += w * val;
            }
        }
        for v in result.values_mut() {
            *v /= weight_sum;
            *v = v.clamp(0.0, 1.0);
        }
        result
    }

    /// Find the nearest anchor to `pad` in PAD space.
    pub fn nearest_anchor(&self, pad: &PadPoint) -> Option<&EmotionAnchor> {
        self.anchors.iter().min_by(|a, b| {
            pad.distance(&a.pad)
                .partial_cmp(&pad.distance(&b.pad))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Blend expression weights along the valence (pleasure) axis at `valence ∈ [-1, 1]`.
    ///
    /// Arousal and dominance are held at 0.
    pub fn valence_blend(&self, valence: f32) -> HashMap<String, f32> {
        let pad = PadPoint::new(valence, 0.0, 0.0);
        self.evaluate(&pad)
    }

    /// Blend expression weights along the arousal axis at `arousal ∈ [-1, 1]`.
    ///
    /// Pleasure and dominance are held at 0.
    pub fn arousal_blend(&self, arousal: f32) -> HashMap<String, f32> {
        let pad = PadPoint::new(0.0, arousal, 0.0);
        self.evaluate(&pad)
    }
}

impl Default for EmotionSpace {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// EmotionTransition
// ---------------------------------------------------------------------------

/// A time-parameterised transition between two PAD points.
pub struct EmotionTransition {
    /// Starting PAD state.
    pub from: PadPoint,
    /// Target PAD state.
    pub to: PadPoint,
    /// Total transition duration in seconds.
    pub duration_seconds: f32,
}

impl EmotionTransition {
    /// Create a new transition.
    pub fn new(from: PadPoint, to: PadPoint, duration: f32) -> Self {
        Self {
            from,
            to,
            duration_seconds: duration.max(1e-6),
        }
    }

    /// Evaluate the transition at `t_seconds` using **linear** interpolation.
    ///
    /// `t_seconds` is clamped to `[0, duration_seconds]`.
    pub fn evaluate(&self, t_seconds: f32) -> PadPoint {
        let t = (t_seconds / self.duration_seconds).clamp(0.0, 1.0);
        self.from.lerp(&self.to, t)
    }

    /// Evaluate the transition at `t_seconds` using **smoothstep** interpolation.
    ///
    /// Provides ease-in / ease-out behaviour.  `t_seconds` is clamped to the
    /// duration range.
    pub fn evaluate_smooth(&self, t_seconds: f32) -> PadPoint {
        let t = (t_seconds / self.duration_seconds).clamp(0.0, 1.0);
        let s = t * t * (3.0 - 2.0 * t); // smoothstep
        self.from.lerp(&self.to, s)
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Return a human-readable description of a PAD point based on its quadrant.
pub fn pad_to_description(pad: &PadPoint) -> &'static str {
    match (
        pad.pleasure >= 0.0,
        pad.arousal >= 0.0,
        pad.dominance >= 0.0,
    ) {
        (true, true, true) => "happy/excited/dominant",
        (true, true, false) => "happy/excited/submissive",
        (true, false, true) => "happy/calm/dominant",
        (true, false, false) => "happy/calm/submissive",
        (false, true, true) => "unhappy/excited/dominant",
        (false, true, false) => "unhappy/excited/submissive",
        (false, false, true) => "unhappy/calm/dominant",
        (false, false, false) => "unhappy/calm/submissive",
    }
}

/// Compute the inverse-distance weighting kernel value for a given `distance` and `power`.
///
/// Returns `1.0 / distance.powf(power)`. Caller should guard against `distance ≈ 0`.
pub fn idw_weight(distance: f32, power: f32) -> f32 {
    if distance < 1e-12 {
        f32::MAX
    } else {
        1.0 / distance.powf(power)
    }
}

/// Linearly mix two expression weight maps.
///
/// For each key present in either map: `result[k] = a[k] * (1 - t) + b[k] * t`.
/// The result is then clamped to `[0.0, 1.0]`.
pub fn mix_expressions(
    a: &HashMap<String, f32>,
    b: &HashMap<String, f32>,
    t: f32,
) -> HashMap<String, f32> {
    let t = t.clamp(0.0, 1.0);
    let one_minus_t = 1.0 - t;

    let mut result: HashMap<String, f32> = HashMap::new();

    for (k, va) in a {
        let vb = b.get(k).copied().unwrap_or(0.0);
        result.insert(k.clone(), (va * one_minus_t + vb * t).clamp(0.0, 1.0));
    }
    for (k, vb) in b {
        if !result.contains_key(k) {
            let va = a.get(k).copied().unwrap_or(0.0);
            result.insert(k.clone(), (va * one_minus_t + vb * t).clamp(0.0, 1.0));
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
    use std::fs;

    fn write_tmp(name: &str, content: &str) {
        let path = format!("/tmp/{name}");
        fs::write(&path, content).expect("write tmp file");
    }

    // 1. PadPoint::neutral is (0,0,0)
    #[test]
    fn test_pad_neutral() {
        let n = PadPoint::neutral();
        assert_eq!(n.pleasure, 0.0);
        assert_eq!(n.arousal, 0.0);
        assert_eq!(n.dominance, 0.0);
        write_tmp("test_pad_neutral.txt", "ok");
    }

    // 2. Named emotion constructors return correct values
    #[test]
    fn test_named_emotions() {
        let h = PadPoint::happy();
        assert!((h.pleasure - 0.8).abs() < 1e-6);
        assert!((h.arousal - 0.5).abs() < 1e-6);
        assert!((h.dominance - 0.3).abs() < 1e-6);

        let s = PadPoint::sad();
        assert!(s.pleasure < 0.0);

        let a = PadPoint::angry();
        assert!(a.arousal > 0.5);
        assert!(a.dominance > 0.5);
        write_tmp("test_named_emotions.txt", "ok");
    }

    // 3. Distance between two equal points is 0
    #[test]
    fn test_distance_same() {
        let p = PadPoint::new(0.3, -0.1, 0.5);
        assert!(p.distance(&p) < 1e-6);
        write_tmp("test_distance_same.txt", "ok");
    }

    // 4. Distance between happy and sad is nonzero
    #[test]
    fn test_distance_nonzero() {
        let d = PadPoint::happy().distance(&PadPoint::sad());
        assert!(d > 0.5, "distance should be substantial: {d}");
        write_tmp("test_distance_nonzero.txt", "ok");
    }

    // 5. lerp midpoint
    #[test]
    fn test_lerp_midpoint() {
        let a = PadPoint::new(0.0, 0.0, 0.0);
        let b = PadPoint::new(1.0, 1.0, 1.0);
        let mid = a.lerp(&b, 0.5);
        assert!((mid.pleasure - 0.5).abs() < 1e-6);
        assert!((mid.arousal - 0.5).abs() < 1e-6);
        assert!((mid.dominance - 0.5).abs() < 1e-6);
        write_tmp("test_lerp_midpoint.txt", "ok");
    }

    // 6. lerp endpoints
    #[test]
    fn test_lerp_endpoints() {
        let a = PadPoint::happy();
        let b = PadPoint::sad();
        let at0 = a.lerp(&b, 0.0);
        let at1 = a.lerp(&b, 1.0);
        assert!((at0.pleasure - a.pleasure).abs() < 1e-6);
        assert!((at1.pleasure - b.pleasure).abs() < 1e-6);
        write_tmp("test_lerp_endpoints.txt", "ok");
    }

    // 7. clamp brings out-of-range values into [-1,1]
    #[test]
    fn test_clamp() {
        let p = PadPoint::new(2.5, -3.0, 0.5).clamp();
        assert!((p.pleasure - 1.0).abs() < 1e-6);
        assert!((p.arousal + 1.0).abs() < 1e-6);
        assert!((p.dominance - 0.5).abs() < 1e-6);
        write_tmp("test_clamp.txt", "ok");
    }

    // 8. EmotionSpace::default_space has 8 anchors
    #[test]
    fn test_default_space_count() {
        let space = EmotionSpace::default_space();
        assert_eq!(space.anchor_count(), 8);
        write_tmp("test_default_space_count.txt", "ok");
    }

    // 9. find_anchor finds by name
    #[test]
    fn test_find_anchor() {
        let space = EmotionSpace::default_space();
        assert!(space.find_anchor("happy").is_some());
        assert!(space.find_anchor("nonexistent").is_none());
        write_tmp("test_find_anchor.txt", "ok");
    }

    // 10. evaluate at anchor position returns its weights (approximately)
    #[test]
    fn test_evaluate_at_anchor() {
        let space = EmotionSpace::default_space();
        let happy_pad = PadPoint::happy();
        let weights = space.evaluate(&happy_pad);
        // Should have some weights since we are very close to the happy anchor
        assert!(
            !weights.is_empty(),
            "weights should not be empty near happy anchor"
        );
        write_tmp("test_evaluate_at_anchor.txt", "ok");
    }

    // 11. evaluate_rbf returns values in [0, 1]
    #[test]
    fn test_evaluate_rbf_range() {
        let space = EmotionSpace::default_space();
        let pad = PadPoint::new(0.2, 0.1, -0.1);
        let weights = space.evaluate_rbf(&pad, 0.5);
        for (k, v) in &weights {
            assert!(*v >= 0.0 && *v <= 1.0, "weight for {k} out of range: {v}");
        }
        write_tmp("test_evaluate_rbf_range.txt", "ok");
    }

    // 12. nearest_anchor for happy PAD point
    #[test]
    fn test_nearest_anchor() {
        let space = EmotionSpace::default_space();
        let near = space.nearest_anchor(&PadPoint::happy());
        assert!(near.is_some());
        assert_eq!(near.expect("should succeed").name, "happy");
        write_tmp("test_nearest_anchor.txt", "ok");
    }

    // 13. EmotionTransition linear at t=0, t=duration, t=mid
    #[test]
    fn test_transition_linear() {
        let from = PadPoint::neutral();
        let to = PadPoint::happy();
        let tr = EmotionTransition::new(from, to, 2.0);

        let at0 = tr.evaluate(0.0);
        assert!((at0.pleasure - from.pleasure).abs() < 1e-6);

        let at2 = tr.evaluate(2.0);
        assert!((at2.pleasure - to.pleasure).abs() < 1e-6);

        let at1 = tr.evaluate(1.0);
        assert!(
            (at1.pleasure - 0.4).abs() < 1e-5,
            "mid pleasure: {}",
            at1.pleasure
        );
        write_tmp("test_transition_linear.txt", "ok");
    }

    // 14. EmotionTransition smooth is different from linear in the interior
    #[test]
    fn test_transition_smooth() {
        let from = PadPoint::neutral();
        let to = PadPoint::happy();
        let tr = EmotionTransition::new(from, to, 2.0);

        // At the endpoints, smooth == linear
        let s0 = tr.evaluate_smooth(0.0);
        assert!((s0.pleasure - from.pleasure).abs() < 1e-6);

        let s2 = tr.evaluate_smooth(2.0);
        assert!((s2.pleasure - to.pleasure).abs() < 1e-6);

        // At t=0.25 * duration (t=0.5 s, normalised 0.25), smoothstep != linear
        let lin = tr.evaluate(0.5);
        let smooth = tr.evaluate_smooth(0.5);
        // smoothstep at 0.25 = 0.25^2*(3-2*0.25) = 0.0625*2.5 = 0.15625 < 0.25 (linear)
        assert!(
            smooth.pleasure < lin.pleasure,
            "smooth should lag linear in first half: smooth={}, lin={}",
            smooth.pleasure,
            lin.pleasure
        );
        write_tmp("test_transition_smooth.txt", "ok");
    }

    // 15. pad_to_description covers all octants
    #[test]
    fn test_pad_to_description() {
        let desc = pad_to_description(&PadPoint::happy());
        assert!(desc.contains("happy"), "desc: {desc}");

        let desc2 = pad_to_description(&PadPoint::new(-0.5, -0.5, -0.5));
        assert!(desc2.contains("unhappy"));
        write_tmp("test_pad_to_description.txt", "ok");
    }

    // 16. idw_weight decreases as distance increases
    #[test]
    fn test_idw_weight_decreasing() {
        let w1 = idw_weight(1.0, 2.0);
        let w2 = idw_weight(2.0, 2.0);
        let w3 = idw_weight(4.0, 2.0);
        assert!(w1 > w2, "w1={w1} w2={w2}");
        assert!(w2 > w3, "w2={w2} w3={w3}");
        write_tmp("test_idw_weight_decreasing.txt", "ok");
    }

    // 17. mix_expressions at t=0 returns a, at t=1 returns b
    #[test]
    fn test_mix_expressions_endpoints() {
        let a: HashMap<String, f32> = [("smile", 0.8_f32), ("brow", 0.2_f32)]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        let b: HashMap<String, f32> = [("smile", 0.0_f32), ("brow", 1.0_f32)]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();

        let r0 = mix_expressions(&a, &b, 0.0);
        assert!((r0["smile"] - 0.8).abs() < 1e-6);

        let r1 = mix_expressions(&a, &b, 1.0);
        assert!((r1["smile"] - 0.0).abs() < 1e-6);
        assert!((r1["brow"] - 1.0).abs() < 1e-6);
        write_tmp("test_mix_expressions_endpoints.txt", "ok");
    }

    // 18. mix_expressions midpoint
    #[test]
    fn test_mix_expressions_midpoint() {
        let a: HashMap<String, f32> = [("x", 0.0_f32)]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        let b: HashMap<String, f32> = [("x", 1.0_f32)]
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        let mid = mix_expressions(&a, &b, 0.5);
        assert!((mid["x"] - 0.5).abs() < 1e-6);
        write_tmp("test_mix_expressions_midpoint.txt", "ok");
    }

    // 19. valence_blend and arousal_blend produce non-empty results
    #[test]
    fn test_axis_blends() {
        let space = EmotionSpace::default_space();
        let vw = space.valence_blend(0.8);
        assert!(!vw.is_empty(), "valence_blend should produce weights");
        let aw = space.arousal_blend(-0.5);
        assert!(!aw.is_empty(), "arousal_blend should produce weights");
        write_tmp("test_axis_blends.txt", "ok");
    }
}
