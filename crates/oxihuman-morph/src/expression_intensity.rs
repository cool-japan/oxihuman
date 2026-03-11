#![allow(dead_code)]
//! Expression intensity curves and time-based evaluation.

/// Intensity curve shape.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntensityCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// Expression intensity state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionIntensity {
    /// Current intensity in [0, 1].
    pub intensity: f32,
    /// Peak intensity value.
    pub peak: f32,
    /// Ramp-up duration in seconds.
    pub ramp_up: f32,
    /// Ramp-down duration in seconds.
    pub ramp_down: f32,
    /// Curve type.
    pub curve: IntensityCurve,
    /// Human-readable label.
    pub name: String,
}

/// Create a new [`ExpressionIntensity`].
#[allow(dead_code)]
pub fn new_expression_intensity(name: &str, peak: f32) -> ExpressionIntensity {
    ExpressionIntensity {
        intensity: 0.0,
        peak: peak.clamp(0.0, 1.0),
        ramp_up: 0.3,
        ramp_down: 0.5,
        curve: IntensityCurve::Linear,
        name: name.to_string(),
    }
}

/// Set the current intensity, clamped to [0, 1].
#[allow(dead_code)]
pub fn set_intensity(ei: &mut ExpressionIntensity, value: f32) {
    ei.intensity = value.clamp(0.0, 1.0);
}

/// Get the current intensity.
#[allow(dead_code)]
pub fn get_intensity(ei: &ExpressionIntensity) -> f32 {
    ei.intensity
}

/// Compute ramp-up value at time t (0 = start, ramp_up = full peak).
#[allow(dead_code)]
pub fn intensity_ramp_up(ei: &ExpressionIntensity, t: f32) -> f32 {
    if ei.ramp_up <= 0.0 {
        return ei.peak;
    }
    let frac = (t / ei.ramp_up).clamp(0.0, 1.0);
    apply_curve(frac, ei.curve) * ei.peak
}

/// Compute ramp-down value at time t (0 = start of decay, ramp_down = zero).
#[allow(dead_code)]
pub fn intensity_ramp_down(ei: &ExpressionIntensity, t: f32) -> f32 {
    if ei.ramp_down <= 0.0 {
        return 0.0;
    }
    let frac = (t / ei.ramp_down).clamp(0.0, 1.0);
    (1.0 - apply_curve(frac, ei.curve)) * ei.peak
}

/// Evaluate intensity at an arbitrary time, assuming ramp_up then hold then ramp_down.
/// hold_duration is the time at peak between ramps.
#[allow(dead_code)]
pub fn intensity_at_time(ei: &ExpressionIntensity, t: f32, hold_duration: f32) -> f32 {
    if t < 0.0 {
        return 0.0;
    }
    if t < ei.ramp_up {
        return intensity_ramp_up(ei, t);
    }
    let after_ramp_up = t - ei.ramp_up;
    if after_ramp_up < hold_duration {
        return ei.peak;
    }
    let decay_t = after_ramp_up - hold_duration;
    intensity_ramp_down(ei, decay_t)
}

/// Return the peak intensity.
#[allow(dead_code)]
pub fn peak_intensity(ei: &ExpressionIntensity) -> f32 {
    ei.peak
}

/// Convert intensity to a morph weight (identity mapping, clamped).
#[allow(dead_code)]
pub fn intensity_to_weight(ei: &ExpressionIntensity) -> f32 {
    ei.intensity.clamp(0.0, 1.0)
}

fn apply_curve(t: f32, curve: IntensityCurve) -> f32 {
    match curve {
        IntensityCurve::Linear => t,
        IntensityCurve::EaseIn => t * t,
        IntensityCurve::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        IntensityCurve::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - 2.0 * (1.0 - t) * (1.0 - t)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_expression_intensity() {
        let ei = new_expression_intensity("smile", 0.8);
        assert_eq!(ei.name, "smile");
        assert!((ei.peak - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_get_intensity() {
        let mut ei = new_expression_intensity("frown", 1.0);
        set_intensity(&mut ei, 0.5);
        assert!((get_intensity(&ei) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_intensity_clamps() {
        let mut ei = new_expression_intensity("x", 1.0);
        set_intensity(&mut ei, 2.0);
        assert!((get_intensity(&ei) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ramp_up_at_zero() {
        let ei = new_expression_intensity("x", 1.0);
        assert!(intensity_ramp_up(&ei, 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ramp_up_at_full() {
        let ei = new_expression_intensity("x", 1.0);
        assert!((intensity_ramp_up(&ei, ei.ramp_up) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ramp_down_at_full() {
        let ei = new_expression_intensity("x", 1.0);
        assert!((intensity_ramp_down(&ei, ei.ramp_down) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_intensity_at_time_before_zero() {
        let ei = new_expression_intensity("x", 1.0);
        assert!(intensity_at_time(&ei, -1.0, 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_intensity_at_time_hold() {
        let ei = new_expression_intensity("x", 1.0);
        let val = intensity_at_time(&ei, ei.ramp_up + 0.1, 1.0);
        assert!((val - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_peak_intensity() {
        let ei = new_expression_intensity("x", 0.7);
        assert!((peak_intensity(&ei) - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_intensity_to_weight() {
        let mut ei = new_expression_intensity("x", 1.0);
        set_intensity(&mut ei, 0.6);
        assert!((intensity_to_weight(&ei) - 0.6).abs() < f32::EPSILON);
    }
}
