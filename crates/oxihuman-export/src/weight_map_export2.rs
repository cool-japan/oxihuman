//! Weight map export (new, distinct from weight_map_export).
#![allow(dead_code)]

/// Weight map export data.
#[allow(dead_code)]
pub struct WeightMapExport2 {
    pub weights: Vec<f32>,
    pub vertex_count: usize,
}

/// Create a new weight map export.
#[allow(dead_code)]
pub fn new_weight_map_export2(vertex_count: usize) -> WeightMapExport2 {
    WeightMapExport2 { weights: vec![0.0; vertex_count], vertex_count }
}

/// Set weight at vertex index.
#[allow(dead_code)]
pub fn set_weight2(wm: &mut WeightMapExport2, i: usize, w: f32) {
    if i < wm.weights.len() { wm.weights[i] = w.clamp(0.0, 1.0); }
}

/// Get weight at vertex index.
#[allow(dead_code)]
pub fn get_weight2(wm: &WeightMapExport2, i: usize) -> f32 {
    wm.weights.get(i).copied().unwrap_or(0.0)
}

/// Export weight map to JSON.
#[allow(dead_code)]
pub fn export_weight_map2_json(wm: &WeightMapExport2) -> String {
    let vals: Vec<String> = wm.weights.iter().map(|w| format!("{:.4}", w)).collect();
    format!("[{}]", vals.join(","))
}

/// Get vertex count.
#[allow(dead_code)]
pub fn weight_map2_vertex_count(wm: &WeightMapExport2) -> usize { wm.vertex_count }

/// Get the index of the vertex with the maximum weight.
#[allow(dead_code)]
pub fn max_weight2_index(wm: &WeightMapExport2) -> Option<usize> {
    if wm.weights.is_empty() { return None; }
    wm.weights.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)).map(|(i,_)| i)
}

/// Normalize weights so they sum to 1.
#[allow(dead_code)]
pub fn normalize_weights2(wm: &mut WeightMapExport2) {
    let sum: f32 = wm.weights.iter().sum();
    if sum > 1e-8 { for w in wm.weights.iter_mut() { *w /= sum; } }
}

/// Convert weights to bytes (f32 LE).
#[allow(dead_code)]
pub fn weight_map2_to_bytes(wm: &WeightMapExport2) -> Vec<u8> {
    wm.weights.iter().flat_map(|w| w.to_le_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_weight_map_size() {
        let wm = new_weight_map_export2(5);
        assert_eq!(weight_map2_vertex_count(&wm), 5);
    }

    #[test]
    fn test_set_get_weight() {
        let mut wm = new_weight_map_export2(5);
        set_weight2(&mut wm, 2, 0.75);
        assert!((get_weight2(&wm, 2) - 0.75).abs() < 1e-5);
    }

    #[test]
    fn test_get_weight_oob() {
        let wm = new_weight_map_export2(3);
        assert!((get_weight2(&wm, 100)).abs() < 1e-5);
    }

    #[test]
    fn test_export_json() {
        let wm = new_weight_map_export2(2);
        let j = export_weight_map2_json(&wm);
        assert!(!j.is_empty());
    }

    #[test]
    fn test_max_weight_index() {
        let mut wm = new_weight_map_export2(3);
        set_weight2(&mut wm, 1, 0.9);
        assert_eq!(max_weight2_index(&wm), Some(1));
    }

    #[test]
    fn test_normalize_weights() {
        let mut wm = new_weight_map_export2(3);
        set_weight2(&mut wm, 0, 1.0);
        set_weight2(&mut wm, 1, 1.0);
        set_weight2(&mut wm, 2, 0.0);
        normalize_weights2(&mut wm);
        let sum: f32 = wm.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_weight_map_to_bytes() {
        let wm = new_weight_map_export2(3);
        let b = weight_map2_to_bytes(&wm);
        assert_eq!(b.len(), 3 * 4);
    }

    #[test]
    fn test_set_weight_clamped() {
        let mut wm = new_weight_map_export2(2);
        set_weight2(&mut wm, 0, 2.0);
        assert!((get_weight2(&wm, 0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_max_weight_empty() {
        let wm = new_weight_map_export2(0);
        assert!(max_weight2_index(&wm).is_none());
    }
}
