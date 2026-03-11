#![allow(dead_code)]

//! Vertex weight painting.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WeightPaint {
    pub weights: HashMap<usize, f32>,
    pub vertex_count: usize,
}

#[allow(dead_code)]
pub fn new_weight_paint(vertex_count: usize) -> WeightPaint {
    WeightPaint { weights: HashMap::new(), vertex_count }
}

#[allow(dead_code)]
pub fn paint_weight(wp: &mut WeightPaint, vertex: usize, weight: f32) {
    if vertex < wp.vertex_count {
        wp.weights.insert(vertex, weight.clamp(0.0, 1.0));
    }
}

#[allow(dead_code)]
pub fn weight_at_vertex(wp: &WeightPaint, vertex: usize) -> f32 {
    wp.weights.get(&vertex).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn smooth_weights_wp(wp: &mut WeightPaint, adjacency: &[Vec<usize>]) {
    let old = wp.weights.clone();
    for &v in old.keys() {
        if v < adjacency.len() {
            let neighbors = &adjacency[v];
            if !neighbors.is_empty() {
                let sum: f32 = neighbors.iter().map(|&n| old.get(&n).copied().unwrap_or(0.0)).sum();
                let avg = sum / neighbors.len() as f32;
                let cur = old.get(&v).copied().unwrap_or(0.0);
                wp.weights.insert(v, (cur + avg) * 0.5);
            }
        }
    }
}

#[allow(dead_code)]
pub fn normalize_weights_wp(wp: &mut WeightPaint) {
    let max_w = wp.weights.values().copied().fold(0.0f32, f32::max);
    if max_w > 1e-12 {
        for w in wp.weights.values_mut() {
            *w /= max_w;
        }
    }
}

#[allow(dead_code)]
pub fn clear_paint(wp: &mut WeightPaint) {
    wp.weights.clear();
}

#[allow(dead_code)]
pub fn paint_to_bytes(wp: &WeightPaint) -> Vec<u8> {
    let mut result = vec![0u8; wp.vertex_count];
    for (&v, &w) in &wp.weights {
        if v < result.len() {
            result[v] = (w * 255.0).round() as u8;
        }
    }
    result
}

#[allow(dead_code)]
pub fn paint_vertex_count(wp: &WeightPaint) -> usize {
    wp.weights.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() { let wp = new_weight_paint(10); assert_eq!(wp.vertex_count, 10); }
    #[test]
    fn test_paint() { let mut wp = new_weight_paint(3); paint_weight(&mut wp, 0, 0.5); assert!((weight_at_vertex(&wp, 0) - 0.5).abs() < 1e-6); }
    #[test]
    fn test_paint_clamp() { let mut wp = new_weight_paint(3); paint_weight(&mut wp, 0, 2.0); assert!((weight_at_vertex(&wp, 0) - 1.0).abs() < 1e-6); }
    #[test]
    fn test_unpainted_zero() { let wp = new_weight_paint(3); assert!((weight_at_vertex(&wp, 0)).abs() < 1e-6); }
    #[test]
    fn test_smooth() {
        let mut wp = new_weight_paint(3);
        paint_weight(&mut wp, 0, 1.0);
        paint_weight(&mut wp, 1, 0.0);
        let adj = vec![vec![1], vec![0], vec![]];
        smooth_weights_wp(&mut wp, &adj);
        assert!(weight_at_vertex(&wp, 0) < 1.0);
    }
    #[test]
    fn test_normalize() {
        let mut wp = new_weight_paint(3);
        paint_weight(&mut wp, 0, 0.5);
        normalize_weights_wp(&mut wp);
        assert!((weight_at_vertex(&wp, 0) - 1.0).abs() < 1e-6);
    }
    #[test]
    fn test_clear() { let mut wp = new_weight_paint(3); paint_weight(&mut wp, 0, 1.0); clear_paint(&mut wp); assert_eq!(paint_vertex_count(&wp), 0); }
    #[test]
    fn test_to_bytes() { let mut wp = new_weight_paint(3); paint_weight(&mut wp, 1, 1.0); let b = paint_to_bytes(&wp); assert_eq!(b[1], 255); assert_eq!(b[0], 0); }
    #[test]
    fn test_paint_vertex_count() { let mut wp = new_weight_paint(10); paint_weight(&mut wp, 0, 0.5); assert_eq!(paint_vertex_count(&wp), 1); }
    #[test]
    fn test_out_of_bounds() { let mut wp = new_weight_paint(2); paint_weight(&mut wp, 5, 1.0); assert_eq!(paint_vertex_count(&wp), 0); }
}
