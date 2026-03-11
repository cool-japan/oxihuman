// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Per-vertex paint mask storage and operations.

/// Paint mask — one weight per vertex in [0, 1].
#[derive(Debug, Clone)]
pub struct PaintMask {
    pub weights: Vec<f32>,
}

impl PaintMask {
    /// Create a new mask with all weights set to zero.
    pub fn new(vertex_count: usize) -> Self {
        Self {
            weights: vec![0.0; vertex_count],
        }
    }

    /// Return the number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.weights.len()
    }

    /// Set the weight of a single vertex (clamped to `[0,1]`).
    pub fn set(&mut self, vertex: usize, weight: f32) {
        if let Some(w) = self.weights.get_mut(vertex) {
            *w = weight.clamp(0.0, 1.0);
        }
    }

    /// Get the weight of a vertex.
    pub fn get(&self, vertex: usize) -> f32 {
        self.weights.get(vertex).copied().unwrap_or(0.0)
    }

    /// Invert all weights (w → 1 - w).
    pub fn invert(&mut self) {
        for w in &mut self.weights {
            *w = 1.0 - *w;
        }
    }

    /// Clamp all weights to [lo, hi].
    pub fn clamp_range(&mut self, lo: f32, hi: f32) {
        for w in &mut self.weights {
            *w = w.clamp(lo, hi);
        }
    }
}

/// Compute average weight of the mask.
pub fn average_weight(mask: &PaintMask) -> f32 {
    if mask.weights.is_empty() {
        return 0.0;
    }
    let sum: f32 = mask.weights.iter().sum();
    sum / mask.weights.len() as f32
}

/// Count vertices whose weight is above a threshold.
pub fn count_above(mask: &PaintMask, threshold: f32) -> usize {
    mask.weights.iter().filter(|&&w| w > threshold).count()
}

/// Fill mask from a byte slice (values 0-255 → 0.0-1.0).
#[allow(clippy::needless_range_loop)]
pub fn from_bytes(mask: &mut PaintMask, data: &[u8]) {
    let len = mask.weights.len().min(data.len());
    for i in 0..len {
        mask.weights[i] = data[i] as f32 / 255.0;
    }
}

/// Convert mask to bytes (0.0-1.0 → 0-255).
pub fn to_bytes(mask: &PaintMask) -> Vec<u8> {
    mask.weights
        .iter()
        .map(|&w| (w.clamp(0.0, 1.0) * 255.0) as u8)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mask4() -> PaintMask {
        PaintMask::new(4)
    }

    #[test]
    fn test_new_all_zero() {
        /* new mask starts with all weights zero */
        let m = mask4();
        assert!(m.weights.iter().all(|&w| w == 0.0));
    }

    #[test]
    fn test_set_and_get() {
        /* set and get round-trip */
        let mut m = mask4();
        m.set(2, 0.75);
        assert!((m.get(2) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_set_clamps() {
        /* set clamps weight to [0,1] */
        let mut m = mask4();
        m.set(0, 1.5);
        assert!((m.get(0) - 1.0).abs() < 1e-6);
        m.set(0, -0.5);
        assert!((m.get(0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_invert() {
        /* invert flips weights */
        let mut m = mask4();
        m.set(0, 0.25);
        m.invert();
        assert!((m.get(0) - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_average_weight() {
        /* average weight is computed correctly */
        let mut m = PaintMask::new(2);
        m.set(0, 0.4);
        m.set(1, 0.6);
        assert!((average_weight(&m) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_count_above() {
        /* count above threshold is correct */
        let mut m = mask4();
        m.set(0, 0.5);
        m.set(1, 0.8);
        assert_eq!(count_above(&m, 0.4), 2);
    }

    #[test]
    fn test_from_to_bytes_round_trip() {
        /* byte round-trip is approximately correct */
        let mut m = PaintMask::new(3);
        let data = vec![0u8, 128, 255];
        from_bytes(&mut m, &data);
        let out = to_bytes(&m);
        assert_eq!(out[0], 0);
        assert_eq!(out[2], 255);
    }

    #[test]
    fn test_get_out_of_bounds() {
        /* get out of bounds returns 0.0 */
        let m = mask4();
        assert_eq!(m.get(999), 0.0);
    }
}
