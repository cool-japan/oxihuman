#![allow(dead_code)]
//! Per-vertex weight map for mesh operations.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VertexWeightMapMesh { weights: Vec<f32> }

#[allow(dead_code)]
pub fn new_weight_map_mesh(vertex_count: usize) -> VertexWeightMapMesh {
    VertexWeightMapMesh { weights: vec![0.0; vertex_count] }
}
#[allow(dead_code)]
pub fn set_vertex_weight_vwm(wm: &mut VertexWeightMapMesh, idx: usize, weight: f32) {
    if idx < wm.weights.len() { wm.weights[idx] = weight; }
}
#[allow(dead_code)]
pub fn get_vertex_weight_vwm(wm: &VertexWeightMapMesh, idx: usize) -> f32 {
    wm.weights.get(idx).copied().unwrap_or(0.0)
}
#[allow(dead_code)]
pub fn weight_count_vwm(wm: &VertexWeightMapMesh) -> usize { wm.weights.len() }
#[allow(dead_code)]
pub fn normalize_weight_map(wm: &mut VertexWeightMapMesh) {
    let max_w = wm.weights.iter().copied().fold(0.0f32, f32::max);
    if max_w > 1e-10 { for w in &mut wm.weights { *w /= max_w; } }
}
#[allow(dead_code)]
pub fn weight_map_to_bytes_vwm(wm: &VertexWeightMapMesh) -> Vec<u8> {
    let mut b = Vec::with_capacity(wm.weights.len() * 4);
    for &w in &wm.weights { b.extend_from_slice(&w.to_le_bytes()); }
    b
}
#[allow(dead_code)]
pub fn weight_map_to_json_vwm(wm: &VertexWeightMapMesh) -> String {
    let ws: Vec<String> = wm.weights.iter().map(|w| format!("{:.6}", w)).collect();
    format!("{{\"weights\":[{}]}}", ws.join(","))
}
#[allow(dead_code)]
pub fn clear_weight_map_vwm(wm: &mut VertexWeightMapMesh) { wm.weights.clear(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_new() { let wm = new_weight_map_mesh(5); assert_eq!(weight_count_vwm(&wm), 5); }
    #[test] fn test_set_get() { let mut wm = new_weight_map_mesh(3); set_vertex_weight_vwm(&mut wm, 1, 0.5); assert!((get_vertex_weight_vwm(&wm, 1) - 0.5).abs() < 1e-6); }
    #[test] fn test_count() { let wm = new_weight_map_mesh(10); assert_eq!(weight_count_vwm(&wm), 10); }
    #[test] fn test_normalize() { let mut wm = new_weight_map_mesh(3); set_vertex_weight_vwm(&mut wm, 0, 2.0); set_vertex_weight_vwm(&mut wm, 1, 4.0); normalize_weight_map(&mut wm); assert!((get_vertex_weight_vwm(&wm, 1) - 1.0).abs() < 1e-6); }
    #[test] fn test_bytes() { let wm = new_weight_map_mesh(3); assert_eq!(weight_map_to_bytes_vwm(&wm).len(), 12); }
    #[test] fn test_json() { let wm = new_weight_map_mesh(2); assert!(weight_map_to_json_vwm(&wm).contains("weights")); }
    #[test] fn test_clear() { let mut wm = new_weight_map_mesh(5); clear_weight_map_vwm(&mut wm); assert_eq!(weight_count_vwm(&wm), 0); }
    #[test] fn test_oob_set() { let mut wm = new_weight_map_mesh(2); set_vertex_weight_vwm(&mut wm, 99, 1.0); assert_eq!(weight_count_vwm(&wm), 2); }
    #[test] fn test_oob_get() { let wm = new_weight_map_mesh(2); assert!((get_vertex_weight_vwm(&wm, 99) - 0.0).abs() < 1e-9); }
    #[test] fn test_default_zero() { let wm = new_weight_map_mesh(3); for i in 0..3 { assert!((get_vertex_weight_vwm(&wm, i) - 0.0).abs() < 1e-9); } }
}
