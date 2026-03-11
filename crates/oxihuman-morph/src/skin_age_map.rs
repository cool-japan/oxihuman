#![allow(dead_code)]
//! Skin age map: per-vertex age factors for aging skin effects.

/// A map of per-vertex age factors.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SkinAgeMap {
    factors: Vec<f32>,
}

/// Create a new age map with `vertex_count` vertices, all at factor 0.
#[allow(dead_code)]
pub fn new_skin_age_map(vertex_count: usize) -> SkinAgeMap {
    SkinAgeMap {
        factors: vec![0.0; vertex_count],
    }
}

/// Set the age factor for vertex `index`.
#[allow(dead_code)]
pub fn set_age_factor(map: &mut SkinAgeMap, index: usize, factor: f32) {
    if let Some(f) = map.factors.get_mut(index) {
        *f = factor.clamp(0.0, 1.0);
    }
}

/// Get the age factor at vertex `index`.
#[allow(dead_code)]
pub fn age_factor_at(map: &SkinAgeMap, index: usize) -> f32 {
    map.factors.get(index).copied().unwrap_or(0.0)
}

/// Return the number of vertices.
#[allow(dead_code)]
pub fn age_vertex_count(map: &SkinAgeMap) -> usize {
    map.factors.len()
}

/// Convert age factors to morph params: returns (average_age, max_age, min_age).
#[allow(dead_code)]
pub fn age_to_params(map: &SkinAgeMap) -> (f32, f32, f32) {
    if map.factors.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let sum: f32 = map.factors.iter().sum();
    let avg = sum / map.factors.len() as f32;
    let max = map.factors.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min = map.factors.iter().cloned().fold(f32::INFINITY, f32::min);
    (avg, max, min)
}

/// Serialize to JSON-like string (summary only).
#[allow(dead_code)]
pub fn age_map_to_json(map: &SkinAgeMap) -> String {
    let (avg, max, min) = age_to_params(map);
    format!(
        "{{\"vertex_count\":{},\"avg\":{avg},\"max\":{max},\"min\":{min}}}",
        map.factors.len()
    )
}

/// Smooth the age map by averaging each vertex with its neighbors (simple box filter).
/// Uses `[i-1, i, i+1]` averaging.
#[allow(dead_code)]
pub fn smooth_age_map(map: &mut SkinAgeMap) {
    if map.factors.len() < 3 {
        return;
    }
    let old = map.factors.clone();
    for i in 1..old.len() - 1 {
        map.factors[i] = (old[i - 1] + old[i] + old[i + 1]) / 3.0;
    }
}

/// Reset all factors to 0.
#[allow(dead_code)]
pub fn clear_age_map(map: &mut SkinAgeMap) {
    for f in &mut map.factors {
        *f = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let m = new_skin_age_map(10);
        assert_eq!(age_vertex_count(&m), 10);
    }

    #[test]
    fn test_set_get_factor() {
        let mut m = new_skin_age_map(5);
        set_age_factor(&mut m, 2, 0.8);
        assert!((age_factor_at(&m, 2) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_factor_clamp() {
        let mut m = new_skin_age_map(5);
        set_age_factor(&mut m, 0, 2.0);
        assert!((age_factor_at(&m, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_factor_out_of_range() {
        let m = new_skin_age_map(5);
        assert!((age_factor_at(&m, 99) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_age_to_params_empty() {
        let m = new_skin_age_map(0);
        let (avg, max, min) = age_to_params(&m);
        assert!((avg - 0.0).abs() < 1e-6);
        assert!((max - 0.0).abs() < 1e-6);
        assert!((min - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_age_to_params() {
        let mut m = new_skin_age_map(3);
        set_age_factor(&mut m, 0, 0.2);
        set_age_factor(&mut m, 1, 0.4);
        set_age_factor(&mut m, 2, 0.6);
        let (avg, max, min) = age_to_params(&m);
        assert!((avg - 0.4).abs() < 1e-6);
        assert!((max - 0.6).abs() < 1e-6);
        assert!((min - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let m = new_skin_age_map(3);
        let json = age_map_to_json(&m);
        assert!(json.contains("\"vertex_count\":3"));
    }

    #[test]
    fn test_smooth() {
        let mut m = new_skin_age_map(5);
        set_age_factor(&mut m, 0, 0.0);
        set_age_factor(&mut m, 1, 0.0);
        set_age_factor(&mut m, 2, 0.9);
        set_age_factor(&mut m, 3, 0.0);
        set_age_factor(&mut m, 4, 0.0);
        smooth_age_map(&mut m);
        assert!(age_factor_at(&m, 2) < 0.9);
    }

    #[test]
    fn test_clear() {
        let mut m = new_skin_age_map(5);
        set_age_factor(&mut m, 0, 0.5);
        clear_age_map(&mut m);
        assert!((age_factor_at(&m, 0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_smooth_small() {
        let mut m = new_skin_age_map(2);
        set_age_factor(&mut m, 0, 1.0);
        set_age_factor(&mut m, 1, 0.0);
        smooth_age_map(&mut m);
        // Should be unchanged for < 3 vertices
        assert!((age_factor_at(&m, 0) - 1.0).abs() < 1e-6);
    }
}
