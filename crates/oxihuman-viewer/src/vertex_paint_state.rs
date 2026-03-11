#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Vertex paint state management for interactive morph target editing.
//!
//! Provides a full paint-state machine with multiple modes (weight, displace,
//! smooth, erase, select), configurable brush falloff, undo/redo history,
//! symmetry mirroring, flood fill, and morph-target delta export.

use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Paint mode
// ---------------------------------------------------------------------------

/// Which operation the brush performs on each stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum PaintMode {
    /// Paint vertex weights clamped to `[0.0, 1.0]`.
    #[default]
    Weight,
    /// Paint displacement along the vertex normal.
    Displace,
    /// Smooth (average) existing values with neighbours.
    Smooth,
    /// Erase -- push values towards zero.
    Erase,
    /// Select / deselect vertices (toggle selection mask).
    Select,
}


// ---------------------------------------------------------------------------
// Falloff curve
// ---------------------------------------------------------------------------

/// Brush falloff profile.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(Default)]
pub enum FalloffCurve {
    /// Linear ramp from 1.0 at centre to 0.0 at edge.
    Linear,
    /// Hermite smooth-step (3t^2 - 2t^3).
    #[default]
    Smooth,
    /// Steep ramp that drops off quickly near the edge.
    Sharp,
    /// Constant 1.0 everywhere inside radius.
    Flat,
    /// Power curve: `(1 - t)^exponent`.  Exponent must be > 0.
    Custom(f64),
}


impl FalloffCurve {
    /// Evaluate the falloff for a normalised distance `t` in `[0, 1]`.
    /// Returns a weight in `[0, 1]`.
    pub fn evaluate(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => 1.0 - t,
            Self::Smooth => {
                let u = 1.0 - t;
                u * u * (3.0 - 2.0 * u)
            }
            Self::Sharp => {
                let u = 1.0 - t;
                u * u * u
            }
            Self::Flat => 1.0,
            Self::Custom(exp) => {
                let exp = if *exp > 0.0 { *exp } else { 1.0 };
                (1.0 - t).powf(exp)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Brush config
// ---------------------------------------------------------------------------

/// Configuration for the paint brush.
#[derive(Debug, Clone)]
pub struct PaintBrushConfig {
    /// Brush radius in world-space units.
    pub radius: f64,
    /// Strength multiplier applied per dab (0.0 -- 1.0 recommended).
    pub strength: f64,
    /// Falloff curve.
    pub falloff: FalloffCurve,
    /// When `true`, repeated dabs in the same stroke accumulate.
    pub accumulate: bool,
}

impl Default for PaintBrushConfig {
    fn default() -> Self {
        Self {
            radius: 0.05,
            strength: 0.5,
            falloff: FalloffCurve::default(),
            accumulate: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Symmetry
// ---------------------------------------------------------------------------

/// Which axis to mirror across.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum SymmetryAxis {
    #[default]
    X,
    Y,
    Z,
}


/// Symmetry settings for mirrored painting.
#[derive(Debug, Clone)]
pub struct SymmetryConfig {
    /// Whether symmetry painting is active.
    pub enabled: bool,
    /// Axis to mirror across.
    pub axis: SymmetryAxis,
    /// Distance tolerance when matching mirror vertices.
    pub tolerance: f64,
    /// Pre-computed mirror map: vertex `i` mirrors to `mirror_map[i]`.
    pub mirror_map: Vec<Option<usize>>,
}

impl Default for SymmetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            axis: SymmetryAxis::X,
            tolerance: 1e-4,
            mirror_map: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// History / undo-redo
// ---------------------------------------------------------------------------

/// A single reversible paint action.
#[derive(Debug, Clone)]
pub struct PaintAction {
    /// Indices of affected vertices.
    pub affected_vertices: Vec<usize>,
    /// Values *before* the action.
    pub old_values: Vec<f64>,
    /// Values *after* the action.
    pub new_values: Vec<f64>,
    /// Human-readable description for UI display.
    pub description: String,
}

/// Undo / redo stack with a configurable maximum depth.
#[derive(Debug)]
pub struct PaintHistory {
    undo_stack: VecDeque<PaintAction>,
    redo_stack: Vec<PaintAction>,
    max_history: usize,
}

impl PaintHistory {
    /// Create a new history with the given maximum depth.
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_history),
            redo_stack: Vec::new(),
            max_history,
        }
    }

    /// Push a new action, clearing the redo stack.
    pub fn push(&mut self, action: PaintAction) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(action);
        self.redo_stack.clear();
    }

    /// Pop the most recent action for undo.  Returns `None` if empty.
    pub fn pop_undo(&mut self) -> Option<PaintAction> {
        self.undo_stack.pop_back()
    }

    /// Pop the most recent redo action.  Returns `None` if empty.
    pub fn pop_redo(&mut self) -> Option<PaintAction> {
        self.redo_stack.pop()
    }

    /// Move an action onto the redo stack (called after undo).
    pub fn push_redo(&mut self, action: PaintAction) {
        self.redo_stack.push(action);
    }

    /// Move an action back onto the undo stack (called after redo).
    pub fn push_undo(&mut self, action: PaintAction) {
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(action);
    }

    /// Number of available undo steps.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of available redo steps.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for PaintHistory {
    fn default() -> Self {
        Self::new(128)
    }
}

// ---------------------------------------------------------------------------
// Paint result
// ---------------------------------------------------------------------------

/// The outcome of a single paint operation, returned to the caller so it can
/// update GPU buffers, UI, etc.
#[derive(Debug, Clone)]
pub struct PaintResult {
    /// Vertex indices that were modified.
    pub modified_vertices: Vec<usize>,
    /// Values before modification (parallel to `modified_vertices`).
    pub old_values: Vec<f64>,
    /// Values after modification (parallel to `modified_vertices`).
    pub new_values: Vec<f64>,
}

// ---------------------------------------------------------------------------
// Legacy compat -- kept for existing callers
// ---------------------------------------------------------------------------

/// Legacy vertex paint mode state (colour-layer painting).
#[derive(Debug, Clone)]
pub struct LegacyVertexPaintState {
    pub color: [f32; 4],
    pub radius: f32,
    pub strength: f32,
    pub blend_mode: u8,
    pub active_layer: String,
}

pub fn new_vertex_paint_state() -> LegacyVertexPaintState {
    LegacyVertexPaintState {
        color: [1.0, 1.0, 1.0, 1.0],
        radius: 0.1,
        strength: 1.0,
        blend_mode: 0,
        active_layer: "Col".to_string(),
    }
}

pub fn vp_set_color(state: &mut LegacyVertexPaintState, color: [f32; 4]) {
    state.color = color;
}

pub fn vp_set_radius(state: &mut LegacyVertexPaintState, r: f32) {
    state.radius = r.max(0.0);
}

pub fn vp_blend_mode_name(mode: u8) -> &'static str {
    match mode {
        0 => "Mix",
        1 => "Add",
        2 => "Subtract",
        3 => "Multiply",
        4 => "Overlay",
        _ => "Custom",
    }
}

// ---------------------------------------------------------------------------
// Main state
// ---------------------------------------------------------------------------

/// Vertex paint state for interactive morph-target editing.
///
/// Holds the current paint mode, brush configuration, per-vertex value buffer,
/// selection mask, symmetry settings, and undo / redo history.
pub struct VertexPaintState {
    /// Current paint mode.
    mode: PaintMode,
    /// Active brush configuration.
    brush: PaintBrushConfig,
    /// Undo / redo history.
    history: PaintHistory,
    /// Per-vertex weight / delta values being painted.
    values: Vec<f64>,
    /// Per-vertex selection mask.
    selection: Vec<bool>,
    /// Symmetry settings.
    symmetry: SymmetryConfig,
    /// Total number of vertices.
    vertex_count: usize,
}

impl VertexPaintState {
    // -- Construction --------------------------------------------------------

    /// Create a new paint state for a mesh with `vertex_count` vertices.
    pub fn new(vertex_count: usize) -> Self {
        Self {
            mode: PaintMode::default(),
            brush: PaintBrushConfig::default(),
            history: PaintHistory::default(),
            values: vec![0.0; vertex_count],
            selection: vec![false; vertex_count],
            symmetry: SymmetryConfig::default(),
            vertex_count,
        }
    }

    // -- Accessors -----------------------------------------------------------

    /// Current paint mode.
    pub fn mode(&self) -> PaintMode {
        self.mode
    }

    /// Set the paint mode.
    pub fn set_mode(&mut self, mode: PaintMode) {
        self.mode = mode;
    }

    /// Current brush configuration (immutable reference).
    pub fn brush(&self) -> &PaintBrushConfig {
        &self.brush
    }

    /// Replace the entire brush configuration.
    pub fn set_brush(&mut self, brush: PaintBrushConfig) {
        self.brush = brush;
    }

    /// Current symmetry configuration (immutable reference).
    pub fn symmetry(&self) -> &SymmetryConfig {
        &self.symmetry
    }

    /// Replace the symmetry configuration.
    pub fn set_symmetry(&mut self, sym: SymmetryConfig) {
        self.symmetry = sym;
    }

    /// Read-only slice of per-vertex values.
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Read-only slice of per-vertex selection flags.
    pub fn selection(&self) -> &[bool] {
        &self.selection
    }

    /// Number of vertices.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    /// Reference to the history (for inspecting undo/redo depth).
    pub fn history(&self) -> &PaintHistory {
        &self.history
    }

    // -- Clear / reset -------------------------------------------------------

    /// Reset all values to zero and clear the selection mask.
    pub fn clear(&mut self) {
        for v in &mut self.values {
            *v = 0.0;
        }
        for s in &mut self.selection {
            *s = false;
        }
    }

    /// Clear values, selection, **and** history.
    pub fn reset(&mut self) {
        self.clear();
        self.history.clear();
    }

    // -- Painting ------------------------------------------------------------

    /// Apply a paint dab centred on `hit_vertex`.
    ///
    /// All vertices within `brush.radius` of `hit_vertex` (measured using
    /// Euclidean distance in `vertex_positions`) are affected.  If symmetry
    /// is enabled the mirror of each affected vertex is also painted.
    ///
    /// The action is automatically recorded in the undo history.
    pub fn paint_at(
        &mut self,
        hit_vertex: usize,
        vertex_positions: &[[f64; 3]],
        _adjacency: &[Vec<usize>],
    ) -> anyhow::Result<PaintResult> {
        if hit_vertex >= self.vertex_count {
            anyhow::bail!(
                "hit_vertex {} out of range (vertex_count = {})",
                hit_vertex,
                self.vertex_count
            );
        }
        if vertex_positions.len() != self.vertex_count {
            anyhow::bail!(
                "vertex_positions length {} != vertex_count {}",
                vertex_positions.len(),
                self.vertex_count
            );
        }

        let center = vertex_positions[hit_vertex];
        let radius = self.brush.radius;
        let radius_sq = radius * radius;

        // Gather vertices inside the brush sphere.
        let mut affected: Vec<usize> = vertex_positions[..self.vertex_count]
            .iter()
            .enumerate()
            .filter(|(_, vp)| dist_sq(&center, vp) <= radius_sq)
            .map(|(i, _)| i)
            .collect();

        // Also gather mirrored vertices.
        if self.symmetry.enabled && !self.symmetry.mirror_map.is_empty() {
            let mut mirrored: Vec<usize> = Vec::new();
            for &idx in &affected {
                if let Some(Some(m)) = self.symmetry.mirror_map.get(idx) {
                    if *m < self.vertex_count && !affected.contains(m) && !mirrored.contains(m) {
                        mirrored.push(*m);
                    }
                }
            }
            affected.extend(mirrored);
        }

        affected.sort_unstable();
        affected.dedup();

        let old_vals: Vec<f64> = affected.iter().map(|&i| self.values[i]).collect();

        // Apply the brush to each affected vertex.
        for &idx in &affected {
            let d = dist(&center, &vertex_positions[idx]);
            let t = if radius > 0.0 { d / radius } else { 0.0 };
            let falloff = self.brush.falloff.evaluate(t);
            let strength = self.brush.strength * falloff;

            self.apply_mode(idx, strength);
        }

        let new_vals: Vec<f64> = affected.iter().map(|&i| self.values[i]).collect();

        let action = PaintAction {
            affected_vertices: affected.clone(),
            old_values: old_vals.clone(),
            new_values: new_vals.clone(),
            description: format!("paint_at vertex {}", hit_vertex),
        };
        self.history.push(action);

        Ok(PaintResult {
            modified_vertices: affected,
            old_values: old_vals,
            new_values: new_vals,
        })
    }

    /// Apply a paint stroke across a sequence of vertices (e.g. mouse drag).
    ///
    /// Each vertex in `vertices` is treated as the centre of a separate dab;
    /// the whole stroke is bundled into **one** undo action.
    pub fn paint_stroke(
        &mut self,
        vertices: &[usize],
        vertex_positions: &[[f64; 3]],
        _adjacency: &[Vec<usize>],
    ) -> anyhow::Result<PaintResult> {
        if vertex_positions.len() != self.vertex_count {
            anyhow::bail!(
                "vertex_positions length {} != vertex_count {}",
                vertex_positions.len(),
                self.vertex_count
            );
        }

        let radius = self.brush.radius;
        let radius_sq = radius * radius;

        // Collect the full set of affected vertices across every dab.
        let mut affected_set: Vec<usize> = Vec::new();
        // Snapshot before any modification.
        let snapshot: Vec<f64> = self.values.clone();

        for &center_idx in vertices {
            if center_idx >= self.vertex_count {
                continue;
            }
            let center = vertex_positions[center_idx];
            for (i, vpos) in vertex_positions[..self.vertex_count].iter().enumerate() {
                let d2 = dist_sq(&center, vpos);
                if d2 <= radius_sq {
                    let d = d2.sqrt();
                    let t = if radius > 0.0 { d / radius } else { 0.0 };
                    let falloff = self.brush.falloff.evaluate(t);
                    let strength = self.brush.strength * falloff;
                    self.apply_mode(i, strength);
                    if !affected_set.contains(&i) {
                        affected_set.push(i);
                    }
                }
            }

            // Mirror.
            if self.symmetry.enabled && !self.symmetry.mirror_map.is_empty() {
                let local_affected: Vec<usize> = affected_set.clone();
                // Collect mirror targets first to avoid borrow conflicts.
                let mut mirror_targets: Vec<(usize, f64)> = Vec::new();
                for &idx in &local_affected {
                    if let Some(Some(m)) = self.symmetry.mirror_map.get(idx) {
                        let m = *m;
                        if m < self.vertex_count && !affected_set.contains(&m) {
                            let center_m = vertex_positions[center_idx];
                            let d = dist(&center_m, &vertex_positions[m]);
                            let t = if radius > 0.0 { d / radius } else { 0.0 };
                            if t <= 1.0 {
                                let falloff = self.brush.falloff.evaluate(t);
                                let strength = self.brush.strength * falloff;
                                mirror_targets.push((m, strength));
                            }
                        }
                    }
                }
                for (m, strength) in mirror_targets {
                    self.apply_mode(m, strength);
                    affected_set.push(m);
                }
            }
        }

        affected_set.sort_unstable();
        affected_set.dedup();

        let old_vals: Vec<f64> = affected_set.iter().map(|&i| snapshot[i]).collect();
        let new_vals: Vec<f64> = affected_set.iter().map(|&i| self.values[i]).collect();

        let action = PaintAction {
            affected_vertices: affected_set.clone(),
            old_values: old_vals.clone(),
            new_values: new_vals.clone(),
            description: format!("paint_stroke ({} dabs)", vertices.len()),
        };
        self.history.push(action);

        Ok(PaintResult {
            modified_vertices: affected_set,
            old_values: old_vals,
            new_values: new_vals,
        })
    }

    /// Smooth the values of the given vertices by averaging with their
    /// neighbours for `iterations` rounds.
    ///
    /// Uses simple Laplacian smoothing: `v[i] = mean(v[neighbours])`.
    pub fn smooth_region(
        &mut self,
        vertices: &[usize],
        adjacency: &[Vec<usize>],
        iterations: usize,
    ) -> anyhow::Result<PaintResult> {
        if adjacency.len() != self.vertex_count {
            anyhow::bail!(
                "adjacency length {} != vertex_count {}",
                adjacency.len(),
                self.vertex_count
            );
        }

        let old_vals: Vec<f64> = vertices.iter().map(|&i| self.values[i]).collect();

        for _ in 0..iterations {
            // Build a temporary copy so reads are from the previous iteration.
            let prev = self.values.clone();
            for &i in vertices {
                if i >= self.vertex_count {
                    continue;
                }
                let neighbours = &adjacency[i];
                if neighbours.is_empty() {
                    continue;
                }
                let sum: f64 = neighbours
                    .iter()
                    .filter(|&&n| n < self.vertex_count)
                    .map(|&n| prev[n])
                    .sum();
                let count = neighbours.iter().filter(|&&n| n < self.vertex_count).count();
                if count > 0 {
                    self.values[i] = sum / count as f64;
                }
            }
        }

        let new_vals: Vec<f64> = vertices.iter().map(|&i| self.values[i]).collect();

        let action = PaintAction {
            affected_vertices: vertices.to_vec(),
            old_values: old_vals.clone(),
            new_values: new_vals.clone(),
            description: format!("smooth_region ({} iters)", iterations),
        };
        self.history.push(action);

        Ok(PaintResult {
            modified_vertices: vertices.to_vec(),
            old_values: old_vals,
            new_values: new_vals,
        })
    }

    /// Flood-fill from `start_vertex`, expanding through adjacency while the
    /// absolute difference in value relative to the start is <= `threshold`.
    ///
    /// Returns the list of filled vertex indices.  Their `selection` flag is
    /// set to `true`.
    pub fn flood_fill(
        &mut self,
        start_vertex: usize,
        adjacency: &[Vec<usize>],
        threshold: f64,
    ) -> anyhow::Result<Vec<usize>> {
        if start_vertex >= self.vertex_count {
            anyhow::bail!(
                "start_vertex {} out of range (vertex_count = {})",
                start_vertex,
                self.vertex_count
            );
        }
        if adjacency.len() != self.vertex_count {
            anyhow::bail!(
                "adjacency length {} != vertex_count {}",
                adjacency.len(),
                self.vertex_count
            );
        }

        let start_val = self.values[start_vertex];
        let mut visited = vec![false; self.vertex_count];
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        visited[start_vertex] = true;
        queue.push_back(start_vertex);

        while let Some(v) = queue.pop_front() {
            result.push(v);
            self.selection[v] = true;

            for &nb in &adjacency[v] {
                if nb < self.vertex_count && !visited[nb]
                    && (self.values[nb] - start_val).abs() <= threshold {
                        visited[nb] = true;
                        queue.push_back(nb);
                    }
            }
        }

        Ok(result)
    }

    // -- Undo / redo ---------------------------------------------------------

    /// Undo the last paint action.  Returns `Ok(true)` if an action was
    /// undone, `Ok(false)` if the undo stack is empty.
    pub fn undo(&mut self) -> anyhow::Result<bool> {
        let action = match self.history.pop_undo() {
            Some(a) => a,
            None => return Ok(false),
        };

        // Restore old values.
        for (i, &idx) in action.affected_vertices.iter().enumerate() {
            if idx < self.vertex_count {
                self.values[idx] = action.old_values[i];
            }
        }

        self.history.push_redo(action);
        Ok(true)
    }

    /// Redo the last undone action.  Returns `Ok(true)` if an action was
    /// redone, `Ok(false)` if the redo stack is empty.
    pub fn redo(&mut self) -> anyhow::Result<bool> {
        let action = match self.history.pop_redo() {
            Some(a) => a,
            None => return Ok(false),
        };

        // Re-apply new values.
        for (i, &idx) in action.affected_vertices.iter().enumerate() {
            if idx < self.vertex_count {
                self.values[idx] = action.new_values[i];
            }
        }

        self.history.push_undo(action);
        Ok(true)
    }

    // -- Mirror map ----------------------------------------------------------

    /// Build a symmetry mirror map from vertex positions.
    ///
    /// For each vertex, finds the closest vertex on the opposite side of the
    /// given `axis` within `tolerance`.  Returns `None` for vertices that have
    /// no match.
    pub fn build_mirror_map(
        positions: &[[f64; 3]],
        axis: SymmetryAxis,
        tolerance: f64,
    ) -> Vec<Option<usize>> {
        let n = positions.len();
        let mut map: Vec<Option<usize>> = vec![None; n];
        let axis_idx = match axis {
            SymmetryAxis::X => 0,
            SymmetryAxis::Y => 1,
            SymmetryAxis::Z => 2,
        };
        let tol_sq = tolerance * tolerance;

        for i in 0..n {
            if map[i].is_some() {
                continue;
            }
            let mut mirrored = positions[i];
            mirrored[axis_idx] = -mirrored[axis_idx];

            let mut best_dist_sq = f64::MAX;
            let mut best_j: Option<usize> = None;

            for (j, jpos) in positions[..n].iter().enumerate() {
                if j == i {
                    continue;
                }
                let d2 = dist_sq(&mirrored, jpos);
                if d2 < best_dist_sq {
                    best_dist_sq = d2;
                    best_j = Some(j);
                }
            }

            if best_dist_sq <= tol_sq {
                if let Some(j) = best_j {
                    map[i] = Some(j);
                    map[j] = Some(i);
                }
            }
        }

        map
    }

    // -- Export ---------------------------------------------------------------

    /// Export the current painted values as morph-target displacement deltas.
    ///
    /// In `Displace` mode the value is interpreted as a displacement magnitude
    /// along the vertex normal.  In `Weight` mode the value scales a unit
    /// displacement along the normal.  Other modes treat the value the same
    /// as weight mode.
    pub fn to_deltas(
        &self,
        vertex_positions: &[[f64; 3]],
        vertex_normals: &[[f64; 3]],
    ) -> Vec<[f64; 3]> {
        let n = self
            .vertex_count
            .min(vertex_positions.len())
            .min(vertex_normals.len());
        let mut deltas = vec![[0.0_f64; 3]; n];

        for i in 0..n {
            let w = self.values[i];
            if w.abs() < 1e-12 {
                continue;
            }
            let nx = vertex_normals[i][0];
            let ny = vertex_normals[i][1];
            let nz = vertex_normals[i][2];
            // Normalise the normal vector to be safe.
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len < 1e-12 {
                continue;
            }
            let inv_len = 1.0 / len;
            deltas[i] = [w * nx * inv_len, w * ny * inv_len, w * nz * inv_len];
        }

        deltas
    }

    // -- Internal helpers ----------------------------------------------------

    /// Apply the current paint mode to vertex `idx` with the given `strength`.
    fn apply_mode(&mut self, idx: usize, strength: f64) {
        if idx >= self.vertex_count {
            return;
        }
        match self.mode {
            PaintMode::Weight => {
                if self.brush.accumulate {
                    self.values[idx] = (self.values[idx] + strength).clamp(0.0, 1.0);
                } else {
                    // Lerp towards 1.0.
                    self.values[idx] += (1.0 - self.values[idx]) * strength;
                    self.values[idx] = self.values[idx].clamp(0.0, 1.0);
                }
            }
            PaintMode::Displace => {
                self.values[idx] += strength;
            }
            PaintMode::Smooth => {
                // Smooth is handled externally via `smooth_region`; when used
                // as a raw mode we just average towards zero.
                self.values[idx] *= 1.0 - strength;
            }
            PaintMode::Erase => {
                self.values[idx] *= 1.0 - strength;
            }
            PaintMode::Select => {
                self.selection[idx] = true;
            }
        }
    }

    /// Compute brush falloff weight for a given world-space `distance`.
    fn compute_falloff(&self, distance: f64) -> f64 {
        let radius = self.brush.radius;
        if radius <= 0.0 {
            return if distance <= 0.0 { 1.0 } else { 0.0 };
        }
        let t = (distance / radius).clamp(0.0, 1.0);
        self.brush.falloff.evaluate(t)
    }
}

// ---------------------------------------------------------------------------
// Free-standing geometry helpers
// ---------------------------------------------------------------------------

/// Squared Euclidean distance between two 3-D points.
#[inline]
fn dist_sq(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

/// Euclidean distance between two 3-D points.
#[inline]
fn dist(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    dist_sq(a, b).sqrt()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- helpers ---

    /// Build a simple 4-vertex line: 0--1--2--3 along X.
    fn line_positions() -> Vec<[f64; 3]> {
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ]
    }

    fn line_adjacency() -> Vec<Vec<usize>> {
        vec![vec![1], vec![0, 2], vec![1, 3], vec![2]]
    }

    fn up_normals(n: usize) -> Vec<[f64; 3]> {
        vec![[0.0, 1.0, 0.0]; n]
    }

    // -- Legacy compat -------------------------------------------------------

    #[test]
    fn test_legacy_new_defaults() {
        let s = new_vertex_paint_state();
        assert_eq!(s.color, [1.0, 1.0, 1.0, 1.0]);
        assert!((s.radius - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_legacy_set_color() {
        let mut s = new_vertex_paint_state();
        vp_set_color(&mut s, [0.5, 0.3, 0.1, 1.0]);
        assert!((s.color[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_legacy_set_radius() {
        let mut s = new_vertex_paint_state();
        vp_set_radius(&mut s, 0.4);
        assert!((s.radius - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_legacy_set_radius_clamps_negative() {
        let mut s = new_vertex_paint_state();
        vp_set_radius(&mut s, -1.0);
        assert!((s.radius).abs() < 1e-6);
    }

    #[test]
    fn test_legacy_blend_mode_mix() {
        assert_eq!(vp_blend_mode_name(0), "Mix");
    }

    #[test]
    fn test_legacy_blend_mode_add() {
        assert_eq!(vp_blend_mode_name(1), "Add");
    }

    #[test]
    fn test_legacy_blend_mode_multiply() {
        assert_eq!(vp_blend_mode_name(3), "Multiply");
    }

    #[test]
    fn test_legacy_blend_mode_custom() {
        assert_eq!(vp_blend_mode_name(99), "Custom");
    }

    #[test]
    fn test_legacy_active_layer_default() {
        let s = new_vertex_paint_state();
        assert_eq!(s.active_layer, "Col");
    }

    // -- FalloffCurve --------------------------------------------------------

    #[test]
    fn test_falloff_linear_endpoints() {
        let f = FalloffCurve::Linear;
        assert!((f.evaluate(0.0) - 1.0).abs() < 1e-12);
        assert!((f.evaluate(1.0)).abs() < 1e-12);
    }

    #[test]
    fn test_falloff_smooth_endpoints() {
        let f = FalloffCurve::Smooth;
        assert!((f.evaluate(0.0) - 1.0).abs() < 1e-12);
        assert!((f.evaluate(1.0)).abs() < 1e-12);
    }

    #[test]
    fn test_falloff_flat() {
        let f = FalloffCurve::Flat;
        assert!((f.evaluate(0.0) - 1.0).abs() < 1e-12);
        assert!((f.evaluate(0.5) - 1.0).abs() < 1e-12);
        assert!((f.evaluate(1.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_falloff_sharp() {
        let f = FalloffCurve::Sharp;
        assert!((f.evaluate(0.0) - 1.0).abs() < 1e-12);
        assert!((f.evaluate(1.0)).abs() < 1e-12);
        // Mid-point should be less than linear mid-point.
        assert!(f.evaluate(0.5) < 0.5);
    }

    #[test]
    fn test_falloff_custom() {
        let f = FalloffCurve::Custom(2.0);
        assert!((f.evaluate(0.0) - 1.0).abs() < 1e-12);
        assert!((f.evaluate(1.0)).abs() < 1e-12);
        // (1 - 0.5)^2 = 0.25
        assert!((f.evaluate(0.5) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_falloff_clamped() {
        let f = FalloffCurve::Linear;
        assert!((f.evaluate(-0.5) - 1.0).abs() < 1e-12);
        assert!((f.evaluate(2.0)).abs() < 1e-12);
    }

    // -- PaintHistory --------------------------------------------------------

    #[test]
    fn test_history_push_and_undo() {
        let mut h = PaintHistory::new(4);
        assert_eq!(h.undo_count(), 0);
        h.push(PaintAction {
            affected_vertices: vec![0],
            old_values: vec![0.0],
            new_values: vec![1.0],
            description: "a".into(),
        });
        assert_eq!(h.undo_count(), 1);
        let a = h.pop_undo();
        assert!(a.is_some());
        assert_eq!(h.undo_count(), 0);
    }

    #[test]
    fn test_history_max_depth() {
        let mut h = PaintHistory::new(2);
        for i in 0..5 {
            h.push(PaintAction {
                affected_vertices: vec![i],
                old_values: vec![0.0],
                new_values: vec![1.0],
                description: format!("action {}", i),
            });
        }
        assert_eq!(h.undo_count(), 2);
    }

    #[test]
    fn test_history_redo_cleared_on_push() {
        let mut h = PaintHistory::new(8);
        h.push(PaintAction {
            affected_vertices: vec![0],
            old_values: vec![0.0],
            new_values: vec![1.0],
            description: "a".into(),
        });
        let a = h.pop_undo();
        assert!(a.is_some());
        if let Some(action) = a {
            h.push_redo(action);
        }
        assert_eq!(h.redo_count(), 1);
        // New push should clear redo.
        h.push(PaintAction {
            affected_vertices: vec![1],
            old_values: vec![0.0],
            new_values: vec![1.0],
            description: "b".into(),
        });
        assert_eq!(h.redo_count(), 0);
    }

    // -- VertexPaintState basics ---------------------------------------------

    #[test]
    fn test_new_state() {
        let s = VertexPaintState::new(100);
        assert_eq!(s.vertex_count(), 100);
        assert_eq!(s.values().len(), 100);
        assert_eq!(s.selection().len(), 100);
        assert!(s.values().iter().all(|&v| v == 0.0));
        assert!(s.selection().iter().all(|&v| !v));
    }

    #[test]
    fn test_set_mode() {
        let mut s = VertexPaintState::new(4);
        s.set_mode(PaintMode::Displace);
        assert_eq!(s.mode(), PaintMode::Displace);
    }

    #[test]
    fn test_clear() {
        let mut s = VertexPaintState::new(4);
        s.values[0] = 1.0;
        s.selection[1] = true;
        s.clear();
        assert!(s.values().iter().all(|&v| v == 0.0));
        assert!(s.selection().iter().all(|&v| !v));
    }

    // -- paint_at ------------------------------------------------------------

    #[test]
    fn test_paint_at_basic() {
        let pos = line_positions();
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        s.set_brush(PaintBrushConfig {
            radius: 1.5,
            strength: 1.0,
            falloff: FalloffCurve::Flat,
            accumulate: true,
        });
        s.set_mode(PaintMode::Weight);

        let res = s.paint_at(0, &pos, &adj);
        assert!(res.is_ok());
        let r = res.expect("paint_at failed in test");
        // Vertices 0 and 1 are within radius 1.5 of vertex 0.
        assert!(r.modified_vertices.contains(&0));
        assert!(r.modified_vertices.contains(&1));
        // Values should have been bumped.
        assert!(s.values()[0] > 0.0);
        assert!(s.values()[1] > 0.0);
    }

    #[test]
    fn test_paint_at_out_of_range() {
        let pos = line_positions();
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        let res = s.paint_at(99, &pos, &adj);
        assert!(res.is_err());
    }

    #[test]
    fn test_paint_at_records_undo() {
        let pos = line_positions();
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        s.set_brush(PaintBrushConfig {
            radius: 10.0,
            strength: 0.5,
            falloff: FalloffCurve::Flat,
            accumulate: true,
        });
        let _ = s.paint_at(0, &pos, &adj);
        assert_eq!(s.history().undo_count(), 1);
    }

    // -- undo / redo ---------------------------------------------------------

    #[test]
    fn test_undo_redo() {
        let pos = line_positions();
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        s.set_brush(PaintBrushConfig {
            radius: 10.0,
            strength: 0.5,
            falloff: FalloffCurve::Flat,
            accumulate: true,
        });
        let _ = s.paint_at(0, &pos, &adj);
        let val_after = s.values()[0];
        assert!(val_after > 0.0);

        // Undo.
        let undone = s.undo();
        assert!(undone.is_ok());
        assert!(matches!(undone, Ok(true)));
        assert!((s.values()[0]).abs() < 1e-12);

        // Redo.
        let redone = s.redo();
        assert!(redone.is_ok());
        assert!(matches!(redone, Ok(true)));
        assert!((s.values()[0] - val_after).abs() < 1e-12);
    }

    #[test]
    fn test_undo_empty() {
        let mut s = VertexPaintState::new(4);
        let r = s.undo();
        assert!(matches!(r, Ok(false)));
    }

    #[test]
    fn test_redo_empty() {
        let mut s = VertexPaintState::new(4);
        let r = s.redo();
        assert!(matches!(r, Ok(false)));
    }

    // -- smooth_region -------------------------------------------------------

    #[test]
    fn test_smooth_region() {
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        // Set up a spike at vertex 1.
        s.values[1] = 1.0;

        let res = s.smooth_region(&[1], &adj, 3);
        assert!(res.is_ok());
        // After smoothing, vertex 1 should be lower than 1.0.
        assert!(s.values()[1] < 1.0);
    }

    #[test]
    fn test_smooth_region_adjacency_mismatch() {
        let mut s = VertexPaintState::new(4);
        let bad_adj: Vec<Vec<usize>> = vec![vec![1], vec![0]]; // wrong length
        let res = s.smooth_region(&[0], &bad_adj, 1);
        assert!(res.is_err());
    }

    // -- flood_fill ----------------------------------------------------------

    #[test]
    fn test_flood_fill_uniform() {
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        // All values are 0.0 => everything reachable.
        let res = s.flood_fill(0, &adj, 0.0);
        assert!(res.is_ok());
        if let Ok(filled) = res {
            assert_eq!(filled.len(), 4);
        }
    }

    #[test]
    fn test_flood_fill_with_barrier() {
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        // Create a barrier at vertex 2.
        s.values[2] = 5.0;
        let res = s.flood_fill(0, &adj, 0.1);
        assert!(res.is_ok());
        if let Ok(filled) = res {
            // Should stop before vertex 2 (and therefore not reach 3 either).
            assert!(!filled.contains(&2));
            assert!(!filled.contains(&3));
        }
    }

    #[test]
    fn test_flood_fill_out_of_range() {
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        let res = s.flood_fill(99, &adj, 0.0);
        assert!(res.is_err());
    }

    // -- paint_stroke --------------------------------------------------------

    #[test]
    fn test_paint_stroke() {
        let pos = line_positions();
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        s.set_brush(PaintBrushConfig {
            radius: 0.5,
            strength: 0.3,
            falloff: FalloffCurve::Flat,
            accumulate: true,
        });
        s.set_mode(PaintMode::Displace);

        let res = s.paint_stroke(&[0, 1, 2], &pos, &adj);
        assert!(res.is_ok());
        if let Ok(r) = res {
            assert!(!r.modified_vertices.is_empty());
        }
        assert_eq!(s.history().undo_count(), 1); // one composite action
    }

    // -- build_mirror_map ----------------------------------------------------

    #[test]
    fn test_build_mirror_map_x() {
        let positions = vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let map = VertexPaintState::build_mirror_map(&positions, SymmetryAxis::X, 0.01);
        assert_eq!(map[0], Some(1));
        assert_eq!(map[1], Some(0));
    }

    #[test]
    fn test_build_mirror_map_y() {
        let positions = vec![[0.0, -2.0, 0.0], [0.0, 2.0, 0.0]];
        let map = VertexPaintState::build_mirror_map(&positions, SymmetryAxis::Y, 0.01);
        assert_eq!(map[0], Some(1));
        assert_eq!(map[1], Some(0));
    }

    // -- to_deltas -----------------------------------------------------------

    #[test]
    fn test_to_deltas_basic() {
        let mut s = VertexPaintState::new(3);
        s.values[0] = 0.0;
        s.values[1] = 1.0;
        s.values[2] = -0.5;

        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let normals = up_normals(3);
        let deltas = s.to_deltas(&positions, &normals);
        assert_eq!(deltas.len(), 3);
        // Vertex 0: weight 0 => delta [0,0,0].
        assert!((deltas[0][0]).abs() < 1e-12);
        assert!((deltas[0][1]).abs() < 1e-12);
        // Vertex 1: weight 1, normal [0,1,0] => delta [0, 1, 0].
        assert!((deltas[1][1] - 1.0).abs() < 1e-12);
        // Vertex 2: weight -0.5, normal [0,1,0] => delta [0, -0.5, 0].
        assert!((deltas[2][1] + 0.5).abs() < 1e-12);
    }

    #[test]
    fn test_to_deltas_zero_normal() {
        let mut s = VertexPaintState::new(1);
        s.values[0] = 1.0;
        let positions = vec![[0.0, 0.0, 0.0]];
        let normals = vec![[0.0, 0.0, 0.0]];
        let deltas = s.to_deltas(&positions, &normals);
        // Zero normal => zero delta.
        assert!((deltas[0][0]).abs() < 1e-12);
    }

    // -- compute_falloff (private, tested through paint_at indirectly) -------

    #[test]
    fn test_compute_falloff_direct() {
        let s = VertexPaintState::new(1);
        assert!((s.compute_falloff(0.0) - 1.0).abs() < 1e-6);
    }

    // -- erase mode ----------------------------------------------------------

    #[test]
    fn test_erase_mode() {
        let pos = line_positions();
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        s.values[0] = 1.0;
        s.values[1] = 1.0;
        s.set_mode(PaintMode::Erase);
        s.set_brush(PaintBrushConfig {
            radius: 1.5,
            strength: 0.5,
            falloff: FalloffCurve::Flat,
            accumulate: false,
        });
        let _ = s.paint_at(0, &pos, &adj);
        // Values should have decreased.
        assert!(s.values()[0] < 1.0);
    }

    // -- select mode ---------------------------------------------------------

    #[test]
    fn test_select_mode() {
        let pos = line_positions();
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        s.set_mode(PaintMode::Select);
        s.set_brush(PaintBrushConfig {
            radius: 1.5,
            strength: 1.0,
            falloff: FalloffCurve::Flat,
            accumulate: false,
        });
        let _ = s.paint_at(0, &pos, &adj);
        assert!(s.selection()[0]);
        assert!(s.selection()[1]);
    }

    // -- symmetry painting ---------------------------------------------------

    #[test]
    fn test_symmetry_paint() {
        let positions = vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let adj = vec![vec![1], vec![0]];
        let mirror_map =
            VertexPaintState::build_mirror_map(&positions, SymmetryAxis::X, 0.01);

        let mut s = VertexPaintState::new(2);
        s.set_symmetry(SymmetryConfig {
            enabled: true,
            axis: SymmetryAxis::X,
            tolerance: 0.01,
            mirror_map,
        });
        s.set_brush(PaintBrushConfig {
            radius: 0.5,
            strength: 1.0,
            falloff: FalloffCurve::Flat,
            accumulate: true,
        });

        let _ = s.paint_at(0, &positions, &adj);
        // Both vertex 0 and its mirror (vertex 1) should be affected.
        assert!(s.values()[0] > 0.0);
        assert!(s.values()[1] > 0.0);
    }

    // -- reset ---------------------------------------------------------------

    #[test]
    fn test_reset_clears_history() {
        let pos = line_positions();
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        s.set_brush(PaintBrushConfig {
            radius: 10.0,
            strength: 0.5,
            falloff: FalloffCurve::Flat,
            accumulate: true,
        });
        let _ = s.paint_at(0, &pos, &adj);
        assert!(s.history().undo_count() > 0);
        s.reset();
        assert_eq!(s.history().undo_count(), 0);
        assert!(s.values().iter().all(|&v| v == 0.0));
    }

    // -- PaintBrushConfig default -------------------------------------------

    #[test]
    fn test_brush_default() {
        let b = PaintBrushConfig::default();
        assert!((b.radius - 0.05).abs() < 1e-12);
        assert!((b.strength - 0.5).abs() < 1e-12);
        assert!(!b.accumulate);
    }

    // -- edge cases ----------------------------------------------------------

    #[test]
    fn test_zero_vertex_state() {
        let s = VertexPaintState::new(0);
        assert_eq!(s.vertex_count(), 0);
        assert!(s.values().is_empty());
    }

    #[test]
    fn test_paint_stroke_skips_out_of_range() {
        let pos = line_positions();
        let adj = line_adjacency();
        let mut s = VertexPaintState::new(4);
        s.set_brush(PaintBrushConfig {
            radius: 0.5,
            strength: 0.3,
            falloff: FalloffCurve::Flat,
            accumulate: true,
        });
        // Include an out-of-range index -- should be silently skipped.
        let res = s.paint_stroke(&[0, 99], &pos, &adj);
        assert!(res.is_ok());
    }
}
