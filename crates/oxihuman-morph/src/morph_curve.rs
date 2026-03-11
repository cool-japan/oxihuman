//! Morph curve — maps a linear input [0,1] through a curve for non-linear morph response.

// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A single control point on the curve.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct CurvePoint {
    /// Normalized input parameter in [0, 1].
    pub t: f32,
    /// Output value at this point.
    pub value: f32,
}

/// Configuration for a morph curve.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphCurveConfig {
    /// Whether to clamp the input t to [0, 1] before evaluating.
    pub clamp_input: bool,
}

/// A piecewise-linear morph curve defined by a sorted list of control points.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphCurve {
    pub config: MorphCurveConfig,
    pub points: Vec<CurvePoint>,
}

/// Returns a default `MorphCurveConfig`.
#[allow(dead_code)]
pub fn default_morph_curve_config() -> MorphCurveConfig {
    MorphCurveConfig { clamp_input: true }
}

/// Creates a new `MorphCurve` initialized to the identity (linear) mapping.
#[allow(dead_code)]
pub fn new_morph_curve(cfg: &MorphCurveConfig) -> MorphCurve {
    let mut curve = MorphCurve {
        config: cfg.clone(),
        points: Vec::new(),
    };
    morph_curve_linear(&mut curve);
    curve
}

/// Adds a control point at `(t, value)` and keeps the list sorted by `t`.
#[allow(dead_code)]
pub fn morph_curve_add_point(curve: &mut MorphCurve, t: f32, value: f32) {
    curve.points.retain(|p| (p.t - t).abs() > 1e-7);
    curve.points.push(CurvePoint { t, value });
    curve
        .points
        .sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap_or(std::cmp::Ordering::Equal));
}

/// Evaluates the curve at parameter `t` using piecewise-linear interpolation.
/// Returns 0.0 if no points are defined.
#[allow(dead_code)]
pub fn morph_curve_evaluate(curve: &MorphCurve, t: f32) -> f32 {
    if curve.points.is_empty() {
        return 0.0;
    }
    let t = if curve.config.clamp_input {
        t.clamp(0.0, 1.0)
    } else {
        t
    };
    // Before first point
    if t <= curve.points[0].t {
        return curve.points[0].value;
    }
    // After last point
    let last = curve.points.len() - 1;
    if t >= curve.points[last].t {
        return curve.points[last].value;
    }
    // Piecewise linear search
    for i in 0..last {
        let p0 = &curve.points[i];
        let p1 = &curve.points[i + 1];
        if t >= p0.t && t <= p1.t {
            let span = p1.t - p0.t;
            if span < 1e-12 {
                return p0.value;
            }
            let frac = (t - p0.t) / span;
            return p0.value + frac * (p1.value - p0.value);
        }
    }
    curve.points[last].value
}

/// Returns the number of control points.
#[allow(dead_code)]
pub fn morph_curve_point_count(curve: &MorphCurve) -> usize {
    curve.points.len()
}

/// Removes all control points.
#[allow(dead_code)]
pub fn morph_curve_clear(curve: &mut MorphCurve) {
    curve.points.clear();
}

/// Resets the curve to the identity (linear) mapping: (0,0) → (1,1).
#[allow(dead_code)]
pub fn morph_curve_linear(curve: &mut MorphCurve) {
    curve.points.clear();
    curve.points.push(CurvePoint { t: 0.0, value: 0.0 });
    curve.points.push(CurvePoint { t: 1.0, value: 1.0 });
}

/// Sets the curve to an ease-in-out shape with 5 control points.
#[allow(dead_code)]
pub fn morph_curve_ease_in_out(curve: &mut MorphCurve) {
    curve.points.clear();
    // Smooth-step approximation via control points
    let pts = [
        (0.0_f32, 0.0_f32),
        (0.25, 0.062_5),
        (0.5, 0.5),
        (0.75, 0.937_5),
        (1.0, 1.0),
    ];
    for (t, v) in pts {
        curve.points.push(CurvePoint { t, value: v });
    }
}

/// Inverts the curve so that value ↦ 1 − value at each control point.
#[allow(dead_code)]
pub fn morph_curve_invert(curve: &mut MorphCurve) {
    for p in &mut curve.points {
        p.value = 1.0 - p.value;
    }
}

/// Evaluates the curve and clamps the output to [0, 1].
#[allow(dead_code)]
pub fn morph_curve_clamp_output(curve: &MorphCurve, t: f32) -> f32 {
    morph_curve_evaluate(curve, t).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = default_morph_curve_config();
        assert!(cfg.clamp_input);
    }

    #[test]
    fn test_new_curve_is_linear() {
        let cfg = default_morph_curve_config();
        let curve = new_morph_curve(&cfg);
        assert_eq!(morph_curve_point_count(&curve), 2);
        assert!((morph_curve_evaluate(&curve, 0.0) - 0.0).abs() < 1e-6);
        assert!((morph_curve_evaluate(&curve, 0.5) - 0.5).abs() < 1e-6);
        assert!((morph_curve_evaluate(&curve, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_point_sorted() {
        let cfg = default_morph_curve_config();
        let mut curve = new_morph_curve(&cfg);
        morph_curve_add_point(&mut curve, 0.3, 0.1);
        // Points should be sorted by t
        for i in 1..curve.points.len() {
            assert!(curve.points[i].t >= curve.points[i - 1].t);
        }
    }

    #[test]
    fn test_evaluate_midpoint() {
        let cfg = default_morph_curve_config();
        let mut curve = new_morph_curve(&cfg);
        morph_curve_add_point(&mut curve, 0.5, 0.8);
        let v = morph_curve_evaluate(&curve, 0.25);
        // Between (0.0,0.0) and (0.5,0.8): at t=0.25 → frac=0.5 → value=0.4
        assert!((v - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_clear() {
        let cfg = default_morph_curve_config();
        let mut curve = new_morph_curve(&cfg);
        morph_curve_clear(&mut curve);
        assert_eq!(morph_curve_point_count(&curve), 0);
        assert!((morph_curve_evaluate(&curve, 0.5) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_ease_in_out() {
        let cfg = default_morph_curve_config();
        let mut curve = new_morph_curve(&cfg);
        morph_curve_ease_in_out(&mut curve);
        assert_eq!(morph_curve_point_count(&curve), 5);
        // Symmetry: f(0.5) ≈ 0.5
        assert!((morph_curve_evaluate(&curve, 0.5) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_invert() {
        let cfg = default_morph_curve_config();
        let mut curve = new_morph_curve(&cfg);
        morph_curve_invert(&mut curve);
        // (0,0)→(0,1) and (1,1)→(1,0)
        assert!((morph_curve_evaluate(&curve, 0.0) - 1.0).abs() < 1e-6);
        assert!((morph_curve_evaluate(&curve, 1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp_output() {
        let cfg = default_morph_curve_config();
        let mut curve = new_morph_curve(&cfg);
        // Add a point that pushes output above 1 when extrapolated (clamp_input=false)
        morph_curve_clear(&mut curve);
        curve.points.push(CurvePoint { t: 0.0, value: -0.5 });
        curve.points.push(CurvePoint { t: 1.0, value: 1.5 });
        let clamped = morph_curve_clamp_output(&curve, 0.5);
        assert!((0.0..=1.0).contains(&clamped));
    }

    #[test]
    fn test_linear_reset() {
        let cfg = default_morph_curve_config();
        let mut curve = new_morph_curve(&cfg);
        morph_curve_ease_in_out(&mut curve);
        morph_curve_linear(&mut curve);
        assert_eq!(morph_curve_point_count(&curve), 2);
        assert!((morph_curve_evaluate(&curve, 0.75) - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_clamp_input() {
        let cfg = default_morph_curve_config(); // clamp_input = true
        let curve = new_morph_curve(&cfg);
        // t > 1 should clamp to 1.0
        assert!((morph_curve_evaluate(&curve, 2.0) - 1.0).abs() < 1e-6);
        // t < 0 should clamp to 0.0
        assert!((morph_curve_evaluate(&curve, -1.0) - 0.0).abs() < 1e-6);
    }
}
