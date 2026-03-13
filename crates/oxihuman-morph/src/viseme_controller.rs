//! Viseme-based lip-sync controller mapping phonemes to mouth shapes.
//!
//! A viseme is a visual representation of a phoneme (speech sound). This module
//! provides a controller that maps phoneme strings to morph target weights,
//! allowing real-time lip synchronization for digital humans.

#[allow(dead_code)]
/// Configuration for the viseme controller.
#[derive(Debug, Clone)]
pub struct VisemeConfig {
    /// Maximum number of viseme entries.
    pub max_entries: usize,
    /// Blend speed for smooth transitions (weight units per step).
    pub blend_speed: f32,
}

#[allow(dead_code)]
impl VisemeConfig {
    fn new() -> Self {
        Self {
            max_entries: 32,
            blend_speed: 0.1,
        }
    }
}

/// Returns the default viseme configuration.
#[allow(dead_code)]
pub fn default_viseme_config() -> VisemeConfig {
    VisemeConfig::new()
}

/// A single viseme entry mapping a phoneme label to a morph weight.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VisemeEntry {
    /// Phoneme label (e.g. "AH", "EE", "OO").
    pub phoneme: String,
    /// Target morph weight [0.0, 1.0].
    pub weight: f32,
}

/// Lip-sync viseme controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VisemeController {
    config: VisemeConfig,
    entries: Vec<VisemeEntry>,
    active_index: Option<usize>,
    current_weight: f32,
}

/// Creates a new `VisemeController` with the given configuration.
#[allow(dead_code)]
pub fn new_viseme_controller(config: VisemeConfig) -> VisemeController {
    VisemeController {
        config,
        entries: Vec::new(),
        active_index: None,
        current_weight: 0.0,
    }
}

/// Adds a viseme entry (phoneme → weight) to the controller.
/// Returns `false` if the entry limit is reached.
#[allow(dead_code)]
pub fn viseme_add_entry(ctrl: &mut VisemeController, phoneme: &str, weight: f32) -> bool {
    if ctrl.entries.len() >= ctrl.config.max_entries {
        return false;
    }
    // Replace existing phoneme if already present.
    for e in &mut ctrl.entries {
        if e.phoneme == phoneme {
            e.weight = weight.clamp(0.0, 1.0);
            return true;
        }
    }
    ctrl.entries.push(VisemeEntry {
        phoneme: phoneme.to_string(),
        weight: weight.clamp(0.0, 1.0),
    });
    true
}

/// Sets the active phoneme by name. Returns `false` if phoneme not found.
#[allow(dead_code)]
pub fn viseme_set_active(ctrl: &mut VisemeController, phoneme: &str) -> bool {
    for (i, e) in ctrl.entries.iter().enumerate() {
        if e.phoneme == phoneme {
            ctrl.active_index = Some(i);
            return true;
        }
    }
    false
}

/// Returns the current blended weight of the active viseme.
#[allow(dead_code)]
pub fn viseme_weight(ctrl: &VisemeController) -> f32 {
    ctrl.current_weight
}

/// Returns the number of viseme entries registered.
#[allow(dead_code)]
pub fn viseme_entry_count(ctrl: &VisemeController) -> usize {
    ctrl.entries.len()
}

/// Steps the controller towards the active viseme target weight.
/// Call once per frame. Returns the new current weight.
#[allow(dead_code)]
pub fn viseme_blend_to(ctrl: &mut VisemeController) -> f32 {
    let target = match ctrl.active_index {
        Some(i) => ctrl.entries[i].weight,
        None => 0.0,
    };
    let diff = target - ctrl.current_weight;
    if diff.abs() <= ctrl.config.blend_speed {
        ctrl.current_weight = target;
    } else {
        ctrl.current_weight += diff.signum() * ctrl.config.blend_speed;
    }
    ctrl.current_weight
}

/// Clears all viseme entries and resets the controller.
#[allow(dead_code)]
pub fn viseme_clear(ctrl: &mut VisemeController) {
    ctrl.entries.clear();
    ctrl.active_index = None;
    ctrl.current_weight = 0.0;
}

/// Serialises the controller to a simple JSON string.
#[allow(dead_code)]
pub fn viseme_controller_to_json(ctrl: &VisemeController) -> String {
    let entries: Vec<String> = ctrl
        .entries
        .iter()
        .map(|e| format!("{{\"phoneme\":\"{}\",\"weight\":{:.4}}}", e.phoneme, e.weight))
        .collect();
    format!(
        "{{\"active_index\":{},\"current_weight\":{:.4},\"entries\":[{}]}}",
        ctrl.active_index
            .map(|i| i.to_string())
            .unwrap_or_else(|| "null".to_string()),
        ctrl.current_weight,
        entries.join(",")
    )
}

/// Returns a list of all phoneme labels registered in the controller.
#[allow(dead_code)]
pub fn viseme_phoneme_list(ctrl: &VisemeController) -> Vec<String> {
    ctrl.entries.iter().map(|e| e.phoneme.clone()).collect()
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctrl() -> VisemeController {
        let cfg = default_viseme_config();
        let mut ctrl = new_viseme_controller(cfg);
        viseme_add_entry(&mut ctrl, "AH", 0.9);
        viseme_add_entry(&mut ctrl, "EE", 0.6);
        viseme_add_entry(&mut ctrl, "OO", 0.75);
        ctrl
    }

    #[test]
    fn test_entry_count() {
        let ctrl = make_ctrl();
        assert_eq!(viseme_entry_count(&ctrl), 3);
    }

    #[test]
    fn test_add_and_retrieve_phoneme_list() {
        let ctrl = make_ctrl();
        let list = viseme_phoneme_list(&ctrl);
        assert!(list.contains(&"AH".to_string()));
        assert!(list.contains(&"EE".to_string()));
        assert!(list.contains(&"OO".to_string()));
    }

    #[test]
    fn test_set_active_valid() {
        let mut ctrl = make_ctrl();
        assert!(viseme_set_active(&mut ctrl, "EE"));
    }

    #[test]
    fn test_set_active_invalid() {
        let mut ctrl = make_ctrl();
        assert!(!viseme_set_active(&mut ctrl, "ZZ"));
    }

    #[test]
    fn test_blend_to_converges() {
        let mut ctrl = make_ctrl();
        viseme_set_active(&mut ctrl, "AH");
        for _ in 0..100 {
            viseme_blend_to(&mut ctrl);
        }
        assert!((viseme_weight(&ctrl) - 0.9).abs() < 1e-4);
    }

    #[test]
    fn test_clear_resets_state() {
        let mut ctrl = make_ctrl();
        viseme_set_active(&mut ctrl, "OO");
        viseme_blend_to(&mut ctrl);
        viseme_clear(&mut ctrl);
        assert_eq!(viseme_entry_count(&ctrl), 0);
        assert_eq!(viseme_weight(&ctrl), 0.0);
    }

    #[test]
    fn test_duplicate_phoneme_updates_weight() {
        let mut ctrl = make_ctrl();
        viseme_add_entry(&mut ctrl, "AH", 0.2);
        // Count should stay the same.
        assert_eq!(viseme_entry_count(&ctrl), 3);
        viseme_set_active(&mut ctrl, "AH");
        for _ in 0..100 {
            viseme_blend_to(&mut ctrl);
        }
        assert!((viseme_weight(&ctrl) - 0.2).abs() < 1e-4);
    }

    #[test]
    fn test_weight_clamp() {
        let mut ctrl = new_viseme_controller(default_viseme_config());
        viseme_add_entry(&mut ctrl, "X", 2.5);
        let list = ctrl.entries.iter().find(|e| e.phoneme == "X").expect("should succeed");
        assert!(list.weight <= 1.0);
    }

    #[test]
    fn test_to_json_contains_phoneme() {
        let ctrl = make_ctrl();
        let json = viseme_controller_to_json(&ctrl);
        assert!(json.contains("\"AH\""));
        assert!(json.contains("entries"));
    }

    #[test]
    fn test_max_entries_limit() {
        let cfg = VisemeConfig {
            max_entries: 2,
            blend_speed: 0.1,
        };
        let mut ctrl = new_viseme_controller(cfg);
        assert!(viseme_add_entry(&mut ctrl, "A", 0.5));
        assert!(viseme_add_entry(&mut ctrl, "B", 0.5));
        assert!(!viseme_add_entry(&mut ctrl, "C", 0.5));
        assert_eq!(viseme_entry_count(&ctrl), 2);
    }

    #[test]
    fn test_no_active_blends_to_zero() {
        let mut ctrl = make_ctrl();
        // Never set active, weight should trend toward 0.
        for _ in 0..50 {
            viseme_blend_to(&mut ctrl);
        }
        assert!((viseme_weight(&ctrl)).abs() < 1e-4);
    }
}
