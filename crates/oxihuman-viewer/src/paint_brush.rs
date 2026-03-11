// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Paint brush tool for vertex color and weight painting.
//!
//! Manages brush state, stroke recording, and per-vertex falloff
//! computation without any external dependencies.

#![allow(dead_code)]

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Operational mode of the paint brush.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BrushMode {
    /// Add paint value to existing vertex data.
    Paint,
    /// Average neighboring vertex values (smooth).
    Smooth,
    /// Subtract paint value from existing vertex data.
    Erase,
    /// Apply a Gaussian-style blur to vertex data.
    Blur,
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// Tuning parameters for the paint brush.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrushConfig {
    /// Brush radius in world units. Default: `0.1`.
    pub radius: f32,
    /// Paint strength in `[0, 1]`. Default: `0.5`.
    pub strength: f32,
    /// Falloff exponent. Higher values give harder edges. Default: `2.0`.
    pub falloff_exponent: f32,
    /// Whether to use symmetry when painting. Default: `false`.
    pub use_symmetry: bool,
    /// Active brush mode. Default: `BrushMode::Paint`.
    pub mode: BrushMode,
}

impl Default for BrushConfig {
    fn default() -> Self {
        Self {
            radius: 0.1,
            strength: 0.5,
            falloff_exponent: 2.0,
            use_symmetry: false,
            mode: BrushMode::Paint,
        }
    }
}

/// A single recorded brush stroke consisting of screen/world positions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BrushStroke {
    /// World-space positions of each sample point along the stroke.
    pub points: Vec<[f32; 3]>,
    /// Brush radius at the time this stroke was begun.
    pub radius: f32,
    /// Brush strength at the time this stroke was begun.
    pub strength: f32,
    /// Mode used for this stroke.
    pub mode: BrushMode,
}

/// State for the interactive paint brush tool.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PaintBrush {
    /// Current brush configuration.
    pub config: BrushConfig,
    /// History of completed strokes.
    pub strokes: Vec<BrushStroke>,
    /// The in-progress stroke, if any.
    pub active_stroke: Option<BrushStroke>,
}

// ── Type aliases ──────────────────────────────────────────────────────────────

/// A list of (vertex index, weight) pairs for affected vertices.
pub type AffectedVertices = Vec<(usize, f32)>;

// ── Functions ─────────────────────────────────────────────────────────────────

/// Return a default [`BrushConfig`].
#[allow(dead_code)]
pub fn default_brush_config() -> BrushConfig {
    BrushConfig::default()
}

/// Create a new [`PaintBrush`] with the given config.
#[allow(dead_code)]
pub fn new_paint_brush(config: BrushConfig) -> PaintBrush {
    PaintBrush {
        config,
        strokes: Vec::new(),
        active_stroke: None,
    }
}

/// Begin a new paint stroke at `point`.
///
/// Replaces any in-progress stroke.
#[allow(dead_code)]
pub fn begin_stroke(brush: &mut PaintBrush, point: [f32; 3]) {
    let stroke = BrushStroke {
        points: vec![point],
        radius: brush.config.radius,
        strength: brush.config.strength,
        mode: brush.config.mode,
    };
    brush.active_stroke = Some(stroke);
}

/// Append `point` to the active stroke, if one exists.
#[allow(dead_code)]
pub fn continue_stroke(brush: &mut PaintBrush, point: [f32; 3]) {
    if let Some(stroke) = brush.active_stroke.as_mut() {
        stroke.points.push(point);
    }
}

/// Finalize the active stroke and commit it to the history.
#[allow(dead_code)]
pub fn end_stroke(brush: &mut PaintBrush) {
    if let Some(stroke) = brush.active_stroke.take() {
        brush.strokes.push(stroke);
    }
}

/// Find all vertex indices within `radius` of `center`, returning
/// `(index, falloff_weight)` pairs.
///
/// `positions` — flat `[x, y, z, ...]` vertex positions.
#[allow(dead_code)]
pub fn brush_affected_vertices(
    positions: &[f32],
    center: [f32; 3],
    radius: f32,
    falloff_exponent: f32,
) -> AffectedVertices {
    let vc = positions.len() / 3;
    let mut result = Vec::new();
    for i in 0..vc {
        let vx = positions[i * 3];
        let vy = positions[i * 3 + 1];
        let vz = positions[i * 3 + 2];
        let dx = vx - center[0];
        let dy = vy - center[1];
        let dz = vz - center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        if dist <= radius {
            let w = brush_falloff(dist, radius, falloff_exponent);
            result.push((i, w));
        }
    }
    result
}

/// Compute a distance-based falloff weight in `[0, 1]`.
///
/// `dist` must be ≤ `radius`.  The returned weight is `1` at the center
/// and `0` at the edge, shaped by `exponent`.
#[allow(dead_code)]
pub fn brush_falloff(dist: f32, radius: f32, exponent: f32) -> f32 {
    if radius <= 0.0 {
        return 0.0;
    }
    let t = (dist / radius).clamp(0.0, 1.0);
    (1.0 - t).powf(exponent)
}

/// Apply a brush stroke to a mutable slice of per-vertex values.
///
/// Each value is clamped to `[0, 1]` after the operation.
#[allow(dead_code)]
pub fn apply_brush_stroke(
    values: &mut [f32],
    affected: &AffectedVertices,
    strength: f32,
    mode: BrushMode,
) {
    for &(idx, weight) in affected {
        if idx >= values.len() {
            continue;
        }
        let delta = strength * weight;
        match mode {
            BrushMode::Paint => values[idx] = (values[idx] + delta).clamp(0.0, 1.0),
            BrushMode::Erase => values[idx] = (values[idx] - delta).clamp(0.0, 1.0),
            BrushMode::Smooth | BrushMode::Blur => {
                values[idx] = (values[idx] * (1.0 - delta) + 0.5 * delta).clamp(0.0, 1.0)
            }
        }
    }
}

/// Return the number of completed strokes.
#[allow(dead_code)]
pub fn stroke_count(brush: &PaintBrush) -> usize {
    brush.strokes.len()
}

/// Set the brush mode.
#[allow(dead_code)]
pub fn set_brush_mode(brush: &mut PaintBrush, mode: BrushMode) {
    brush.config.mode = mode;
}

/// Return the current brush radius.
#[allow(dead_code)]
pub fn brush_radius(brush: &PaintBrush) -> f32 {
    brush.config.radius
}

/// Set the brush radius (clamped to a minimum of `0.0`).
#[allow(dead_code)]
pub fn set_brush_radius(brush: &mut PaintBrush, radius: f32) {
    brush.config.radius = radius.max(0.0);
}

/// Return the current brush strength.
#[allow(dead_code)]
pub fn brush_strength(brush: &PaintBrush) -> f32 {
    brush.config.strength
}

/// Serialize the brush state to a compact JSON string.
#[allow(dead_code)]
pub fn paint_brush_to_json(brush: &PaintBrush) -> String {
    format!(
        r#"{{"radius":{:.4},"strength":{:.4},"falloff_exponent":{:.4},"mode":"{:?}","stroke_count":{}}}"#,
        brush.config.radius,
        brush.config.strength,
        brush.config.falloff_exponent,
        brush.config.mode,
        brush.strokes.len()
    )
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_brush_config() {
        let cfg = default_brush_config();
        assert!((cfg.radius - 0.1).abs() < 1e-6);
        assert!((cfg.strength - 0.5).abs() < 1e-6);
        assert_eq!(cfg.mode, BrushMode::Paint);
        assert!(!cfg.use_symmetry);
    }

    #[test]
    fn test_new_paint_brush() {
        let brush = new_paint_brush(default_brush_config());
        assert!(brush.strokes.is_empty());
        assert!(brush.active_stroke.is_none());
    }

    #[test]
    fn test_begin_stroke() {
        let mut brush = new_paint_brush(default_brush_config());
        begin_stroke(&mut brush, [0.0, 0.0, 0.0]);
        assert!(brush.active_stroke.is_some());
        let pts = &brush.active_stroke.as_ref().unwrap().points;
        assert_eq!(pts.len(), 1);
    }

    #[test]
    fn test_continue_stroke() {
        let mut brush = new_paint_brush(default_brush_config());
        begin_stroke(&mut brush, [0.0, 0.0, 0.0]);
        continue_stroke(&mut brush, [1.0, 0.0, 0.0]);
        continue_stroke(&mut brush, [2.0, 0.0, 0.0]);
        assert_eq!(brush.active_stroke.as_ref().unwrap().points.len(), 3);
    }

    #[test]
    fn test_end_stroke_commits_to_history() {
        let mut brush = new_paint_brush(default_brush_config());
        begin_stroke(&mut brush, [0.0, 0.0, 0.0]);
        end_stroke(&mut brush);
        assert_eq!(stroke_count(&brush), 1);
        assert!(brush.active_stroke.is_none());
    }

    #[test]
    fn test_end_stroke_no_active_is_noop() {
        let mut brush = new_paint_brush(default_brush_config());
        end_stroke(&mut brush);
        assert_eq!(stroke_count(&brush), 0);
    }

    #[test]
    fn test_brush_falloff_center() {
        let w = brush_falloff(0.0, 1.0, 2.0);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_brush_falloff_edge() {
        let w = brush_falloff(1.0, 1.0, 2.0);
        assert!(w < 1e-6);
    }

    #[test]
    fn test_brush_falloff_zero_radius() {
        let w = brush_falloff(0.0, 0.0, 2.0);
        assert!((w).abs() < 1e-6);
    }

    #[test]
    fn test_brush_affected_vertices() {
        let pos = vec![0.0f32, 0.0, 0.0, 5.0, 0.0, 0.0];
        let affected = brush_affected_vertices(&pos, [0.0, 0.0, 0.0], 1.0, 2.0);
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].0, 0);
    }

    #[test]
    fn test_apply_brush_stroke_paint() {
        let mut values = vec![0.0f32; 3];
        let affected = vec![(0, 1.0), (1, 0.5)];
        apply_brush_stroke(&mut values, &affected, 0.5, BrushMode::Paint);
        assert!((values[0] - 0.5).abs() < 1e-6);
        assert!((values[1] - 0.25).abs() < 1e-6);
        assert!((values[2]).abs() < 1e-6);
    }

    #[test]
    fn test_apply_brush_stroke_erase() {
        let mut values = vec![1.0f32; 2];
        let affected = vec![(0, 1.0)];
        apply_brush_stroke(&mut values, &affected, 0.5, BrushMode::Erase);
        assert!((values[0] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_apply_brush_stroke_clamps() {
        let mut values = vec![0.9f32];
        let affected = vec![(0, 1.0)];
        apply_brush_stroke(&mut values, &affected, 1.0, BrushMode::Paint);
        assert!((values[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_brush_mode() {
        let mut brush = new_paint_brush(default_brush_config());
        set_brush_mode(&mut brush, BrushMode::Smooth);
        assert_eq!(brush.config.mode, BrushMode::Smooth);
    }

    #[test]
    fn test_set_brush_radius() {
        let mut brush = new_paint_brush(default_brush_config());
        set_brush_radius(&mut brush, 0.5);
        assert!((brush_radius(&brush) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_brush_radius_clamp() {
        let mut brush = new_paint_brush(default_brush_config());
        set_brush_radius(&mut brush, -1.0);
        assert!((brush_radius(&brush)).abs() < 1e-6);
    }

    #[test]
    fn test_brush_strength_getter() {
        let brush = new_paint_brush(default_brush_config());
        assert!((brush_strength(&brush) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_paint_brush_to_json() {
        let brush = new_paint_brush(default_brush_config());
        let json = paint_brush_to_json(&brush);
        assert!(json.contains("radius"));
        assert!(json.contains("strength"));
        assert!(json.contains("stroke_count"));
    }
}
