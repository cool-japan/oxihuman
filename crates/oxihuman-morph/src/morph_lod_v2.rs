#![allow(dead_code)]

//! LOD-aware morph blend with distance-based weight reduction.

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphLodV2 {
    pub base_weight: f32,
    pub lod_distances: Vec<f32>,
    pub lod_scales: Vec<f32>,
    pub current_lod: usize,
}

#[allow(dead_code)]
pub fn new_morph_lod_v2(base_weight: f32) -> MorphLodV2 {
    MorphLodV2 {
        base_weight,
        lod_distances: vec![10.0, 25.0, 50.0, 100.0],
        lod_scales: vec![1.0, 0.75, 0.5, 0.25],
        current_lod: 0,
    }
}

#[allow(dead_code)]
pub fn mlv2_effective_weight(lod: &MorphLodV2) -> f32 {
    let scale = lod
        .lod_scales
        .get(lod.current_lod)
        .copied()
        .unwrap_or(1.0);
    (lod.base_weight * scale).clamp(0.0, 1.0)
}

#[allow(dead_code)]
pub fn mlv2_set_distance(lod: &mut MorphLodV2, distance: f32) {
    let idx = lod
        .lod_distances
        .iter()
        .position(|&d| distance < d)
        .unwrap_or(lod.lod_distances.len());
    lod.current_lod = idx.min(lod.lod_scales.len().saturating_sub(1));
}

#[allow(dead_code)]
pub fn mlv2_add_level(lod: &mut MorphLodV2, distance: f32, scale: f32) {
    lod.lod_distances.push(distance);
    lod.lod_scales.push(scale.clamp(0.0, 1.0));
}

#[allow(dead_code)]
pub fn mlv2_reset(lod: &mut MorphLodV2) {
    lod.current_lod = 0;
}

#[allow(dead_code)]
pub fn mlv2_lod_count(lod: &MorphLodV2) -> usize {
    lod.lod_distances.len()
}

#[allow(dead_code)]
pub fn mlv2_is_full_detail(lod: &MorphLodV2) -> bool {
    lod.current_lod == 0
}

#[allow(dead_code)]
pub fn mlv2_to_json(lod: &MorphLodV2) -> String {
    format!(
        "{{\"base_weight\":{},\"current_lod\":{},\"lod_count\":{}}}",
        lod.base_weight,
        lod.current_lod,
        lod.lod_distances.len()
    )
}

#[allow(dead_code)]
pub fn mlv2_scale_at(lod: &MorphLodV2, level: usize) -> f32 {
    lod.lod_scales.get(level).copied().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_morph_lod_v2() {
        let lod = new_morph_lod_v2(0.8);
        assert!((lod.base_weight - 0.8).abs() < 1e-6);
        assert_eq!(lod.current_lod, 0);
    }

    #[test]
    fn test_effective_weight_full_detail() {
        let lod = new_morph_lod_v2(1.0);
        let w = mlv2_effective_weight(&lod);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_distance_lod_change() {
        let mut lod = new_morph_lod_v2(1.0);
        mlv2_set_distance(&mut lod, 30.0);
        assert!(lod.current_lod > 0);
    }

    #[test]
    fn test_effective_weight_reduced_at_distance() {
        let mut lod = new_morph_lod_v2(1.0);
        mlv2_set_distance(&mut lod, 30.0);
        let w = mlv2_effective_weight(&lod);
        assert!(w < 1.0);
    }

    #[test]
    fn test_add_level() {
        let mut lod = new_morph_lod_v2(1.0);
        let before = mlv2_lod_count(&lod);
        mlv2_add_level(&mut lod, 200.0, 0.1);
        assert_eq!(mlv2_lod_count(&lod), before + 1);
    }

    #[test]
    fn test_reset() {
        let mut lod = new_morph_lod_v2(1.0);
        mlv2_set_distance(&mut lod, 60.0);
        mlv2_reset(&mut lod);
        assert_eq!(lod.current_lod, 0);
    }

    #[test]
    fn test_is_full_detail() {
        let lod = new_morph_lod_v2(1.0);
        assert!(mlv2_is_full_detail(&lod));
    }

    #[test]
    fn test_to_json() {
        let lod = new_morph_lod_v2(0.5);
        let json = mlv2_to_json(&lod);
        assert!(json.contains("base_weight"));
    }

    #[test]
    fn test_scale_at() {
        let lod = new_morph_lod_v2(1.0);
        let s = mlv2_scale_at(&lod, 0);
        assert!((s - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_scale_at_out_of_bounds() {
        let lod = new_morph_lod_v2(1.0);
        let s = mlv2_scale_at(&lod, 100);
        assert!((s - 0.0).abs() < 1e-6);
    }
}
