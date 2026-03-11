// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

pub struct SnapshotManager {
    pub snapshots: Vec<Vec<f32>>,
    pub max_snapshots: usize,
}

impl SnapshotManager {
    pub fn new(max: usize) -> Self {
        SnapshotManager {
            snapshots: Vec::new(),
            max_snapshots: max,
        }
    }
}

pub fn new_snapshot_manager(max: usize) -> SnapshotManager {
    SnapshotManager::new(max)
}

pub fn snapshot_save(m: &mut SnapshotManager, data: &[f32]) {
    if m.snapshots.len() >= m.max_snapshots && m.max_snapshots > 0 {
        m.snapshots.remove(0);
    }
    m.snapshots.push(data.to_vec());
}

pub fn snapshot_count(m: &SnapshotManager) -> usize {
    m.snapshots.len()
}

pub fn snapshot_get(m: &SnapshotManager, idx: usize) -> Option<&[f32]> {
    m.snapshots.get(idx).map(|v| v.as_slice())
}

pub fn snapshot_latest(m: &SnapshotManager) -> Option<&[f32]> {
    m.snapshots.last().map(|v| v.as_slice())
}

pub fn snapshot_clear(m: &mut SnapshotManager) {
    m.snapshots.clear();
}

pub fn snapshot_is_full(m: &SnapshotManager) -> bool {
    m.max_snapshots > 0 && m.snapshots.len() >= m.max_snapshots
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        /* new manager has zero snapshots */
        let m = new_snapshot_manager(5);
        assert_eq!(snapshot_count(&m), 0);
    }

    #[test]
    fn test_save_and_count() {
        /* saving increments count */
        let mut m = new_snapshot_manager(5);
        snapshot_save(&mut m, &[1.0, 2.0]);
        assert_eq!(snapshot_count(&m), 1);
    }

    #[test]
    fn test_get() {
        /* get retrieves correct snapshot */
        let mut m = new_snapshot_manager(5);
        snapshot_save(&mut m, &[3.0, 4.0]);
        let s = snapshot_get(&m, 0).unwrap();
        assert_eq!(s, &[3.0f32, 4.0]);
    }

    #[test]
    fn test_latest() {
        /* latest returns most recent snapshot */
        let mut m = new_snapshot_manager(5);
        snapshot_save(&mut m, &[1.0]);
        snapshot_save(&mut m, &[9.0]);
        assert_eq!(snapshot_latest(&m).unwrap(), &[9.0f32]);
    }

    #[test]
    fn test_eviction() {
        /* oldest snapshot evicted when full */
        let mut m = new_snapshot_manager(2);
        snapshot_save(&mut m, &[1.0]);
        snapshot_save(&mut m, &[2.0]);
        snapshot_save(&mut m, &[3.0]);
        assert_eq!(snapshot_count(&m), 2);
        assert_eq!(snapshot_get(&m, 0).unwrap(), &[2.0f32]);
    }

    #[test]
    fn test_is_full() {
        /* is_full returns true at max capacity */
        let mut m = new_snapshot_manager(2);
        snapshot_save(&mut m, &[1.0]);
        snapshot_save(&mut m, &[2.0]);
        assert!(snapshot_is_full(&m));
    }

    #[test]
    fn test_clear() {
        /* clear removes all snapshots */
        let mut m = new_snapshot_manager(5);
        snapshot_save(&mut m, &[1.0]);
        snapshot_clear(&mut m);
        assert_eq!(snapshot_count(&m), 0);
    }

    #[test]
    fn test_get_out_of_bounds() {
        /* get beyond count returns None */
        let m = new_snapshot_manager(5);
        assert!(snapshot_get(&m, 99).is_none());
    }
}
