//! Wrinkle map generation based on mesh deformation.

#[allow(dead_code)]
pub struct WrinkleConfig {
    pub frequency: f32,
    pub amplitude: f32,
    pub threshold: f32,
    pub blend_sharpness: f32,
}

#[allow(dead_code)]
pub struct WrinkleRegion {
    pub name: String,
    pub vertex_indices: Vec<usize>,
    pub direction: [f32; 3],
    pub intensity: f32,
}

#[allow(dead_code)]
pub struct WrinkleMap {
    pub values: Vec<f32>,
    pub vertex_count: usize,
    pub config: WrinkleConfig,
}

#[allow(dead_code)]
pub fn default_wrinkle_config() -> WrinkleConfig {
    WrinkleConfig {
        frequency: 1.0,
        amplitude: 1.0,
        threshold: 0.01,
        blend_sharpness: 2.0,
    }
}

#[allow(dead_code)]
pub fn new_wrinkle_map(vertex_count: usize, cfg: WrinkleConfig) -> WrinkleMap {
    WrinkleMap {
        values: vec![0.0; vertex_count],
        vertex_count,
        config: cfg,
    }
}

#[allow(dead_code)]
pub fn compute_wrinkle_from_deformation(
    original: &[[f32; 3]],
    deformed: &[[f32; 3]],
    cfg: &WrinkleConfig,
) -> WrinkleMap {
    let n = original.len().min(deformed.len());
    let mut values = vec![0.0f32; n];
    for (i, (orig, def)) in original.iter().zip(deformed.iter()).enumerate() {
        let dx = def[0] - orig[0];
        let dy = def[1] - orig[1];
        let dz = def[2] - orig[2];
        let mag = (dx * dx + dy * dy + dz * dz).sqrt();
        if mag > cfg.threshold {
            values[i] = (mag * cfg.amplitude).min(1.0);
        }
    }
    WrinkleMap {
        values,
        vertex_count: n,
        config: WrinkleConfig {
            frequency: cfg.frequency,
            amplitude: cfg.amplitude,
            threshold: cfg.threshold,
            blend_sharpness: cfg.blend_sharpness,
        },
    }
}

#[allow(dead_code)]
pub fn add_procedural_wrinkles(
    map: &mut WrinkleMap,
    region: &WrinkleRegion,
    positions: &[[f32; 3]],
) {
    let dir = region.direction;
    let dir_len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
    let dir_norm = if dir_len > 1e-6 {
        [dir[0] / dir_len, dir[1] / dir_len, dir[2] / dir_len]
    } else {
        [1.0, 0.0, 0.0]
    };

    for &idx in &region.vertex_indices {
        if idx >= map.vertex_count || idx >= positions.len() {
            continue;
        }
        let pos = positions[idx];
        let proj = pos[0] * dir_norm[0] + pos[1] * dir_norm[1] + pos[2] * dir_norm[2];
        let wave = ((proj * map.config.frequency * std::f32::consts::TAU).sin() * 0.5 + 0.5)
            * region.intensity;
        map.values[idx] = (map.values[idx] + wave).min(1.0);
    }
}

#[allow(dead_code)]
pub fn smooth_wrinkle_map(map: &mut WrinkleMap, adjacency: &[Vec<usize>], iterations: u32) {
    for _ in 0..iterations {
        let old = map.values.clone();
        for (i, neighbors) in adjacency.iter().enumerate() {
            if i >= map.vertex_count {
                continue;
            }
            if neighbors.is_empty() {
                continue;
            }
            let sum: f32 = neighbors
                .iter()
                .filter(|&&n| n < map.vertex_count)
                .map(|&n| old[n])
                .sum();
            let valid_count = neighbors.iter().filter(|&&n| n < map.vertex_count).count();
            if valid_count > 0 {
                map.values[i] = (old[i] + sum / valid_count as f32) * 0.5;
            }
        }
    }
}

#[allow(dead_code)]
pub fn wrinkle_map_min(map: &WrinkleMap) -> f32 {
    map.values.iter().cloned().fold(f32::INFINITY, f32::min)
}

#[allow(dead_code)]
pub fn wrinkle_map_max(map: &WrinkleMap) -> f32 {
    map.values.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

#[allow(dead_code)]
pub fn normalize_wrinkle_map(map: &mut WrinkleMap) {
    let mn = wrinkle_map_min(map);
    let mx = wrinkle_map_max(map);
    let range = mx - mn;
    if range > 1e-6 {
        for v in &mut map.values {
            *v = (*v - mn) / range;
        }
    }
}

#[allow(dead_code)]
pub fn blend_wrinkle_maps(a: &WrinkleMap, b: &WrinkleMap, t: f32) -> WrinkleMap {
    let t = t.clamp(0.0, 1.0);
    let n = a.vertex_count.min(b.vertex_count);
    let values: Vec<f32> = a.values[..n]
        .iter()
        .zip(b.values[..n].iter())
        .map(|(av, bv)| av * (1.0 - t) + bv * t)
        .collect();
    WrinkleMap {
        vertex_count: n,
        values,
        config: WrinkleConfig {
            frequency: a.config.frequency,
            amplitude: a.config.amplitude,
            threshold: a.config.threshold,
            blend_sharpness: a.config.blend_sharpness,
        },
    }
}

#[allow(dead_code)]
pub fn wrinkle_to_normal_delta(
    map: &WrinkleMap,
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
) -> Vec<[f32; 3]> {
    let n = map.vertex_count.min(positions.len()).min(normals.len());
    (0..n)
        .map(|i| {
            let w = map.values[i] * map.config.amplitude;
            let nrm = normals[i];
            [nrm[0] * w, nrm[1] * w, nrm[2] * w]
        })
        .collect()
}

#[allow(dead_code)]
pub fn threshold_wrinkle_map(map: &WrinkleMap, threshold: f32) -> Vec<bool> {
    map.values.iter().map(|&v| v >= threshold).collect()
}

#[allow(dead_code)]
pub fn wrinkle_region_average(map: &WrinkleMap, indices: &[usize]) -> f32 {
    if indices.is_empty() {
        return 0.0;
    }
    let valid: Vec<f32> = indices
        .iter()
        .filter(|&&i| i < map.vertex_count)
        .map(|&i| map.values[i])
        .collect();
    if valid.is_empty() {
        return 0.0;
    }
    valid.iter().sum::<f32>() / valid.len() as f32
}

#[allow(dead_code)]
pub fn apply_wrinkle_weight(map: &mut WrinkleMap, weight: f32) {
    for v in &mut map.values {
        *v = (*v * weight).clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wrinkle_map() {
        let cfg = default_wrinkle_config();
        let map = new_wrinkle_map(10, cfg);
        assert_eq!(map.vertex_count, 10);
        assert_eq!(map.values.len(), 10);
        assert!(map.values.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_default_wrinkle_config() {
        let cfg = default_wrinkle_config();
        assert!(cfg.frequency > 0.0);
        assert!(cfg.amplitude > 0.0);
        assert!(cfg.threshold >= 0.0);
        assert!(cfg.blend_sharpness > 0.0);
    }

    #[test]
    fn test_compute_wrinkle_from_deformation_no_deform() {
        let positions = vec![[0.0f32; 3]; 5];
        let cfg = default_wrinkle_config();
        let map = compute_wrinkle_from_deformation(&positions, &positions, &cfg);
        assert_eq!(map.vertex_count, 5);
        assert!(map.values.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_compute_wrinkle_from_deformation_with_deform() {
        let original = vec![[0.0f32; 3]; 3];
        let deformed = vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.0]];
        let cfg = default_wrinkle_config();
        let map = compute_wrinkle_from_deformation(&original, &deformed, &cfg);
        assert!(map.values[0] > 0.0);
        assert_eq!(map.values[1], 0.0);
        assert!(map.values[2] > 0.0);
    }

    #[test]
    fn test_normalize_flat() {
        let cfg = default_wrinkle_config();
        let mut map = new_wrinkle_map(3, cfg);
        map.values = vec![0.2, 0.6, 1.0];
        normalize_wrinkle_map(&mut map);
        assert!((map.values[0]).abs() < 1e-5);
        assert!((map.values[2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_normalize_uniform() {
        let cfg = default_wrinkle_config();
        let mut map = new_wrinkle_map(3, cfg);
        map.values = vec![0.5, 0.5, 0.5];
        // Range = 0, should not change values
        normalize_wrinkle_map(&mut map);
        assert!(map.values.iter().all(|&v| (v - 0.5).abs() < 1e-5));
    }

    #[test]
    fn test_blend_wrinkle_maps() {
        let cfg_a = default_wrinkle_config();
        let cfg_b = default_wrinkle_config();
        let mut a = new_wrinkle_map(4, cfg_a);
        let mut b = new_wrinkle_map(4, cfg_b);
        a.values = vec![0.0; 4];
        b.values = vec![1.0; 4];
        let blended = blend_wrinkle_maps(&a, &b, 0.5);
        assert!(blended.values.iter().all(|&v| (v - 0.5).abs() < 1e-5));
    }

    #[test]
    fn test_blend_wrinkle_maps_clamp() {
        let a = new_wrinkle_map(2, default_wrinkle_config());
        let b = new_wrinkle_map(2, default_wrinkle_config());
        let blended = blend_wrinkle_maps(&a, &b, 2.0);
        assert_eq!(blended.vertex_count, 2);
    }

    #[test]
    fn test_threshold_wrinkle_map() {
        let cfg = default_wrinkle_config();
        let mut map = new_wrinkle_map(4, cfg);
        map.values = vec![0.1, 0.5, 0.3, 0.8];
        let mask = threshold_wrinkle_map(&map, 0.4);
        assert_eq!(mask, vec![false, true, false, true]);
    }

    #[test]
    fn test_smooth_wrinkle_map_no_panic() {
        let cfg = default_wrinkle_config();
        let mut map = new_wrinkle_map(3, cfg);
        map.values = vec![1.0, 0.0, 0.5];
        let adjacency = vec![vec![1usize], vec![0usize, 2usize], vec![1usize]];
        smooth_wrinkle_map(&mut map, &adjacency, 2);
        assert_eq!(map.values.len(), 3);
    }

    #[test]
    fn test_wrinkle_region_average() {
        let cfg = default_wrinkle_config();
        let mut map = new_wrinkle_map(5, cfg);
        map.values = vec![0.2, 0.4, 0.6, 0.8, 1.0];
        let avg = wrinkle_region_average(&map, &[0, 1, 2]);
        assert!((avg - 0.4).abs() < 1e-5);
    }

    #[test]
    fn test_wrinkle_region_average_empty() {
        let map = new_wrinkle_map(3, default_wrinkle_config());
        let avg = wrinkle_region_average(&map, &[]);
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn test_wrinkle_map_min_max() {
        let cfg = default_wrinkle_config();
        let mut map = new_wrinkle_map(4, cfg);
        map.values = vec![0.1, 0.5, 0.9, 0.3];
        assert!((wrinkle_map_min(&map) - 0.1).abs() < 1e-5);
        assert!((wrinkle_map_max(&map) - 0.9).abs() < 1e-5);
    }

    #[test]
    fn test_apply_wrinkle_weight() {
        let cfg = default_wrinkle_config();
        let mut map = new_wrinkle_map(3, cfg);
        map.values = vec![0.5, 1.0, 0.25];
        apply_wrinkle_weight(&mut map, 0.5);
        assert!((map.values[0] - 0.25).abs() < 1e-5);
        assert!((map.values[1] - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_wrinkle_to_normal_delta() {
        let cfg = default_wrinkle_config();
        let mut map = new_wrinkle_map(2, cfg);
        map.values = vec![0.5, 1.0];
        let positions = vec![[0.0f32; 3]; 2];
        let normals = vec![[0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let deltas = wrinkle_to_normal_delta(&map, &positions, &normals);
        assert_eq!(deltas.len(), 2);
        assert!((deltas[0][1] - 0.5).abs() < 1e-5);
        assert!((deltas[1][2] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_add_procedural_wrinkles() {
        let cfg = default_wrinkle_config();
        let mut map = new_wrinkle_map(3, cfg);
        let region = WrinkleRegion {
            name: "test".to_string(),
            vertex_indices: vec![0, 1, 2],
            direction: [1.0, 0.0, 0.0],
            intensity: 0.5,
        };
        let positions = vec![[0.0f32; 3]; 3];
        add_procedural_wrinkles(&mut map, &region, &positions);
        assert_eq!(map.values.len(), 3);
    }
}
