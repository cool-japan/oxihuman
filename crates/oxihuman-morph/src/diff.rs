// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

use crate::params::ParamState;

/// The difference between two ParamState values.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParamDiff {
    pub d_height: f32,
    pub d_weight: f32,
    pub d_muscle: f32,
    pub d_age: f32,
    /// Extra keys that changed: (key, old_value, new_value).
    pub extra_changes: Vec<(String, f32, f32)>,
    /// Extra keys added in `b` (not present in `a`).
    pub extra_added: Vec<(String, f32)>,
    /// Extra keys removed in `b` (present in `a` but not `b`).
    pub extra_removed: Vec<(String, f32)>,
}

impl ParamDiff {
    /// Compute the diff: b - a for each field.
    pub fn compute(a: &ParamState, b: &ParamState) -> Self {
        let d_height = b.height - a.height;
        let d_weight = b.weight - a.weight;
        let d_muscle = b.muscle - a.muscle;
        let d_age = b.age - a.age;

        let mut extra_changes = Vec::new();
        let mut extra_added = Vec::new();
        let mut extra_removed = Vec::new();

        // Keys in b
        for (key, &bval) in &b.extra {
            if let Some(&aval) = a.extra.get(key) {
                if (bval - aval).abs() > 0.0 {
                    extra_changes.push((key.clone(), aval, bval));
                }
            } else {
                extra_added.push((key.clone(), bval));
            }
        }

        // Keys only in a (removed in b)
        for (key, &aval) in &a.extra {
            if !b.extra.contains_key(key) {
                extra_removed.push((key.clone(), aval));
            }
        }

        ParamDiff {
            d_height,
            d_weight,
            d_muscle,
            d_age,
            extra_changes,
            extra_added,
            extra_removed,
        }
    }

    /// True if all diffs are (near) zero.
    pub fn is_zero(&self, tolerance: f32) -> bool {
        self.d_height.abs() <= tolerance
            && self.d_weight.abs() <= tolerance
            && self.d_muscle.abs() <= tolerance
            && self.d_age.abs() <= tolerance
            && self
                .extra_changes
                .iter()
                .all(|(_, a, b)| (b - a).abs() <= tolerance)
            && self.extra_added.is_empty()
            && self.extra_removed.is_empty()
    }

    /// L2 norm of the numeric diffs (height, weight, muscle, age).
    pub fn magnitude(&self) -> f32 {
        (self.d_height.powi(2) + self.d_weight.powi(2) + self.d_muscle.powi(2) + self.d_age.powi(2))
            .sqrt()
    }

    /// Apply the diff to a ParamState: a + diff = b.
    pub fn apply(&self, a: &ParamState) -> ParamState {
        let mut result = a.clone();
        result.height += self.d_height;
        result.weight += self.d_weight;
        result.muscle += self.d_muscle;
        result.age += self.d_age;

        for (key, _old, new) in &self.extra_changes {
            result.extra.insert(key.clone(), *new);
        }
        for (key, val) in &self.extra_added {
            result.extra.insert(key.clone(), *val);
        }
        for (key, _) in &self.extra_removed {
            result.extra.remove(key);
        }

        result
    }

    /// Scale the diff by a factor.
    pub fn scaled(&self, factor: f32) -> ParamDiff {
        ParamDiff {
            d_height: self.d_height * factor,
            d_weight: self.d_weight * factor,
            d_muscle: self.d_muscle * factor,
            d_age: self.d_age * factor,
            extra_changes: self
                .extra_changes
                .iter()
                .map(|(k, a, b)| {
                    let mid = a + (b - a) * factor;
                    (k.clone(), *a, mid)
                })
                .collect(),
            extra_added: self
                .extra_added
                .iter()
                .map(|(k, v)| (k.clone(), v * factor))
                .collect(),
            extra_removed: self.extra_removed.clone(),
        }
    }

    /// Human-readable description of what changed.
    pub fn describe(&self, threshold: f32) -> String {
        let mut changes = Vec::new();
        if self.d_height.abs() > threshold {
            changes.push(format!("height {:+.3}", self.d_height));
        }
        if self.d_weight.abs() > threshold {
            changes.push(format!("weight {:+.3}", self.d_weight));
        }
        if self.d_muscle.abs() > threshold {
            changes.push(format!("muscle {:+.3}", self.d_muscle));
        }
        if self.d_age.abs() > threshold {
            changes.push(format!("age {:+.3}", self.d_age));
        }
        if changes.is_empty() {
            "no significant changes".into()
        } else {
            changes.join(", ")
        }
    }
}

/// Statistics about vertex displacement between two position buffers.
#[derive(Debug, Clone)]
pub struct MeshDiffStats {
    pub vertex_count: usize,
    /// Number of vertices that moved more than `threshold`.
    pub changed_count: usize,
    pub max_displacement: f32,
    pub avg_displacement: f32,
    pub rms_displacement: f32,
    /// Index of the vertex with maximum displacement.
    pub max_vertex_idx: usize,
}

impl MeshDiffStats {
    /// Compute stats comparing two position buffers (must be same length).
    pub fn compute(a: &[[f32; 3]], b: &[[f32; 3]], threshold: f32) -> Self {
        let n = a.len().min(b.len());
        if n == 0 {
            return MeshDiffStats {
                vertex_count: 0,
                changed_count: 0,
                max_displacement: 0.0,
                avg_displacement: 0.0,
                rms_displacement: 0.0,
                max_vertex_idx: 0,
            };
        }

        let displacements: Vec<f32> = (0..n)
            .map(|i| {
                let dx = b[i][0] - a[i][0];
                let dy = b[i][1] - a[i][1];
                let dz = b[i][2] - a[i][2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .collect();

        let changed_count = displacements.iter().filter(|&&d| d > threshold).count();
        let max_displacement = displacements.iter().cloned().fold(0.0f32, f32::max);
        let avg = displacements.iter().sum::<f32>() / n as f32;
        let rms = (displacements.iter().map(|d| d * d).sum::<f32>() / n as f32).sqrt();
        let max_idx = displacements
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0);

        MeshDiffStats {
            vertex_count: n,
            changed_count,
            max_displacement,
            avg_displacement: avg,
            rms_displacement: rms,
            max_vertex_idx: max_idx,
        }
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "vertices: {}, changed: {}, max_disp: {:.4}, avg_disp: {:.4}, rms_disp: {:.4}, max_vertex_idx: {}",
            self.vertex_count,
            self.changed_count,
            self.max_displacement,
            self.avg_displacement,
            self.rms_displacement,
            self.max_vertex_idx,
        )
    }
}

/// Per-vertex displacement magnitudes between two position buffers.
#[allow(dead_code)]
pub fn vertex_displacements(a: &[[f32; 3]], b: &[[f32; 3]]) -> Vec<f32> {
    let n = a.len().min(b.len());
    (0..n)
        .map(|i| {
            let dx = b[i][0] - a[i][0];
            let dy = b[i][1] - a[i][1];
            let dz = b[i][2] - a[i][2];
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .collect()
}

/// Find the top-N most displaced vertices (by displacement magnitude).
/// Returns (vertex_index, displacement) sorted descending.
#[allow(dead_code)]
pub fn top_displaced_vertices(a: &[[f32; 3]], b: &[[f32; 3]], n: usize) -> Vec<(usize, f32)> {
    let mut displacements: Vec<(usize, f32)> =
        vertex_displacements(a, b).into_iter().enumerate().collect();
    displacements.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal));
    displacements.truncate(n);
    displacements
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(h: f32, w: f32, m: f32, a: f32) -> ParamState {
        ParamState {
            height: h,
            weight: w,
            muscle: m,
            age: a,
            extra: Default::default(),
        }
    }

    #[test]
    fn diff_compute_height() {
        let diff = ParamDiff::compute(&p(0.3, 0.5, 0.5, 0.5), &p(0.7, 0.5, 0.5, 0.5));
        assert!((diff.d_height - 0.4).abs() < 1e-5);
    }

    #[test]
    fn diff_is_zero_for_identical() {
        let a = p(0.5, 0.5, 0.5, 0.5);
        let diff = ParamDiff::compute(&a, &a);
        assert!(diff.is_zero(1e-6));
    }

    #[test]
    fn diff_magnitude_correct() {
        let diff = ParamDiff {
            d_height: 3.0,
            d_weight: 4.0,
            d_muscle: 0.0,
            d_age: 0.0,
            extra_changes: vec![],
            extra_added: vec![],
            extra_removed: vec![],
        };
        assert!((diff.magnitude() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn diff_apply_roundtrip() {
        let a = p(0.3, 0.5, 0.5, 0.5);
        let b = p(0.7, 0.5, 0.5, 0.5);
        let diff = ParamDiff::compute(&a, &b);
        let result = diff.apply(&a);
        assert!((result.height - b.height).abs() < 1e-5);
        assert!((result.weight - b.weight).abs() < 1e-5);
        assert!((result.muscle - b.muscle).abs() < 1e-5);
        assert!((result.age - b.age).abs() < 1e-5);
    }

    #[test]
    fn diff_scaled_halves() {
        let a = p(0.2, 0.5, 0.5, 0.5);
        let b = p(0.8, 0.5, 0.5, 0.5);
        let diff = ParamDiff::compute(&a, &b);
        let scaled = diff.scaled(0.5);
        assert!((scaled.d_height - diff.d_height * 0.5).abs() < 1e-5);
    }

    #[test]
    fn diff_describe_nonempty() {
        let a = p(0.2, 0.5, 0.5, 0.5);
        let b = p(0.8, 0.5, 0.5, 0.5);
        let diff = ParamDiff::compute(&a, &b);
        let desc = diff.describe(0.01);
        assert!(!desc.is_empty());
        assert_ne!(desc, "no significant changes");
    }

    #[test]
    fn mesh_diff_zero_for_same() {
        let positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let stats = MeshDiffStats::compute(&positions, &positions, 1e-6);
        assert_eq!(stats.max_displacement, 0.0);
    }

    #[test]
    fn mesh_diff_detects_change() {
        let a: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let mut b = a.clone();
        b[1] = [2.0, 0.0, 0.0]; // move vertex 1 by 1 unit
        let stats = MeshDiffStats::compute(&a, &b, 0.5);
        assert_eq!(stats.changed_count, 1);
    }

    #[test]
    fn vertex_displacements_length() {
        let a: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let b: Vec<[f32; 3]> = vec![[0.1, 0.0, 0.0], [1.0, 0.1, 0.0], [0.0, 1.0, 0.1]];
        let disps = vertex_displacements(&a, &b);
        assert_eq!(disps.len(), a.len());
    }

    #[test]
    fn top_displaced_sorted_desc() {
        let a: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let b: Vec<[f32; 3]> = vec![
            [1.0, 0.0, 0.0], // disp = 1.0
            [3.0, 0.0, 0.0], // disp = 3.0
            [2.0, 0.0, 0.0], // disp = 2.0
            [0.5, 0.0, 0.0], // disp = 0.5
        ];
        let top = top_displaced_vertices(&a, &b, 3);
        assert_eq!(top.len(), 3);
        // Should be sorted descending: 3.0, 2.0, 1.0
        assert!(top[0].1 >= top[1].1);
        assert!(top[1].1 >= top[2].1);
        // Top should be vertex 1 (disp=3.0)
        assert_eq!(top[0].0, 1);
    }
}
