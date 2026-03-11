//! Skin tension/stretch morph — applies corrective morphs at joints under extreme poses.

/// Configuration for the skin tension system.
#[allow(dead_code)]
pub struct SkinTensionConfig {
    /// Global sensitivity multiplier applied to all tension zones.
    pub global_sensitivity: f32,
    /// Angle (radians) below which no tension is applied.
    pub default_threshold: f32,
    /// Maximum weight clamp for any single tension zone.
    pub max_weight_clamp: f32,
}

/// A single tension zone associated with a named joint.
#[allow(dead_code)]
pub struct TensionZone {
    /// Name of the associated joint.
    pub joint_name: String,
    /// Bend angle threshold (radians) below which no tension activates.
    pub threshold: f32,
    /// Maximum morph weight for this zone.
    pub max_weight: f32,
    /// Current computed morph weight [0..max_weight].
    pub current_weight: f32,
}

/// Runtime state for the skin tension morph system.
#[allow(dead_code)]
pub struct SkinTensionState {
    /// All registered tension zones.
    pub zones: Vec<TensionZone>,
    /// Global sensitivity multiplier.
    pub sensitivity: f32,
}

/// Returns a default `SkinTensionConfig`.
#[allow(dead_code)]
pub fn default_skin_tension_config() -> SkinTensionConfig {
    SkinTensionConfig {
        global_sensitivity: 1.0,
        default_threshold: 0.2,
        max_weight_clamp: 1.0,
    }
}

/// Creates a new `SkinTensionState` from a config.
#[allow(dead_code)]
pub fn new_skin_tension_state(cfg: &SkinTensionConfig) -> SkinTensionState {
    SkinTensionState {
        zones: Vec::new(),
        sensitivity: cfg.global_sensitivity,
    }
}

/// Adds a new tension zone for the named joint.
/// Does nothing if a zone for that joint already exists.
#[allow(dead_code)]
pub fn add_tension_zone(
    state: &mut SkinTensionState,
    joint_name: &str,
    threshold: f32,
    max_weight: f32,
) {
    if state.zones.iter().any(|z| z.joint_name == joint_name) {
        return;
    }
    state.zones.push(TensionZone {
        joint_name: joint_name.to_string(),
        threshold: threshold.max(0.0),
        max_weight: max_weight.clamp(0.0, 1.0),
        current_weight: 0.0,
    });
}

/// Updates the tension weight for a joint given the current bend angle (radians).
/// If no zone exists for the joint, does nothing.
#[allow(dead_code)]
pub fn update_tension(state: &mut SkinTensionState, joint_name: &str, bend_angle_rad: f32) {
    let sensitivity = state.sensitivity;
    if let Some(zone) = state.zones.iter_mut().find(|z| z.joint_name == joint_name) {
        let excess = (bend_angle_rad.abs() - zone.threshold).max(0.0);
        let raw = excess * sensitivity;
        zone.current_weight = raw.clamp(0.0, zone.max_weight);
    }
}

/// Returns the current morph weight for the named joint's tension zone, or 0.0 if not found.
#[allow(dead_code)]
pub fn tension_morph_weight(state: &SkinTensionState, joint_name: &str) -> f32 {
    state
        .zones
        .iter()
        .find(|z| z.joint_name == joint_name)
        .map(|z| z.current_weight)
        .unwrap_or(0.0)
}

/// Returns the number of registered tension zones.
#[allow(dead_code)]
pub fn tension_zone_count(state: &SkinTensionState) -> usize {
    state.zones.len()
}

/// Resets all tension zone weights to zero.
#[allow(dead_code)]
pub fn reset_tension(state: &mut SkinTensionState) {
    for zone in &mut state.zones {
        zone.current_weight = 0.0;
    }
}

/// Returns true if the named joint's tension zone has a non-trivial weight.
#[allow(dead_code)]
pub fn tension_zone_is_active(state: &SkinTensionState, joint_name: &str) -> bool {
    state
        .zones
        .iter()
        .find(|z| z.joint_name == joint_name)
        .map(|z| z.current_weight > 0.001)
        .unwrap_or(false)
}

/// Returns all current tension weights as a `Vec<f32>` in zone registration order.
#[allow(dead_code)]
pub fn all_tension_weights(state: &SkinTensionState) -> Vec<f32> {
    state.zones.iter().map(|z| z.current_weight).collect()
}

/// Sets the global sensitivity multiplier and re-clamps existing zone weights.
#[allow(dead_code)]
pub fn set_tension_sensitivity(state: &mut SkinTensionState, sensitivity: f32) {
    state.sensitivity = sensitivity.max(0.0);
    for zone in &mut state.zones {
        zone.current_weight = zone.current_weight.clamp(0.0, zone.max_weight);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state() -> SkinTensionState {
        let cfg = default_skin_tension_config();
        new_skin_tension_state(&cfg)
    }

    #[test]
    fn test_default_config() {
        let cfg = default_skin_tension_config();
        assert!((cfg.global_sensitivity - 1.0).abs() < 1e-6);
        assert!((cfg.max_weight_clamp - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_new_state_empty() {
        let state = make_state();
        assert_eq!(tension_zone_count(&state), 0);
    }

    #[test]
    fn test_add_tension_zone() {
        let mut state = make_state();
        add_tension_zone(&mut state, "elbow_l", 0.3, 0.8);
        assert_eq!(tension_zone_count(&state), 1);
        // Duplicate ignored
        add_tension_zone(&mut state, "elbow_l", 0.1, 0.5);
        assert_eq!(tension_zone_count(&state), 1);
    }

    #[test]
    fn test_update_tension_no_activation_below_threshold() {
        let mut state = make_state();
        add_tension_zone(&mut state, "knee_r", 1.0, 1.0);
        update_tension(&mut state, "knee_r", 0.5);
        assert!((tension_morph_weight(&state, "knee_r")).abs() < 1e-6);
    }

    #[test]
    fn test_update_tension_activates_above_threshold() {
        let mut state = make_state();
        add_tension_zone(&mut state, "knee_r", 0.5, 1.0);
        update_tension(&mut state, "knee_r", 1.2);
        let w = tension_morph_weight(&state, "knee_r");
        assert!(w > 0.0);
        assert!(w <= 1.0);
    }

    #[test]
    fn test_tension_morph_weight_unknown_joint() {
        let state = make_state();
        assert!((tension_morph_weight(&state, "does_not_exist")).abs() < 1e-6);
    }

    #[test]
    fn test_reset_tension() {
        let mut state = make_state();
        add_tension_zone(&mut state, "elbow_l", 0.1, 1.0);
        update_tension(&mut state, "elbow_l", 1.5);
        assert!(tension_zone_is_active(&state, "elbow_l"));
        reset_tension(&mut state);
        assert!(!tension_zone_is_active(&state, "elbow_l"));
    }

    #[test]
    fn test_all_tension_weights_length() {
        let mut state = make_state();
        add_tension_zone(&mut state, "shoulder_l", 0.2, 0.9);
        add_tension_zone(&mut state, "shoulder_r", 0.2, 0.9);
        let weights = all_tension_weights(&state);
        assert_eq!(weights.len(), 2);
    }

    #[test]
    fn test_set_tension_sensitivity() {
        let mut state = make_state();
        add_tension_zone(&mut state, "wrist_l", 0.1, 1.0);
        update_tension(&mut state, "wrist_l", 0.5);
        let w1 = tension_morph_weight(&state, "wrist_l");
        set_tension_sensitivity(&mut state, 2.0);
        update_tension(&mut state, "wrist_l", 0.5);
        let w2 = tension_morph_weight(&state, "wrist_l");
        assert!(w2 > w1);
    }

    #[test]
    fn test_tension_zone_is_active() {
        let mut state = make_state();
        add_tension_zone(&mut state, "hip_l", 0.3, 1.0);
        assert!(!tension_zone_is_active(&state, "hip_l"));
        update_tension(&mut state, "hip_l", 1.0);
        assert!(tension_zone_is_active(&state, "hip_l"));
    }

    #[test]
    fn test_max_weight_clamped() {
        let mut state = make_state();
        add_tension_zone(&mut state, "ankle_l", 0.1, 0.5);
        update_tension(&mut state, "ankle_l", 100.0);
        let w = tension_morph_weight(&state, "ankle_l");
        assert!(w <= 0.5 + 1e-6);
    }
}
