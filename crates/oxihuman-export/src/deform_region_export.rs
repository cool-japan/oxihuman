// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0
#![allow(dead_code)]

/// Export deformation region data (vertex groups with falloff weights).
#[allow(dead_code)]
pub struct DeformRegion {
    pub name: String,
    pub vertex_indices: Vec<u32>,
    pub weights: Vec<f32>,
    pub falloff_type: FalloffType,
}

#[allow(dead_code)]
pub enum FalloffType {
    Linear,
    Smooth,
    Constant,
    Sphere,
}

#[allow(dead_code)]
pub struct DeformRegionExport {
    pub regions: Vec<DeformRegion>,
}

#[allow(dead_code)]
pub fn new_deform_region_export() -> DeformRegionExport {
    DeformRegionExport { regions: vec![] }
}

#[allow(dead_code)]
pub fn add_region(export: &mut DeformRegionExport, region: DeformRegion) {
    export.regions.push(region);
}

#[allow(dead_code)]
pub fn region_count(export: &DeformRegionExport) -> usize {
    export.regions.len()
}

#[allow(dead_code)]
pub fn total_vertex_count(export: &DeformRegionExport) -> usize {
    export.regions.iter().map(|r| r.vertex_indices.len()).sum()
}

#[allow(dead_code)]
pub fn find_region<'a>(export: &'a DeformRegionExport, name: &str) -> Option<&'a DeformRegion> {
    export.regions.iter().find(|r| r.name == name)
}

#[allow(dead_code)]
pub fn region_avg_weight(region: &DeformRegion) -> f32 {
    if region.weights.is_empty() {
        return 0.0;
    }
    region.weights.iter().sum::<f32>() / region.weights.len() as f32
}

#[allow(dead_code)]
pub fn region_max_weight(region: &DeformRegion) -> f32 {
    region.weights.iter().cloned().fold(0.0f32, f32::max)
}

#[allow(dead_code)]
pub fn validate_region(region: &DeformRegion) -> bool {
    region.vertex_indices.len() == region.weights.len()
        && region.weights.iter().all(|&w| (0.0..=1.0).contains(&w))
        && !region.name.is_empty()
}

#[allow(dead_code)]
pub fn normalize_region_weights(region: &mut DeformRegion) {
    let max_w = region.weights.iter().cloned().fold(0.0f32, f32::max);
    if max_w > 1e-10 {
        for w in &mut region.weights {
            *w /= max_w;
        }
    }
}

#[allow(dead_code)]
pub fn region_to_json(region: &DeformRegion) -> String {
    format!(
        "{{\"name\":\"{}\",\"vertex_count\":{},\"avg_weight\":{}}}",
        region.name,
        region.vertex_indices.len(),
        region_avg_weight(region)
    )
}

#[allow(dead_code)]
pub fn deform_region_export_to_json(export: &DeformRegionExport) -> String {
    format!(
        "{{\"region_count\":{},\"total_vertices\":{}}}",
        export.regions.len(),
        total_vertex_count(export)
    )
}

#[allow(dead_code)]
pub fn falloff_name(ft: &FalloffType) -> &'static str {
    match ft {
        FalloffType::Linear => "linear",
        FalloffType::Smooth => "smooth",
        FalloffType::Constant => "constant",
        FalloffType::Sphere => "sphere",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hand_region() -> DeformRegion {
        DeformRegion {
            name: "hand".to_string(),
            vertex_indices: vec![0, 1, 2, 3],
            weights: vec![1.0, 0.8, 0.5, 0.2],
            falloff_type: FalloffType::Smooth,
        }
    }

    #[test]
    fn test_add_region() {
        let mut e = new_deform_region_export();
        add_region(&mut e, hand_region());
        assert_eq!(region_count(&e), 1);
    }

    #[test]
    fn test_total_vertex_count() {
        let mut e = new_deform_region_export();
        add_region(&mut e, hand_region());
        assert_eq!(total_vertex_count(&e), 4);
    }

    #[test]
    fn test_find_region() {
        let mut e = new_deform_region_export();
        add_region(&mut e, hand_region());
        assert!(find_region(&e, "hand").is_some());
    }

    #[test]
    fn test_region_avg_weight() {
        let r = hand_region();
        let avg = region_avg_weight(&r);
        assert!((avg - 0.625).abs() < 0.01);
    }

    #[test]
    fn test_region_max_weight() {
        let r = hand_region();
        assert!((region_max_weight(&r) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_validate_region() {
        let r = hand_region();
        assert!(validate_region(&r));
    }

    #[test]
    fn test_normalize_region_weights() {
        let mut r = DeformRegion {
            name: "test".to_string(),
            vertex_indices: vec![0, 1],
            weights: vec![0.5, 0.25],
            falloff_type: FalloffType::Linear,
        };
        normalize_region_weights(&mut r);
        assert!((r.weights[0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_to_json() {
        let r = hand_region();
        let j = region_to_json(&r);
        assert!(j.contains("hand"));
    }

    #[test]
    fn test_export_to_json() {
        let mut e = new_deform_region_export();
        add_region(&mut e, hand_region());
        let j = deform_region_export_to_json(&e);
        assert!(j.contains("region_count"));
    }

    #[test]
    fn test_falloff_name() {
        assert_eq!(falloff_name(&FalloffType::Smooth), "smooth");
        assert_eq!(falloff_name(&FalloffType::Sphere), "sphere");
    }
}
