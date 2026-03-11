#![allow(dead_code)]

use std::collections::HashMap;

/// A snapshot of all pose parameters at a point in time.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseSnapshot {
    params: HashMap<String, f32>,
}

/// Difference between two pose snapshots.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SnapshotDiff {
    pub diffs: HashMap<String, f32>,
}

/// Take a snapshot from a parameter map.
#[allow(dead_code)]
pub fn take_pose_snapshot(params: &HashMap<String, f32>) -> PoseSnapshot {
    PoseSnapshot { params: params.clone() }
}

/// Return the number of parameters in the snapshot.
#[allow(dead_code)]
pub fn snapshot_param_count(snap: &PoseSnapshot) -> usize {
    snap.params.len()
}

/// Compute the difference between two snapshots.
#[allow(dead_code)]
pub fn snapshot_diff(a: &PoseSnapshot, b: &PoseSnapshot) -> SnapshotDiff {
    let mut diffs = HashMap::new();
    let mut keys: Vec<&String> = a.params.keys().chain(b.params.keys()).collect();
    keys.sort_unstable();
    keys.dedup();
    for k in keys {
        let va = a.params.get(k).copied().unwrap_or(0.0);
        let vb = b.params.get(k).copied().unwrap_or(0.0);
        let d = vb - va;
        if d.abs() > 1e-9 {
            diffs.insert(k.clone(), d);
        }
    }
    SnapshotDiff { diffs }
}

/// Apply a snapshot to a mutable parameter map.
#[allow(dead_code)]
pub fn apply_snapshot(snap: &PoseSnapshot, dest: &mut HashMap<String, f32>) {
    for (k, &v) in &snap.params {
        dest.insert(k.clone(), v);
    }
}

/// Serialize snapshot to a JSON string.
#[allow(dead_code)]
pub fn snapshot_to_json(snap: &PoseSnapshot) -> String {
    let mut entries: Vec<String> = snap.params.iter()
        .map(|(k, v)| format!(r#""{}": {:.6}"#, k, v))
        .collect();
    entries.sort();
    format!("{{{}}}", entries.join(", "))
}

/// Stub: parse a JSON string into a snapshot.
#[allow(dead_code)]
pub fn snapshot_from_json_stub(json: &str) -> PoseSnapshot {
    let _ = json;
    PoseSnapshot { params: HashMap::new() }
}

/// Blend two snapshots at parameter t (0..=1).
#[allow(dead_code)]
pub fn snapshot_blend(a: &PoseSnapshot, b: &PoseSnapshot, t: f32) -> PoseSnapshot {
    let t = t.clamp(0.0, 1.0);
    let mut params = HashMap::new();
    let mut keys: Vec<&String> = a.params.keys().chain(b.params.keys()).collect();
    keys.sort_unstable();
    keys.dedup();
    for k in keys {
        let va = a.params.get(k).copied().unwrap_or(0.0);
        let vb = b.params.get(k).copied().unwrap_or(0.0);
        params.insert(k.clone(), va + t * (vb - va));
    }
    PoseSnapshot { params }
}

/// Check if the snapshot represents the identity (all zeros).
#[allow(dead_code)]
pub fn snapshot_is_identity(snap: &PoseSnapshot) -> bool {
    snap.params.values().all(|v| v.abs() < 1e-9)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_params() -> HashMap<String, f32> {
        let mut m = HashMap::new();
        m.insert("x".to_string(), 0.5);
        m.insert("y".to_string(), 0.3);
        m
    }

    #[test]
    fn test_take_pose_snapshot() {
        let snap = take_pose_snapshot(&sample_params());
        assert_eq!(snapshot_param_count(&snap), 2);
    }

    #[test]
    fn test_snapshot_param_count() {
        let snap = take_pose_snapshot(&HashMap::new());
        assert_eq!(snapshot_param_count(&snap), 0);
    }

    #[test]
    fn test_snapshot_diff() {
        let a = take_pose_snapshot(&[("x".to_string(), 0.0)].into());
        let b = take_pose_snapshot(&[("x".to_string(), 1.0)].into());
        let d = snapshot_diff(&a, &b);
        assert!((d.diffs["x"] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_snapshot() {
        let snap = take_pose_snapshot(&sample_params());
        let mut dest = HashMap::new();
        apply_snapshot(&snap, &mut dest);
        assert!((dest["x"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_snapshot_to_json() {
        let snap = take_pose_snapshot(&[("a".to_string(), 0.5)].into());
        let json = snapshot_to_json(&snap);
        assert!(json.contains("\"a\""));
    }

    #[test]
    fn test_snapshot_from_json_stub() {
        let snap = snapshot_from_json_stub("{}");
        assert_eq!(snapshot_param_count(&snap), 0);
    }

    #[test]
    fn test_snapshot_blend() {
        let a = take_pose_snapshot(&[("k".to_string(), 0.0)].into());
        let b = take_pose_snapshot(&[("k".to_string(), 1.0)].into());
        let blended = snapshot_blend(&a, &b, 0.5);
        assert!((blended.params["k"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_snapshot_is_identity() {
        let snap = take_pose_snapshot(&[("k".to_string(), 0.0)].into());
        assert!(snapshot_is_identity(&snap));
    }

    #[test]
    fn test_snapshot_is_not_identity() {
        let snap = take_pose_snapshot(&sample_params());
        assert!(!snapshot_is_identity(&snap));
    }

    #[test]
    fn test_diff_no_changes() {
        let a = take_pose_snapshot(&sample_params());
        let d = snapshot_diff(&a, &a);
        assert!(d.diffs.is_empty());
    }
}
