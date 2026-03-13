// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Multiresolution mesh editing levels.
//! Stores a stack of subdivision levels and displacement deltas for each.

/// A single level in a multiresolution mesh.
#[derive(Debug, Clone)]
pub struct MrLevel {
    pub positions: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub displacements: Vec<[f32; 3]>,
}

/// Container for all multiresolution levels.
#[derive(Debug, Clone)]
pub struct MultiresolutionMesh {
    pub levels: Vec<MrLevel>,
    pub base_level: usize,
}

impl MultiresolutionMesh {
    /// Create a new multiresolution mesh from a base mesh.
    pub fn new(positions: Vec<[f32; 3]>, indices: Vec<u32>) -> Self {
        let displacements = vec![[0.0_f32; 3]; positions.len()];
        let level0 = MrLevel {
            positions,
            indices,
            displacements,
        };
        Self {
            levels: vec![level0],
            base_level: 0,
        }
    }

    /// Return the number of levels stored.
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }
}

/// Push a new level by simple midpoint subdivision (stub).
pub fn push_level(mr: &mut MultiresolutionMesh) {
    let top = mr.levels.last().expect("at least one level");
    let new_positions = top.positions.clone();
    let new_indices = top.indices.clone();
    let displacements = vec![[0.0_f32; 3]; new_positions.len()];
    mr.levels.push(MrLevel {
        positions: new_positions,
        indices: new_indices,
        displacements,
    });
}

/// Pop the finest level (if more than one level exists).
pub fn pop_level(mr: &mut MultiresolutionMesh) -> bool {
    if mr.levels.len() > 1 {
        mr.levels.pop();
        true
    } else {
        false
    }
}

/// Apply a displacement to a vertex at the given level.
pub fn apply_displacement(
    mr: &mut MultiresolutionMesh,
    level: usize,
    vertex: usize,
    delta: [f32; 3],
) {
    if let Some(lvl) = mr.levels.get_mut(level) {
        if let Some(d) = lvl.displacements.get_mut(vertex) {
            d[0] += delta[0];
            d[1] += delta[1];
            d[2] += delta[2];
        }
    }
}

/// Compute total displacement magnitude at a given level.
pub fn total_displacement_magnitude(mr: &MultiresolutionMesh, level: usize) -> f32 {
    if let Some(lvl) = mr.levels.get(level) {
        lvl.displacements
            .iter()
            .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
            .sum()
    } else {
        0.0
    }
}

/// Reset all displacements at the given level to zero.
pub fn reset_displacements(mr: &mut MultiresolutionMesh, level: usize) {
    if let Some(lvl) = mr.levels.get_mut(level) {
        for d in &mut lvl.displacements {
            *d = [0.0, 0.0, 0.0];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_mr() -> MultiresolutionMesh {
        let pos = vec![[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = vec![0u32, 1, 2];
        MultiresolutionMesh::new(pos, idx)
    }

    #[test]
    fn test_new_has_one_level() {
        /* newly created mesh starts with one level */
        let mr = sample_mr();
        assert_eq!(mr.level_count(), 1);
    }

    #[test]
    fn test_push_level_increments_count() {
        /* push should add a level */
        let mut mr = sample_mr();
        push_level(&mut mr);
        assert_eq!(mr.level_count(), 2);
    }

    #[test]
    fn test_pop_level_decrements_count() {
        /* pop on multi-level mesh returns true and removes level */
        let mut mr = sample_mr();
        push_level(&mut mr);
        assert!(pop_level(&mut mr));
        assert_eq!(mr.level_count(), 1);
    }

    #[test]
    fn test_pop_level_base_returns_false() {
        /* pop on single-level mesh should return false */
        let mut mr = sample_mr();
        assert!(!pop_level(&mut mr));
        assert_eq!(mr.level_count(), 1);
    }

    #[test]
    fn test_apply_displacement_accumulates() {
        /* displacement accumulates correctly */
        let mut mr = sample_mr();
        apply_displacement(&mut mr, 0, 0, [1.0, 0.0, 0.0]);
        apply_displacement(&mut mr, 0, 0, [0.0, 2.0, 0.0]);
        let d = mr.levels[0].displacements[0];
        assert!((d[0] - 1.0).abs() < 1e-6);
        assert!((d[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_total_displacement_magnitude_zero_initially() {
        /* newly created level has zero displacement magnitude */
        let mr = sample_mr();
        assert!((total_displacement_magnitude(&mr, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_total_displacement_magnitude_nonzero_after_apply() {
        /* after displacement magnitude becomes positive */
        let mut mr = sample_mr();
        apply_displacement(&mut mr, 0, 0, [3.0, 4.0, 0.0]);
        let mag = total_displacement_magnitude(&mr, 0);
        assert!(mag > 4.9);
    }

    #[test]
    fn test_reset_displacements() {
        /* reset returns displacements to zero */
        let mut mr = sample_mr();
        apply_displacement(&mut mr, 0, 0, [1.0, 1.0, 1.0]);
        reset_displacements(&mut mr, 0);
        assert!((total_displacement_magnitude(&mr, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_invalid_level_ignored() {
        /* operations on invalid level should not panic */
        let mut mr = sample_mr();
        apply_displacement(&mut mr, 99, 0, [1.0, 0.0, 0.0]);
        reset_displacements(&mut mr, 99);
        assert_eq!(total_displacement_magnitude(&mr, 99), 0.0);
    }
}
