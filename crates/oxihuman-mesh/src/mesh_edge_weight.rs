#![allow(dead_code)]

//! Per-edge weight storage for mesh operations.

use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EdgeWeight {
    pub weights: HashMap<(u32, u32), f32>,
}

fn canonical(a: u32, b: u32) -> (u32, u32) { if a < b { (a, b) } else { (b, a) } }

#[allow(dead_code)]
pub fn set_edge_weight(ew: &mut EdgeWeight, a: u32, b: u32, weight: f32) {
    ew.weights.insert(canonical(a, b), weight);
}

#[allow(dead_code)]
pub fn get_edge_weight(ew: &EdgeWeight, a: u32, b: u32) -> f32 {
    ew.weights.get(&canonical(a, b)).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn edge_weight_count(ew: &EdgeWeight) -> usize {
    ew.weights.len()
}

#[allow(dead_code)]
pub fn weighted_edges(ew: &EdgeWeight) -> Vec<((u32, u32), f32)> {
    let mut result: Vec<_> = ew.weights.iter().map(|(&k, &v)| (k, v)).collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[allow(dead_code)]
pub fn max_edge_weight(ew: &EdgeWeight) -> f32 {
    ew.weights.values().copied().fold(f32::NEG_INFINITY, f32::max)
}

#[allow(dead_code)]
pub fn min_edge_weight(ew: &EdgeWeight) -> f32 {
    ew.weights.values().copied().fold(f32::INFINITY, f32::min)
}

#[allow(dead_code)]
pub fn edge_weights_to_json(ew: &EdgeWeight) -> String {
    let entries: Vec<String> = ew.weights.iter()
        .map(|(&(a, b), &w)| format!("{{\"edge\":[{},{}],\"weight\":{:.4}}}", a, b, w))
        .collect();
    format!("{{\"count\":{},\"edges\":[{}]}}", ew.weights.len(), entries.join(","))
}

#[allow(dead_code)]
pub fn clear_edge_weights(ew: &mut EdgeWeight) {
    ew.weights.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ew() -> EdgeWeight { EdgeWeight { weights: HashMap::new() } }

    #[test]
    fn test_set_get() { let mut e = ew(); set_edge_weight(&mut e, 0, 1, 0.5); assert!((get_edge_weight(&e, 0, 1) - 0.5).abs() < 1e-6); }
    #[test]
    fn test_canonical() { let mut e = ew(); set_edge_weight(&mut e, 1, 0, 0.5); assert!((get_edge_weight(&e, 0, 1) - 0.5).abs() < 1e-6); }
    #[test]
    fn test_missing() { let e = ew(); assert!((get_edge_weight(&e, 0, 1)).abs() < 1e-6); }
    #[test]
    fn test_count() { let mut e = ew(); set_edge_weight(&mut e, 0, 1, 1.0); assert_eq!(edge_weight_count(&e), 1); }
    #[test]
    fn test_weighted_edges() { let mut e = ew(); set_edge_weight(&mut e, 0, 1, 1.0); let w = weighted_edges(&e); assert_eq!(w.len(), 1); }
    #[test]
    fn test_max() { let mut e = ew(); set_edge_weight(&mut e, 0, 1, 0.5); set_edge_weight(&mut e, 1, 2, 0.9); assert!((max_edge_weight(&e) - 0.9).abs() < 1e-6); }
    #[test]
    fn test_min() { let mut e = ew(); set_edge_weight(&mut e, 0, 1, 0.5); set_edge_weight(&mut e, 1, 2, 0.1); assert!((min_edge_weight(&e) - 0.1).abs() < 1e-6); }
    #[test]
    fn test_to_json() { let e = ew(); assert!(edge_weights_to_json(&e).contains("\"count\":0")); }
    #[test]
    fn test_clear() { let mut e = ew(); set_edge_weight(&mut e, 0, 1, 1.0); clear_edge_weights(&mut e); assert_eq!(edge_weight_count(&e), 0); }
    #[test]
    fn test_overwrite() { let mut e = ew(); set_edge_weight(&mut e, 0, 1, 0.5); set_edge_weight(&mut e, 0, 1, 0.9); assert!((get_edge_weight(&e, 0, 1) - 0.9).abs() < 1e-6); }
}
