#![allow(dead_code)]
//! Morph region mask: per-region weight masks for selective morph application.

use std::collections::HashMap;

/// A mask assigning weights to named regions.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MorphRegionMask {
    regions: HashMap<String, f32>,
}

/// Create a new empty region mask.
#[allow(dead_code)]
pub fn new_morph_region_mask() -> MorphRegionMask {
    MorphRegionMask {
        regions: HashMap::new(),
    }
}

/// Set the weight for a named region.
#[allow(dead_code)]
pub fn set_region_weight_mrm(mask: &mut MorphRegionMask, region: &str, weight: f32) {
    mask.regions.insert(region.to_string(), weight.clamp(0.0, 1.0));
}

/// Get the weight for a named region.
#[allow(dead_code)]
pub fn get_region_weight_mrm(mask: &MorphRegionMask, region: &str) -> f32 {
    mask.regions.get(region).copied().unwrap_or(0.0)
}

/// Return the number of regions.
#[allow(dead_code)]
pub fn region_count_mrm(mask: &MorphRegionMask) -> usize {
    mask.regions.len()
}

/// Apply the mask to a weight: `weight * region_weight`.
#[allow(dead_code)]
pub fn apply_region_mask(mask: &MorphRegionMask, region: &str, weight: f32) -> f32 {
    weight * get_region_weight_mrm(mask, region)
}

/// Invert the mask (each weight becomes `1.0 - weight`).
#[allow(dead_code)]
pub fn invert_region_mask(mask: &mut MorphRegionMask) {
    for v in mask.regions.values_mut() {
        *v = 1.0 - *v;
    }
}

/// Serialize to JSON-like string.
#[allow(dead_code)]
pub fn mask_to_json_mrm(mask: &MorphRegionMask) -> String {
    let entries: Vec<String> = mask
        .regions
        .iter()
        .map(|(k, v)| format!("\"{}\":{}", k, v))
        .collect();
    format!("{{\"regions\":{{{}}}}}", entries.join(","))
}

/// Clear all regions.
#[allow(dead_code)]
pub fn clear_region_mask(mask: &mut MorphRegionMask) {
    mask.regions.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_mask() {
        let m = new_morph_region_mask();
        assert_eq!(region_count_mrm(&m), 0);
    }

    #[test]
    fn test_set_get_region() {
        let mut m = new_morph_region_mask();
        set_region_weight_mrm(&mut m, "face", 0.8);
        assert!((get_region_weight_mrm(&m, "face") - 0.8).abs() < 1e-6);
    }

    #[test]
    fn test_missing_region() {
        let m = new_morph_region_mask();
        assert!((get_region_weight_mrm(&m, "nope") - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_clamp() {
        let mut m = new_morph_region_mask();
        set_region_weight_mrm(&mut m, "a", 2.0);
        assert!((get_region_weight_mrm(&m, "a") - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_mask() {
        let mut m = new_morph_region_mask();
        set_region_weight_mrm(&mut m, "body", 0.5);
        assert!((apply_region_mask(&m, "body", 1.0) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_invert() {
        let mut m = new_morph_region_mask();
        set_region_weight_mrm(&mut m, "face", 0.3);
        invert_region_mask(&mut m);
        assert!((get_region_weight_mrm(&m, "face") - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_to_json() {
        let m = new_morph_region_mask();
        let json = mask_to_json_mrm(&m);
        assert!(json.contains("\"regions\""));
    }

    #[test]
    fn test_clear() {
        let mut m = new_morph_region_mask();
        set_region_weight_mrm(&mut m, "a", 0.5);
        clear_region_mask(&mut m);
        assert_eq!(region_count_mrm(&m), 0);
    }

    #[test]
    fn test_region_count() {
        let mut m = new_morph_region_mask();
        set_region_weight_mrm(&mut m, "a", 0.5);
        set_region_weight_mrm(&mut m, "b", 0.5);
        assert_eq!(region_count_mrm(&m), 2);
    }

    #[test]
    fn test_apply_mask_missing() {
        let m = new_morph_region_mask();
        assert!((apply_region_mask(&m, "x", 1.0) - 0.0).abs() < 1e-6);
    }
}
