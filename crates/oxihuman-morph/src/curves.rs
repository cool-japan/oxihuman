// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Weight curves that map a raw parameter value `t ∈ [0,1]` to a morph
//! blend weight `∈ [0,1]` using various easing functions.

// ── helpers ───────────────────────────────────────────────────────────────────

fn bezier_x(p1x: f32, p2x: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * t * p1x + 3.0 * mt * t * t * p2x + t * t * t
}

fn bezier_y(p1y: f32, p2y: f32, t: f32) -> f32 {
    let mt = 1.0 - t;
    3.0 * mt * mt * t * p1y + 3.0 * mt * t * t * p2y + t * t * t
}

/// Binary-search for the Bezier parameter `u` such that `bezier_x(p1x, p2x, u) ≈ x`.
fn bezier_evaluate(p1: [f32; 2], p2: [f32; 2], x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let mut lo = 0.0_f32;
    let mut hi = 1.0_f32;
    for _ in 0..10 {
        let mid = (lo + hi) * 0.5;
        let bx = bezier_x(p1[0], p2[0], mid);
        if bx < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let u = (lo + hi) * 0.5;
    bezier_y(p1[1], p2[1], u)
}

// ── trait ─────────────────────────────────────────────────────────────────────

/// A curve maps an input `t ∈ [0,1]` to an output weight `∈ [0,1]`.
pub trait WeightCurve: Send + Sync {
    fn evaluate(&self, t: f32) -> f32;
    fn name(&self) -> &str;
}

// ── concrete structs ──────────────────────────────────────────────────────────

/// Linear: output = t
#[allow(dead_code)]
pub struct LinearCurve;

/// Ease-in (quadratic): output = t²
#[allow(dead_code)]
pub struct EaseInCurve;

/// Ease-out (quadratic): output = 1 − (1−t)²
#[allow(dead_code)]
pub struct EaseOutCurve;

/// Ease-in-out (cubic Hermite S-curve): output = t² × (3 − 2t)
#[allow(dead_code)]
pub struct SmoothStepCurve;

/// Ease-in-out (quintic): output = t³ × (t × (6t − 15) + 10)
#[allow(dead_code)]
pub struct SmootherStepCurve;

/// Power curve: output = t^exponent
#[allow(dead_code)]
pub struct PowerCurve {
    pub exponent: f32,
}

/// Stepped curve: output jumps at evenly-spaced thresholds.
#[allow(dead_code)]
pub struct SteppedCurve {
    pub steps: usize,
}

/// Cubic Bezier curve with P0=(0,0) and P3=(1,1) fixed.
#[allow(dead_code)]
pub struct BezierCurve {
    pub p1: [f32; 2],
    pub p2: [f32; 2],
}

/// Clamped identity: remaps [min, max] to `[0,1]` and clamps outside.
#[allow(dead_code)]
pub struct ClampedCurve {
    pub min: f32,
    pub max: f32,
}

// ── WeightCurve impls ─────────────────────────────────────────────────────────

impl WeightCurve for LinearCurve {
    fn evaluate(&self, t: f32) -> f32 {
        t.clamp(0.0, 1.0)
    }
    fn name(&self) -> &str {
        "Linear"
    }
}

impl WeightCurve for EaseInCurve {
    fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t
    }
    fn name(&self) -> &str {
        "EaseIn"
    }
}

impl WeightCurve for EaseOutCurve {
    fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        1.0 - u * u
    }
    fn name(&self) -> &str {
        "EaseOut"
    }
}

impl WeightCurve for SmoothStepCurve {
    fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    fn name(&self) -> &str {
        "SmoothStep"
    }
}

impl WeightCurve for SmootherStepCurve {
    fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        t * t * t * (t * (6.0 * t - 15.0) + 10.0)
    }
    fn name(&self) -> &str {
        "SmootherStep"
    }
}

impl WeightCurve for PowerCurve {
    fn evaluate(&self, t: f32) -> f32 {
        t.clamp(0.0, 1.0).powf(self.exponent)
    }
    fn name(&self) -> &str {
        "Power"
    }
}

impl WeightCurve for SteppedCurve {
    fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        if self.steps <= 1 {
            return t;
        }
        let step = (t * self.steps as f32).floor() / (self.steps - 1) as f32;
        step.clamp(0.0, 1.0)
    }
    fn name(&self) -> &str {
        "Stepped"
    }
}

impl WeightCurve for BezierCurve {
    fn evaluate(&self, t: f32) -> f32 {
        bezier_evaluate(self.p1, self.p2, t)
    }
    fn name(&self) -> &str {
        "Bezier"
    }
}

impl WeightCurve for ClampedCurve {
    fn evaluate(&self, t: f32) -> f32 {
        let range = self.max - self.min;
        if range.abs() < f32::EPSILON {
            return 0.0;
        }
        ((t - self.min) / range).clamp(0.0, 1.0)
    }
    fn name(&self) -> &str {
        "Clamped"
    }
}

// ── serialisable enum ─────────────────────────────────────────────────────────

/// A serialisable enum that wraps all curve types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CurveKind {
    Linear,
    EaseIn,
    EaseOut,
    SmoothStep,
    SmootherStep,
    Power { exponent: f32 },
    Stepped { steps: usize },
    Bezier { p1: [f32; 2], p2: [f32; 2] },
    Clamped { min: f32, max: f32 },
}

impl CurveKind {
    pub fn evaluate(&self, t: f32) -> f32 {
        match self {
            CurveKind::Linear => LinearCurve.evaluate(t),
            CurveKind::EaseIn => EaseInCurve.evaluate(t),
            CurveKind::EaseOut => EaseOutCurve.evaluate(t),
            CurveKind::SmoothStep => SmoothStepCurve.evaluate(t),
            CurveKind::SmootherStep => SmootherStepCurve.evaluate(t),
            CurveKind::Power { exponent } => PowerCurve {
                exponent: *exponent,
            }
            .evaluate(t),
            CurveKind::Stepped { steps } => SteppedCurve { steps: *steps }.evaluate(t),
            CurveKind::Bezier { p1, p2 } => BezierCurve { p1: *p1, p2: *p2 }.evaluate(t),
            CurveKind::Clamped { min, max } => ClampedCurve {
                min: *min,
                max: *max,
            }
            .evaluate(t),
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            CurveKind::Linear => "Linear",
            CurveKind::EaseIn => "EaseIn",
            CurveKind::EaseOut => "EaseOut",
            CurveKind::SmoothStep => "SmoothStep",
            CurveKind::SmootherStep => "SmootherStep",
            CurveKind::Power { .. } => "Power",
            CurveKind::Stepped { .. } => "Stepped",
            CurveKind::Bezier { .. } => "Bezier",
            CurveKind::Clamped { .. } => "Clamped",
        }
    }

    /// Preset for "age" parameter — slow start, fast end (ease-in).
    pub fn age() -> Self {
        CurveKind::EaseIn
    }

    /// Preset for a general weight parameter — smooth mid-range.
    pub fn weight_param() -> Self {
        CurveKind::SmoothStep
    }

    /// Preset for muscle definition — sub-linear response.
    pub fn muscle() -> Self {
        CurveKind::Power { exponent: 0.7 }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < EPSILON
    }

    #[test]
    fn linear_at_half() {
        assert!(approx_eq(LinearCurve.evaluate(0.5), 0.5));
    }

    #[test]
    fn ease_in_at_half_less_than_half() {
        assert!(EaseInCurve.evaluate(0.5) < 0.5);
    }

    #[test]
    fn ease_out_at_half_greater_than_half() {
        assert!(EaseOutCurve.evaluate(0.5) > 0.5);
    }

    #[test]
    fn smooth_step_at_half() {
        assert!(approx_eq(SmoothStepCurve.evaluate(0.5), 0.5));
    }

    #[test]
    fn all_curves_zero_at_zero() {
        let curves = [
            CurveKind::Linear,
            CurveKind::EaseIn,
            CurveKind::EaseOut,
            CurveKind::SmoothStep,
            CurveKind::SmootherStep,
            CurveKind::Power { exponent: 2.0 },
            CurveKind::Stepped { steps: 4 },
            CurveKind::Bezier {
                p1: [0.25, 0.1],
                p2: [0.75, 0.9],
            },
            CurveKind::Clamped { min: 0.0, max: 1.0 },
        ];
        for curve in &curves {
            assert!(
                approx_eq(curve.evaluate(0.0), 0.0),
                "{} did not return 0.0 at t=0",
                curve.name()
            );
        }
    }

    #[test]
    fn all_curves_one_at_one() {
        let curves = [
            CurveKind::Linear,
            CurveKind::EaseIn,
            CurveKind::EaseOut,
            CurveKind::SmoothStep,
            CurveKind::SmootherStep,
            CurveKind::Power { exponent: 2.0 },
            CurveKind::Stepped { steps: 4 },
            CurveKind::Bezier {
                p1: [0.25, 0.1],
                p2: [0.75, 0.9],
            },
            CurveKind::Clamped { min: 0.0, max: 1.0 },
        ];
        for curve in &curves {
            assert!(
                approx_eq(curve.evaluate(1.0), 1.0),
                "{} did not return 1.0 at t=1",
                curve.name()
            );
        }
    }

    #[test]
    fn power_curve_exponent_2() {
        let result = PowerCurve { exponent: 2.0 }.evaluate(0.5);
        assert!(approx_eq(result, 0.25), "expected 0.25, got {result}");
    }

    #[test]
    fn stepped_curve_midpoint() {
        let result = SteppedCurve { steps: 4 }.evaluate(0.5);
        let valid = [0.0_f32, 1.0 / 3.0, 2.0 / 3.0, 1.0];
        assert!(
            valid.iter().any(|&v| (result - v).abs() < 0.02),
            "stepped midpoint {result} not in expected set"
        );
    }

    #[test]
    fn bezier_endpoints() {
        let curve = BezierCurve {
            p1: [0.42, 0.0],
            p2: [0.58, 1.0],
        };
        assert!(approx_eq(curve.evaluate(0.0), 0.0));
        assert!(approx_eq(curve.evaluate(1.0), 1.0));
    }

    #[test]
    fn curve_kind_serialize() {
        let original = CurveKind::Power { exponent: 2.0 };
        let json = serde_json::to_string(&original).expect("serialise");
        let decoded: CurveKind = serde_json::from_str(&json).expect("deserialise");
        if let CurveKind::Power { exponent } = decoded {
            assert!(approx_eq(exponent, 2.0));
        } else {
            panic!("deserialised to wrong variant");
        }
    }

    #[test]
    fn age_preset_is_ease_in() {
        assert!(matches!(CurveKind::age(), CurveKind::EaseIn));
    }
}
