// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

//! Blend target (shape key target) export utilities.

/// A single blend target with vertex deltas.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlendTargetExport {
    pub name: String,
    pub deltas: Vec<[f32; 3]>,
    pub weight: f32,
}

/// Create a new blend target.
#[allow(dead_code)]
pub fn new_blend_target(name: &str, vertex_count: usize) -> BlendTargetExport {
    BlendTargetExport {
        name: name.to_string(),
        deltas: vec![[0.0; 3]; vertex_count],
        weight: 0.0,
    }
}

/// Set delta for a vertex.
#[allow(dead_code)]
pub fn set_delta(target: &mut BlendTargetExport, idx: usize, delta: [f32; 3]) {
    if idx < target.deltas.len() {
        target.deltas[idx] = delta;
    }
}

/// Get delta at vertex.
#[allow(dead_code)]
pub fn get_delta(target: &BlendTargetExport, idx: usize) -> Option<[f32; 3]> {
    target.deltas.get(idx).copied()
}

/// Vertex count.
#[allow(dead_code)]
pub fn bt_vertex_count(target: &BlendTargetExport) -> usize {
    target.deltas.len()
}

/// Set weight.
#[allow(dead_code)]
pub fn set_weight(target: &mut BlendTargetExport, w: f32) {
    target.weight = w.clamp(0.0, 1.0);
}

/// Max delta magnitude.
#[allow(dead_code)]
pub fn max_delta_magnitude(target: &BlendTargetExport) -> f32 {
    target
        .deltas
        .iter()
        .map(|d| (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        .fold(0.0f32, f32::max)
}

/// Count non-zero deltas.
#[allow(dead_code)]
pub fn nonzero_delta_count(target: &BlendTargetExport) -> usize {
    target
        .deltas
        .iter()
        .filter(|d| d[0].abs() > 1e-9 || d[1].abs() > 1e-9 || d[2].abs() > 1e-9)
        .count()
}

/// Validate blend target.
#[allow(dead_code)]
pub fn bt_validate(target: &BlendTargetExport) -> bool {
    (0.0..=1.0).contains(&target.weight)
        && target
            .deltas
            .iter()
            .all(|d| d[0].is_finite() && d[1].is_finite() && d[2].is_finite())
}

/// Export to JSON.
#[allow(dead_code)]
pub fn blend_target_to_json(target: &BlendTargetExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"vertices\":{},\"weight\":{:.6},\"max_delta\":{:.6}}}",
        target.name,
        bt_vertex_count(target),
        target.weight,
        max_delta_magnitude(target)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let t = new_blend_target("smile", 10);
        assert_eq!(bt_vertex_count(&t), 10);
        assert_eq!(t.name, "smile");
    }

    #[test]
    fn test_set_get_delta() {
        let mut t = new_blend_target("test", 3);
        set_delta(&mut t, 1, [1.0, 2.0, 3.0]);
        let d = get_delta(&t, 1).unwrap();
        assert!((d[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_get_oob() {
        let t = new_blend_target("test", 1);
        assert!(get_delta(&t, 5).is_none());
    }

    #[test]
    fn test_set_weight() {
        let mut t = new_blend_target("test", 1);
        set_weight(&mut t, 0.5);
        assert!((t.weight - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_weight_clamp() {
        let mut t = new_blend_target("test", 1);
        set_weight(&mut t, 1.5);
        assert!((t.weight - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_max_delta() {
        let mut t = new_blend_target("test", 2);
        set_delta(&mut t, 0, [3.0, 4.0, 0.0]);
        assert!((max_delta_magnitude(&t) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_nonzero_count() {
        let mut t = new_blend_target("test", 3);
        set_delta(&mut t, 0, [1.0, 0.0, 0.0]);
        assert_eq!(nonzero_delta_count(&t), 1);
    }

    #[test]
    fn test_validate() {
        let t = new_blend_target("test", 2);
        assert!(bt_validate(&t));
    }

    #[test]
    fn test_to_json() {
        let t = new_blend_target("blink", 5);
        let j = blend_target_to_json(&t);
        assert!(j.contains("\"name\":\"blink\""));
    }

    #[test]
    fn test_empty() {
        let t = new_blend_target("empty", 0);
        assert_eq!(bt_vertex_count(&t), 0);
        assert!((max_delta_magnitude(&t)).abs() < 1e-9);
    }
}
