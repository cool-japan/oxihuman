// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Progressive mesh — vertex split/collapse operations for continuous LOD.

/// A vertex split record: collapses vertex `v1` into `v0`.
#[derive(Debug, Clone, Copy)]
pub struct VertexSplit {
    pub v0: u32,
    pub v1: u32,
    pub error: f32,
}

/// A progressive mesh storing the base mesh and a list of vertex splits.
#[derive(Debug, Default, Clone)]
pub struct ProgressiveMesh {
    pub base_vertex_count: usize,
    pub base_triangle_count: usize,
    pub splits: Vec<VertexSplit>,
    pub current_level: usize,
}

impl ProgressiveMesh {
    /// Creates a new progressive mesh.
    pub fn new(base_vertex_count: usize, base_triangle_count: usize) -> Self {
        Self {
            base_vertex_count,
            base_triangle_count,
            splits: Vec::new(),
            current_level: 0,
        }
    }

    /// Adds a vertex split record.
    pub fn push_split(&mut self, split: VertexSplit) {
        self.splits.push(split);
    }

    /// Applies `n` splits (increases detail).
    pub fn refine(&mut self, n: usize) -> usize {
        let available = self.splits.len().saturating_sub(self.current_level);
        let to_apply = n.min(available);
        self.current_level += to_apply;
        to_apply
    }

    /// Collapses `n` splits (reduces detail).
    pub fn coarsen(&mut self, n: usize) -> usize {
        let to_remove = n.min(self.current_level);
        self.current_level = self.current_level.saturating_sub(to_remove);
        to_remove
    }

    /// Returns the current vertex count estimate.
    pub fn current_vertex_count(&self) -> usize {
        self.base_vertex_count + self.current_level
    }

    /// Returns the maximum detail level available.
    pub fn max_level(&self) -> usize {
        self.splits.len()
    }
}

/// Computes a simplistic quadric error for a collapse.
pub fn quadric_error(p0: [f32; 3], p1: [f32; 3]) -> f32 {
    let dx = p0[0] - p1[0];
    let dy = p0[1] - p1[1];
    let dz = p0[2] - p1[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Sorts splits by ascending error (greedy collapse order).
pub fn sort_splits_by_error(splits: &mut [VertexSplit]) {
    splits.sort_by(|a, b| {
        a.error
            .partial_cmp(&b.error)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Returns the split with the smallest error, if any.
pub fn min_error_split(splits: &[VertexSplit]) -> Option<VertexSplit> {
    splits.iter().copied().min_by(|a, b| {
        a.error
            .partial_cmp(&b.error)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Filters splits whose error exceeds `threshold`.
pub fn filter_splits_by_threshold(splits: &[VertexSplit], threshold: f32) -> Vec<VertexSplit> {
    splits
        .iter()
        .copied()
        .filter(|s| s.error <= threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_progressive_mesh() {
        /* Base counts should be stored */
        let pm = ProgressiveMesh::new(100, 50);
        assert_eq!(pm.base_vertex_count, 100);
        assert_eq!(pm.base_triangle_count, 50);
    }

    #[test]
    fn test_push_split() {
        /* Pushing a split increases max_level */
        let mut pm = ProgressiveMesh::new(100, 50);
        pm.push_split(VertexSplit {
            v0: 0,
            v1: 1,
            error: 0.1,
        });
        assert_eq!(pm.max_level(), 1);
    }

    #[test]
    fn test_refine_increases_level() {
        /* Refine should move current_level up */
        let mut pm = ProgressiveMesh::new(100, 50);
        for i in 0..5 {
            pm.push_split(VertexSplit {
                v0: i,
                v1: i + 1,
                error: 0.01 * i as f32,
            });
        }
        pm.refine(3);
        assert_eq!(pm.current_level, 3);
    }

    #[test]
    fn test_coarsen_decreases_level() {
        /* Coarsen should reduce current_level */
        let mut pm = ProgressiveMesh::new(100, 50);
        for i in 0..5 {
            pm.push_split(VertexSplit {
                v0: i,
                v1: i + 1,
                error: 0.01,
            });
        }
        pm.refine(5);
        pm.coarsen(2);
        assert_eq!(pm.current_level, 3);
    }

    #[test]
    fn test_coarsen_clamps_at_zero() {
        /* Coarsen beyond zero should not underflow */
        let mut pm = ProgressiveMesh::new(100, 50);
        pm.coarsen(999);
        assert_eq!(pm.current_level, 0);
    }

    #[test]
    fn test_current_vertex_count() {
        /* Vertex count should be base + current_level */
        let mut pm = ProgressiveMesh::new(200, 100);
        for i in 0..4 {
            pm.push_split(VertexSplit {
                v0: i,
                v1: i + 10,
                error: 0.0,
            });
        }
        pm.refine(4);
        assert_eq!(pm.current_vertex_count(), 204);
    }

    #[test]
    fn test_quadric_error_same_point() {
        /* Same point should have zero error */
        let p = [1.0f32, 2.0, 3.0];
        assert_eq!(quadric_error(p, p), 0.0);
    }

    #[test]
    fn test_sort_splits_by_error() {
        /* After sort, errors should be ascending */
        let mut splits = vec![
            VertexSplit {
                v0: 0,
                v1: 1,
                error: 0.5,
            },
            VertexSplit {
                v0: 1,
                v1: 2,
                error: 0.1,
            },
        ];
        sort_splits_by_error(&mut splits);
        assert!(splits[0].error <= splits[1].error);
    }

    #[test]
    fn test_min_error_split_empty() {
        /* Empty slice should return None */
        assert!(min_error_split(&[]).is_none());
    }

    #[test]
    fn test_filter_splits_by_threshold() {
        /* Only splits below threshold should remain */
        let splits = vec![
            VertexSplit {
                v0: 0,
                v1: 1,
                error: 0.1,
            },
            VertexSplit {
                v0: 1,
                v1: 2,
                error: 0.9,
            },
        ];
        let filtered = filter_splits_by_threshold(&splits, 0.5);
        assert_eq!(filtered.len(), 1);
    }
}
