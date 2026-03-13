// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)]

//! Corrective shape key export for pose-driven blendshapes.

/// A corrective shape driven by a pose.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CorrectiveShapeExport {
    pub name: String,
    pub driver_bone: String,
    pub driver_axis: u8,
    pub driver_min: f32,
    pub driver_max: f32,
    pub deltas: Vec<[f32; 3]>,
}

/// Create a new corrective shape.
#[allow(dead_code)]
pub fn new_corrective_shape(
    name: &str,
    driver: &str,
    vertex_count: usize,
) -> CorrectiveShapeExport {
    CorrectiveShapeExport {
        name: name.to_string(),
        driver_bone: driver.to_string(),
        driver_axis: 0,
        driver_min: 0.0,
        driver_max: 1.0,
        deltas: vec![[0.0; 3]; vertex_count],
    }
}

/// Set driver range.
#[allow(dead_code)]
pub fn set_driver_range(e: &mut CorrectiveShapeExport, min: f32, max: f32) {
    e.driver_min = min;
    e.driver_max = max;
}

/// Set driver axis (0=X, 1=Y, 2=Z).
#[allow(dead_code)]
pub fn set_driver_axis(e: &mut CorrectiveShapeExport, axis: u8) {
    e.driver_axis = axis.min(2);
}

/// Set delta.
#[allow(dead_code)]
pub fn cs_set_delta(e: &mut CorrectiveShapeExport, idx: usize, d: [f32; 3]) {
    if idx < e.deltas.len() {
        e.deltas[idx] = d;
    }
}

/// Vertex count.
#[allow(dead_code)]
pub fn cs_vertex_count(e: &CorrectiveShapeExport) -> usize {
    e.deltas.len()
}

/// Non-zero delta count.
#[allow(dead_code)]
pub fn cs_nonzero_count(e: &CorrectiveShapeExport) -> usize {
    e.deltas
        .iter()
        .filter(|d| d[0].abs() > 1e-9 || d[1].abs() > 1e-9 || d[2].abs() > 1e-9)
        .count()
}

/// Evaluate driver weight from input value.
#[allow(dead_code)]
pub fn evaluate_driver(e: &CorrectiveShapeExport, value: f32) -> f32 {
    let range = e.driver_max - e.driver_min;
    if range.abs() < 1e-12 {
        return 0.0;
    }
    ((value - e.driver_min) / range).clamp(0.0, 1.0)
}

/// Validate.
#[allow(dead_code)]
pub fn cs_validate(e: &CorrectiveShapeExport) -> bool {
    !e.name.is_empty() && !e.driver_bone.is_empty() && e.driver_axis <= 2
}

/// Export to JSON.
#[allow(dead_code)]
pub fn corrective_shape_to_json(e: &CorrectiveShapeExport) -> String {
    format!(
        "{{\"name\":\"{}\",\"driver\":\"{}\",\"vertices\":{}}}",
        e.name,
        e.driver_bone,
        cs_vertex_count(e)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_new() {
        let e = new_corrective_shape("elbow_fix", "forearm", 10);
        assert_eq!(cs_vertex_count(&e), 10);
    }
    #[test]
    fn test_set_range() {
        let mut e = new_corrective_shape("a", "b", 1);
        set_driver_range(&mut e, -1.0, 1.0);
        assert!((e.driver_min - (-1.0)).abs() < 1e-6);
    }
    #[test]
    fn test_set_axis() {
        let mut e = new_corrective_shape("a", "b", 1);
        set_driver_axis(&mut e, 1);
        assert_eq!(e.driver_axis, 1);
    }
    #[test]
    fn test_axis_clamp() {
        let mut e = new_corrective_shape("a", "b", 1);
        set_driver_axis(&mut e, 5);
        assert_eq!(e.driver_axis, 2);
    }
    #[test]
    fn test_set_delta() {
        let mut e = new_corrective_shape("a", "b", 2);
        cs_set_delta(&mut e, 0, [1.0, 0.0, 0.0]);
        assert!((e.deltas[0][0] - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_nonzero() {
        let mut e = new_corrective_shape("a", "b", 3);
        cs_set_delta(&mut e, 0, [1.0, 0.0, 0.0]);
        assert_eq!(cs_nonzero_count(&e), 1);
    }
    #[test]
    fn test_evaluate_driver() {
        let e = new_corrective_shape("a", "b", 1);
        assert!((evaluate_driver(&e, 0.5) - 0.5).abs() < 1e-6);
    }
    #[test]
    fn test_evaluate_clamp() {
        let e = new_corrective_shape("a", "b", 1);
        assert!((evaluate_driver(&e, 2.0) - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_validate() {
        let e = new_corrective_shape("a", "b", 1);
        assert!(cs_validate(&e));
    }
    #[test]
    fn test_to_json() {
        let e = new_corrective_shape("fix", "bone", 5);
        assert!(corrective_shape_to_json(&e).contains("\"name\":\"fix\""));
    }
}
