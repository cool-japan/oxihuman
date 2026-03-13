#![allow(dead_code)]

//! Pose snapshot capture and restore.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseSnapshotV2 {
    pub name: String,
    pub joint_rotations: HashMap<String, [f32; 4]>,
    pub joint_translations: HashMap<String, [f32; 3]>,
    pub timestamp: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PoseSnapshotStore {
    pub snapshots: Vec<PoseSnapshotV2>,
    pub max_snapshots: usize,
}

#[allow(dead_code)]
pub fn new_pose_snapshot_store(max_snapshots: usize) -> PoseSnapshotStore {
    PoseSnapshotStore {
        snapshots: Vec::new(),
        max_snapshots,
    }
}

#[allow(dead_code)]
pub fn pss_capture(
    store: &mut PoseSnapshotStore,
    name: &str,
    rotations: HashMap<String, [f32; 4]>,
    translations: HashMap<String, [f32; 3]>,
    timestamp: f32,
) {
    if store.snapshots.len() >= store.max_snapshots {
        store.snapshots.remove(0);
    }
    store.snapshots.push(PoseSnapshotV2 {
        name: name.to_string(),
        joint_rotations: rotations,
        joint_translations: translations,
        timestamp,
    });
}

#[allow(dead_code)]
pub fn pss_get<'a>(store: &'a PoseSnapshotStore, name: &str) -> Option<&'a PoseSnapshotV2> {
    store.snapshots.iter().rev().find(|s| s.name == name)
}

#[allow(dead_code)]
pub fn pss_snapshot_count(store: &PoseSnapshotStore) -> usize {
    store.snapshots.len()
}

#[allow(dead_code)]
pub fn pss_remove(store: &mut PoseSnapshotStore, name: &str) {
    store.snapshots.retain(|s| s.name != name);
}

#[allow(dead_code)]
pub fn pss_clear(store: &mut PoseSnapshotStore) {
    store.snapshots.clear();
}

#[allow(dead_code)]
pub fn pss_latest(store: &PoseSnapshotStore) -> Option<&PoseSnapshotV2> {
    store.snapshots.last()
}

#[allow(dead_code)]
pub fn pss_joint_count(snapshot: &PoseSnapshotV2) -> usize {
    snapshot.joint_rotations.len()
}

#[allow(dead_code)]
pub fn pss_to_json(store: &PoseSnapshotStore) -> String {
    format!(
        "{{\"snapshot_count\":{},\"max_snapshots\":{}}}",
        store.snapshots.len(),
        store.max_snapshots
    )
}

#[allow(dead_code)]
pub fn pss_has(store: &PoseSnapshotStore, name: &str) -> bool {
    store.snapshots.iter().any(|s| s.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(store: &mut PoseSnapshotStore, name: &str) {
        let mut rots = HashMap::new();
        rots.insert("hip".to_string(), [0.0_f32, 0.0, 0.0, 1.0]);
        pss_capture(store, name, rots, HashMap::new(), 0.0);
    }

    #[test]
    fn test_new_store() {
        let store = new_pose_snapshot_store(10);
        assert_eq!(pss_snapshot_count(&store), 0);
    }

    #[test]
    fn test_capture() {
        let mut store = new_pose_snapshot_store(10);
        make_snapshot(&mut store, "idle");
        assert_eq!(pss_snapshot_count(&store), 1);
    }

    #[test]
    fn test_get_snapshot() {
        let mut store = new_pose_snapshot_store(10);
        make_snapshot(&mut store, "walk");
        assert!(pss_get(&store, "walk").is_some());
    }

    #[test]
    fn test_get_nonexistent() {
        let store = new_pose_snapshot_store(10);
        assert!(pss_get(&store, "nonexistent").is_none());
    }

    #[test]
    fn test_remove() {
        let mut store = new_pose_snapshot_store(10);
        make_snapshot(&mut store, "run");
        pss_remove(&mut store, "run");
        assert!(!pss_has(&store, "run"));
    }

    #[test]
    fn test_clear() {
        let mut store = new_pose_snapshot_store(10);
        make_snapshot(&mut store, "a");
        make_snapshot(&mut store, "b");
        pss_clear(&mut store);
        assert_eq!(pss_snapshot_count(&store), 0);
    }

    #[test]
    fn test_eviction_at_capacity() {
        let mut store = new_pose_snapshot_store(2);
        make_snapshot(&mut store, "a");
        make_snapshot(&mut store, "b");
        make_snapshot(&mut store, "c");
        assert_eq!(pss_snapshot_count(&store), 2);
        assert!(!pss_has(&store, "a"));
    }

    #[test]
    fn test_latest() {
        let mut store = new_pose_snapshot_store(5);
        make_snapshot(&mut store, "first");
        make_snapshot(&mut store, "last");
        assert!(pss_latest(&store).is_some_and(|s| s.name == "last"));
    }

    #[test]
    fn test_joint_count() {
        let mut store = new_pose_snapshot_store(5);
        make_snapshot(&mut store, "test");
        let snap = pss_get(&store, "test").expect("should succeed");
        assert_eq!(pss_joint_count(snap), 1);
    }

    #[test]
    fn test_to_json() {
        let store = new_pose_snapshot_store(5);
        let json = pss_to_json(&store);
        assert!(json.contains("max_snapshots"));
    }
}
