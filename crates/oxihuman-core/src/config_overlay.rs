//! Config layering/overlay system — merges multiple config layers with priority ordering.

use std::collections::HashMap;

/// Priority level for a configuration layer.
/// Higher numeric values win when the same key exists in multiple layers.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OverlayPriority {
    /// Lowest priority — shipped defaults.
    Base,
    /// User-level overrides.
    User,
    /// Runtime/session overrides.
    Session,
    /// Highest priority — programmatic overrides.
    Override,
}

/// A single configuration layer mapping string keys to string values.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigLayer {
    /// Priority of this layer.
    pub priority: OverlayPriority,
    /// Key-value pairs for this layer.
    pub entries: HashMap<String, String>,
}

/// A multi-layer config overlay that resolves values by priority.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConfigOverlay {
    /// All registered layers, keyed by priority.
    pub layers: HashMap<OverlayPriority, ConfigLayer>,
}

/// Creates an empty `ConfigOverlay` with no layers.
#[allow(dead_code)]
pub fn new_config_overlay() -> ConfigOverlay {
    ConfigOverlay {
        layers: HashMap::new(),
    }
}

/// Sets (or overwrites) a key in the layer at the given priority.
/// If the layer does not exist, it is created.
#[allow(dead_code)]
pub fn overlay_set(overlay: &mut ConfigOverlay, priority: OverlayPriority, key: &str, value: &str) {
    let layer = overlay.layers.entry(priority).or_insert_with(|| ConfigLayer {
        priority,
        entries: HashMap::new(),
    });
    layer.entries.insert(key.to_string(), value.to_string());
}

/// Returns the value for `key` from the highest-priority layer that contains it,
/// or `None` if no layer holds the key.
#[allow(dead_code)]
pub fn overlay_get<'a>(overlay: &'a ConfigOverlay, key: &str) -> Option<&'a str> {
    // Priorities sorted descending so the first hit wins.
    let mut priorities: Vec<OverlayPriority> = overlay.layers.keys().copied().collect();
    priorities.sort_by(|a, b| b.cmp(a));
    for p in priorities {
        if let Some(layer) = overlay.layers.get(&p) {
            if let Some(val) = layer.entries.get(key) {
                return Some(val.as_str());
            }
        }
    }
    None
}

/// Removes a key from the specified priority layer.
/// Returns `true` if the key existed and was removed.
#[allow(dead_code)]
pub fn overlay_remove(
    overlay: &mut ConfigOverlay,
    priority: OverlayPriority,
    key: &str,
) -> bool {
    if let Some(layer) = overlay.layers.get_mut(&priority) {
        return layer.entries.remove(key).is_some();
    }
    false
}

/// Returns the number of active layers (layers with at least one entry).
#[allow(dead_code)]
pub fn overlay_layer_count(overlay: &ConfigOverlay) -> usize {
    overlay
        .layers
        .values()
        .filter(|l| !l.entries.is_empty())
        .count()
}

/// Returns the total number of unique keys across all layers.
/// Keys present in multiple layers are counted once.
#[allow(dead_code)]
pub fn overlay_key_count(overlay: &ConfigOverlay) -> usize {
    let mut keys: Vec<&str> = Vec::new();
    for layer in overlay.layers.values() {
        for k in layer.entries.keys() {
            if !keys.contains(&k.as_str()) {
                keys.push(k.as_str());
            }
        }
    }
    keys.len()
}

/// Removes all entries in the layer at the given priority (the layer itself is also dropped).
#[allow(dead_code)]
pub fn overlay_clear_layer(overlay: &mut ConfigOverlay, priority: OverlayPriority) {
    overlay.layers.remove(&priority);
}

/// Returns a flat list of all `(key, resolved_value)` pairs using the highest-priority value
/// for each key. The list is sorted by key for deterministic output.
#[allow(dead_code)]
pub fn overlay_to_flat_map(overlay: &ConfigOverlay) -> Vec<(String, String)> {
    // Collect all unique keys.
    let mut keys: Vec<String> = Vec::new();
    for layer in overlay.layers.values() {
        for k in layer.entries.keys() {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
    }
    keys.sort();
    keys.into_iter()
        .filter_map(|k| overlay_get(overlay, &k).map(|v| (k, v.to_string())))
        .collect()
}

/// Returns the numeric priority value for an `OverlayPriority` variant.
/// Base=0, User=1, Session=2, Override=3.
#[allow(dead_code)]
pub fn overlay_priority_value(p: OverlayPriority) -> u8 {
    match p {
        OverlayPriority::Base => 0,
        OverlayPriority::User => 1,
        OverlayPriority::Session => 2,
        OverlayPriority::Override => 3,
    }
}

/// Merges `other` on top of `base`, returning a new overlay.
/// Entries in `other` take precedence over entries in `base` for the same (priority, key) pair.
#[allow(dead_code)]
pub fn overlay_merge(base: &ConfigOverlay, other: &ConfigOverlay) -> ConfigOverlay {
    let mut result = base.clone();
    for (priority, layer) in &other.layers {
        for (k, v) in &layer.entries {
            overlay_set(&mut result, *priority, k, v);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_overlay() -> ConfigOverlay {
        new_config_overlay()
    }

    #[test]
    fn test_new_overlay_empty() {
        let ov = make_overlay();
        assert_eq!(overlay_layer_count(&ov), 0);
        assert_eq!(overlay_key_count(&ov), 0);
    }

    #[test]
    fn test_set_and_get() {
        let mut ov = make_overlay();
        overlay_set(&mut ov, OverlayPriority::Base, "width", "800");
        assert_eq!(overlay_get(&ov, "width"), Some("800"));
    }

    #[test]
    fn test_higher_priority_wins() {
        let mut ov = make_overlay();
        overlay_set(&mut ov, OverlayPriority::Base, "theme", "dark");
        overlay_set(&mut ov, OverlayPriority::User, "theme", "light");
        assert_eq!(overlay_get(&ov, "theme"), Some("light"));
    }

    #[test]
    fn test_override_wins_all() {
        let mut ov = make_overlay();
        overlay_set(&mut ov, OverlayPriority::Base, "fps", "30");
        overlay_set(&mut ov, OverlayPriority::User, "fps", "60");
        overlay_set(&mut ov, OverlayPriority::Override, "fps", "120");
        assert_eq!(overlay_get(&ov, "fps"), Some("120"));
    }

    #[test]
    fn test_get_missing_key_returns_none() {
        let ov = make_overlay();
        assert_eq!(overlay_get(&ov, "nonexistent"), None);
    }

    #[test]
    fn test_overlay_remove() {
        let mut ov = make_overlay();
        overlay_set(&mut ov, OverlayPriority::User, "lang", "en");
        assert!(overlay_remove(&mut ov, OverlayPriority::User, "lang"));
        assert_eq!(overlay_get(&ov, "lang"), None);
        // Removing non-existent key returns false.
        assert!(!overlay_remove(&mut ov, OverlayPriority::User, "lang"));
    }

    #[test]
    fn test_overlay_clear_layer() {
        let mut ov = make_overlay();
        overlay_set(&mut ov, OverlayPriority::Session, "volume", "80");
        overlay_set(&mut ov, OverlayPriority::Base, "volume", "50");
        overlay_clear_layer(&mut ov, OverlayPriority::Session);
        assert_eq!(overlay_get(&ov, "volume"), Some("50"));
    }

    #[test]
    fn test_overlay_to_flat_map() {
        let mut ov = make_overlay();
        overlay_set(&mut ov, OverlayPriority::Base, "a", "1");
        overlay_set(&mut ov, OverlayPriority::User, "b", "2");
        overlay_set(&mut ov, OverlayPriority::User, "a", "3");
        let flat = overlay_to_flat_map(&ov);
        // "a" resolves to User value "3", "b" resolves to "2".
        let a_val = flat.iter().find(|(k, _)| k == "a").map(|(_, v)| v.as_str());
        let b_val = flat.iter().find(|(k, _)| k == "b").map(|(_, v)| v.as_str());
        assert_eq!(a_val, Some("3"));
        assert_eq!(b_val, Some("2"));
    }

    #[test]
    fn test_overlay_merge() {
        let mut base = make_overlay();
        let mut other = make_overlay();
        overlay_set(&mut base, OverlayPriority::Base, "x", "base_x");
        overlay_set(&mut other, OverlayPriority::Base, "x", "other_x");
        overlay_set(&mut other, OverlayPriority::User, "y", "other_y");
        let merged = overlay_merge(&base, &other);
        // other overrides base for key "x" at Base priority.
        assert_eq!(overlay_get(&merged, "x"), Some("other_x"));
        assert_eq!(overlay_get(&merged, "y"), Some("other_y"));
    }

    #[test]
    fn test_priority_values_ordering() {
        assert!(overlay_priority_value(OverlayPriority::Base) < overlay_priority_value(OverlayPriority::User));
        assert!(overlay_priority_value(OverlayPriority::User) < overlay_priority_value(OverlayPriority::Session));
        assert!(overlay_priority_value(OverlayPriority::Session) < overlay_priority_value(OverlayPriority::Override));
    }
}
