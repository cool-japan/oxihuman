//! Overall body shape morph controller — height, weight, muscle, and fat distribution.

/// Configuration for the body shape morph system.
#[allow(dead_code)]
pub struct BodyShapeConfig {
    /// Minimum allowed height scale factor.
    pub min_height_scale: f32,
    /// Maximum allowed height scale factor.
    pub max_height_scale: f32,
    /// Minimum allowed weight value [0..1].
    pub min_weight: f32,
    /// Maximum allowed weight value [0..1].
    pub max_weight: f32,
}

/// Runtime state holding the four body shape dimensions.
#[allow(dead_code)]
#[derive(Clone)]
pub struct BodyShapeState {
    /// Vertical scale relative to neutral (1.0 = neutral).
    pub height_scale: f32,
    /// Overall body mass morph weight [0..1].
    pub weight: f32,
    /// Muscle definition morph weight [0..1].
    pub muscle: f32,
    /// Body fat distribution morph weight [0..1].
    pub fat: f32,
}

/// Returns a default `BodyShapeConfig`.
#[allow(dead_code)]
pub fn default_body_shape_config() -> BodyShapeConfig {
    BodyShapeConfig {
        min_height_scale: 0.5,
        max_height_scale: 2.0,
        min_weight: 0.0,
        max_weight: 1.0,
    }
}

/// Creates a new `BodyShapeState` initialised to neutral values.
#[allow(dead_code)]
pub fn new_body_shape_state(cfg: &BodyShapeConfig) -> BodyShapeState {
    let height_scale = 1.0f32.clamp(cfg.min_height_scale, cfg.max_height_scale);
    BodyShapeState {
        height_scale,
        weight: 0.5,
        muscle: 0.5,
        fat: 0.5,
    }
}

/// Sets the height scale, clamped to [0.5 .. 2.0].
#[allow(dead_code)]
pub fn set_body_height_scale(state: &mut BodyShapeState, scale: f32) {
    state.height_scale = scale.clamp(0.5, 2.0);
}

/// Sets the body weight morph weight, clamped to [0 .. 1].
#[allow(dead_code)]
pub fn set_body_weight(state: &mut BodyShapeState, weight: f32) {
    state.weight = weight.clamp(0.0, 1.0);
}

/// Sets the muscle morph weight, clamped to [0 .. 1].
#[allow(dead_code)]
pub fn set_body_muscle(state: &mut BodyShapeState, muscle: f32) {
    state.muscle = muscle.clamp(0.0, 1.0);
}

/// Sets the fat morph weight, clamped to [0 .. 1].
#[allow(dead_code)]
pub fn set_body_fat(state: &mut BodyShapeState, fat: f32) {
    state.fat = fat.clamp(0.0, 1.0);
}

/// Returns the four morph weights as `[height_scale, weight, muscle, fat]`.
#[allow(dead_code)]
pub fn body_shape_morph_weights(state: &BodyShapeState) -> [f32; 4] {
    [state.height_scale, state.weight, state.muscle, state.fat]
}

/// Resets the body shape state to neutral (height_scale 1.0, all others 0.5).
#[allow(dead_code)]
pub fn reset_body_shape(state: &mut BodyShapeState) {
    state.height_scale = 1.0;
    state.weight = 0.5;
    state.muscle = 0.5;
    state.fat = 0.5;
}

/// Linearly interpolates between two `BodyShapeState` values. `t` is clamped to [0 .. 1].
#[allow(dead_code)]
pub fn blend_body_shapes(a: &BodyShapeState, b: &BodyShapeState, t: f32) -> BodyShapeState {
    let t = t.clamp(0.0, 1.0);
    BodyShapeState {
        height_scale: a.height_scale + (b.height_scale - a.height_scale) * t,
        weight: a.weight + (b.weight - a.weight) * t,
        muscle: a.muscle + (b.muscle - a.muscle) * t,
        fat: a.fat + (b.fat - a.fat) * t,
    }
}

/// Estimates a simple BMI-like scalar from weight, height_scale, and fat.
/// Formula: `(weight + fat) / (height_scale * height_scale)`, normalised to [0 .. 1].
#[allow(dead_code)]
pub fn body_bmi_estimate(state: &BodyShapeState) -> f32 {
    let numerator = (state.weight + state.fat) * 0.5;
    let denom = (state.height_scale * state.height_scale).max(0.01);
    (numerator / denom).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> BodyShapeState {
        let cfg = default_body_shape_config();
        new_body_shape_state(&cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_body_shape_config();
        assert!((cfg.min_height_scale - 0.5).abs() < 1e-6);
        assert!((cfg.max_height_scale - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_neutral() {
        let state = make_state();
        assert!((state.height_scale - 1.0).abs() < 1e-6);
        assert!((state.weight - 0.5).abs() < 1e-6);
        assert!((state.muscle - 0.5).abs() < 1e-6);
        assert!((state.fat - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_body_height_scale_clamped() {
        let mut state = make_state();
        set_body_height_scale(&mut state, 5.0);
        assert!((state.height_scale - 2.0).abs() < 1e-6);
        set_body_height_scale(&mut state, -1.0);
        assert!((state.height_scale - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_body_weight_clamped() {
        let mut state = make_state();
        set_body_weight(&mut state, 1.5);
        assert!((state.weight - 1.0).abs() < 1e-6);
        set_body_weight(&mut state, -0.5);
        assert!((state.weight - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_body_muscle_and_fat() {
        let mut state = make_state();
        set_body_muscle(&mut state, 0.8);
        set_body_fat(&mut state, 0.2);
        assert!((state.muscle - 0.8).abs() < 1e-6);
        assert!((state.fat - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_morph_weights_array() {
        let mut state = make_state();
        set_body_height_scale(&mut state, 1.2);
        set_body_weight(&mut state, 0.7);
        set_body_muscle(&mut state, 0.6);
        set_body_fat(&mut state, 0.3);
        let w = body_shape_morph_weights(&state);
        assert!((w[0] - 1.2).abs() < 1e-5);
        assert!((w[1] - 0.7).abs() < 1e-5);
        assert!((w[2] - 0.6).abs() < 1e-5);
        assert!((w[3] - 0.3).abs() < 1e-5);
    }

    #[test]
    fn test_reset_body_shape() {
        let mut state = make_state();
        set_body_height_scale(&mut state, 1.8);
        set_body_weight(&mut state, 0.9);
        reset_body_shape(&mut state);
        assert!((state.height_scale - 1.0).abs() < 1e-6);
        assert!((state.weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_blend_body_shapes_midpoint() {
        let mut a = make_state();
        let mut b = make_state();
        set_body_height_scale(&mut a, 1.0);
        set_body_height_scale(&mut b, 2.0);
        let mid = blend_body_shapes(&a, &b, 0.5);
        assert!((mid.height_scale - 1.5).abs() < 1e-5);
    }

    #[test]
    fn test_blend_body_shapes_t_clamped() {
        let a = make_state();
        let mut b = make_state();
        set_body_weight(&mut b, 1.0);
        let blended = blend_body_shapes(&a, &b, 2.0);
        // t clamped to 1.0 → result should equal b
        assert!((blended.weight - b.weight).abs() < 1e-5);
    }

    #[test]
    fn test_bmi_estimate_in_range() {
        let state = make_state();
        let bmi = body_bmi_estimate(&state);
        assert!(bmi >= 0.0);
        assert!(bmi <= 1.0);
    }
}
