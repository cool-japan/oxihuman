// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Animation blend buffer — blends multiple animations by weighted mixing.

/// A single animation contribution with a weight.
#[derive(Debug, Clone)]
pub struct AnimContribution {
    pub name: String,
    pub weight: f32,
    pub positions: Vec<[f32; 3]>,
}

/// An animation blend buffer holding weighted contributions.
#[derive(Debug, Default, Clone)]
pub struct BlendAnimBuffer {
    pub contributions: Vec<AnimContribution>,
    pub vertex_count: usize,
}

impl BlendAnimBuffer {
    /// Creates a new blend buffer for the given vertex count.
    pub fn new(vertex_count: usize) -> Self {
        Self {
            vertex_count,
            contributions: Vec::new(),
        }
    }

    /// Adds a contribution.
    pub fn add_contribution(&mut self, contrib: AnimContribution) {
        self.contributions.push(contrib);
    }

    /// Returns the sum of all contribution weights.
    pub fn total_weight(&self) -> f32 {
        self.contributions.iter().map(|c| c.weight).sum()
    }

    /// Returns the number of contributions.
    pub fn contribution_count(&self) -> usize {
        self.contributions.len()
    }

    /// Clears all contributions.
    pub fn clear(&mut self) {
        self.contributions.clear();
    }
}

/// Evaluates the blended vertex positions (weighted average).
#[allow(clippy::needless_range_loop)]
pub fn evaluate_blend(buf: &BlendAnimBuffer) -> Vec<[f32; 3]> {
    let n = buf.vertex_count;
    let total = buf.total_weight();
    if total < f32::EPSILON || n == 0 {
        return vec![[0.0; 3]; n];
    }
    let mut result = vec![[0.0f32; 3]; n];
    for contrib in &buf.contributions {
        let w = contrib.weight / total;
        for i in 0..n.min(contrib.positions.len()) {
            result[i][0] += contrib.positions[i][0] * w;
            result[i][1] += contrib.positions[i][1] * w;
            result[i][2] += contrib.positions[i][2] * w;
        }
    }
    result
}

/// Normalizes contribution weights so they sum to 1.
pub fn normalize_weights(buf: &mut BlendAnimBuffer) {
    let total = buf.total_weight();
    if total < f32::EPSILON {
        return;
    }
    for c in buf.contributions.iter_mut() {
        c.weight /= total;
    }
}

/// Removes contributions with zero or negative weight.
pub fn prune_zero_weights(buf: &mut BlendAnimBuffer) {
    buf.contributions.retain(|c| c.weight > 0.0);
}

/// Returns the dominant contribution (highest weight).
pub fn dominant_contribution(buf: &BlendAnimBuffer) -> Option<&AnimContribution> {
    buf.contributions.iter().max_by(|a, b| {
        a.weight
            .partial_cmp(&b.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_contrib(name: &str, weight: f32, n: usize) -> AnimContribution {
        AnimContribution {
            name: name.to_string(),
            weight,
            positions: vec![[1.0, 0.0, 0.0]; n],
        }
    }

    #[test]
    fn test_new_blend_buffer_empty() {
        /* New buffer should have no contributions */
        assert_eq!(BlendAnimBuffer::new(10).contribution_count(), 0);
    }

    #[test]
    fn test_total_weight_empty() {
        /* Empty buffer should have zero total weight */
        assert_eq!(BlendAnimBuffer::new(10).total_weight(), 0.0);
    }

    #[test]
    fn test_add_contribution() {
        /* Adding a contribution should increase count */
        let mut buf = BlendAnimBuffer::new(5);
        buf.add_contribution(make_contrib("walk", 1.0, 5));
        assert_eq!(buf.contribution_count(), 1);
    }

    #[test]
    fn test_total_weight_accumulates() {
        /* Total weight should sum all contributions */
        let mut buf = BlendAnimBuffer::new(5);
        buf.add_contribution(make_contrib("a", 0.3, 5));
        buf.add_contribution(make_contrib("b", 0.7, 5));
        assert!((buf.total_weight() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_evaluate_blend_single() {
        /* Single contribution with weight 1 should return its positions */
        let mut buf = BlendAnimBuffer::new(1);
        buf.add_contribution(make_contrib("a", 1.0, 1));
        let r = evaluate_blend(&buf);
        assert!((r[0][0] - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_normalize_weights() {
        /* After normalization total weight should be 1 */
        let mut buf = BlendAnimBuffer::new(2);
        buf.add_contribution(make_contrib("a", 2.0, 2));
        buf.add_contribution(make_contrib("b", 2.0, 2));
        normalize_weights(&mut buf);
        assert!((buf.total_weight() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_prune_zero_weights() {
        /* Contributions with weight 0 should be removed */
        let mut buf = BlendAnimBuffer::new(2);
        buf.add_contribution(make_contrib("zero", 0.0, 2));
        buf.add_contribution(make_contrib("positive", 0.5, 2));
        prune_zero_weights(&mut buf);
        assert_eq!(buf.contribution_count(), 1);
    }

    #[test]
    fn test_dominant_contribution_none_empty() {
        /* Empty buffer should return None */
        assert!(dominant_contribution(&BlendAnimBuffer::new(5)).is_none());
    }

    #[test]
    fn test_dominant_contribution_found() {
        /* Contribution with highest weight should be returned */
        let mut buf = BlendAnimBuffer::new(2);
        buf.add_contribution(make_contrib("a", 0.2, 2));
        buf.add_contribution(make_contrib("b", 0.8, 2));
        assert_eq!(dominant_contribution(&buf).expect("should succeed").name, "b");
    }

    #[test]
    fn test_clear() {
        /* Clear should remove all contributions */
        let mut buf = BlendAnimBuffer::new(2);
        buf.add_contribution(make_contrib("a", 1.0, 2));
        buf.clear();
        assert_eq!(buf.contribution_count(), 0);
    }
}
