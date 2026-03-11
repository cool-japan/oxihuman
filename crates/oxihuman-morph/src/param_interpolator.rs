//! Interpolate named float parameters over time using various curve modes.
//!
//! Supports linear, step, and smoothstep interpolation between keyframes.

#![allow(dead_code)]

/// Interpolation curve mode between keyframes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InterpolationMode {
    /// Linear interpolation between keys.
    Linear,
    /// Hold previous key value until next key.
    Step,
    /// Smooth cubic S-curve interpolation.
    Smoothstep,
}

/// Configuration for a `ParamInterpolator`.
#[derive(Debug, Clone)]
pub struct ParamInterpolatorConfig {
    /// Default interpolation mode applied to new keys.
    pub default_mode: InterpolationMode,
    /// Maximum number of keys allowed per parameter.
    pub max_keys: usize,
}

/// A single keyframe entry for a named parameter.
#[derive(Debug, Clone)]
pub struct ParamKey {
    /// Time in seconds for this key.
    pub time: f32,
    /// Value of the parameter at this key.
    pub value: f32,
    /// Interpolation mode from this key to the next.
    pub mode: InterpolationMode,
}

/// Interpolates named float parameters over time.
#[derive(Debug, Clone)]
pub struct ParamInterpolator {
    config: ParamInterpolatorConfig,
    name: String,
    keys: Vec<ParamKey>,
    current_mode: InterpolationMode,
}

/// Build a default `ParamInterpolatorConfig`.
#[allow(dead_code)]
pub fn default_param_interpolator_config() -> ParamInterpolatorConfig {
    ParamInterpolatorConfig {
        default_mode: InterpolationMode::Linear,
        max_keys: 256,
    }
}

/// Create a new `ParamInterpolator` with the given name.
#[allow(dead_code)]
pub fn new_param_interpolator(name: &str, config: ParamInterpolatorConfig) -> ParamInterpolator {
    let mode = config.default_mode;
    ParamInterpolator {
        config,
        name: name.to_string(),
        keys: Vec::new(),
        current_mode: mode,
    }
}

/// Add a keyframe to the interpolator (inserted in sorted time order).
#[allow(dead_code)]
pub fn param_interp_add_key(interp: &mut ParamInterpolator, time: f32, value: f32) {
    if interp.keys.len() >= interp.config.max_keys {
        return;
    }
    let mode = interp.current_mode;
    let key = ParamKey { time, value, mode };
    let pos = interp.keys.partition_point(|k| k.time <= time);
    interp.keys.insert(pos, key);
}

/// Evaluate the interpolated value at the given time.
#[allow(dead_code)]
pub fn param_interp_evaluate(interp: &ParamInterpolator, time: f32) -> f32 {
    if interp.keys.is_empty() {
        return 0.0;
    }
    let first = &interp.keys[0];
    let last = &interp.keys[interp.keys.len() - 1];
    if time <= first.time {
        return first.value;
    }
    if time >= last.time {
        return last.value;
    }
    // Find surrounding keys
    let next_idx = interp.keys.partition_point(|k| k.time <= time);
    let prev_idx = next_idx - 1;
    let k0 = &interp.keys[prev_idx];
    let k1 = &interp.keys[next_idx];
    let span = k1.time - k0.time;
    if span <= f32::EPSILON {
        return k1.value;
    }
    let t = (time - k0.time) / span;
    match k0.mode {
        InterpolationMode::Linear => k0.value + t * (k1.value - k0.value),
        InterpolationMode::Step => k0.value,
        InterpolationMode::Smoothstep => {
            let s = t * t * (3.0 - 2.0 * t);
            k0.value + s * (k1.value - k0.value)
        }
    }
}

/// Return the number of keyframes.
#[allow(dead_code)]
pub fn param_interp_key_count(interp: &ParamInterpolator) -> usize {
    interp.keys.len()
}

/// Return the total duration (last key time - first key time), or 0.
#[allow(dead_code)]
pub fn param_interp_duration(interp: &ParamInterpolator) -> f32 {
    if interp.keys.len() < 2 {
        return 0.0;
    }
    interp.keys[interp.keys.len() - 1].time - interp.keys[0].time
}

/// Set the interpolation mode used for subsequently added keys.
#[allow(dead_code)]
pub fn param_interp_set_mode(interp: &mut ParamInterpolator, mode: InterpolationMode) {
    interp.current_mode = mode;
}

/// Serialize the interpolator state to a JSON string.
#[allow(dead_code)]
pub fn param_interp_to_json(interp: &ParamInterpolator) -> String {
    let keys_json: Vec<String> = interp
        .keys
        .iter()
        .map(|k| {
            let mode_str = match k.mode {
                InterpolationMode::Linear => "linear",
                InterpolationMode::Step => "step",
                InterpolationMode::Smoothstep => "smoothstep",
            };
            format!(
                "{{\"time\":{},\"value\":{},\"mode\":\"{}\"}}",
                k.time, k.value, mode_str
            )
        })
        .collect();
    format!(
        "{{\"name\":\"{}\",\"key_count\":{},\"duration\":{},\"keys\":[{}]}}",
        interp.name,
        interp.keys.len(),
        param_interp_duration(interp),
        keys_json.join(",")
    )
}

/// Remove all keyframes.
#[allow(dead_code)]
pub fn param_interp_clear(interp: &mut ParamInterpolator) {
    interp.keys.clear();
}

/// Reset to initial state (clear keys, restore default mode).
#[allow(dead_code)]
pub fn param_interp_reset(interp: &mut ParamInterpolator) {
    interp.keys.clear();
    interp.current_mode = interp.config.default_mode;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_interp() -> ParamInterpolator {
        let cfg = default_param_interpolator_config();
        new_param_interpolator("test", cfg)
    }

    #[test]
    fn test_empty_evaluates_zero() {
        let interp = make_interp();
        assert_eq!(param_interp_evaluate(&interp, 0.5), 0.0);
    }

    #[test]
    fn test_single_key() {
        let mut interp = make_interp();
        param_interp_add_key(&mut interp, 1.0, 42.0);
        assert_eq!(param_interp_evaluate(&interp, 0.0), 42.0);
        assert_eq!(param_interp_evaluate(&interp, 1.0), 42.0);
        assert_eq!(param_interp_evaluate(&interp, 2.0), 42.0);
    }

    #[test]
    fn test_linear_midpoint() {
        let mut interp = make_interp();
        param_interp_add_key(&mut interp, 0.0, 0.0);
        param_interp_add_key(&mut interp, 1.0, 10.0);
        let v = param_interp_evaluate(&interp, 0.5);
        assert!((v - 5.0).abs() < 1e-5, "Expected 5.0, got {}", v);
    }

    #[test]
    fn test_step_mode() {
        let mut interp = make_interp();
        param_interp_set_mode(&mut interp, InterpolationMode::Step);
        param_interp_add_key(&mut interp, 0.0, 1.0);
        param_interp_add_key(&mut interp, 1.0, 2.0);
        let v = param_interp_evaluate(&interp, 0.5);
        assert!((v - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_smoothstep_midpoint() {
        let mut interp = make_interp();
        param_interp_set_mode(&mut interp, InterpolationMode::Smoothstep);
        param_interp_add_key(&mut interp, 0.0, 0.0);
        param_interp_add_key(&mut interp, 1.0, 1.0);
        let v = param_interp_evaluate(&interp, 0.5);
        // smoothstep(0.5) = 0.5
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_duration() {
        let mut interp = make_interp();
        assert_eq!(param_interp_duration(&interp), 0.0);
        param_interp_add_key(&mut interp, 0.0, 0.0);
        param_interp_add_key(&mut interp, 3.0, 1.0);
        assert!((param_interp_duration(&interp) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_key_count_and_clear() {
        let mut interp = make_interp();
        param_interp_add_key(&mut interp, 0.0, 0.0);
        param_interp_add_key(&mut interp, 1.0, 1.0);
        assert_eq!(param_interp_key_count(&interp), 2);
        param_interp_clear(&mut interp);
        assert_eq!(param_interp_key_count(&interp), 0);
    }

    #[test]
    fn test_reset_restores_mode() {
        let mut interp = make_interp();
        param_interp_set_mode(&mut interp, InterpolationMode::Step);
        param_interp_add_key(&mut interp, 0.0, 5.0);
        param_interp_reset(&mut interp);
        assert_eq!(interp.current_mode, InterpolationMode::Linear);
        assert_eq!(param_interp_key_count(&interp), 0);
    }

    #[test]
    fn test_to_json_contains_name() {
        let interp = make_interp();
        let json = param_interp_to_json(&interp);
        assert!(json.contains("\"name\":\"test\""));
    }

    #[test]
    fn test_keys_sorted_on_insert() {
        let mut interp = make_interp();
        param_interp_add_key(&mut interp, 2.0, 2.0);
        param_interp_add_key(&mut interp, 0.0, 0.0);
        param_interp_add_key(&mut interp, 1.0, 1.0);
        assert_eq!(param_interp_key_count(&interp), 3);
        let v = param_interp_evaluate(&interp, 0.5);
        assert!((v - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_max_keys_enforced() {
        let cfg = ParamInterpolatorConfig {
            default_mode: InterpolationMode::Linear,
            max_keys: 2,
        };
        let mut interp = new_param_interpolator("limited", cfg);
        param_interp_add_key(&mut interp, 0.0, 0.0);
        param_interp_add_key(&mut interp, 1.0, 1.0);
        param_interp_add_key(&mut interp, 2.0, 2.0); // should be ignored
        assert_eq!(param_interp_key_count(&interp), 2);
    }
}
