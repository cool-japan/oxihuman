#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export baked texture/lighting data manifest.

#[allow(dead_code)]
pub struct BakeItem {
    pub name: String,
    pub bake_type: String,
    pub resolution: u32,
    pub uv_layer: String,
    pub margin: u32,
}

#[allow(dead_code)]
pub struct BakeExport {
    pub items: Vec<BakeItem>,
}

#[allow(dead_code)]
pub fn new_bake_export() -> BakeExport {
    BakeExport { items: vec![] }
}

#[allow(dead_code)]
pub fn add_bake_item(exp: &mut BakeExport, name: &str, type_: &str, res: u32, uv: &str, margin: u32) {
    exp.items.push(BakeItem {
        name: name.to_string(),
        bake_type: type_.to_string(),
        resolution: res,
        uv_layer: uv.to_string(),
        margin,
    });
}

#[allow(dead_code)]
pub fn export_bake_to_json(exp: &BakeExport) -> String {
    let items_str: Vec<String> = exp.items.iter().map(|i| {
        format!(
            r#"{{"name":"{}","type":"{}","resolution":{},"uv_layer":"{}","margin":{}}}"#,
            i.name, i.bake_type, i.resolution, i.uv_layer, i.margin
        )
    }).collect();
    format!(r#"{{"bake_items":[{}]}}"#, items_str.join(","))
}

#[allow(dead_code)]
pub fn bake_count(exp: &BakeExport) -> usize {
    exp.items.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_bake_export();
        assert_eq!(bake_count(&e), 0);
    }

    #[test]
    fn add_item_increments_count() {
        let mut e = new_bake_export();
        add_bake_item(&mut e, "diffuse", "DIFFUSE", 1024, "UVMap", 16);
        assert_eq!(bake_count(&e), 1);
    }

    #[test]
    fn item_name_stored() {
        let mut e = new_bake_export();
        add_bake_item(&mut e, "ao_map", "AO", 512, "UVMap", 4);
        assert_eq!(e.items[0].name, "ao_map");
    }

    #[test]
    fn item_resolution_stored() {
        let mut e = new_bake_export();
        add_bake_item(&mut e, "normal", "NORMAL", 2048, "UV1", 8);
        assert_eq!(e.items[0].resolution, 2048);
    }

    #[test]
    fn item_margin_stored() {
        let mut e = new_bake_export();
        add_bake_item(&mut e, "shadow", "SHADOW", 256, "UV2", 32);
        assert_eq!(e.items[0].margin, 32);
    }

    #[test]
    fn export_json_contains_name() {
        let mut e = new_bake_export();
        add_bake_item(&mut e, "mymap", "COMBINED", 512, "UV", 2);
        let json = export_bake_to_json(&e);
        assert!(json.contains("mymap"));
    }

    #[test]
    fn export_json_empty() {
        let e = new_bake_export();
        let json = export_bake_to_json(&e);
        assert!(json.contains("bake_items"));
    }

    #[test]
    fn multiple_items() {
        let mut e = new_bake_export();
        add_bake_item(&mut e, "a", "DIFFUSE", 512, "UV", 4);
        add_bake_item(&mut e, "b", "AO", 256, "UV", 2);
        assert_eq!(bake_count(&e), 2);
    }

    #[test]
    fn uv_layer_stored() {
        let mut e = new_bake_export();
        add_bake_item(&mut e, "x", "NORMAL", 1024, "SecondUV", 8);
        assert_eq!(e.items[0].uv_layer, "SecondUV");
    }
}
