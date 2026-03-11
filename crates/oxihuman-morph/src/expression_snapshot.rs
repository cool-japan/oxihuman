//! Capture and restore full expression state (all morph weights at a point in time).
//!
//! An [`ExpressionSnapshot`] stores a frozen copy of a morph-weight map keyed
//! by target name. Snapshots can be diffed, blended, and serialized.

use std::collections::HashMap;

/// Configuration for snapshot behaviour.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// Maximum number of entries the snapshot may hold. 0 = unlimited.
    pub max_entries: usize,
    /// Whether to strip entries with weight == 0 on capture.
    pub strip_zero_weights: bool,
}

/// A frozen copy of a morph-weight state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionSnapshot {
    config: SnapshotConfig,
    weights: HashMap<String, f32>,
}

/// Return a default [`SnapshotConfig`].
#[allow(dead_code)]
pub fn default_snapshot_config() -> SnapshotConfig {
    SnapshotConfig {
        max_entries: 0,
        strip_zero_weights: true,
    }
}

/// Create a new, empty [`ExpressionSnapshot`].
#[allow(dead_code)]
pub fn new_expression_snapshot(config: SnapshotConfig) -> ExpressionSnapshot {
    ExpressionSnapshot {
        config,
        weights: HashMap::new(),
    }
}

/// Capture the weights from `source` into `snap`.
/// Respects `strip_zero_weights` and `max_entries`.
#[allow(dead_code)]
pub fn snapshot_capture(snap: &mut ExpressionSnapshot, source: &HashMap<String, f32>) {
    snap.weights.clear();
    for (k, &v) in source {
        if snap.config.strip_zero_weights && v.abs() < 1e-9 {
            continue;
        }
        if snap.config.max_entries > 0 && snap.weights.len() >= snap.config.max_entries {
            break;
        }
        snap.weights.insert(k.clone(), v);
    }
}

/// Copy the snapshot weights into `dest`.  Weights not present in the snapshot
/// are set to 0 in `dest` for any key already in `dest`.
#[allow(dead_code)]
pub fn snapshot_restore(snap: &ExpressionSnapshot, dest: &mut HashMap<String, f32>) {
    // Zero out all current keys
    for v in dest.values_mut() {
        *v = 0.0;
    }
    for (k, &v) in &snap.weights {
        dest.insert(k.clone(), v);
    }
}

/// Return a new map containing only keys whose weights differ between `a` and `b`
/// (difference > `threshold`).  Values in the returned map are `b[key] - a[key]`.
#[allow(dead_code)]
pub fn snapshot_diff(
    a: &ExpressionSnapshot,
    b: &ExpressionSnapshot,
    threshold: f32,
) -> HashMap<String, f32> {
    let mut result = HashMap::new();
    // Collect all keys
    let mut keys: Vec<&String> = a.weights.keys().chain(b.weights.keys()).collect();
    keys.sort_unstable();
    keys.dedup();
    for k in keys {
        let va = a.weights.get(k).copied().unwrap_or(0.0);
        let vb = b.weights.get(k).copied().unwrap_or(0.0);
        if (vb - va).abs() > threshold {
            result.insert(k.clone(), vb - va);
        }
    }
    result
}

/// Return a new snapshot that is the linear interpolation between `a` and `b`
/// at parameter `t` (0 = pure `a`, 1 = pure `b`).
#[allow(dead_code)]
pub fn snapshot_blend(
    a: &ExpressionSnapshot,
    b: &ExpressionSnapshot,
    t: f32,
) -> ExpressionSnapshot {
    let t = t.clamp(0.0, 1.0);
    let mut weights = HashMap::new();
    let mut keys: Vec<&String> = a.weights.keys().chain(b.weights.keys()).collect();
    keys.sort_unstable();
    keys.dedup();
    for k in keys {
        let va = a.weights.get(k).copied().unwrap_or(0.0);
        let vb = b.weights.get(k).copied().unwrap_or(0.0);
        weights.insert(k.clone(), va + t * (vb - va));
    }
    ExpressionSnapshot {
        config: a.config.clone(),
        weights,
    }
}

/// Return the number of entries stored in the snapshot.
#[allow(dead_code)]
pub fn snapshot_entry_count(snap: &ExpressionSnapshot) -> usize {
    snap.weights.len()
}

/// Get the weight for a specific target, or `None` if not stored.
#[allow(dead_code)]
pub fn snapshot_get_weight(snap: &ExpressionSnapshot, target: &str) -> Option<f32> {
    snap.weights.get(target).copied()
}

/// Serialize the snapshot to a compact JSON string.
#[allow(dead_code)]
pub fn snapshot_to_json(snap: &ExpressionSnapshot) -> String {
    let mut entries: Vec<String> = snap
        .weights
        .iter()
        .map(|(k, v)| format!(r#""{}":{:.6}"#, k, v))
        .collect();
    entries.sort();
    format!(
        r#"{{"entry_count":{},"weights":{{{}}}}}"#,
        snap.weights.len(),
        entries.join(",")
    )
}

/// Remove all entries from the snapshot.
#[allow(dead_code)]
pub fn snapshot_clear(snap: &mut ExpressionSnapshot) {
    snap.weights.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_source() -> HashMap<String, f32> {
        let mut m = HashMap::new();
        m.insert("smile".into(), 0.8);
        m.insert("blink".into(), 0.5);
        m.insert("zero".into(), 0.0);
        m
    }

    #[test]
    fn test_capture_strips_zero() {
        let mut snap = new_expression_snapshot(default_snapshot_config());
        snapshot_capture(&mut snap, &make_source());
        // "zero" weight should have been stripped
        assert!(snapshot_get_weight(&snap, "zero").is_none());
        assert_eq!(snapshot_entry_count(&snap), 2);
    }

    #[test]
    fn test_capture_no_strip() {
        let cfg = SnapshotConfig {
            max_entries: 0,
            strip_zero_weights: false,
        };
        let mut snap = new_expression_snapshot(cfg);
        snapshot_capture(&mut snap, &make_source());
        assert!(snapshot_get_weight(&snap, "zero").is_some());
    }

    #[test]
    fn test_restore_sets_weights() {
        let mut snap = new_expression_snapshot(default_snapshot_config());
        snapshot_capture(&mut snap, &make_source());
        let mut dest = HashMap::new();
        dest.insert("smile".into(), 0.0f32);
        snapshot_restore(&snap, &mut dest);
        assert!((dest["smile"] - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_diff_detects_changes() {
        let mut snap_a = new_expression_snapshot(default_snapshot_config());
        let mut snap_b = new_expression_snapshot(default_snapshot_config());
        let src_a: HashMap<String, f32> = [("smile".to_string(), 0.5)].into();
        let src_b: HashMap<String, f32> = [("smile".to_string(), 0.9)].into();
        snapshot_capture(&mut snap_a, &src_a);
        snapshot_capture(&mut snap_b, &src_b);
        let diff = snapshot_diff(&snap_a, &snap_b, 0.01);
        assert!(diff.contains_key("smile"));
        assert!((diff["smile"] - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_diff_threshold_filters() {
        let mut snap_a = new_expression_snapshot(default_snapshot_config());
        let mut snap_b = new_expression_snapshot(default_snapshot_config());
        let src_a: HashMap<String, f32> = [("smile".to_string(), 0.5)].into();
        let src_b: HashMap<String, f32> = [("smile".to_string(), 0.501)].into();
        snapshot_capture(&mut snap_a, &src_a);
        snapshot_capture(&mut snap_b, &src_b);
        let diff = snapshot_diff(&snap_a, &snap_b, 0.1);
        assert!(!diff.contains_key("smile"));
    }

    #[test]
    fn test_blend_midpoint() {
        let mut snap_a = new_expression_snapshot(default_snapshot_config());
        let mut snap_b = new_expression_snapshot(default_snapshot_config());
        let src_a: HashMap<String, f32> = [("k".to_string(), 0.0)].into();
        let src_b: HashMap<String, f32> = [("k".to_string(), 1.0)].into();
        snapshot_capture(&mut snap_a, &src_a);
        snapshot_capture(&mut snap_b, &src_b);
        let blended = snapshot_blend(&snap_a, &snap_b, 0.5);
        let w = snapshot_get_weight(&blended, "k").unwrap_or(0.0);
        assert!((w - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_clear() {
        let mut snap = new_expression_snapshot(default_snapshot_config());
        snapshot_capture(&mut snap, &make_source());
        assert!(snapshot_entry_count(&snap) > 0);
        snapshot_clear(&mut snap);
        assert_eq!(snapshot_entry_count(&snap), 0);
    }

    #[test]
    fn test_to_json_format() {
        let mut snap = new_expression_snapshot(default_snapshot_config());
        snapshot_capture(&mut snap, &make_source());
        let json = snapshot_to_json(&snap);
        assert!(json.contains("entry_count"));
        assert!(json.contains("weights"));
    }

    #[test]
    fn test_get_weight_missing_returns_none() {
        let snap = new_expression_snapshot(default_snapshot_config());
        assert!(snapshot_get_weight(&snap, "nonexistent").is_none());
    }
}
