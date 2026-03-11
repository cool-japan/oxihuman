#![allow(dead_code)]
// Copyright (C) 2026 COOLJAPAN OU (Team KitaSan)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Export Unity asset metadata.

#[allow(dead_code)]
pub struct UnityAsset {
    pub name: String,
    pub guid: String,
    pub asset_type: String,
    pub import_settings: Vec<(String, String)>,
}

#[allow(dead_code)]
pub struct UnityExport {
    pub assets: Vec<UnityAsset>,
}

#[allow(dead_code)]
pub fn new_unity_export() -> UnityExport {
    UnityExport { assets: Vec::new() }
}

#[allow(dead_code)]
pub fn add_asset(exp: &mut UnityExport, name: &str, guid: &str, type_: &str) {
    exp.assets.push(UnityAsset {
        name: name.to_string(),
        guid: guid.to_string(),
        asset_type: type_.to_string(),
        import_settings: Vec::new(),
    });
}

#[allow(dead_code)]
pub fn add_import_setting(asset: &mut UnityAsset, key: &str, val: &str) {
    asset.import_settings.push((key.to_string(), val.to_string()));
}

#[allow(dead_code)]
pub fn asset_count(exp: &UnityExport) -> usize {
    exp.assets.len()
}

#[allow(dead_code)]
pub fn export_unity_to_json(exp: &UnityExport) -> String {
    let mut s = "{\"assets\":[".to_string();
    for (i, a) in exp.assets.iter().enumerate() {
        if i > 0 { s.push(','); }
        s.push_str(&format!("{{\"name\":\"{}\",\"guid\":\"{}\",\"type\":\"{}\",\"settings\":{}}}", a.name, a.guid, a.asset_type, a.import_settings.len()));
    }
    s.push_str("]}");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_export_empty() {
        let e = new_unity_export();
        assert!(e.assets.is_empty());
    }

    #[test]
    fn add_asset_stored() {
        let mut e = new_unity_export();
        add_asset(&mut e, "Player", "abc123", "Mesh");
        assert_eq!(e.assets.len(), 1);
    }

    #[test]
    fn asset_count_correct() {
        let mut e = new_unity_export();
        add_asset(&mut e, "A", "g1", "Mesh");
        add_asset(&mut e, "B", "g2", "Material");
        assert_eq!(asset_count(&e), 2);
    }

    #[test]
    fn asset_name_stored() {
        let mut e = new_unity_export();
        add_asset(&mut e, "Sword", "guid1", "Mesh");
        assert_eq!(e.assets[0].name, "Sword");
    }

    #[test]
    fn asset_guid_stored() {
        let mut e = new_unity_export();
        add_asset(&mut e, "A", "deadbeef", "Texture");
        assert_eq!(e.assets[0].guid, "deadbeef");
    }

    #[test]
    fn add_import_setting_stored() {
        let mut e = new_unity_export();
        add_asset(&mut e, "A", "g", "Mesh");
        add_import_setting(&mut e.assets[0], "compression", "lz4");
        assert_eq!(e.assets[0].import_settings.len(), 1);
    }

    #[test]
    fn import_setting_key_value_correct() {
        let mut e = new_unity_export();
        add_asset(&mut e, "A", "g", "Mesh");
        add_import_setting(&mut e.assets[0], "sRGB", "true");
        assert_eq!(e.assets[0].import_settings[0].0, "sRGB");
        assert_eq!(e.assets[0].import_settings[0].1, "true");
    }

    #[test]
    fn export_json_contains_name() {
        let mut e = new_unity_export();
        add_asset(&mut e, "UnityCharacter", "g", "SkinnedMesh");
        let j = export_unity_to_json(&e);
        assert!(j.contains("UnityCharacter"));
    }

    #[test]
    fn export_json_contains_guid() {
        let mut e = new_unity_export();
        add_asset(&mut e, "A", "unique-guid-123", "M");
        let j = export_unity_to_json(&e);
        assert!(j.contains("unique-guid-123"));
    }
}
