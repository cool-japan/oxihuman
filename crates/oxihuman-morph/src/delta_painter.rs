// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Programmatic vertex delta painting for creating morph targets.
//!
//! Provides a brush-based workflow for interactively or procedurally
//! building morph target deltas, including masking, mirroring, and
//! Laplacian smoothing.

use serde::{Deserialize, Serialize};

// ── Brush types ──────────────────────────────────────────────────────────────

/// Brush falloff function controlling how influence decreases with distance.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BrushFalloff {
    /// Linearly decreasing from center to edge.
    Linear,
    /// Hermite (smooth-step) curve — soft center, soft edge.
    Smooth,
    /// Sharp peak at centre, rapid drop-off.
    Sharp,
    /// Uniform influence across the entire brush radius.
    Flat,
}

impl BrushFalloff {
    /// Evaluate the falloff at a normalised distance `t ∈ [0, 1]`.
    /// Returns a value in `[0, 1]` where 1 is full influence.
    pub fn evaluate(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => 1.0 - t,
            Self::Smooth => {
                let u = 1.0 - t;
                // Hermite smooth-step: 3u² - 2u³
                u * u * (3.0 - 2.0 * u)
            }
            Self::Sharp => {
                let u = 1.0 - t;
                u * u * u
            }
            Self::Flat => 1.0,
        }
    }
}

/// A painting brush that determines how deltas are applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaintBrush {
    /// World-space radius of the brush.
    pub radius: f64,
    /// Falloff function.
    pub falloff: BrushFalloff,
    /// Overall strength multiplier `[0, 1]`.
    pub strength: f64,
}

impl Default for PaintBrush {
    fn default() -> Self {
        Self {
            radius: 0.05,
            falloff: BrushFalloff::Smooth,
            strength: 1.0,
        }
    }
}

// ── Mirror axis ──────────────────────────────────────────────────────────────

/// Axis across which to mirror deltas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirrorAxis {
    X,
    Y,
    Z,
}

impl MirrorAxis {
    /// Index into `[f64; 3]` for this axis.
    #[inline]
    pub fn idx(self) -> usize {
        match self {
            Self::X => 0,
            Self::Y => 1,
            Self::Z => 2,
        }
    }
}

// ── MorphTargetData ──────────────────────────────────────────────────────────

/// Exported morph-target data ready for storage or further processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorphTargetData {
    /// Human-readable name.
    pub name: String,
    /// Full (dense) delta array, length = vertex count.
    pub deltas: Vec<[f64; 3]>,
    /// Indices of vertices whose delta is non-zero.
    pub sparse_indices: Vec<usize>,
    /// Corresponding non-zero deltas (same order as `sparse_indices`).
    pub sparse_deltas: Vec<[f64; 3]>,
}

// ── DeltaPainter ─────────────────────────────────────────────────────────────

/// Interactive / procedural delta painter.
///
/// Accumulates per-vertex displacements and an optional mask, then exports
/// the result as a [`MorphTargetData`].
pub struct DeltaPainter {
    vertex_count: usize,
    deltas: Vec<[f64; 3]>,
    /// Per-vertex mask; 0 = no effect, 1 = full effect.
    mask: Vec<f64>,
}

impl DeltaPainter {
    /// Create a new painter for the given vertex count (all deltas zero, mask = 1).
    pub fn new(vertex_count: usize) -> Self {
        Self {
            vertex_count,
            deltas: vec![[0.0; 3]; vertex_count],
            mask: vec![1.0; vertex_count],
        }
    }

    /// Number of vertices managed by this painter.
    #[inline]
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    // ── painting ─────────────────────────────────────────────────────────

    /// Paint `delta` at `center_vertex`, with brush falloff to neighbours.
    ///
    /// `vertex_positions` must have length ≥ `self.vertex_count`.
    pub fn paint_at(
        &mut self,
        center_vertex: usize,
        delta: [f64; 3],
        brush: &PaintBrush,
        vertex_positions: &[[f64; 3]],
    ) -> anyhow::Result<()> {
        if center_vertex >= self.vertex_count {
            anyhow::bail!(
                "center_vertex {} out of range (vertex_count = {})",
                center_vertex,
                self.vertex_count
            );
        }
        if vertex_positions.len() < self.vertex_count {
            anyhow::bail!(
                "vertex_positions length {} < vertex_count {}",
                vertex_positions.len(),
                self.vertex_count
            );
        }
        if brush.radius <= 0.0 {
            anyhow::bail!("brush radius must be positive, got {}", brush.radius);
        }

        let center_pos = vertex_positions[center_vertex];
        let radius_sq = brush.radius * brush.radius;

        for (i, vpos) in vertex_positions.iter().enumerate().take(self.vertex_count) {
            let dx = vpos[0] - center_pos[0];
            let dy = vpos[1] - center_pos[1];
            let dz = vpos[2] - center_pos[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq > radius_sq {
                continue;
            }
            let dist = dist_sq.sqrt();
            let t = dist / brush.radius;
            let influence = brush.falloff.evaluate(t) * brush.strength;
            self.deltas[i][0] += delta[0] * influence;
            self.deltas[i][1] += delta[1] * influence;
            self.deltas[i][2] += delta[2] * influence;
        }
        Ok(())
    }

    /// Paint along a stroke — a series of vertex indices.
    ///
    /// The same `delta` and brush are applied at each vertex in turn.
    pub fn paint_stroke(
        &mut self,
        vertices: &[usize],
        delta: [f64; 3],
        brush: &PaintBrush,
        vertex_positions: &[[f64; 3]],
    ) -> anyhow::Result<()> {
        for &v in vertices {
            self.paint_at(v, delta, brush, vertex_positions)?;
        }
        Ok(())
    }

    // ── masking ──────────────────────────────────────────────────────────

    /// Set the mask value for a single vertex.
    pub fn set_mask(&mut self, vertex: usize, value: f64) -> anyhow::Result<()> {
        if vertex >= self.vertex_count {
            anyhow::bail!(
                "vertex {} out of range (vertex_count = {})",
                vertex,
                self.vertex_count
            );
        }
        self.mask[vertex] = value.clamp(0.0, 1.0);
        Ok(())
    }

    /// Set the mask for all vertices in `vertices` to `value`.
    pub fn mask_vertex_group(&mut self, vertices: &[usize], value: f64) {
        let clamped = value.clamp(0.0, 1.0);
        for &v in vertices {
            if v < self.vertex_count {
                self.mask[v] = clamped;
            }
        }
    }

    /// Clear the mask (set all to 1.0).
    pub fn clear_mask(&mut self) {
        for m in &mut self.mask {
            *m = 1.0;
        }
    }

    /// Invert the mask (1 - m for each vertex).
    pub fn invert_mask(&mut self) {
        for m in &mut self.mask {
            *m = 1.0 - *m;
        }
    }

    // ── mirror ───────────────────────────────────────────────────────────

    /// Mirror deltas across the given axis.
    ///
    /// For each vertex at position P, finds the closest vertex at the
    /// mirror-reflected position (within `tolerance`) and copies the delta
    /// with the axis component negated.
    pub fn mirror(
        &mut self,
        axis: MirrorAxis,
        vertex_positions: &[[f64; 3]],
        tolerance: f64,
    ) -> anyhow::Result<()> {
        if vertex_positions.len() < self.vertex_count {
            anyhow::bail!(
                "vertex_positions length {} < vertex_count {}",
                vertex_positions.len(),
                self.vertex_count
            );
        }
        if tolerance <= 0.0 {
            anyhow::bail!("tolerance must be positive, got {}", tolerance);
        }

        let ax = axis.idx();
        let tol_sq = tolerance * tolerance;
        let original_deltas = self.deltas.clone();

        // Build pairs: for each vertex on the positive side, find its mirror.
        for i in 0..self.vertex_count {
            let pos = vertex_positions[i];
            // Only process vertices on the positive side of the axis.
            if pos[ax] < 0.0 {
                continue;
            }

            // Construct mirror position.
            let mut mirror_pos = pos;
            mirror_pos[ax] = -mirror_pos[ax];

            // Find closest vertex to mirror_pos.
            let mut best_j: Option<usize> = None;
            let mut best_dist_sq = f64::MAX;
            for (j, jpos) in vertex_positions.iter().enumerate().take(self.vertex_count) {
                let dp0 = jpos[0] - mirror_pos[0];
                let dp1 = jpos[1] - mirror_pos[1];
                let dp2 = jpos[2] - mirror_pos[2];
                let dsq = dp0 * dp0 + dp1 * dp1 + dp2 * dp2;
                if dsq < best_dist_sq {
                    best_dist_sq = dsq;
                    best_j = Some(j);
                }
            }

            if let Some(j) = best_j {
                if best_dist_sq <= tol_sq {
                    let mut d = original_deltas[i];
                    d[ax] = -d[ax]; // negate the mirror axis component
                    self.deltas[j] = d;
                }
            }
        }
        Ok(())
    }

    // ── smoothing ────────────────────────────────────────────────────────

    /// Apply Laplacian smoothing to the accumulated deltas.
    ///
    /// `adjacency[v]` is the list of vertex indices adjacent to vertex `v`.
    pub fn smooth(&mut self, iterations: usize, adjacency: &[Vec<usize>]) -> anyhow::Result<()> {
        if adjacency.len() < self.vertex_count {
            anyhow::bail!(
                "adjacency length {} < vertex_count {}",
                adjacency.len(),
                self.vertex_count
            );
        }
        for _ in 0..iterations {
            let prev = self.deltas.clone();
            for (i, nbrs) in adjacency[..self.vertex_count].iter().enumerate() {
                if nbrs.is_empty() {
                    continue;
                }
                let mut avg = [0.0_f64; 3];
                let mut count = 0usize;
                for &nb in nbrs {
                    if nb < self.vertex_count {
                        avg[0] += prev[nb][0];
                        avg[1] += prev[nb][1];
                        avg[2] += prev[nb][2];
                        count += 1;
                    }
                }
                if count > 0 {
                    let c = count as f64;
                    self.deltas[i] = [avg[0] / c, avg[1] / c, avg[2] / c];
                }
            }
        }
        Ok(())
    }

    // ── output ───────────────────────────────────────────────────────────

    /// Return the final delta array with the mask applied.
    pub fn get_deltas(&self) -> Vec<[f64; 3]> {
        self.deltas
            .iter()
            .zip(self.mask.iter())
            .map(|(d, &m)| [d[0] * m, d[1] * m, d[2] * m])
            .collect()
    }

    /// Return a reference to the raw (un-masked) deltas.
    pub fn raw_deltas(&self) -> &[[f64; 3]] {
        &self.deltas
    }

    /// Return a reference to the mask.
    pub fn mask(&self) -> &[f64] {
        &self.mask
    }

    /// Clear all deltas (reset to zero).
    pub fn clear(&mut self) {
        for d in &mut self.deltas {
            *d = [0.0; 3];
        }
    }

    /// Export as a [`MorphTargetData`].
    pub fn to_morph_target(&self, name: &str) -> MorphTargetData {
        let deltas = self.get_deltas();
        let threshold = 1e-12;
        let mut sparse_indices = Vec::new();
        let mut sparse_deltas = Vec::new();
        for (i, d) in deltas.iter().enumerate() {
            let mag_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            if mag_sq > threshold * threshold {
                sparse_indices.push(i);
                sparse_deltas.push(*d);
            }
        }
        MorphTargetData {
            name: name.to_owned(),
            deltas,
            sparse_indices,
            sparse_deltas,
        }
    }

    /// Set a specific vertex delta directly (bypassing brush).
    pub fn set_delta(&mut self, vertex: usize, delta: [f64; 3]) -> anyhow::Result<()> {
        if vertex >= self.vertex_count {
            anyhow::bail!(
                "vertex {} out of range (vertex_count = {})",
                vertex,
                self.vertex_count
            );
        }
        self.deltas[vertex] = delta;
        Ok(())
    }

    /// Add to a specific vertex delta directly (bypassing brush).
    pub fn add_delta(&mut self, vertex: usize, delta: [f64; 3]) -> anyhow::Result<()> {
        if vertex >= self.vertex_count {
            anyhow::bail!(
                "vertex {} out of range (vertex_count = {})",
                vertex,
                self.vertex_count
            );
        }
        self.deltas[vertex][0] += delta[0];
        self.deltas[vertex][1] += delta[1];
        self.deltas[vertex][2] += delta[2];
        Ok(())
    }

    /// Scale all deltas by a uniform factor.
    pub fn scale_all(&mut self, factor: f64) {
        for d in &mut self.deltas {
            d[0] *= factor;
            d[1] *= factor;
            d[2] *= factor;
        }
    }

    /// Blend another painter's deltas into this one.
    pub fn blend_from(&mut self, other: &DeltaPainter, weight: f64) -> anyhow::Result<()> {
        if other.vertex_count != self.vertex_count {
            anyhow::bail!(
                "vertex_count mismatch: self={}, other={}",
                self.vertex_count,
                other.vertex_count
            );
        }
        let w = weight.clamp(0.0, 1.0);
        for i in 0..self.vertex_count {
            self.deltas[i][0] += other.deltas[i][0] * w;
            self.deltas[i][1] += other.deltas[i][1] * w;
            self.deltas[i][2] += other.deltas[i][2] * w;
        }
        Ok(())
    }
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_positions() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
        ]
    }

    #[test]
    fn test_new_painter() {
        let p = DeltaPainter::new(10);
        assert_eq!(p.vertex_count(), 10);
        assert_eq!(p.raw_deltas().len(), 10);
        assert_eq!(p.mask().len(), 10);
        for d in p.raw_deltas() {
            assert_eq!(*d, [0.0; 3]);
        }
        for &m in p.mask() {
            assert!((m - 1.0).abs() < 1e-15);
        }
    }

    #[test]
    fn test_paint_at_center() {
        let positions = simple_positions();
        let mut p = DeltaPainter::new(4);
        let brush = PaintBrush {
            radius: 0.5,
            falloff: BrushFalloff::Flat,
            strength: 1.0,
        };
        p.paint_at(0, [0.0, 0.0, 1.0], &brush, &positions)
            .expect("paint_at should succeed");

        // Vertex 0 is at center — should get full delta
        let d = p.raw_deltas()[0];
        assert!((d[2] - 1.0).abs() < 1e-10);

        // Vertex 1 is at distance 1.0 > radius 0.5 — should be zero
        let d1 = p.raw_deltas()[1];
        assert!(d1[2].abs() < 1e-10);
    }

    #[test]
    fn test_paint_at_out_of_range() {
        let positions = simple_positions();
        let mut p = DeltaPainter::new(4);
        let brush = PaintBrush::default();
        let result = p.paint_at(10, [0.0, 0.0, 1.0], &brush, &positions);
        assert!(result.is_err());
    }

    #[test]
    fn test_paint_stroke() {
        let positions = simple_positions();
        let mut p = DeltaPainter::new(4);
        let brush = PaintBrush {
            radius: 0.01,
            falloff: BrushFalloff::Flat,
            strength: 1.0,
        };
        p.paint_stroke(&[0, 1], [0.0, 1.0, 0.0], &brush, &positions)
            .expect("paint_stroke should succeed");

        // Both vertices 0 and 1 should have Y delta
        assert!(p.raw_deltas()[0][1] > 0.5);
        assert!(p.raw_deltas()[1][1] > 0.5);
    }

    #[test]
    fn test_mask_blocks_output() {
        let mut p = DeltaPainter::new(3);
        p.set_delta(0, [1.0, 0.0, 0.0]).expect("set ok");
        p.set_delta(1, [1.0, 0.0, 0.0]).expect("set ok");
        p.set_mask(0, 0.0).expect("mask ok");
        p.set_mask(1, 0.5).expect("mask ok");

        let out = p.get_deltas();
        assert!(out[0][0].abs() < 1e-15, "masked vertex should be zero");
        assert!((out[1][0] - 0.5).abs() < 1e-10, "half-masked should be 0.5");
        assert!((out[2][0]).abs() < 1e-15, "untouched vertex should be zero");
    }

    #[test]
    fn test_mask_vertex_group() {
        let mut p = DeltaPainter::new(5);
        p.mask_vertex_group(&[1, 3], 0.25);
        assert!((p.mask()[1] - 0.25).abs() < 1e-15);
        assert!((p.mask()[3] - 0.25).abs() < 1e-15);
        assert!((p.mask()[0] - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_invert_mask() {
        let mut p = DeltaPainter::new(3);
        p.set_mask(0, 0.0).expect("ok");
        p.set_mask(1, 0.3).expect("ok");
        p.invert_mask();
        assert!((p.mask()[0] - 1.0).abs() < 1e-15);
        assert!((p.mask()[1] - 0.7).abs() < 1e-10);
        assert!((p.mask()[2] - 0.0).abs() < 1e-15);
    }

    #[test]
    fn test_smooth_reduces_peak() {
        let mut p = DeltaPainter::new(3);
        p.set_delta(0, [10.0, 0.0, 0.0]).expect("ok");
        // Simple chain: 0-1, 1-2
        let adj = vec![vec![1], vec![0, 2], vec![1]];
        let before = p.raw_deltas()[0][0];
        p.smooth(1, &adj).expect("smooth ok");
        let after = p.raw_deltas()[0][0];
        assert!(after < before, "smoothing should reduce peak");
    }

    #[test]
    fn test_mirror_x() {
        // Two vertices symmetric about X=0
        let positions = vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mut p = DeltaPainter::new(2);
        p.set_delta(1, [0.5, 0.3, 0.1]).expect("ok");
        p.mirror(MirrorAxis::X, &positions, 0.1).expect("mirror ok");
        // Vertex 0 should get the mirror: X negated
        let d0 = p.raw_deltas()[0];
        assert!(
            (d0[0] - (-0.5)).abs() < 1e-10,
            "X component should be negated"
        );
        assert!((d0[1] - 0.3).abs() < 1e-10);
        assert!((d0[2] - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_to_morph_target_sparse() {
        let mut p = DeltaPainter::new(5);
        p.set_delta(1, [1.0, 0.0, 0.0]).expect("ok");
        p.set_delta(3, [0.0, 2.0, 0.0]).expect("ok");
        let mt = p.to_morph_target("test_sparse");
        assert_eq!(mt.name, "test_sparse");
        assert_eq!(mt.deltas.len(), 5);
        assert_eq!(mt.sparse_indices.len(), 2);
        assert_eq!(mt.sparse_deltas.len(), 2);
        assert!(mt.sparse_indices.contains(&1));
        assert!(mt.sparse_indices.contains(&3));
    }

    #[test]
    fn test_clear() {
        let mut p = DeltaPainter::new(3);
        p.set_delta(0, [1.0, 2.0, 3.0]).expect("ok");
        p.clear();
        for d in p.raw_deltas() {
            assert_eq!(*d, [0.0; 3]);
        }
    }

    #[test]
    fn test_scale_all() {
        let mut p = DeltaPainter::new(2);
        p.set_delta(0, [1.0, 2.0, 3.0]).expect("ok");
        p.scale_all(0.5);
        let d = p.raw_deltas()[0];
        assert!((d[0] - 0.5).abs() < 1e-15);
        assert!((d[1] - 1.0).abs() < 1e-15);
        assert!((d[2] - 1.5).abs() < 1e-15);
    }

    #[test]
    fn test_blend_from() {
        let mut a = DeltaPainter::new(3);
        a.set_delta(0, [1.0, 0.0, 0.0]).expect("ok");
        let mut b = DeltaPainter::new(3);
        b.set_delta(0, [0.0, 2.0, 0.0]).expect("ok");
        a.blend_from(&b, 0.5).expect("blend ok");
        let d = a.raw_deltas()[0];
        assert!((d[0] - 1.0).abs() < 1e-15);
        assert!((d[1] - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_brush_falloff_linear() {
        assert!((BrushFalloff::Linear.evaluate(0.0) - 1.0).abs() < 1e-15);
        assert!((BrushFalloff::Linear.evaluate(1.0)).abs() < 1e-15);
        assert!((BrushFalloff::Linear.evaluate(0.5) - 0.5).abs() < 1e-15);
    }

    #[test]
    fn test_brush_falloff_smooth() {
        assert!((BrushFalloff::Smooth.evaluate(0.0) - 1.0).abs() < 1e-15);
        assert!((BrushFalloff::Smooth.evaluate(1.0)).abs() < 1e-15);
        // Smooth at t=0.5 should be 0.5 (Hermite property)
        assert!((BrushFalloff::Smooth.evaluate(0.5) - 0.5).abs() < 1e-15);
    }

    #[test]
    fn test_brush_falloff_sharp() {
        assert!((BrushFalloff::Sharp.evaluate(0.0) - 1.0).abs() < 1e-15);
        assert!((BrushFalloff::Sharp.evaluate(1.0)).abs() < 1e-15);
        // Sharp at 0.5 = (0.5)^3 = 0.125
        assert!((BrushFalloff::Sharp.evaluate(0.5) - 0.125).abs() < 1e-15);
    }

    #[test]
    fn test_brush_falloff_flat() {
        assert!((BrushFalloff::Flat.evaluate(0.0) - 1.0).abs() < 1e-15);
        assert!((BrushFalloff::Flat.evaluate(0.5) - 1.0).abs() < 1e-15);
        assert!((BrushFalloff::Flat.evaluate(1.0) - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_set_mask_out_of_range() {
        let mut p = DeltaPainter::new(3);
        assert!(p.set_mask(5, 0.5).is_err());
    }

    #[test]
    fn test_smooth_adjacency_too_short() {
        let mut p = DeltaPainter::new(5);
        let adj = vec![vec![1], vec![0]]; // too short
        assert!(p.smooth(1, &adj).is_err());
    }

    #[test]
    fn test_mirror_tolerance_too_small() {
        let positions = vec![[1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]];
        let mut p = DeltaPainter::new(2);
        assert!(p.mirror(MirrorAxis::X, &positions, -0.1).is_err());
    }

    #[test]
    fn test_add_delta() {
        let mut p = DeltaPainter::new(2);
        p.set_delta(0, [1.0, 2.0, 3.0]).expect("ok");
        p.add_delta(0, [0.5, 0.5, 0.5]).expect("ok");
        let d = p.raw_deltas()[0];
        assert!((d[0] - 1.5).abs() < 1e-15);
        assert!((d[1] - 2.5).abs() < 1e-15);
        assert!((d[2] - 3.5).abs() < 1e-15);
    }

    #[test]
    fn test_clear_mask() {
        let mut p = DeltaPainter::new(3);
        p.set_mask(0, 0.0).expect("ok");
        p.set_mask(1, 0.5).expect("ok");
        p.clear_mask();
        for &m in p.mask() {
            assert!((m - 1.0).abs() < 1e-15);
        }
    }
}
