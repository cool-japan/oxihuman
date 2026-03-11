// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Micro-expression injection layer: transient involuntary expression flashes.

use std::collections::HashMap;

// ── Types ────────────────────────────────────────────────────────────────────

/// A single micro-expression definition.
#[allow(dead_code)]
pub struct MicroExpression {
    /// Human-readable name, e.g. "disgust_flash".
    pub name: String,
    /// Morph target weights for this expression.
    pub morph_weights: HashMap<String, f32>,
    /// Typical duration in seconds (0.04..0.2).
    pub duration: f32,
    /// Intensity multiplier 0..1.
    pub intensity: f32,
}

/// A scheduled micro-expression event in a timeline.
#[allow(dead_code)]
pub struct MicroExpressionEvent {
    /// The micro-expression to play.
    pub expr: MicroExpression,
    /// When (seconds) to start the event.
    pub trigger_time: f32,
    /// Fade-in duration in seconds.
    pub fade_in: f32,
    /// Fade-out duration in seconds.
    pub fade_out: f32,
}

/// A layer that applies micro-expressions on top of a base weight state.
#[allow(dead_code)]
pub struct MicroExpressionLayer {
    /// Scheduled events.
    pub events: Vec<MicroExpressionEvent>,
    /// Base morph weights (always present).
    pub base_weights: HashMap<String, f32>,
}

// ── Implementations ──────────────────────────────────────────────────────────

impl MicroExpressionLayer {
    /// Create a new layer with the given base weights and no events.
    #[allow(dead_code)]
    pub fn new(base_weights: HashMap<String, f32>) -> Self {
        Self {
            events: Vec::new(),
            base_weights,
        }
    }

    /// Schedule a new micro-expression event.
    #[allow(dead_code)]
    pub fn add_event(&mut self, event: MicroExpressionEvent) {
        self.events.push(event);
    }

    /// Sample the combined morph weights at time `t`.
    #[allow(dead_code)]
    pub fn sample(&self, t: f32) -> HashMap<String, f32> {
        let mut result = self.base_weights.clone();

        for event in &self.events {
            let blend = micro_expr_weight_at(event, t);
            if blend > 0.0 {
                result = merge_weights(&result, &event.expr.morph_weights, blend);
            }
        }
        result
    }
}

// ── Free functions ────────────────────────────────────────────────────────────

/// Compute the trapezoid envelope weight for a micro-expression event at time `t`.
/// Returns a value in 0..=`intensity`.
#[allow(dead_code)]
pub fn micro_expr_weight_at(event: &MicroExpressionEvent, t: f32) -> f32 {
    let start = event.trigger_time;
    let end = start + event.expr.duration;

    // Before or after the event window (including fades).
    if t < start - event.fade_in || t > end + event.fade_out {
        return 0.0;
    }

    let env = if t < start {
        // Fade-in phase.
        let elapsed = t - (start - event.fade_in);
        (elapsed / event.fade_in).clamp(0.0, 1.0)
    } else if t <= end {
        // Plateau phase.
        1.0
    } else {
        // Fade-out phase.
        let elapsed = t - end;
        1.0 - (elapsed / event.fade_out).clamp(0.0, 1.0)
    };

    env * event.expr.intensity
}

/// Additively blend `overlay` onto `base` with a `blend` factor.
/// Result is clamped to 1.0 per key.
#[allow(dead_code)]
pub fn merge_weights(
    base: &HashMap<String, f32>,
    overlay: &HashMap<String, f32>,
    blend: f32,
) -> HashMap<String, f32> {
    let mut result = base.clone();
    for (k, &v) in overlay {
        let existing = result.get(k).copied().unwrap_or(0.0);
        result.insert(k.clone(), (existing + v * blend).clamp(0.0, 1.0));
    }
    result
}

/// Return a standard library of named micro expressions.
#[allow(dead_code)]
pub fn standard_micro_expressions() -> Vec<MicroExpression> {
    vec![
        MicroExpression {
            name: "disgust_flash".to_string(),
            morph_weights: [
                ("nose_wrinkle".to_string(), 0.8),
                ("upper_lip_raise".to_string(), 0.7),
                ("brow_lower_inner".to_string(), 0.5),
            ]
            .into_iter()
            .collect(),
            duration: 0.12,
            intensity: 0.85,
        },
        MicroExpression {
            name: "fear_flash".to_string(),
            morph_weights: [
                ("brow_raise_inner".to_string(), 0.9),
                ("eye_widen".to_string(), 0.8),
                ("lip_stretch".to_string(), 0.6),
            ]
            .into_iter()
            .collect(),
            duration: 0.10,
            intensity: 0.80,
        },
        MicroExpression {
            name: "surprise_flash".to_string(),
            morph_weights: [
                ("brow_raise_outer".to_string(), 0.95),
                ("eye_widen".to_string(), 0.9),
                ("jaw_drop".to_string(), 0.7),
            ]
            .into_iter()
            .collect(),
            duration: 0.08,
            intensity: 0.90,
        },
        MicroExpression {
            name: "contempt_flash".to_string(),
            morph_weights: [
                ("lip_corner_pull_right".to_string(), 0.7),
                ("brow_lower_right".to_string(), 0.4),
            ]
            .into_iter()
            .collect(),
            duration: 0.15,
            intensity: 0.75,
        },
        MicroExpression {
            name: "joy_flash".to_string(),
            morph_weights: [
                ("cheek_raise".to_string(), 0.8),
                ("lip_corner_pull".to_string(), 0.9),
                ("eye_squint".to_string(), 0.6),
            ]
            .into_iter()
            .collect(),
            duration: 0.18,
            intensity: 0.70,
        },
    ]
}

/// Inject random micro-expressions into `layer` over `duration` seconds at `rate`
/// events/second using a simple LCG for deterministic randomness from `seed`.
#[allow(dead_code)]
pub fn inject_random_micros(layer: &mut MicroExpressionLayer, duration: f32, rate: f32, seed: u64) {
    let library = standard_micro_expressions();
    if library.is_empty() || rate <= 0.0 || duration <= 0.0 {
        return;
    }

    // LCG constants (Knuth).
    let mut state = seed;
    let lcg_next = |s: &mut u64| -> u64 {
        *s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *s
    };

    // Expected number of events ≈ duration * rate; simulate Poisson-ish with fixed spacing + jitter.
    let expected = (duration * rate).ceil() as usize;
    let avg_interval = duration / (expected as f32).max(1.0);

    let mut t = 0.0_f32;
    for _ in 0..expected {
        // Jitter the interval by ±50% of avg_interval.
        let rand_u = lcg_next(&mut state);
        let jitter = (rand_u % 1000) as f32 / 1000.0 - 0.5; // [-0.5, 0.5)
        t += avg_interval * (1.0 + jitter);
        if t >= duration {
            break;
        }

        // Pick a random expression from the library.
        let idx_u = lcg_next(&mut state);
        let idx = (idx_u % library.len() as u64) as usize;
        let src = &library[idx];

        let event = MicroExpressionEvent {
            expr: MicroExpression {
                name: src.name.clone(),
                morph_weights: src.morph_weights.clone(),
                duration: src.duration,
                intensity: src.intensity,
            },
            trigger_time: t,
            fade_in: 0.02,
            fade_out: 0.04,
        };
        layer.add_event(event);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn simple_event(trigger: f32, duration: f32, intensity: f32) -> MicroExpressionEvent {
        MicroExpressionEvent {
            expr: MicroExpression {
                name: "test".to_string(),
                morph_weights: [("brow".to_string(), 1.0)].into_iter().collect(),
                duration,
                intensity,
            },
            trigger_time: trigger,
            fade_in: 0.1,
            fade_out: 0.1,
        }
    }

    // 1. micro_expr_weight_at before event → 0
    #[test]
    fn test_weight_before_event() {
        let ev = simple_event(1.0, 0.1, 0.8);
        assert!((micro_expr_weight_at(&ev, 0.5) - 0.0).abs() < 1e-6);
    }

    // 2. micro_expr_weight_at at peak → intensity
    #[test]
    fn test_weight_at_peak() {
        let ev = simple_event(1.0, 0.1, 0.8);
        // Plateau is 1.0 .. 1.1; t=1.05 is peak.
        let w = micro_expr_weight_at(&ev, 1.05);
        assert!((w - 0.8).abs() < 1e-5);
    }

    // 3. micro_expr_weight_at during fade_in → partial
    #[test]
    fn test_weight_during_fade_in() {
        let ev = simple_event(1.0, 0.1, 1.0);
        // fade_in=0.1, start=1.0. At t=0.95, elapsed=0.05 → env=0.5 → weight=0.5
        let w = micro_expr_weight_at(&ev, 0.95);
        assert!((w - 0.5).abs() < 1e-5);
    }

    // 4. micro_expr_weight_at during fade_out → partial
    #[test]
    fn test_weight_during_fade_out() {
        let ev = simple_event(1.0, 0.1, 1.0);
        // end=1.1, fade_out=0.1. At t=1.15, elapsed=0.05 → env=0.5
        let w = micro_expr_weight_at(&ev, 1.15);
        assert!((w - 0.5).abs() < 1e-5);
    }

    // 5. micro_expr_weight_at after event → 0
    #[test]
    fn test_weight_after_event() {
        let ev = simple_event(1.0, 0.1, 0.8);
        // end=1.1, fade_out=0.1, so fully gone at t > 1.2
        let w = micro_expr_weight_at(&ev, 1.5);
        assert!((w - 0.0).abs() < 1e-6);
    }

    // 6. sample at peak returns merged weights
    #[test]
    fn test_sample_at_peak_merges() {
        let base: HashMap<String, f32> = [("brow".to_string(), 0.0)].into_iter().collect();
        let mut layer = MicroExpressionLayer::new(base);
        layer.add_event(simple_event(0.0, 0.5, 1.0));
        // At t=0.25 (plateau), brow should be 1.0 (base 0.0 + 1.0*1.0).
        let result = layer.sample(0.25);
        assert!((result["brow"] - 1.0).abs() < 1e-5);
    }

    // 7. merge_weights: additive blend
    #[test]
    fn test_merge_weights_additive() {
        let base: HashMap<String, f32> = [("cheek".to_string(), 0.3)].into_iter().collect();
        let overlay: HashMap<String, f32> = [("cheek".to_string(), 0.4)].into_iter().collect();
        let result = merge_weights(&base, &overlay, 1.0);
        assert!((result["cheek"] - 0.7).abs() < 1e-5);
    }

    // 8. merge_weights: clamp to 1.0
    #[test]
    fn test_merge_weights_clamp() {
        let base: HashMap<String, f32> = [("x".to_string(), 0.8)].into_iter().collect();
        let overlay: HashMap<String, f32> = [("x".to_string(), 0.9)].into_iter().collect();
        let result = merge_weights(&base, &overlay, 1.0);
        assert!((result["x"] - 1.0).abs() < 1e-6, "should clamp to 1.0");
    }

    // 9. standard_micro_expressions count >= 5
    #[test]
    fn test_standard_micro_expressions_count() {
        let lib = standard_micro_expressions();
        assert!(lib.len() >= 5);
    }

    // 10. inject_random_micros adds events to the layer
    #[test]
    fn test_inject_random_micros_adds_events() {
        let base: HashMap<String, f32> = HashMap::new();
        let mut layer = MicroExpressionLayer::new(base);
        inject_random_micros(&mut layer, 10.0, 2.0, 42);
        assert!(
            !layer.events.is_empty(),
            "should have injected at least one event"
        );
    }

    // 11. layer with no events returns base weights unchanged
    #[test]
    fn test_layer_no_events_returns_base() {
        let base: HashMap<String, f32> = [("nose".to_string(), 0.4), ("jaw".to_string(), 0.6)]
            .into_iter()
            .collect();
        let layer = MicroExpressionLayer::new(base.clone());
        let result = layer.sample(5.0);
        for (k, &v) in &base {
            assert!((result[k] - v).abs() < 1e-6);
        }
    }

    // 12. merge_weights: new key in overlay is added to result
    #[test]
    fn test_merge_weights_new_key_added() {
        let base: HashMap<String, f32> = [("a".to_string(), 0.5)].into_iter().collect();
        let overlay: HashMap<String, f32> = [("b".to_string(), 0.6)].into_iter().collect();
        let result = merge_weights(&base, &overlay, 0.5);
        assert!((result["a"] - 0.5).abs() < 1e-6);
        assert!((result["b"] - 0.3).abs() < 1e-5);
    }
}
