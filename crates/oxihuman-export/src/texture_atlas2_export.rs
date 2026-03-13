// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: Apache-2.0

//! Texture atlas packing export (extended version).

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AtlasRegion2 {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextureAtlasExport2 {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub regions: Vec<AtlasRegion2>,
}

#[allow(dead_code)]
pub fn new_texture_atlas_export2(width: u32, height: u32) -> TextureAtlasExport2 {
    TextureAtlasExport2 { atlas_width: width, atlas_height: height, regions: Vec::new() }
}

#[allow(dead_code)]
pub fn ta2_add_region(exp: &mut TextureAtlasExport2, region: AtlasRegion2) {
    exp.regions.push(region);
}

#[allow(dead_code)]
pub fn ta2_get_region(exp: &TextureAtlasExport2, id: u32) -> Option<&AtlasRegion2> {
    exp.regions.iter().find(|r| r.id == id)
}

#[allow(dead_code)]
pub fn ta2_region_count(exp: &TextureAtlasExport2) -> usize {
    exp.regions.len()
}

#[allow(dead_code)]
pub fn ta2_utilization(exp: &TextureAtlasExport2) -> f32 {
    let total = (exp.atlas_width * exp.atlas_height) as f32;
    if total < 1.0 { return 0.0; }
    let used: u32 = exp.regions.iter().map(|r| r.width * r.height).sum();
    used as f32 / total
}

#[allow(dead_code)]
pub fn ta2_to_json(exp: &TextureAtlasExport2) -> String {
    format!(
        r#"{{"width":{},"height":{},"regions":{},"utilization":{:.4}}}"#,
        exp.atlas_width, exp.atlas_height, exp.regions.len(), ta2_utilization(exp)
    )
}

#[allow(dead_code)]
pub fn ta2_validate(exp: &TextureAtlasExport2) -> bool {
    exp.atlas_width > 0 && exp.atlas_height > 0
}

#[allow(dead_code)]
pub fn ta2_find_by_name<'a>(exp: &'a TextureAtlasExport2, name: &str) -> Option<&'a AtlasRegion2> {
    exp.regions.iter().find(|r| r.name == name)
}

#[allow(dead_code)]
pub fn ta2_remove_region(exp: &mut TextureAtlasExport2, id: u32) -> bool {
    let before = exp.regions.len();
    exp.regions.retain(|r| r.id != id);
    exp.regions.len() < before
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_region(id: u32, name: &str, w: u32, h: u32) -> AtlasRegion2 {
        AtlasRegion2 { id, x: 0, y: 0, width: w, height: h, name: name.to_string() }
    }

    #[test]
    fn new_atlas_empty() {
        let exp = new_texture_atlas_export2(512, 512);
        assert_eq!(ta2_region_count(&exp), 0);
    }

    #[test]
    fn add_region_increments() {
        let mut exp = new_texture_atlas_export2(512, 512);
        ta2_add_region(&mut exp, make_region(1, "face", 128, 128));
        assert_eq!(ta2_region_count(&exp), 1);
    }

    #[test]
    fn get_region_by_id() {
        let mut exp = new_texture_atlas_export2(512, 512);
        ta2_add_region(&mut exp, make_region(10, "body", 256, 256));
        assert!(ta2_get_region(&exp, 10).is_some());
        assert!(ta2_get_region(&exp, 99).is_none());
    }

    #[test]
    fn utilization_calculates() {
        let mut exp = new_texture_atlas_export2(100, 100);
        ta2_add_region(&mut exp, make_region(1, "r", 50, 50));
        assert!((ta2_utilization(&exp) - 0.25).abs() < 1e-4);
    }

    #[test]
    fn find_by_name() {
        let mut exp = new_texture_atlas_export2(512, 512);
        ta2_add_region(&mut exp, make_region(1, "diffuse", 128, 128));
        assert!(ta2_find_by_name(&exp, "diffuse").is_some());
        assert!(ta2_find_by_name(&exp, "missing").is_none());
    }

    #[test]
    fn remove_region() {
        let mut exp = new_texture_atlas_export2(512, 512);
        ta2_add_region(&mut exp, make_region(1, "r", 64, 64));
        let removed = ta2_remove_region(&mut exp, 1);
        assert!(removed);
        assert_eq!(ta2_region_count(&exp), 0);
    }

    #[test]
    fn validate_ok() {
        let exp = new_texture_atlas_export2(256, 256);
        assert!(ta2_validate(&exp));
    }

    #[test]
    fn to_json_has_fields() {
        let exp = new_texture_atlas_export2(512, 512);
        let json = ta2_to_json(&exp);
        assert!(json.contains("width"));
        assert!(json.contains("utilization"));
    }
}
