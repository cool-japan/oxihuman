#![allow(dead_code)]

/// Per-vertex body fat distribution map.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BodyFatMap {
    levels: Vec<f32>,
}

#[allow(dead_code)]
pub fn new_body_fat_map(vertex_count: usize) -> BodyFatMap {
    BodyFatMap { levels: vec![0.0; vertex_count] }
}

#[allow(dead_code)]
pub fn set_fat_level(map: &mut BodyFatMap, index: usize, level: f32) {
    if index < map.levels.len() {
        map.levels[index] = level.clamp(0.0, 1.0);
    }
}

#[allow(dead_code)]
pub fn fat_level_at(map: &BodyFatMap, index: usize) -> f32 {
    map.levels.get(index).copied().unwrap_or(0.0)
}

#[allow(dead_code)]
pub fn fat_vertex_count(map: &BodyFatMap) -> usize {
    map.levels.len()
}

#[allow(dead_code)]
pub fn fat_to_params(map: &BodyFatMap) -> Vec<f32> {
    map.levels.clone()
}

#[allow(dead_code)]
pub fn smooth_fat_map(map: &mut BodyFatMap, iterations: usize) {
    for _ in 0..iterations {
        let prev = map.levels.clone();
        for i in 0..map.levels.len() {
            let left = if i > 0 { prev[i - 1] } else { prev[i] };
            let right = if i + 1 < prev.len() { prev[i + 1] } else { prev[i] };
            map.levels[i] = (left + prev[i] + right) / 3.0;
        }
    }
}

#[allow(dead_code)]
pub fn fat_map_to_json(map: &BodyFatMap) -> String {
    let vals: Vec<String> = map.levels.iter().map(|v| format!("{:.4}", v)).collect();
    format!("{{\"vertex_count\":{},\"levels\":[{}]}}", map.levels.len(), vals.join(","))
}

#[allow(dead_code)]
pub fn clear_fat_map(map: &mut BodyFatMap) {
    for v in map.levels.iter_mut() { *v = 0.0; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = new_body_fat_map(10);
        assert_eq!(fat_vertex_count(&m), 10);
    }

    #[test]
    fn test_set_get() {
        let mut m = new_body_fat_map(5);
        set_fat_level(&mut m, 2, 0.7);
        assert!((fat_level_at(&m, 2) - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        let mut m = new_body_fat_map(3);
        set_fat_level(&mut m, 0, 2.0);
        assert!((fat_level_at(&m, 0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_out_of_bounds() {
        let m = new_body_fat_map(3);
        assert!((fat_level_at(&m, 99)).abs() < 1e-6);
    }

    #[test]
    fn test_to_params() {
        let mut m = new_body_fat_map(2);
        set_fat_level(&mut m, 0, 0.5);
        set_fat_level(&mut m, 1, 0.3);
        let p = fat_to_params(&m);
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn test_smooth() {
        let mut m = new_body_fat_map(5);
        set_fat_level(&mut m, 2, 1.0);
        smooth_fat_map(&mut m, 1);
        assert!(fat_level_at(&m, 1) > 0.0);
        assert!(fat_level_at(&m, 3) > 0.0);
    }

    #[test]
    fn test_to_json() {
        let m = new_body_fat_map(2);
        let j = fat_map_to_json(&m);
        assert!(j.contains("vertex_count"));
    }

    #[test]
    fn test_clear() {
        let mut m = new_body_fat_map(3);
        set_fat_level(&mut m, 0, 1.0);
        clear_fat_map(&mut m);
        assert!((fat_level_at(&m, 0)).abs() < 1e-6);
    }

    #[test]
    fn test_set_out_of_bounds_noop() {
        let mut m = new_body_fat_map(2);
        set_fat_level(&mut m, 99, 1.0);
        assert_eq!(fat_vertex_count(&m), 2);
    }

    #[test]
    fn test_empty_map() {
        let m = new_body_fat_map(0);
        assert_eq!(fat_vertex_count(&m), 0);
    }
}
