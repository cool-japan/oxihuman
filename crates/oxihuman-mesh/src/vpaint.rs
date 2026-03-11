// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

/// A named per-vertex weight map. Values are clamped to [0.0, 1.0].
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightMap {
    pub name: String,
    pub weights: Vec<f32>,
}

#[allow(dead_code)]
impl WeightMap {
    /// Create a map of `count` vertices all initialized to `initial_value`.
    pub fn new(name: impl Into<String>, count: usize, initial_value: f32) -> Self {
        let value = initial_value.clamp(0.0, 1.0);
        Self {
            name: name.into(),
            weights: vec![value; count],
        }
    }

    /// Create a map filled with zeros.
    pub fn zeros(name: impl Into<String>, count: usize) -> Self {
        Self::new(name, count, 0.0)
    }

    /// Create a map filled with ones.
    pub fn ones(name: impl Into<String>, count: usize) -> Self {
        Self::new(name, count, 1.0)
    }

    pub fn len(&self) -> usize {
        self.weights.len()
    }

    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }

    /// Paint a single vertex with value (clamped to 0..1).
    pub fn paint(&mut self, vertex_idx: usize, value: f32) {
        if let Some(w) = self.weights.get_mut(vertex_idx) {
            *w = value.clamp(0.0, 1.0);
        }
    }

    /// Paint all vertices within radius of `center` position (from positions slice).
    /// Uses linear falloff from center to radius edge.
    pub fn paint_sphere(
        &mut self,
        positions: &[[f32; 3]],
        center: [f32; 3],
        radius: f32,
        value: f32,
    ) {
        let clamped_value = value.clamp(0.0, 1.0);
        let radius_sq = radius * radius;

        for (i, pos) in positions.iter().enumerate() {
            if i >= self.weights.len() {
                break;
            }
            let dx = pos[0] - center[0];
            let dy = pos[1] - center[1];
            let dz = pos[2] - center[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;

            if dist_sq <= radius_sq {
                let dist = dist_sq.sqrt();
                let falloff = if radius > 0.0 {
                    1.0 - (dist / radius)
                } else {
                    1.0
                };
                let blended = (self.weights[i] + clamped_value * falloff).clamp(0.0, 1.0);
                self.weights[i] = blended;
            }
        }
    }

    /// Flood fill: set all vertices to a value.
    pub fn flood_fill(&mut self, value: f32) {
        let v = value.clamp(0.0, 1.0);
        for w in &mut self.weights {
            *w = v;
        }
    }

    /// Normalize: rescale weights to [0, 1] range.
    pub fn normalize(&mut self) {
        let min = self.weights.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = self
            .weights
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        let range = max - min;
        if range > 0.0 {
            for w in &mut self.weights {
                *w = (*w - min) / range;
            }
        }
    }

    /// Invert: weight = 1.0 - weight.
    pub fn invert(&mut self) {
        for w in &mut self.weights {
            *w = 1.0 - *w;
        }
    }

    /// Smooth: each weight becomes average of itself and neighbors.
    /// `adjacency[i]` = list of vertex indices adjacent to vertex i.
    pub fn smooth_once(&mut self, adjacency: &[Vec<usize>]) {
        let old = self.weights.clone();
        for (i, w) in self.weights.iter_mut().enumerate() {
            if i >= adjacency.len() {
                break;
            }
            let neighbors = &adjacency[i];
            if neighbors.is_empty() {
                continue;
            }
            let sum: f32 = neighbors
                .iter()
                .map(|&j| old.get(j).copied().unwrap_or(0.0))
                .sum();
            let avg = (old[i] + sum) / (neighbors.len() as f32 + 1.0);
            *w = avg.clamp(0.0, 1.0);
        }
    }

    /// Get weight at vertex index (returns 0.0 if out of bounds).
    pub fn get(&self, vertex_idx: usize) -> f32 {
        self.weights.get(vertex_idx).copied().unwrap_or(0.0)
    }

    /// Sum of all weights.
    pub fn total(&self) -> f32 {
        self.weights.iter().sum()
    }

    /// Number of vertices with weight > threshold.
    pub fn count_above(&self, threshold: f32) -> usize {
        self.weights.iter().filter(|&&w| w > threshold).count()
    }

    /// Blend two weight maps: self * t + other * (1-t).
    pub fn blend(&self, other: &WeightMap, t: f32) -> WeightMap {
        let t = t.clamp(0.0, 1.0);
        let len = self.weights.len().min(other.weights.len());
        let weights: Vec<f32> = (0..len)
            .map(|i| (self.weights[i] * t + other.weights[i] * (1.0 - t)).clamp(0.0, 1.0))
            .collect();
        WeightMap {
            name: self.name.clone(),
            weights,
        }
    }

    /// Multiply element-wise with another weight map.
    pub fn multiply(&self, other: &WeightMap) -> WeightMap {
        let len = self.weights.len().min(other.weights.len());
        let weights: Vec<f32> = (0..len)
            .map(|i| (self.weights[i] * other.weights[i]).clamp(0.0, 1.0))
            .collect();
        WeightMap {
            name: self.name.clone(),
            weights,
        }
    }
}

/// Build adjacency list from mesh indices (triangles: groups of 3).
#[allow(dead_code)]
pub fn build_adjacency(vertex_count: usize, indices: &[u32]) -> Vec<Vec<usize>> {
    let mut adj: Vec<std::collections::HashSet<usize>> =
        vec![std::collections::HashSet::new(); vertex_count];

    for tri in indices.chunks_exact(3) {
        let (a, b, c) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        if a < vertex_count && b < vertex_count && c < vertex_count {
            adj[a].insert(b);
            adj[a].insert(c);
            adj[b].insert(a);
            adj[b].insert(c);
            adj[c].insert(a);
            adj[c].insert(b);
        }
    }

    adj.into_iter()
        .map(|set| set.into_iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_map_new_initializes_correctly() {
        let map = WeightMap::new("test", 5, 0.7);
        assert_eq!(map.name, "test");
        assert_eq!(map.len(), 5);
        for &w in &map.weights {
            assert!((w - 0.7).abs() < 1e-6);
        }
    }

    #[test]
    fn weight_map_zeros_all_zero() {
        let map = WeightMap::zeros("z", 4);
        assert_eq!(map.len(), 4);
        for &w in &map.weights {
            assert_eq!(w, 0.0);
        }
    }

    #[test]
    fn weight_map_ones_all_one() {
        let map = WeightMap::ones("o", 4);
        assert_eq!(map.len(), 4);
        for &w in &map.weights {
            assert_eq!(w, 1.0);
        }
    }

    #[test]
    fn paint_clamps_above_one() {
        let mut map = WeightMap::zeros("m", 3);
        map.paint(1, 2.5);
        assert!((map.get(1) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn paint_clamps_below_zero() {
        let mut map = WeightMap::ones("m", 3);
        map.paint(0, -0.5);
        assert!((map.get(0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn flood_fill_sets_all() {
        let mut map = WeightMap::zeros("m", 5);
        map.flood_fill(0.42);
        for &w in &map.weights {
            assert!((w - 0.42).abs() < 1e-6);
        }
    }

    #[test]
    fn normalize_rescales_range() {
        let mut map = WeightMap {
            name: "n".into(),
            weights: vec![0.2, 0.4, 0.6, 0.8],
        };
        map.normalize();
        let min = map.weights.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = map
            .weights
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, f32::max);
        assert!((min - 0.0).abs() < 1e-6);
        assert!((max - 1.0).abs() < 1e-6);
    }

    #[test]
    fn invert_flips_weights() {
        let mut map = WeightMap::new("m", 3, 0.3);
        map.invert();
        for &w in &map.weights {
            assert!((w - 0.7).abs() < 1e-5);
        }
    }

    #[test]
    fn paint_sphere_affects_nearby_vertices() {
        let positions = vec![[0.0f32, 0.0, 0.0], [0.1, 0.0, 0.0], [0.2, 0.0, 0.0]];
        let mut map = WeightMap::zeros("m", 3);
        map.paint_sphere(&positions, [0.0, 0.0, 0.0], 0.5, 1.0);
        // All three are within radius 0.5, so they should be > 0
        assert!(map.get(0) > 0.0);
        assert!(map.get(1) > 0.0);
        assert!(map.get(2) > 0.0);
    }

    #[test]
    fn paint_sphere_does_not_affect_distant_vertices() {
        let positions = vec![
            [0.0f32, 0.0, 0.0],
            [5.0, 0.0, 0.0], // far away
        ];
        let mut map = WeightMap::zeros("m", 2);
        map.paint_sphere(&positions, [0.0, 0.0, 0.0], 0.5, 1.0);
        assert!(map.get(0) > 0.0);
        assert_eq!(map.get(1), 0.0);
    }

    #[test]
    fn smooth_once_averages_neighbors() {
        // vertex 0: weight 1.0, neighbor = vertex 1 (weight 0.0)
        // after smooth: (1.0 + 0.0) / 2 = 0.5
        let mut map = WeightMap {
            name: "s".into(),
            weights: vec![1.0, 0.0],
        };
        let adjacency = vec![vec![1usize], vec![0usize]];
        map.smooth_once(&adjacency);
        assert!((map.get(0) - 0.5).abs() < 1e-5);
        assert!((map.get(1) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn blend_at_zero_equals_other() {
        let a = WeightMap::ones("a", 4);
        let b = WeightMap::zeros("b", 4);
        let result = a.blend(&b, 0.0);
        // t=0: a*0 + b*1 = b = all zeros
        for &w in &result.weights {
            assert!((w - 0.0).abs() < 1e-6);
        }
    }

    #[test]
    fn blend_at_one_equals_self() {
        let a = WeightMap::ones("a", 4);
        let b = WeightMap::zeros("b", 4);
        let result = a.blend(&b, 1.0);
        // t=1: a*1 + b*0 = a = all ones
        for &w in &result.weights {
            assert!((w - 1.0).abs() < 1e-6);
        }
    }

    #[test]
    fn count_above_threshold() {
        let map = WeightMap {
            name: "c".into(),
            weights: vec![0.1, 0.5, 0.9, 0.3, 0.7],
        };
        assert_eq!(map.count_above(0.4), 3); // 0.5, 0.9, 0.7
        assert_eq!(map.count_above(0.8), 1); // 0.9
    }

    #[test]
    fn build_adjacency_triangle_has_mutual_edges() {
        // Single triangle: 0-1-2
        let indices = vec![0u32, 1, 2];
        let adj = build_adjacency(3, &indices);
        // Each vertex should know about the other two
        assert!(adj[0].contains(&1));
        assert!(adj[0].contains(&2));
        assert!(adj[1].contains(&0));
        assert!(adj[1].contains(&2));
        assert!(adj[2].contains(&0));
        assert!(adj[2].contains(&1));
    }

    #[test]
    fn multiply_zeros_result() {
        let a = WeightMap::ones("a", 4);
        let b = WeightMap::zeros("b", 4);
        let result = a.multiply(&b);
        for &w in &result.weights {
            assert_eq!(w, 0.0);
        }
    }
}
