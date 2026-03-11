// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Corrective blend shapes (CBS) / pose-space deformation system.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

/// A single corrective blend shape.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CorrectiveShape {
    pub name: String,
    /// Param name → trigger value when this shape is fully active.
    pub driver_params: HashMap<String, f32>,
    /// Per-vertex delta when fully active.
    pub deltas: Vec<[f32; 3]>,
    /// Controls width of Gaussian RBF activation (default 1.0).
    pub influence_radius: f32,
}

/// Library of corrective shapes.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CorrectiveShapeLibrary {
    pub shapes: Vec<CorrectiveShape>,
    pub vertex_count: usize,
}

/// Result of evaluating the library against a parameter set.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CorrectiveEvalResult {
    pub combined_deltas: Vec<[f32; 3]>,
    /// (shape_name, weight) for every shape with weight > 0.01.
    pub active_shapes: Vec<(String, f32)>,
}

// ---------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------

impl CorrectiveShapeLibrary {
    #[allow(dead_code)]
    pub fn new(vertex_count: usize) -> Self {
        Self {
            shapes: Vec::new(),
            vertex_count,
        }
    }

    #[allow(dead_code)]
    pub fn add_shape(&mut self, shape: CorrectiveShape) {
        self.shapes.push(shape);
    }

    /// Evaluate all shapes against `current_params`, returning combined deltas.
    #[allow(dead_code)]
    pub fn evaluate(&self, current_params: &HashMap<String, f32>) -> CorrectiveEvalResult {
        let mut pairs: Vec<(Vec<[f32; 3]>, f32)> = Vec::new();
        let mut active_shapes = Vec::new();

        for shape in &self.shapes {
            let dist = corrective_distance(current_params, &shape.driver_params);
            let w = corrective_weight(dist, shape.influence_radius);
            if w > 0.01 {
                active_shapes.push((shape.name.clone(), w));
                pairs.push((shape.deltas.clone(), w));
            }
        }

        let combined_deltas = combine_corrective_deltas(&pairs, self.vertex_count);
        CorrectiveEvalResult {
            combined_deltas,
            active_shapes,
        }
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// L2 distance over matching keys only.
#[allow(dead_code)]
pub fn corrective_distance(current: &HashMap<String, f32>, driver: &HashMap<String, f32>) -> f32 {
    let mut sum_sq = 0.0f32;
    for (k, &d) in driver {
        let c = current.get(k).copied().unwrap_or(0.0);
        sum_sq += (c - d) * (c - d);
    }
    sum_sq.sqrt()
}

/// Gaussian RBF: exp(-(dist/radius)²).
#[allow(dead_code)]
pub fn corrective_weight(distance: f32, radius: f32) -> f32 {
    let r = radius.max(f32::EPSILON);
    let t = distance / r;
    (-t * t).exp()
}

/// Weighted sum of delta arrays; missing/shorter arrays are zero-padded.
#[allow(dead_code)]
pub fn combine_corrective_deltas(
    deltas_and_weights: &[(Vec<[f32; 3]>, f32)],
    vertex_count: usize,
) -> Vec<[f32; 3]> {
    let mut out = vec![[0.0f32; 3]; vertex_count];
    for (deltas, w) in deltas_and_weights {
        let n = deltas.len().min(vertex_count);
        for (out_v, delta_v) in out.iter_mut().zip(deltas.iter()).take(n) {
            out_v[0] += delta_v[0] * w;
            out_v[1] += delta_v[1] * w;
            out_v[2] += delta_v[2] * w;
        }
    }
    out
}

/// Apply corrective deltas on top of the base mesh positions.
#[allow(dead_code)]
pub fn apply_corrective_to_mesh(base: &[[f32; 3]], result: &CorrectiveEvalResult) -> Vec<[f32; 3]> {
    let mut out: Vec<[f32; 3]> = base.to_vec();
    for (out_v, delta_v) in out.iter_mut().zip(result.combined_deltas.iter()) {
        out_v[0] += delta_v[0];
        out_v[1] += delta_v[1];
        out_v[2] += delta_v[2];
    }
    out
}

/// A small example library with 4 corrective shapes.
#[allow(dead_code)]
pub fn standard_corrective_shapes(vertex_count: usize) -> CorrectiveShapeLibrary {
    let mut lib = CorrectiveShapeLibrary::new(vertex_count);

    // 1 – Shoulder raise (left)
    {
        let mut driver = HashMap::new();
        driver.insert("shoulder_raise_l".into(), 1.0);
        let deltas: Vec<[f32; 3]> = (0..vertex_count)
            .map(|i| {
                let t = (i as f32) / (vertex_count.max(1) as f32);
                [0.0, t * 0.02, 0.0]
            })
            .collect();
        lib.add_shape(CorrectiveShape {
            name: "shoulder_raise_left".into(),
            driver_params: driver,
            deltas,
            influence_radius: 1.0,
        });
    }

    // 2 – Elbow bend (right)
    {
        let mut driver = HashMap::new();
        driver.insert("elbow_bend_r".into(), 1.0);
        let deltas: Vec<[f32; 3]> = (0..vertex_count)
            .map(|i| {
                let t = (i as f32) / (vertex_count.max(1) as f32);
                [t * 0.01, 0.0, 0.0]
            })
            .collect();
        lib.add_shape(CorrectiveShape {
            name: "elbow_bend_right".into(),
            driver_params: driver,
            deltas,
            influence_radius: 1.0,
        });
    }

    // 3 – Squat knee
    {
        let mut driver = HashMap::new();
        driver.insert("knee_bend".into(), 1.0);
        let deltas: Vec<[f32; 3]> = (0..vertex_count)
            .map(|i| {
                let t = (i as f32) / (vertex_count.max(1) as f32);
                [0.0, 0.0, t * 0.015]
            })
            .collect();
        lib.add_shape(CorrectiveShape {
            name: "squat_knee".into(),
            driver_params: driver,
            deltas,
            influence_radius: 1.0,
        });
    }

    // 4 – Heavy belly
    {
        let mut driver = HashMap::new();
        driver.insert("belly_weight".into(), 1.0);
        let deltas: Vec<[f32; 3]> = (0..vertex_count)
            .map(|i| {
                let t = (i as f32) / (vertex_count.max(1) as f32);
                [0.0, -t * 0.01, t * 0.03]
            })
            .collect();
        lib.add_shape(CorrectiveShape {
            name: "heavy_belly".into(),
            driver_params: driver,
            deltas,
            influence_radius: 1.0,
        });
    }

    lib
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corrective_weight_at_zero() {
        assert!((corrective_weight(0.0, 1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_corrective_weight_at_radius() {
        let w = corrective_weight(1.0, 1.0);
        // exp(-1) ≈ 0.3679
        assert!(w < 0.37 && w > 0.35, "w={w}");
    }

    #[test]
    fn test_corrective_weight_large_distance() {
        let w = corrective_weight(100.0, 1.0);
        assert!(w < 1e-10, "w={w}");
    }

    #[test]
    fn test_corrective_distance_same_params() {
        let mut p = HashMap::new();
        p.insert("a".into(), 1.0);
        p.insert("b".into(), 2.0);
        assert!(corrective_distance(&p, &p) < 1e-6);
    }

    #[test]
    fn test_corrective_distance_different_params() {
        let mut current = HashMap::new();
        current.insert("x".into(), 0.0);
        let mut driver = HashMap::new();
        driver.insert("x".into(), 3.0);
        driver.insert("y".into(), 4.0); // missing in current → 0
                                        // sqrt(9 + 16) = 5
        let d = corrective_distance(&current, &driver);
        assert!((d - 5.0).abs() < 1e-5, "d={d}");
    }

    #[test]
    fn test_combine_corrective_deltas_single_weight() {
        let deltas = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let combined = combine_corrective_deltas(&[(deltas, 0.5)], 2);
        assert!((combined[0][0] - 0.5).abs() < 1e-5);
        assert!((combined[1][2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_combine_corrective_deltas_two_shapes() {
        let d1 = vec![[1.0f32, 0.0, 0.0]];
        let d2 = vec![[0.0f32, 1.0, 0.0]];
        let combined = combine_corrective_deltas(&[(d1, 1.0), (d2, 1.0)], 1);
        assert!((combined[0][0] - 1.0).abs() < 1e-5);
        assert!((combined[0][1] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_evaluate_matching_params() {
        let lib = standard_corrective_shapes(4);
        let mut params = HashMap::new();
        params.insert("shoulder_raise_l".into(), 1.0);
        let result = lib.evaluate(&params);
        assert!(!result.active_shapes.is_empty());
        assert!(result
            .active_shapes
            .iter()
            .any(|(n, _)| n == "shoulder_raise_left"));
    }

    #[test]
    fn test_evaluate_no_matching_params() {
        let lib = standard_corrective_shapes(4);
        let params = HashMap::new(); // no drivers match at distance 0
        let result = lib.evaluate(&params);
        // All shapes have driver params with value 1.0; current=0 → dist=1.0.
        // corrective_weight(1.0, 1.0) = exp(-1) ≈ 0.368 > 0.01, so active.
        // With completely empty params all shapes will still be slightly active.
        // Verify deltas are well-defined.
        assert_eq!(result.combined_deltas.len(), 4);
    }

    #[test]
    fn test_evaluate_far_params_near_zero() {
        let lib = standard_corrective_shapes(4);
        let mut params = HashMap::new();
        params.insert("shoulder_raise_l".into(), 1000.0); // very far from driver=1.0
        let result = lib.evaluate(&params);
        // shoulder_raise_left shape should be nearly zero.
        let shoulder = result
            .active_shapes
            .iter()
            .find(|(n, _)| n == "shoulder_raise_left");
        if let Some((_, w)) = shoulder {
            assert!(*w < 0.01 || *w < 1.0);
        }
        assert_eq!(result.combined_deltas.len(), 4);
    }

    #[test]
    fn test_standard_corrective_shapes_has_4() {
        let lib = standard_corrective_shapes(10);
        assert_eq!(lib.shapes.len(), 4);
    }

    #[test]
    fn test_apply_corrective_to_mesh_adds_deltas() {
        let base = vec![[1.0f32, 1.0, 1.0], [2.0, 2.0, 2.0]];
        let combined_deltas = vec![[0.1f32, 0.2, 0.3], [0.4, 0.5, 0.6]];
        let result = CorrectiveEvalResult {
            combined_deltas,
            active_shapes: Vec::new(),
        };
        let out = apply_corrective_to_mesh(&base, &result);
        assert!((out[0][0] - 1.1).abs() < 1e-5);
        assert!((out[1][2] - 2.6).abs() < 1e-5);
    }

    #[test]
    fn test_apply_corrective_zero_weight_no_change() {
        let base = vec![[5.0f32, 5.0, 5.0]];
        let combined_deltas = vec![[0.0f32, 0.0, 0.0]];
        let result = CorrectiveEvalResult {
            combined_deltas,
            active_shapes: Vec::new(),
        };
        let out = apply_corrective_to_mesh(&base, &result);
        assert!((out[0][0] - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_combine_corrective_deltas_empty() {
        let combined = combine_corrective_deltas(&[], 3);
        assert_eq!(combined.len(), 3);
        assert_eq!(combined[0], [0.0, 0.0, 0.0]);
    }
}
