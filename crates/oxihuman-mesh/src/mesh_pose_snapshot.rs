// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Single-frame mesh pose snapshot — captures and restores a mesh pose at one moment in time.

/// A pose snapshot stores per-vertex positions and optionally normals at a single frame.
#[derive(Debug, Default, Clone)]
pub struct PoseSnapshot {
    pub label: String,
    pub positions: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub timestamp: f64,
}

impl PoseSnapshot {
    /// Creates a new pose snapshot.
    pub fn new(label: impl Into<String>, positions: Vec<[f32; 3]>, timestamp: f64) -> Self {
        Self {
            label: label.into(),
            positions,
            normals: None,
            timestamp,
        }
    }

    /// Attaches normal data to the snapshot.
    pub fn with_normals(mut self, normals: Vec<[f32; 3]>) -> Self {
        self.normals = Some(normals);
        self
    }

    /// Returns the number of vertices in the snapshot.
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Returns true if this snapshot has normal data.
    pub fn has_normals(&self) -> bool {
        self.normals.is_some()
    }
}

/// A collection of named pose snapshots.
#[derive(Debug, Default, Clone)]
pub struct PoseSnapshotLibrary {
    pub snapshots: Vec<PoseSnapshot>,
}

impl PoseSnapshotLibrary {
    /// Creates an empty library.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a snapshot.
    pub fn add(&mut self, snapshot: PoseSnapshot) {
        self.snapshots.push(snapshot);
    }

    /// Finds a snapshot by label.
    pub fn find(&self, label: &str) -> Option<&PoseSnapshot> {
        self.snapshots.iter().find(|s| s.label == label)
    }

    /// Returns the number of snapshots.
    pub fn count(&self) -> usize {
        self.snapshots.len()
    }
}

/// Computes the bounding box of a snapshot.
pub fn snapshot_aabb(snap: &PoseSnapshot) -> ([f32; 3], [f32; 3]) {
    if snap.positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = snap.positions[0];
    let mut max = snap.positions[0];
    for p in &snap.positions {
        for k in 0..3 {
            min[k] = min[k].min(p[k]);
            max[k] = max[k].max(p[k]);
        }
    }
    (min, max)
}

/// Computes the displacement between two snapshots (per-vertex L2 norm, summed).
pub fn snapshot_displacement(a: &PoseSnapshot, b: &PoseSnapshot) -> f32 {
    a.positions
        .iter()
        .zip(b.positions.iter())
        .map(|(pa, pb)| {
            let dx = pa[0] - pb[0];
            let dy = pa[1] - pb[1];
            let dz = pa[2] - pb[2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

/// Applies a uniform scale to all positions in a snapshot.
pub fn scale_snapshot(snap: &mut PoseSnapshot, scale: f32) {
    for p in snap.positions.iter_mut() {
        p[0] *= scale;
        p[1] *= scale;
        p[2] *= scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snap(n: usize) -> PoseSnapshot {
        PoseSnapshot::new("test", vec![[1.0, 0.0, 0.0]; n], 0.0)
    }

    #[test]
    fn test_new_snapshot() {
        /* Snapshot should store vertex count correctly */
        assert_eq!(make_snap(5).vertex_count(), 5);
    }

    #[test]
    fn test_has_normals_false() {
        /* New snapshot without normals should return false */
        assert!(!make_snap(3).has_normals());
    }

    #[test]
    fn test_has_normals_true() {
        /* Snapshot with normals should return true */
        let snap = make_snap(2).with_normals(vec![[0.0, 1.0, 0.0]; 2]);
        assert!(snap.has_normals());
    }

    #[test]
    fn test_library_add_and_count() {
        /* Adding snapshots should increase count */
        let mut lib = PoseSnapshotLibrary::new();
        lib.add(make_snap(3));
        assert_eq!(lib.count(), 1);
    }

    #[test]
    fn test_library_find_existing() {
        /* find should return the snapshot with matching label */
        let mut lib = PoseSnapshotLibrary::new();
        lib.add(make_snap(2));
        assert!(lib.find("test").is_some());
    }

    #[test]
    fn test_library_find_missing() {
        /* find should return None for missing label */
        let lib = PoseSnapshotLibrary::new();
        assert!(lib.find("nope").is_none());
    }

    #[test]
    fn test_snapshot_aabb_empty() {
        /* Empty snapshot should return zero AABB */
        let snap = PoseSnapshot::new("empty", vec![], 0.0);
        let (min, max) = snapshot_aabb(&snap);
        assert_eq!(min, [0.0; 3]);
        assert_eq!(max, [0.0; 3]);
    }

    #[test]
    fn test_snapshot_aabb_single() {
        /* Single point AABB should be that point */
        let snap = PoseSnapshot::new("s", vec![[2.0, 3.0, 4.0]], 0.0);
        let (min, max) = snapshot_aabb(&snap);
        assert_eq!(min, [2.0, 3.0, 4.0]);
        assert_eq!(max, [2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_snapshot_displacement_same() {
        /* Same snapshots should have zero displacement */
        let a = make_snap(3);
        let b = make_snap(3);
        assert!((snapshot_displacement(&a, &b)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_scale_snapshot() {
        /* Scaling by 2 should double all positions */
        let mut snap = make_snap(2);
        scale_snapshot(&mut snap, 2.0);
        assert!((snap.positions[0][0] - 2.0).abs() < f32::EPSILON);
    }
}
