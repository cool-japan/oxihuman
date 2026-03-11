#![allow(dead_code)]
//! Expression keyframe: stores expression weights at specific times for interpolation.

use std::collections::HashMap;

/// Interpolation type for keyframes.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeInterp {
    Linear,
    Step,
    Smooth,
}

/// A keyframe with time, weights, and interpolation type.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionKeyframe {
    time: f32,
    weights: HashMap<String, f32>,
    interp: KeyframeInterp,
}

/// Create a new keyframe at the given time.
#[allow(dead_code)]
pub fn new_expression_keyframe(time: f32) -> ExpressionKeyframe {
    ExpressionKeyframe {
        time,
        weights: HashMap::new(),
        interp: KeyframeInterp::Linear,
    }
}

/// Return the time of the keyframe.
#[allow(dead_code)]
pub fn keyframe_time(kf: &ExpressionKeyframe) -> f32 {
    kf.time
}

/// Return a reference to the weights.
#[allow(dead_code)]
pub fn keyframe_weights(kf: &ExpressionKeyframe) -> &HashMap<String, f32> {
    &kf.weights
}

/// Return the interpolation type.
#[allow(dead_code)]
pub fn keyframe_interpolation_ek(kf: &ExpressionKeyframe) -> &KeyframeInterp {
    &kf.interp
}

/// Return the number of weight entries.
#[allow(dead_code)]
pub fn keyframe_count_ek(kf: &ExpressionKeyframe) -> usize {
    kf.weights.len()
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn keyframe_to_json(kf: &ExpressionKeyframe) -> String {
    let interp_str = match kf.interp {
        KeyframeInterp::Linear => "linear",
        KeyframeInterp::Step => "step",
        KeyframeInterp::Smooth => "smooth",
    };
    format!(
        "{{\"time\":{},\"interp\":\"{}\",\"weight_count\":{}}}",
        kf.time,
        interp_str,
        kf.weights.len()
    )
}

/// Interpolate between two keyframes at `t` (0..1).
#[allow(dead_code)]
pub fn interpolate_keyframes(
    a: &ExpressionKeyframe,
    b: &ExpressionKeyframe,
    t: f32,
) -> HashMap<String, f32> {
    let t_clamped = t.clamp(0.0, 1.0);
    let mut result = HashMap::new();
    let all_keys: std::collections::HashSet<&String> =
        a.weights.keys().chain(b.weights.keys()).collect();
    for key in all_keys {
        let va = a.weights.get(key).copied().unwrap_or(0.0);
        let vb = b.weights.get(key).copied().unwrap_or(0.0);
        result.insert(key.clone(), va + (vb - va) * t_clamped);
    }
    result
}

/// Return the total duration spanned by a slice of keyframes.
#[allow(dead_code)]
pub fn keyframes_duration(keyframes: &[ExpressionKeyframe]) -> f32 {
    if keyframes.len() < 2 {
        return 0.0;
    }
    let first = keyframes.iter().map(|k| k.time).fold(f32::INFINITY, f32::min);
    let last = keyframes.iter().map(|k| k.time).fold(f32::NEG_INFINITY, f32::max);
    last - first
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_keyframe() {
        let kf = new_expression_keyframe(1.0);
        assert!((keyframe_time(&kf) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_keyframe_weights() {
        let mut kf = new_expression_keyframe(0.0);
        kf.weights.insert("smile".to_string(), 0.5);
        assert_eq!(keyframe_count_ek(&kf), 1);
    }

    #[test]
    fn test_interpolation_type() {
        let kf = new_expression_keyframe(0.0);
        assert_eq!(*keyframe_interpolation_ek(&kf), KeyframeInterp::Linear);
    }

    #[test]
    fn test_to_json() {
        let kf = new_expression_keyframe(1.5);
        let json = keyframe_to_json(&kf);
        assert!(json.contains("\"time\":1.5"));
    }

    #[test]
    fn test_interpolate() {
        let mut a = new_expression_keyframe(0.0);
        a.weights.insert("x".to_string(), 0.0);
        let mut b = new_expression_keyframe(1.0);
        b.weights.insert("x".to_string(), 1.0);
        let result = interpolate_keyframes(&a, &b, 0.5);
        assert!((result["x"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_interpolate_clamp() {
        let a = new_expression_keyframe(0.0);
        let mut b = new_expression_keyframe(1.0);
        b.weights.insert("x".to_string(), 1.0);
        let result = interpolate_keyframes(&a, &b, 2.0);
        assert!((result["x"] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_keyframes_duration() {
        let kfs = vec![
            new_expression_keyframe(0.0),
            new_expression_keyframe(2.0),
            new_expression_keyframe(5.0),
        ];
        assert!((keyframes_duration(&kfs) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn test_keyframes_duration_single() {
        let kfs = vec![new_expression_keyframe(1.0)];
        assert!((keyframes_duration(&kfs) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_keyframes_duration_empty() {
        let kfs: Vec<ExpressionKeyframe> = vec![];
        assert!((keyframes_duration(&kfs) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_weights_ref() {
        let kf = new_expression_keyframe(0.0);
        assert!(keyframe_weights(&kf).is_empty());
    }
}
