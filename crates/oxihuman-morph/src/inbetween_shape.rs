// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! In-between (combo) shape key stub.

/// An in-between shape definition: activates at a specific driver weight.
#[derive(Debug, Clone)]
pub struct InbetweenShape {
    pub name: String,
    pub trigger_weight: f32,
    pub deltas: Vec<[f32; 3]>,
    pub current_weight: f32,
}

impl InbetweenShape {
    pub fn new(name: &str, trigger_weight: f32, vertex_count: usize) -> Self {
        InbetweenShape {
            name: name.to_string(),
            trigger_weight: trigger_weight.clamp(0.0, 1.0),
            deltas: vec![[0.0; 3]; vertex_count],
            current_weight: 0.0,
        }
    }
}

/// Create a new inbetween shape.
pub fn new_inbetween_shape(name: &str, trigger_weight: f32, vertex_count: usize) -> InbetweenShape {
    InbetweenShape::new(name, trigger_weight, vertex_count)
}

/// Evaluate the inbetween weight given the driver weight.
/// Returns the blended activation weight.
pub fn inbetween_evaluate(shape: &mut InbetweenShape, driver_weight: f32) -> f32 {
    /* Activation is a tent function centered at trigger_weight with width 0.5 */
    let dist = (driver_weight - shape.trigger_weight).abs();
    let half_width = 0.25_f32;
    if dist >= half_width {
        shape.current_weight = 0.0;
    } else {
        shape.current_weight = 1.0 - dist / half_width;
    }
    shape.current_weight
}

/// Set the delta for a vertex.
pub fn inbetween_set_delta(shape: &mut InbetweenShape, index: usize, delta: [f32; 3]) {
    if index < shape.deltas.len() {
        shape.deltas[index] = delta;
    }
}

/// Reset the current weight.
pub fn inbetween_reset(shape: &mut InbetweenShape) {
    shape.current_weight = 0.0;
}

/// Return vertex count.
pub fn inbetween_vertex_count(shape: &InbetweenShape) -> usize {
    shape.deltas.len()
}

/// Return a JSON-like string.
pub fn inbetween_to_json(shape: &InbetweenShape) -> String {
    format!(
        r#"{{"name":"{}","trigger":{:.4},"weight":{:.4},"vertices":{}}}"#,
        shape.name,
        shape.trigger_weight,
        shape.current_weight,
        shape.deltas.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_inbetween_vertex_count() {
        let s = new_inbetween_shape("smile_half", 0.5, 10);
        assert_eq!(
            inbetween_vertex_count(&s),
            10, /* vertex count must match */
        );
    }

    #[test]
    fn test_trigger_weight_clamped() {
        let s = new_inbetween_shape("test", 2.0, 5);
        assert!((s.trigger_weight - 1.0).abs() < 1e-5, /* trigger clamped to 1 */);
    }

    #[test]
    fn test_evaluate_at_trigger_is_one() {
        let mut s = new_inbetween_shape("brow_half", 0.5, 5);
        let w = inbetween_evaluate(&mut s, 0.5);
        assert!((w - 1.0).abs() < 1e-5, /* at trigger point weight should be 1 */);
    }

    #[test]
    fn test_evaluate_far_from_trigger_is_zero() {
        let mut s = new_inbetween_shape("brow_half", 0.5, 5);
        let w = inbetween_evaluate(&mut s, 0.0);
        assert!((w).abs() < 1e-5 /* far from trigger should give 0 */,);
    }

    #[test]
    fn test_reset_zeroes_weight() {
        let mut s = new_inbetween_shape("lip_half", 0.3, 4);
        inbetween_evaluate(&mut s, 0.3);
        inbetween_reset(&mut s);
        assert!((s.current_weight).abs() < 1e-6, /* reset should zero weight */);
    }

    #[test]
    fn test_set_delta_updates() {
        let mut s = new_inbetween_shape("test", 0.5, 5);
        inbetween_set_delta(&mut s, 0, [1.0, 2.0, 3.0]);
        assert!((s.deltas[0][0] - 1.0).abs() < 1e-5, /* delta x must match */);
    }

    #[test]
    fn test_set_delta_out_of_bounds_ignored() {
        let mut s = new_inbetween_shape("test", 0.5, 2);
        inbetween_set_delta(&mut s, 99, [1.0, 0.0, 0.0]);
        assert_eq!(
            inbetween_vertex_count(&s),
            2, /* vertex count unchanged */
        );
    }

    #[test]
    fn test_to_json_contains_name() {
        let s = new_inbetween_shape("smile_quarter", 0.25, 3);
        let j = inbetween_to_json(&s);
        assert!(j.contains("smile_quarter"), /* JSON must contain shape name */);
    }

    #[test]
    fn test_initial_weight_zero() {
        let s = new_inbetween_shape("test", 0.5, 3);
        assert!((s.current_weight).abs() < 1e-6, /* initial weight should be 0 */);
    }

    #[test]
    fn test_evaluate_partial_activation() {
        let mut s = new_inbetween_shape("mid", 0.5, 3);
        let w = inbetween_evaluate(&mut s, 0.625);
        assert!(w > 0.0 && w < 1.0, /* partial activation should be between 0 and 1 */);
    }
}
