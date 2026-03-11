#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export Unreal Engine asset metadata.

#[allow(dead_code)]
pub struct UnrealAsset {
    pub name: String,
    pub asset_type: String,
    pub path: String,
    pub lod_count: u32,
}

#[allow(dead_code)]
pub struct UnrealExport {
    pub assets: Vec<UnrealAsset>,
}

#[allow(dead_code)]
pub fn new_unreal_export() -> UnrealExport {
    UnrealExport { assets: Vec::new() }
}

#[allow(dead_code)]
pub fn add_asset(exp: &mut UnrealExport, name: &str, type_: &str, path: &str, lods: u32) {
    exp.assets.push(UnrealAsset {
        name: name.to_string(),
        asset_type: type_.to_string(),
        path: path.to_string(),
        lod_count: lods,
    });
}

#[allow(dead_code)]
pub fn asset_count(exp: &UnrealExport) -> usize {
    exp.assets.len()
}

#[allow(dead_code)]
pub fn export_unreal_to_json(exp: &UnrealExport) -> String {
    let mut s = "{\"assets\":[".to_string();
    for (i, a) in exp.assets.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&format!(
            "{{\"name\":\"{}\",\"type\":\"{}\",\"path\":\"{}\",\"lods\":{}}}",
            a.name, a.asset_type, a.path, a.lod_count
        ));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_unreal_export();
        assert!(e.assets.is_empty());
    }

    #[test]
    fn add_asset_stored() {
        let mut e = new_unreal_export();
        add_asset(&mut e, "PlayerMesh", "StaticMesh", "/Game/Meshes/Player", 3);
        assert_eq!(e.assets.len(), 1);
    }

    #[test]
    fn asset_count_correct() {
        let mut e = new_unreal_export();
        add_asset(&mut e, "A", "SM", "/A", 1);
        add_asset(&mut e, "B", "SK", "/B", 2);
        assert_eq!(asset_count(&e), 2);
    }

    #[test]
    fn asset_name_stored() {
        let mut e = new_unreal_export();
        add_asset(&mut e, "HeroChar", "SkeletalMesh", "/Hero", 4);
        assert_eq!(e.assets[0].name, "HeroChar");
    }

    #[test]
    fn asset_type_stored() {
        let mut e = new_unreal_export();
        add_asset(&mut e, "A", "Blueprint", "/A", 0);
        assert_eq!(e.assets[0].asset_type, "Blueprint");
    }

    #[test]
    fn asset_path_stored() {
        let mut e = new_unreal_export();
        add_asset(&mut e, "A", "T", "/Game/Textures/Diffuse", 0);
        assert_eq!(e.assets[0].path, "/Game/Textures/Diffuse");
    }

    #[test]
    fn asset_lod_count_stored() {
        let mut e = new_unreal_export();
        add_asset(&mut e, "A", "T", "/A", 5);
        assert_eq!(e.assets[0].lod_count, 5);
    }

    #[test]
    fn export_json_contains_name() {
        let mut e = new_unreal_export();
        add_asset(&mut e, "UnrealAssetName", "SM", "/X", 1);
        let j = export_unreal_to_json(&e);
        assert!(j.contains("UnrealAssetName"));
    }

    #[test]
    fn export_json_contains_assets_key() {
        let e = new_unreal_export();
        let j = export_unreal_to_json(&e);
        assert!(j.contains("assets"));
    }
}
