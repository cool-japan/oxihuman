//! Wrinkle map blend control — maps joint angles to wrinkle texture blend weights.

/// Configuration for the wrinkle map system.
#[allow(dead_code)]
pub struct WrinkleMapConfig {
    /// Default global sensitivity multiplier for all zones.
    pub global_sensitivity: f32,
    /// Default angle threshold (radians) below which no wrinkle activates.
    pub default_threshold_rad: f32,
    /// Maximum blend weight clamp for any zone.
    pub max_weight_clamp: f32,
}

/// A single wrinkle zone tied to a named region.
#[allow(dead_code)]
pub struct WrinkleZone {
    /// Human-readable zone identifier (e.g. "elbow_l", "knuckle_r").
    pub name: String,
    /// Bend angle (radians) below which the wrinkle is silent.
    pub threshold_rad: f32,
    /// Maximum allowed blend weight for this zone.
    pub max_weight: f32,
    /// Current computed blend weight [0..max_weight].
    pub current_weight: f32,
}

/// Runtime state for the wrinkle map blend system.
#[allow(dead_code)]
pub struct WrinkleMapState {
    /// All registered wrinkle zones.
    pub zones: Vec<WrinkleZone>,
    /// Global sensitivity multiplier applied when computing weights.
    pub sensitivity: f32,
}

/// Returns a default `WrinkleMapConfig`.
#[allow(dead_code)]
pub fn default_wrinkle_map_config() -> WrinkleMapConfig {
    WrinkleMapConfig {
        global_sensitivity: 1.0,
        default_threshold_rad: 0.15,
        max_weight_clamp: 1.0,
    }
}

/// Creates a new `WrinkleMapState` from the given config.
#[allow(dead_code)]
pub fn new_wrinkle_map_state(cfg: &WrinkleMapConfig) -> WrinkleMapState {
    WrinkleMapState {
        zones: Vec::new(),
        sensitivity: cfg.global_sensitivity,
    }
}

/// Adds a wrinkle zone. If a zone with that name already exists, does nothing.
#[allow(dead_code)]
pub fn add_wrinkle_zone(
    state: &mut WrinkleMapState,
    name: &str,
    threshold_rad: f32,
    max_weight: f32,
) {
    if state.zones.iter().any(|z| z.name == name) {
        return;
    }
    state.zones.push(WrinkleZone {
        name: name.to_string(),
        threshold_rad: threshold_rad.max(0.0),
        max_weight: max_weight.clamp(0.0, 1.0),
        current_weight: 0.0,
    });
}

/// Updates the wrinkle weight for the named zone given the current joint bend angle (radians).
/// If no zone matches the name, does nothing.
#[allow(dead_code)]
pub fn update_wrinkle(state: &mut WrinkleMapState, zone_name: &str, bend_angle_rad: f32) {
    let sensitivity = state.sensitivity;
    if let Some(zone) = state.zones.iter_mut().find(|z| z.name == zone_name) {
        let excess = (bend_angle_rad.abs() - zone.threshold_rad).max(0.0);
        let raw = excess * sensitivity;
        zone.current_weight = raw.clamp(0.0, zone.max_weight);
    }
}

/// Returns the current blend weight for the named zone, or `0.0` if not found.
#[allow(dead_code)]
pub fn wrinkle_weight(state: &WrinkleMapState, zone_name: &str) -> f32 {
    state
        .zones
        .iter()
        .find(|z| z.name == zone_name)
        .map(|z| z.current_weight)
        .unwrap_or(0.0)
}

/// Returns all current blend weights in zone registration order.
#[allow(dead_code)]
pub fn all_wrinkle_weights(state: &WrinkleMapState) -> Vec<f32> {
    state.zones.iter().map(|z| z.current_weight).collect()
}

/// Returns the number of registered wrinkle zones.
#[allow(dead_code)]
pub fn wrinkle_zone_count(state: &WrinkleMapState) -> usize {
    state.zones.len()
}

/// Resets all zone weights to zero without removing zones.
#[allow(dead_code)]
pub fn reset_wrinkles(state: &mut WrinkleMapState) {
    for zone in &mut state.zones {
        zone.current_weight = 0.0;
    }
}

/// Returns `true` if the named zone has a non-trivial blend weight.
#[allow(dead_code)]
pub fn wrinkle_zone_is_active(state: &WrinkleMapState, zone_name: &str) -> bool {
    state
        .zones
        .iter()
        .find(|z| z.name == zone_name)
        .map(|z| z.current_weight > 0.001)
        .unwrap_or(false)
}

/// Sets the global sensitivity multiplier. Existing weights are re-clamped.
#[allow(dead_code)]
pub fn set_wrinkle_sensitivity(state: &mut WrinkleMapState, sensitivity: f32) {
    state.sensitivity = sensitivity.max(0.0);
    for zone in &mut state.zones {
        zone.current_weight = zone.current_weight.clamp(0.0, zone.max_weight);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> WrinkleMapState {
        let cfg = default_wrinkle_map_config();
        new_wrinkle_map_state(&cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_wrinkle_map_config();
        assert!((cfg.global_sensitivity - 1.0).abs() < 1e-6);
        assert!((cfg.max_weight_clamp - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_empty() {
        let state = make_state();
        assert_eq!(wrinkle_zone_count(&state), 0);
    }

    #[test]
    fn test_add_wrinkle_zone() {
        let mut state = make_state();
        add_wrinkle_zone(&mut state, "knuckle_l", 0.2, 0.9);
        assert_eq!(wrinkle_zone_count(&state), 1);
        // Duplicate is ignored.
        add_wrinkle_zone(&mut state, "knuckle_l", 0.1, 0.5);
        assert_eq!(wrinkle_zone_count(&state), 1);
    }

    #[test]
    fn test_no_activation_below_threshold() {
        let mut state = make_state();
        add_wrinkle_zone(&mut state, "elbow_r", 1.0, 1.0);
        update_wrinkle(&mut state, "elbow_r", 0.5);
        assert!(wrinkle_weight(&state, "elbow_r").abs() < 1e-6);
    }

    #[test]
    fn test_activation_above_threshold() {
        let mut state = make_state();
        add_wrinkle_zone(&mut state, "elbow_r", 0.3, 1.0);
        update_wrinkle(&mut state, "elbow_r", 1.0);
        let w = wrinkle_weight(&state, "elbow_r");
        assert!(w > 0.0);
        assert!(w <= 1.0);
    }

    #[test]
    fn test_unknown_zone_returns_zero() {
        let state = make_state();
        assert!((wrinkle_weight(&state, "nonexistent")).abs() < 1e-6);
    }

    #[test]
    fn test_reset_wrinkles() {
        let mut state = make_state();
        add_wrinkle_zone(&mut state, "knee_l", 0.1, 1.0);
        update_wrinkle(&mut state, "knee_l", 1.5);
        assert!(wrinkle_zone_is_active(&state, "knee_l"));
        reset_wrinkles(&mut state);
        assert!(!wrinkle_zone_is_active(&state, "knee_l"));
    }

    #[test]
    fn test_all_wrinkle_weights_length() {
        let mut state = make_state();
        add_wrinkle_zone(&mut state, "shoulder_l", 0.2, 0.9);
        add_wrinkle_zone(&mut state, "shoulder_r", 0.2, 0.9);
        assert_eq!(all_wrinkle_weights(&state).len(), 2);
    }

    #[test]
    fn test_set_wrinkle_sensitivity_increases_weight() {
        let mut state = make_state();
        add_wrinkle_zone(&mut state, "wrist_l", 0.1, 1.0);
        update_wrinkle(&mut state, "wrist_l", 0.5);
        let w1 = wrinkle_weight(&state, "wrist_l");
        set_wrinkle_sensitivity(&mut state, 3.0);
        update_wrinkle(&mut state, "wrist_l", 0.5);
        let w2 = wrinkle_weight(&state, "wrist_l");
        assert!(w2 > w1);
    }

    #[test]
    fn test_max_weight_clamped() {
        let mut state = make_state();
        add_wrinkle_zone(&mut state, "ankle_l", 0.1, 0.4);
        update_wrinkle(&mut state, "ankle_l", 100.0);
        let w = wrinkle_weight(&state, "ankle_l");
        assert!(w <= 0.4 + 1e-6);
    }

    #[test]
    fn test_zone_is_active_false_before_update() {
        let mut state = make_state();
        add_wrinkle_zone(&mut state, "hip_l", 0.3, 1.0);
        assert!(!wrinkle_zone_is_active(&state, "hip_l"));
        update_wrinkle(&mut state, "hip_l", 1.0);
        assert!(wrinkle_zone_is_active(&state, "hip_l"));
    }
}
