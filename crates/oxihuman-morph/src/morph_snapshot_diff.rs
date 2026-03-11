#![allow(dead_code)]
//! Computes diffs between morph parameter snapshots.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MorphSnapshotDiff {
    changes: HashMap<String, (f32, f32)>,
}

#[allow(dead_code)]
pub fn compute_snapshot_diff(
    before: &HashMap<String, f32>,
    after: &HashMap<String, f32>,
) -> MorphSnapshotDiff {
    let mut changes = HashMap::new();
    for (k, &va) in before {
        let vb = after.get(k).copied().unwrap_or(0.0);
        if (va - vb).abs() > 1e-9 {
            changes.insert(k.clone(), (va, vb));
        }
    }
    for (k, &vb) in after {
        if !before.contains_key(k) && vb.abs() > 1e-9 {
            changes.insert(k.clone(), (0.0, vb));
        }
    }
    MorphSnapshotDiff { changes }
}

#[allow(dead_code)]
pub fn diff_param_count(d: &MorphSnapshotDiff) -> usize {
    d.changes.len()
}

#[allow(dead_code)]
pub fn diff_max_change(d: &MorphSnapshotDiff) -> f32 {
    d.changes
        .values()
        .map(|(a, b)| (b - a).abs())
        .fold(0.0_f32, f32::max)
}

#[allow(dead_code)]
pub fn diff_min_change(d: &MorphSnapshotDiff) -> f32 {
    if d.changes.is_empty() {
        return 0.0;
    }
    d.changes
        .values()
        .map(|(a, b)| (b - a).abs())
        .fold(f32::INFINITY, f32::min)
}

#[allow(dead_code)]
pub fn diff_average_change(d: &MorphSnapshotDiff) -> f32 {
    if d.changes.is_empty() {
        return 0.0;
    }
    let total: f32 = d.changes.values().map(|(a, b)| (b - a).abs()).sum();
    total / d.changes.len() as f32
}

#[allow(dead_code)]
pub fn diff_changed_params(d: &MorphSnapshotDiff) -> Vec<&str> {
    d.changes.keys().map(|k| k.as_str()).collect()
}

#[allow(dead_code)]
pub fn diff_to_json(d: &MorphSnapshotDiff) -> String {
    let entries: Vec<String> = d
        .changes
        .iter()
        .map(|(k, (a, b))| format!("\"{k}\":[{a},{b}]"))
        .collect();
    format!("{{{}}}", entries.join(","))
}

#[allow(dead_code)]
pub fn diff_is_zero(d: &MorphSnapshotDiff) -> bool {
    d.changes.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map(pairs: &[(&str, f32)]) -> HashMap<String, f32> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    #[test]
    fn test_compute_snapshot_diff_empty() {
        let d = compute_snapshot_diff(&HashMap::new(), &HashMap::new());
        assert!(diff_is_zero(&d));
    }

    #[test]
    fn test_compute_snapshot_diff() {
        let a = make_map(&[("x", 0.0)]);
        let b = make_map(&[("x", 1.0)]);
        let d = compute_snapshot_diff(&a, &b);
        assert_eq!(diff_param_count(&d), 1);
    }

    #[test]
    fn test_diff_max_change() {
        let a = make_map(&[("x", 0.0), ("y", 0.0)]);
        let b = make_map(&[("x", 0.5), ("y", 1.0)]);
        let d = compute_snapshot_diff(&a, &b);
        assert!((diff_max_change(&d) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_diff_min_change() {
        let a = make_map(&[("x", 0.0), ("y", 0.0)]);
        let b = make_map(&[("x", 0.3), ("y", 0.8)]);
        let d = compute_snapshot_diff(&a, &b);
        assert!((diff_min_change(&d) - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_diff_average_change() {
        let a = make_map(&[("x", 0.0), ("y", 0.0)]);
        let b = make_map(&[("x", 0.4), ("y", 0.6)]);
        let d = compute_snapshot_diff(&a, &b);
        assert!((diff_average_change(&d) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_diff_changed_params() {
        let a = make_map(&[("x", 0.0)]);
        let b = make_map(&[("x", 1.0)]);
        let d = compute_snapshot_diff(&a, &b);
        assert_eq!(diff_changed_params(&d).len(), 1);
    }

    #[test]
    fn test_diff_to_json() {
        let a = make_map(&[("x", 0.0)]);
        let b = make_map(&[("x", 1.0)]);
        let d = compute_snapshot_diff(&a, &b);
        let json = diff_to_json(&d);
        assert!(json.contains("\"x\""));
    }

    #[test]
    fn test_diff_is_zero_no_changes() {
        let a = make_map(&[("x", 0.5)]);
        let b = make_map(&[("x", 0.5)]);
        let d = compute_snapshot_diff(&a, &b);
        assert!(diff_is_zero(&d));
    }

    #[test]
    fn test_new_param_in_after() {
        let a = HashMap::new();
        let b = make_map(&[("new", 0.5)]);
        let d = compute_snapshot_diff(&a, &b);
        assert_eq!(diff_param_count(&d), 1);
    }

    #[test]
    fn test_diff_min_change_empty() {
        let d = compute_snapshot_diff(&HashMap::new(), &HashMap::new());
        assert!(diff_min_change(&d).abs() < 1e-6);
    }
}
