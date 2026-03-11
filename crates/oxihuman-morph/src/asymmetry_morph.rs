//! Asymmetry morph module.
//!
//! Applies controlled asymmetry offsets to left/right morph target pairs,
//! producing subtle organic variation that avoids the "perfectly symmetrical"
//! look of procedurally generated digital humans.

/// Configuration for the asymmetry morph system.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AsymmetryConfig {
    /// Maximum number of asymmetry entries.
    pub max_entries: usize,
    /// Global strength multiplier [0.0, 1.0].
    pub global_strength: f32,
}

#[allow(dead_code)]
impl AsymmetryConfig {
    fn new() -> Self {
        Self {
            max_entries: 64,
            global_strength: 1.0,
        }
    }
}

/// Returns the default asymmetry configuration.
#[allow(dead_code)]
pub fn default_asymmetry_config() -> AsymmetryConfig {
    AsymmetryConfig::new()
}

/// One left/right asymmetry entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AsymmetryEntry {
    /// Human-readable label (e.g. "mouth_corner").
    pub label: String,
    /// Offset applied to the left side weight (positive = more on left).
    pub left_offset: f32,
    /// Offset applied to the right side weight (positive = more on right).
    pub right_offset: f32,
    /// Per-entry strength [0.0, 1.0].
    pub strength: f32,
}

/// Asymmetry morph controller.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AsymmetryMorph {
    config: AsymmetryConfig,
    entries: Vec<AsymmetryEntry>,
    active: bool,
}

/// Creates a new `AsymmetryMorph` with the given configuration.
#[allow(dead_code)]
pub fn new_asymmetry_morph(config: AsymmetryConfig) -> AsymmetryMorph {
    AsymmetryMorph {
        config,
        entries: Vec::new(),
        active: true,
    }
}

/// Adds an asymmetry entry. Returns `false` when the entry limit is reached.
#[allow(dead_code)]
pub fn asymmetry_add_entry(
    morph: &mut AsymmetryMorph,
    label: &str,
    left_offset: f32,
    right_offset: f32,
    strength: f32,
) -> bool {
    if morph.entries.len() >= morph.config.max_entries {
        return false;
    }
    morph.entries.push(AsymmetryEntry {
        label: label.to_string(),
        left_offset: left_offset.clamp(-1.0, 1.0),
        right_offset: right_offset.clamp(-1.0, 1.0),
        strength: strength.clamp(0.0, 1.0),
    });
    true
}

/// Applies asymmetry to a pair of base weights, returning `(left_out, right_out)`.
/// The result is clamped to [0.0, 1.0].
#[allow(dead_code)]
pub fn asymmetry_apply(morph: &AsymmetryMorph, label: &str, base_left: f32, base_right: f32) -> (f32, f32) {
    if !morph.active {
        return (base_left, base_right);
    }
    let mut left = base_left;
    let mut right = base_right;
    let gs = morph.config.global_strength;
    for e in &morph.entries {
        if e.label == label {
            let eff = e.strength * gs;
            left = (left + e.left_offset * eff).clamp(0.0, 1.0);
            right = (right + e.right_offset * eff).clamp(0.0, 1.0);
        }
    }
    (left, right)
}

/// Returns the number of asymmetry entries.
#[allow(dead_code)]
pub fn asymmetry_entry_count(morph: &AsymmetryMorph) -> usize {
    morph.entries.len()
}

/// Sets the global strength multiplier.
#[allow(dead_code)]
pub fn asymmetry_set_strength(morph: &mut AsymmetryMorph, strength: f32) {
    morph.config.global_strength = strength.clamp(0.0, 1.0);
}

/// Removes all asymmetry entries and deactivates the morph.
#[allow(dead_code)]
pub fn asymmetry_clear(morph: &mut AsymmetryMorph) {
    morph.entries.clear();
    morph.active = false;
}

/// Serialises the morph to a simple JSON string.
#[allow(dead_code)]
pub fn asymmetry_to_json(morph: &AsymmetryMorph) -> String {
    let entries: Vec<String> = morph
        .entries
        .iter()
        .map(|e| {
            format!(
                "{{\"label\":\"{}\",\"left_offset\":{:.4},\"right_offset\":{:.4},\"strength\":{:.4}}}",
                e.label, e.left_offset, e.right_offset, e.strength
            )
        })
        .collect();
    format!(
        "{{\"active\":{},\"global_strength\":{:.4},\"entries\":[{}]}}",
        morph.active,
        morph.config.global_strength,
        entries.join(",")
    )
}

/// Re-enables the morph without changing entries.
#[allow(dead_code)]
pub fn asymmetry_reset(morph: &mut AsymmetryMorph) {
    morph.active = true;
}

/// Returns `true` if the morph is active.
#[allow(dead_code)]
pub fn asymmetry_is_active(morph: &AsymmetryMorph) -> bool {
    morph.active
}

// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    fn make_morph() -> AsymmetryMorph {
        let cfg = default_asymmetry_config();
        let mut m = new_asymmetry_morph(cfg);
        asymmetry_add_entry(&mut m, "mouth_corner", 0.1, -0.1, 1.0);
        asymmetry_add_entry(&mut m, "eye_lid", 0.05, 0.0, 0.5);
        m
    }

    #[test]
    fn test_entry_count() {
        let m = make_morph();
        assert_eq!(asymmetry_entry_count(&m), 2);
    }

    #[test]
    fn test_apply_mouth_corner() {
        let m = make_morph();
        let (l, r) = asymmetry_apply(&m, "mouth_corner", 0.5, 0.5);
        assert!((l - 0.6).abs() < 1e-4);
        assert!((r - 0.4).abs() < 1e-4);
    }

    #[test]
    fn test_apply_unknown_label_unchanged() {
        let m = make_morph();
        let (l, r) = asymmetry_apply(&m, "nose", 0.3, 0.7);
        assert!((l - 0.3).abs() < 1e-4);
        assert!((r - 0.7).abs() < 1e-4);
    }

    #[test]
    fn test_inactive_morph_passthrough() {
        let mut m = make_morph();
        asymmetry_clear(&mut m);
        let (l, r) = asymmetry_apply(&m, "mouth_corner", 0.5, 0.5);
        assert!((l - 0.5).abs() < 1e-4);
        assert!((r - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_set_strength_zero_no_effect() {
        let mut m = make_morph();
        asymmetry_set_strength(&mut m, 0.0);
        let (l, r) = asymmetry_apply(&m, "mouth_corner", 0.5, 0.5);
        assert!((l - 0.5).abs() < 1e-4);
        assert!((r - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_clamp_on_overflow() {
        let cfg = default_asymmetry_config();
        let mut m = new_asymmetry_morph(cfg);
        asymmetry_add_entry(&mut m, "test", 1.0, -1.0, 1.0);
        let (l, r) = asymmetry_apply(&m, "test", 0.9, 0.1);
        assert!(l <= 1.0);
        assert!(r >= 0.0);
    }

    #[test]
    fn test_reset_reactivates() {
        let mut m = make_morph();
        asymmetry_clear(&mut m);
        assert!(!asymmetry_is_active(&m));
        asymmetry_reset(&mut m);
        assert!(asymmetry_is_active(&m));
    }

    #[test]
    fn test_to_json_contains_label() {
        let m = make_morph();
        let json = asymmetry_to_json(&m);
        assert!(json.contains("mouth_corner"));
        assert!(json.contains("active"));
    }

    #[test]
    fn test_max_entries_limit() {
        let cfg = AsymmetryConfig {
            max_entries: 1,
            global_strength: 1.0,
        };
        let mut m = new_asymmetry_morph(cfg);
        assert!(asymmetry_add_entry(&mut m, "a", 0.1, 0.1, 1.0));
        assert!(!asymmetry_add_entry(&mut m, "b", 0.1, 0.1, 1.0));
    }

    #[test]
    fn test_global_strength_clamp() {
        let mut m = make_morph();
        asymmetry_set_strength(&mut m, 5.0);
        assert!(m.config.global_strength <= 1.0);
    }
}
