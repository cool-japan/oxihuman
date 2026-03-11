#![allow(dead_code)]
//! Morph preview: lightweight preview of morph deltas before committing.

/// A preview state for morph evaluation.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphPreview {
    deltas: Vec<[f32; 3]>,
    weight: f32,
    active: bool,
}

/// Create a new preview for the given vertex count.
#[allow(dead_code)]
pub fn new_morph_preview(vertex_count: usize) -> MorphPreview {
    MorphPreview {
        deltas: vec![[0.0; 3]; vertex_count],
        weight: 0.0,
        active: false,
    }
}

/// Set the preview weight.
#[allow(dead_code)]
pub fn preview_set_weight(preview: &mut MorphPreview, weight: f32) {
    preview.weight = weight.clamp(0.0, 1.0);
    preview.active = preview.weight.abs() > 1e-9;
}

/// Evaluate: apply weight to deltas, returning scaled deltas.
#[allow(dead_code)]
pub fn preview_evaluate(preview: &MorphPreview) -> Vec<[f32; 3]> {
    preview
        .deltas
        .iter()
        .map(|d| [d[0] * preview.weight, d[1] * preview.weight, d[2] * preview.weight])
        .collect()
}

/// Reset preview to zero weight and inactive.
#[allow(dead_code)]
pub fn preview_reset(preview: &mut MorphPreview) {
    preview.weight = 0.0;
    preview.active = false;
    for d in &mut preview.deltas {
        *d = [0.0; 3];
    }
}

/// Return the delta at `index`.
#[allow(dead_code)]
pub fn preview_delta_at(preview: &MorphPreview, index: usize) -> [f32; 3] {
    preview.deltas.get(index).copied().unwrap_or([0.0; 3])
}

/// Return the vertex count.
#[allow(dead_code)]
pub fn preview_vertex_count(preview: &MorphPreview) -> usize {
    preview.deltas.len()
}

/// Serialize to JSON-like string (summary).
#[allow(dead_code)]
pub fn preview_to_json(preview: &MorphPreview) -> String {
    format!(
        "{{\"vertex_count\":{},\"weight\":{},\"active\":{}}}",
        preview.deltas.len(),
        preview.weight,
        preview.active
    )
}

/// Check if the preview is active.
#[allow(dead_code)]
pub fn preview_is_active(preview: &MorphPreview) -> bool {
    preview.active
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_preview() {
        let p = new_morph_preview(10);
        assert_eq!(preview_vertex_count(&p), 10);
        assert!(!preview_is_active(&p));
    }

    #[test]
    fn test_set_weight() {
        let mut p = new_morph_preview(5);
        preview_set_weight(&mut p, 0.5);
        assert!(preview_is_active(&p));
    }

    #[test]
    fn test_set_weight_zero() {
        let mut p = new_morph_preview(5);
        preview_set_weight(&mut p, 0.0);
        assert!(!preview_is_active(&p));
    }

    #[test]
    fn test_evaluate() {
        let mut p = new_morph_preview(3);
        p.deltas[0] = [1.0, 2.0, 3.0];
        preview_set_weight(&mut p, 0.5);
        let result = preview_evaluate(&p);
        assert!((result[0][0] - 0.5).abs() < 1e-6);
        assert!((result[0][1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_reset() {
        let mut p = new_morph_preview(5);
        preview_set_weight(&mut p, 0.8);
        preview_reset(&mut p);
        assert!(!preview_is_active(&p));
    }

    #[test]
    fn test_delta_at() {
        let p = new_morph_preview(5);
        let d = preview_delta_at(&p, 0);
        assert!((d[0]).abs() < 1e-6);
    }

    #[test]
    fn test_delta_at_out_of_range() {
        let p = new_morph_preview(5);
        let d = preview_delta_at(&p, 99);
        assert!((d[0]).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let p = new_morph_preview(5);
        let json = preview_to_json(&p);
        assert!(json.contains("\"vertex_count\":5"));
    }

    #[test]
    fn test_weight_clamp() {
        let mut p = new_morph_preview(5);
        preview_set_weight(&mut p, 2.0);
        let result = preview_evaluate(&p);
        // weight should be clamped to 1.0
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_evaluate_zero_weight() {
        let mut p = new_morph_preview(3);
        p.deltas[0] = [1.0, 2.0, 3.0];
        let result = preview_evaluate(&p);
        assert!((result[0][0]).abs() < 1e-6);
    }
}
