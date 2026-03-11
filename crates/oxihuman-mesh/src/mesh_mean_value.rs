// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

use std::f32::consts::PI;

/// Mean value coordinates for 2D polygon interpolation.
#[allow(dead_code)]
pub struct MeanValueWeights {
    pub weights: Vec<f32>,
    pub point: [f32; 2],
}

fn dist2(a: [f32; 2], b: [f32; 2]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    (dx * dx + dy * dy).sqrt()
}

fn cross2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

fn dot2(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// Compute mean value coordinates of point p w.r.t. a convex polygon.
#[allow(dead_code)]
pub fn mean_value_coords_2d(polygon: &[[f32; 2]], p: [f32; 2]) -> MeanValueWeights {
    let n = polygon.len();
    if n == 0 {
        return MeanValueWeights {
            weights: vec![],
            point: p,
        };
    }

    let mut weights = vec![0.0f32; n];

    // Check if p coincides with a vertex
    for (i, &v) in polygon.iter().enumerate() {
        if dist2(p, v) < 1e-8 {
            weights[i] = 1.0;
            return MeanValueWeights { weights, point: p };
        }
    }

    let mut total = 0.0f32;
    for i in 0..n {
        let j = (i + n - 1) % n;
        let k = (i + 1) % n;
        let ri = dist2(p, polygon[i]);
        let rj = dist2(p, polygon[j]);
        let rk = dist2(p, polygon[k]);

        let ei = [(polygon[i][0] - p[0]) / ri, (polygon[i][1] - p[1]) / ri];
        let ej = [(polygon[j][0] - p[0]) / rj, (polygon[j][1] - p[1]) / rj];
        let ek = [(polygon[k][0] - p[0]) / rk, (polygon[k][1] - p[1]) / rk];

        let tan_half_j = {
            let cross = cross2(ej, ei);
            let dot_val = dot2(ej, ei);
            if (1.0 + dot_val).abs() < 1e-8 {
                0.0
            } else {
                cross / (1.0 + dot_val)
            }
        };
        let tan_half_k = {
            let cross = cross2(ei, ek);
            let dot_val = dot2(ei, ek);
            if (1.0 + dot_val).abs() < 1e-8 {
                0.0
            } else {
                cross / (1.0 + dot_val)
            }
        };

        let w = (tan_half_j + tan_half_k) / ri;
        weights[i] = w;
        total += w;
    }

    if total.abs() > 1e-10 {
        for w in &mut weights {
            *w /= total;
        }
    }

    MeanValueWeights { weights, point: p }
}

#[allow(dead_code)]
pub fn interpolate_scalar(polygon_values: &[f32], mvw: &MeanValueWeights) -> f32 {
    polygon_values
        .iter()
        .zip(mvw.weights.iter())
        .map(|(&v, &w)| v * w)
        .sum()
}

#[allow(dead_code)]
pub fn weights_sum(mvw: &MeanValueWeights) -> f32 {
    mvw.weights.iter().sum()
}

#[allow(dead_code)]
pub fn weights_count(mvw: &MeanValueWeights) -> usize {
    mvw.weights.len()
}

/// Build a regular polygon for testing.
#[allow(dead_code)]
pub fn regular_polygon(n: usize, radius: f32) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let angle = 2.0 * PI * i as f32 / n as f32;
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect()
}

#[allow(dead_code)]
pub fn mean_value_to_json(mvw: &MeanValueWeights) -> String {
    let ws: Vec<String> = mvw.weights.iter().map(|w| format!("{:.6}", w)).collect();
    format!(
        "{{\"point\":[{:.6},{:.6}],\"weights\":[{}]}}",
        mvw.point[0],
        mvw.point[1],
        ws.join(",")
    )
}

#[allow(dead_code)]
pub fn max_weight(mvw: &MeanValueWeights) -> f32 {
    mvw.weights
        .iter()
        .cloned()
        .fold(f32::NEG_INFINITY, f32::max)
}

#[allow(dead_code)]
pub fn dominant_vertex(mvw: &MeanValueWeights) -> Option<usize> {
    if mvw.weights.is_empty() {
        return None;
    }
    let mut best = 0;
    let mut best_w = mvw.weights[0];
    for (i, &w) in mvw.weights.iter().enumerate() {
        if w > best_w {
            best_w = w;
            best = i;
        }
    }
    Some(best)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square() -> Vec<[f32; 2]> {
        vec![[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
    }

    #[test]
    fn test_center_weights_sum_to_one() {
        let sq = square();
        let mvw = mean_value_coords_2d(&sq, [0.0, 0.0]);
        let s = weights_sum(&mvw);
        assert!((s - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_center_weights_count() {
        let sq = square();
        let mvw = mean_value_coords_2d(&sq, [0.0, 0.0]);
        assert_eq!(weights_count(&mvw), 4);
    }

    #[test]
    fn test_coincident_vertex() {
        let sq = square();
        let mvw = mean_value_coords_2d(&sq, [-1.0, -1.0]);
        assert!((mvw.weights[0] - 1.0).abs() < 1e-6);
        assert!(mvw.weights[1].abs() < 1e-6);
    }

    #[test]
    fn test_interpolate_constant() {
        let sq = square();
        let vals = vec![5.0f32; 4];
        let mvw = mean_value_coords_2d(&sq, [0.0, 0.0]);
        let v = interpolate_scalar(&vals, &mvw);
        assert!((v - 5.0).abs() < 1e-3);
    }

    #[test]
    fn test_empty_polygon() {
        let mvw = mean_value_coords_2d(&[], [0.0, 0.0]);
        assert_eq!(mvw.weights.len(), 0);
    }

    #[test]
    fn test_regular_polygon_count() {
        let p = regular_polygon(6, 1.0);
        assert_eq!(p.len(), 6);
    }

    #[test]
    fn test_max_weight_positive() {
        let sq = square();
        let mvw = mean_value_coords_2d(&sq, [0.5, 0.0]);
        assert!(max_weight(&mvw) > 0.0);
    }

    #[test]
    fn test_dominant_vertex_some() {
        let sq = square();
        let mvw = mean_value_coords_2d(&sq, [0.0, 0.0]);
        assert!(dominant_vertex(&mvw).is_some());
    }

    #[test]
    fn test_to_json() {
        let sq = square();
        let mvw = mean_value_coords_2d(&sq, [0.0, 0.0]);
        let j = mean_value_to_json(&mvw);
        assert!(j.contains("point"));
        assert!(j.contains("weights"));
    }

    #[test]
    fn test_weights_all_finite() {
        let sq = square();
        let mvw = mean_value_coords_2d(&sq, [0.1, 0.2]);
        for &w in &mvw.weights {
            assert!(w.is_finite());
        }
    }
}
