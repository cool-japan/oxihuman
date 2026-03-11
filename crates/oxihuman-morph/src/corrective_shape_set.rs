#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Set of corrective shapes triggered by primary morphs.

/// A single corrective shape definition.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CorrectiveShapeDef {
    pub name: String,
    /// Per-vertex position offsets `[dx, dy, dz]`.
    pub offsets: Vec<[f32; 3]>,
    /// Index into the driver weight array.
    pub driver_index: usize,
    /// Driver weight threshold at which this shape activates.
    pub threshold: f32,
}

/// A collection of corrective shapes.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct CorrectiveShapeSet {
    pub shapes: Vec<CorrectiveShapeDef>,
}

/// Create an empty `CorrectiveShapeSet`.
#[allow(dead_code)]
pub fn new_corrective_shape_set() -> CorrectiveShapeSet {
    CorrectiveShapeSet::default()
}

/// Add a corrective shape to the set.
#[allow(dead_code)]
pub fn add_corrective(
    css: &mut CorrectiveShapeSet,
    name: &str,
    driver: usize,
    threshold: f32,
    offsets: Vec<[f32; 3]>,
) {
    css.shapes.push(CorrectiveShapeDef {
        name: name.to_string(),
        offsets,
        driver_index: driver,
        threshold: threshold.clamp(0.0, 1.0),
    });
}

/// Evaluate active correctives given a driver-weight slice.
///
/// Returns a flat vector of summed per-vertex offsets (x, y, z interleaved).
/// The vertex count is taken from the first active shape that has offsets; all
/// shapes are assumed to share the same vertex count.
#[allow(dead_code)]
pub fn evaluate_correctives(css: &CorrectiveShapeSet, weights: &[f32]) -> Vec<f32> {
    let n_verts = css
        .shapes
        .iter()
        .map(|s| s.offsets.len())
        .max()
        .unwrap_or(0);
    let mut result = vec![0.0_f32; n_verts * 3];
    for shape in &css.shapes {
        let driver_w = weights.get(shape.driver_index).copied().unwrap_or(0.0);
        if driver_w < shape.threshold {
            continue;
        }
        let blend = ((driver_w - shape.threshold) / (1.0 - shape.threshold + 1e-9)).clamp(0.0, 1.0);
        for (i, off) in shape.offsets.iter().enumerate() {
            result[i * 3] += off[0] * blend;
            result[i * 3 + 1] += off[1] * blend;
            result[i * 3 + 2] += off[2] * blend;
        }
    }
    result
}

/// Return the number of shapes in the set.
#[allow(dead_code)]
pub fn corrective_count(css: &CorrectiveShapeSet) -> usize {
    css.shapes.len()
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_set_is_empty() {
        let css = new_corrective_shape_set();
        assert_eq!(corrective_count(&css), 0);
    }

    #[test]
    fn add_corrective_increments_count() {
        let mut css = new_corrective_shape_set();
        add_corrective(&mut css, "jaw_open", 0, 0.5, vec![[0.1, 0.0, 0.0]]);
        assert_eq!(corrective_count(&css), 1);
    }

    #[test]
    fn evaluate_empty_returns_empty() {
        let css = new_corrective_shape_set();
        let result = evaluate_correctives(&css, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn corrective_not_triggered_below_threshold() {
        let mut css = new_corrective_shape_set();
        add_corrective(&mut css, "s", 0, 0.8, vec![[1.0, 0.0, 0.0]]);
        let result = evaluate_correctives(&css, &[0.5]);
        assert!(result.iter().all(|&v| v.abs() < 1e-6));
    }

    #[test]
    fn corrective_triggered_above_threshold() {
        let mut css = new_corrective_shape_set();
        add_corrective(&mut css, "s", 0, 0.5, vec![[1.0, 0.0, 0.0]]);
        let result = evaluate_correctives(&css, &[1.0]);
        assert!(result[0] > 0.0);
    }

    #[test]
    fn threshold_clamped_to_range() {
        let mut css = new_corrective_shape_set();
        add_corrective(&mut css, "s", 0, 2.0, vec![[1.0, 0.0, 0.0]]);
        assert!((css.shapes[0].threshold - 1.0).abs() < 1e-6);
    }

    #[test]
    fn driver_index_out_of_range_no_panic() {
        let mut css = new_corrective_shape_set();
        add_corrective(&mut css, "s", 99, 0.0, vec![[1.0, 0.0, 0.0]]);
        let result = evaluate_correctives(&css, &[1.0]);
        // driver weight defaults to 0.0 so it should not activate above threshold=0.0 boundary case
        let _ = result;
    }

    #[test]
    fn two_shapes_sum_offsets() {
        let mut css = new_corrective_shape_set();
        add_corrective(&mut css, "a", 0, 0.0, vec![[0.3, 0.0, 0.0]]);
        add_corrective(&mut css, "b", 0, 0.0, vec![[0.2, 0.0, 0.0]]);
        let result = evaluate_correctives(&css, &[1.0]);
        // Both activated fully at weight 1.0
        assert!(result[0] > 0.4);
    }

    #[test]
    fn corrective_count_multiple() {
        let mut css = new_corrective_shape_set();
        for i in 0..5usize {
            add_corrective(&mut css, "s", i, 0.5, vec![]);
        }
        assert_eq!(corrective_count(&css), 5);
    }

    #[test]
    fn evaluate_returns_correct_length() {
        let mut css = new_corrective_shape_set();
        add_corrective(&mut css, "s", 0, 0.0, vec![[0.0; 3]; 4]);
        let result = evaluate_correctives(&css, &[1.0]);
        assert_eq!(result.len(), 12); // 4 verts × 3
    }
}
